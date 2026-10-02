use axum::{
    Router,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{RwLock, mpsc};
use base64::prelude::*;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

// ==========================================\n// DATA MODELS & PERSISTENCE
// ==========================================
fn default_gold() -> u32 {
    100
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserCard {
    pub id: String,
    pub hero_class: String, // "Knight", "Archer", "Mage", "Assassin", "Cleric"
    pub name: String,
    pub star_level: u32,
    pub level: u32,
    pub is_starter: bool,
    pub hp_bonus: f32,
    pub atk_bonus: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserItem {
    pub id: String,
    pub item_type: String,
    pub name: String,
    pub count: u32,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub display_name: String,
    pub avatar: String,
    #[serde(default)]
    pub password_hash: String,
    pub elo: i32,
    pub wins: u32,
    pub losses: u32,
    pub matches: u32,
    #[serde(default = "default_gold")]
    pub gold: u32,
    #[serde(default)]
    pub cards: Vec<UserCard>,
    #[serde(default)]
    pub items: Vec<UserItem>,
    pub created_at: String,
    pub last_login: String,
}

impl User {
    pub fn sanitized(mut self) -> Self {
        self.password_hash.clear();
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchRecord {
    pub match_id: String,
    pub host: String,
    pub guest: String,
    pub winner: Option<String>,
    pub rounds: usize,
    pub timestamp: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct DatabaseData {
    pub users: HashMap<String, User>,
    pub matches: Vec<MatchRecord>,
}

pub fn create_starter_card(username: &str, avatar: &str) -> UserCard {
    let (hero_class, name) = match avatar {
        "archer" => ("Archer", "Cung Thủ Thần Nhãn"),
        "mage" => ("Mage", "Pháp Sư Băng Hoả"),
        "assassin" => ("Assassin", "Sát Thủ Bóng Đêm"),
        "cleric" => ("Cleric", "Mục Sư Thánh Quang"),
        _ => ("Knight", "Hiệp Sĩ Hoàng Gia"),
    };
    UserCard {
        id: format!("card_{}_starter", username),
        hero_class: hero_class.to_string(),
        name: name.to_string(),
        star_level: 1,
        level: 1,
        is_starter: true,
        hp_bonus: 0.0,
        atk_bonus: 0.0,
    }
}


// ==========================================
// ORACLE AUTONOMOUS DATABASE (ALWAYS FREE) CLIENT
// ==========================================
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
        let pass = std::env::var("ORACLE_ADB_PASSWORD").unwrap_or_else(|_| "TacticalArenaDb2026#".to_string());

        let creds = format!("{}:{}", user, pass);
        let b64 = BASE64_STANDARD.encode(creds.as_bytes());
        let auth_header = format!("Basic {}", b64);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
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
        let sql = "SELECT username, display_name, avatar, password_hash, elo, wins, losses, matches, gold, cards_json, items_json, created_at, last_login FROM USERS";
        let val = self.execute_sql(sql).await?;
        let mut users = Vec::new();
        if let Some(items) = val["items"][0]["resultSet"]["items"].as_array() {
            for row in items {
                let username = row["username"].as_str().unwrap_or("").to_string();
                if username.is_empty() {
                    continue;
                }
                let display_name = row["display_name"].as_str().unwrap_or(&username).to_string();
                let avatar = row["avatar"].as_str().unwrap_or("knight").to_string();
                let password_hash = row["password_hash"].as_str().unwrap_or("").to_string();
                let elo = row["elo"].as_i64().unwrap_or(1000) as i32;
                let wins = row["wins"].as_u64().unwrap_or(0) as u32;
                let losses = row["losses"].as_u64().unwrap_or(0) as u32;
                let matches = row["matches"].as_u64().unwrap_or(0) as u32;
                let gold = row["gold"].as_u64().unwrap_or(100) as u32;

                let cards: Vec<UserCard> = row["cards_json"]
                    .as_str()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();

                let items: Vec<UserItem> = row["items_json"]
                    .as_str()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();

                let created_at = row["created_at"].as_str().unwrap_or("").to_string();
                let last_login = row["last_login"].as_str().unwrap_or("").to_string();

                users.push(User {
                    username,
                    display_name,
                    avatar,
                    password_hash,
                    elo,
                    wins,
                    losses,
                    matches,
                    gold,
                    cards,
                    items,
                    created_at,
                    last_login,
                });
            }
        }
        Ok(users)
    }

    pub async fn save_user(&self, user: &User) -> Result<(), String> {
        let cards_json = serde_json::to_string(&user.cards)
            .unwrap_or_else(|_| "[]".to_string())
            .replace('\'', "''");
        let items_json = serde_json::to_string(&user.items)
            .unwrap_or_else(|_| "[]".to_string())
            .replace('\'', "''");
        let username = user.username.replace('\'', "''");
        let display_name = user.display_name.replace('\'', "''");
        let avatar = user.avatar.replace('\'', "''");
        let pwd = user.password_hash.replace('\'', "''");
        let created_at = user.created_at.replace('\'', "''");
        let last_login = user.last_login.replace('\'', "''");

        let sql = format!(
            "MERGE INTO USERS u             USING (                 SELECT                     '{username}' AS username,                     '{display_name}' AS display_name,                     '{avatar}' AS avatar,                     '{pwd}' AS password_hash,                     {elo} AS elo,                     {wins} AS wins,                     {losses} AS losses,                     {matches} AS matches,                     {gold} AS gold,                     '{cards_json}' AS cards_json,                     '{items_json}' AS items_json,                     '{created_at}' AS created_at,                     '{last_login}' AS last_login                 FROM DUAL             ) s             ON (u.username = s.username)             WHEN MATCHED THEN                 UPDATE SET                     u.display_name = s.display_name,                     u.avatar = s.avatar,                     u.password_hash = s.password_hash,                     u.elo = s.elo,                     u.wins = s.wins,                     u.losses = s.losses,                     u.matches = s.matches,                     u.gold = s.gold,                     u.cards_json = s.cards_json,                     u.items_json = s.items_json,                     u.last_login = s.last_login             WHEN NOT MATCHED THEN                 INSERT (username, display_name, avatar, password_hash, elo, wins, losses, matches, gold, cards_json, items_json, created_at, last_login)                 VALUES (s.username, s.display_name, s.avatar, s.password_hash, s.elo, s.wins, s.losses, s.matches, s.gold, s.cards_json, s.items_json, s.created_at, s.last_login)",
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

        self.execute_sql(&sql).await.map(|_| ())
    }

    pub async fn record_match(&self, m: &MatchRecord) -> Result<(), String> {
        let match_id = m.match_id.replace('\'', "''");
        let host = m.host.replace('\'', "''");
        let guest = m.guest.replace('\'', "''");
        let winner = m.winner.as_deref().unwrap_or("").replace('\'', "''");
        let timestamp = m.timestamp.replace('\'', "''");

        let sql = format!(
            "INSERT INTO MATCH_HISTORY (match_id, room_id, player_red, player_blue, winner, duration_sec, timestamp)             VALUES ('{match_id}', '{match_id}', '{host}', '{guest}', '{winner}', {rounds}, '{timestamp}')",
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

pub struct Database {
    file_path: PathBuf,
    data: DatabaseData,
    adb: Option<OracleAdbClient>,
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let file_path = path.as_ref().to_path_buf();
        if let Some(parent) = file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut data = if file_path.exists() {
            fs::read_to_string(&file_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            DatabaseData::default()
        };

        for user in data.users.values_mut() {
            if user.gold == 0 {
                user.gold = 100;
            }
            if user.cards.is_empty() {
                user.cards
                    .push(create_starter_card(&user.username, &user.avatar));
            }
        }

        Self { file_path, data, adb: None }
    }

    pub async fn new_with_adb(path: impl AsRef<Path>) -> Self {
        let file_path = path.as_ref().to_path_buf();
        if let Some(parent) = file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let adb = OracleAdbClient::new_from_env();
        let mut data = DatabaseData::default();
        let mut loaded_from_adb = false;

        if let Some(ref client) = adb {
            match client.load_all_users().await {
                Ok(users) => {
                    println!("[ORACLE AUTONOMOUS DB] Loaded {} users from Oracle Cloud ADB.", users.len());
                    for u in users {
                        data.users.insert(u.username.clone(), u);
                    }
                    loaded_from_adb = true;
                }
                Err(e) => {
                    eprintln!("[ORACLE ADB WARNING] Could not load from ADB ({}). Falling back to local disk.", e);
                }
            }
        }

        if !loaded_from_adb && file_path.exists() {
            if let Ok(content) = fs::read_to_string(&file_path) {
                if let Ok(local_data) = serde_json::from_str::<DatabaseData>(&content) {
                    data = local_data;
                    println!("[LOCAL DB] Loaded {} users and {} matches from local file.", data.users.len(), data.matches.len());
                }
            }
        }

        for user in data.users.values_mut() {
            if user.gold == 0 {
                user.gold = 100;
            }
            if user.cards.is_empty() {
                user.cards
                    .push(create_starter_card(&user.username, &user.avatar));
            }
        }

        Self { file_path, data, adb }
    }

    fn persist_user_adb(&self, user: &User) {
        if let Some(ref client) = self.adb {
            let client = client.clone();
            let user = user.clone();
            tokio::spawn(async move {
                if let Err(e) = client.save_user(&user).await {
                    eprintln!("[ORACLE ADB ERROR] Failed to save user {}: {}", user.username, e);
                }
            });
        }
    }

    fn persist_match_adb(&self, m: &MatchRecord) {
        if let Some(ref client) = self.adb {
            let client = client.clone();
            let m = m.clone();
            tokio::spawn(async move {
                if let Err(e) = client.record_match(&m).await {
                    eprintln!("[ORACLE ADB ERROR] Failed to save match {}: {}", m.match_id, e);
                }
            });
        }
    }

    fn save(&self) {
        let tmp = self.file_path.with_extension("tmp");
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(&tmp, json);
            let _ = fs::rename(&tmp, &self.file_path);
        }
    }

    fn hash_password(pwd: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}_tac_arena_salt", pwd).as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn register(
        &mut self,
        username: &str,
        password: &str,
        display_name: &str,
        avatar: &str,
    ) -> Result<User, String> {
        let clean = username.trim().to_lowercase();
        if clean.len() < 3 || clean.len() > 20 {
            return Err("Tên tài khoản phải từ 3 đến 20 ký tự!".to_string());
        }
        if !clean.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err("Tên tài khoản chỉ chứa chữ cái, số và dấu gạch dưới!".to_string());
        }
        if password.len() < 4 {
            return Err("Mật khẩu phải từ 4 ký tự trở lên!".to_string());
        }
        if self.data.users.contains_key(&clean) {
            return Err("Tên tài khoản này đã được sử dụng!".to_string());
        }

        let now = chrono_now();
        let avatar_str = if avatar.is_empty() {
            "knight".to_string()
        } else {
            avatar.to_string()
        };
        let starter_card = create_starter_card(&clean, &avatar_str);

        let user = User {
            username: clean.clone(),
            display_name: if display_name.trim().is_empty() {
                clean.clone()
            } else {
                display_name.trim().chars().take(24).collect()
            },
            avatar: avatar_str,
            password_hash: Self::hash_password(password),
            elo: 1000,
            wins: 0,
            losses: 0,
            matches: 0,
            gold: 100,
            cards: vec![starter_card],
            items: vec![],
            created_at: now.clone(),
            last_login: now,
        };

        self.data.users.insert(clean, user.clone());
        self.save();
        self.persist_user_adb(&user);
        Ok(user)
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<User, String> {
        let clean = username.trim().to_lowercase();
        let user = self
            .data
            .users
            .get_mut(&clean)
            .ok_or_else(|| "Tài khoản không tồn tại!".to_string())?;

        let hash = Self::hash_password(password);
        if user.password_hash != hash {
            return Err("Mật khẩu không chính xác!".to_string());
        }

        user.last_login = chrono_now();
        if user.gold == 0 {
            user.gold = 100;
        }
        if user.cards.is_empty() {
            user.cards
                .push(create_starter_card(&user.username, &user.avatar));
        }

        let res = user.clone();
        self.save();
        self.persist_user_adb(&res);
        Ok(res)
    }

    pub fn get_user(&self, username: &str) -> Option<User> {
        self.data.users.get(&username.to_lowercase()).cloned()
    }

    pub fn buy_card(
        &mut self,
        username: &str,
        hero_class: &str,
    ) -> Result<(User, UserCard), String> {
        let clean = username.trim().to_lowercase();
        let user = self
            .data
            .users
            .get_mut(&clean)
            .ok_or("Người chơi không tồn tại!")?;

        let (cost, name) = match hero_class {
            "Knight" => (50, "Hiệp Sĩ Hoàng Gia"),
            "Archer" => (50, "Cung Thủ Thần Nhãn"),
            "Mage" => (60, "Pháp Sư Băng Hoả"),
            "Assassin" => (60, "Sát Thủ Bóng Đêm"),
            "Cleric" => (55, "Mục Sư Thánh Quang"),
            _ => return Err("Loại thẻ tướng không hợp lệ!".to_string()),
        };

        if user.gold < cost {
            return Err(format!(
                "Bạn không đủ vàng! Cần {} vàng, hiện có {} vàng.",
                cost, user.gold
            ));
        }

        if user.cards.len() >= 25 {
            return Err("Kho thẻ bài đã đạt giới hạn tối đa (25 thẻ)!".to_string());
        }

        user.gold -= cost;
        let card_id = format!(
            "card_{}_{}_{}",
            clean,
            hero_class.to_lowercase(),
            chrono_now()
        );
        let new_card = UserCard {
            id: card_id,
            hero_class: hero_class.to_string(),
            name: name.to_string(),
            star_level: 1,
            level: 1,
            is_starter: false,
            hp_bonus: 0.0,
            atk_bonus: 0.0,
        };

        user.cards.push(new_card.clone());
        let res_user = user.clone();
        self.save();
        self.persist_user_adb(&res_user);
        Ok((res_user, new_card))
    }

    pub fn upgrade_card(
        &mut self,
        username: &str,
        card_id: &str,
        upgrade_type: &str,
    ) -> Result<(User, String), String> {
        let clean = username.trim().to_lowercase();
        let user = self
            .data
            .users
            .get_mut(&clean)
            .ok_or("Người chơi không tồn tại!")?;

        let card = user
            .cards
            .iter_mut()
            .find(|c| c.id == card_id)
            .ok_or("Không tìm thấy thẻ bài này trong kho!")?;

        let msg = match upgrade_type {
            "level" => {
                if card.level >= 10 {
                    return Err("Thẻ bài đã đạt cấp tối đa (Cấp 10)!".to_string());
                }
                let cost = card.level * 20;
                if user.gold < cost {
                    return Err(format!(
                        "Cần {} vàng để nâng cấp (hiện có {} vàng)!",
                        cost, user.gold
                    ));
                }
                user.gold -= cost;
                card.level += 1;
                card.hp_bonus += 25.0;
                card.atk_bonus += 5.0;
                format!(
                    "Nâng cấp thành công lên Cấp {}! (+25 HP, +5 ATK)",
                    card.level
                )
            }
            "star" => {
                if card.star_level >= 3 {
                    return Err("Thẻ bài đã đạt số sao tối đa (3★)!".to_string());
                }
                let cost = 100;
                if user.gold < cost {
                    return Err(format!(
                        "Cần {} vàng để nâng sao (hiện có {} vàng)!",
                        cost, user.gold
                    ));
                }
                user.gold -= cost;
                card.star_level += 1;
                card.hp_bonus += 60.0;
                card.atk_bonus += 15.0;
                format!(
                    "Đột phá thành công lên {}★! (+60 HP, +15 ATK)",
                    card.star_level
                )
            }
            _ => return Err("Loại nâng cấp không hợp lệ!".to_string()),
        };

        let res_user = user.clone();
        self.save();
        self.persist_user_adb(&res_user);
        Ok((res_user, msg))
    }

    pub fn sell_card(&mut self, username: &str, card_id: &str) -> Result<(User, u32), String> {
        let clean = username.trim().to_lowercase();
        let user = self
            .data
            .users
            .get_mut(&clean)
            .ok_or("Người chơi không tồn tại!")?;

        let idx = user
            .cards
            .iter()
            .position(|c| c.id == card_id)
            .ok_or("Không tìm thấy thẻ bài này trong kho!")?;

        if user.cards[idx].is_starter {
            return Err("Thẻ bài khởi đầu là linh hồn của bạn, không thể bán đi!".to_string());
        }

        let card = &user.cards[idx];
        let base_cost = match card.hero_class.as_str() {
            "Knight" | "Archer" => 50,
            "Mage" | "Assassin" => 60,
            _ => 55,
        };
        let refund = (base_cost / 2)
            + (card.level.saturating_sub(1) * 10)
            + (card.star_level.saturating_sub(1) * 40);

        user.cards.remove(idx);
        user.gold += refund;
        let res_user = user.clone();
        self.save();
        self.persist_user_adb(&res_user);
        Ok((res_user, refund))
    }

    pub fn reward_match(
        &mut self,
        username: &str,
        mode: &str,
        win: bool,
    ) -> Result<(User, u32), String> {
        let clean = username.trim().to_lowercase();
        let user = self
            .data
            .users
            .get_mut(&clean)
            .ok_or("Người chơi không tồn tại!")?;

        let gold_earned = if win {
            if mode == "pve" { 40 } else { 80 }
        } else {
            10
        };

        user.gold += gold_earned;
        let res_user = user.clone();
        self.save();
        self.persist_user_adb(&res_user);
        Ok((res_user, gold_earned))
    }

    pub fn record_match(
        &mut self,
        match_id: &str,
        host_user: &str,
        guest_user: &str,
        winner_user: Option<&str>,
        rounds: usize,
    ) {
        let rec = MatchRecord {
            match_id: match_id.to_string(),
            host: host_user.to_string(),
            guest: guest_user.to_string(),
            winner: winner_user.map(str::to_string),
            rounds,
            timestamp: chrono_now(),
        };

        if let Some(win) = winner_user {
            let is_host_win = win.eq_ignore_ascii_case(host_user);
            if let Some(h) = self.data.users.get_mut(&host_user.to_lowercase()) {
                h.matches += 1;
                if is_host_win {
                    h.wins += 1;
                    h.elo += 25;
                    h.gold += 80;
                } else {
                    h.losses += 1;
                    h.elo = (h.elo - 20).max(500);
                    h.gold += 15;
                }
                let h_clone = h.clone();
                self.persist_user_adb(&h_clone);
            }
            if let Some(g) = self.data.users.get_mut(&guest_user.to_lowercase()) {
                g.matches += 1;
                if !is_host_win {
                    g.wins += 1;
                    g.elo += 25;
                    g.gold += 80;
                } else {
                    g.losses += 1;
                    g.elo = (g.elo - 20).max(500);
                    g.gold += 15;
                }
                let g_clone = g.clone();
                self.persist_user_adb(&g_clone);
            }
        }

        self.persist_match_adb(&rec);
        self.data.matches.insert(0, rec);
        if self.data.matches.len() > 200 {
            self.data.matches.truncate(200);
        }
        self.save();
    }

    pub fn get_leaderboard(&self, limit: usize) -> Vec<User> {
        let mut list: Vec<User> = self.data.users.values().cloned().collect();
        list.sort_by(|a, b| b.elo.cmp(&a.elo).then_with(|| b.wins.cmp(&a.wins)));
        list.into_iter().take(limit).map(|u| u.sanitized()).collect()
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

// ==========================================\n// MULTIPLAYER ROOM & MATCHMAKING
// ==========================================
type Tx = mpsc::UnboundedSender<Message>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitData {
    pub col: usize,
    pub row: usize,
    pub class: String,
    pub star_level: u8,
}

#[derive(Clone, Debug)]
pub struct PlayerSession {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub elo: i32,
    pub hp: i32,
    pub ready: bool,
    pub lineup: Vec<UnitData>,
    pub tx: Tx,
}

pub struct Room {
    pub code: String,
    pub round: usize,
    pub host: PlayerSession,
    pub guest: Option<PlayerSession>,
}

#[derive(Serialize)]
pub struct PublicRoomInfo {
    pub code: String,
    pub host_name: String,
    pub host_avatar: String,
    pub host_elo: i32,
}

pub struct QuickMatchEntry {
    pub session: PlayerSession,
}

pub struct AppState {
    pub db: Arc<RwLock<Database>>,
    pub rooms: Arc<RwLock<HashMap<String, Room>>>,
    pub quick_match: Arc<RwLock<Option<QuickMatchEntry>>>,
}

fn generate_room_code() -> String {
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen_range(1000..9999);
    format!("{:04}", num)
}

// ==========================================\n// REST API REQUEST DTOs
// ==========================================
#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    display_name: Option<String>,
    avatar: Option<String>,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct ProfileQuery {
    username: String,
}

#[derive(Deserialize)]
struct BuyCardRequest {
    username: String,
    hero_class: String,
}

#[derive(Deserialize)]
struct UpgradeCardRequest {
    username: String,
    card_id: String,
    upgrade_type: String, // "level" or "star"
}

#[derive(Deserialize)]
struct SellCardRequest {
    username: String,
    card_id: String,
}

#[derive(Deserialize)]
struct MatchRewardRequest {
    username: String,
    mode: Option<String>,
    win: bool,
}

#[derive(Deserialize)]
struct RecordMatchRequest {
    match_id: Option<String>,
    host: String,
    guest: String,
    winner: Option<String>,
    rounds: Option<usize>,
}

async fn handle_register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.register(
        &payload.username,
        &payload.password,
        payload.display_name.as_deref().unwrap_or(""),
        payload.avatar.as_deref().unwrap_or("knight"),
    ) {
        Ok(user) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "user": user.sanitized() })),
        ),
        Err(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": msg })),
        ),
    }
}

async fn handle_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.login(&payload.username, &payload.password) {
        Ok(user) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "user": user.sanitized() })),
        ),
        Err(msg) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "success": false, "message": msg })),
        ),
    }
}

