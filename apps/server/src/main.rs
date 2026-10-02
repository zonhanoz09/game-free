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
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

fn default_gold() -> u32 {
    100
}
fn default_gems() -> u32 {
    10
}
fn default_level() -> u32 {
    1
}
fn default_rank_tier() -> String {
    "Đồng".to_string()
}
fn default_rank_division() -> u8 {
    3
}
fn default_mmr() -> i32 {
    1200
}
fn default_quantity() -> u32 {
    1
}
fn default_avatar_id() -> String {
    "avatar_knight".to_string()
}
fn default_cardback_id() -> String {
    "cb_classic".to_string()
}
fn default_board_skin() -> String {
    "board_arena".to_string()
}
fn default_user_id() -> String {
    format!("p_{}", chrono_now())
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
                promo_msg = format!("🎉 Chúc mừng! Bạn đã thăng hạng lên bậc {}!", self.rank_tier);
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
        errors.push(format!("Bộ bài tối đa 30 lá bài (hiện có {} lá).", total_cards));
    }

    for entry in cards {
        if entry.count == 0 {
            continue;
        }
        match user_cards.iter().find(|c| c.id == entry.id) {
            Some(card) => {
                let max_copies = if card.is_starter { 1 } else { card.quantity.max(1).min(2) };
                if entry.count > max_copies {
                    errors.push(format!(
                        "Lá '{}' vượt quá giới hạn (tối đa {} bản sao, đã chọn {}).",
                        card.name, max_copies, entry.count
                    ));
                }
                if card.hero_class != hero_class && card.hero_class != "Neutral" {
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
        if let Some(c_items) = all_cards_val.as_ref().and_then(|v| v["items"][0]["resultSet"]["items"].as_array()) {
            for row in c_items {
                let pid = row.get("player_id").or_else(|| row.get("PLAYER_ID")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let cid = row.get("card_id").or_else(|| row.get("CARD_ID")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let hclass = row.get("hero_class").or_else(|| row.get("HERO_CLASS")).and_then(|v| v.as_str()).unwrap_or("Knight").to_string();
                let cname = row.get("card_name").or_else(|| row.get("CARD_NAME")).and_then(|v| v.as_str()).unwrap_or("Chiến Binh").to_string();
                let qty = row.get("quantity").or_else(|| row.get("QUANTITY")).and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                let is_foil = row.get("is_foil").or_else(|| row.get("IS_FOIL")).and_then(|v| v.as_u64()).unwrap_or(0) == 1;
                let s_lvl = row.get("star_level").or_else(|| row.get("STAR_LEVEL")).and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                let c_lvl = row.get("card_level").or_else(|| row.get("CARD_LEVEL")).and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                let is_st = row.get("is_starter").or_else(|| row.get("IS_STARTER")).and_then(|v| v.as_u64()).unwrap_or(0) == 1;
                let hp_b = row.get("hp_bonus").or_else(|| row.get("HP_BONUS")).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let atk_b = row.get("atk_bonus").or_else(|| row.get("ATK_BONUS")).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

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
        if let Some(d_items) = all_decks_val.as_ref().and_then(|v| v["items"][0]["resultSet"]["items"].as_array()) {
            for row in d_items {
                let id = row.get("id").or_else(|| row.get("ID")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let pid = row.get("player_id").or_else(|| row.get("PLAYER_ID")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let dname = row.get("deck_name").or_else(|| row.get("DECK_NAME")).and_then(|v| v.as_str()).unwrap_or("Bộ Bài").to_string();
                let hclass = row.get("hero_class").or_else(|| row.get("HERO_CLASS")).and_then(|v| v.as_str()).unwrap_or("Knight").to_string();
                let cback = row.get("cardback_id").or_else(|| row.get("CARDBACK_ID")).and_then(|v| v.as_str()).unwrap_or("cb_classic").to_string();
                let is_valid = row.get("is_valid").or_else(|| row.get("IS_VALID")).and_then(|v| v.as_u64()).unwrap_or(1) == 1;
                let raw_data = row.get("cards_data").or_else(|| row.get("CARDS_DATA")).and_then(|v| v.as_str()).unwrap_or("[]");
                let cards_data: Vec<DeckCardEntry> = serde_json::from_str(raw_data).unwrap_or_default();

                decks_by_player.entry(pid.clone()).or_default().push(PlayerDeck {
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
                    row.get(key1).or_else(|| row.get(key2)).and_then(|v| v.as_str()).unwrap_or("").to_string()
                };
                let get_u = |key1: &str, key2: &str, def: u64| -> u64 {
                    row.get(key1).or_else(|| row.get(key2)).and_then(|v| v.as_u64()).unwrap_or(def)
                };
                let get_i = |key1: &str, key2: &str, def: i64| -> i64 {
                    row.get(key1).or_else(|| row.get(key2)).and_then(|v| v.as_i64()).unwrap_or(def)
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
                let items: Vec<UserItem> = serde_json::from_str(&get_s("items_json", "ITEMS_JSON")).unwrap_or_default();

                let mut user = User {
                    id: if id.is_empty() { format!("p_{}", username) } else { id },
                    username,
                    display_name: if display_name.is_empty() { "Người chơi".to_string() } else { display_name },
                    avatar: if avatar.is_empty() { "knight".to_string() } else { avatar },
                    avatar_id: if avatar_id.is_empty() { "avatar_knight".to_string() } else { avatar_id },
                    cardback_id: if cardback_id.is_empty() { "cb_classic".to_string() } else { cardback_id },
                    board_skin: if board_skin.is_empty() { "board_arena".to_string() } else { board_skin },
                    password_hash,
                    level,
                    current_exp,
                    elo,
                    wins,
                    losses,
                    matches,
                    gold,
                    gems,
                    rank_tier: if rank_tier.is_empty() { "Đồng".to_string() } else { rank_tier },
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
            let cdata = serde_json::to_string(&deck.cards_data).unwrap_or_else(|_| "[]".to_string()).replace('\'', "''");
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
        let cards_json = serde_json::to_string(&user.cards).unwrap_or_else(|_| "[]".to_string()).replace('\'', "''");
        let items_json = serde_json::to_string(&user.items).unwrap_or_else(|_| "[]".to_string()).replace('\'', "''");
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

// ==========================================
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
            user.ensure_valid_id_and_deck();
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
            user.ensure_valid_id_and_deck();
        }

        Self { file_path, data, adb }
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
        println!("[ORACLE ADB] Successfully fetched {} players from Oracle ADB.", count);

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
                    eprintln!("[ORACLE ADB ERROR] Failed to save user {}: {}", user.username, e);
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
                    eprintln!("[ORACLE ADB ERROR] Failed to record match {}: {}", m.match_id, e);
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

    pub fn foil_card(
        &mut self,
        username: &str,
        card_id: &str,
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

        let msg = format!("✨ Chúc mừng! Lá '{}' đã trở thành THẺ TINH ANH (Foil Hologram) (+20 HP, +6 ATK)!", card.name);
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

        let idx = user.decks.iter().position(|d| d.id == deck_id).ok_or("Không tìm thấy bộ bài!")?;
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
        Ok((res_user, gold_earned, exp_earned, gems_earned, leveled_up, promo_msg))
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

// MULTIPLAYER ROOM & MATCHMAKING
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

// ==========================================\n// ==========================================
// REST API REQUEST DTOs
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
struct CustomizeRequest {
    username: String,
    display_name: Option<String>,
    avatar_id: Option<String>,
    cardback_id: Option<String>,
    board_skin: Option<String>,
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
struct FoilCardRequest {
    username: String,
    card_id: String,
}

#[derive(Deserialize)]
struct SellCardRequest {
    username: String,
    card_id: String,
}

#[derive(Deserialize)]
struct SaveDeckRequest {
    username: String,
    deck_id: Option<String>,
    deck_name: String,
    hero_class: String,
    cardback_id: Option<String>,
    cards: Vec<DeckCardEntry>,
}

#[derive(Deserialize)]
struct DeleteDeckRequest {
    username: String,
    deck_id: String,
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
        Some(user) => {
            let exp_needed = user.exp_needed_for_next_level();
            let exp_percent = if exp_needed > 0 {
                ((user.current_exp as f64) / (exp_needed as f64) * 100.0).min(100.0)
            } else {
                0.0
            };
            let total_foils = user.cards.iter().filter(|c| c.is_foil).count();
            let recent_matches: Vec<MatchRecord> = db
                .data
                .matches
                .iter()
                .filter(|m| m.host.eq_ignore_ascii_case(&user.username) || m.guest.eq_ignore_ascii_case(&user.username))
                .take(10)
                .cloned()
                .collect();

            let resp = serde_json::json!({
                "success": true,
                "player": {
                    "id": user.id,
                    "username": user.username,
                    "display_name": user.display_name,
                    "avatar_id": user.avatar_id,
                    "cardback_id": user.cardback_id,
                    "board_skin": user.board_skin,
                    "level": user.level,
                    "current_exp": user.current_exp,
                    "exp_needed": exp_needed,
                    "exp_percent": exp_percent,
                    "gold": user.gold,
                    "gems": user.gems,
                },
                "rating": {
                    "rank_tier": user.rank_tier,
                    "rank_division": user.rank_division,
                    "rank_stars": user.rank_stars,
                    "rank_display": user.get_rank_display(),
                    "stars_display": user.get_stars_display(),
                    "mmr": user.mmr,
                    "total_matches": user.matches,
                    "wins": user.wins,
                    "losses": user.losses,
                    "win_rate": format!("{:.1}%", user.win_rate_percent()),
                    "win_streak": user.win_streak,
                    "best_streak": user.best_streak,
                },
                "collection": {
                    "total_cards": user.cards.len(),
                    "total_foils": total_foils,
                    "max_collection": 25,
                    "cards": user.cards,
                },
                "decks": user.decks,
                "recent_matches": recent_matches,
                "user": user.sanitized(),
            });

            (StatusCode::OK, Json(resp))
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "message": "Không tìm thấy người chơi!" })),
        ),
    }
}

async fn handle_customize(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CustomizeRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.customize_profile(
        &payload.username,
        payload.display_name,
        payload.avatar_id,
        payload.cardback_id,
        payload.board_skin,
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

async fn handle_get_decks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
) -> impl IntoResponse {
    let db = state.db.read().await;
    match db.get_user(&query.username) {
        Some(user) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "decks": user.decks })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "message": "Không tìm thấy người chơi!" })),
        ),
    }
}

async fn handle_save_deck(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SaveDeckRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.save_deck(
        &payload.username,
        payload.deck_id,
        payload.deck_name,
        payload.hero_class,
        payload.cardback_id,
        payload.cards,
    ) {
        Ok((user, deck)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "user": user.sanitized(), "deck": deck })),
        ),
        Err(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": msg })),
        ),
    }
}

async fn handle_delete_deck(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DeleteDeckRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.delete_deck(&payload.username, &payload.deck_id) {
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

async fn handle_foil_card(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<FoilCardRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.foil_card(&payload.username, &payload.card_id) {
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
        Ok((user, gold, exp, gems, leveled_up, promo_msg)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "user": user.sanitized(),
                "gold_earned": gold,
                "exp_earned": exp,
                "gems_earned": gems,
                "leveled_up": leveled_up,
                "promo_msg": promo_msg
            })),
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

// WEBSOCKET HANDLER
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

                "HERO_EMOTE" => {
                    let emote = parsed.get("emote").and_then(|v| v.as_str()).unwrap_or("👋");
                    if let Some(code) = &current_room {
                        let rooms_guard = state.rooms.read().await;
                        if let Some(room) = rooms_guard.get(code) {
                            let emote_msg = serde_json::json!({
                                "type": "HERO_EMOTE",
                                "emote": emote
                            }).to_string();
                            if current_role.as_deref() == Some("host") {
                                if let Some(ref g) = room.guest {
                                    let _ = g.tx.send(Message::Text(emote_msg.into()));
                                }
                            } else if current_role.as_deref() == Some("guest") {
                                let _ = room.host.tx.send(Message::Text(emote_msg.into()));
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
        .route("/api/user/customize", post(handle_customize))
        .route("/api/decks", get(handle_get_decks))
        .route("/api/decks/save", post(handle_save_deck))
        .route("/api/decks/delete", post(handle_delete_deck))
        .route("/api/cards/foil", post(handle_foil_card))
        .route("/api/shop/buy_card", post(handle_buy_card))
        .route("/api/cards/upgrade", post(handle_upgrade_card))
        .route("/api/cards/sell", post(handle_sell_card))
        .route("/api/match/reward", post(handle_match_reward))
        .route("/api/leaderboard", get(handle_leaderboard))
        .route("/api/match/record", post(handle_record_match))
        .fallback_service(ServeDir::new("dist/wasm"))
        .layer(cors)
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!("=======================================================");
    println!(" 🦀 3v3 TACTICAL ARENA - RUST WEBSOCKET SERVER");
    println!(" 🌐 Server Listening on: http://{}", addr);
    println!(" ⚡ WebSocket Endpoint: ws://{}/ws", addr);
    println!(" 📦 Static Assets: dist/wasm");
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

        // 1. Register gives starter card + 100G + 10 Gems + Starter Deck
        let user = db.register("test_hero", "1234", "Hero Test", "knight").expect("register failed");
        assert_eq!(user.gold, 100);
        assert_eq!(user.gems, 10);
        assert_eq!(user.cards.len(), 1);
        assert_eq!(user.decks.len(), 1);
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

        // 5. Reward match (PvE victory +45G, +60 EXP, +2 Gems)
        let (user, earned, exp, gems, _, _) = db.reward_match("test_hero", "pve", true).expect("reward failed");
        assert_eq!(earned, 45);
        assert_eq!(exp, 60);
        assert_eq!(gems, 2);
        assert_eq!(user.gold, 75);

        // Grant gems for testing foil upgrade
        {
            let u = db.data.users.get_mut("test_hero").unwrap();
            u.gems = 60;
        }

        // 6. Foil upgrade (costs 50 gems)
        let (user, _msg) = db.foil_card("test_hero", &new_card.id).expect("foil failed");
        assert_eq!(user.gems, 10); // 60 - 50 = 10
        let foiled = user.cards.iter().find(|c| c.id == new_card.id).unwrap();
        assert!(foiled.is_foil);

        // 7. Profile Customization
        let user = db.customize_profile("test_hero", Some("Đại Tướng".to_string()), Some("avatar_mage".to_string()), Some("cb_dragon".to_string()), Some("board_lava".to_string())).expect("customize failed");
        assert_eq!(user.display_name, "Đại Tướng");
        assert_eq!(user.avatar_id, "avatar_mage");
        assert_eq!(user.cardback_id, "cb_dragon");
        assert_eq!(user.board_skin, "board_lava");

        // 8. Deck Builder Engine & Validation
        let cards_data = vec![
            DeckCardEntry { id: starter.id.clone(), count: 1 },
            DeckCardEntry { id: new_card.id.clone(), count: 1 },
        ];
        // Archer card in Knight deck should fail validation
        let (is_valid, errors) = validate_deck("Bộ Bài Chiến Binh", "Knight", &cards_data, &user.cards);
        assert!(!is_valid);
        assert!(errors.iter().any(|e| e.contains("không phù hợp")));

        // Valid Archer deck
        let valid_cards = vec![DeckCardEntry { id: new_card.id.clone(), count: 1 }];
        let (is_valid_archer, errors_archer) = validate_deck("Bộ Bài Xạ Thủ", "Archer", &valid_cards, &user.cards);
        assert!(is_valid_archer);
        assert!(errors_archer.is_empty());

        let (user, deck) = db.save_deck("test_hero", None, "Bộ Bài Xạ Thủ".to_string(), "Archer".to_string(), Some("cb_dragon".to_string()), valid_cards).expect("save deck failed");
        assert_eq!(user.decks.len(), 2);
        assert!(deck.is_valid);

        // 9. Sell the Archer card (non-starter is sellable)
        let (user, refund) = db.sell_card("test_hero", &new_card.id).expect("sell non-starter failed");
        assert!(refund > 0);
        assert_eq!(user.cards.len(), 1);
        assert_eq!(user.cards[0].id, starter.id); // only starter remains

        let _ = std::fs::remove_file(test_db_path);
    }
}
