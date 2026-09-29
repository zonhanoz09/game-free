use crate::assets_3d::Game3dAssets;
use crate::board::grid_to_world_pos;
use crate::types::*;
use crate::units::Unit;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::f32::consts::PI;

#[derive(Component)]
pub struct ActionGauge {
    pub current: f32,
}

#[derive(Component)]
pub struct FloatingText {
    pub world_pos: Vec3,
    pub timer: Timer,
    pub velocity: Vec3,
}

#[derive(Component)]
pub struct DashAnimation {
    pub origin: Vec3,
    pub target: Vec3,
    pub timer: Timer,
    pub returning: bool,
    pub damage_dealt: bool,
    pub target_entity: Entity,
    pub damage: f32,
    pub is_crit: bool,
}

#[derive(Component)]
pub struct Projectile3d {
    pub start: Vec3,
    pub target_pos: Vec3,
    pub target_entity: Entity,
    pub timer: Timer,
    pub damage: f32,
    pub is_heal: bool,
    pub is_crit: bool,
    pub aoe_row: Option<(usize, Faction)>,
    pub arc_height: f32,
}

#[derive(Resource)]
pub struct BattleRng {
    pub state: u64,
}

impl Default for BattleRng {
    fn default() -> Self {
        Self { state: 123456789 }
    }
}

impl BattleRng {
    pub fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 32) as u32 as f32) / (u32::MAX as f32)
    }
}

pub fn on_enter_battle(mut commands: Commands, units: Query<Entity, With<Unit>>) {
    for entity in units.iter() {
        commands.entity(entity).insert(ActionGauge { current: 0.0 });
    }
}

pub fn on_exit_battle(
    mut commands: Commands,
    projectiles: Query<Entity, With<Projectile3d>>,
    floating_texts: Query<Entity, With<FloatingText>>,
    dashes: Query<(Entity, &DashAnimation)>,
) {
    for entity in projectiles.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in floating_texts.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for (entity, dash) in dashes.iter() {
        commands.entity(entity).remove::<DashAnimation>();
        commands
            .entity(entity)
            .insert(Transform::from_translation(dash.origin));
    }
}

pub fn spawn_floating_text(
    commands: &mut Commands,
    world_pos: Vec3,
    text: &str,
    color: Color,
    font_size: f32,
) {
    let spawn_pos = world_pos + Vec3::new(0.0, 2.5, 0.0);
    commands.spawn((
        Text2d::new(text.to_string()),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(0.0, 0.0, 100.0), // Will be mapped to 2D screen coordinates
        FloatingText {
            world_pos: spawn_pos,
            timer: Timer::from_seconds(1.1, TimerMode::Once),
            velocity: Vec3::new(0.0, 1.4, 0.0),
        },
    ));
}

