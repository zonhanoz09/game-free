use super::*;

pub fn to_clob_expr(s: &str) -> String {
    if s.is_empty() {
        return "EMPTY_CLOB()".to_string();
    }
    let escaped = s.replace('\'', "''");
    let mut chunks = Vec::new();
    let mut current = String::new();
    for ch in escaped.chars() {
        current.push(ch);
        if current.len() >= 2000 {
            chunks.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
        .into_iter()
        .map(|c| format!("TO_CLOB('{}')", c))
        .collect::<Vec<_>>()
        .join(" || ")
}

#[derive(Clone, Debug)]
pub struct OracleAdbClient {
    pub client: reqwest::Client,
    pub sql_url: String,
    pub auth_header: String,
}

impl OracleAdbClient {
    pub fn new_from_env() -> Option<Self> {
        let sql_url = std::env::var("ORACLE_ADB_URL").unwrap_or_else(|_| {
            "https://G7262C948FBC089-GAMEDB.adb.ap-singapore-1.oraclecloudapps.com/ords/admin/_/sql".to_string()
        });
        let user = std::env::var("ORACLE_ADB_USER").unwrap_or_else(|_| "ADMIN".to_string());
        let pass = std::env::var("ORACLE_ADB_PASSWORD")
            .unwrap_or_else(|_| "TacticalArenaDb2026#".to_string());

        let creds = format!("{}:{}", user, pass);
        let b64 = BASE64_STANDARD.encode(creds.as_bytes());
        let auth_header = format!("Basic {}", b64);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .build()
            .ok()?;

        Some(Self {
            client,
            sql_url,
            auth_header,
        })
    }

    pub async fn execute_sql(&self, sql: &str) -> Result<serde_json::Value, String> {
        let resp = self
            .client
            .post(&self.sql_url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "application/sql")
            .body(sql.to_string())
            .send()
            .await
            .map_err(|e| format!("HTTP request error: {}", e))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| format!("Read body error: {}", e))?;
        if !status.is_success() {
            return Err(format!("ADB error status {}: {}", status, body));
        }

        let json: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(json)
    }

    pub async fn load_all_users(&self) -> Result<Vec<User>, String> {
        let sql = "SELECT p.id, p.username, p.display_name, p.avatar_id, p.cardback_id, p.board_skin, p.player_level, p.current_exp, p.gold, p.gems, p.created_at, r.rank_tier, r.rank_division, r.rank_stars, r.mmr, r.total_matches, r.wins, r.win_streak, r.best_streak, u.avatar, u.password_hash, u.losses, u.cards_json, u.items_json, u.last_login FROM PLAYERS p LEFT JOIN PLAYER_RATINGS r ON p.id = r.player_id LEFT JOIN USERS u ON p.username = u.username";

        let val = self.execute_sql(sql).await?;
        let mut users = Vec::new();

        let cards_sql = "SELECT player_id, card_id, hero_class, card_name, quantity, is_foil, star_level, card_level, is_starter, hp_bonus, atk_bonus FROM player_cards";
        let decks_sql = "SELECT id, player_id, deck_name, hero_class, cardback_id, cards_data, is_valid FROM player_decks";

        let all_cards_val = self.execute_sql(cards_sql).await.ok();
        let all_decks_val = self.execute_sql(decks_sql).await.ok();

        let mut cards_by_player: HashMap<String, Vec<UserCard>> = HashMap::new();
        if let Some(c_items) = all_cards_val
            .as_ref()
            .and_then(|v| v["items"][0]["resultSet"]["items"].as_array())
        {
            for row in c_items {
                let pid = row
                    .get("player_id")
                    .or_else(|| row.get("PLAYER_ID"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let cid = row
                    .get("card_id")
                    .or_else(|| row.get("CARD_ID"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let hclass = row
                    .get("hero_class")
                    .or_else(|| row.get("HERO_CLASS"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Knight")
                    .to_string();
                let cname = row
                    .get("card_name")
                    .or_else(|| row.get("CARD_NAME"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Chiến Binh")
                    .to_string();
                let qty = row
                    .get("quantity")
                    .or_else(|| row.get("QUANTITY"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u32;
                let is_foil = row
                    .get("is_foil")
                    .or_else(|| row.get("IS_FOIL"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    == 1;
                let s_lvl = row
                    .get("star_level")
                    .or_else(|| row.get("STAR_LEVEL"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u32;
                let c_lvl = row
                    .get("card_level")
                    .or_else(|| row.get("CARD_LEVEL"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u32;
                let is_st = row
                    .get("is_starter")
                    .or_else(|| row.get("IS_STARTER"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    == 1;
                let hp_b = row
                    .get("hp_bonus")
                    .or_else(|| row.get("HP_BONUS"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;
                let atk_b = row
                    .get("atk_bonus")
                    .or_else(|| row.get("ATK_BONUS"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;

                cards_by_player.entry(pid).or_default().push(UserCard {
                    id: cid,
                    hero_class: hclass,
                    name: cname,
                    star_level: s_lvl,
                    level: c_lvl,
                    is_starter: is_st,
                    hp_bonus: hp_b,
                    atk_bonus: atk_b,
                    quantity: qty,
                    is_foil,
                });
            }
        }

        let mut decks_by_player: HashMap<String, Vec<PlayerDeck>> = HashMap::new();
        if let Some(d_items) = all_decks_val
            .as_ref()
            .and_then(|v| v["items"][0]["resultSet"]["items"].as_array())
        {
            for row in d_items {
                let id = row
                    .get("id")
                    .or_else(|| row.get("ID"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let pid = row
                    .get("player_id")
                    .or_else(|| row.get("PLAYER_ID"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let dname = row
                    .get("deck_name")
                    .or_else(|| row.get("DECK_NAME"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Bộ Bài")
                    .to_string();
                let hclass = row
                    .get("hero_class")
                    .or_else(|| row.get("HERO_CLASS"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Knight")
                    .to_string();
                let cback = row
                    .get("cardback_id")
                    .or_else(|| row.get("CARDBACK_ID"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("cb_classic")
                    .to_string();
                let is_valid = row
                    .get("is_valid")
                    .or_else(|| row.get("IS_VALID"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1)
                    == 1;
                let raw_data = row
                    .get("cards_data")
                    .or_else(|| row.get("CARDS_DATA"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("[]");
                let cards_data: Vec<DeckCardEntry> =
                    serde_json::from_str(raw_data).unwrap_or_default();

                decks_by_player
                    .entry(pid.clone())
                    .or_default()
                    .push(PlayerDeck {
                        id,
                        player_id: pid,
                        deck_name: dname,
                        hero_class: hclass,
                        cardback_id: cback,
                        cards_data,
                        is_valid,
                        validation_errors: vec![],
                        updated_at: chrono_now(),
                    });
            }
        }

        if let Some(items) = val["items"][0]["resultSet"]["items"].as_array() {
            for row in items {
                let get_s = |key1: &str, key2: &str| -> String {
                    row.get(key1)
                        .or_else(|| row.get(key2))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                };
                let get_u = |key1: &str, key2: &str, def: u64| -> u64 {
                    row.get(key1)
                        .or_else(|| row.get(key2))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(def)
                };
                let get_i = |key1: &str, key2: &str, def: i64| -> i64 {
                    row.get(key1)
                        .or_else(|| row.get(key2))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(def)
                };

                let username = get_s("username", "USERNAME");
                if username.is_empty() {
                    continue;
                }
                let id = get_s("id", "ID");
                let display_name = get_s("display_name", "DISPLAY_NAME");
                let avatar = get_s("avatar", "AVATAR");
                let avatar_id = get_s("avatar_id", "AVATAR_ID");
                let cardback_id = get_s("cardback_id", "CARDBACK_ID");
                let board_skin = get_s("board_skin", "BOARD_SKIN");
                let password_hash = get_s("password_hash", "PASSWORD_HASH");
                let level = get_u("player_level", "PLAYER_LEVEL", 1) as u32;
                let current_exp = get_u("current_exp", "CURRENT_EXP", 0);
                let elo = get_i("mmr", "MMR", 1000) as i32;
                let wins = get_u("wins", "WINS", 0) as u32;
                let losses = get_u("losses", "LOSSES", 0) as u32;
                let matches = get_u("total_matches", "TOTAL_MATCHES", 0) as u32;
                let gold = get_u("gold", "GOLD", 100) as u32;
                let gems = get_u("gems", "GEMS", 10) as u32;
                let rank_tier = get_s("rank_tier", "RANK_TIER");
                let rank_division = get_u("rank_division", "RANK_DIVISION", 3) as u8;
                let rank_stars = get_u("rank_stars", "RANK_STARS", 0) as u8;
                let mmr = get_i("mmr", "MMR", 1200) as i32;
                let win_streak = get_u("win_streak", "WIN_STREAK", 0) as u32;
                let best_streak = get_u("best_streak", "BEST_STREAK", 0) as u32;
                let created_at = get_s("created_at", "CREATED_AT");
                let last_login = get_s("last_login", "LAST_LOGIN");

                let mut cards = cards_by_player.remove(&id).unwrap_or_default();
                if cards.is_empty() {
                    let raw_cards = get_s("cards_json", "CARDS_JSON");
                    cards = serde_json::from_str(&raw_cards).unwrap_or_default();
                }

                let decks = decks_by_player.remove(&id).unwrap_or_default();
                let items: Vec<UserItem> =
                    serde_json::from_str(&get_s("items_json", "ITEMS_JSON")).unwrap_or_default();

                let mut user = User {
                    id: if id.is_empty() {
                        format!("p_{}", username)
                    } else {
                        id
                    },
                    username,
                    display_name: if display_name.is_empty() {
                        "Người chơi".to_string()
                    } else {
                        display_name
                    },
                    avatar: if avatar.is_empty() {
                        "knight".to_string()
                    } else {
                        avatar
                    },
                    avatar_id: if avatar_id.is_empty() {
                        "avatar_knight".to_string()
                    } else {
                        avatar_id
                    },
                    cardback_id: if cardback_id.is_empty() {
                        "cb_classic".to_string()
                    } else {
                        cardback_id
                    },
                    board_skin: if board_skin.is_empty() {
                        "board_arena".to_string()
                    } else {
                        board_skin
                    },
                    password_hash,
                    level,
                    current_exp,
                    elo,
                    wins,
                    losses,
                    matches,
                    gold,
                    gems,
                    battle_slots: 3,
                    rank_tier: if rank_tier.is_empty() {
                        "Đồng".to_string()
                    } else {
                        rank_tier
                    },
                    rank_division,
                    rank_stars,
                    mmr,
                    win_streak,
                    best_streak,
                    cards,
                    decks,
                    items,
                    created_at,
                    last_login,
                };
                user.ensure_valid_id_and_deck();
                users.push(user);
            }
        }
        Ok(users)
    }

    pub async fn save_user(&self, user: &User) -> Result<(), String> {
        let username = user.username.replace('\'', "''");
        let display_name = user.display_name.replace('\'', "''");
        let avatar = user.avatar.replace('\'', "''");
        let avatar_id = user.avatar_id.replace('\'', "''");
        let cardback_id = user.cardback_id.replace('\'', "''");
        let board_skin = user.board_skin.replace('\'', "''");
        let pwd = user.password_hash.replace('\'', "''");
        let created_at = user.created_at.replace('\'', "''");
        let last_login = user.last_login.replace('\'', "''");
        let user_id = user.id.replace('\'', "''");
        let rank_tier = user.rank_tier.replace('\'', "''");

        // 1. Merge into PLAYERS
        let sql_player = format!(
            "MERGE INTO players p USING (SELECT '{id}' AS id, '{username}' AS username, '{display_name}' AS display_name, '{avatar_id}' AS avatar_id, '{cardback_id}' AS cardback_id, '{board_skin}' AS board_skin, {level} AS player_level, {current_exp} AS current_exp, {gold} AS gold, {gems} AS gems FROM DUAL) s ON (p.id = s.id OR p.username = s.username) WHEN MATCHED THEN UPDATE SET p.display_name = s.display_name, p.avatar_id = s.avatar_id, p.cardback_id = s.cardback_id, p.board_skin = s.board_skin, p.player_level = s.player_level, p.current_exp = s.current_exp, p.gold = s.gold, p.gems = s.gems WHEN NOT MATCHED THEN INSERT (id, username, display_name, avatar_id, cardback_id, board_skin, player_level, current_exp, gold, gems) VALUES (s.id, s.username, s.display_name, s.avatar_id, s.cardback_id, s.board_skin, s.player_level, s.current_exp, s.gold, s.gems)",
            id = user_id,
            username = username,
            display_name = display_name,
            avatar_id = avatar_id,
            cardback_id = cardback_id,
            board_skin = board_skin,
            level = user.level,
            current_exp = user.current_exp,
            gold = user.gold,
            gems = user.gems
        );
        let _ = self.execute_sql(&sql_player).await;

        // 2. Merge into PLAYER_RATINGS
        let sql_rating = format!(
            "MERGE INTO player_ratings r USING (SELECT '{player_id}' AS player_id, 1 AS season_id, '{rank_tier}' AS rank_tier, {rank_division} AS rank_division, {rank_stars} AS rank_stars, {mmr} AS mmr, {total_matches} AS total_matches, {wins} AS wins, {win_streak} AS win_streak, {best_streak} AS best_streak FROM DUAL) s ON (r.player_id = s.player_id AND r.season_id = s.season_id) WHEN MATCHED THEN UPDATE SET r.rank_tier = s.rank_tier, r.rank_division = s.rank_division, r.rank_stars = s.rank_stars, r.mmr = s.mmr, r.total_matches = s.total_matches, r.wins = s.wins, r.win_streak = s.win_streak, r.best_streak = s.best_streak WHEN NOT MATCHED THEN INSERT (player_id, season_id, rank_tier, rank_division, rank_stars, mmr, total_matches, wins, win_streak, best_streak) VALUES (s.player_id, s.season_id, s.rank_tier, s.rank_division, s.rank_stars, s.mmr, s.total_matches, s.wins, s.win_streak, s.best_streak)",
            player_id = user_id,
            rank_tier = rank_tier,
            rank_division = user.rank_division,
            rank_stars = user.rank_stars,
            mmr = user.mmr,
            total_matches = user.matches,
            wins = user.wins,
            win_streak = user.win_streak,
            best_streak = user.best_streak
        );
        let _ = self.execute_sql(&sql_rating).await;

        // 3. Upsert Cards into PLAYER_CARDS
        for card in &user.cards {
            let cid = card.id.replace('\'', "''");
            let hclass = card.hero_class.replace('\'', "''");
            let cname = card.name.replace('\'', "''");
            let is_st = if card.is_starter { 1 } else { 0 };
            let is_f = if card.is_foil { 1 } else { 0 };
            let sql_c = format!(
                "MERGE INTO player_cards c USING (SELECT '{pid}' AS player_id, '{cid}' AS card_id, '{hclass}' AS hero_class, '{cname}' AS card_name, {qty} AS quantity, {is_foil} AS is_foil, {slvl} AS star_level, {clvl} AS card_level, {is_starter} AS is_starter, {hp_b} AS hp_bonus, {atk_b} AS atk_bonus FROM DUAL) s ON (c.player_id = s.player_id AND c.card_id = s.card_id AND c.is_foil = s.is_foil) WHEN MATCHED THEN UPDATE SET c.quantity = s.quantity, c.card_level = s.card_level, c.star_level = s.star_level, c.hp_bonus = s.hp_bonus, c.atk_bonus = s.atk_bonus WHEN NOT MATCHED THEN INSERT (player_id, card_id, hero_class, card_name, quantity, is_foil, star_level, card_level, is_starter, hp_bonus, atk_bonus) VALUES (s.player_id, s.card_id, s.hero_class, s.card_name, s.quantity, s.is_foil, s.star_level, s.card_level, s.is_starter, s.hp_bonus, s.atk_bonus)",
                pid = user_id,
                cid = cid,
                hclass = hclass,
                cname = cname,
                qty = card.quantity,
                is_foil = is_f,
                slvl = card.star_level,
                clvl = card.level,
                is_starter = is_st,
                hp_b = card.hp_bonus,
                atk_b = card.atk_bonus
            );
            let _ = self.execute_sql(&sql_c).await;
        }

        // 4. Upsert Decks into PLAYER_DECKS
        for deck in &user.decks {
            let did = deck.id.replace('\'', "''");
            let dname = deck.deck_name.replace('\'', "''");
            let hclass = deck.hero_class.replace('\'', "''");
            let cback = deck.cardback_id.replace('\'', "''");
            let cdata = serde_json::to_string(&deck.cards_data)
                .unwrap_or_else(|_| "[]".to_string())
                .replace('\'', "''");
            let is_v = if deck.is_valid { 1 } else { 0 };

            let sql_d = format!(
                "MERGE INTO player_decks d USING (SELECT '{id}' AS id, '{pid}' AS player_id, '{dname}' AS deck_name, '{hclass}' AS hero_class, '{cback}' AS cardback_id, '{cdata}' AS cards_data, {is_v} AS is_valid FROM DUAL) s ON (d.id = s.id) WHEN MATCHED THEN UPDATE SET d.deck_name = s.deck_name, d.hero_class = s.hero_class, d.cardback_id = s.cardback_id, d.cards_data = s.cards_data, d.is_valid = s.is_valid, d.updated_at = CURRENT_TIMESTAMP WHEN NOT MATCHED THEN INSERT (id, player_id, deck_name, hero_class, cardback_id, cards_data, is_valid) VALUES (s.id, s.player_id, s.deck_name, s.hero_class, s.cardback_id, s.cards_data, s.is_valid)",
                id = did,
                pid = user_id,
                dname = dname,
                hclass = hclass,
                cback = cback,
                cdata = cdata,
                is_v = is_v
            );
            let _ = self.execute_sql(&sql_d).await;
        }

        // 5. Legacy USERS table
        let cards_json = serde_json::to_string(&user.cards)
            .unwrap_or_else(|_| "[]".to_string())
            .replace('\'', "''");
        let items_json = serde_json::to_string(&user.items)
            .unwrap_or_else(|_| "[]".to_string())
            .replace('\'', "''");
        let sql_users = format!(
            "MERGE INTO USERS u USING (SELECT '{username}' AS username, '{display_name}' AS display_name, '{avatar}' AS avatar, '{pwd}' AS password_hash, {elo} AS elo, {wins} AS wins, {losses} AS losses, {matches} AS matches, {gold} AS gold, '{cards_json}' AS cards_json, '{items_json}' AS items_json, '{created_at}' AS created_at, '{last_login}' AS last_login FROM DUAL) s ON (u.username = s.username) WHEN MATCHED THEN UPDATE SET u.display_name = s.display_name, u.avatar = s.avatar, u.password_hash = s.password_hash, u.elo = s.elo, u.wins = s.wins, u.losses = s.losses, u.matches = s.matches, u.gold = s.gold, u.cards_json = s.cards_json, u.items_json = s.items_json, u.last_login = s.last_login WHEN NOT MATCHED THEN INSERT (username, display_name, avatar, password_hash, elo, wins, losses, matches, gold, cards_json, items_json, created_at, last_login) VALUES (s.username, s.display_name, s.avatar, s.password_hash, s.elo, s.wins, s.losses, s.matches, s.gold, s.cards_json, s.items_json, s.created_at, s.last_login)",
            username = username,
            display_name = display_name,
            avatar = avatar,
            pwd = pwd,
            elo = user.elo,
            wins = user.wins,
            losses = user.losses,
            matches = user.matches,
            gold = user.gold,
            cards_json = cards_json,
            items_json = items_json,
            created_at = created_at,
            last_login = last_login,
        );
        self.execute_sql(&sql_users).await.map(|_| ())
    }

    pub async fn delete_user(&self, username: &str) -> Result<(), String> {
        let clean = username.replace('\'', "''");
        let user_id = format!("p_{}", clean);
        let sql = format!(
            "BEGIN \
                DELETE FROM player_cards WHERE player_id = '{user_id}'; \
                DELETE FROM player_decks WHERE player_id = '{user_id}'; \
                DELETE FROM player_ratings WHERE player_id = '{user_id}'; \
                DELETE FROM player_formations WHERE username = '{clean}' OR player_id = '{user_id}'; \
                DELETE FROM players WHERE username = '{clean}' OR id = '{user_id}'; \
                DELETE FROM users WHERE username = '{clean}'; \
            END;",
            clean = clean,
            user_id = user_id
        );
        self.execute_sql(&sql).await.map(|_| ())
    }

    pub async fn delete_deck(&self, deck_id: &str) -> Result<(), String> {
        let did = deck_id.replace('\'', "''");
        let sql = format!("DELETE FROM player_decks WHERE id = '{}'", did);
        self.execute_sql(&sql).await.map(|_| ())
    }

    pub async fn delete_card(&self, player_id: &str, card_id: &str) -> Result<(), String> {
        let pid = player_id.replace('\'', "''");
        let cid = card_id.replace('\'', "''");
        let sql = format!(
            "DELETE FROM player_cards WHERE player_id = '{}' AND card_id = '{}'",
            pid, cid
        );
        self.execute_sql(&sql).await.map(|_| ())
    }

    pub async fn save_formation(
        &self,
        username: &str,
        formation: &game_data_schema::PlayerFormationRow,
    ) -> Result<(), String> {
        let u = username.replace('\'', "''");
        let pid = format!("p_{}", u);
        let f_type = formation.formation_type.replace('\'', "''");
        let fid = format!("{}_{}", u, f_type);
        let f_data = serde_json::to_string(formation).unwrap_or_else(|_| "{}".to_string());
        let clob_expr = to_clob_expr(&f_data);

        let sql = format!(
            "MERGE INTO player_formations f USING (SELECT '{fid}' AS formation_id, '{pid}' AS player_id, '{u}' AS username, '{ftype}' AS formation_type, {clob_expr} AS formation_data FROM DUAL) s ON (f.formation_id = s.formation_id) WHEN MATCHED THEN UPDATE SET f.formation_data = s.formation_data, f.updated_at = CURRENT_TIMESTAMP WHEN NOT MATCHED THEN INSERT (formation_id, player_id, username, formation_type, formation_data, updated_at) VALUES (s.formation_id, s.player_id, s.username, s.formation_type, s.formation_data, CURRENT_TIMESTAMP)",
            fid = fid,
            pid = pid,
            u = u,
            ftype = f_type,
            clob_expr = clob_expr
        );
        self.execute_sql(&sql).await.map(|_| ())
    }

    pub async fn load_all_formations(
        &self,
    ) -> Result<HashMap<String, Vec<game_data_schema::PlayerFormationRow>>, String> {
        let sql = "SELECT username, formation_type, formation_data FROM player_formations";
        let val = self.execute_sql(sql).await?;
        let mut map: HashMap<String, Vec<game_data_schema::PlayerFormationRow>> = HashMap::new();
        if let Some(items) = val["items"][0]["resultSet"]["items"].as_array() {
            for row in items {
                let u = row
                    .get("username")
                    .or_else(|| row.get("USERNAME"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let f_data = row
                    .get("formation_data")
                    .or_else(|| row.get("FORMATION_DATA"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}");
                if let Ok(form) =
                    serde_json::from_str::<game_data_schema::PlayerFormationRow>(f_data)
                {
                    map.entry(u.to_lowercase()).or_default().push(form);
                }
            }
        }
        Ok(map)
    }

    pub async fn save_master_config(&self, key: &str, data_json: &str) -> Result<(), String> {
        let k = key.replace('\'', "''");
        let clob_expr = to_clob_expr(data_json);
        let sql = format!(
            "MERGE INTO master_configs m USING (SELECT '{k}' AS config_key, {clob_expr} AS config_data FROM DUAL) s ON (m.config_key = s.config_key) WHEN MATCHED THEN UPDATE SET m.config_data = s.config_data, m.updated_at = CURRENT_TIMESTAMP WHEN NOT MATCHED THEN INSERT (config_key, config_data, updated_at) VALUES (s.config_key, s.config_data, CURRENT_TIMESTAMP)",
            k = k,
            clob_expr = clob_expr
        );
        self.execute_sql(&sql).await.map(|_| ())
    }

    pub async fn load_all_master_configs(&self) -> Result<HashMap<String, String>, String> {
        let sql = "SELECT config_key, config_data FROM master_configs";
        let val = self.execute_sql(sql).await?;
        let mut configs = HashMap::new();
        if let Some(items) = val["items"][0]["resultSet"]["items"].as_array() {
            for row in items {
                let k = row
                    .get("config_key")
                    .or_else(|| row.get("CONFIG_KEY"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let d = row
                    .get("config_data")
                    .or_else(|| row.get("CONFIG_DATA"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !k.is_empty() {
                    configs.insert(k.to_string(), d.to_string());
                }
            }
        }
        Ok(configs)
    }

    pub async fn record_match(&self, m: &MatchRecord) -> Result<(), String> {
        let match_id = m.match_id.replace('\'', "''");
        let host = m.host.replace('\'', "''");
        let guest = m.guest.replace('\'', "''");
        let winner = m.winner.as_deref().unwrap_or("").replace('\'', "''");
        let timestamp = m.timestamp.replace('\'', "''");

        let sql = format!(
            "INSERT INTO MATCH_HISTORY (match_id, room_id, player_red, player_blue, winner, duration_sec, timestamp) VALUES ('{match_id}', '{match_id}', '{host}', '{guest}', '{winner}', {rounds}, '{timestamp}')",
            match_id = match_id,
            host = host,
            guest = guest,
            winner = winner,
            rounds = m.rounds,
            timestamp = timestamp
        );

        self.execute_sql(&sql).await.map(|_| ())
    }
}
