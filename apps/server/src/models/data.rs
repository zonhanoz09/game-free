use super::*;

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
            self.id = format!("p_{}", &self.username);
        }
        if self.avatar_id.is_empty() {
            self.avatar_id = format!("avatar_{}", &self.avatar);
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

        for c in &mut self.cards {
            if c.quantity == 0 {
                c.quantity = 1;
            }
        }

        if self.decks.is_empty() && !self.cards.is_empty() {
            let starter = &self.cards[0];
            self.decks.push(PlayerDeck {
                id: format!("deck_{}_starter", self.username),
                player_id: self.id.clone(),
                deck_name: "Bộ Bài Tiên Phong".to_string(),
                hero_class: starter.hero_class.clone(),
                cardback_id: self.cardback_id.clone(),
                cards_data: vec![DeckCardEntry {
                    id: starter.id.clone(),
                    count: 1,
                    position: Some(0),
                }],
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
    if total_cards > 30 {
        errors.push(format!(
            "Bộ bài tối đa 30 lá bài (hiện có {} lá).",
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
                    card.quantity.max(1).min(2)
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
        quantity: 1,
        is_foil: false,
    }
}
