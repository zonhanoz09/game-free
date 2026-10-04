//! WebSocket lobby and matchmaking message handlers.

use crate::{AppState, generate_room_code, multiplayer::{PlayerSession, PublicRoomInfo, QuickMatchEntry, Room}};
use super::find_room_and_role;
use axum::extract::ws::Message;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn handle_lobby_message(
    msg_type: &str,
    parsed: &serde_json::Value,
    state: &Arc<AppState>,
    tx: &mpsc::UnboundedSender<Message>,
    current_room: &mut Option<String>,
    current_role: &mut Option<String>,
) -> bool {
    match msg_type {
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
                    true
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
                        settled_round: 0,
                        host: session,
                        guest: None,
                        authority: None,
                    };

                    rooms_guard.insert(code.clone(), room);
                    *current_room = Some(code.clone());
                    *current_role = Some("host".to_string());

                    let res = serde_json::json!({
                        "type": "ROOM_CREATED",
                        "room_code": code,
                        "role": "host",
                        "player_name": player_name
                    });
                    let _ = tx.send(Message::Text(res.to_string().into()));
                    println!("[RUST WS] Room {} created by {}", code, player_name);
                    true
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
                            *current_room = Some(code.clone());
                            *current_role = Some("guest".to_string());

                            let host_avatar = room.host.avatar.clone();
                            let to_host = serde_json::json!({
                                "type": "ROOM_JOINED",
                                "room_code": code,
                                "role": "host",
                                "player_name": host_name,
                                "opponent_name": player_name,
                                "opponent_avatar": avatar,
                                "host_avatar": host_avatar,
                                "round": room.round
                            });
                            let _ = room.host.tx.send(Message::Text(to_host.to_string().into()));

                            let to_guest = serde_json::json!({
                                "type": "ROOM_JOINED",
                                "room_code": code,
                                "role": "guest",
                                "player_name": player_name,
                                "opponent_name": host_name,
                                "opponent_avatar": host_avatar,
                                "host_avatar": host_avatar,
                                "round": room.round
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
                    true
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
                            settled_round: 0,
                            host: queued.session,
                            guest: Some(guest_session),
                            authority: None,
                        };

                        let host_avatar = room.host.avatar.clone();
                        let to_host = serde_json::json!({
                            "type": "ROOM_JOINED",
                            "room_code": code,
                            "role": "host",
                            "player_name": host_name,
                            "opponent_name": player_name,
                            "opponent_avatar": avatar,
                            "host_avatar": host_avatar,
                            "round": room.round
                        });
                        let _ = room.host.tx.send(Message::Text(to_host.to_string().into()));

                        let to_guest = serde_json::json!({
                            "type": "ROOM_JOINED",
                            "room_code": code,
                            "role": "guest",
                            "player_name": player_name,
                            "opponent_name": host_name,
                            "opponent_avatar": host_avatar,
                            "host_avatar": host_avatar,
                            "round": room.round
                        });
                        let _ = tx.send(Message::Text(to_guest.to_string().into()));

                        *current_room = Some(code.clone());
                        *current_role = Some("guest".to_string());
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
                    true
                }

                "CANCEL_MATCH" => {
                    let mut qm_guard = state.quick_match.write().await;
                    if let Some(ref entry) = *qm_guard {
                        if entry.session.tx.same_channel(tx) {
                            *qm_guard = None;
                            let res = serde_json::json!({ "type": "MATCH_CANCELED" });
                            let _ = tx.send(Message::Text(res.to_string().into()));
                        }
                    }
                    true
                }

                "LEAVE_ROOM" | "CANCEL_ROOM" => {
                    let room_code_hint = parsed.get("room_code").and_then(|v| v.as_str());
                    let mut rooms_guard = state.rooms.write().await;
                    if let Some((code, role)) = find_room_and_role(
                        &rooms_guard,
                        room_code_hint,
                        current_room,
                        current_role,
                        tx,
                    ) {
                        if role == "host" {
                            if let Some(room) = rooms_guard.remove(&code) {
                                if let Some(ref g) = room.guest {
                                    let _ = g.tx.send(Message::Text(
                                        serde_json::json!({
                                            "type": "ROOM_CLOSED",
                                            "message": "Chủ phòng đã đóng phòng."
                                        })
                                        .to_string()
                                        .into(),
                                    ));
                                }
                            }
                        } else if let Some(room) = rooms_guard.remove(&code) {
                            let _ = room.host.tx.send(Message::Text(
                                serde_json::json!({
                                    "type": "OPPONENT_LEFT",
                                    "message": "Đối thủ đã rời khỏi phòng."
                                })
                                .to_string()
                                .into(),
                            ));
                            let end_message = serde_json::json!({
                                "type": "MATCH_END",
                                "winner": room.host.name,
                                "winner_role": "host",
                                "player_hp": 100,
                                "opponent_hp": 0,
                                "gold_reward": 80,
                                "consolation_gold": 25
                            })
                            .to_string();
                            let _ = room.host.tx.send(Message::Text(end_message.into()));
                        }
                        *current_room = None;
                        *current_role = None;
                    }
                    let _ = tx.send(Message::Text(
                        serde_json::json!({ "type": "ROOM_LEFT" }).to_string().into(),
                    ));
                    true
                }


        _ => false,
    }
}
