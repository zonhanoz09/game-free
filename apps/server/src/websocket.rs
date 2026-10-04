use super::*;

pub mod battle;
pub mod lobby;
pub mod progression;
pub mod social;

pub(crate) fn stable_battle_seed(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

pub(crate) fn find_room_and_role(
    rooms: &HashMap<String, Room>,
    room_code_hint: Option<&str>,
    current_room: &Option<String>,
    current_role: &Option<String>,
    tx: &mpsc::UnboundedSender<Message>,
) -> Option<(String, String)> {
    // 1. If room_code_hint is provided
    if let Some(code) = room_code_hint {
        if let Some(room) = rooms.get(code) {
            if room.host.tx.same_channel(tx) {
                return Some((code.to_string(), "host".to_string()));
            }
            if room
                .guest
                .as_ref()
                .map(|g| g.tx.same_channel(tx))
                .unwrap_or(false)
            {
                return Some((code.to_string(), "guest".to_string()));
            }
            if let Some(role) = current_role {
                return Some((code.to_string(), role.clone()));
            }
        }
    }

    // 2. If current_room is set
    if let Some(code) = current_room {
        if let Some(room) = rooms.get(code) {
            if room.host.tx.same_channel(tx) {
                return Some((code.clone(), "host".to_string()));
            }
            if room
                .guest
                .as_ref()
                .map(|g| g.tx.same_channel(tx))
                .unwrap_or(false)
            {
                return Some((code.clone(), "guest".to_string()));
            }
            if let Some(role) = current_role {
                return Some((code.clone(), role.clone()));
            }
            return Some((code.clone(), "host".to_string()));
        }
    }

    // 3. Fallback: search all rooms by socket channel
    for (code, room) in rooms.iter() {
        if room.host.tx.same_channel(tx) {
            return Some((code.clone(), "host".to_string()));
        }
        if room
            .guest
            .as_ref()
            .map(|g| g.tx.same_channel(tx))
            .unwrap_or(false)
        {
            return Some((code.clone(), "guest".to_string()));
        }
    }

    None
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_client(socket, state))
}

pub async fn handle_ws_client(socket: WebSocket, state: Arc<AppState>) {
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

            if lobby::handle_lobby_message(&msg_type, &parsed, &state, &tx, &mut current_room, &mut current_role).await {
                continue;
            }
            if battle::handle_battle_message(&msg_type, &parsed, &state, &tx, &mut current_room, &mut current_role).await {
                continue;
            }
            if progression::handle_progression_message(&msg_type, &parsed, &state, &tx).await {
                continue;
            }
            if social::handle_social_message(&msg_type, &parsed, &state, &tx, &current_room, &current_role).await {
                continue;
            }
        }
    }

    // Cleanup on disconnect
    {
        let mut rooms_guard = state.rooms.write().await;
        if let Some((code, role)) =
            find_room_and_role(&rooms_guard, None, &current_room, &current_role, &tx)
        {
            if let Some(room) = rooms_guard.get_mut(&code) {
                if role == "host" {
                    if let Some(ref g) = room.guest {
                        let _ = g.tx.send(Message::Text(
                            serde_json::json!({ "type": "ERROR", "message": "Chủ phòng đã thoát trận đấu." })
                                .to_string()
                                .into(),
                        ));
                    }
                    rooms_guard.remove(&code);
                } else if role == "guest" {
                    let _ = room.host.tx.send(Message::Text(
                        serde_json::json!({ "type": "ERROR", "message": "Đối thủ đã rời khỏi phòng." })
                            .to_string()
                            .into(),
                    ));
                    rooms_guard.remove(&code);
                }
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
