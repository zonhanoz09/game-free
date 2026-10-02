use crate::audio::PlaySoundEvent;
use crate::battle::ActionGauge;
use crate::board::grid_to_world_pos;
use crate::types::*;
use crate::units::{Unit, spawn_unit, spawn_unit_ext};
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
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PvpMessage {
    #[serde(alias = "CreateRoom", alias = "CREATE_ROOM")]
    CreateRoom { room_code: String },
    #[serde(alias = "JoinRoom", alias = "JOIN_ROOM")]
    JoinRoom { room_code: String },
    #[serde(alias = "RoomJoined", alias = "ROOM_JOINED")]
    RoomJoined {
        room_code: String,
        role: String,
        player_name: String,
        opponent_name: String,
    },
    #[serde(alias = "PlayerReady", alias = "PLAYER_READY")]
    PlayerReady { lineup: Vec<PvpUnitData> },
    #[serde(alias = "StartRound", alias = "START_ROUND")]
    StartRound {
        round: usize,
        opponent_lineup: Vec<PvpUnitData>,
        player_hp: i32,
        opponent_hp: i32,
    },
    #[serde(alias = "BattleFinished", alias = "BATTLE_FINISHED")]
    BattleFinished {
        winner_role: String,
        player_survivors: usize,
    },
    #[serde(alias = "UpdateMatchHp", alias = "UPDATE_MATCH_HP")]
    UpdateMatchHp {
        player_hp: i32,
        opponent_hp: i32,
        damage_dealt: i32,
    },
    #[serde(alias = "MatchEnd", alias = "MATCH_END")]
    MatchEnd { winner: String },
    #[serde(alias = "Error", alias = "ERROR")]
    Error { message: String },
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
    _state: Res<State<GameState>>,
    mut commands: Commands,
    textures: Res<GameTextures>,
    all_board_units: Query<(Entity, &Unit), With<GridPos>>,
    mut player_units: Query<
        (
            Entity,
            &Unit,
            &GridPos,
            &mut Transform,
            &mut Visibility,
            &mut UnitStats,
            Option<&mut ActionGauge>,
        ),
        Without<DeadUnit>,
    >,
    dead_player_units: Query<(Entity, &Unit, &GridPos), With<DeadUnit>>,
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
                    info!(
                        "[PVP] Room joined: {} as {}",
                        pvp_mgr.room_code, pvp_mgr.role
                    );
                }
                PvpMessage::StartRound {
                    round,
                    opponent_lineup,
                    player_hp,
                    opponent_hp,
                } => {
                    pvp_mgr.active = true;
                    pvp_mgr.round = round;
                    pvp_mgr.player_hp = player_hp;
                    pvp_mgr.opponent_hp = opponent_hp;
                    pvp_mgr.opponent_lineup = opponent_lineup.clone();
                    pvp_mgr.is_ready = false;
                    pvp_mgr.opponent_ready = false;

                    // 1. Despawn ONLY existing enemy units on board (DO NOT delete player units!)
                    for (ent, unit) in all_board_units.iter() {
                        if unit.faction == Faction::Enemy {
                            commands.entity(ent).despawn_recursive();
                        }
                    }

                    // 2. Reset player units to full health & reset Action Gauge
                    for (_, unit, grid, mut transform, mut vis, mut stats, maybe_gauge) in
                        player_units.iter_mut()
                    {
                        if unit.faction == Faction::Player {
                            let pos = grid_to_world_pos(grid.col, grid.row, Faction::Player);
                            transform.translation =
                                Vec3::new(pos.x, pos.y, 10.0 + (grid.row as f32 * -0.5));
                            *vis = Visibility::Inherited;
                            stats.hp = stats.max_hp;
                            stats.shield = 0.0;
                            stats.mana = 0.0;
                            if let Some(mut gauge) = maybe_gauge {
                                gauge.current = 0.0;
                            }
                        }
                    }

                    // 3. Respawn any dead player units
                    for (entity, unit, grid) in dead_player_units.iter() {
                        if unit.faction == Faction::Player {
                            commands.entity(entity).despawn_recursive();
                            spawn_unit(
                                &mut commands,
                                &textures,
                                unit.class,
                                Faction::Player,
                                grid.col,
                                grid.row,
                            );
                        }
                    }

                    // 4. Spawn opponent lineup on enemy side!
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

                    // Fallback: If opponent has no units, spawn starter enemy squad
                    if opponent_lineup.is_empty() {
                        spawn_unit_ext(
                            &mut commands,
                            &textures,
                            UnitClass::Knight,
                            Faction::Enemy,
                            0,
                            0,
                            1,
                            false,
                        );
                        spawn_unit_ext(
                            &mut commands,
                            &textures,
                            UnitClass::Archer,
                            Faction::Enemy,
                            1,
                            1,
                            1,
                            false,
                        );
                        spawn_unit_ext(
                            &mut commands,
                            &textures,
                            UnitClass::Assassin,
                            Faction::Enemy,
                            2,
                            2,
                            1,
                            false,
                        );
                    }

                    sound_events.send(PlaySoundEvent(crate::audio::SoundEffect::Click));
                    next_state.set(GameState::Battle);
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
                _ => {}
            }
        } else {
            warn!("[PVP NET] Failed to parse message JSON: {}", raw);
        }
    }
}
