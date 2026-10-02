use super::*;

#[test]
fn test_battle_tick_system_finds_units_and_initiates_attack() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<PlaySoundEvent>();
    app.init_resource::<BattleTurnManager>();
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
            ActionGauge { current: 100.0 }, // near full
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
    // Also tick ActionGauge manually or let Time run
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(500));
    app.update();

    // Check if ActionGauge increased or attacker was activated
    let turn_mgr = app.world().resource::<BattleTurnManager>();
    assert_eq!(
        turn_mgr.active_attacker,
        Some(player_ent),
        "Player knight should have triggered an attack!"
    );
}
