use super::*;
use game_logic::{BattleState, BoardSlot, CombatEvent, TeamSide, UnitState};

pub fn battle_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut rng: ResMut<BattleRng>,
    hit_stop: Res<HitStopManager>,
    mut camera_shake: ResMut<CameraShake2d>,
    mut turn_manager: ResMut<BattleTurnManager>,
    mut adapter: ResMut<BattleSimulationAdapter>,
    mut units: Query<
        (
            Entity,
            &Unit,
            &mut UnitStats,
            &GridPos,
            &Transform,
            &mut ActionGauge,
            Option<&mut ChibiSquashStretch>,
            Option<&BattleUnitId>,
        ),
        Without<DeadUnit>,
    >,
    spotlight_query: Query<Entity, With<ActiveTurnSpotlight2d>>,
    boss_query: Query<&BossUnit>,
    dash_query: Query<&DashAnimation2d>,
    proj_query: Query<&Projectile2d>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if hit_stop.active {
        return;
    }

    let dt = time.delta_secs() * speed.multiplier;

    let has_active_dash = dash_query.iter().next().is_some();
    let has_active_proj = proj_query.iter().next().is_some();

    if turn_manager.active_attacker.is_some() {
        if !has_active_dash && !has_active_proj {
            // Watchdog auto-recovery: previous animation completed or despawned
            turn_manager.active_attacker = None;
            turn_manager.cooldown_timer.reset();
        } else {
            return;
        }
    }

    turn_manager
        .cooldown_timer
        .tick(std::time::Duration::from_secs_f32(dt));
    if !turn_manager.cooldown_timer.finished() {
        return;
    }

    for spot in spotlight_query.iter() {
        if let Some(e) = commands.get_entity(spot) { e.despawn_recursive(); }
    }

    // Helper closure to compute deterministic unit ID if BattleUnitId not yet applied
    let unit_ids: Vec<(Entity, u32)> = {
        let mut p_idx = 0usize;
        let mut e_idx = 0usize;
        units
            .iter()
            .map(|(ent, unit, _, _, _, _, _, maybe_id)| {
                let side = match unit.faction {
                    Faction::Player => TeamSide::Attacker,
                    Faction::Enemy => TeamSide::Defender,
                };
                let index = if unit.faction == Faction::Player {
                    let cur = p_idx;
                    p_idx += 1;
                    cur
                } else {
                    let cur = e_idx;
                    e_idx += 1;
                    cur
                };
                let uid = maybe_id
                    .map(|id| id.0)
                    .unwrap_or_else(|| calculate_unit_id(side, index, adapter.seed));
                (ent, uid)
            })
            .collect()
    };

    for &(entity, uid) in &unit_ids {
        if let Some(mut e) = commands.get_entity(entity) { e.insert(BattleUnitId(uid)); }
    }

    // Auto-initialize headless battle simulation for single player if not already created
    if adapter.battle_state.is_none() {
        let mut sim_state = BattleState::default();

        for (entity, unit, stats, grid, _, gauge, _, _) in units.iter() {
            let side = match unit.faction {
                Faction::Player => TeamSide::Attacker,
                Faction::Enemy => TeamSide::Defender,
            };
            let uid = unit_ids
                .iter()
                .find(|(e, _)| *e == entity)
                .map(|(_, id)| *id)
                .unwrap_or(1);

            if let Some(slot) =
                BoardSlot::new(side, (grid.col.min(2)) as u8, (grid.row.min(2)) as u8)
            {
                let mut u_state = UnitState::new(
                    uid,
                    side,
                    slot,
                    stats.max_hp.max(1.0) as u32,
                    stats.atk.max(1.0) as u32,
                    stats.def.max(0.0) as u32,
                    stats.speed.max(1.0) as u32,
                );
                u_state.hp = stats.hp.max(1.0) as u32;
                u_state.rage = stats.mana as u16;
                u_state.gauge.current = (gauge.current * 100.0) as u32;
                let _ = sim_state.add_unit(u_state);
            }
        }
        adapter.battle_state = Some(sim_state);
    }

    // Step simulation if no events are pending
    if adapter.pending_events.is_empty() {
        if let Some(ref mut battle) = adapter.battle_state {
            if let Some(winner) = battle.winner() {
                adapter.settled_winner = Some(winner);
                adapter
                    .pending_events
                    .push_back(CombatEvent::BattleEnded { winner });
            } else {
                let mut advance_limit = 0;
                while battle.next_ready_unit().is_none() && advance_limit < 1000 {
                    battle.advance_gauges();
                    advance_limit += 1;
                }

                // Synchronize visual action gauges with simulation gauges
                for u in &battle.units {
                    if let Some(&(ent, _)) = unit_ids.iter().find(|(_, id)| *id == u.id) {
                        if let Ok((.., mut visual_gauge, _, _)) = units.get_mut(ent) {
                            visual_gauge.current =
                                (u.gauge.current as f32 / 100.0).clamp(0.0, 100.0);
                        }
                    }
                }

                if let Some(actor_id) = battle.next_ready_unit() {
                    let actor_class = unit_ids
                        .iter()
                        .find(|(_, id)| *id == actor_id)
                        .and_then(|(ent, _)| units.get(*ent).ok())
                        .map(|(_, u, ..)| u.class)
                        .unwrap_or(UnitClass::Knight);

                    let (normal, ultimate) = skill_specs_for_class(actor_class);
                    let is_crit = rng.next_f32() < 0.20;
                    match battle.step(&normal, &ultimate, is_crit) {
                        Ok(events) => {
                            adapter.pending_events.extend(events);
                        }
                        Err(e) => {
                            warn!(
                                "[BATTLE] Step error with ultimate: {:?}, attempting normal skill fallback",
                                e
                            );
                            match battle.step(&normal, &normal, is_crit) {
                                Ok(events) => {
                                    adapter.pending_events.extend(events);
                                }
                                Err(e2) => {
                                    warn!(
                                        "[BATTLE] Normal skill fallback failed: {:?}, forcing gauge consumption for unit {}",
                                        e2, actor_id
                                    );
                                    if let Some(actor_u) =
                                        battle.units.iter_mut().find(|u| u.id == actor_id)
                                    {
                                        actor_u.gauge.consume_turn();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Consume authoritative events and drive visual presentation
    let mut current_actor_ent: Option<Entity> = None;
    let mut current_actor_pos = Vec2::ZERO;
    let mut current_actor_class = UnitClass::Knight;
    let mut current_is_ultimate = false;
    let mut spawned_any_attack = false;

    while let Some(event) = adapter.pending_events.front() {
        match *event {
            CombatEvent::ActionReady { unit_id } => {
                if current_actor_ent.is_some() {
                    break;
                }
                adapter.pending_events.pop_front();
                if let Some(&(ent, _)) = unit_ids.iter().find(|(_, id)| *id == unit_id) {
                    if let Ok((_, unit, _, _, transform, mut gauge, maybe_squash, _)) =
                        units.get_mut(ent)
                    {
                        current_actor_ent = Some(ent);
                        current_actor_pos = transform.translation.xy();
                        current_actor_class = unit.class;
                        gauge.current = 0.0;
                        if let Some(mut squash) = maybe_squash {
                            squash.target_scale = Vec3::new(0.85, 1.28, 1.0);
                        }
                        commands.spawn((
                            Sprite {
                                custom_size: Some(Vec2::new(84.0, 84.0)),
                                color: Color::srgba(1.0, 0.85, 0.25, 0.65),
                                ..default()
                            },
                            Transform::from_xyz(current_actor_pos.x, current_actor_pos.y, 2.0),
                            ActiveTurnSpotlight2d {
                                attacker_entity: ent,
                            },
                        ));
                        turn_manager.active_attacker = Some(ent);
                        turn_manager.acted_this_cycle.insert(ent);
                    }
                }
            }
            CombatEvent::UltimateTriggered { unit_id } => {
                adapter.pending_events.pop_front();
                current_is_ultimate = true;
                sound_events.send(PlaySoundEvent(SoundEffect::Ultimate));
                camera_shake.add_trauma(0.50);
                if let Some(&(ent, _)) = unit_ids.iter().find(|(_, id)| *id == unit_id) {
                    if let Ok((_, unit, ..)) = units.get(ent) {
                        let is_boss = boss_query.get(ent).is_ok();
                        let text = if is_boss {
                            "[BOSS TUYỆT KỸ]\nĐỘNG ĐẤT DIỆT THẾ!".to_string()
                        } else {
                            format!("[TUYỆT KỸ]\n{}!", unit.class.ultimate_name().to_uppercase())
                        };
                        spawn_floating_text(
                            &mut commands,
                            &mut rng,
                            current_actor_pos + Vec2::new(0.0, 36.0),
                            &text,
                            Color::srgb(1.0, 0.88, 0.2),
                            17.0,
                        );
                    }
                }
            }
            CombatEvent::Attack {
                attacker_id: _,
                target_id,
                critical,
            } => {
                adapter.pending_events.pop_front();

                let mut damage_amount = 20u32;
                if let Some(CombatEvent::Damage {
                    target_id: tid,
                    amount,
                    target_hp,
                    ..
                }) = adapter.pending_events.front()
                {
                    if *tid == target_id {
                        damage_amount = *amount;
                        let hp_val = *target_hp as f32;
                        adapter.pending_events.pop_front();
                        if let Some(&(t_ent, _)) = unit_ids.iter().find(|(_, id)| *id == target_id)
                        {
                            if let Ok((_, _, mut stats, ..)) = units.get_mut(t_ent) {
                                stats.hp = hp_val;
                            }
                        }
                    }
                }

                let target_info =
                    unit_ids
                        .iter()
                        .find(|(_, id)| *id == target_id)
                        .and_then(|&(t_ent, _)| {
                            units.get(t_ent).ok().map(|(_, u, _, g, t, ..)| {
                                (t_ent, t.translation.xy(), *g, u.faction)
                            })
                        });

                if let Some((target_ent, target_pos, target_grid, target_faction)) = target_info {
                    spawned_any_attack = true;
                    let is_melee = current_actor_class.is_melee();
                    if is_melee {
                        sound_events.send(PlaySoundEvent(
                            if current_actor_class == UnitClass::Assassin {
                                SoundEffect::Dagger
                            } else {
                                SoundEffect::Slash
                            },
                        ));
                        let offset_dir = (current_actor_pos - target_pos).normalize_or_zero();
                        let dash_target = target_pos + offset_dir * 46.0;

                        commands.spawn((
                            Sprite {
                                custom_size: Some(Vec2::splat(if current_is_ultimate {
                                    28.0
                                } else {
                                    18.0
                                })),
                                color: if current_is_ultimate {
                                    Color::srgba(1.0, 0.85, 0.25, 0.85)
                                } else {
                                    Color::srgba(0.85, 0.85, 0.90, 0.65)
                                },
                                ..default()
                            },
                            Transform::from_xyz(current_actor_pos.x, current_actor_pos.y, 35.0),
                            CombatVfx2d {
                                timer: Timer::from_seconds(0.20, TimerMode::Once),
                                initial_scale: Vec2::splat(1.0),
                                target_scale: Vec2::splat(2.4),
                                rotate_speed: 1.0,
                            },
                        ));

                        if let Some(actor_ent) = current_actor_ent {
                            if let Some(mut e) = commands.get_entity(actor_ent) {
                                e.insert(DashAnimation2d {
                                    origin: current_actor_pos,
                                    target: dash_target,
                                    timer: Timer::from_seconds(
                                        if current_is_ultimate { 0.30 } else { 0.25 },
                                        TimerMode::Once,
                                    ),
                                    returning: false,
                                    damage_dealt: false,
                                    target_entity: target_ent,
                                    attacker_entity: actor_ent,
                                    damage: damage_amount as f32,
                                    is_crit: critical,
                                    is_ultimate: current_is_ultimate,
                                    class: current_actor_class,
                                });
                            }
                        }
                    } else {
                        sound_events.send(PlaySoundEvent(
                            if current_actor_class.is_magic() || current_actor_class.is_healer() {
                                SoundEffect::Magic
                            } else {
                                SoundEffect::Arrow
                            },
                        ));
                        let (col, sz, arc_h, aoe) = if current_actor_class.is_magic() {
                            (
                                current_actor_class.color(),
                                Vec2::new(18.0, 18.0),
                                24.0,
                                Some((target_grid.col, target_faction)),
                            )
                        } else if current_actor_class.is_healer() {
                            (
                                Color::srgb(0.95, 0.85, 0.3),
                                Vec2::new(16.0, 16.0),
                                20.0,
                                None,
                            )
                        } else {
                            (
                                Color::srgb(0.30, 0.95, 0.40),
                                Vec2::new(22.0, 6.0),
                                35.0,
                                None,
                            )
                        };

                        commands.spawn((
                            Sprite {
                                custom_size: Some(sz),
                                color: col,
                                ..default()
                            },
                            Transform::from_xyz(current_actor_pos.x, current_actor_pos.y, 50.0),
                            Projectile2d {
                                start: current_actor_pos,
                                target_pos,
                                target_entity: target_ent,
                                timer: Timer::from_seconds(0.32, TimerMode::Once),
                                damage: damage_amount as f32,
                                is_heal: false,
                                is_crit: critical,
                                is_ultimate: current_is_ultimate,
                                aoe_row: aoe,
                                arc_height: arc_h,
                                class: current_actor_class,
                            },
                        ));
                    }
                }
            }
            CombatEvent::Damage {
                target_id,
                amount,
                target_hp,
                ..
            } => {
                adapter.pending_events.pop_front();
                if let Some(&(t_ent, _)) = unit_ids.iter().find(|(_, id)| *id == target_id) {
                    if let Ok((_, _, mut stats, _, transform, ..)) = units.get_mut(t_ent) {
                        stats.hp = target_hp as f32;
                        spawn_floating_text(
                            &mut commands,
                            &mut rng,
                            transform.translation.xy() + Vec2::new(0.0, 28.0),
                            &format!("-{}", amount),
                            Color::srgb(1.0, 0.35, 0.35),
                            14.0,
                        );
                    }
                }
            }
            CombatEvent::Heal {
                target_id,
                amount,
                target_hp,
                ..
            } => {
                adapter.pending_events.pop_front();
                sound_events.send(PlaySoundEvent(SoundEffect::Heal));
                if let Some(&(t_ent, _)) = unit_ids.iter().find(|(_, id)| *id == target_id) {
                    if let Ok((_, _, mut stats, _, transform, ..)) = units.get_mut(t_ent) {
                        stats.hp = target_hp as f32;
                        spawn_floating_text(
                            &mut commands,
                            &mut rng,
                            transform.translation.xy() + Vec2::new(0.0, 28.0),
                            &format!("+{}", amount),
                            Color::srgb(0.25, 0.95, 0.35),
                            15.0,
                        );
                    }
                }
            }
            CombatEvent::RageChanged { unit_id, rage } => {
                adapter.pending_events.pop_front();
                if let Some(&(u_ent, _)) = unit_ids.iter().find(|(_, id)| *id == unit_id) {
                    if let Ok((_, _, mut stats, ..)) = units.get_mut(u_ent) {
                        stats.mana = rage as f32;
                    }
                }
            }
            CombatEvent::Stunned { unit_id } => {
                adapter.pending_events.pop_front();
                if let Some(&(u_ent, _)) = unit_ids.iter().find(|(_, id)| *id == unit_id) {
                    if let Ok((_, _, _, _, transform, ..)) = units.get(u_ent) {
                        spawn_floating_text(
                            &mut commands,
                            &mut rng,
                            transform.translation.xy() + Vec2::new(0.0, 32.0),
                            "CHOÁNG!",
                            Color::srgb(1.0, 0.85, 0.2),
                            14.0,
                        );
                    }
                }
            }
            CombatEvent::StatusApplied { .. } => {
                adapter.pending_events.pop_front();
            }
            CombatEvent::Defeated { unit_id } => {
                adapter.pending_events.pop_front();
                if let Some(&(u_ent, _)) = unit_ids.iter().find(|(_, id)| *id == unit_id) {
                    if let Some(mut e) = commands.get_entity(u_ent) { e.insert(DeadUnit); }
                }
            }
            CombatEvent::BattleEnded { winner } => {
                adapter.pending_events.pop_front();
                adapter.settled_winner = Some(winner);
                break;
            }
        }
    }

    if current_actor_ent.is_some() && !spawned_any_attack {
        turn_manager.active_attacker = None;
        turn_manager.cooldown_timer.reset();
    }
}