async fn handle_profile(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
) -> impl IntoResponse {
    let db = state.db.read().await;
    match db.get_user(&query.username) {
        Some(user) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "user": user.sanitized() })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "message": "Không tìm thấy người chơi!" })),
        ),
    }
}

async fn handle_buy_card(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BuyCardRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.buy_card(&payload.username, &payload.hero_class) {
        Ok((user, card)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "user": user.sanitized(), "card": card })),
        ),
        Err(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": msg })),
        ),
    }
}

async fn handle_upgrade_card(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpgradeCardRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.upgrade_card(&payload.username, &payload.card_id, &payload.upgrade_type) {
        Ok((user, message)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "user": user.sanitized(), "message": message })),
        ),
        Err(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": msg })),
        ),
    }
}

async fn handle_sell_card(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SellCardRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.sell_card(&payload.username, &payload.card_id) {
        Ok((user, refund)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "user": user.sanitized(), "refund": refund })),
        ),
        Err(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": msg })),
        ),
    }
}

async fn handle_match_reward(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MatchRewardRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    let mode = payload.mode.as_deref().unwrap_or("pve");
    match db.reward_match(&payload.username, mode, payload.win) {
        Ok((user, gold_earned)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "user": user.sanitized(), "gold_earned": gold_earned })),
        ),
        Err(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": msg })),
        ),
    }
}

