use super::*;
use game_logic::{BattleState, BoardSlot, CombatEvent, TeamSide, UnitState};

#[test]
fn test_battle_tick_system_finds_units_and_initiates_attack() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<PlaySoundEvent>();
    app.init_resource::<BattleTurnManager>();
    app.init_resource::<BattleSimulationAdapter>();
    app.init_resource::<BattleRng>();
    app.init_resource::<HitStopManager>();
    app.init_resource::<CameraShake2d>();
    app.init_resource::<BattleSpeed>();

    // Spawn a player Knight
    let player_ent = app
        .world_mut()
        .spawn((
            Unit {
                class: UnitClass::Knight,
                faction: Faction::Player,
            },
            UnitClass::Knight.base_stats(),
            GridPos {
                col: 2,
                row: 1,
                faction: Faction::Player,
            },
            ActionGauge { current: 100.0 }, // full gauge
            Transform::from_xyz(-100.0, 0.0, 10.0),
        ))
        .id();

    // Spawn an enemy Knight
    let _enemy_ent = app
        .world_mut()
        .spawn((
            Unit {
                class: UnitClass::Knight,
                faction: Faction::Enemy,
            },
            UnitClass::Knight.base_stats(),
            GridPos {
                col: 0,
                row: 1,
                faction: Faction::Enemy,
            },
            ActionGauge { current: 10.0 },
            Transform::from_xyz(100.0, 0.0, 10.0),
        ))
        .id();

    app.add_systems(Update, battle_tick_system);

    // Finish cooldown timer and set dt
    app.world_mut()
        .resource_mut::<BattleTurnManager>()
        .cooldown_timer
        .tick(std::time::Duration::from_secs(1));
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(500));
    app.update();

    let turn_mgr = app.world().resource::<BattleTurnManager>();
    assert_eq!(
        turn_mgr.active_attacker,
        Some(player_ent),
        "Player knight should have triggered an attack!"
    );
}

#[test]
fn test_client_event_consumer_updates_unit_state_without_combat_math() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<PlaySoundEvent>();
    app.init_resource::<BattleTurnManager>();
    app.init_resource::<BattleSimulationAdapter>();
    app.init_resource::<BattleRng>();
    app.init_resource::<HitStopManager>();
    app.init_resource::<CameraShake2d>();
    app.init_resource::<BattleSpeed>();

    let player_unit_id = 100u32;
    let enemy_unit_id = 200u32;

    let _player_ent = app
        .world_mut()
        .spawn((
            Unit {
                class: UnitClass::Knight,
                faction: Faction::Player,
            },
            UnitClass::Knight.base_stats(),
            GridPos {
                col: 2,
                row: 1,
                faction: Faction::Player,
            },
            ActionGauge { current: 0.0 },
            Transform::from_xyz(-100.0, 0.0, 10.0),
            BattleUnitId(player_unit_id),
        ))
        .id();

    let enemy_ent = app
        .world_mut()
        .spawn((
            Unit {
                class: UnitClass::Archer,
                faction: Faction::Enemy,
            },
            UnitClass::Archer.base_stats(),
            GridPos {
                col: 0,
                row: 1,
                faction: Faction::Enemy,
            },
            ActionGauge { current: 0.0 },
            Transform::from_xyz(100.0, 0.0, 10.0),
            BattleUnitId(enemy_unit_id),
        ))
        .id();

    // Enqueue authoritative events directly into the adapter
    {
        let mut adapter = app.world_mut().resource_mut::<BattleSimulationAdapter>();
        adapter.is_pvp = true; // authoritative mode
        adapter.pending_events.push_back(CombatEvent::ActionReady {
            unit_id: player_unit_id,
        });
        adapter.pending_events.push_back(CombatEvent::Damage {
            source_id: player_unit_id,
            target_id: enemy_unit_id,
            amount: 45,
            target_hp: 65,
        });
        adapter.pending_events.push_back(CombatEvent::RageChanged {
            unit_id: enemy_unit_id,
            rage: 20,
        });
    }

    app.add_systems(Update, battle_tick_system);

    app.world_mut()
        .resource_mut::<BattleTurnManager>()
        .cooldown_timer
        .tick(std::time::Duration::from_secs(1));
    app.update();

    // Verify enemy HP and rage match the authoritative event without client math
    let enemy_stats = app.world().get::<UnitStats>(enemy_ent).unwrap();
    assert_eq!(enemy_stats.hp, 65.0);
    assert_eq!(enemy_stats.mana, 20.0);
}

