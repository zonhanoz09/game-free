use super::*;
use crate::authority::AuthoritativeBattle;

fn stable_battle_seed(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

fn find_room_and_role(
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
                        settled_round: 0,
                        host: session,
                        guest: None,
                        authority: None,
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

                "LEAVE_ROOM" | "CANCEL_ROOM" => {
                    let room_code_hint = parsed.get("room_code").and_then(|v| v.as_str());
                    let mut rooms_guard = state.rooms.write().await;
                    if let Some((code, role)) = find_room_and_role(
                        &rooms_guard,
                        room_code_hint,
                        &current_room,
                        &current_role,
                        &tx,
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
                        } else if let Some(room) = rooms_guard.get_mut(&code) {
                            room.guest = None;
                            let _ = room.host.tx.send(Message::Text(
                                serde_json::json!({
                                    "type": "OPPONENT_LEFT",
                                    "message": "Đối thủ đã rời khỏi phòng."
                                })
                                .to_string()
                                .into(),
                            ));
                        }
                        current_room = None;
                        current_role = None;
                    }
                    let _ = tx.send(Message::Text(
                        serde_json::json!({ "type": "ROOM_LEFT" }).to_string().into(),
                    ));
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

                    let room_code_hint = parsed.get("room_code").and_then(|v| v.as_str());
                    let mut rooms_guard = state.rooms.write().await;
                    if let Some((code, role)) = find_room_and_role(
                        &rooms_guard,
                        room_code_hint,
                        &current_room,
                        &current_role,
                        &tx,
                    ) {
                        current_room = Some(code.clone());
                        current_role = Some(role.clone());

                        if let Some(room) = rooms_guard.get_mut(&code) {
                            if role == "host" {
                                room.host.ready = true;
                                room.host.lineup = lineup;
                                if let Some(ref g) = room.guest {
                                    let _ = g.tx.send(Message::Text(
                                        serde_json::json!({ "type": "OPPONENT_READY" })
                                            .to_string()
                                            .into(),
                                    ));
                                }
                            } else if role == "guest" {
                                if let Some(ref mut g) = room.guest {
                                    g.ready = true;
                                    g.lineup = lineup;
                                }
                                let _ = room.host.tx.send(Message::Text(
                                    serde_json::json!({ "type": "OPPONENT_READY" })
                                        .to_string()
                                        .into(),
                                ));
                            }

                            println!(
                                "[RUST WS] Player {} locked in {} units in room {}",
                                role,
                                if role == "host" {
                                    room.host.lineup.len()
                                } else {
                                    room.guest.as_ref().map(|g| g.lineup.len()).unwrap_or(0)
                                },
                                code
                            );

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
                                let battle_id = format!("{}-{}", code, round);
                                let seed = stable_battle_seed(&battle_id);
                                room.authority =
                                    AuthoritativeBattle::from_setup(game_protocol::BattleSetup {
                                        battle_id,
                                        seed,
                                        config_version: "combat-v1".to_string(),
                                        config_fingerprint: "release-v1".to_string(),
                                        attacker: host_lineup.clone(),
                                        defender: guest_lineup.clone(),
                                    })
                                    .ok();

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
                    } else {
                        println!("[RUST WS] PLAYER_READY received but no room found for sender");
                    }
                }

                "BATTLE_COMMAND" => {
                    let command: game_protocol::CombatCommand = match serde_json::from_value(
                        parsed.get("command").cloned().unwrap_or(parsed.clone()),
                    ) {
                        Ok(command) => command,
                        Err(error) => {
                            let _ = tx.send(Message::Text(
                                serde_json::json!({
                                    "type": "ERROR",
                                    "message": format!("Lệnh chiến đấu không hợp lệ: {}", error)
                                })
                                .to_string()
                                .into(),
                            ));
                            continue;
                        }
                    };
                    let room_code_hint = parsed.get("room_code").and_then(|v| v.as_str());
                    let mut rooms_guard = state.rooms.write().await;
                    if let Some((code, _)) = find_room_and_role(
                        &rooms_guard,
                        room_code_hint,
                        &current_room,
                        &current_role,
                        &tx,
                    ) {
                        current_room = Some(code.clone());
                        if let Some(room) = rooms_guard.get_mut(&code) {
                            match room.authority.as_mut() {
                                Some(authority) => match authority.apply(command) {
                                    Ok(_) => {
                                        let result = authority.result();
                                        let message = serde_json::json!({
                                            "type": "BATTLE_RESULT",
                                            "result": result
                                        });
                                        let _ = room
                                            .host
                                            .tx
                                            .send(Message::Text(message.to_string().into()));
                                        if let Some(guest) = &room.guest {
                                            let _ = guest
                                                .tx
                                                .send(Message::Text(message.to_string().into()));
                                        }
                                        if let Some(winner) = result.winner.as_deref() {
                                            if authority.is_settled() {
                                                continue;
                                            }
                                            let survivors = match winner {
                                                "ATTACKER" => authority.surviving_count(
                                                    game_logic::TeamSide::Attacker,
                                                ),
                                                "DEFENDER" => authority.surviving_count(
                                                    game_logic::TeamSide::Defender,
                                                ),
                                                _ => 0,
                                            }
                                                as i32;
                                            let damage = 10 + survivors * 3;
                                            if winner == "ATTACKER" {
                                                if let Some(guest) = room.guest.as_mut() {
                                                    guest.hp = (guest.hp - damage).max(0);
                                                }
                                            } else if winner == "DEFENDER" {
                                                room.host.hp = (room.host.hp - damage).max(0);
                                            }
                                            let host_hp = room.host.hp;
                                            let guest_hp = room
                                                .guest
                                                .as_ref()
                                                .map(|guest| guest.hp)
                                                .unwrap_or(0);
                                            let host_hp_message = serde_json::json!({
                                                "type": "UPDATE_MATCH_HP",
                                                "player_hp": host_hp,
                                                "opponent_hp": guest_hp,
                                                "damage_dealt": damage
                                            })
                                            .to_string();
                                            let _ = room
                                                .host
                                                .tx
                                                .send(Message::Text(host_hp_message.into()));
                                            if let Some(guest) = &room.guest {
                                                let guest_hp_message = serde_json::json!({
                                                    "type": "UPDATE_MATCH_HP",
                                                    "player_hp": guest_hp,
                                                    "opponent_hp": host_hp,
                                                    "damage_dealt": damage
                                                })
                                                .to_string();
                                                let _ =
                                                    guest.tx.send(Message::Text(guest_hp_message.into()));
                                            }
                                            if host_hp <= 0 || guest_hp <= 0 {
                                                let winner_id = if host_hp > 0 {
                                                    room.host.id.clone()
                                                } else {
                                                    room.guest
                                                        .as_ref()
                                                        .map(|guest| guest.id.clone())
                                                        .unwrap_or_default()
                                                };
                                                let winner_name = if host_hp > 0 {
                                                    room.host.name.clone()
                                                } else {
                                                    room.guest
                                                        .as_ref()
                                                        .map(|guest| guest.name.clone())
                                                        .unwrap_or_default()
                                                };
                                                let end_message = serde_json::json!({
                                                    "type": "MATCH_END",
                                                    "winner": winner_name,
                                                    "gold_reward": 80,
                                                    "consolation_gold": 15
                                                })
                                                .to_string();
                                                let _ = room.host.tx.send(Message::Text(
                                                    end_message.clone().into(),
                                                ));
                                                if let Some(guest) = &room.guest {
                                                    let _ = guest
                                                        .tx
                                                        .send(Message::Text(end_message.into()));
                                                }
                                                let guest_id = room
                                                    .guest
                                                    .as_ref()
                                                    .map(|guest| guest.id.as_str())
                                                    .unwrap_or("");
                                                state.db.write().await.record_match(
                                                    &format!("m_{}", code),
                                                    &room.host.id,
                                                    guest_id,
                                                    Some(&winner_id),
                                                    room.round,
                                                );
                                                authority.mark_settled();
                                                rooms_guard.remove(&code);
                                            } else {
                                                room.round += 1;
                                                room.authority = None;
                                            }
                                        }
                                    }
                                    Err(error) => {
                                        let _ = tx.send(Message::Text(
                                            serde_json::json!({
                                                "type": "ERROR",
                                                "message": format!("Lệnh bị từ chối: {:?}", error)
                                            })
                                            .to_string()
                                            .into(),
                                        ));
                                    }
                                },
                                None => {
                                    let _ = tx.send(Message::Text(
                                        serde_json::json!({
                                            "type": "ERROR",
                                            "message": "Trận đấu chưa khởi tạo simulation authority."
                                        })
                                        .to_string()
                                        .into(),
                                    ));
                                }
                            }
                        }
                    }
                }

                "BATTLE_FINISHED" => {
                    let room_code_hint = parsed.get("room_code").and_then(|v| v.as_str());
                    let mut rooms_guard = state.rooms.write().await;
                    if let Some((code, role)) = find_room_and_role(
                        &rooms_guard,
                        room_code_hint,
                        &current_room,
                        &current_role,
                        &tx,
                    ) {
                        current_room = Some(code.clone());
                        current_role = Some(role.clone());

                        if let Some(room) = rooms_guard.get_mut(&code) {
                            let msg_round = parsed
                                .get("round")
                                .and_then(|v| v.as_u64())
                                .map(|v| v as usize)
                                .unwrap_or(room.round);

                            if room.settled_round >= msg_round {
                                // Already settled this round, send back current HP
                                println!(
                                    "[RUST WS] [PVP DEDUP] Round {} in room {} already settled (settled_round={}), deduplicating BATTLE_FINISHED from {}",
                                    msg_round, code, room.settled_round, role
                                );
                                let host_hp = room.host.hp;
                                let guest_hp = room.guest.as_ref().map(|g| g.hp).unwrap_or(0);
                                let sync_msg = serde_json::json!({
                                    "type": "UPDATE_MATCH_HP",
                                    "player_hp": if role == "host" { host_hp } else { guest_hp },
                                    "opponent_hp": if role == "host" { guest_hp } else { host_hp },
                                    "damage_dealt": 0
                                });
                                let _ = tx.send(Message::Text(sync_msg.to_string().into()));
                                continue;
                            }

                            room.settled_round = msg_round.max(room.round);
                            let survivors = parsed
                                .get("player_survivors")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(1) as i32;
                            let damage = 10 + survivors * 4;
                            let winner_role = parsed
                                .get("winner_role")
                                .and_then(|v| v.as_str())
                                .unwrap_or("host");

                            if winner_role == "host" {
                                if let Some(ref mut g) = room.guest {
                                    g.hp = (g.hp - damage).max(0);
                                }
                            } else if winner_role == "guest" {
                                room.host.hp = (room.host.hp - damage).max(0);
                            } else if winner_role == "draw" {
                                // Both players take tiebreaker damage on draw to prevent infinite matches
                                let draw_damage = 10;
                                room.host.hp = (room.host.hp - draw_damage).max(0);
                                if let Some(ref mut g) = room.guest {
                                    g.hp = (g.hp - draw_damage).max(0);
                                }
                                println!(
                                    "[RUST WS] Round {} in room {} was a DRAW. Both players take {} tiebreaker damage.",
                                    room.round, code, draw_damage
                                );
                            }

                            let host_hp = room.host.hp;
                            let guest_hp = room.guest.as_ref().map(|g| g.hp).unwrap_or(0);

                            // Send UPDATE_MATCH_HP to host
                            let to_host = serde_json::json!({
                                "type": "UPDATE_MATCH_HP",
                                "player_hp": host_hp,
                                "opponent_hp": guest_hp,
                                "damage_dealt": damage
                            });
                            let _ = room.host.tx.send(Message::Text(to_host.to_string().into()));

                            // Send UPDATE_MATCH_HP to guest
                            if let Some(ref g) = room.guest {
                                let to_guest = serde_json::json!({
                                    "type": "UPDATE_MATCH_HP",
                                    "player_hp": guest_hp,
                                    "opponent_hp": host_hp,
                                    "damage_dealt": damage
                                });
                                let _ = g.tx.send(Message::Text(to_guest.to_string().into()));
                            }

                            println!(
                                "[RUST WS] [BATTLE SETTLED] Round {} in room {} settled by {}. Winner: {} | Host HP: {}, Guest HP: {} (Damage: {})",
                                room.round, code, role, winner_role, host_hp, guest_hp, damage
                            );

                            const MAX_MATCH_ROUNDS: usize = 20;
                            if host_hp <= 0 || guest_hp <= 0 || room.round >= MAX_MATCH_ROUNDS {
                                let (winner_name, winner_id) = if host_hp > guest_hp {
                                    (room.host.name.clone(), room.host.id.clone())
                                } else if guest_hp > host_hp {
                                    (
                                        room.guest.as_ref().map(|g| g.name.clone()).unwrap_or_default(),
                                        room.guest.as_ref().map(|g| g.id.clone()).unwrap_or_default(),
                                    )
                                } else {
                                    // Tiebreaker at round limit or mutual elimination: host wins tiebreak
                                    (room.host.name.clone(), room.host.id.clone())
                                };

                                println!(
                                    "[RUST WS] [MATCH END] Room {} match ended after round {}. Winner: {} ({}) | Final Host HP: {}, Guest HP: {}",
                                    code, room.round, winner_name, winner_id, host_hp, guest_hp
                                );

                                let end_message = serde_json::json!({
                                    "type": "MATCH_END",
                                    "winner": winner_name,
                                    "gold_reward": 80,
                                    "consolation_gold": 25
                                })
                                .to_string();

                                let _ =
                                    room.host.tx.send(Message::Text(end_message.clone().into()));
                                if let Some(ref g) = room.guest {
                                    let _ = g.tx.send(Message::Text(end_message.into()));
                                }

                                let guest_id = room
                                    .guest
                                    .as_ref()
                                    .map(|guest| guest.id.as_str())
                                    .unwrap_or("");
                                state.db.write().await.record_match(
                                    &format!("m_{}", code),
                                    &room.host.id,
                                    guest_id,
                                    Some(&winner_id),
                                    room.round,
                                );
                                rooms_guard.remove(&code);
                            } else {
                                room.round += 1;
                                room.host.ready = false;
                                if let Some(ref mut g) = room.guest {
                                    g.ready = false;
                                }
                                room.authority = None;
                            }
                        }
                    }
                }

                "GACHA_PULL" => {
                    let count = parsed.get("count").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                    let username = parsed
                        .get("username")
                        .and_then(|v| v.as_str())
                        .unwrap_or("guest");
                    let mut prog_guard = state.player_progression.write().await;
                    let player = prog_guard
                        .entry(username.to_string())
                        .or_insert_with(|| game_logic::gacha::PlayerProgressionState::new(5000));
                    let mut rng = game_logic::gacha::GachaRng::new(rand::random());
                    match player.pull(&mut rng, count, 100) {
                        Ok(results) => {
                            let items: Vec<game_protocol::GachaPullItemData> = results
                                .into_iter()
                                .map(|r| game_protocol::GachaPullItemData {
                                    hero_id: r.hero_id,
                                    rarity: format!("{:?}", r.rarity),
                                    is_duplicate: r.is_duplicate,
                                    shards_granted: r.shards_granted,
                                    pity_before: r.pity_before,
                                    pity_after: r.pity_after,
                                })
                                .collect();
                            let msg = game_protocol::PvpMessage::GachaPullResult {
                                results: items,
                                pity_counter: player.pity_counter,
                                remaining_currency: player.currency,
                            };
                            let _ =
                                tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                        }
                        Err(err) => {
                            let msg = game_protocol::PvpMessage::Error {
                                message: format!("Gacha pull failed: {:?}", err),
                            };
                            let _ =
                                tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                        }
                    }
                }

                "HERO_UPGRADE_STAR" => {
                    let hero_id = parsed.get("hero_id").and_then(|v| v.as_str()).unwrap_or("");
                    let username = parsed
                        .get("username")
                        .and_then(|v| v.as_str())
                        .unwrap_or("guest");
                    let mut prog_guard = state.player_progression.write().await;
                    let player = prog_guard
                        .entry(username.to_string())
                        .or_insert_with(|| game_logic::gacha::PlayerProgressionState::new(5000));
                    match player.upgrade_star(hero_id) {
                        Ok(new_star) => {
                            let shards = player.heroes.get(hero_id).map(|h| h.shards).unwrap_or(0);
                            let msg = game_protocol::PvpMessage::HeroUpgradeStarResult {
                                hero_id: hero_id.to_string(),
                                new_star,
                                remaining_shards: shards,
                            };
                            let _ =
                                tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                        }
                        Err(err) => {
                            let msg = game_protocol::PvpMessage::Error {
                                message: format!("Star upgrade failed: {:?}", err),
                            };
                            let _ =
                                tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                        }
                    }
                }

                "HERO_UPGRADE_LEVEL" => {
                    let hero_id = parsed.get("hero_id").and_then(|v| v.as_str()).unwrap_or("");
                    let levels = parsed.get("levels").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                    let username = parsed
                        .get("username")
                        .and_then(|v| v.as_str())
                        .unwrap_or("guest");
                    let mut prog_guard = state.player_progression.write().await;
                    let player = prog_guard
                        .entry(username.to_string())
                        .or_insert_with(|| game_logic::gacha::PlayerProgressionState::new(5000));
                    match player.upgrade_level(hero_id, levels, 50) {
                        Ok(new_level) => {
                            let msg = game_protocol::PvpMessage::HeroUpgradeLevelResult {
                                hero_id: hero_id.to_string(),
                                new_level,
                                remaining_currency: player.currency,
                            };
                            let _ =
                                tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                        }
                        Err(err) => {
                            let msg = game_protocol::PvpMessage::Error {
                                message: format!("Level upgrade failed: {:?}", err),
                            };
                            let _ =
                                tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                        }
                    }
                }

                "PROGRESSION_SYNC" => {
                    let username = parsed
                        .get("username")
                        .and_then(|v| v.as_str())
                        .unwrap_or("guest");
                    let mut prog_guard = state.player_progression.write().await;
                    let player = prog_guard
                        .entry(username.to_string())
                        .or_insert_with(|| game_logic::gacha::PlayerProgressionState::new(5000));
                    let heroes: Vec<game_protocol::HeroProgressData> = player
                        .heroes
                        .values()
                        .map(|h| game_protocol::HeroProgressData {
                            hero_id: h.hero_id.clone(),
                            star_level: h.star_level,
                            shards: h.shards,
                            level: h.level,
                        })
                        .collect();
                    let msg = game_protocol::PvpMessage::ProgressionSync {
                        currency: player.currency,
                        pity_counter: player.pity_counter,
                        heroes,
                    };
                    let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                }

                "HERO_EMOTE" => {
                    let emote = parsed.get("emote").and_then(|v| v.as_str()).unwrap_or("👋");
                    let room_code_hint = parsed.get("room_code").and_then(|v| v.as_str());
                    let rooms_guard = state.rooms.read().await;
                    if let Some((code, role)) = find_room_and_role(
                        &rooms_guard,
                        room_code_hint,
                        &current_room,
                        &current_role,
                        &tx,
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
                }

                _ => {}
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
                    room.guest = None;
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
