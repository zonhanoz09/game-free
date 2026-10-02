use super::*;

pub struct Database {
    file_path: PathBuf,
    pub(crate) data: DatabaseData,
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
            user.ensure_valid_id_and_deck();
        }

        Self {
            file_path,
            data,
            adb: None,
        }
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
                    println!(
                        "[ORACLE AUTONOMOUS DB] Loaded {} users from Oracle Cloud ADB.",
                        users.len()
                    );
                    for u in users {
                        data.users.insert(u.username.clone(), u);
                    }
                    loaded_from_adb = true;
                }
                Err(e) => {
                    eprintln!(
                        "[ORACLE ADB WARNING] Could not load from ADB ({}). Falling back to local disk.",
                        e
                    );
                }
            }
        }

        if !loaded_from_adb && file_path.exists() {
            if let Ok(content) = fs::read_to_string(&file_path) {
                if let Ok(local_data) = serde_json::from_str::<DatabaseData>(&content) {
                    data = local_data;
                    println!(
                        "[LOCAL DB] Loaded {} users and {} matches from local file.",
                        data.users.len(),
                        data.matches.len()
                    );
                }
            }
        }

        for user in data.users.values_mut() {
            user.ensure_valid_id_and_deck();
        }

        Self {
            file_path,
            data,
            adb,
        }
    }

    pub fn set_adb_client(&mut self, adb: OracleAdbClient) {
        self.adb = Some(adb);
    }

    pub async fn init_from_adb(&mut self) -> Result<usize, String> {
        let adb = match &self.adb {
            Some(c) => c.clone(),
            None => return Ok(0),
        };

        println!("[ORACLE ADB] Loading players and cards from cloud database...");
        let cloud_users = adb.load_all_users().await?;
        let count = cloud_users.len();
        println!(
            "[ORACLE ADB] Successfully fetched {} players from Oracle ADB.",
            count
        );

        for mut user in cloud_users {
            user.ensure_valid_id_and_deck();
            let key = user.username.to_lowercase();
            self.data.users.insert(key, user);
        }

        self.save();
        Ok(count)
    }

    fn persist_user_adb(&self, user: &User) {
        if let Some(adb) = &self.adb {
            let adb = adb.clone();
            let user = user.clone();
            tokio::spawn(async move {
                if let Err(e) = adb.save_user(&user).await {
                    eprintln!(
                        "[ORACLE ADB ERROR] Failed to save user {}: {}",
                        user.username, e
                    );
                }
            });
        }
    }

    fn persist_match_adb(&self, m: &MatchRecord) {
        if let Some(adb) = &self.adb {
            let adb = adb.clone();
            let m = m.clone();
            tokio::spawn(async move {
                if let Err(e) = adb.record_match(&m).await {
                    eprintln!(
                        "[ORACLE ADB ERROR] Failed to record match {}: {}",
                        m.match_id, e
                    );
                }
            });
        }
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(&self.file_path, json);
        }
    }

    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn register(
        &mut self,
        username: &str,
        password: &str,
        display_name: &str,
        avatar: &str,
    ) -> Result<User, String> {
        let clean = username.trim().to_lowercase();
        if clean.is_empty() {
            return Err("Tên đăng nhập không được để trống!".to_string());
        }
        if clean.len() < 3 || clean.len() > 30 {
            return Err("Tên đăng nhập phải từ 3 đến 30 ký tự!".to_string());
        }
        if password.len() < 4 {
            return Err("Mật khẩu phải có ít nhất 4 ký tự!".to_string());
        }
        if self.data.users.contains_key(&clean) {
            return Err("Tài khoản đã tồn tại, vui lòng chọn tên khác!".to_string());
        }

        let dname = if display_name.trim().is_empty() {
            username.trim()
        } else {
            display_name.trim()
        };
        let av = if avatar.trim().is_empty() {
            "knight"
        } else {
            avatar.trim()
        };

        let now = chrono_now();
        let starter_card = create_starter_card(&clean, av);
        let user_id = format!("p_{}", clean);
        let avatar_id = format!("avatar_{}", av);

        let starter_deck = PlayerDeck {
            id: format!("deck_{}_starter", clean),
            player_id: user_id.clone(),
            deck_name: "Bộ Bài Tiên Phong".to_string(),
            hero_class: starter_card.hero_class.clone(),
            cardback_id: "cb_classic".to_string(),
            cards_data: vec![DeckCardEntry {
                id: starter_card.id.clone(),
                count: 1,
                position: Some(0),
            }],
            is_valid: true,
            validation_errors: vec![],
            updated_at: now.clone(),
        };

        let user = User {
            id: user_id,
            username: clean.clone(),
            display_name: dname.to_string(),
            avatar: av.to_string(),
            avatar_id,
            cardback_id: "cb_classic".to_string(),
            board_skin: "board_arena".to_string(),
            password_hash: Self::hash_password(password),
            level: 1,
            current_exp: 0,
            elo: 1000,
            wins: 0,
            losses: 0,
            matches: 0,
            gold: 100,
            gems: 10,
            rank_tier: "Đồng".to_string(),
            rank_division: 3,
            rank_stars: 0,
            mmr: 1200,
            win_streak: 0,
            best_streak: 0,
            cards: vec![starter_card],
            decks: vec![starter_deck],
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
        user.ensure_valid_id_and_deck();

        let res = user.clone();
        self.save();
        self.persist_user_adb(&res);
        Ok(res)
    }

    pub fn get_user(&self, username: &str) -> Option<User> {
        self.data.users.get(&username.to_lowercase()).cloned()
    }

    pub fn customize_profile(
        &mut self,
        username: &str,
        display_name: Option<String>,
        avatar_id: Option<String>,
        cardback_id: Option<String>,
        board_skin: Option<String>,
    ) -> Result<User, String> {
        let clean = username.trim().to_lowercase();
        let user = self
            .data
            .users
            .get_mut(&clean)
            .ok_or_else(|| "Người chơi không tồn tại!".to_string())?;

        if let Some(dn) = display_name {
            let t = dn.trim();
            if !t.is_empty() {
                user.display_name = t.to_string();
            }
        }
        if let Some(aid) = avatar_id {
            let t = aid.trim();
            if !t.is_empty() {
                user.avatar_id = t.to_string();
                if let Some(base) = t.strip_prefix("avatar_") {
                    user.avatar = base.to_string();
                }
            }
        }
        if let Some(cb) = cardback_id {
            let t = cb.trim();
            if !t.is_empty() {
                user.cardback_id = t.to_string();
            }
        }
        if let Some(bs) = board_skin {
            let t = bs.trim();
            if !t.is_empty() {
                user.board_skin = t.to_string();
            }
        }

        let res = user.clone();
        self.save();
        self.persist_user_adb(&res);
        Ok(res)
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
            quantity: 1,
            is_foil: false,
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
                card.hp_bonus += 50.0;
                card.atk_bonus += 12.0;
                format!(
                    "Đột phá thành công lên {}★! (+50 HP, +12 ATK)",
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

    pub fn foil_card(&mut self, username: &str, card_id: &str) -> Result<(User, String), String> {
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

        if card.is_foil {
            return Err("Thẻ bài này đã là Thẻ Tinh Anh (Foil Hologram)!".to_string());
        }

        let gem_cost = 50;
        let gold_cost = 150;
        if user.gems >= gem_cost {
            user.gems -= gem_cost;
        } else if user.gold >= gold_cost {
            user.gold -= gold_cost;
        } else {
            return Err(format!(
                "Bạn cần {} Gems hoặc {} Vàng để nâng cấp Thẻ Tinh Anh Foil!",
                gem_cost, gold_cost
            ));
        }

        card.is_foil = true;
        card.hp_bonus += 20.0;
        card.atk_bonus += 6.0;

        let msg = format!(
            "✨ Chúc mừng! Lá '{}' đã trở thành THẺ TINH ANH (Foil Hologram) (+20 HP, +6 ATK)!",
            card.name
        );
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
            .ok_or("Không tìm thấy thẻ bài cần bán!")?;

        if user.cards[idx].is_starter {
            return Err("Không thể bán Thẻ Bài Khởi Đầu!".to_string());
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

    pub fn save_deck(
        &mut self,
        username: &str,
        deck_id: Option<String>,
        deck_name: String,
        hero_class: String,
        cardback_id: Option<String>,
        cards: Vec<DeckCardEntry>,
    ) -> Result<(User, PlayerDeck), String> {
        let clean = username.trim().to_lowercase();
        let user = self
            .data
            .users
            .get_mut(&clean)
            .ok_or("Người chơi không tồn tại!")?;

        let (is_valid, errors) = validate_deck(&deck_name, &hero_class, &cards, &user.cards);
        let d_id = deck_id.unwrap_or_else(|| format!("deck_{}_{}", clean, chrono_now()));
        let cb = cardback_id.unwrap_or_else(|| user.cardback_id.clone());

        let deck = PlayerDeck {
            id: d_id.clone(),
            player_id: user.id.clone(),
            deck_name,
            hero_class,
            cardback_id: cb,
            cards_data: cards,
            is_valid,
            validation_errors: errors,
            updated_at: chrono_now(),
        };

        if let Some(pos) = user.decks.iter().position(|d| d.id == d_id) {
            user.decks[pos] = deck.clone();
        } else {
            if user.decks.len() >= 6 {
                return Err("Bạn đã đạt giới hạn tối đa 6 bộ bài!".to_string());
            }
            user.decks.push(deck.clone());
        }

        let res_user = user.clone();
        self.save();
        self.persist_user_adb(&res_user);
        Ok((res_user, deck))
    }

    pub fn delete_deck(&mut self, username: &str, deck_id: &str) -> Result<User, String> {
        let clean = username.trim().to_lowercase();
        let user = self
            .data
            .users
            .get_mut(&clean)
            .ok_or("Người chơi không tồn tại!")?;

        if user.decks.len() <= 1 {
            return Err("Không thể xóa! Cần giữ lại ít nhất 1 bộ bài.".to_string());
        }

        let idx = user
            .decks
            .iter()
            .position(|d| d.id == deck_id)
            .ok_or("Không tìm thấy bộ bài!")?;
        user.decks.remove(idx);

        let res_user = user.clone();
        self.save();
        self.persist_user_adb(&res_user);
        Ok(res_user)
    }

    pub fn reward_match(
        &mut self,
        username: &str,
        mode: &str,
        win: bool,
    ) -> Result<(User, u32, u64, u32, bool, String), String> {
        let clean = username.trim().to_lowercase();
        let user = self
            .data
            .users
            .get_mut(&clean)
            .ok_or("Người chơi không tồn tại!")?;

        let (gold_earned, exp_earned, gems_earned) = if win {
            if mode == "pvp" {
                (80, 120, 5)
            } else {
                (45, 60, 2)
            }
        } else {
            (15, 30, 0)
        };

        user.gold += gold_earned;
        user.gems += gems_earned;

        let (leveled_up, _) = user.add_exp(exp_earned);
        let mut promo_msg = String::new();

        if mode == "pvp" {
            if win {
                let (_, msg) = user.record_win_rating();
                promo_msg = msg;
            } else {
                user.record_loss_rating();
            }
        } else {
            user.matches += 1;
            if win {
                user.wins += 1;
                user.win_streak += 1;
                if user.win_streak > user.best_streak {
                    user.best_streak = user.win_streak;
                }
            } else {
                user.losses += 1;
                user.win_streak = 0;
            }
        }

        let res_user = user.clone();
        self.save();
        self.persist_user_adb(&res_user);
        Ok((
            res_user,
            gold_earned,
            exp_earned,
            gems_earned,
            leveled_up,
            promo_msg,
        ))
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
                if is_host_win {
                    h.gold += 80;
                    h.gems += 5;
                    h.add_exp(120);
                    h.record_win_rating();
                } else {
                    h.gold += 15;
                    h.add_exp(30);
                    h.record_loss_rating();
                }
                let h_clone = h.clone();
                self.persist_user_adb(&h_clone);
            }
            if let Some(g) = self.data.users.get_mut(&guest_user.to_lowercase()) {
                if !is_host_win {
                    g.gold += 80;
                    g.gems += 5;
                    g.add_exp(120);
                    g.record_win_rating();
                } else {
                    g.gold += 15;
                    g.add_exp(30);
                    g.record_loss_rating();
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
        list.sort_by(|a, b| b.mmr.cmp(&a.mmr).then_with(|| b.wins.cmp(&a.wins)));
        list.truncate(limit);
        list
    }
}
