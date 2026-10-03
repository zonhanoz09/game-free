use super::*;
use crate::units::ActiveStatusEffects;

pub fn update_dash_animations(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut rng: ResMut<BattleRng>,
    mut hit_stop: ResMut<HitStopManager>,
    mut camera_shake: ResMut<CameraShake2d>,
    mut turn_manager: ResMut<BattleTurnManager>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut DashAnimation2d,
        Option<&mut ChibiSquashStretch>,
    )>,
    mut target_query: Query<
        (
            Entity,
            &mut UnitStats,
            &Transform,
            Option<&mut ChibiSquashStretch>,
            Option<&mut ActionGauge>,
            Option<&UnitHitRecoil2d>,
            Option<&mut ActiveStatusEffects>,
        ),
        Without<DashAnimation2d>,
    >,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if hit_stop.active {
        return;
    }

    let dt = time.delta_secs() * speed.multiplier;

    for (entity, mut transform, mut dash, mut maybe_attacker_squash) in query.iter_mut() {
        dash.timer.tick(std::time::Duration::from_secs_f32(dt));
        let progress = dash.timer.fraction();

        if !dash.returning {
            let mut pos = dash.origin.lerp(dash.target, progress);
            pos.y += (progress * PI).sin() * 24.0;
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;

            if progress < 0.25 {
                if let Some(ref mut s) = maybe_attacker_squash {
                    s.target_scale = Vec3::new(1.20, 0.78, 1.0);
                }
            } else if progress < 0.85 {
                if let Some(ref mut s) = maybe_attacker_squash {
                    s.target_scale = Vec3::new(0.82, 1.34, 1.0);
                }
            } else {
                if let Some(ref mut s) = maybe_attacker_squash {
                    s.target_scale = Vec3::new(1.30, 0.70, 1.0);
                }
            }

            if dash.timer.finished() {
                if !dash.damage_dealt {
                    dash.damage_dealt = true;

                    hit_stop.trigger(if dash.is_ultimate {
                        0.11
                    } else if dash.is_crit {
                        0.088
                    } else {
                        0.065
                    });
                    camera_shake.add_trauma(if dash.is_ultimate {
                        0.65
                    } else if dash.is_crit {
                        0.48
                    } else {
                        0.30
                    });

                    if let Ok((
                        target_ent,
                        mut target_stats,
                        target_transform,
                        mut maybe_target_squash,
                        mut maybe_gauge,
                        maybe_recoil,
                        mut maybe_effects,
                    )) = target_query.get_mut(dash.target_entity)
                    {
                        let raw_dmg = dash.damage;
                        let mut actual_dmg = raw_dmg;

                        if target_stats.shield > 0.0 {
                            let absorbed = actual_dmg.min(target_stats.shield);
                            target_stats.shield -= absorbed;
                            actual_dmg -= absorbed;
                            sound_events.send(PlaySoundEvent(SoundEffect::Shield));
                            spawn_floating_text(
                                &mut commands,
                                &mut rng,
                                target_transform.translation.xy() + Vec2::new(0.0, 18.0),
                                &format!("SHIELD -{:.0}", absorbed),
                                Color::srgb(0.35, 0.85, 1.0),
                                13.0,
                            );
                        } else {
                            sound_events.send(PlaySoundEvent(SoundEffect::Hit));
                        }

                        target_stats.hp -= actual_dmg;
                        // Mana/rage is updated from authoritative simulation events
                        info!(
                            "[COMBAT MELEE] {:?} struck target for {:.1} dmg (Crit: {}, Ult: {}) -> Target HP: {:.1}/{:.1}",
                            dash.class,
                            actual_dmg,
                            dash.is_crit,
                            dash.is_ultimate,
                            target_stats.hp,
                            target_stats.max_hp
                        );

                        if let Some(ref mut effects) = maybe_effects {
                            if dash.is_ultimate {
                                match dash.class {
                                    UnitClass::Knight | UnitClass::SunCe => {
                                        effects.add("stun", "💫", "Choáng", 1);
                                    }
                                    UnitClass::Assassin => {
                                        effects.add("bleed", "🩸", "Chảy Máu", 2);
                                    }
                                    UnitClass::CaoCao | UnitClass::DianWei => {
                                        effects.add("burn", "🔥", "Hỏa Sát", 2);
                                    }
                                    _ => {
                                        effects.add("weaken", "💥", "Suy Yếu", 2);
                                    }
                                }
                            } else if dash.is_crit {
                                effects.add("vulnerable", "🎯", "Sơ Hở", 1);
                            }
                        }

                        if dash.is_ultimate && (dash.class == UnitClass::Knight || dash.class == UnitClass::SunCe) {
                            if let Some(ref mut g) = maybe_gauge {
                                g.current = (g.current - 35.0).max(0.0);
                            }
                            spawn_floating_text(
                                &mut commands,
                                &mut rng,
                                target_transform.translation.xy() + Vec2::new(0.0, 32.0),
                                "DISRUPTED! -35 ATB",
                                Color::srgb(1.0, 0.85, 0.2),
                                13.0,
                            );
                        }

                        if dash.is_ultimate
                            && dash.class == UnitClass::Assassin
                            && target_stats.hp <= 0.0
                        {
                            spawn_floating_text(
                                &mut commands,
                                &mut rng,
                                dash.origin + Vec2::new(0.0, 30.0),
                                "+50 MANA EXECUTE!",
                                Color::srgb(1.0, 0.88, 0.2),
                                15.0,
                            );
                        }

                        if let Some(ref mut s) = maybe_target_squash {
                            s.target_scale = Vec3::new(1.35, 0.68, 1.0);
                        }

                        let text = if dash.is_ultimate {
                            format!("-[{:.0}] ULTIMATE!", actual_dmg)
                        } else if dash.is_crit {
                            format!("-{:.0} CRIT!", actual_dmg)
                        } else {
                            format!("-{:.0}", actual_dmg)
                        };
                        let col = if dash.is_ultimate {
                            Color::srgb(1.0, 0.88, 0.2)
                        } else if dash.is_crit {
                            Color::srgb(1.0, 0.85, 0.1)
                        } else {
                            Color::srgb(1.0, 0.25, 0.25)
                        };
                        spawn_floating_text(
                            &mut commands,
                            &mut rng,
                            dash.target,
                            &text,
                            col,
                            if dash.is_ultimate {
                                20.0
                            } else if dash.is_crit {
                                19.0
                            } else {
                                15.0
                            },
                        );

                        let hit_pos = target_transform.translation.xy();

                        if dash.class != UnitClass::Assassin {
                            commands.spawn((
                                Sprite {
                                    custom_size: Some(Vec2::new(
                                        if dash.is_ultimate { 62.0 } else { 42.0 },
                                        if dash.is_ultimate { 12.0 } else { 8.0 },
                                    )),
                                    color: Color::srgb(1.0, 0.85, 0.3),
                                    ..default()
                                },
                                Transform::from_xyz(hit_pos.x, hit_pos.y, 40.0)
                                    .with_rotation(Quat::from_rotation_z(0.65)),
                                CombatVfx2d {
                                    timer: Timer::from_seconds(0.24, TimerMode::Once),
                                    initial_scale: Vec2::splat(0.6),
                                    target_scale: Vec2::splat(if dash.is_ultimate {
                                        2.4
                                    } else {
                                        1.8
                                    }),
                                    rotate_speed: 6.0,
                                },
                            ));

                            commands.spawn((
                                Sprite {
                                    custom_size: Some(Vec2::splat(if dash.is_ultimate {
                                        44.0
                                    } else {
                                        28.0
                                    })),
                                    color: Color::srgba(1.0, 0.9, 0.4, 0.8),
                                    ..default()
                                },
                                Transform::from_xyz(hit_pos.x, hit_pos.y, 38.0),
                                CombatVfx2d {
                                    timer: Timer::from_seconds(0.25, TimerMode::Once),
                                    initial_scale: Vec2::splat(0.5),
                                    target_scale: Vec2::splat(if dash.is_ultimate {
                                        3.0
                                    } else {
                                        2.2
                                    }),
                                    rotate_speed: 0.0,
                                },
                            ));

                            spawn_impact_sparks(
                                &mut commands,
                                &mut rng,
                                hit_pos,
                                Color::srgb(1.0, 0.85, 0.3),
                                if dash.is_ultimate { 14 } else { 8 },
                            );
                        } else {
                            commands.spawn((
                                Sprite {
                                    custom_size: Some(Vec2::new(
                                        if dash.is_ultimate { 54.0 } else { 38.0 },
                                        if dash.is_ultimate { 9.0 } else { 6.0 },
                                    )),
                                    color: Color::srgb(1.0, 0.18, 0.22),
                                    ..default()
                                },
                                Transform::from_xyz(hit_pos.x, hit_pos.y, 40.0)
                                    .with_rotation(Quat::from_rotation_z(0.78)),
                                CombatVfx2d {
                                    timer: Timer::from_seconds(0.25, TimerMode::Once),
                                    initial_scale: Vec2::splat(0.7),
                                    target_scale: Vec2::splat(if dash.is_ultimate {
                                        2.2
                                    } else {
                                        1.65
                                    }),
                                    rotate_speed: 0.0,
                                },
                            ));
                            commands.spawn((
                                Sprite {
                                    custom_size: Some(Vec2::new(
                                        if dash.is_ultimate { 54.0 } else { 38.0 },
                                        if dash.is_ultimate { 9.0 } else { 6.0 },
                                    )),
                                    color: Color::srgb(0.2, 0.9, 0.3),
                                    ..default()
                                },
                                Transform::from_xyz(hit_pos.x, hit_pos.y, 40.0)
                                    .with_rotation(Quat::from_rotation_z(-0.78)),
                                CombatVfx2d {
                                    timer: Timer::from_seconds(0.25, TimerMode::Once),
                                    initial_scale: Vec2::splat(0.7),
                                    target_scale: Vec2::splat(if dash.is_ultimate {
                                        2.2
                                    } else {
                                        1.65
                                    }),
                                    rotate_speed: 0.0,
                                },
                            ));

                            spawn_impact_sparks(
                                &mut commands,
                                &mut rng,
                                hit_pos,
                                Color::srgb(0.9, 0.2, 0.2),
                                if dash.is_ultimate { 16 } else { 9 },
                            );
                        }

                        let orig_pos = if let Some(existing) = maybe_recoil {
                            existing.original_pos
                        } else {
                            target_transform.translation.xy()
                        };
                        let recoil_dir = (dash.target - dash.origin).normalize_or_zero() * 16.0;
                        commands.entity(target_ent).insert(UnitHitRecoil2d {
                            original_pos: orig_pos,
                            recoil_offset: recoil_dir,
                            timer: Timer::from_seconds(0.18, TimerMode::Once),
                        });
                    }
                }

                dash.returning = true;
                dash.timer.reset();
            }
        } else {
            let mut pos = dash.target.lerp(dash.origin, progress);
            pos.y += (progress * PI).sin() * 14.0;
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;

            if dash.timer.finished() {
                transform.translation.x = dash.origin.x;
                transform.translation.y = dash.origin.y;
                if let Some(ref mut s) = maybe_attacker_squash {
                    s.target_scale = Vec3::ONE;
                }
                commands.entity(entity).remove::<DashAnimation2d>();

                turn_manager.active_attacker = None;
                turn_manager.cooldown_timer.reset();
            }
        }
    }
}

