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
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{RwLock, mpsc};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

mod config;
use config::*;

mod api;
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
