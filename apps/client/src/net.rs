use crate::audio::PlaySoundEvent;
use crate::battle::ActionGauge;
use crate::board::grid_to_world_pos;
use crate::types::*;
use crate::units::{Unit, spawn_unit, spawn_unit_ext};
use bevy::prelude::*;
pub use game_protocol::{DeckCardData, PvpMessage, PvpUnitData};
use std::sync::Mutex;

#[derive(Resource, Debug, Default)]
pub struct PlayerDeck {
    pub cards: Vec<DeckCardData>,
}

#[derive(Event, Clone, Debug)]
pub struct AuthoritativeReplayEvent(pub serde_json::Value);

fn parse_unit_class(class: &str) -> UnitClass {
    match class {
        "zhao_yun" | "Knight" => UnitClass::Knight,
        "huang_zhong" | "Archer" => UnitClass::Archer,
        "zhuge_liang" | "Mage" => UnitClass::Mage,
        "zhang_he_yan_liang" | "zhang_he" | "Assassin" => UnitClass::Assassin,
        "hua_tuo" | "Cleric" => UnitClass::Cleric,
        "cao_cao" => UnitClass::CaoCao,
        "dian_wei" => UnitClass::DianWei,
        "guo_jia" => UnitClass::GuoJia,
        "sun_ce" => UnitClass::SunCe,
        "lu_xun" => UnitClass::LuXun,
        "da_qiao_xiao_qiao" => UnitClass::DaQiaoXiaoQiao,
        "jia_xu" => UnitClass::JiaXu,
        "Triệu Vân" => UnitClass::Knight,
        "Hoàng Trung" => UnitClass::Archer,
        "Gia Cát Lượng" => UnitClass::Mage,
        "Trương Cáp & Nhan Lương" | "Trương Cáp" => UnitClass::Assassin,
        "Hoa Đà" => UnitClass::Cleric,
        "Tào Tháo" => UnitClass::CaoCao,
        "Điển Vi" => UnitClass::DianWei,
        "Quách Gia" => UnitClass::GuoJia,
        "Tôn Sách" => UnitClass::SunCe,
        "Lục Tốn" => UnitClass::LuXun,
        "Đại Kiều & Tiểu Kiều" | "Đại Kiều" => UnitClass::DaQiaoXiaoQiao,
        "Giả Hủ" => UnitClass::JiaXu,
        _ => UnitClass::Knight,
    }
}

pub fn consume_authoritative_replay_events(
    mut events: EventReader<AuthoritativeReplayEvent>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    for AuthoritativeReplayEvent(event) in events.read() {
        let kind = event
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        match kind {
            "UltimateTriggered" => {
                sound_events.send(PlaySoundEvent(crate::audio::SoundEffect::Ultimate));
            }
            "Heal" => {
                sound_events.send(PlaySoundEvent(crate::audio::SoundEffect::Heal));
            }
            _ => {}
        }
        info!("[PVP REPLAY] Consumed authoritative event {}", kind);
    }
}

pub fn general_unit_class(class: &str) -> UnitClass {
    parse_unit_class(class)
}

/// Formation slots are numbered by tier (front, middle, back), then
/// top-to-bottom within that tier.
pub fn formation_position_to_grid(position: usize, faction: Faction) -> (usize, usize) {
    let position = position.min(8);
    let tier = position / 3;
    let row = position % 3;
    let col = match faction {
        Faction::Player => 2 - tier,
        Faction::Enemy => tier,
    };
    (col, row)
}

#[allow(dead_code)]
pub fn grid_to_formation_position(col: usize, row: usize, faction: Faction) -> usize {
    let tier = match faction {
        Faction::Player => 2 - col.min(2),
        Faction::Enemy => col.min(2),
    };
    tier * 3 + row.min(2)
}

#[derive(Resource, Default, Debug, Clone)]
pub struct GachaClientState {
    pub currency: u64,
    pub pity_counter: u32,
    pub recent_pulls: Vec<game_protocol::GachaPullItemData>,
    pub heroes: Vec<game_protocol::HeroProgressData>,
    #[allow(dead_code)]
    pub last_error: Option<String>,
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
    pub authoritative_battle_id: Option<String>,
    pub authoritative_turn: u32,
    pub authoritative_events: Vec<serde_json::Value>,
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
            authoritative_battle_id: None,
            authoritative_turn: 0,
            authoritative_events: Vec::new(),
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
        info!("[PVP OUTGOING] Sending to JS: {}", json);
        rust_to_js_pvp(&json);
    }
}

