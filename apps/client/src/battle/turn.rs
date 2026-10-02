use super::*;

pub fn battle_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut rng: ResMut<BattleRng>,
    hit_stop: Res<HitStopManager>,
    mut camera_shake: ResMut<CameraShake2d>,
    mut turn_manager: ResMut<BattleTurnManager>,
    mut units: Query<
        (
            Entity,
            &Unit,
            &mut UnitStats,
            &GridPos,
            &Transform,
            &mut ActionGauge,
            Option<&mut ChibiSquashStretch>,
        ),
        Without<DeadUnit>,
    >,
    spotlight_query: Query<Entity, With<ActiveTurnSpotlight2d>>,
    boss_query: Query<&BossUnit>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if hit_stop.active {
        return;
    }

    let dt = time.delta_secs() * speed.multiplier;

    if turn_manager.active_attacker.is_some() {
        return;
    }

    turn_manager
        .cooldown_timer
        .tick(std::time::Duration::from_secs_f32(dt));
    if !turn_manager.cooldown_timer.finished() {
        return;
    }

    for spot in spotlight_query.iter() {
        commands.entity(spot).despawn_recursive();
    }

    for (_, _, stats, _, _, mut gauge, _) in units.iter_mut() {
        gauge.current += stats.speed * 8.5 * dt;
    }

    let mut best_candidate: Option<(Entity, UnitClass, Faction, UnitStats, GridPos, Vec2)> = None;
    let mut highest_gauge = 99.99f32;

    for (entity, unit, stats, grid, transform, gauge, _) in units.iter() {
        if gauge.current >= 100.0 && gauge.current > highest_gauge {
            highest_gauge = gauge.current;
            best_candidate = Some((
                entity,
                unit.class,
                unit.faction,
                *stats,
                *grid,
                transform.translation.xy(),
            ));
        }
    }

    let Some((actor_entity, class, faction, stats, grid, actor_pos)) = best_candidate else {
        return;
    };

    let is_ultimate = stats.mana >= stats.max_mana;
    if is_ultimate {
        info!(
            "[ACTION] [ULTIMATE] {:?} {:?} cast ULTIMATE: {}!",
            faction,
            class,
            class.ultimate_name()
        );
    } else {
        info!(
            "[ACTION] {:?} {:?} took turn (Speed: {:.0}, ATB full)",
            faction, class, stats.speed
        );
    }

    if let Ok((_, _, mut actor_stats, _, _, mut gauge, maybe_squash)) = units.get_mut(actor_entity)
    {
        gauge.current -= 100.0;
        if let Some(mut squash) = maybe_squash {
            squash.target_scale = Vec3::new(0.85, 1.28, 1.0);
        }
        if is_ultimate {
            actor_stats.mana = 0.0;
            if class == UnitClass::Knight {
                actor_stats.shield += 80.0;
            }
        } else {
            actor_stats.mana = (actor_stats.mana + 25.0).min(actor_stats.max_mana);
        }
    }

    turn_manager.active_attacker = Some(actor_entity);
    turn_manager.acted_this_cycle.insert(actor_entity);

    let living_units: std::collections::HashSet<Entity> = units.iter().map(|(e, ..)| e).collect();
    turn_manager
        .acted_this_cycle
        .retain(|e| living_units.contains(e));
    let living_count = living_units.len();

    if living_count > 0 && turn_manager.acted_this_cycle.len() >= living_count {
        turn_manager.cycle_turn_count += 1;
        turn_manager.acted_this_cycle.clear();
        crate::net::rust_to_js_pvp(&format!(
            r#"{{"type":"ROUND_TURN_COMPLETED","turn":{},"total":{}}}"#,
            turn_manager.cycle_turn_count, living_count
        ));
    } else {
        crate::net::rust_to_js_pvp(&format!(
            r#"{{"type":"STRIKE_ACTION","acted":{},"total":{},"current_turn":{}}}"#,
            turn_manager.acted_this_cycle.len(),
            living_count,
            turn_manager.cycle_turn_count
        ));
    }

    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(84.0, 84.0)),
            color: Color::srgba(1.0, 0.85, 0.25, 0.65),
            ..default()
        },
        Transform::from_xyz(actor_pos.x, actor_pos.y, 2.0),
        ActiveTurnSpotlight2d {
            attacker_entity: actor_entity,
        },
    ));

    let is_boss_unit = boss_query.get(actor_entity).is_ok();
    if is_ultimate {
        sound_events.send(PlaySoundEvent(SoundEffect::Ultimate));
        if is_boss_unit {
            spawn_floating_text(
                &mut commands,
                &mut rng,
                actor_pos + Vec2::new(0.0, 42.0),
                "[BOSS ULTIMATE]\nCATACLYSMIC EARTHQUAKE!",
                Color::srgb(1.0, 0.25, 0.25),
                21.0,
            );
            camera_shake.add_trauma(0.85);
        } else {
            spawn_floating_text(
                &mut commands,
                &mut rng,
                actor_pos + Vec2::new(0.0, 36.0),
                &format!("[ULTIMATE]\n{}!", class.ultimate_name().to_uppercase()),
                Color::srgb(1.0, 0.88, 0.2),
                17.0,
            );
            camera_shake.add_trauma(0.50);
        }
    } else {
        match class {
            UnitClass::Cleric => sound_events.send(PlaySoundEvent(SoundEffect::Heal)),
            UnitClass::Archer => sound_events.send(PlaySoundEvent(SoundEffect::Arrow)),
            UnitClass::Mage => sound_events.send(PlaySoundEvent(SoundEffect::Magic)),
            UnitClass::Knight => sound_events.send(PlaySoundEvent(SoundEffect::Slash)),
            UnitClass::Assassin => sound_events.send(PlaySoundEvent(SoundEffect::Dagger)),
        };
    }

    let opponent_faction = match faction {
        Faction::Player => Faction::Enemy,
        Faction::Enemy => Faction::Player,
    };

    struct TargetSnapshot {
        entity: Entity,
        faction: Faction,
        hp: f32,
        max_hp: f32,
        grid: GridPos,
        pos: Vec2,
    }

    let target_list: Vec<TargetSnapshot> = units
        .iter()
        .map(|(e, u, s, g, t, _, _)| TargetSnapshot {
            entity: e,
            faction: u.faction,
            hp: s.hp,
            max_hp: s.max_hp,
            grid: *g,
            pos: t.translation.xy(),
        })
        .collect();

    // Case 1: Cleric
    if class == UnitClass::Cleric {
        if is_ultimate {
            let heal_amount = 45.0 + stats.atk * 1.60;
            for t in target_list.iter() {
                if t.faction == faction && t.hp > 0.0 {
                    let t_pos = t.pos;
                    let t_entity = t.entity;
                    commands.spawn((
                        Sprite {
                            custom_size: Some(Vec2::splat(44.0)),
                            color: Color::srgba(1.0, 0.92, 0.35, 0.85),
                            ..default()
                        },
                        Transform::from_xyz(t_pos.x, t_pos.y, 38.0),
                        CombatVfx2d {
                            timer: Timer::from_seconds(0.40, TimerMode::Once),
                            initial_scale: Vec2::splat(0.4),
                            target_scale: Vec2::splat(2.5),
                            rotate_speed: 2.0,
                        },
                    ));
                    commands.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(18.0, 28.0)),
                            color: Color::srgb(1.0, 0.98, 0.5),
                            ..default()
                        },
                        Transform::from_xyz(t_pos.x, t_pos.y + 110.0, 50.0),
                        Projectile2d {
                            start: t_pos + Vec2::new(0.0, 110.0),
                            target_pos: t_pos,
                            target_entity: t_entity,
                            timer: Timer::from_seconds(0.28, TimerMode::Once),
                            damage: heal_amount,
                            is_heal: true,
                            is_crit: true,
                            is_ultimate: true,
                            aoe_row: None,
                            arc_height: 10.0,
                            class,
                        },
                    ));
                }
            }
            for (_, u, _, _, _, mut a_gauge, _) in units.iter_mut() {
                if u.faction == faction {
                    a_gauge.current = (a_gauge.current + 25.0).min(100.0);
                }
            }
        } else {
            let mut lowest_ally: Option<(Entity, f32, Vec2)> = None;
            for t in target_list.iter() {
                if t.faction == faction && t.hp > 0.0 {
                    let hp_ratio = t.hp / t.max_hp;
                    if lowest_ally.is_none() || hp_ratio < lowest_ally.unwrap().1 {
                        lowest_ally = Some((t.entity, hp_ratio, t.pos));
                    }
                }
            }

            if let Some((target_entity, _, target_pos)) = lowest_ally {
                let heal_amount = 32.0 + stats.atk * 0.50;
                commands.spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(20.0, 20.0)),
                        color: Color::srgb(1.0, 0.95, 0.4),
                        ..default()
                    },
                    Transform::from_xyz(actor_pos.x, actor_pos.y, 50.0),
                    Projectile2d {
                        start: actor_pos,
                        target_pos,
                        target_entity,
                        timer: Timer::from_seconds(0.38, TimerMode::Once),
                        damage: heal_amount,
                        is_heal: true,
                        is_crit: false,
                        is_ultimate: false,
                        aoe_row: None,
                        arc_height: 38.0,
                        class,
                    },
                ));
            } else {
                turn_manager.active_attacker = None;
                turn_manager.cooldown_timer.reset();
            }
        }
        return;
    }

    // Case 2: Archer / Mage Ultimate All-Target Attacks
    if is_ultimate && class == UnitClass::Archer {
        let mut count = 0;
        for t in target_list.iter() {
            if t.faction == opponent_faction && t.hp > 0.0 {
                let t_pos = t.pos;
                let t_entity = t.entity;
                let raw_dmg = stats.atk * 1.35;
                commands.spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(24.0, 7.0)),
                        color: Color::srgb(0.25, 1.0, 0.45),
                        ..default()
                    },
                    Transform::from_xyz(actor_pos.x, actor_pos.y, 50.0),
                    Projectile2d {
                        start: actor_pos + Vec2::new(0.0, 20.0),
                        target_pos: t_pos,
                        target_entity: t_entity,
                        timer: Timer::from_seconds(0.32 + count as f32 * 0.05, TimerMode::Once),
                        damage: raw_dmg,
                        is_heal: false,
                        is_crit: true,
                        is_ultimate: true,
                        aoe_row: None,
                        arc_height: 60.0 + count as f32 * 8.0,
                        class,
                    },
                ));
                count += 1;
            }
        }
        if count == 0 {
            turn_manager.active_attacker = None;
            turn_manager.cooldown_timer.reset();
        }
        return;
    }

    if is_ultimate && class == UnitClass::Mage {
        let mut count = 0;
        for t in target_list.iter() {
            if t.faction == opponent_faction && t.hp > 0.0 {
                let t_pos = t.pos;
                let t_entity = t.entity;
                let raw_dmg = stats.atk * 1.60;
                commands.spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(26.0, 32.0)),
                        color: Color::srgb(0.9, 0.4, 1.0),
                        ..default()
                    },
                    Transform::from_xyz(t_pos.x, t_pos.y + 130.0, 50.0),
                    Projectile2d {
                        start: t_pos + Vec2::new(0.0, 130.0),
                        target_pos: t_pos,
                        target_entity: t_entity,
                        timer: Timer::from_seconds(0.26 + count as f32 * 0.04, TimerMode::Once),
                        damage: raw_dmg,
                        is_heal: false,
                        is_crit: true,
                        is_ultimate: true,
                        aoe_row: None,
                        arc_height: 8.0,
                        class,
                    },
                ));
                count += 1;
            }
        }
        if count == 0 {
            turn_manager.active_attacker = None;
            turn_manager.cooldown_timer.reset();
        }
        return;
    }

    // Case 3: Single / Focused Attacks
    let mut target_candidate: Option<(Entity, Vec2, GridPos)> = None;

    match class {
        UnitClass::Assassin => {
            let mut best_backline_dist = -1i32;
            let mut lowest_hp = f32::MAX;

            for t in target_list.iter() {
                if t.faction == opponent_faction && t.hp > 0.0 {
                    let depth = match opponent_faction {
                        Faction::Enemy => t.grid.col as i32,
                        Faction::Player => 2 - t.grid.col as i32,
                    };

                    if depth > best_backline_dist
                        || (depth == best_backline_dist && t.hp < lowest_hp)
                    {
                        best_backline_dist = depth;
                        lowest_hp = t.hp;
                        target_candidate = Some((t.entity, t.pos, t.grid));
                    }
                }
            }
        }
        UnitClass::Archer => {
            let mut lowest_hp = f32::MAX;

            for t in target_list.iter() {
                if t.faction == opponent_faction && t.hp > 0.0 {
                    if t.hp < lowest_hp {
                        lowest_hp = t.hp;
                        target_candidate = Some((t.entity, t.pos, t.grid));
                    }
                }
            }
        }
        UnitClass::Knight | UnitClass::Mage => {
            let mut min_col = usize::MAX;
            let mut best_row_diff = usize::MAX;

            for t in target_list.iter() {
                if t.faction == opponent_faction && t.hp > 0.0 {
                    let frontline_col = match opponent_faction {
                        Faction::Player => 2 - t.grid.col,
                        Faction::Enemy => t.grid.col,
                    };

                    let row_diff = (grid.row as i32 - t.grid.row as i32).unsigned_abs() as usize;

                    if frontline_col < min_col
                        || (frontline_col == min_col && row_diff < best_row_diff)
                    {
                        min_col = frontline_col;
                        best_row_diff = row_diff;
                        target_candidate = Some((t.entity, t.pos, t.grid));
                    }
                }
            }
        }
        UnitClass::Cleric => {}
    }

    if let Some((target_entity, target_pos, target_grid)) = target_candidate {
        let is_crit = is_ultimate || rng.next_f32() < stats.crit_rate;
        let crit_mult = if is_crit { 1.5 } else { 1.0 };
        let mult = if is_ultimate {
            match class {
                UnitClass::Knight => 2.20,
                UnitClass::Assassin => 2.80,
                _ => 1.50,
            }
        } else {
            1.0
        };
        let raw_dmg = stats.atk * mult * crit_mult;

        let is_melee = class == UnitClass::Knight || class == UnitClass::Assassin;

        if is_melee {
            let offset_dir = (actor_pos - target_pos).normalize_or_zero();
            let dash_target = target_pos + offset_dir * 46.0;

            commands.spawn((
                Sprite {
                    custom_size: Some(Vec2::splat(if is_ultimate { 28.0 } else { 18.0 })),
                    color: if is_ultimate {
                        Color::srgba(1.0, 0.85, 0.25, 0.85)
                    } else {
                        Color::srgba(0.85, 0.85, 0.90, 0.65)
                    },
                    ..default()
                },
                Transform::from_xyz(actor_pos.x, actor_pos.y, 35.0),
                CombatVfx2d {
                    timer: Timer::from_seconds(0.20, TimerMode::Once),
                    initial_scale: Vec2::splat(1.0),
                    target_scale: Vec2::splat(2.4),
                    rotate_speed: 1.0,
                },
            ));

            commands.entity(actor_entity).insert(DashAnimation2d {
                origin: actor_pos,
                target: dash_target,
                timer: Timer::from_seconds(if is_ultimate { 0.30 } else { 0.25 }, TimerMode::Once),
                returning: false,
                damage_dealt: false,
                target_entity,
                attacker_entity: actor_entity,
                damage: raw_dmg,
                is_crit,
                is_ultimate,
                class,
            });
        } else {
            let (col, sz, arc_h, aoe) = if class == UnitClass::Mage {
                (
                    Color::srgb(0.75, 0.30, 1.0),
                    Vec2::new(18.0, 18.0),
                    24.0,
                    Some((target_grid.row, opponent_faction)),
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
                Transform::from_xyz(actor_pos.x, actor_pos.y, 50.0),
                Projectile2d {
                    start: actor_pos,
                    target_pos,
                    target_entity,
                    timer: Timer::from_seconds(0.32, TimerMode::Once),
                    damage: raw_dmg,
                    is_heal: false,
                    is_crit,
                    is_ultimate,
                    aoe_row: aoe,
                    arc_height: arc_h,
                    class,
                },
            ));
        }
    } else {
        turn_manager.active_attacker = None;
        turn_manager.cooldown_timer.reset();
    }
}