pub fn update_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut FloatingText)>,
) {
    let dt = time.delta_secs() * speed.multiplier;
    let Ok((camera, camera_transform)) = cameras.get_single() else {
        return;
    };
    let Ok(window) = windows.get_single() else {
        return;
    };
    let win_w = window.width();
    let win_h = window.height();

    for (entity, mut transform, mut color, mut ft) in query.iter_mut() {
        ft.timer.tick(std::time::Duration::from_secs_f32(dt));
        let vel = ft.velocity;
        ft.world_pos += vel * dt;

        let alpha = 1.0 - ft.timer.fraction();
        color.0 = color.0.with_alpha(alpha);

        if let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, ft.world_pos) {
            // Convert viewport pixel pos to 2D Camera coordinate (center is 0,0)
            transform.translation.x = viewport_pos.x - win_w * 0.5;
            transform.translation.y = win_h * 0.5 - viewport_pos.y;
        }

        if ft.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn battle_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    assets_3d: Res<Game3dAssets>,
    mut rng: ResMut<BattleRng>,
    mut units: Query<
        (
            Entity,
            &Unit,
            &UnitStats,
            &GridPos,
            &Transform,
            &mut ActionGauge,
        ),
        (Without<DeadUnit>, Without<DashAnimation>),
    >,
    all_targets: Query<(Entity, &Unit, &UnitStats, &GridPos, &Transform), Without<DeadUnit>>,
) {
    let dt = time.delta_secs() * speed.multiplier;

    let mut actions = Vec::new();

    for (entity, unit, stats, grid, transform, mut gauge) in units.iter_mut() {
        gauge.current += stats.speed * 6.5 * dt;

        if gauge.current >= 100.0 {
            gauge.current -= 100.0;
            actions.push((
                entity,
                unit.class,
                unit.faction,
                *stats,
                *grid,
                transform.translation,
            ));
        }
    }

    for (actor_entity, class, faction, stats, grid, actor_pos) in actions {
        let opponent_faction = match faction {
            Faction::Player => Faction::Enemy,
            Faction::Enemy => Faction::Player,
        };

        if class == UnitClass::Cleric {
            let mut lowest_ally: Option<(Entity, f32, Vec3)> = None;

            for (t_entity, t_unit, t_stats, _, t_transform) in all_targets.iter() {
                if t_unit.faction == faction && t_stats.hp > 0.0 {
                    let hp_ratio = t_stats.hp / t_stats.max_hp;
                    if lowest_ally.is_none() || hp_ratio < lowest_ally.unwrap().1 {
                        lowest_ally = Some((t_entity, hp_ratio, t_transform.translation));
                    }
                }
            }

            if let Some((target_entity, _, target_pos)) = lowest_ally {
                let heal_amount = 26.0 + stats.atk * 0.45;
                commands.spawn((
                    Mesh3d(assets_3d.divine_star.clone()),
                    MeshMaterial3d(assets_3d.heal_glow.clone()),
                    Transform::from_translation(actor_pos + Vec3::new(0.0, 1.6, 0.0)),
                    Projectile3d {
                        start: actor_pos + Vec3::new(0.0, 1.6, 0.0),
                        target_pos: target_pos + Vec3::new(0.0, 1.4, 0.0),
                        target_entity,
                        timer: Timer::from_seconds(0.40, TimerMode::Once),
                        damage: heal_amount,
                        is_heal: true,
                        is_crit: false,
                        aoe_row: None,
                        arc_height: 1.8,
                    },
                ));
            }
            continue;
        }

        let mut target_candidate: Option<(Entity, Vec3, GridPos)> = None;

        match class {
            UnitClass::Assassin => {
                let mut best_backline_dist = -1i32;
                let mut lowest_hp = f32::MAX;

                for (t_entity, t_unit, t_stats, t_grid, t_transform) in all_targets.iter() {
                    if t_unit.faction == opponent_faction && t_stats.hp > 0.0 {
                        let depth = match opponent_faction {
                            Faction::Enemy => t_grid.col as i32,
                            Faction::Player => 2 - t_grid.col as i32,
                        };

                        if depth > best_backline_dist
                            || (depth == best_backline_dist && t_stats.hp < lowest_hp)
                        {
                            best_backline_dist = depth;
                            lowest_hp = t_stats.hp;
                            target_candidate = Some((t_entity, t_transform.translation, *t_grid));
                        }
                    }
                }
            }
            UnitClass::Archer => {
                let mut lowest_hp = f32::MAX;

                for (t_entity, t_unit, t_stats, t_grid, t_transform) in all_targets.iter() {
                    if t_unit.faction == opponent_faction && t_stats.hp > 0.0 {
                        if t_stats.hp < lowest_hp {
                            lowest_hp = t_stats.hp;
                            target_candidate = Some((t_entity, t_transform.translation, *t_grid));
                        }
                    }
                }
            }
            _ => {
                let mut best_score = i32::MAX;

                for (t_entity, t_unit, t_stats, t_grid, t_transform) in all_targets.iter() {
                    if t_unit.faction == opponent_faction && t_stats.hp > 0.0 {
                        let row_diff = (t_grid.row as i32 - grid.row as i32).abs();
                        let col_depth = match opponent_faction {
                            Faction::Enemy => t_grid.col as i32,
                            Faction::Player => 2 - t_grid.col as i32,
                        };
                        let score = row_diff * 10 + col_depth;

                        if score < best_score {
                            best_score = score;
                            target_candidate = Some((t_entity, t_transform.translation, *t_grid));
                        }
                    }
                }
            }
        }

        if let Some((target_entity, target_pos, target_grid)) = target_candidate {
            let is_crit = rng.next_f32() < stats.crit_rate;
            let raw_dmg = stats.atk * if is_crit { 1.6 } else { 1.0 };

            if class == UnitClass::Knight || class == UnitClass::Assassin {
                let duration = if class == UnitClass::Assassin {
                    0.25
                } else {
                    0.35
                };
                commands.entity(actor_entity).insert(DashAnimation {
                    origin: actor_pos,
                    target: target_pos,
                    timer: Timer::from_seconds(duration, TimerMode::Once),
                    returning: false,
                    damage_dealt: false,
                    target_entity,
                    damage: raw_dmg,
                    is_crit,
                });
            } else {
                let aoe = if class == UnitClass::Mage {
                    Some((target_grid.row, opponent_faction))
                } else {
                    None
                };

                let (mesh, mat, arc_h) = if class == UnitClass::Mage {
                    (
                        assets_3d.magic_orb.clone(),
                        assets_3d.mage_crystal.clone(),
                        0.6,
                    )
                } else {
                    (
                        assets_3d.arrow_head.clone(),
                        assets_3d.arrow_glow.clone(),
                        1.4,
                    )
                };

                let start_pt = actor_pos + Vec3::new(0.0, 1.2, 0.0);
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_translation(start_pt),
                    Projectile3d {
                        start: start_pt,
                        target_pos: target_pos + Vec3::new(0.0, 1.2, 0.0),
                        target_entity,
                        timer: Timer::from_seconds(0.35, TimerMode::Once),
                        damage: raw_dmg,
                        is_heal: false,
                        is_crit,
                        aoe_row: aoe,
                        arc_height: arc_h,
                    },
                ));
            }
        }
    }
}

