use super::*;
use axum::http::HeaderMap;

fn is_authorized(headers: &HeaderMap) -> bool {
    let expected = std::env::var("ADMIN_KEY").unwrap_or_else(|_| "admin_secret_2026".to_string());
    if let Some(key) = headers.get("x-admin-key").and_then(|v| v.to_str().ok()) {
        if key == expected {
            return true;
        }
    }
    // Also check authorization header bearer
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            if token.trim() == expected {
                return true;
            }
        }
    }
    false
}

macro_rules! check_auth {
    ($headers:expr) => {
        if !is_authorized(&$headers) {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Unauthorized: Invalid or missing X-Admin-Key header"
                })),
            );
        }
    };
}

// ==========================================
// DTOs
// ==========================================
#[derive(Deserialize)]
pub(crate) struct AdminUserUpdateRequest {
    pub username: String,
    pub display_name: Option<String>,
    pub gold: Option<u32>,
    pub gems: Option<u32>,
    pub level: Option<u32>,
    pub elo: Option<i32>,
    pub battle_slots: Option<u8>,
    pub rank_tier: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AdminUserCreateRequest {
    pub username: String,
    pub password: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub gold: Option<u32>,
    pub gems: Option<u32>,
    pub battle_slots: Option<u8>,
}

#[derive(Deserialize)]
pub(crate) struct AdminUserDeleteRequest {
    pub username: String,
}

#[derive(Deserialize)]
pub(crate) struct AdminGrantCardRequest {
    pub username: String,
    pub hero_class: String,
    pub level: Option<u32>,
    pub star_level: Option<u32>,
    pub is_foil: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct AdminRemoveCardRequest {
    pub username: String,
    pub card_id: String,
}

#[derive(Deserialize)]
pub(crate) struct AdminDeleteEntityRequest {
    pub id: String,
}

#[derive(Deserialize)]
pub(crate) struct AdminDeleteTemplateSkillRequest {
    pub card_template_id: String,
    pub skill_id: String,
}

// ==========================================
// Handlers
// ==========================================

pub async fn handle_admin_overview(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let db = state.db.read().await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "counts": {
                "total_users": db.data.users.len(),
                "total_matches": db.data.matches.len(),
                "total_templates": db.data.master_templates.len(),
                "total_skills": db.data.master_skills.len(),
                "total_effects": db.data.master_effects.len(),
                "total_cells": db.data.master_cells.len(),
                "total_rarities": db.data.master_rarities.len(),
                "total_line_configs": db.data.master_lines.len(),
                "total_shop_cards": db.data.shop_cards.len(),
                "total_template_skills": db.data.master_template_skills.len(),
                "total_synergies": db.data.master_synergies.len(),
            },
            "timestamp": chrono_now(),
        })),
    )
}

pub async fn handle_admin_list_users(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let db = state.db.read().await;

    let users: Vec<User> = db
        .data
        .users
        .values()
        .cloned()
        .map(|mut u| {
            u.password_hash = "******".to_string();
            u
        })
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "users": users,
        })),
    )
}

pub async fn handle_admin_create_user(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminUserCreateRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    let av = payload.avatar.as_deref().unwrap_or("knight");
    let dname = payload.display_name.as_deref().unwrap_or(&payload.username);

    match db.register(&payload.username, &payload.password, dname, av) {
        Ok(mut user) => {
            if let Some(gold) = payload.gold {
                user.gold = gold;
            }
            if let Some(gems) = payload.gems {
                user.gems = gems;
            }
            if let Some(slots) = payload.battle_slots {
                user.battle_slots = slots.clamp(1, 5);
            }
            let key = payload.username.to_lowercase();
            db.data.users.insert(key, user.clone());
            db.persist_user_adb(&user);
            db.save();

            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "success": true,
                    "user": user.sanitized(),
                })),
            )
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": err,
            })),
        ),
    }
}

pub async fn handle_admin_update_user(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminUserUpdateRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;
    let key = payload.username.to_lowercase();

    let user = match db.data.users.get_mut(&key) {
        Some(u) => u,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("User '{}' not found", payload.username),
                })),
            );
        }
    };

    if let Some(dn) = payload.display_name {
        user.display_name = dn;
    }
    if let Some(g) = payload.gold {
        user.gold = g;
    }
    if let Some(g) = payload.gems {
        user.gems = g;
    }
    if let Some(l) = payload.level {
        user.level = l;
    }
    if let Some(e) = payload.elo {
        user.elo = e;
    }
    if let Some(b) = payload.battle_slots {
        user.battle_slots = b.clamp(1, 5);
    }
    if let Some(rt) = payload.rank_tier {
        user.rank_tier = rt;
    }
    if let Some(ref pwd) = payload.password {
        if !pwd.trim().is_empty() {
            let mut hasher = Sha256::new();
            hasher.update(pwd.as_bytes());
            user.password_hash = format!("{:x}", hasher.finalize());
        }
    }

    let sanitized = user.clone().sanitized();
    let u_to_save = user.clone();
    db.persist_user_adb(&u_to_save);
    db.save();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "user": sanitized,
        })),
    )
}

