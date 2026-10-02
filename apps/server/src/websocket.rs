use super::*;

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
                            })
                            .to_string();
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