pub fn update_dash_animations(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(Entity, &mut Transform, &mut DashAnimation)>,
    mut target_query: Query<&mut UnitStats>,
) {
    let dt = time.delta_secs() * speed.multiplier;

    for (entity, mut transform, mut dash) in query.iter_mut() {
        dash.timer.tick(std::time::Duration::from_secs_f32(dt));
        let progress = dash.timer.fraction();

        if !dash.returning {
            let mut pos = dash.origin.lerp(dash.target, progress);
            pos.y += (progress * PI).sin() * 0.35; // Slight combat leap arc
            transform.translation = pos;

            if dash.timer.finished() {
                if !dash.damage_dealt {
                    dash.damage_dealt = true;

                    if let Ok(mut target_stats) = target_query.get_mut(dash.target_entity) {
                        let actual_dmg =
                            (dash.damage * (100.0 / (100.0 + target_stats.def))).max(4.0);
                        target_stats.hp -= actual_dmg;

                        let text = if dash.is_crit {
                            format!("-{:.0} CHÍ MẠNG!", actual_dmg)
                        } else {
                            format!("-{:.0}", actual_dmg)
                        };
                        let col = if dash.is_crit {
                            Color::srgb(1.0, 0.85, 0.1)
                        } else {
                            Color::srgb(1.0, 0.25, 0.25)
                        };
                        spawn_floating_text(
                            &mut commands,
                            dash.target,
                            &text,
                            col,
                            if dash.is_crit { 18.0 } else { 14.0 },
                        );
                    }
                }

                dash.returning = true;
                dash.timer.reset();
            }
        } else {
            let mut pos = dash.target.lerp(dash.origin, progress);
            pos.y += (progress * PI).sin() * 0.2;
            transform.translation = pos;

            if dash.timer.finished() {
                transform.translation = dash.origin;
                commands.entity(entity).remove::<DashAnimation>();
            }
        }
    }
}

