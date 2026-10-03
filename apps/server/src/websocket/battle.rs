//! WebSocket battle message handlers (PLAYER_READY, BATTLE_COMMAND, BATTLE_FINISHED).

use crate::{AppState, authority::AuthoritativeBattle, multiplayer::UnitData};
use super::stable_battle_seed;
use axum::extract::ws::Message;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn handle_battle_message(
    msg_type: &str,
    parsed: &serde_json::Value,
    state: &Arc<AppState>,
    tx: &mpsc::UnboundedSender<Message>,
    current_room: &mut Option<String>,
    current_role: &mut Option<String>,
) -> bool {
    match msg_type {
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
                    if let Some((code, role)) = super::find_room_and_role(
                        &rooms_guard,
                        room_code_hint,
                        current_room,
                        current_role,
                        tx,
                    ) {
                        *current_room = Some(code.clone());
                        *current_role = Some(role.clone());

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
                    true
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
                            return true;
                        }
                    };
                    let room_code_hint = parsed.get("room_code").and_then(|v| v.as_str());
                    let mut rooms_guard = state.rooms.write().await;
                    if let Some((code, _)) = super::find_room_and_role(
                        &rooms_guard,
                        room_code_hint,
                        current_room,
                        current_role,
                        tx,
                    ) {
                        *current_room = Some(code.clone());
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
                                                return true;
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
                    true
                }

                "BATTLE_FINISHED" => {
                    let room_code_hint = parsed.get("room_code").and_then(|v| v.as_str());
                    let mut rooms_guard = state.rooms.write().await;
                    if let Some((code, role)) = super::find_room_and_role(
                        &rooms_guard,
                        room_code_hint,
                        current_room,
                        current_role,
                        tx,
                    ) {
                        *current_room = Some(code.clone());
                        *current_role = Some(role.clone());

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
                                return true;
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
                    true
                }


        _ => false,
    }
}