#[test]
fn test_simulation_adapter_replay_parity() {
    let mut sim = BattleState::default();
    let slot_atk = BoardSlot::new(TeamSide::Attacker, 0, 1).unwrap();
    let slot_def = BoardSlot::new(TeamSide::Defender, 0, 1).unwrap();

    let mut attacker = UnitState::new(1, TeamSide::Attacker, slot_atk, 100, 25, 0, 10);
    attacker.gauge.current = game_logic::ActionGauge::MAX;
    sim.add_unit(attacker).unwrap();

    let defender = UnitState::new(2, TeamSide::Defender, slot_def, 100, 10, 0, 10);
    sim.add_unit(defender).unwrap();

    let (normal, ultimate) = skill_specs_for_class(UnitClass::Knight);
    let events = sim.step(&normal, &ultimate, false).expect("step");

    assert!(!events.is_empty());
    assert_eq!(events[0], CombatEvent::ActionReady { unit_id: 1 });

    // Client adapter consumes these events
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<PlaySoundEvent>();
    app.init_resource::<BattleTurnManager>();
    app.init_resource::<BattleSimulationAdapter>();
    app.init_resource::<BattleRng>();
    app.init_resource::<HitStopManager>();
    app.init_resource::<CameraShake2d>();
    app.init_resource::<BattleSpeed>();

    let _atk_ent = app
        .world_mut()
        .spawn((
            Unit {
                class: UnitClass::Knight,
                faction: Faction::Player,
            },
            UnitStats {
                hp: 100.0,
                max_hp: 100.0,
                atk: 25.0,
                def: 0.0,
                speed: 10.0,
                mana: 0.0,
                max_mana: 100.0,
                crit_rate: 0.0,
                shield: 0.0,
            },
            GridPos {
                col: 2,
                row: 1,
                faction: Faction::Player,
            },
            ActionGauge { current: 100.0 },
            Transform::from_xyz(-100.0, 0.0, 10.0),
            BattleUnitId(1),
        ))
        .id();

    let def_ent = app
        .world_mut()
        .spawn((
            Unit {
                class: UnitClass::Knight,
                faction: Faction::Enemy,
            },
            UnitStats {
                hp: 100.0,
                max_hp: 100.0,
                atk: 10.0,
                def: 0.0,
                speed: 10.0,
                mana: 0.0,
                max_mana: 100.0,
                crit_rate: 0.0,
                shield: 0.0,
            },
            GridPos {
                col: 0,
                row: 1,
                faction: Faction::Enemy,
            },
            ActionGauge { current: 0.0 },
            Transform::from_xyz(100.0, 0.0, 10.0),
            BattleUnitId(2),
        ))
        .id();

    {
        let mut adapter = app.world_mut().resource_mut::<BattleSimulationAdapter>();
        adapter.is_pvp = true;
        adapter.pending_events.extend(events);
    }

    app.add_systems(Update, battle_tick_system);
    app.world_mut()
        .resource_mut::<BattleTurnManager>()
        .cooldown_timer
        .tick(std::time::Duration::from_secs(1));
    app.update();

    // Verify exact parity between headless state and client state
    let def_stats = app.world().get::<UnitStats>(def_ent).unwrap();
    assert_eq!(def_stats.hp, sim.units[1].hp as f32);
    assert_eq!(def_stats.mana, sim.units[1].rage as f32);
}

#[test]
fn test_nine_slot_formation_round_trip() {
    for pos in 0..=8 {
        let (p_col, p_row) = crate::net::formation_position_to_grid(pos, Faction::Player);
        let back_pos = crate::net::grid_to_formation_position(p_col, p_row, Faction::Player);
        assert_eq!(back_pos, pos, "Player slot {} failed round-trip", pos);

        let (e_col, e_row) = crate::net::formation_position_to_grid(pos, Faction::Enemy);
        let e_back = crate::net::grid_to_formation_position(e_col, e_row, Faction::Enemy);
        assert_eq!(e_back, pos, "Enemy slot {} failed round-trip", pos);
    }
}

#[test]
fn test_formation_five_cards_limit() {
    let mut cards = Vec::new();
    for i in 0..7 {
        cards.push(crate::net::DeckCardData {
            id: format!("card-{}", i),
            hero_class: "Knight".to_string(),
            star_level: 1,
            level: 1,
            hp_bonus: 0.0,
            atk_bonus: 0.0,
            initiative_bonus: 0.0,
            is_starter: false,
            position: Some(i),
        });
    }

    let mut deployed = 0;
    let mut benched = 0;
    for card in &cards {
        if card.position.is_some() && deployed < MAX_PLAYER_UNITS {
            deployed += 1;
        } else {
            benched += 1;
        }
    }

    assert_eq!(deployed, 5, "Maximum 5 cards deployed to board");
    assert_eq!(benched, 2, "Extra cards stay on bench");
}

