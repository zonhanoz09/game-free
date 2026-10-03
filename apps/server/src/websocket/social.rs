//! WebSocket social and in-match emote handlers.

use crate::AppState;
use axum::extract::ws::Message;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn handle_social_message(
    msg_type: &str,
    parsed: &serde_json::Value,
    state: &Arc<AppState>,
    tx: &mpsc::UnboundedSender<Message>,
    current_room: &Option<String>,
    current_role: &Option<String>,
) -> bool {
    if msg_type != "HERO_EMOTE" {
        return false;
    }

    let emote = parsed.get("emote").and_then(|v| v.as_str()).unwrap_or("👋");
    let room_code_hint = parsed.get("room_code").and_then(|v| v.as_str());
    let rooms_guard = state.rooms.read().await;
    if let Some((code, role)) = super::find_room_and_role(
        &rooms_guard,
        room_code_hint,
        current_room,
        current_role,
        tx,
    ) {
        if let Some(room) = rooms_guard.get(&code) {
            let emote_msg = serde_json::json!({
                "type": "HERO_EMOTE",
                "emote": emote
            })
            .to_string();
            if role == "host" {
                if let Some(ref g) = room.guest {
                    let _ = g.tx.send(Message::Text(emote_msg.into()));
                }
            } else if role == "guest" {
                let _ = room.host.tx.send(Message::Text(emote_msg.into()));
            }
        }
    }
    true
}
