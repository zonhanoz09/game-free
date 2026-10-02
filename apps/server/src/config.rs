use std::time::{SystemTime, UNIX_EPOCH};

pub fn chrono_now() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_secs().to_string()
}

pub fn default_gold() -> u32 {
    100
}

pub fn default_gems() -> u32 {
    10
}

pub fn default_level() -> u32 {
    1
}

pub fn default_rank_tier() -> String {
    "Đồng".to_string()
}

pub fn default_rank_division() -> u8 {
    3
}

pub fn default_mmr() -> i32 {
    1200
}

pub fn default_quantity() -> u32 {
    1
}

pub fn default_avatar_id() -> String {
    "avatar_knight".to_string()
}

pub fn default_cardback_id() -> String {
    "cb_classic".to_string()
}

pub fn default_board_skin() -> String {
    "board_arena".to_string()
}

pub fn default_user_id() -> String {
    format!("p_{}", chrono_now())
}
