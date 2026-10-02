use super::*;

// ==========================================\n// ==========================================
// REST API REQUEST DTOs
// ==========================================
#[derive(Deserialize)]
pub(crate) struct RegisterRequest {
    username: String,
    password: String,
    display_name: Option<String>,
    avatar: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub(crate) struct ProfileQuery {
    username: String,
}

#[derive(Deserialize)]
pub(crate) struct CustomizeRequest {
    username: String,
    display_name: Option<String>,
    avatar_id: Option<String>,
    cardback_id: Option<String>,
    board_skin: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct BuyCardRequest {
    username: String,
    hero_class: String,
}

#[derive(Deserialize)]
pub(crate) struct UpgradeCardRequest {
    username: String,
    card_id: String,
    upgrade_type: String, // "level" or "star"
}

#[derive(Deserialize)]
pub(crate) struct FoilCardRequest {
    username: String,
    card_id: String,
}

#[derive(Deserialize)]
pub(crate) struct SellCardRequest {
    username: String,
    card_id: String,
}

#[derive(Deserialize)]
pub(crate) struct SaveDeckRequest {
    username: String,
    deck_id: Option<String>,
    deck_name: String,
    hero_class: String,
    cardback_id: Option<String>,
    cards: Vec<DeckCardEntry>,
}

#[derive(Deserialize)]
pub(crate) struct DeleteDeckRequest {
    username: String,
    deck_id: String,
}

#[derive(Deserialize)]
pub(crate) struct MatchRewardRequest {
    username: String,
    mode: Option<String>,
    win: bool,
}

#[derive(Deserialize)]
pub(crate) struct RecordMatchRequest {
    match_id: Option<String>,
    host: String,
    guest: String,
    winner: Option<String>,
    rounds: Option<usize>,
}

pub async fn handle_register(
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

pub async fn handle_login(
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

pub async fn handle_profile(
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
                .filter(|m| {
                    m.host.eq_ignore_ascii_case(&user.username)
                        || m.guest.eq_ignore_ascii_case(&user.username)
                })
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

pub async fn handle_customize(
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

pub async fn handle_get_decks(
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

pub async fn handle_save_deck(
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

pub async fn handle_delete_deck(
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

pub async fn handle_buy_card(
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

pub async fn handle_upgrade_card(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpgradeCardRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.upgrade_card(&payload.username, &payload.card_id, &payload.upgrade_type) {
        Ok((user, message)) => (
            StatusCode::OK,
            Json(
                serde_json::json!({ "success": true, "user": user.sanitized(), "message": message }),
            ),
        ),
        Err(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": msg })),
        ),
    }
}

pub async fn handle_foil_card(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<FoilCardRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.foil_card(&payload.username, &payload.card_id) {
        Ok((user, message)) => (
            StatusCode::OK,
            Json(
                serde_json::json!({ "success": true, "user": user.sanitized(), "message": message }),
            ),
        ),
        Err(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": msg })),
        ),
    }
}

pub async fn handle_sell_card(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SellCardRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.sell_card(&payload.username, &payload.card_id) {
        Ok((user, refund)) => (
            StatusCode::OK,
            Json(
                serde_json::json!({ "success": true, "user": user.sanitized(), "refund": refund }),
            ),
        ),
        Err(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": msg })),
        ),
    }
}

pub async fn handle_match_reward(
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

pub async fn handle_leaderboard(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.read().await;
    let list = db.get_leaderboard(20);
    Json(serde_json::json!({ "success": true, "leaderboard": list }))
}

pub async fn handle_record_match(
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