pub async fn handle_admin_delete_user(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminUserDeleteRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;
    let key = payload.username.to_lowercase();

    if db.data.users.remove(&key).is_some() {
        db.data.player_formations.remove(&key);
        db.delete_user_adb(&payload.username);
        db.save();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("User '{}' deleted successfully", payload.username),
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": format!("User '{}' not found", payload.username),
            })),
        )
    }
}

pub async fn handle_admin_grant_card(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminGrantCardRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;
    let key = payload.username.to_lowercase();

    let user = match db.data.users.get_mut(&key) {
        Some(u) => u,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("User '{}' not found", payload.username),
                })),
            );
        }
    };

    let hero_name = general_display_name(&payload.hero_class);
    let star_level = payload.star_level.unwrap_or(1);
    let level = payload.level.unwrap_or(1);
    let is_foil = payload.is_foil.unwrap_or(false);

    // If card already exists in collection, increment quantity or update
    if let Some(existing) = user
        .cards
        .iter_mut()
        .find(|c| c.hero_class == payload.hero_class)
    {
        existing.quantity += 1;
        if star_level > existing.star_level {
            existing.star_level = star_level;
        }
        if level > existing.level {
            existing.level = level;
        }
        if is_foil {
            existing.is_foil = true;
        }
    } else {
        let card = UserCard {
            id: format!("c_{}_{}_{}", key, payload.hero_class, chrono_now()),
            hero_class: payload.hero_class.clone(),
            name: hero_name.to_string(),
            star_level,
            level,
            is_starter: false,
            hp_bonus: 0.0,
            atk_bonus: 0.0,
            quantity: 1,
            is_foil,
        };
        user.cards.push(card);
    }

    let sanitized = user.clone().sanitized();
    let u_to_save = user.clone();
    db.persist_user_adb(&u_to_save);
    db.save();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": format!("Card '{}' granted to user '{}'", payload.hero_class, payload.username),
            "user": sanitized,
        })),
    )
}

pub async fn handle_admin_remove_card(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminRemoveCardRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;
    let key = payload.username.to_lowercase();

    let user = match db.data.users.get_mut(&key) {
        Some(u) => u,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("User '{}' not found", payload.username),
                })),
            );
        }
    };

    if let Some(pos) = user.cards.iter().position(|c| c.id == payload.card_id) {
        let card_id = payload.card_id.clone();
        let user_id = user.id.clone();
        user.cards.remove(pos);
        let u_to_save = user.clone();
        db.delete_card_adb(&user_id, &card_id);
        db.persist_user_adb(&u_to_save);
        db.save();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("Card '{}' removed", payload.card_id),
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Card not found in user collection",
            })),
        )
    }
}

// ==========================================
// Shop Cards CRUD
// ==========================================

pub async fn handle_admin_list_shop_cards(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let db = state.db.read().await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "cards": db.data.shop_cards,
        })),
    )
}

pub async fn handle_admin_save_shop_card(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(card): Json<ShopCardItem>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .shop_cards
        .iter()
        .position(|c| c.card_id == card.card_id)
    {
        db.data.shop_cards[pos] = card.clone();
    } else {
        db.data.shop_cards.push(card.clone());
    }
    db.save();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "card": card,
        })),
    )
}

pub async fn handle_admin_delete_shop_card(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminDeleteEntityRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .shop_cards
        .iter()
        .position(|c| c.card_id == payload.id)
    {
        db.data.shop_cards.remove(pos);
        db.save();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("Đã xóa thẻ {} khỏi cửa hàng", payload.id)
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Không tìm thấy thẻ cửa hàng"
            })),
        )
    }
}

// ==========================================
// Template CRUD
// ==========================================

pub async fn handle_admin_list_templates(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let db = state.db.read().await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "templates": db.data.master_templates,
        })),
    )
}

