use super::*;

fn default_battle_slots() -> u8 {
    3
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
    #[serde(default = "default_quantity")]
    pub quantity: u32,
    #[serde(default)]
    pub is_foil: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeckCardEntry {
    pub id: String,
    pub count: u32,
    #[serde(default)]
    pub position: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerDeck {
    pub id: String,
    pub player_id: String,
    pub deck_name: String,
    pub hero_class: String,
    pub cardback_id: String,
    pub cards_data: Vec<DeckCardEntry>,
    pub is_valid: bool,
    #[serde(default)]
    pub validation_errors: Vec<String>,
    pub updated_at: String,
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
    #[serde(default = "default_user_id")]
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub avatar: String,
    #[serde(default = "default_avatar_id")]
    pub avatar_id: String,
    #[serde(default = "default_cardback_id")]
    pub cardback_id: String,
    #[serde(default = "default_board_skin")]
    pub board_skin: String,
    #[serde(default)]
    pub password_hash: String,
    #[serde(default = "default_level")]
    pub level: u32,
    #[serde(default)]
    pub current_exp: u64,
    pub elo: i32,
    pub wins: u32,
    pub losses: u32,
    pub matches: u32,
    #[serde(default = "default_gold")]
    pub gold: u32,
    #[serde(default = "default_gems")]
    pub gems: u32,
    /// Number of cards that may start a battle. New accounts begin with one.
    #[serde(default = "default_battle_slots")]
    pub battle_slots: u8,
    #[serde(default = "default_rank_tier")]
    pub rank_tier: String,
    #[serde(default = "default_rank_division")]
    pub rank_division: u8,
    #[serde(default)]
    pub rank_stars: u8,
    #[serde(default = "default_mmr")]
    pub mmr: i32,
    #[serde(default)]
    pub win_streak: u32,
    #[serde(default)]
    pub best_streak: u32,
    #[serde(default)]
    pub cards: Vec<UserCard>,
    #[serde(default)]
    pub decks: Vec<PlayerDeck>,
    #[serde(default)]
    pub items: Vec<UserItem>,
    pub created_at: String,
    pub last_login: String,
}

impl User {
    pub fn ensure_battle_slots(&mut self) {
        if self.battle_slots < 3 { self.battle_slots = 3; }
        self.battle_slots = self.battle_slots.clamp(3, 5);
    }
    pub fn sanitized(mut self) -> Self {
        self.password_hash.clear();
        self
    }

    pub fn exp_needed_for_next_level(&self) -> u64 {
        (self.level as u64) * 100
    }

    pub fn add_exp(&mut self, exp_gain: u64) -> (bool, u32) {
        self.current_exp += exp_gain;
        let mut leveled_up = false;
        let mut count = 0;
        while self.level < 100 {
            let needed = self.exp_needed_for_next_level();
            if self.current_exp >= needed {
                self.current_exp -= needed;
                self.level += 1;
                self.gold += self.level * 40;
                self.gems += 5;
                leveled_up = true;
                count += 1;
            } else {
                break;
            }
        }
        (leveled_up, count)
    }

    pub fn record_win_rating(&mut self) -> (bool, String) {
        self.wins += 1;
        self.matches += 1;
        self.win_streak += 1;
        if self.win_streak > self.best_streak {
            self.best_streak = self.win_streak;
        }
        self.mmr += 25;

        let stars_to_add = if self.win_streak >= 3 { 2 } else { 1 };
        self.rank_stars += stars_to_add;

        let max_stars = match self.rank_tier.as_str() {
            "Đồng" | "Bronze" | "Bạc" | "Silver" => 3,
            "Vàng" | "Gold" | "Bạch Kim" | "Platinum" => 4,
            "Kim Cương" | "Diamond" => 5,
            _ => 5,
        };

        let mut promoted = false;
        let mut promo_msg = String::new();
        if self.rank_tier != "Cao Thủ" && self.rank_stars >= max_stars {
            self.rank_stars -= max_stars;
            if self.rank_division > 1 {
                self.rank_division -= 1;
                promo_msg = format!("Thăng cấp lên {} {}!", self.rank_tier, self.rank_division);
                promoted = true;
            } else {
                let next_tier = match self.rank_tier.as_str() {
                    "Đồng" | "Bronze" => "Bạc",
                    "Bạc" | "Silver" => "Vàng",
                    "Vàng" | "Gold" => "Bạch Kim",
                    "Bạch Kim" | "Platinum" => "Kim Cương",
                    "Kim Cương" | "Diamond" => "Cao Thủ",
                    _ => "Cao Thủ",
                };
                self.rank_tier = next_tier.to_string();
                self.rank_division = 3;
                promo_msg = format!(
                    "🎉 Chúc mừng! Bạn đã thăng hạng lên bậc {}!",
                    self.rank_tier
                );
                promoted = true;
            }
        }
        (promoted, promo_msg)
    }

    pub fn record_loss_rating(&mut self) {
        self.losses += 1;
        self.matches += 1;
        self.win_streak = 0;
        self.mmr = (self.mmr - 18).max(600);
        if self.rank_tier != "Cao Thủ" && self.rank_stars > 0 {
            self.rank_stars -= 1;
        }
    }

    pub fn get_rank_display(&self) -> String {
        if self.rank_tier == "Cao Thủ" || self.rank_tier == "Master" {
            "CAO THỦ".to_string()
        } else {
            let div_roman = match self.rank_division {
                1 => "I",
                2 => "II",
                _ => "III",
            };
            format!("{} {}", self.rank_tier.to_uppercase(), div_roman)
        }
    }

    pub fn get_stars_display(&self) -> String {
        let max_stars = match self.rank_tier.as_str() {
            "Đồng" | "Bronze" | "Bạc" | "Silver" => 3,
            "Vàng" | "Gold" | "Bạch Kim" | "Platinum" => 4,
            "Kim Cương" | "Diamond" => 5,
            _ => 5,
        };
        let stars = self.rank_stars.min(max_stars);
        let filled = "★".repeat(stars as usize);
        let empty = "☆".repeat((max_stars - stars) as usize);
        format!("{}{}", filled, empty)
    }

    pub fn win_rate_percent(&self) -> f32 {
        if self.matches == 0 {
            0.0
        } else {
            ((self.wins as f32) / (self.matches as f32)) * 100.0
        }
    }

    pub fn ensure_valid_id_and_deck(&mut self) {
        if self.id.is_empty() {
            self.id = format!("p_{}", self.username);
        }
        if self.avatar_id.is_empty() {
            self.avatar_id = format!("avatar_{}", self.avatar);
        }
        if self.cardback_id.is_empty() {
            self.cardback_id = "cb_classic".to_string();
        }
        if self.board_skin.is_empty() {
            self.board_skin = "board_arena".to_string();
        }
        if self.level == 0 {
            self.level = 1;
        }
        if self.gems == 0 {
            self.gems = 10;
        }
        if self.rank_tier.is_empty() {
            self.rank_tier = "Đồng".to_string();
            self.rank_division = 3;
            self.rank_stars = 0;
            self.mmr = 1200;
        }

        let roster_is_legacy = self
            .cards
            .iter()
            .any(|c| !is_three_kingdoms_general(&c.hero_class));
        if roster_is_legacy || self.cards.is_empty() {
            self.cards = create_starter_cards(&self.username);
            self.decks.clear();
        } else if self.cards.len() < 3 {
            let defaults = create_starter_cards(&self.username);
            for d in defaults {
                if !self.cards.iter().any(|c| c.hero_class == d.hero_class) {
                    self.cards.push(d);
                }
            }
        }

        if self.battle_slots < 3 {
            self.battle_slots = 3;
        }
        self.battle_slots = self.battle_slots.clamp(3, 5);

        for c in &mut self.cards {
            if c.quantity == 0 {
                c.quantity = 1;
            }
        }

        if self.decks.is_empty() && !self.cards.is_empty() {
            let deck_cards: Vec<DeckCardEntry> = self
                .cards
                .iter()
                .take(3)
                .enumerate()
                .map(|(idx, c)| DeckCardEntry {
                    id: c.id.clone(),
                    count: 1,
                    position: Some(match idx {
                        0 => 0,
                        1 => 3,
                        _ => 6,
                    }),
                })
                .collect();

            self.decks.push(PlayerDeck {
                id: format!("deck_{}_starter", self.username),
                player_id: self.id.clone(),
                deck_name: "Bộ Bài Tiên Phong".to_string(),
                hero_class: "Tactical".to_string(),
                cardback_id: self.cardback_id.clone(),
                cards_data: deck_cards,
                is_valid: true,
                validation_errors: vec![],
                updated_at: chrono_now(),
            });
        }
    }
}

pub fn validate_deck(
    deck_name: &str,
    hero_class: &str,
    cards: &[DeckCardEntry],
    user_cards: &[UserCard],
) -> (bool, Vec<String>) {
    let mut errors = Vec::new();
    let total_cards: u32 = cards.iter().map(|c| c.count).sum();

    if deck_name.trim().is_empty() {
        errors.push("Tên bộ bài không được để trống!".to_string());
    }

    if total_cards < 1 {
        errors.push("Bộ bài cần tối thiểu 1 lá bài!".to_string());
    }
    if total_cards > 5 {
        errors.push(format!(
            "Đội hình ra trận tối đa 5 thẻ bài (hiện có {} thẻ).",
            total_cards
        ));
    }

    for entry in cards {
        if entry.count == 0 {
            continue;
        }
        match user_cards.iter().find(|c| c.id == entry.id) {
            Some(card) => {
                let max_copies = if card.is_starter {
                    1
                } else {
                    card.quantity.clamp(1, 2)
                };
                if entry.count > max_copies {
                    errors.push(format!(
                        "Lá '{}' vượt quá giới hạn (tối đa {} bản sao, đã chọn {}).",
                        card.name, max_copies, entry.count
                    ));
                }
                if hero_class != "Tactical"
                    && card.hero_class != hero_class
                    && card.hero_class != "Neutral"
                {
                    errors.push(format!(
                        "Lá '{}' ({}) không phù hợp với Tướng hệ {}.",
                        card.name, card.hero_class, hero_class
                    ));
                }
            }
            None => {
                errors.push(format!("Bạn không sở hữu thẻ ID '{}' trong kho!", entry.id));
            }
        }
    }

    (errors.is_empty(), errors)
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
    #[serde(default)]
    pub master_rarities: HashMap<String, game_data_schema::RarityConfig>,
    #[serde(default)]
    pub master_lines: Vec<game_data_schema::FormationLineConfig>,
    #[serde(default)]
    pub master_skills: Vec<game_data_schema::SkillRow>,
    #[serde(default)]
    pub master_templates: Vec<game_data_schema::CardTemplateRow>,
    #[serde(default)]
    pub master_effects: Vec<game_data_schema::StatusEffectRow>,
    #[serde(default)]
    pub master_cells: Vec<game_data_schema::BoardCellConfigRow>,
    #[serde(default)]
    pub player_formations: HashMap<String, Vec<game_data_schema::PlayerFormationRow>>,
    #[serde(default)]
    pub shop_cards: Vec<ShopCardItem>,
    #[serde(default)]
    pub master_template_skills: Vec<game_data_schema::CardTemplateSkillRow>,
    #[serde(default)]
    pub master_synergies: Vec<game_data_schema::FormationSynergyRow>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ShopCardItem {
    pub card_id: String,
    pub name: String,
    pub icon: String,
    pub faction: String,
    pub role: String,
    pub rarity: String,
    pub price: u32,
    pub is_available: bool,
    pub description: String,
    pub skill_name: String,
}

pub fn default_shop_cards() -> Vec<ShopCardItem> {
    vec![
        ShopCardItem {
            card_id: "zhao_yun".to_string(),
            name: "Triệu Vân".to_string(),
            icon: "🐉".to_string(),
            faction: "SHU".to_string(),
            role: "Đấu Sĩ Cơ Động".to_string(),
            rarity: "Rare".to_string(),
            price: 55,
            is_available: true,
            description: "Xông vào mục tiêu yếu máu nhất, tăng 40% Né đòn và phản kích khi né thành công.".to_string(),
            skill_name: "Thất Tiến Thất Xuất".to_string(),
        },
        ShopCardItem {
            card_id: "huang_zhong".to_string(),
            name: "Hoàng Trung".to_string(),
            icon: "🏹".to_string(),
            faction: "SHU".to_string(),
            role: "Xạ Thủ Tỉa Xa".to_string(),
            rarity: "Epic".to_string(),
            price: 55,
            is_available: true,
            description: "Bắn xuyên mục tiêu đứng sau khiên; sát thương tăng theo khoảng cách.".to_string(),
            skill_name: "Bách Bộ Xuyên Dương".to_string(),
        },
        ShopCardItem {
            card_id: "zhuge_liang".to_string(),
            name: "Gia Cát Lượng".to_string(),
            icon: "🪶".to_string(),
            faction: "SHU".to_string(),
            role: "Pháp Sư Khống Chế".to_string(),
            rarity: "Legendary".to_string(),
            price: 65,
            is_available: true,
            description: "Thổi lùi hàng trước địch về hàng giữa và gây choáng 1 hiệp.".to_string(),
            skill_name: "Đông Phong Triệu Hoán".to_string(),
        },
        ShopCardItem {
            card_id: "cao_cao".to_string(),
            name: "Tào Tháo".to_string(),
            icon: "👑".to_string(),
            faction: "WEI".to_string(),
            role: "Chủ Lực Buffer".to_string(),
            rarity: "Legendary".to_string(),
            price: 70,
            is_available: true,
            description: "Tăng công tướng Ngụy xung quanh và giảm Nộ của địch có công cao nhất.".to_string(),
            skill_name: "Hiệp Thiên Tử".to_string(),
        },
        ShopCardItem {
            card_id: "dian_wei".to_string(),
            name: "Điển Vi".to_string(),
            icon: "🛡️".to_string(),
            faction: "WEI".to_string(),
            role: "Siêu Đỡ Đòn".to_string(),
            rarity: "Epic".to_string(),
            price: 60,
            is_available: true,
            description: "Khiêu khích ba ô đối diện và phản lại 25% sát thương nhận vào.".to_string(),
            skill_name: "Thiết Kích Chắn Cửa".to_string(),
        },
        ShopCardItem {
            card_id: "guo_jia".to_string(),
            name: "Quách Gia".to_string(),
            icon: "❄️".to_string(),
            faction: "WEI".to_string(),
            role: "Pháp Sư Băng".to_string(),
            rarity: "Epic".to_string(),
            price: 65,
            is_available: true,
            description: "Đóng băng địch có Nộ cao nhất và khóa thanh Nộ trong 2 lượt.".to_string(),
            skill_name: "Thập Thắng Thập Bại".to_string(),
        },
        ShopCardItem {
            card_id: "sun_ce".to_string(),
            name: "Tôn Sách".to_string(),
            icon: "⚔️".to_string(),
            faction: "WU".to_string(),
            role: "Đấu Sĩ Càn Quét".to_string(),
            rarity: "Rare".to_string(),
            price: 55,
            is_available: true,
            description: "Quét hình chữ T và hút 20% công của mục tiêu trúng đòn.".to_string(),
            skill_name: "Bá Vương Tả Đãng".to_string(),
        },
        ShopCardItem {
            card_id: "lu_xun".to_string(),
            name: "Lục Tốn".to_string(),
            icon: "🔥".to_string(),
            faction: "WU".to_string(),
            role: "Pháp Sư Thiêu Đốt".to_string(),
            rarity: "Legendary".to_string(),
            price: 65,
            is_available: true,
            description: "Đặt ấn lửa lên một tuyến; mục tiêu bị đánh sẽ phát nổ lan trên/dưới.".to_string(),
            skill_name: "Hỏa Thiêu Liên Doanh".to_string(),
        },
        ShopCardItem {
            card_id: "da_qiao_xiao_qiao".to_string(),
            name: "Đại Kiều & Tiểu Kiều".to_string(),
            icon: "🌸".to_string(),
            faction: "WU".to_string(),
            role: "Hồi Máu Hóa Giải".to_string(),
            rarity: "Legendary".to_string(),
            price: 60,
            is_available: true,
            description: "Hồi máu theo công cho hàng sau và xóa mọi hiệu ứng bất lợi.".to_string(),
            skill_name: "Quốc Sắc Lưu Hương".to_string(),
        },
        ShopCardItem {
            card_id: "zhang_he_yan_liang".to_string(),
            name: "Trương Cáp & Nhan Lương".to_string(),
            icon: "🗡️".to_string(),
            faction: "QUN".to_string(),
            role: "Sát Thủ Hậu Tuyến".to_string(),
            rarity: "Epic".to_string(),
            price: 65,
            is_available: true,
            description: "Ngay hiệp 1 nhảy ra sau xạ thủ/pháp sư địch để dồn sát thương.".to_string(),
            skill_name: "Tập Kích Hậu Tuyến".to_string(),
        },
        ShopCardItem {
            card_id: "hua_tuo".to_string(),
            name: "Hoa Đà".to_string(),
            icon: "⚕️".to_string(),
            faction: "QUN".to_string(),
            role: "Thần Y Hồi Sinh".to_string(),
            rarity: "Legendary".to_string(),
            price: 75,
            is_available: true,
            description: "Hồi máu diện rộng và có 30% cơ hội hồi sinh đồng minh vừa tử trận với 40% HP.".to_string(),
            skill_name: "Thanh Nang Di Thư".to_string(),
        },
        ShopCardItem {
            card_id: "jia_xu".to_string(),
            name: "Giả Hủ".to_string(),
            icon: "☠️".to_string(),
            faction: "QUN".to_string(),
            role: "Thuật Sĩ Độc".to_string(),
            rarity: "Epic".to_string(),
            price: 65,
            is_available: true,
            description: "Đầu độc toàn bộ địch, giảm 50% khả năng hồi phục máu.".to_string(),
            skill_name: "Loạn Vũ Độc Kế".to_string(),
        },
    ]
}

impl DatabaseData {
    pub fn ensure_master_data(&mut self) {
        if self.master_rarities.is_empty() {
            self.master_rarities = game_data_schema::default_rarity_configs();
        }
        if self.master_lines.is_empty() {
            self.master_lines = game_data_schema::default_formation_line_configs();
        }
        if self.master_skills.is_empty() {
            self.master_skills = game_data_schema::default_master_skills();
        } else {
            for default_skill in game_data_schema::default_master_skills() {
                if !self.master_skills.iter().any(|s| s.skill_id == default_skill.skill_id) {
                    self.master_skills.push(default_skill);
                }
            }
        }
        if self.master_templates.is_empty() {
            self.master_templates = game_data_schema::default_master_templates();
        } else {
            for default_tpl in game_data_schema::default_master_templates() {
                if !self.master_templates.iter().any(|t| t.card_template_id == default_tpl.card_template_id) {
                    self.master_templates.push(default_tpl);
                }
            }
        }
        if self.master_effects.is_empty() {
            self.master_effects = game_data_schema::default_master_effects();
        }
        if self.master_cells.is_empty() {
            self.master_cells = game_data_schema::default_master_cells();
        }
        if self.shop_cards.is_empty() {
            self.shop_cards = default_shop_cards();
        } else {
            for default_card in default_shop_cards() {
                if !self.shop_cards.iter().any(|c| c.card_id == default_card.card_id) {
                    self.shop_cards.push(default_card);
                }
            }
        }
        if self.master_template_skills.is_empty() {
            self.master_template_skills = game_data_schema::default_master_template_skills();
        } else {
            for def_ts in game_data_schema::default_master_template_skills() {
                if !self.master_template_skills.iter().any(|ts| ts.card_template_id == def_ts.card_template_id && ts.skill_id == def_ts.skill_id) {
                    self.master_template_skills.push(def_ts);
                }
            }
        }
        if self.master_synergies.is_empty() {
            self.master_synergies = game_data_schema::default_master_synergies();
        } else {
            for def_syn in game_data_schema::default_master_synergies() {
                if !self.master_synergies.iter().any(|s| s.synergy_id == def_syn.synergy_id) {
                    self.master_synergies.push(def_syn);
                }
            }
        }
    }

    pub fn apply_master_config(&mut self, key: &str, data: &str) {
        match key {
            "master_rarities" => {
                if let Ok(val) = serde_json::from_str(data) {
                    self.master_rarities = val;
                }
            }
            "master_lines" => {
                if let Ok(val) = serde_json::from_str(data) {
                    self.master_lines = val;
                }
            }
            "master_skills" => {
                if let Ok(val) = serde_json::from_str(data) {
                    self.master_skills = val;
                }
            }
            "master_templates" => {
                if let Ok(val) = serde_json::from_str(data) {
                    self.master_templates = val;
                }
            }
            "master_effects" => {
                if let Ok(val) = serde_json::from_str(data) {
                    self.master_effects = val;
                }
            }
            "master_cells" => {
                if let Ok(val) = serde_json::from_str(data) {
                    self.master_cells = val;
                }
            }
            "shop_cards" => {
                if let Ok(val) = serde_json::from_str(data) {
                    self.shop_cards = val;
                }
            }
            "master_template_skills" => {
                if let Ok(val) = serde_json::from_str(data) {
                    self.master_template_skills = val;
                }
            }
            "master_synergies" => {
                if let Ok(val) = serde_json::from_str(data) {
                    self.master_synergies = val;
                }
            }
            _ => {}
        }
    }
}

pub const THREE_KINGDOMS_GENERALS: [&str; 12] = [
    "zhao_yun",
    "huang_zhong",
    "zhuge_liang",
    "cao_cao",
    "dian_wei",
    "guo_jia",
    "sun_ce",
    "lu_xun",
    "da_qiao_xiao_qiao",
    "zhang_he_yan_liang",
    "hua_tuo",
    "jia_xu",
];

pub fn is_three_kingdoms_general(hero_class: &str) -> bool {
    THREE_KINGDOMS_GENERALS.contains(&hero_class)
}

pub fn general_display_name(hero_class: &str) -> &'static str {
    match hero_class {
        "zhao_yun" => "Triệu Vân",
        "huang_zhong" => "Hoàng Trung",
        "zhuge_liang" => "Gia Cát Lượng",
        "cao_cao" => "Tào Tháo",
        "dian_wei" => "Điển Vi",
        "guo_jia" => "Quách Gia",
        "sun_ce" => "Tôn Sách",
        "lu_xun" => "Lục Tốn",
        "da_qiao_xiao_qiao" => "Đại Kiều & Tiểu Kiều",
        "zhang_he_yan_liang" => "Trương Cáp & Nhan Lương",
        "hua_tuo" => "Hoa Đà",
        "jia_xu" => "Giả Hủ",
        _ => "Tướng Tam Quốc",
    }
}

pub fn create_starter_cards(username: &str) -> Vec<UserCard> {
    vec![
        UserCard {
            id: format!("general_{}_zhao_yun", username),
            hero_class: "zhao_yun".to_string(),
            name: general_display_name("zhao_yun").to_string(),
            star_level: 1,
            level: 1,
            is_starter: true,
            hp_bonus: 0.0,
            atk_bonus: 0.0,
            quantity: 1,
            is_foil: false,
        },
        UserCard {
            id: format!("general_{}_huang_zhong", username),
            hero_class: "huang_zhong".to_string(),
            name: general_display_name("huang_zhong").to_string(),
            star_level: 1,
            level: 1,
            is_starter: true,
            hp_bonus: 0.0,
            atk_bonus: 0.0,
            quantity: 1,
            is_foil: false,
        },
        UserCard {
            id: format!("general_{}_zhuge_liang", username),
            hero_class: "zhuge_liang".to_string(),
            name: general_display_name("zhuge_liang").to_string(),
            star_level: 1,
            level: 1,
            is_starter: true,
            hp_bonus: 0.0,
            atk_bonus: 0.0,
            quantity: 1,
            is_foil: false,
        },
    ]
}

#[allow(dead_code)]
pub fn create_starter_card(username: &str, _avatar: &str) -> UserCard {
    create_starter_cards(username)[0].clone()
}