async fn handle_leaderboard(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.read().await;
    let list = db.get_leaderboard(20);
    Json(serde_json::json!({ "success": true, "leaderboard": list }))
}

async fn handle_record_match(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RecordMatchRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    let match_id = payload
        .match_id
        .unwrap_or_else(|| format!("m_{}", chrono_now()));
    db.record_match(
        &match_id,
        &payload.host,
        &payload.guest,
        payload.winner.as_deref(),
        payload.rounds.unwrap_or(1),
    );
    Json(serde_json::json!({ "success": true }))
}

// ==========================================\n// WEBSOCKET HANDLER
// ==========================================
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_client(socket, state))
}

async fn handle_ws_client(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut current_room: Option<String> = None;
    let mut current_role: Option<String> = None;

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            let parsed: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let raw_type = parsed
                .get("type")
                .or_else(|| parsed.get("action"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let msg_type = raw_type.to_uppercase();

            match msg_type.as_str() {
                "GET_ROOMS" | "LOBBY_ROOMS" => {
                    let rooms_guard = state.rooms.read().await;
                    let list: Vec<PublicRoomInfo> = rooms_guard
                        .values()
                        .filter(|r| r.guest.is_none())
                        .map(|r| PublicRoomInfo {
                            code: r.code.clone(),
                            host_name: r.host.name.clone(),
                            host_avatar: r.host.avatar.clone(),
                            host_elo: r.host.elo,
                        })
                        .collect();

                    let res = serde_json::json!({
                        "type": "LOBBY_ROOMS_RESPONSE",
                        "rooms": list
                    });
                    let _ = tx.send(Message::Text(res.to_string().into()));
                }

                "CREATE_ROOM" => {
                    let player_id = parsed
                        .get("player_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("guest");
                    let player_name = parsed
                        .get("player_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Player 1");
                    let avatar = parsed
                        .get("avatar")
                        .and_then(|v| v.as_str())
                        .unwrap_or("knight");
                    let elo = parsed.get("elo").and_then(|v| v.as_i64()).unwrap_or(1000) as i32;

                    let mut rooms_guard = state.rooms.write().await;
                    let mut code = generate_room_code();
                    while rooms_guard.contains_key(&code) {
                        code = generate_room_code();
                    }

                    let session = PlayerSession {
                        id: player_id.to_string(),
                        name: player_name.to_string(),
                        avatar: avatar.to_string(),
                        elo,
                        hp: 100,
                        ready: false,
                        lineup: vec![],
                        tx: tx.clone(),
                    };

                    let room = Room {
                        code: code.clone(),
                        round: 1,
                        host: session,
                        guest: None,
                    };

                    rooms_guard.insert(code.clone(), room);
                    current_room = Some(code.clone());
                    current_role = Some("host".to_string());

                    let res = serde_json::json!({
                        "type": "ROOM_CREATED",
                        "room_code": code,
                        "role": "host",
                        "player_name": player_name
                    });
                    let _ = tx.send(Message::Text(res.to_string().into()));
                    println!("[RUST WS] Room {} created by {}", code, player_name);
                }

                "JOIN_ROOM" => {
                    let code = parsed
                        .get("room_code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_uppercase();

                    let player_id = parsed
                        .get("player_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("guest");
                    let player_name = parsed
                        .get("player_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Player 2");
                    let avatar = parsed
                        .get("avatar")
                        .and_then(|v| v.as_str())
                        .unwrap_or("archer");
                    let elo = parsed.get("elo").and_then(|v| v.as_i64()).unwrap_or(1000) as i32;

                    let mut rooms_guard = state.rooms.write().await;
                    if let Some(room) = rooms_guard.get_mut(&code) {
                        if room.guest.is_none() {
                            let guest_session = PlayerSession {
                                id: player_id.to_string(),
                                name: player_name.to_string(),
                                avatar: avatar.to_string(),
                                elo,
                                hp: 100,
                                ready: false,
                                lineup: vec![],
                                tx: tx.clone(),
                            };

                            let host_name = room.host.name.clone();
                            room.guest = Some(guest_session);
                            current_room = Some(code.clone());
                            current_role = Some("guest".to_string());

                            let to_host = serde_json::json!({
                                "type": "ROOM_JOINED",
                                "room_code": code,
                                "role": "host",
                                "player_name": host_name,
                                "opponent_name": player_name
                            });
                            let _ = room.host.tx.send(Message::Text(to_host.to_string().into()));

                            let to_guest = serde_json::json!({
                                "type": "ROOM_JOINED",
                                "room_code": code,
                                "role": "guest",
                                "player_name": player_name,
                                "opponent_name": host_name
                            });
                            let _ = tx.send(Message::Text(to_guest.to_string().into()));
                            println!("[RUST WS] {} joined room {}", player_name, code);
                        } else {
                            let err =
                                serde_json::json!({ "type": "ERROR", "message": "Phòng đã đầy!" });
                            let _ = tx.send(Message::Text(err.to_string().into()));
                        }
                    } else {
                        let err = serde_json::json!({ "type": "ERROR", "message": "Mã phòng không tồn tại!" });
                        let _ = tx.send(Message::Text(err.to_string().into()));
                    }
                }

                "QUICK_MATCH" => {
                    let player_id = parsed
                        .get("player_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("guest");
                    let player_name = parsed
                        .get("player_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Player");
                    let avatar = parsed
                        .get("avatar")
                        .and_then(|v| v.as_str())
                        .unwrap_or("knight");
                    let elo = parsed.get("elo").and_then(|v| v.as_i64()).unwrap_or(1000) as i32;

                    let mut qm_guard = state.quick_match.write().await;
                    if let Some(queued) = qm_guard.take() {
                        let mut rooms_guard = state.rooms.write().await;
                        let mut code = generate_room_code();
                        while rooms_guard.contains_key(&code) {
                            code = generate_room_code();
                        }

                        let guest_session = PlayerSession {
                            id: player_id.to_string(),
                            name: player_name.to_string(),
                            avatar: avatar.to_string(),
                            elo,
                            hp: 100,
                            ready: false,
                            lineup: vec![],
                            tx: tx.clone(),
                        };

                        let host_name = queued.session.name.clone();
                        let room = Room {
                            code: code.clone(),
                            round: 1,
                            host: queued.session,
                            guest: Some(guest_session),
                        };

                        let to_host = serde_json::json!({
                            "type": "ROOM_JOINED",
                            "room_code": code,
                            "role": "host",
                            "player_name": host_name,
                            "opponent_name": player_name
                        });
                        let _ = room.host.tx.send(Message::Text(to_host.to_string().into()));

                        let to_guest = serde_json::json!({
                            "type": "ROOM_JOINED",
                            "room_code": code,
                            "role": "guest",
                            "player_name": player_name,
                            "opponent_name": host_name
                        });
                        let _ = tx.send(Message::Text(to_guest.to_string().into()));

                        current_room = Some(code.clone());
                        current_role = Some("guest".to_string());
                        rooms_guard.insert(code, room);
                    } else {
                        *qm_guard = Some(QuickMatchEntry {
                            session: PlayerSession {
                                id: player_id.to_string(),
                                name: player_name.to_string(),
                                avatar: avatar.to_string(),
                                elo,
                                hp: 100,
                                ready: false,
                                lineup: vec![],
                                tx: tx.clone(),
                            },
                        });
                        let res = serde_json::json!({ "type": "WAITING_FOR_MATCH" });
                        let _ = tx.send(Message::Text(res.to_string().into()));
                    }
                }

                "CANCEL_MATCH" => {
                    let mut qm_guard = state.quick_match.write().await;
                    if let Some(ref entry) = *qm_guard {
                        if entry.session.tx.same_channel(&tx) {
                            *qm_guard = None;
                            let res = serde_json::json!({ "type": "MATCH_CANCELED" });
                            let _ = tx.send(Message::Text(res.to_string().into()));
                        }
                    }
                }

                "PLAYER_READY" => {
                    let lineup_raw = parsed.get("lineup").and_then(|v| v.as_array());
                    let mut lineup: Vec<UnitData> = vec![];
                    if let Some(arr) = lineup_raw {
                        for item in arr {
                            if let (Some(col), Some(row), Some(class), Some(star)) = (
                                item.get("col").and_then(|v| v.as_u64()),
                                item.get("row").and_then(|v| v.as_u64()),
                                item.get("class").and_then(|v| v.as_str()),
                                item.get("star_level").and_then(|v| v.as_u64()),
                            ) {
                                lineup.push(UnitData {
                                    col: col as usize,
                                    row: row as usize,
                                    class: class.to_string(),
                                    star_level: star as u8,
                                });
                            }
                        }
                    }

                    if let Some(code) = &current_room {
                        let mut rooms_guard = state.rooms.write().await;
                        if let Some(room) = rooms_guard.get_mut(code) {
                            if current_role.as_deref() == Some("host") {
                                room.host.ready = true;
                                room.host.lineup = lineup;
                            } else if current_role.as_deref() == Some("guest") {
                                if let Some(ref mut g) = room.guest {
                                    g.ready = true;
                                    g.lineup = lineup;
                                }
                            }

                            // If both are ready, start round
                            let both_ready = room.host.ready
                                && room.guest.as_ref().map(|g| g.ready).unwrap_or(false);

                            if both_ready {
                                room.host.ready = false;
                                if let Some(ref mut g) = room.guest {
                                    g.ready = false;
                                }

                                let host_lineup = room.host.lineup.clone();
                                let guest_lineup = room
                                    .guest
                                    .as_ref()
                                    .map(|g| g.lineup.clone())
                                    .unwrap_or_default();
                                let host_hp = room.host.hp;
                                let guest_hp = room.guest.as_ref().map(|g| g.hp).unwrap_or(0);
                                let round = room.round;

                                // Send START_ROUND to host (opponent is guest)
                                let to_host = serde_json::json!({
                                    "type": "START_ROUND",
                                    "round": round,
                                    "opponent_lineup": guest_lineup,
                                    "player_hp": host_hp,
                                    "opponent_hp": guest_hp
                                });
                                let _ =
                                    room.host.tx.send(Message::Text(to_host.to_string().into()));

                                // Send START_ROUND to guest (opponent is host)
                                if let Some(ref g) = room.guest {
                                    let to_guest = serde_json::json!({
                                        "type": "START_ROUND",
                                        "round": round,
                                        "opponent_lineup": host_lineup,
                                        "player_hp": guest_hp,
                                        "opponent_hp": host_hp
                                    });
                                    let _ = g.tx.send(Message::Text(to_guest.to_string().into()));
                                }
                                println!("[RUST WS] Started round {} in room {}", round, code);
                            }
                        }
                    }
                }

                "BATTLE_FINISHED" => {
                    let winner_role = parsed
                        .get("winner_role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("draw");
                    let survivors = parsed
                        .get("player_survivors")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1) as i32;

                    if let Some(code) = &current_room {
                        let mut rooms_guard = state.rooms.write().await;
                        if let Some(room) = rooms_guard.get_mut(code) {
                            let damage = 10 + survivors * 3;
                            if winner_role == "host" {
                                if let Some(ref mut g) = room.guest {
                                    g.hp = (g.hp - damage).max(0);
                                }
                            } else if winner_role == "guest" {
                                room.host.hp = (room.host.hp - damage).max(0);
                            }

                            let host_hp = room.host.hp;
                            let guest_hp = room.guest.as_ref().map(|g| g.hp).unwrap_or(0);

                            // Send HP updates
                            let _ = room.host.tx.send(Message::Text(
                                serde_json::json!({
                                    "type": "UPDATE_MATCH_HP",
                                    "player_hp": host_hp,
                                    "opponent_hp": guest_hp,
                                    "damage_dealt": damage
                                })
                                .to_string()
                                .into(),
                            ));

                            if let Some(ref g) = room.guest {
                                let _ = g.tx.send(Message::Text(
                                    serde_json::json!({
                                        "type": "UPDATE_MATCH_HP",
                                        "player_hp": guest_hp,
                                        "opponent_hp": host_hp,
                                        "damage_dealt": damage
                                    })
                                    .to_string()
                                    .into(),
                                ));
                            }

                            // Check match end
                            if host_hp <= 0 || guest_hp <= 0 {
                                let winner_name = if host_hp > 0 {
                                    room.host.name.clone()
                                } else {
                                    room.guest
                                        .as_ref()
                                        .map(|g| g.name.clone())
                                        .unwrap_or_default()
                                };
                                let winner_id = if host_hp > 0 {
                                    room.host.id.clone()
                                } else {
                                    room.guest
                                        .as_ref()
                                        .map(|g| g.id.clone())
                                        .unwrap_or_default()
                                };

                                let end_msg = serde_json::json!({
                                    "type": "MATCH_END",
                                    "winner": winner_name,
                                    "gold_reward": 80,
                                    "consolation_gold": 15
                                })
                                .to_string();

                                let _ = room.host.tx.send(Message::Text(end_msg.clone().into()));
                                if let Some(ref g) = room.guest {
                                    let _ = g.tx.send(Message::Text(end_msg.into()));
                                }

                                // Update DB & Award Gold
                                let mut db = state.db.write().await;
                                db.record_match(
                                    &format!("m_{}", code),
                                    &room.host.id,
                                    &room.guest.as_ref().map(|g| g.id.as_str()).unwrap_or(""),
                                    Some(&winner_id),
                                    room.round,
                                );

                                rooms_guard.remove(code);
                                println!(
                                    "[RUST WS] Match ended in room {}. Winner: {} (+80G)",
                                    code, winner_name
                                );
                            } else {
                                room.round += 1;
                            }
                        }
                    }
                }

                _ => {}
            }
        }
    }

    // Cleanup on disconnect
    if let Some(code) = current_room {
        let mut rooms_guard = state.rooms.write().await;
        if let Some(room) = rooms_guard.get_mut(&code) {
            if current_role.as_deref() == Some("host") {
                if let Some(ref g) = room.guest {
                    let _ = g.tx.send(Message::Text(
                        serde_json::json!({ "type": "ERROR", "message": "Chủ phòng đã thoát trận đấu." })
                            .to_string()
                            .into(),
                    ));
                }
                rooms_guard.remove(&code);
            } else if current_role.as_deref() == Some("guest") {
                let _ = room.host.tx.send(Message::Text(
                    serde_json::json!({ "type": "ERROR", "message": "Đối thủ đã rời khỏi phòng." })
                        .to_string()
                        .into(),
                ));
                room.guest = None;
            }
        }
    }

    let mut qm_guard = state.quick_match.write().await;
    if let Some(ref entry) = *qm_guard {
        if entry.session.tx.same_channel(&tx) {
            *qm_guard = None;
        }
    }
}

// ==========================================\n// MAIN ENTRY POINT
// ==========================================
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let data_dir = PathBuf::from("data");
    let db_path = data_dir.join("game_db.json");
    let db = Database::new_with_adb(db_path).await;

    let state = Arc::new(AppState {
        db: Arc::new(RwLock::new(db)),
        rooms: Arc::new(RwLock::new(HashMap::new())),
        quick_match: Arc::new(RwLock::new(None)),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // REST & WebSocket Routes
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/auth/register", post(handle_register))
        .route("/api/auth/login", post(handle_login))
        .route("/api/user/profile", get(handle_profile))
        .route("/api/shop/buy_card", post(handle_buy_card))
        .route("/api/cards/upgrade", post(handle_upgrade_card))
        .route("/api/cards/sell", post(handle_sell_card))
        .route("/api/match/reward", post(handle_match_reward))
        .route("/api/leaderboard", get(handle_leaderboard))
        .route("/api/match/record", post(handle_record_match))
        .fallback_service(ServeDir::new("wasm_dist"))
        .layer(cors)
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!("=======================================================");
    println!(" 🦀 3v3 TACTICAL ARENA - RUST WEBSOCKET SERVER");
    println!(" 🌐 Server Listening on: http://{}", addr);
    println!(" ⚡ WebSocket Endpoint: ws://{}/ws", addr);
    println!(" 📦 Static Assets: wasm_dist");
    println!("=======================================================");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_economy_and_rules() {
        let temp_dir = std::env::temp_dir();
        let test_db_path = temp_dir.join(format!("test_db_{}.json", chrono_now()));
        let mut db = Database::new(&test_db_path);

        // 1. Register gives starter card + 100G
        let user = db.register("test_hero", "1234", "Hero Test", "knight").expect("register failed");
        assert_eq!(user.gold, 100);
        assert_eq!(user.cards.len(), 1);
        let starter = &user.cards[0];
        assert!(starter.is_starter);
        assert_eq!(starter.hero_class, "Knight");

        // 2. Starter card CANNOT be sold
        let sell_starter = db.sell_card("test_hero", &starter.id);
        assert!(sell_starter.is_err(), "Starter card must not be sellable");

        // 3. Buy Archer card for 50G
        let (user, new_card) = db.buy_card("test_hero", "Archer").expect("buy card failed");
        assert_eq!(user.gold, 50); // 100 - 50 = 50
        assert_eq!(user.cards.len(), 2);
        assert_eq!(new_card.hero_class, "Archer");
        assert!(!new_card.is_starter);

        // 4. Upgrade Archer card level (costs 1 * 20 = 20G)
        let (user, _msg) = db.upgrade_card("test_hero", &new_card.id, "level").expect("upgrade failed");
        assert_eq!(user.gold, 30); // 50 - 20 = 30
        let upgraded = user.cards.iter().find(|c| c.id == new_card.id).unwrap();
        assert_eq!(upgraded.level, 2);
        assert!(upgraded.hp_bonus > 0.0);

        // 5. Reward match (PvE victory +40G)
        let (user, earned) = db.reward_match("test_hero", "pve", true).expect("reward failed");
        assert_eq!(earned, 40);
        assert_eq!(user.gold, 70); // 30 + 40 = 70

        // 6. Sell the Archer card (non-starter is sellable)
        let (user, refund) = db.sell_card("test_hero", &new_card.id).expect("sell non-starter failed");
        assert!(refund > 0);
        assert_eq!(user.cards.len(), 1);
        assert_eq!(user.cards[0].id, starter.id); // only starter remains

        let _ = std::fs::remove_file(test_db_path);
    }
}