pub async fn handle_admin_save_template(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(template): Json<game_data_schema::CardTemplateRow>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .master_templates
        .iter()
        .position(|t| t.card_template_id == template.card_template_id)
    {
        db.data.master_templates[pos] = template.clone();
    } else {
        db.data.master_templates.push(template.clone());
    }
    db.save();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "template": template,
        })),
    )
}

pub async fn handle_admin_delete_template(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminDeleteEntityRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .master_templates
        .iter()
        .position(|t| t.card_template_id == payload.id)
    {
        db.data.master_templates.remove(pos);
        db.save();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("Template '{}' deleted", payload.id),
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Template not found",
            })),
        )
    }
}

// ==========================================
// Skill CRUD
// ==========================================

pub async fn handle_admin_list_skills(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let db = state.db.read().await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "skills": db.data.master_skills,
        })),
    )
}

pub async fn handle_admin_save_skill(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(skill): Json<game_data_schema::SkillRow>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .master_skills
        .iter()
        .position(|s| s.skill_id == skill.skill_id)
    {
        db.data.master_skills[pos] = skill.clone();
    } else {
        db.data.master_skills.push(skill.clone());
    }
    db.save();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "skill": skill,
        })),
    )
}

pub async fn handle_admin_delete_skill(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminDeleteEntityRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .master_skills
        .iter()
        .position(|s| s.skill_id == payload.id)
    {
        db.data.master_skills.remove(pos);
        db.save();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("Skill '{}' deleted", payload.id),
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Skill not found",
            })),
        )
    }
}

// ==========================================
// Rarity & Line Configs CRUD
// ==========================================

pub async fn handle_admin_list_rarities(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let db = state.db.read().await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "rarities": db.data.master_rarities,
        })),
    )
}

pub async fn handle_admin_save_rarity(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(rarity): Json<game_data_schema::RarityConfig>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;
    db.data
        .master_rarities
        .insert(rarity.rarity_id.clone(), rarity.clone());
    db.save();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "rarity": rarity,
        })),
    )
}

pub async fn handle_admin_delete_rarity(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminDeleteEntityRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;
    if db.data.master_rarities.remove(&payload.id).is_some() {
        db.save();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("Rarity '{}' deleted", payload.id),
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Rarity not found",
            })),
        )
    }
}

pub async fn handle_admin_list_lines(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let db = state.db.read().await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "line_configs": db.data.master_lines,
        })),
    )
}

pub async fn handle_admin_save_line(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(line_cfg): Json<game_data_schema::FormationLineConfig>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .master_lines
        .iter()
        .position(|l| l.line_type == line_cfg.line_type)
    {
        db.data.master_lines[pos] = line_cfg.clone();
    } else {
        db.data.master_lines.push(line_cfg.clone());
    }
    db.save();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "line_config": line_cfg,
        })),
    )
}

pub async fn handle_admin_delete_line(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminDeleteEntityRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;
    let target_type = match payload.id.to_uppercase().as_str() {
        "FRONT" => game_data_schema::FormationLineType::Front,
        "MID" => game_data_schema::FormationLineType::Mid,
        "BACK" => game_data_schema::FormationLineType::Back,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Invalid line type. Expected FRONT, MID, or BACK",
                })),
            );
        }
    };
    if let Some(pos) = db
        .data
        .master_lines
        .iter()
        .position(|l| l.line_type == target_type)
    {
        db.data.master_lines.remove(pos);
        db.save();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("Line '{}' deleted", payload.id),
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Line config not found",
            })),
        )
    }
}

// ==========================================
// Status Effects CRUD
// ==========================================

pub async fn handle_admin_list_effects(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let db = state.db.read().await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "effects": db.data.master_effects,
        })),
    )
}

pub async fn handle_admin_save_effect(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(effect): Json<game_data_schema::StatusEffectRow>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .master_effects
        .iter()
        .position(|e| e.effect_id == effect.effect_id)
    {
        db.data.master_effects[pos] = effect.clone();
    } else {
        db.data.master_effects.push(effect.clone());
    }
    db.save();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "effect": effect,
        })),
    )
}

pub async fn handle_admin_delete_effect(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminDeleteEntityRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .master_effects
        .iter()
        .position(|e| e.effect_id == payload.id)
    {
        db.data.master_effects.remove(pos);
        db.save();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("Effect '{}' deleted", payload.id),
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Effect not found",
            })),
        )
    }
}

// ==========================================
// Board Cells CRUD
// ==========================================

pub async fn handle_admin_list_cells(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let db = state.db.read().await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "cells": db.data.master_cells,
        })),
    )
}

