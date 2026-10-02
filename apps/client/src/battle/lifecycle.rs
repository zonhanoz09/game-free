use super::*;

pub fn update_hit_stop_system(time: Res<Time>, mut hit_stop: ResMut<HitStopManager>) {
    hit_stop.update(time.delta_secs());
}

pub fn update_camera_shake(
    time: Res<Time>,
    mut shake: ResMut<CameraShake2d>,
    mut rng: ResMut<BattleRng>,
    mut cam_query: Query<&mut Transform, With<crate::board::MainCamera2d>>,
) {
    let dt = time.delta_secs();
    shake.trauma = (shake.trauma - dt * 2.8).max(0.0);
    let intensity = shake.trauma * shake.trauma;

    let Ok(mut transform) = cam_query.get_single_mut() else {
        return;
    };

    if intensity > 0.001 {
        let ox = rng.random_range(-1.0, 1.0) * intensity * 12.0;
        let oy = rng.random_range(-1.0, 1.0) * intensity * 10.0;
        transform.translation.x = ox;
        transform.translation.y = oy;
    } else {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
}

pub fn on_enter_battle(
    mut commands: Commands,
    units: Query<Entity, (With<Unit>, With<GridPos>)>,
    mut turn_manager: ResMut<BattleTurnManager>,
    mut player_units: Query<(&Unit, &mut UnitStats), (With<GridPos>, Without<DeadUnit>)>,
) {
    turn_manager.active_attacker = None;
    turn_manager.cooldown_timer.reset();
    turn_manager.acted_this_cycle.clear();
    turn_manager.cycle_turn_count = 0;

    for entity in units.iter() {
        commands.entity(entity).insert(ActionGauge { current: 0.0 });
    }

    let mut class_counts = std::collections::HashMap::new();
    for (unit, _) in player_units.iter() {
        if unit.faction == Faction::Player {
            *class_counts.entry(unit.class).or_insert(0) += 1;
        }
    }

    let vanguard_active = class_counts.get(&UnitClass::Knight).copied().unwrap_or(0) >= 2;
    let sharpshooter_active = class_counts.get(&UnitClass::Archer).copied().unwrap_or(0) >= 2;
    let arcanist_active = class_counts.get(&UnitClass::Mage).copied().unwrap_or(0) >= 2;
    let shadow_active = class_counts.get(&UnitClass::Assassin).copied().unwrap_or(0) >= 2;
    let divine_active = class_counts.get(&UnitClass::Cleric).copied().unwrap_or(0) >= 1;

    info!("==================== [BATTLE START] ====================");
    info!(
        "[BATTLE START] Team Synergies Active -> Vanguard: {}, Sharpshooter: {}, Arcanist: {}, Shadow: {}, Divine: {}",
        vanguard_active, sharpshooter_active, arcanist_active, shadow_active, divine_active
    );

    for (unit, mut stats) in player_units.iter_mut() {
        if unit.faction == Faction::Player {
            if vanguard_active {
                stats.def += if unit.class == UnitClass::Knight {
                    35.0
                } else {
                    15.0
                };
            }
            if sharpshooter_active && unit.class == UnitClass::Archer {
                stats.atk *= 1.25;
                stats.crit_rate += 0.15;
            }
            if arcanist_active && unit.class == UnitClass::Mage {
                stats.atk *= 1.30;
                stats.mana = 30.0;
            }
            if shadow_active && unit.class == UnitClass::Assassin {
                stats.crit_rate += 0.25;
            }
            if divine_active {
                stats.hp = (stats.hp + 20.0).min(stats.max_hp);
            }
        }
    }
}

pub fn on_exit_battle(
    mut commands: Commands,
    projectiles: Query<Entity, With<Projectile2d>>,
    floating_texts: Query<Entity, With<FloatingText2d>>,
    vfx_query: Query<Entity, With<CombatVfx2d>>,
    sparks_query: Query<Entity, With<SparkParticle2d>>,
    spotlight_query: Query<Entity, With<ActiveTurnSpotlight2d>>,
    dashes: Query<(Entity, &DashAnimation2d)>,
    mut turn_manager: ResMut<BattleTurnManager>,
    mut transforms: Query<&mut Transform>,
) {
    turn_manager.active_attacker = None;

    for entity in projectiles.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in floating_texts.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in vfx_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in sparks_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in spotlight_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for (entity, dash) in dashes.iter() {
        if let Ok(mut transform) = transforms.get_mut(entity) {
            transform.translation.x = dash.origin.x;
            transform.translation.y = dash.origin.y;
        }
        commands.entity(entity).remove::<DashAnimation2d>();
    }
}