pub fn update_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut rng: ResMut<BattleRng>,
    mut hit_stop: ResMut<HitStopManager>,
    mut camera_shake: ResMut<CameraShake2d>,
    mut turn_manager: ResMut<BattleTurnManager>,
    mut query: Query<(Entity, &mut Transform, &mut Projectile2d)>,
    mut target_query: Query<
        (
            Entity,
            &mut UnitStats,
            &GridPos,
            &Transform,
            Option<&mut ChibiSquashStretch>,
            Option<&UnitHitRecoil2d>,
            Option<&mut ActiveStatusEffects>,
        ),
        Without<Projectile2d>,
    >,
) {
    if hit_stop.active {
        return;
    }

    let dt = time.delta_secs() * speed.multiplier;
    let total_projectiles = query.iter().count();
    let mut finished_count = 0;

    for (proj_entity, mut transform, mut proj) in query.iter_mut() {
        proj.timer.tick(std::time::Duration::from_secs_f32(dt));
        let progress = proj.timer.fraction();

        let mut current_pos = proj.start.lerp(proj.target_pos, progress);
        current_pos.y += (progress * PI).sin() * proj.arc_height;
        transform.translation.x = current_pos.x;
        transform.translation.y = current_pos.y;

        let dir = proj.target_pos - proj.start;
        if dir.length_squared() > 0.001 {
            transform.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));
        }

        if proj.timer.finished() {
            finished_count += 1;
            if proj.is_heal {
                if let Ok((_, mut stats, _, target_transform, mut maybe_target_squash, _, mut maybe_effects)) =
                    target_query.get_mut(proj.target_entity)
                {
                    stats.hp = (stats.hp + proj.damage).min(stats.max_hp);
                    info!(
                        "[COMBAT HEAL] Holy Grace healed target for +{:.1} HP -> New HP: {:.1}/{:.1}",
                        proj.damage, stats.hp, stats.max_hp
                    );
                    if let Some(ref mut effects) = maybe_effects {
                        if proj.is_ultimate {
                            effects.add("divine_shield", "🛡️", "Thánh Thuẫn", 2);
                            effects.add("divine_bless", "⚕️", "Thần Ân", 2);
                        } else {
                            effects.add("heal_regen", "🌿", "Hồi Phục", 2);
                        }
                    }
                    if let Some(ref mut s) = maybe_target_squash {
                        s.target_scale = Vec3::new(0.9, 1.25, 1.0);
                    }

                    let txt = if proj.is_ultimate {
                        format!("+[{:.0}] DIVINE HEAL!", proj.damage)
                    } else {
                        format!("+{:.0} HEAL", proj.damage)
                    };

                    spawn_floating_text(
                        &mut commands,
                        &mut rng,
                        proj.target_pos,
                        &txt,
                        if proj.is_ultimate {
                            Color::srgb(1.0, 0.95, 0.3)
                        } else {
                            Color::srgb(0.25, 0.95, 0.4)
                        },
                        if proj.is_ultimate { 18.0 } else { 16.0 },
                    );

                    let heal_pos = target_transform.translation.xy();

                    commands.spawn((
                        Sprite {
                            custom_size: Some(Vec2::splat(if proj.is_ultimate {
                                48.0
                            } else {
                                32.0
                            })),
                            color: Color::srgba(1.0, 0.92, 0.35, 0.8),
                            ..default()
                        },
                        Transform::from_xyz(heal_pos.x, heal_pos.y, 40.0),
                        CombatVfx2d {
                            timer: Timer::from_seconds(0.35, TimerMode::Once),
                            initial_scale: Vec2::splat(0.4),
                            target_scale: Vec2::splat(if proj.is_ultimate { 3.0 } else { 2.4 }),
                            rotate_speed: 2.0,
                        },
                    ));

                    spawn_impact_sparks(
                        &mut commands,
                        &mut rng,
                        heal_pos,
                        Color::srgb(1.0, 0.9, 0.35),
                        if proj.is_ultimate { 12 } else { 9 },
                    );
                }
            } else {
                hit_stop.trigger(if proj.is_ultimate {
                    0.095
                } else if proj.is_crit {
                    0.085
                } else {
                    0.060
                });

                if let Ok((
                    target_ent,
                    mut stats,
                    _,
                    target_transform,
                    mut maybe_target_squash,
                    maybe_recoil,
                    mut maybe_effects,
                )) = target_query.get_mut(proj.target_entity)
                {
                    let raw_dmg = proj.damage;
                    let mut actual_dmg = raw_dmg;

                    if stats.shield > 0.0 {
                        let absorbed = actual_dmg.min(stats.shield);
                        stats.shield -= absorbed;
                        actual_dmg -= absorbed;
                        spawn_floating_text(
                            &mut commands,
                            &mut rng,
                            proj.target_pos + Vec2::new(0.0, 18.0),
                            &format!("SHIELD -{:.0}", absorbed),
                            Color::srgb(0.35, 0.85, 1.0),
                            13.0,
                        );
                    }

                    stats.hp -= actual_dmg;
                    // Mana/rage is updated from authoritative simulation events
                    info!(
                        "[COMBAT RANGED] {:?} projectile hit for {:.1} dmg (Crit: {}, Ult: {}) -> Target HP: {:.1}/{:.1}",
                        proj.class,
                        actual_dmg,
                        proj.is_crit,
                        proj.is_ultimate,
                        stats.hp,
                        stats.max_hp
                    );

                    if let Some(ref mut effects) = maybe_effects {
                        if proj.is_ultimate {
                            match proj.class {
                                UnitClass::Mage => {
                                    effects.add("burn", "🔥", "Hỏa Kích", 2);
                                }
                                UnitClass::Archer => {
                                    effects.add("pierce", "🏹", "Phá Giáp", 2);
                                }
                                _ => {
                                    effects.add("elemental_shock", "⚡", "Sốc Điện", 2);
                                }
                            }
                        } else if proj.class == UnitClass::Mage {
                            effects.add("burn", "🔥", "Bỏng Nhẹ", 1);
                        }
                    }

                    if let Some(ref mut s) = maybe_target_squash {
                        s.target_scale = Vec3::new(1.30, 0.72, 1.0);
                    }

                    let text = if proj.is_ultimate {
                        format!("-[{:.0}] ULTIMATE!", actual_dmg)
                    } else if proj.is_crit {
                        format!("-{:.0} CRIT!", actual_dmg)
                    } else {
                        format!("-{:.0}", actual_dmg)
                    };
                    let col = if proj.is_ultimate {
                        Color::srgb(1.0, 0.88, 0.2)
                    } else if proj.is_crit {
                        Color::srgb(1.0, 0.85, 0.1)
                    } else {
                        Color::srgb(1.0, 0.3, 0.3)
                    };
                    spawn_floating_text(
                        &mut commands,
                        &mut rng,
                        proj.target_pos,
                        &text,
                        col,
                        if proj.is_ultimate {
                            19.0
                        } else if proj.is_crit {
                            18.0
                        } else {
                            15.0
                        },
                    );

                    let hit_pos = target_transform.translation.xy();

                    if proj.class == UnitClass::Archer {
                        camera_shake.add_trauma(if proj.is_ultimate {
                            0.45
                        } else if proj.is_crit {
                            0.40
                        } else {
                            0.20
                        });

                        spawn_impact_sparks(
                            &mut commands,
                            &mut rng,
                            hit_pos,
                            Color::srgb(0.3, 0.95, 0.4),
                            if proj.is_ultimate { 12 } else { 7 },
                        );
                    } else {
                        camera_shake.add_trauma(if proj.is_ultimate { 0.65 } else { 0.50 });
                        let p_col = proj.class.color();
                        let p_srgba = p_col.to_srgba();

                        commands.spawn((
                            Sprite {
                                custom_size: Some(Vec2::splat(if proj.is_ultimate {
                                    52.0
                                } else {
                                    36.0
                                })),
                                color: Color::srgba(p_srgba.red, p_srgba.green, p_srgba.blue, 0.8),
                                ..default()
                            },
                            Transform::from_xyz(hit_pos.x, hit_pos.y, 40.0),
                            CombatVfx2d {
                                timer: Timer::from_seconds(0.34, TimerMode::Once),
                                initial_scale: Vec2::splat(0.4),
                                target_scale: Vec2::splat(if proj.is_ultimate { 3.5 } else { 2.8 }),
                                rotate_speed: 4.0,
                            },
                        ));

                        spawn_impact_sparks(
                            &mut commands,
                            &mut rng,
                            hit_pos,
                            p_col,
                            if proj.is_ultimate { 15 } else { 10 },
                        );
                    }

                    let orig_pos = if let Some(existing) = maybe_recoil {
                        existing.original_pos
                    } else {
                        target_transform.translation.xy()
                    };
                    let recoil_dir = (proj.target_pos - proj.start).normalize_or_zero() * 14.0;
                    commands.entity(target_ent).insert(UnitHitRecoil2d {
                        original_pos: orig_pos,
                        recoil_offset: recoil_dir,
                        timer: Timer::from_seconds(0.18, TimerMode::Once),
                    });
                }

                if let Some((row, target_faction)) = proj.aoe_row {
                    let splash_dmg = proj.damage * 0.48;
                    for (
                        other_ent,
                        mut other_stats,
                        other_grid,
                        other_transform,
                        mut maybe_other_squash,
                        _,
                        mut maybe_other_effects,
                    ) in target_query.iter_mut()
                    {
                        if other_ent != proj.target_entity
                            && other_grid.row == row
                            && other_grid.faction == target_faction
                            && other_stats.hp > 0.0
                        {
                            let actual_splash_raw = splash_dmg;
                            let mut actual_splash = actual_splash_raw;
                            if other_stats.shield > 0.0 {
                                let absorbed = actual_splash.min(other_stats.shield);
                                other_stats.shield -= absorbed;
                                actual_splash -= absorbed;
                            }
                            other_stats.hp -= actual_splash;
                            // Mana/rage is updated from authoritative simulation events
                            if let Some(ref mut s) = maybe_other_squash {
                                s.target_scale = Vec3::new(1.25, 0.75, 1.0);
                            }

                            if let Some(ref mut effects) = maybe_other_effects {
                                effects.add("burn", "🔥", "Lan Hỏa", 1);
                            }

                            let pos = grid_to_world_pos(
                                other_grid.col,
                                other_grid.row,
                                other_grid.faction,
                            );
                            spawn_floating_text(
                                &mut commands,
                                &mut rng,
                                pos,
                                &format!("-{:.0} SPLASH", actual_splash),
                                Color::srgb(0.88, 0.45, 1.0),
                                13.0,
                            );
                            spawn_impact_sparks(
                                &mut commands,
                                &mut rng,
                                other_transform.translation.xy(),
                                Color::srgb(0.8, 0.3, 1.0),
                                6,
                            );
                        }
                    }
                }
            }

            commands.entity(proj_entity).despawn_recursive();
        }
    }

    if finished_count > 0 && finished_count >= total_projectiles {
        turn_manager.active_attacker = None;
        turn_manager.cooldown_timer.reset();
    }
}
