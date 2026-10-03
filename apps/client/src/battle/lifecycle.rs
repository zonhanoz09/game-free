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
    mut player_units: Query<(&Unit, &mut UnitStats, &mut crate::units::ActiveStatusEffects), (With<GridPos>, Without<DeadUnit>)>,
    mut adapter: ResMut<BattleSimulationAdapter>,
    pvp_mgr: Res<crate::net::PvpManager>,
    rng: Res<BattleRng>,
) {
    turn_manager.active_attacker = None;
    turn_manager.cooldown_timer.reset();
    turn_manager.acted_this_cycle.clear();
    turn_manager.cycle_turn_count = 0;

    adapter.reset(
        pvp_mgr.active,
        rng.state,
        pvp_mgr.authoritative_battle_id.clone(),
    );

    for entity in units.iter() {
        if let Some(mut e) = commands.get_entity(entity) { e.insert(ActionGauge { current: 0.0 }); }
    }

    let mut syn_counts = std::collections::HashMap::new();
    for (unit, _, _) in player_units.iter() {
        if unit.faction == Faction::Player {
            let syn = crate::synergies::class_to_synergy(unit.class);
            *syn_counts.entry(syn).or_insert(0) += 1;
        }
    }

    let vanguard_active = syn_counts
        .get(&crate::synergies::SynergyType::Vanguard)
        .copied()
        .unwrap_or(0)
        >= 2;
    let sharpshooter_active = syn_counts
        .get(&crate::synergies::SynergyType::Sharpshooter)
        .copied()
        .unwrap_or(0)
        >= 2;
    let arcanist_active = syn_counts
        .get(&crate::synergies::SynergyType::Arcanist)
        .copied()
        .unwrap_or(0)
        >= 2;
    let shadow_active = syn_counts
        .get(&crate::synergies::SynergyType::Shadow)
        .copied()
        .unwrap_or(0)
        >= 2;
    let divine_active = syn_counts
        .get(&crate::synergies::SynergyType::Divine)
        .copied()
        .unwrap_or(0)
        >= 1;

    info!("==================== [BATTLE START] ====================");
    info!(
        "[BATTLE START] Team Synergies Active -> Vanguard: {}, Sharpshooter: {}, Arcanist: {}, Shadow: {}, Divine: {}",
        vanguard_active, sharpshooter_active, arcanist_active, shadow_active, divine_active
    );

    for (unit, mut stats, mut fx) in player_units.iter_mut() {
        if unit.faction == Faction::Player {
            let syn = crate::synergies::class_to_synergy(unit.class);
            if vanguard_active {
                stats.def += if syn == crate::synergies::SynergyType::Vanguard {
                    35.0
                } else {
                    15.0
                };
                fx.add("syn_vanguard", "🔰", "Thiết Vệ", 99);
            }
            if sharpshooter_active && syn == crate::synergies::SynergyType::Sharpshooter {
                stats.atk *= 1.25;
                stats.crit_rate += 0.15;
                fx.add("syn_sharpshooter", "🏹", "Thần Xạ", 99);
            }
            if arcanist_active && syn == crate::synergies::SynergyType::Arcanist {
                stats.atk *= 1.30;
                stats.mana = 30.0;
                fx.add("syn_arcanist", "⚡", "Kỳ Môn", 99);
            }
            if shadow_active && syn == crate::synergies::SynergyType::Shadow {
                stats.crit_rate += 0.25;
                fx.add("syn_shadow", "🗡️", "Ám Ảnh", 99);
            }
            if divine_active {
                stats.hp = (stats.hp + 20.0).min(stats.max_hp);
                fx.add("syn_divine", "⚕️", "Thần Ân", 99);
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
    mut adapter: ResMut<BattleSimulationAdapter>,
    mut all_effects: Query<&mut crate::units::ActiveStatusEffects>,
) {
    for mut fx in all_effects.iter_mut() {
        fx.clear();
    }
    turn_manager.active_attacker = None;
    adapter.battle_state = None;
    adapter.pending_events.clear();

    for entity in projectiles.iter() {
        if let Some(e) = commands.get_entity(entity) { e.despawn_recursive(); }
    }
    for entity in floating_texts.iter() {
        if let Some(e) = commands.get_entity(entity) { e.despawn_recursive(); }
    }
    for entity in vfx_query.iter() {
        if let Some(e) = commands.get_entity(entity) { e.despawn_recursive(); }
    }
    for entity in sparks_query.iter() {
        if let Some(e) = commands.get_entity(entity) { e.despawn_recursive(); }
    }
    for entity in spotlight_query.iter() {
        if let Some(e) = commands.get_entity(entity) { e.despawn_recursive(); }
    }
    for (entity, dash) in dashes.iter() {
        if let Ok(mut transform) = transforms.get_mut(entity) {
            transform.translation.x = dash.origin.x;
            transform.translation.y = dash.origin.y;
        }
        if let Some(mut e) = commands.get_entity(entity) { e.remove::<DashAnimation2d>(); }
    }
}