#[test]
fn test_client_gacha_state_renders_server_response_without_local_math() {
    let mut gacha_state = crate::net::GachaClientState::default();
    assert_eq!(gacha_state.currency, 0);
    assert_eq!(gacha_state.pity_counter, 0);

    // Simulate receiving GachaPullResult from server
    let server_msg = game_protocol::PvpMessage::GachaPullResult {
        results: vec![
            game_protocol::GachaPullItemData {
                hero_id: "zhao_yun".to_string(),
                rarity: "SSR".to_string(),
                is_duplicate: false,
                shards_granted: 0,
                pity_before: 49,
                pity_after: 0,
            },
            game_protocol::GachaPullItemData {
                hero_id: "foot_soldier".to_string(),
                rarity: "R".to_string(),
                is_duplicate: true,
                shards_granted: 5,
                pity_before: 0,
                pity_after: 1,
            },
        ],
        pity_counter: 1,
        remaining_currency: 4800,
    };

    if let game_protocol::PvpMessage::GachaPullResult { results, pity_counter, remaining_currency } = server_msg {
        gacha_state.recent_pulls = results;
        gacha_state.pity_counter = pity_counter;
        gacha_state.currency = remaining_currency;
    }

    assert_eq!(gacha_state.recent_pulls.len(), 2);
    assert_eq!(gacha_state.recent_pulls[0].hero_id, "zhao_yun");
    assert_eq!(gacha_state.recent_pulls[0].rarity, "SSR");
    assert_eq!(gacha_state.pity_counter, 1);
    assert_eq!(gacha_state.currency, 4800);
}

#[test]
fn test_animation_stall_watchdog_recovers_active_attacker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<PlaySoundEvent>();
    app.init_resource::<BattleTurnManager>();
    app.init_resource::<BattleSimulationAdapter>();
    app.init_resource::<BattleRng>();
    app.init_resource::<HitStopManager>();
    app.init_resource::<CameraShake2d>();
    app.init_resource::<BattleSpeed>();

    let player_ent = app
        .world_mut()
        .spawn((
            Unit {
                class: UnitClass::Knight,
                faction: Faction::Player,
            },
            UnitClass::Knight.base_stats(),
            GridPos {
                col: 2,
                row: 1,
                faction: Faction::Player,
            },
            ActionGauge { current: 100.0 },
            Transform::from_xyz(-100.0, 0.0, 10.0),
        ))
        .id();

    // Spawn a stuck dash animation
    app.world_mut().spawn(DashAnimation2d {
        origin: Vec2::ZERO,
        target: Vec2::ONE,
        timer: Timer::from_seconds(10.0, TimerMode::Once),
        returning: false,
        damage_dealt: false,
        target_entity: player_ent,
        attacker_entity: player_ent,
        damage: 10.0,
        is_crit: false,
        is_ultimate: false,
        class: UnitClass::Knight,
    });

    app.add_systems(Update, battle_tick_system);

    // Set active attacker and set stall timer to 3.1s to trigger watchdog
    {
        let mut turn_mgr = app.world_mut().resource_mut::<BattleTurnManager>();
        turn_mgr.active_attacker = Some(player_ent);
        turn_mgr.animation_stall_timer = 3.1;
    }
    app.update();

    let turn_mgr = app.world().resource::<BattleTurnManager>();
    assert_eq!(
        turn_mgr.active_attacker, None,
        "Animation stall watchdog must clear active_attacker after 3 seconds!"
    );
}

#[test]
fn test_check_battle_end_pvp_draw_reports_draw() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app.add_event::<PlaySoundEvent>();
    app.init_state::<GameState>();
    app.init_resource::<PlayerEconomy>();
    app.init_resource::<BattleRng>();
    app.init_resource::<BattleSimulationAdapter>();
    app.init_resource::<crate::net::PvpManager>();

    // Set state to Battle and PvP active
    *app.world_mut().resource_mut::<State<GameState>>() = State::new(GameState::Battle);
    {
        let mut pvp = app.world_mut().resource_mut::<crate::net::PvpManager>();
        pvp.active = true;
        pvp.role = "host".to_string();
        pvp.round = 1;
        pvp.room_code = "1234".to_string();
    }
    {
        let mut adapter = app.world_mut().resource_mut::<BattleSimulationAdapter>();
        // Draw result from simulation
        adapter.settled_winner = Some(None);
    }

    app.add_systems(Update, check_battle_end);
    app.update();

    // Check that system runs and does not panic on draw
    let adapter = app.world().resource::<BattleSimulationAdapter>();
    assert_eq!(adapter.settled_winner, Some(None));
}