pub fn update_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(Entity, &mut Transform, &mut Projectile3d)>,
    mut target_query: Query<(Entity, &mut UnitStats, &GridPos)>,
) {
    let dt = time.delta_secs() * speed.multiplier;

    for (proj_entity, mut transform, mut proj) in query.iter_mut() {
        proj.timer.tick(std::time::Duration::from_secs_f32(dt));
        let progress = proj.timer.fraction();

        let mut current_pos = proj.start.lerp(proj.target_pos, progress);
        current_pos.y += (progress * PI).sin() * proj.arc_height;
        transform.translation = current_pos;

        // Rotate projectile towards travel trajectory
        let dir = (proj.target_pos - proj.start).normalize_or_zero();
        if dir.length_squared() > 0.001 {
            transform.look_to(dir, Vec3::Y);
        }

        if proj.timer.finished() {
            if proj.is_heal {
                if let Ok((_, mut stats, _)) = target_query.get_mut(proj.target_entity) {
                    stats.hp = (stats.hp + proj.damage).min(stats.max_hp);
                    spawn_floating_text(
                        &mut commands,
                        proj.target_pos,
                        &format!("+{:.0}", proj.damage),
                        Color::srgb(0.2, 0.95, 0.35),
                        15.0,
                    );
                }
            } else {
                if let Ok((_, mut stats, _)) = target_query.get_mut(proj.target_entity) {
                    let actual_dmg = (proj.damage * (100.0 / (100.0 + stats.def))).max(4.0);
                    stats.hp -= actual_dmg;

                    let text = if proj.is_crit {
                        format!("-{:.0} CHÍ MẠNG!", actual_dmg)
                    } else {
                        format!("-{:.0}", actual_dmg)
                    };
                    let col = if proj.is_crit {
                        Color::srgb(1.0, 0.85, 0.1)
                    } else {
                        Color::srgb(1.0, 0.3, 0.3)
                    };
                    spawn_floating_text(
                        &mut commands,
                        proj.target_pos,
                        &text,
                        col,
                        if proj.is_crit { 18.0 } else { 14.0 },
                    );
                }

                if let Some((row, target_faction)) = proj.aoe_row {
                    let splash_dmg = proj.damage * 0.45;
                    for (other_ent, mut other_stats, other_grid) in target_query.iter_mut() {
                        if other_ent != proj.target_entity
                            && other_grid.row == row
                            && other_grid.faction == target_faction
                            && other_stats.hp > 0.0
                        {
                            let actual_splash =
                                (splash_dmg * (100.0 / (100.0 + other_stats.def))).max(3.0);
                            other_stats.hp -= actual_splash;
                            let pos = grid_to_world_pos(
                                other_grid.col,
                                other_grid.row,
                                other_grid.faction,
                            );
                            spawn_floating_text(
                                &mut commands,
                                pos,
                                &format!("-{:.0} LAN", actual_splash),
                                Color::srgb(0.85, 0.4, 1.0),
                                12.0,
                            );
                        }
                    }
                }
            }

            commands.entity(proj_entity).despawn_recursive();
        }
    }
}

pub fn check_unit_deaths(
    mut commands: Commands,
    mut units: Query<
        (Entity, &UnitStats, &mut Visibility, &mut Transform),
        (With<Unit>, Without<DeadUnit>),
    >,
) {
    for (entity, stats, mut vis, mut transform) in units.iter_mut() {
        if stats.hp <= 0.0 {
            // Dissolve / sink defeated unit beneath the arena stone floor
            transform.translation.y = -2.0;
            *vis = Visibility::Hidden;
            commands.entity(entity).insert(DeadUnit);
        }
    }
}

pub fn check_battle_end(
    units: Query<(&Unit, &UnitStats), Without<DeadUnit>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if *current_state.get() != GameState::Battle {
        return;
    }

    let mut alive_player = 0;
    let mut alive_enemy = 0;

    for (unit, stats) in units.iter() {
        if stats.hp > 0.0 {
            match unit.faction {
                Faction::Player => alive_player += 1,
                Faction::Enemy => alive_enemy += 1,
            }
        }
    }

    if alive_enemy == 0 && alive_player > 0 {
        next_state.set(GameState::Victory);
    } else if alive_player == 0 {
        next_state.set(GameState::Defeat);
    }
}