pub async fn handle_admin_save_cell(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(cell): Json<game_data_schema::BoardCellConfigRow>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .master_cells
        .iter()
        .position(|c| c.slot_id == cell.slot_id)
    {
        db.data.master_cells[pos] = cell.clone();
    } else {
        db.data.master_cells.push(cell.clone());
    }
    db.data.master_cells.sort_by_key(|c| c.slot_id);
    db.save();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "cell": cell,
        })),
    )
}

pub async fn handle_admin_delete_cell(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminDeleteEntityRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let Ok(slot) = payload.id.parse::<u8>() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "Slot ID must be an integer between 1 and 9",
            })),
        );
    };

    let mut db = state.db.write().await;
    if let Some(pos) = db.data.master_cells.iter().position(|c| c.slot_id == slot) {
        db.data.master_cells.remove(pos);
        db.save();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("Cell slot '{}' removed", slot),
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Cell slot not found",
            })),
        )
    }
}

// ==========================================
// Matches & Maintenance
// ==========================================

pub async fn handle_admin_list_matches(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let db = state.db.read().await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "matches": db.data.matches,
        })),
    )
}

pub async fn handle_admin_reset_seeds(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    db.data.master_rarities = game_data_schema::default_rarity_configs();
    db.data.master_lines = game_data_schema::default_formation_line_configs();
    db.data.master_skills = game_data_schema::default_master_skills();
    db.data.master_templates = game_data_schema::default_master_templates();
    db.data.master_effects = game_data_schema::default_master_effects();
    db.data.master_cells = game_data_schema::default_master_cells();
    db.data.shop_cards = default_shop_cards();
    db.data.master_template_skills = game_data_schema::default_master_template_skills();
    db.data.master_synergies = game_data_schema::default_master_synergies();
    db.save();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Master seeds restored to initial defaults",
            "rarities_count": db.data.master_rarities.len(),
            "lines_count": db.data.master_lines.len(),
            "skills_count": db.data.master_skills.len(),
            "templates_count": db.data.master_templates.len(),
            "effects_count": db.data.master_effects.len(),
            "cells_count": db.data.master_cells.len(),
        })),
    )
}


// ==========================================
// Template Skills (Hero-to-Skill) CRUD
// ==========================================

pub async fn handle_admin_list_template_skills(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let db = state.db.read().await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "template_skills": db.data.master_template_skills,
        })),
    )
}

pub async fn handle_admin_save_template_skill(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(ts): Json<game_data_schema::CardTemplateSkillRow>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .master_template_skills
        .iter()
        .position(|item| item.card_template_id == ts.card_template_id && item.skill_id == ts.skill_id)
    {
        db.data.master_template_skills[pos] = ts.clone();
    } else {
        db.data.master_template_skills.push(ts.clone());
    }
    db.save();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "template_skill": ts,
        })),
    )
}

pub async fn handle_admin_delete_template_skill(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminDeleteTemplateSkillRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .master_template_skills
        .iter()
        .position(|item| item.card_template_id == payload.card_template_id && item.skill_id == payload.skill_id)
    {
        db.data.master_template_skills.remove(pos);
        db.save();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("Đã hủy liên kết kỹ năng '{}' khỏi tướng '{}'", payload.skill_id, payload.card_template_id)
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Không tìm thấy liên kết kỹ năng tướng"
            })),
        )
    }
}

// ==========================================
// Formation Synergies (Kích Duyên Đội Hình) CRUD
// ==========================================

pub async fn handle_admin_list_synergies(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    check_auth!(headers);
    let db = state.db.read().await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "synergies": db.data.master_synergies,
        })),
    )
}

pub async fn handle_admin_save_synergy(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(synergy): Json<game_data_schema::FormationSynergyRow>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .master_synergies
        .iter()
        .position(|s| s.synergy_id == synergy.synergy_id)
    {
        db.data.master_synergies[pos] = synergy.clone();
    } else {
        db.data.master_synergies.push(synergy.clone());
    }
    db.save();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "synergy": synergy,
        })),
    )
}

pub async fn handle_admin_delete_synergy(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminDeleteEntityRequest>,
) -> impl IntoResponse {
    check_auth!(headers);
    let mut db = state.db.write().await;

    if let Some(pos) = db
        .data
        .master_synergies
        .iter()
        .position(|s| s.synergy_id == payload.id)
    {
        db.data.master_synergies.remove(pos);
        db.save();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("Đã xóa duyên '{}'", payload.id)
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Không tìm thấy duyên đội hình"
            })),
        )
    }
}