pub fn pvp_network_system(
    mut pvp_mgr: ResMut<PvpManager>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    mut commands: Commands,
    textures: Res<GameTextures>,
    mut player_deck: ResMut<PlayerDeck>,
    mut battle_speed: ResMut<BattleSpeed>,
    all_board_units: Query<(Entity, &Unit), With<GridPos>>,
    bench_units: Query<(Entity, &BenchPos), Without<DeadUnit>>,
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
    mut replay_events: EventWriter<AuthoritativeReplayEvent>,
    mut adapter: ResMut<crate::battle::BattleSimulationAdapter>,
    mut gacha_state: ResMut<GachaClientState>,
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
                PvpMessage::SetDeck { cards } => {
                    info!(
                        "[DECK] Received player deck with {} cards from profile!",
                        cards.len()
                    );
                    player_deck.cards = cards.clone();

                    if *state.get() == GameState::Placement && !cards.is_empty() {
                        for (ent, unit) in all_board_units.iter() {
                            if unit.faction == Faction::Player {
                                if let Some(e) = commands.get_entity(ent) {
                                    e.despawn_recursive();
                                }
                            }
                        }

                        for (ent, _) in bench_units.iter() {
                            if let Some(e) = commands.get_entity(ent) {
                                e.despawn_recursive();
                            }
                        }

                        for (idx, card) in cards.iter().enumerate() {
                            let unit_class = general_unit_class(&card.hero_class);

                            if let Some(position) = card.position {
                                let (col, row) =
                                    formation_position_to_grid(position, Faction::Player);
                                crate::units::spawn_unit_ext_bonus_with_initiative(
                                    &mut commands,
                                    &textures,
                                    unit_class,
                                    Faction::Player,
                                    col,
                                    row,
                                    card.star_level.max(1),
                                    false,
                                    card.hp_bonus,
                                    card.atk_bonus,
                                    card.initiative_bonus,
                                );
                            } else {
                                let slot = idx.min(5);
                                crate::units::spawn_bench_unit_bonus(
                                    &mut commands,
                                    &textures,
                                    unit_class,
                                    slot,
                                    card.star_level.max(1),
                                    card.hp_bonus,
                                    card.atk_bonus,
                                );
                            }
                        }
                    }
                }
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
                    pvp_mgr.authoritative_battle_id = None;
                    pvp_mgr.authoritative_turn = 0;
                    pvp_mgr.authoritative_events.clear();
                    pvp_mgr.opponent_lineup.clear();

                    // Immediately clear any single-player bot enemy units from the board!
                    for (ent, unit) in all_board_units.iter() {
                        if unit.faction == Faction::Enemy {
                            if let Some(e) = commands.get_entity(ent) {
                                e.despawn_recursive();
                            }
                        }
                    }

                    info!(
                        "[PVP] Room joined: {} as {} vs {}",
                        pvp_mgr.room_code, pvp_mgr.role, pvp_mgr.opponent_name
                    );
                }
                PvpMessage::OpponentReady => {
                    pvp_mgr.opponent_ready = true;
                    info!("[PVP] Opponent is ready!");
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
                            if let Some(e) = commands.get_entity(ent) {
                                e.despawn_recursive();
                            }
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
                        let mirrored_col = (2usize).saturating_sub(u.col).min(2);
                        spawn_unit_ext(
                            &mut commands,
                            &textures,
                            parse_unit_class(&u.class),
                            Faction::Enemy,
                            mirrored_col,
                            u.row,
                            u.star_level,
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
                    pvp_mgr.round += 1;
                    pvp_mgr.is_ready = false;
                    pvp_mgr.opponent_ready = false;

                    // Clear defeated enemies from board
                    for (ent, unit) in all_board_units.iter() {
                        if unit.faction == Faction::Enemy {
                            if let Some(e) = commands.get_entity(ent) {
                                e.despawn_recursive();
                            }
                        }
                    }

                    info!(
                        "[PVP] Match HP updated: Player {} HP, Opponent {} HP (Damage: {}) -> Next Round {}",
                        player_hp, opponent_hp, damage_dealt, pvp_mgr.round
                    );
                    next_state.set(GameState::Placement);
                }
                PvpMessage::BattleResult { result } => {
                    let is_local_winner = result.winner.as_deref().is_some_and(|winner| {
                        (winner == "ATTACKER" && pvp_mgr.role == "host")
                            || (winner == "DEFENDER" && pvp_mgr.role == "guest")
                    });
                    pvp_mgr.authoritative_battle_id = Some(result.battle_id.clone());
                    pvp_mgr.authoritative_turn = result.turn;
                    pvp_mgr.authoritative_events = result.replay.events.clone();
                    adapter.is_pvp = true;
                    adapter.battle_id = Some(result.battle_id.clone());
                    adapter.authoritative_turn = result.turn;
                    for event in &result.replay.events {
                        replay_events.send(AuthoritativeReplayEvent(event.clone()));
                        if let Ok(combat_event) =
                            serde_json::from_value::<game_logic::CombatEvent>(event.clone())
                        {
                            adapter.pending_events.push_back(combat_event);
                        }
                    }
                    if let Some(winner) = result.winner.as_deref() {
                        let side = match winner {
                            "ATTACKER" => Some(game_logic::TeamSide::Attacker),
                            "DEFENDER" => Some(game_logic::TeamSide::Defender),
                            _ => None,
                        };
                        adapter.settled_winner = Some(side);
                    }
                    if is_local_winner {
                        info!(
                            "[PVP AUTHORITY] Battle {} resolved in our favor at turn {} ({} events)",
                            result.battle_id,
                            result.turn,
                            result.replay.events.len()
                        );
                    } else if result.winner.is_some() {
                        info!(
                            "[PVP AUTHORITY] Battle {} resolved against us at turn {} ({} events)",
                            result.battle_id,
                            result.turn,
                            result.replay.events.len()
                        );
                    } else {
                        info!(
                            "[PVP AUTHORITY] Battle {} replay advanced to turn {}",
                            result.battle_id, result.turn
                        );
                    }
                }
                PvpMessage::MatchEnd { winner } => {
                    pvp_mgr.match_winner = Some(winner.clone());
                    let is_win = winner.to_lowercase() == pvp_mgr.player_name.to_lowercase()
                        || (winner == "host" && pvp_mgr.role == "host")
                        || (winner == "guest" && pvp_mgr.role == "guest");
                    if is_win {
                        next_state.set(GameState::Victory);
                    } else {
                        next_state.set(GameState::Defeat);
                    }
                    info!("[PVP] Match ended! Winner: {} (is_win: {})", winner, is_win);
                }

                PvpMessage::SetSpeed { speed } => {
                    info!("[PVP SPEED] Changed battle speed to: {}", speed);
                    battle_speed.multiplier = speed.clamp(0.2, 3.0);
                }
                PvpMessage::GachaPullResult {
                    results,
                    pity_counter,
                    remaining_currency,
                } => {
                    info!(
                        "[GACHA] Received {} pulls from server, pity={}",
                        results.len(),
                        pity_counter
                    );
                    gacha_state.recent_pulls = results;
                    gacha_state.pity_counter = pity_counter;
                    gacha_state.currency = remaining_currency;
                }
                PvpMessage::HeroUpgradeStarResult {
                    hero_id,
                    new_star,
                    remaining_shards,
                } => {
                    info!("[PROG] Hero {} upgraded to star {}", hero_id, new_star);
                    if let Some(h) = gacha_state.heroes.iter_mut().find(|h| h.hero_id == hero_id) {
                        h.star_level = new_star;
                        h.shards = remaining_shards;
                    }
                }
                PvpMessage::HeroUpgradeLevelResult {
                    hero_id,
                    new_level,
                    remaining_currency,
                } => {
                    info!("[PROG] Hero {} upgraded to level {}", hero_id, new_level);
                    gacha_state.currency = remaining_currency;
                    if let Some(h) = gacha_state.heroes.iter_mut().find(|h| h.hero_id == hero_id) {
                        h.level = new_level;
                    }
                }
                PvpMessage::ProgressionSync {
                    currency,
                    pity_counter,
                    heroes,
                } => {
                    info!(
                        "[PROG] Sync: currency={}, pity={}, heroes={}",
                        currency,
                        pity_counter,
                        heroes.len()
                    );
                    gacha_state.currency = currency;
                    gacha_state.pity_counter = pity_counter;
                    gacha_state.heroes = heroes;
                }
                PvpMessage::ExitMatch => {
                    info!("[PVP] Exit match, reset state to Placement");
                    pvp_mgr.active = false;
                    pvp_mgr.is_ready = false;
                    pvp_mgr.opponent_ready = false;
                    pvp_mgr.round = 1;
                    pvp_mgr.match_winner = None;
                    adapter.is_pvp = false;
                    adapter.battle_state = None;
                    adapter.settled_winner = None;
                    adapter.pending_events.clear();
                    adapter.reset(false, 0, None);
                    for (ent, unit) in all_board_units.iter() {
                        if unit.faction == Faction::Enemy {
                            if let Some(e) = commands.get_entity(ent) {
                                e.despawn_recursive();
                            }
                        }
                    }
                    next_state.set(GameState::Placement);
                }
                PvpMessage::StartBattle => {
                    info!("[PVP/PVE] StartBattle requested from UI!");
                    if pvp_mgr.active {
                        if !pvp_mgr.is_ready {
                            let mut lineup = Vec::new();
                            for (_, unit, grid, _, _, _, _) in player_units.iter() {
                                if unit.faction == Faction::Player {
                                    lineup.push(game_protocol::PvpUnitData {
                                        col: grid.col,
                                        row: grid.row,
                                        class: unit.class.id_str().to_string(),
                                        star_level: 1,
                                    });
                                }
                            }
                            if lineup.is_empty() {
                                for card in &player_deck.cards {
                                    let (col, row) = match card.position {
                                        Some(pos) => formation_position_to_grid(pos, Faction::Player),
                                        None => (2, 0),
                                    };
                                    lineup.push(game_protocol::PvpUnitData {
                                        col,
                                        row,
                                        class: general_unit_class(&card.hero_class).id_str().to_string(),
                                        star_level: card.star_level.max(1),
                                    });
                                }
                            }
                            if lineup.is_empty() {
                                lineup.push(game_protocol::PvpUnitData {
                                    col: 2,
                                    row: 0,
                                    class: UnitClass::Knight.id_str().to_string(),
                                    star_level: 1,
                                });
                            }
                            pvp_mgr.is_ready = true;
                            send_pvp_message(&game_protocol::PvpMessage::PlayerReady { lineup });
                            info!("[PVP] Ready & Locked In triggered from UI StartBattle!");
                        }
                    } else {
                        let enemy_count = all_board_units
                            .iter()
                            .filter(|(_, u)| u.faction == Faction::Enemy)
                            .count();
                        if enemy_count == 0 {
                            let ai_squad = [
                                (UnitClass::Knight, 0, 1),
                                (UnitClass::Archer, 1, 0),
                                (UnitClass::Mage, 2, 2),
                            ];
                            for (u_class, col, row) in ai_squad {
                                crate::units::spawn_unit_ext(
                                    &mut commands,
                                    &textures,
                                    u_class,
                                    Faction::Enemy,
                                    col,
                                    row,
                                    1,
                                    false,
                                );
                            }
                        }

                        let player_count = all_board_units
                            .iter()
                            .filter(|(_, u)| u.faction == Faction::Player)
                            .count();
                        if player_count == 0 {
                            if !player_deck.cards.is_empty() {
                                for (idx, card) in player_deck.cards.iter().enumerate() {
                                    let unit_class = general_unit_class(&card.hero_class);
                                    let (col, row) = match card.position {
                                        Some(pos) => formation_position_to_grid(pos, Faction::Player),
                                        None => formation_position_to_grid(idx * 3, Faction::Player),
                                    };
                                    crate::units::spawn_unit_ext_bonus_with_initiative(
                                        &mut commands,
                                        &textures,
                                        unit_class,
                                        Faction::Player,
                                        col,
                                        row,
                                        card.star_level.max(1),
                                        false,
                                        card.hp_bonus,
                                        card.atk_bonus,
                                        card.initiative_bonus,
                                    );
                                }
                            } else {
                                crate::units::spawn_unit(
                                    &mut commands,
                                    &textures,
                                    UnitClass::Knight,
                                    Faction::Player,
                                    2,
                                    1,
                                );
                            }
                        }

                        adapter.reset(false, 12345, None);
                        next_state.set(GameState::Battle);
                    }
                }
                _ => {}
            }
        } else {
            warn!("[PVP NET] Failed to parse message JSON: {}", raw);
        }
    }
}
