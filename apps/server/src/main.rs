#![allow(clippy::collapsible_if, clippy::redundant_closure)]
use axum::{
    Router,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    http::header,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use base64::prelude::*;
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    sync::Arc,
};
use tokio::sync::{RwLock, mpsc};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

mod config;
use config::*;

mod admin;
use admin::*;

mod api;
mod authority;
mod models;
mod multiplayer;
#[cfg(test)]
mod tests;
mod websocket;

use api::*;
use models::*;
use multiplayer::*;
use websocket::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let db = Database::new_with_adb().await;

    let state = Arc::new(AppState {
        db: Arc::new(RwLock::new(db)),
        rooms: Arc::new(RwLock::new(HashMap::new())),
        quick_match: Arc::new(RwLock::new(None)),
        player_progression: Arc::new(RwLock::new(HashMap::new())),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        .route("/api/auth/register", post(handle_register))
        .route("/api/auth/login", post(handle_login))
        .route("/api/user/profile", get(handle_profile))
        .route("/api/user/customize", post(handle_customize))
        .route("/api/decks", get(handle_get_decks))
        .route("/api/decks/save", post(handle_save_deck))
        .route("/api/decks/delete", post(handle_delete_deck))
        .route("/api/cards/foil", post(handle_foil_card))
        .route("/api/shop/cards", get(handle_get_shop_cards))
        .route(
            "/api/admin/shop/cards",
            get(handle_admin_list_shop_cards).post(handle_admin_save_shop_card),
        )
        .route(
            "/api/admin/shop/cards/delete",
            post(handle_admin_delete_shop_card),
        )
        .route("/api/shop/buy_card", post(handle_buy_card))
        .route("/api/shop/buy_battle_slot", post(handle_buy_battle_slot))
        .route("/api/cards/upgrade", post(handle_upgrade_card))
        .route("/api/cards/sell", post(handle_sell_card))
        .route("/api/match/reward", post(handle_match_reward))
        .route("/api/leaderboard", get(handle_leaderboard))
        .route("/api/match/record", post(handle_record_match))
        .route("/api/gacha/pull", post(handle_gacha_pull))
        .route("/api/progression", get(handle_get_progression))
        .route("/api/progression/upgrade_star", post(handle_upgrade_star))
        .route("/api/progression/upgrade_level", post(handle_upgrade_level))
        .route("/api/schema/master_data", get(handle_get_master_data))
        .route(
            "/api/formation/calculate_stats",
            post(handle_calculate_formation_stats),
        )
        .route("/api/formation/layout", get(handle_get_formation_layout))
        .route(
            "/api/formation/save_layout",
            post(handle_save_formation_layout),
        )
        .route("/admin", get(admin_handler))
        .route("/admin.html", get(admin_handler))
        .route("/api/admin/overview", get(handle_admin_overview))
        .route("/api/admin/users", get(handle_admin_list_users))
        .route("/api/admin/users/create", post(handle_admin_create_user))
        .route("/api/admin/users/update", post(handle_admin_update_user))
        .route("/api/admin/users/delete", post(handle_admin_delete_user))
        .route("/api/admin/users/grant_card", post(handle_admin_grant_card))
        .route(
            "/api/admin/users/remove_card",
            post(handle_admin_remove_card),
        )
        .route("/api/admin/templates", get(handle_admin_list_templates))
        .route(
            "/api/admin/templates/save",
            post(handle_admin_save_template),
        )
        .route(
            "/api/admin/templates/delete",
            post(handle_admin_delete_template),
        )
        .route("/api/admin/skills", get(handle_admin_list_skills))
        .route("/api/admin/skills/save", post(handle_admin_save_skill))
        .route("/api/admin/skills/delete", post(handle_admin_delete_skill))
        .route("/api/admin/rarities", get(handle_admin_list_rarities))
        .route("/api/admin/rarities/save", post(handle_admin_save_rarity))
        .route(
            "/api/admin/rarities/delete",
            post(handle_admin_delete_rarity),
        )
        .route("/api/admin/lines", get(handle_admin_list_lines))
        .route("/api/admin/lines/save", post(handle_admin_save_line))
        .route("/api/admin/lines/delete", post(handle_admin_delete_line))
        .route("/api/admin/effects", get(handle_admin_list_effects))
        .route("/api/admin/effects/save", post(handle_admin_save_effect))
        .route(
            "/api/admin/effects/delete",
            post(handle_admin_delete_effect),
        )
        .route("/api/admin/cells", get(handle_admin_list_cells))
        .route("/api/admin/cells/save", post(handle_admin_save_cell))
        .route("/api/admin/cells/delete", post(handle_admin_delete_cell))
        .route("/api/admin/template_skills", get(handle_admin_list_template_skills))
        .route(
            "/api/admin/template_skills/save",
            post(handle_admin_save_template_skill),
        )
        .route(
            "/api/admin/template_skills/delete",
            post(handle_admin_delete_template_skill),
        )
        .route("/api/admin/synergies", get(handle_admin_list_synergies))
        .route(
            "/api/admin/synergies/save",
            post(handle_admin_save_synergy),
        )
        .route(
            "/api/admin/synergies/delete",
            post(handle_admin_delete_synergy),
        )
        .route("/api/admin/matches", get(handle_admin_list_matches))
        .route("/api/admin/db/reset_seeds", post(handle_admin_reset_seeds))
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

async fn index_handler() -> impl IntoResponse {
    match fs::read_to_string("dist/wasm/index.html") {
        Ok(index) => (
            StatusCode::OK,
            [
                (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            ],
            index,
        )
            .into_response(),
        Err(error) => {
            tracing::error!("Failed to read web index: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [
                    (header::CACHE_CONTROL, "no-store"),
                    (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                ],
                "Web client unavailable",
            )
                .into_response()
        }
    }
}

async fn admin_handler() -> impl IntoResponse {
    match fs::read_to_string("dist/wasm/admin.html") {
        Ok(html) => (
            StatusCode::OK,
            [
                (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            ],
            html,
        )
            .into_response(),
        Err(error) => {
            tracing::error!("Failed to read admin page: {}", error);
            (
                StatusCode::NOT_FOUND,
                "Admin page not found. Please ensure dist/wasm/admin.html exists.",
            )
                .into_response()
        }
    }
}
