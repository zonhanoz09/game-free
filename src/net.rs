use crate::audio::PlaySoundEvent;
use crate::types::*;
use crate::units::{spawn_unit_ext, Unit};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PvpUnitData {
    pub col: usize,
    pub row: usize,
    pub class: UnitClass,
    pub star_level: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", content = "data")]
pub enum PvpMessage {
    CreateRoom {
        room_code: String,
    },
    JoinRoom {
        room_code: String,
    },
    RoomJoined {
        room_code: String,
        role: String,
        player_name: String,
        opponent_name: String,
    },
    PlayerReady {
        lineup: Vec<PvpUnitData>,
    },
    StartRound {
        round: usize,
        opponent_lineup: Vec<PvpUnitData>,
        player_hp: i32,
        opponent_hp: i32,
    },
    BattleFinished {
        winner_role: String,
        player_survivors: usize,
    },
    UpdateMatchHp {
        player_hp: i32,
        opponent_hp: i32,
        damage_dealt: i32,
    },
    MatchEnd {
        winner: String,
    },
    Error {
        message: String,
    },
}

#[derive(Resource, Debug)]
pub struct PvpManager {
    pub active: bool,
    pub room_code: String,
    pub role: String,
    pub player_name: String,
    pub opponent_name: String,
    pub player_hp: i32,
    pub opponent_hp: i32,
    pub round: usize,
    pub opponent_lineup: Vec<PvpUnitData>,
    pub is_ready: bool,
    pub opponent_ready: bool,
    pub match_winner: Option<String>,
}

impl Default for PvpManager {
    fn default() -> Self {
        Self {
            active: false,
            room_code: String::new(),
            role: "host".to_string(),
            player_name: "Player 1 (Blue)".to_string(),
            opponent_name: "Waiting for Challenger...".to_string(),
            player_hp: 100,
            opponent_hp: 100,
            round: 1,
            opponent_lineup: Vec::new(),
            is_ready: false,
            opponent_ready: false,
            match_winner: None,
        }
    }
}

static INCOMING_PVP_MSGS: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn js_to_rust_pvp(msg: &str) {
    if let Ok(mut q) = INCOMING_PVP_MSGS.lock() {
        q.push(msg.to_string());
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = rust_to_js_pvp)]
    pub fn rust_to_js_pvp(msg: &str);
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn rust_to_js_pvp(_msg: &str) {
    // Desktop stub
}

pub fn send_pvp_message(msg: &PvpMessage) {
    if let Ok(json) = serde_json::to_string(msg) {
        rust_to_js_pvp(&json);
    }
}

pub fn pvp_network_system(
    mut pvp_mgr: ResMut<PvpManager>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    mut commands: Commands,
    textures: Res<GameTextures>,
    enemy_units: Query<Entity, (With<Unit>, With<GridPos>)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    let mut messages = Vec::new();
    if let Ok(mut q) = INCOMING_PVP_MSGS.lock() {
        if !q.is_empty() {
            messages.append(&mut *q);
        }
    }

    for raw in messages {
        info!("[PVP NET] Received message: {}", raw);
        if let Ok(msg) = serde_json::from_str::<PvpMessage>(&raw) {
            match msg {
                PvpMessage::RoomJoined {
                    room_code,
                    role,
                    player_name,
                    opponent_name,
                } => {
                    pvp_mgr.active = true;
                    pvp_mgr.room_code = room_code;
                    pvp_mgr.role = role;
                    pvp_mgr.player_name = player_name;
                    pvp_mgr.opponent_name = opponent_name;
                    pvp_mgr.player_hp = 100;
                    pvp_mgr.opponent_hp = 100;
                    pvp_mgr.round = 1;
                    pvp_mgr.is_ready = false;
                    pvp_mgr.opponent_ready = false;
                    pvp_mgr.match_winner = None;
                    info!("[PVP] Room joined: {} as {}", pvp_mgr.room_code, pvp_mgr.role);
                }
                PvpMessage::StartRound {
                    round,
                    opponent_lineup,
                    player_hp,
                    opponent_hp,
                } => {
                    pvp_mgr.round = round;
                    pvp_mgr.player_hp = player_hp;
                    pvp_mgr.opponent_hp = opponent_hp;
                    pvp_mgr.opponent_lineup = opponent_lineup.clone();
                    pvp_mgr.is_ready = false;
                    pvp_mgr.opponent_ready = false;

                    // Despawn any existing enemies on board
                    for ent in enemy_units.iter() {
                        commands.entity(ent).despawn_recursive();
                    }

                    // Spawn opponent lineup on enemy side!
                    for u in &opponent_lineup {
                        spawn_unit_ext(
                            &mut commands,
                            &textures,
                            u.class,
                            Faction::Enemy,
                            u.col,
                            u.row,
                            u.star_level,
                            false,
                        );
                    }

                    sound_events.send(PlaySoundEvent(crate::audio::SoundEffect::Click));
                    if *state.get() == GameState::Placement {
                        next_state.set(GameState::Battle);
                    }
                    info!(
                        "[PVP] Round {} battle started with {} opponent units!",
                        round,
                        opponent_lineup.len()
                    );
                }
                PvpMessage::UpdateMatchHp {
                    player_hp,
                    opponent_hp,
                    damage_dealt,
                } => {
                    pvp_mgr.player_hp = player_hp;
                    pvp_mgr.opponent_hp = opponent_hp;
                    info!(
                        "[PVP] Match HP updated: Player {} HP, Opponent {} HP (Damage: {})",
                        player_hp, opponent_hp, damage_dealt
                    );
                }
                PvpMessage::MatchEnd { winner } => {
                    pvp_mgr.match_winner = Some(winner.clone());
                    info!("[PVP] Match ended! Winner: {}", winner);
                }
                PvpMessage::Error { message } => {
                    warn!("[PVP NET ERROR] {}", message);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pvp_manager_initial_state() {
        let mgr = PvpManager::default();
        assert_eq!(mgr.player_hp, 100);
        assert_eq!(mgr.opponent_hp, 100);
        assert_eq!(mgr.round, 1);
        assert!(!mgr.active);
        assert!(!mgr.is_ready);
    }

    #[test]
    fn test_pvp_message_serialization() {
        let msg = PvpMessage::StartRound {
            round: 2,
            opponent_lineup: vec![
                PvpUnitData {
                    col: 1,
                    row: 0,
                    class: UnitClass::Knight,
                    star_level: 2,
                },
                PvpUnitData {
                    col: 0,
                    row: 2,
                    class: UnitClass::Mage,
                    star_level: 1,
                },
            ],
            player_hp: 92,
            opponent_hp: 84,
        };

        let json = serde_json::to_string(&msg).expect("Serialize failed");
        assert!(json.contains("StartRound"));
        assert!(json.contains("Knight"));

        let deserialized: PvpMessage = serde_json::from_str(&json).expect("Deserialize failed");
        match deserialized {
            PvpMessage::StartRound { round, opponent_lineup, player_hp, opponent_hp } => {
                assert_eq!(round, 2);
                assert_eq!(player_hp, 92);
                assert_eq!(opponent_hp, 84);
                assert_eq!(opponent_lineup.len(), 2);
                assert_eq!(opponent_lineup[0].class, UnitClass::Knight);
                assert_eq!(opponent_lineup[0].star_level, 2);
            }
            _ => panic!("Incorrect message variant"),
        }
    }
}
