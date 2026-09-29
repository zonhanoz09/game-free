use crate::assets_3d::Game3dAssets;
use crate::board::grid_to_world_pos;
use crate::types::*;
use crate::units::{ChibiSquashStretch, Unit, UnitVisualRoot};
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
    pub class: UnitClass,
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
    pub class: UnitClass,
}

#[derive(Component)]
pub struct CombatVfx {
    pub timer: Timer,
    pub initial_scale: Vec3,
    pub target_scale: Vec3,
    pub rotate_speed: f32,
}

#[derive(Component)]
pub struct SparkParticle {
    pub velocity: Vec3,
    pub timer: Timer,
}

#[derive(Component)]
pub struct UnitHitRecoil {
    pub original_pos: Vec3,
    pub recoil_offset: Vec3,
    pub timer: Timer,
}

#[derive(Component)]
pub struct ActiveTurnSpotlight {
    pub attacker_entity: Entity,
}

#[derive(Component)]
pub struct PointLightFlash {
    pub timer: Timer,
    pub initial_intensity: f32,
}

#[derive(Resource, Default)]
pub struct HitStopManager {
    pub timer: Timer,
    pub active: bool,
}

impl HitStopManager {
    pub fn trigger(&mut self, duration: f32) {
        self.timer = Timer::from_seconds(duration, TimerMode::Once);
        self.active = true;
    }

    pub fn update(&mut self, dt: f32) {
        if self.active {
            self.timer.tick(std::time::Duration::from_secs_f32(dt));
            if self.timer.finished() {
                self.active = false;
            }
        }
    }
}

#[derive(Resource)]
pub struct CameraShake {
    pub trauma: f32,
    pub base_translation: Vec3,
}

impl Default for CameraShake {
    fn default() -> Self {
        Self {
            trauma: 0.0,
            base_translation: Vec3::new(0.0, 14.2, 14.6),
        }
    }
}

impl CameraShake {
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }
}

#[derive(Resource)]
pub struct BattleTurnManager {
    pub active_attacker: Option<Entity>,
    pub cooldown_timer: Timer,
}

impl Default for BattleTurnManager {
    fn default() -> Self {
        Self {
            active_attacker: None,
            cooldown_timer: Timer::from_seconds(0.10, TimerMode::Once),
        }
    }
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

    pub fn random_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

pub fn update_hit_stop_system(time: Res<Time>, mut hit_stop: ResMut<HitStopManager>) {
    hit_stop.update(time.delta_secs());
}

pub fn update_camera_shake(
    time: Res<Time>,
    mut shake: ResMut<CameraShake>,
    mut rng: ResMut<BattleRng>,
    mut cam_query: Query<&mut Transform, With<crate::MainCamera3d>>,
) {
    let dt = time.delta_secs();
    shake.trauma = (shake.trauma - dt * 2.8).max(0.0);
    let intensity = shake.trauma * shake.trauma;

    let Ok(mut transform) = cam_query.get_single_mut() else {
        return;
    };

    if intensity > 0.001 {
        let ox = rng.random_range(-1.0, 1.0) * intensity * 0.42;
        let oy = rng.random_range(-0.7, 0.7) * intensity * 0.32;
        let oz = rng.random_range(-0.5, 0.5) * intensity * 0.22;
        transform.translation = shake.base_translation + Vec3::new(ox, oy, oz);
    } else {
        transform.translation = shake.base_translation;
    }
}

pub fn update_flash_lights(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(Entity, &mut PointLight, &mut PointLightFlash)>,
) {
    let dt = time.delta_secs() * speed.multiplier;
    for (entity, mut light, mut flash) in query.iter_mut() {
        flash.timer.tick(std::time::Duration::from_secs_f32(dt));
        let remaining = 1.0 - flash.timer.fraction();
        light.intensity = flash.initial_intensity * (remaining * remaining);

        if flash.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn on_enter_battle(
    mut commands: Commands,
    units: Query<Entity, With<Unit>>,
    mut turn_manager: ResMut<BattleTurnManager>,
) {
    turn_manager.active_attacker = None;
    turn_manager.cooldown_timer.reset();

    for entity in units.iter() {
        commands.entity(entity).insert(ActionGauge { current: 0.0 });
    }
}

pub fn on_exit_battle(
    mut commands: Commands,
    projectiles: Query<Entity, With<Projectile3d>>,
    floating_texts: Query<Entity, With<FloatingText>>,
    vfx_query: Query<Entity, With<CombatVfx>>,
    sparks_query: Query<Entity, With<SparkParticle>>,
    spotlight_query: Query<Entity, With<ActiveTurnSpotlight>>,
    flash_query: Query<Entity, With<PointLightFlash>>,
    dashes: Query<(Entity, &DashAnimation)>,
    mut turn_manager: ResMut<BattleTurnManager>,
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
    for entity in flash_query.iter() {
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
    rng: &mut BattleRng,
    world_pos: Vec3,
    text: &str,
    color: Color,
    font_size: f32,
) {
    // Scatter position slightly so consecutive damage numbers don't pile up in a straight line
    let offset_x = rng.random_range(-0.45, 0.45);
    let offset_z = rng.random_range(-0.25, 0.25);
    let spawn_pos = world_pos + Vec3::new(offset_x, 2.3, offset_z);
    let upward_speed = rng.random_range(2.6, 3.4);
    let drift_x = rng.random_range(-0.5, 0.5);

    commands.spawn((
        Text2d::new(text.to_string()),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(0.0, 0.0, 100.0).with_scale(Vec3::splat(1.15)),
        FloatingText {
            world_pos: spawn_pos,
            timer: Timer::from_seconds(0.68, TimerMode::Once),
            velocity: Vec3::new(drift_x, upward_speed, 0.0),
        },
    ));
}

pub fn spawn_impact_sparks(
    commands: &mut Commands,
    assets_3d: &Game3dAssets,
    rng: &mut BattleRng,
    pos: Vec3,
    mat: Handle<StandardMaterial>,
    count: usize,
) {
    for _ in 0..count {
        let vx = rng.random_range(-3.2, 3.2);
        let vy = rng.random_range(2.5, 5.5);
        let vz = rng.random_range(-3.2, 3.2);

        commands.spawn((
            Mesh3d(assets_3d.spark_sphere.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(pos).with_scale(Vec3::splat(rng.random_range(0.7, 1.4))),
            SparkParticle {
                velocity: Vec3::new(vx, vy, vz),
                timer: Timer::from_seconds(rng.random_range(0.24, 0.42), TimerMode::Once),
            },
        ));
    }
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
        let progress = ft.timer.fraction();

        // Gravity naturally curves the floating arc
        ft.velocity.y -= 4.0 * dt;
        let vel = ft.velocity;
        ft.world_pos += vel * dt;

        // Pop in scale at start, settle, then shrink out
        let scale = if progress < 0.20 {
            1.0 + (progress / 0.20) * 0.25
        } else if progress < 0.65 {
            1.25 - ((progress - 0.20) / 0.45) * 0.25
        } else {
            1.0 - ((progress - 0.65) / 0.35) * 0.35
        };
        transform.scale = Vec3::splat(scale);

        // Alpha fade out in last 40% of duration
        let alpha = if progress < 0.60 {
            1.0
        } else {
            1.0 - (progress - 0.60) / 0.40
        };
        color.0 = color.0.with_alpha(alpha);

        if let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, ft.world_pos) {
            transform.translation.x = viewport_pos.x - win_w * 0.5;
            transform.translation.y = win_h * 0.5 - viewport_pos.y;
        }

        if ft.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_combat_vfx(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    hit_stop: Res<HitStopManager>,
    mut vfx_query: Query<(Entity, &mut Transform, &mut CombatVfx)>,
    mut sparks_query: Query<
        (Entity, &mut Transform, &mut SparkParticle),
        (Without<CombatVfx>, Without<UnitHitRecoil>),
    >,
    mut recoil_query: Query<
        (Entity, &mut Transform, &mut UnitHitRecoil),
        (Without<CombatVfx>, Without<SparkParticle>),
    >,
) {
    if hit_stop.active {
        return;
    }

    let dt = time.delta_secs() * speed.multiplier;

    // 1. Expand / fade combat effects
    for (entity, mut transform, mut vfx) in vfx_query.iter_mut() {
        vfx.timer.tick(std::time::Duration::from_secs_f32(dt));
        let t = vfx.timer.fraction();

        transform.scale = vfx.initial_scale.lerp(vfx.target_scale, t);
        if vfx.rotate_speed != 0.0 {
            transform.rotate_y(vfx.rotate_speed * dt);
        }

        if vfx.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }

    // 2. Spark particles with physics gravity and friction
    for (entity, mut transform, mut spark) in sparks_query.iter_mut() {
        spark.timer.tick(std::time::Duration::from_secs_f32(dt));
        spark.velocity.y -= 15.0 * dt;
        transform.translation += spark.velocity * dt;
        transform.scale *= (1.0 - spark.timer.fraction()).max(0.01);

        if spark.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }

    // 3. Unit hit recoil spring back
    for (entity, mut transform, mut recoil) in recoil_query.iter_mut() {
        recoil.timer.tick(std::time::Duration::from_secs_f32(dt));
        let fraction = recoil.timer.fraction();

        if fraction < 0.28 {
            transform.translation = recoil.original_pos + recoil.recoil_offset * (fraction / 0.28);
        } else {
            let back = (fraction - 0.28) / 0.72;
            transform.translation =
                (recoil.original_pos + recoil.recoil_offset).lerp(recoil.original_pos, back);
        }

        if recoil.timer.finished() {
            transform.translation = recoil.original_pos;
            commands.entity(entity).remove::<UnitHitRecoil>();
        }
    }
}

pub fn update_turn_spotlight(
    time: Res<Time>,
    mut spotlight_query: Query<(&mut Transform, &ActiveTurnSpotlight)>,
    unit_query: Query<&Transform, (With<UnitVisualRoot>, Without<ActiveTurnSpotlight>)>,
) {
    let t = time.elapsed_secs();
    for (mut spot_transform, spot) in spotlight_query.iter_mut() {
        if let Ok(unit_transform) = unit_query.get(spot.attacker_entity) {
            spot_transform.translation = unit_transform.translation + Vec3::new(0.0, 0.06, 0.0);
            let pulse = 1.0 + (t * 8.0).sin() * 0.12;
            spot_transform.scale = Vec3::new(pulse, 1.0, pulse);
            spot_transform.rotate_y(2.5 * 0.016);
        }
    }
}

pub fn battle_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    assets_3d: Res<Game3dAssets>,
    mut rng: ResMut<BattleRng>,
    hit_stop: Res<HitStopManager>,
    mut turn_manager: ResMut<BattleTurnManager>,
    mut units: Query<
        (
            Entity,
            &Unit,
            &UnitStats,
            &GridPos,
            &Transform,
            &mut ActionGauge,
            &mut ChibiSquashStretch,
        ),
        Without<DeadUnit>,
    >,
    all_targets: Query<(Entity, &Unit, &UnitStats, &GridPos, &Transform), Without<DeadUnit>>,
    spotlight_query: Query<Entity, With<ActiveTurnSpotlight>>,
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

    // Clean up spotlight from previous turn
    for spot in spotlight_query.iter() {
        commands.entity(spot).despawn_recursive();
    }

    // Fill action gauges for all alive champions based on speed
    for (_, _, stats, _, _, mut gauge, _) in units.iter_mut() {
        gauge.current += stats.speed * 8.5 * dt;
    }

    // Pick champion with highest ready gauge (>= 100.0)
    let mut best_candidate: Option<(Entity, UnitClass, Faction, UnitStats, GridPos, Vec3)> = None;
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
                transform.translation,
            ));
        }
    }

    let Some((actor_entity, class, faction, stats, grid, actor_pos)) = best_candidate else {
        return;
    };

    // Deduct 100 gauge points and trigger shooter squash & stretch
    if let Ok((_, _, _, _, _, mut gauge, mut squash)) = units.get_mut(actor_entity) {
        gauge.current -= 100.0;
        // Chibi windup squash
        squash.target_scale = Vec3::new(0.85, 1.28, 0.85);
    }

    turn_manager.active_attacker = Some(actor_entity);

    // Spawn golden turn spotlight ring at actor's feet
    commands.spawn((
        Mesh3d(assets_3d.spotlight_ring.clone()),
        MeshMaterial3d(assets_3d.spotlight_mat.clone()),
        Transform::from_translation(actor_pos + Vec3::new(0.0, 0.06, 0.0)),
        ActiveTurnSpotlight {
            attacker_entity: actor_entity,
        },
    ));

    let opponent_faction = match faction {
        Faction::Player => Faction::Enemy,
        Faction::Enemy => Faction::Player,
    };

    // --- CASE 1: Cleric Healing Ally ---
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
            let heal_amount = 32.0 + stats.atk * 0.50;
            commands.spawn((
                Mesh3d(assets_3d.divine_star.clone()),
                MeshMaterial3d(assets_3d.heal_glow.clone()),
                Transform::from_translation(actor_pos + Vec3::new(0.0, 1.6, 0.0)),
                Projectile3d {
                    start: actor_pos + Vec3::new(0.0, 1.6, 0.0),
                    target_pos: target_pos + Vec3::new(0.0, 1.4, 0.0),
                    target_entity,
                    timer: Timer::from_seconds(0.38, TimerMode::Once),
                    damage: heal_amount,
                    is_heal: true,
                    is_crit: false,
                    aoe_row: None,
                    arc_height: 1.8,
                    class,
                },
            ));
        } else {
            turn_manager.active_attacker = None;
            turn_manager.cooldown_timer.reset();
        }
        return;
    }

    // --- CASE 2: Attack Target Selection ---
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
        UnitClass::Knight | UnitClass::Mage => {
            let mut min_col = usize::MAX;
            let mut best_row_diff = usize::MAX;

            for (t_entity, t_unit, t_stats, t_grid, t_transform) in all_targets.iter() {
                if t_unit.faction == opponent_faction && t_stats.hp > 0.0 {
                    let frontline_col = match opponent_faction {
                        Faction::Player => 2 - t_grid.col,
                        Faction::Enemy => t_grid.col,
                    };

                    let row_diff = (grid.row as i32 - t_grid.row as i32).unsigned_abs() as usize;

                    if frontline_col < min_col
                        || (frontline_col == min_col && row_diff < best_row_diff)
                    {
                        min_col = frontline_col;
                        best_row_diff = row_diff;
                        target_candidate = Some((t_entity, t_transform.translation, *t_grid));
                    }
                }
            }
        }
        UnitClass::Cleric => {
            // Already handled above
        }
    }

    if let Some((target_entity, target_pos, target_grid)) = target_candidate {
        let is_crit = rng.next_f32() < stats.crit_rate;
        let crit_mult = if is_crit { 1.5 } else { 1.0 };
        let raw_dmg = stats.atk * crit_mult;

        let is_melee = class == UnitClass::Knight || class == UnitClass::Assassin;

        if is_melee {
            let offset_dir = (actor_pos - target_pos).normalize_or_zero();
            let dash_target = target_pos + offset_dir * 0.95;

            // Spawn Anime Dust Puff at takeoff
            commands.spawn((
                Mesh3d(assets_3d.dust_puff_mesh.clone()),
                MeshMaterial3d(assets_3d.dust_mat.clone()),
                Transform::from_translation(actor_pos + Vec3::new(0.0, 0.05, 0.0))
                    .with_scale(Vec3::splat(0.4)),
                CombatVfx {
                    timer: Timer::from_seconds(0.22, TimerMode::Once),
                    initial_scale: Vec3::splat(0.4),
                    target_scale: Vec3::splat(1.3),
                    rotate_speed: 1.0,
                },
            ));

            commands.entity(actor_entity).insert(DashAnimation {
                origin: actor_pos,
                target: dash_target,
                timer: Timer::from_seconds(0.26, TimerMode::Once),
                returning: false,
                damage_dealt: false,
                target_entity,
                damage: raw_dmg,
                is_crit,
                class,
            });
        } else {
            let (mesh, mat, arc_h, aoe) = if class == UnitClass::Mage {
                (
                    assets_3d.magic_orb.clone(),
                    assets_3d.lightning.clone(),
                    0.8,
                    Some((target_grid.row, opponent_faction)),
                )
            } else {
                (
                    assets_3d.arrow_head.clone(),
                    assets_3d.arrow_glow.clone(),
                    1.4,
                    None,
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
                    timer: Timer::from_seconds(0.34, TimerMode::Once),
                    damage: raw_dmg,
                    is_heal: false,
                    is_crit,
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

pub fn update_dash_animations(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    assets_3d: Res<Game3dAssets>,
    mut rng: ResMut<BattleRng>,
    mut hit_stop: ResMut<HitStopManager>,
    mut camera_shake: ResMut<CameraShake>,
    mut turn_manager: ResMut<BattleTurnManager>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut DashAnimation,
        &mut ChibiSquashStretch,
    )>,
    mut target_query: Query<
        (Entity, &mut UnitStats, &Transform, &mut ChibiSquashStretch),
        (Without<DashAnimation>, Without<UnitHitRecoil>),
    >,
) {
    if hit_stop.active {
        return;
    }

    let dt = time.delta_secs() * speed.multiplier;

    for (entity, mut transform, mut dash, mut attacker_squash) in query.iter_mut() {
        dash.timer.tick(std::time::Duration::from_secs_f32(dt));
        let progress = dash.timer.fraction();

        if !dash.returning {
            let mut pos = dash.origin.lerp(dash.target, progress);
            pos.y += (progress * PI).sin() * 0.45; // Dynamic leap arc
            transform.translation = pos;

            // Squash & Stretch: Stretch in mid-leap, squash on impact
            if progress < 0.25 {
                attacker_squash.target_scale = Vec3::new(1.20, 0.78, 1.20);
            } else if progress < 0.85 {
                attacker_squash.target_scale = Vec3::new(0.82, 1.34, 0.82);
            } else {
                attacker_squash.target_scale = Vec3::new(1.30, 0.70, 1.30);
            }

            if dash.timer.finished() {
                if !dash.damage_dealt {
                    dash.damage_dealt = true;

                    // Hit Stop: 4 to 6 frames freeze for crunchy weight!
                    hit_stop.trigger(if dash.is_crit { 0.088 } else { 0.065 });

                    // Camera Shake
                    camera_shake.add_trauma(if dash.is_crit { 0.48 } else { 0.30 });

                    if let Ok((target_ent, mut target_stats, target_transform, mut target_squash)) =
                        target_query.get_mut(dash.target_entity)
                    {
                        let actual_dmg =
                            (dash.damage * (100.0 / (100.0 + target_stats.def))).max(5.0);
                        target_stats.hp -= actual_dmg;

                        // Target squashes under heavy blow
                        target_squash.target_scale = Vec3::new(1.35, 0.68, 1.35);

                        let text = if dash.is_crit {
                            format!("-{:.0} CRIT!", actual_dmg)
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
                            &mut rng,
                            dash.target,
                            &text,
                            col,
                            if dash.is_crit { 20.0 } else { 15.0 },
                        );

                        let hit_pos = target_transform.translation + Vec3::new(0.0, 1.1, 0.0);

                        // Flash dynamic point light at impact point
                        let flash_color = if dash.class == UnitClass::Knight {
                            Color::srgb(1.0, 0.82, 0.25)
                        } else {
                            Color::srgb(1.0, 0.18, 0.18)
                        };
                        commands.spawn((
                            PointLight {
                                color: flash_color,
                                intensity: 85_000.0,
                                range: 7.0,
                                shadows_enabled: false,
                                ..default()
                            },
                            Transform::from_translation(hit_pos),
                            PointLightFlash {
                                timer: Timer::from_seconds(0.14, TimerMode::Once),
                                initial_intensity: 85_000.0,
                            },
                        ));

                        if dash.class == UnitClass::Knight {
                            // Luminous Blade Slash Trail Arc
                            commands.spawn((
                                Mesh3d(assets_3d.slash_trail_mesh.clone()),
                                MeshMaterial3d(assets_3d.slash_trail_mat.clone()),
                                Transform::from_translation(hit_pos)
                                    .with_rotation(
                                        Quat::from_rotation_z(0.6) * Quat::from_rotation_x(0.3),
                                    )
                                    .with_scale(Vec3::splat(0.6)),
                                CombatVfx {
                                    timer: Timer::from_seconds(0.28, TimerMode::Once),
                                    initial_scale: Vec3::splat(0.6),
                                    target_scale: Vec3::splat(1.8),
                                    rotate_speed: 7.0,
                                },
                            ));

                            // Ground Shockwave Ring
                            commands.spawn((
                                Mesh3d(assets_3d.shockwave_ring.clone()),
                                MeshMaterial3d(assets_3d.arena_rim_mat.clone()),
                                Transform::from_translation(
                                    target_transform.translation + Vec3::new(0.0, 0.08, 0.0),
                                ),
                                CombatVfx {
                                    timer: Timer::from_seconds(0.25, TimerMode::Once),
                                    initial_scale: Vec3::splat(0.4),
                                    target_scale: Vec3::splat(1.5),
                                    rotate_speed: 0.0,
                                },
                            ));

                            // Landing Dust Puff
                            commands.spawn((
                                Mesh3d(assets_3d.dust_puff_mesh.clone()),
                                MeshMaterial3d(assets_3d.dust_mat.clone()),
                                Transform::from_translation(
                                    dash.target + Vec3::new(0.0, 0.05, 0.0),
                                ),
                                CombatVfx {
                                    timer: Timer::from_seconds(0.24, TimerMode::Once),
                                    initial_scale: Vec3::splat(0.5),
                                    target_scale: Vec3::splat(1.4),
                                    rotate_speed: 0.0,
                                },
                            ));

                            spawn_impact_sparks(
                                &mut commands,
                                &assets_3d,
                                &mut rng,
                                hit_pos,
                                assets_3d.spark_mat.clone(),
                                8,
                            );
                        } else {
                            // Twin X-Cross Slashes for Assassin
                            commands.spawn((
                                Mesh3d(assets_3d.x_slash.clone()),
                                MeshMaterial3d(assets_3d.x_slash_mat.clone()),
                                Transform::from_translation(hit_pos)
                                    .with_rotation(Quat::from_rotation_z(0.78)),
                                CombatVfx {
                                    timer: Timer::from_seconds(0.25, TimerMode::Once),
                                    initial_scale: Vec3::splat(0.7),
                                    target_scale: Vec3::splat(1.65),
                                    rotate_speed: 0.0,
                                },
                            ));
                            commands.spawn((
                                Mesh3d(assets_3d.x_slash.clone()),
                                MeshMaterial3d(assets_3d.poison_blade.clone()),
                                Transform::from_translation(hit_pos)
                                    .with_rotation(Quat::from_rotation_z(-0.78)),
                                CombatVfx {
                                    timer: Timer::from_seconds(0.25, TimerMode::Once),
                                    initial_scale: Vec3::splat(0.7),
                                    target_scale: Vec3::splat(1.65),
                                    rotate_speed: 0.0,
                                },
                            ));

                            // Shadow Dust
                            commands.spawn((
                                Mesh3d(assets_3d.shadow_disc.clone()),
                                MeshMaterial3d(assets_3d.shadow_aura.clone()),
                                Transform::from_translation(
                                    dash.target + Vec3::new(0.0, 0.05, 0.0),
                                ),
                                CombatVfx {
                                    timer: Timer::from_seconds(0.28, TimerMode::Once),
                                    initial_scale: Vec3::splat(0.4),
                                    target_scale: Vec3::splat(1.5),
                                    rotate_speed: 2.0,
                                },
                            ));

                            spawn_impact_sparks(
                                &mut commands,
                                &assets_3d,
                                &mut rng,
                                hit_pos,
                                assets_3d.poison_blade.clone(),
                                9,
                            );
                        }

                        // Target Recoil Jolt
                        let recoil_dir = (dash.target - dash.origin).normalize_or_zero() * 0.32;
                        commands.entity(target_ent).insert(UnitHitRecoil {
                            original_pos: target_transform.translation,
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
            pos.y += (progress * PI).sin() * 0.22;
            transform.translation = pos;

            if dash.timer.finished() {
                transform.translation = dash.origin;
                attacker_squash.target_scale = Vec3::ONE;
                commands.entity(entity).remove::<DashAnimation>();

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
    assets_3d: Res<Game3dAssets>,
    mut rng: ResMut<BattleRng>,
    mut hit_stop: ResMut<HitStopManager>,
    mut camera_shake: ResMut<CameraShake>,
    mut turn_manager: ResMut<BattleTurnManager>,
    mut query: Query<(Entity, &mut Transform, &mut Projectile3d)>,
    mut target_query: Query<
        (
            Entity,
            &mut UnitStats,
            &GridPos,
            &Transform,
            &mut ChibiSquashStretch,
        ),
        Without<Projectile3d>,
    >,
) {
    if hit_stop.active {
        return;
    }

    let dt = time.delta_secs() * speed.multiplier;

    for (proj_entity, mut transform, mut proj) in query.iter_mut() {
        proj.timer.tick(std::time::Duration::from_secs_f32(dt));
        let progress = proj.timer.fraction();

        let mut current_pos = proj.start.lerp(proj.target_pos, progress);
        current_pos.y += (progress * PI).sin() * proj.arc_height;
        transform.translation = current_pos;

        let dir = (proj.target_pos - proj.start).normalize_or_zero();
        if dir.length_squared() > 0.001 {
            transform.look_to(dir, Vec3::Y);
        }

        if proj.timer.finished() {
            if proj.is_heal {
                // --- CLERIC HEAL IMPACT VFX ---
                if let Ok((_, mut stats, _, target_transform, mut target_squash)) =
                    target_query.get_mut(proj.target_entity)
                {
                    stats.hp = (stats.hp + proj.damage).min(stats.max_hp);
                    target_squash.target_scale = Vec3::new(0.9, 1.25, 0.9);

                    spawn_floating_text(
                        &mut commands,
                        &mut rng,
                        proj.target_pos,
                        &format!("+{:.0} HEAL", proj.damage),
                        Color::srgb(0.25, 0.95, 0.4),
                        16.5,
                    );

                    let heal_pos = target_transform.translation + Vec3::new(0.0, 1.6, 0.0);

                    // Celestial Holy Light Flash
                    commands.spawn((
                        PointLight {
                            color: Color::srgb(1.0, 0.95, 0.5),
                            intensity: 95_000.0,
                            range: 8.0,
                            shadows_enabled: false,
                            ..default()
                        },
                        Transform::from_translation(heal_pos),
                        PointLightFlash {
                            timer: Timer::from_seconds(0.18, TimerMode::Once),
                            initial_intensity: 95_000.0,
                        },
                    ));

                    // Descending Divine Holy Light Pillar
                    commands.spawn((
                        Mesh3d(assets_3d.holy_pillar.clone()),
                        MeshMaterial3d(assets_3d.holy_pillar_mat.clone()),
                        Transform::from_translation(heal_pos).with_scale(Vec3::new(0.8, 1.0, 0.8)),
                        CombatVfx {
                            timer: Timer::from_seconds(0.40, TimerMode::Once),
                            initial_scale: Vec3::new(0.8, 1.0, 0.8),
                            target_scale: Vec3::new(1.4, 1.0, 1.4),
                            rotate_speed: 2.0,
                        },
                    ));

                    // Sacred Sanctuary Ground Ring
                    commands.spawn((
                        Mesh3d(assets_3d.holy_ground_ring.clone()),
                        MeshMaterial3d(assets_3d.holy_ground_mat.clone()),
                        Transform::from_translation(
                            target_transform.translation + Vec3::new(0.0, 0.08, 0.0),
                        ),
                        CombatVfx {
                            timer: Timer::from_seconds(0.38, TimerMode::Once),
                            initial_scale: Vec3::splat(0.4),
                            target_scale: Vec3::splat(1.8),
                            rotate_speed: 3.0,
                        },
                    ));

                    spawn_impact_sparks(
                        &mut commands,
                        &assets_3d,
                        &mut rng,
                        target_transform.translation + Vec3::new(0.0, 1.0, 0.0),
                        assets_3d.heal_glow.clone(),
                        9,
                    );
                }
            } else {
                // --- RANGED ATTACK IMPACT VFX ---
                hit_stop.trigger(if proj.is_crit { 0.085 } else { 0.060 });

                if let Ok((target_ent, mut stats, _, target_transform, mut target_squash)) =
                    target_query.get_mut(proj.target_entity)
                {
                    let actual_dmg = (proj.damage * (100.0 / (100.0 + stats.def))).max(5.0);
                    stats.hp -= actual_dmg;

                    target_squash.target_scale = Vec3::new(1.32, 0.70, 1.32);

                    let text = if proj.is_crit {
                        format!("-{:.0} CRIT!", actual_dmg)
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
                        &mut rng,
                        proj.target_pos,
                        &text,
                        col,
                        if proj.is_crit { 20.0 } else { 15.0 },
                    );

                    let hit_pos = target_transform.translation + Vec3::new(0.0, 1.0, 0.0);

                    if proj.class == UnitClass::Archer {
                        camera_shake.add_trauma(if proj.is_crit { 0.40 } else { 0.20 });

                        // Emerald Snipe Flash Light
                        commands.spawn((
                            PointLight {
                                color: Color::srgb(0.25, 1.0, 0.45),
                                intensity: 75_000.0,
                                range: 6.0,
                                shadows_enabled: false,
                                ..default()
                            },
                            Transform::from_translation(hit_pos),
                            PointLightFlash {
                                timer: Timer::from_seconds(0.12, TimerMode::Once),
                                initial_intensity: 75_000.0,
                            },
                        ));

                        spawn_impact_sparks(
                            &mut commands,
                            &assets_3d,
                            &mut rng,
                            hit_pos,
                            assets_3d.arrow_glow.clone(),
                            7,
                        );
                    } else if proj.class == UnitClass::Mage {
                        camera_shake.add_trauma(0.55); // Heavy AOE screen shake!

                        // Arcane Violet Blast Flash Light
                        commands.spawn((
                            PointLight {
                                color: Color::srgb(0.85, 0.35, 1.0),
                                intensity: 110_000.0,
                                range: 8.0,
                                shadows_enabled: false,
                                ..default()
                            },
                            Transform::from_translation(hit_pos),
                            PointLightFlash {
                                timer: Timer::from_seconds(0.16, TimerMode::Once),
                                initial_intensity: 110_000.0,
                            },
                        ));

                        // Mage Expanding Violet Shockwave Ring
                        commands.spawn((
                            Mesh3d(assets_3d.shockwave_ring.clone()),
                            MeshMaterial3d(assets_3d.shockwave_mat.clone()),
                            Transform::from_translation(
                                target_transform.translation + Vec3::new(0.0, 0.12, 0.0),
                            ),
                            CombatVfx {
                                timer: Timer::from_seconds(0.36, TimerMode::Once),
                                initial_scale: Vec3::splat(0.3),
                                target_scale: Vec3::splat(2.5),
                                rotate_speed: 4.5,
                            },
                        ));
                        spawn_impact_sparks(
                            &mut commands,
                            &assets_3d,
                            &mut rng,
                            hit_pos,
                            assets_3d.lightning.clone(),
                            10,
                        );
                    }

                    // Recoil jolt on hit
                    let recoil_dir = (proj.target_pos - proj.start).normalize_or_zero() * 0.28;
                    commands.entity(target_ent).insert(UnitHitRecoil {
                        original_pos: target_transform.translation,
                        recoil_offset: recoil_dir,
                        timer: Timer::from_seconds(0.18, TimerMode::Once),
                    });
                }

                // Mage Row AOE Splash
                if let Some((row, target_faction)) = proj.aoe_row {
                    let splash_dmg = proj.damage * 0.48;
                    for (
                        other_ent,
                        mut other_stats,
                        other_grid,
                        other_transform,
                        mut other_squash,
                    ) in target_query.iter_mut()
                    {
                        if other_ent != proj.target_entity
                            && other_grid.row == row
                            && other_grid.faction == target_faction
                            && other_stats.hp > 0.0
                        {
                            let actual_splash =
                                (splash_dmg * (100.0 / (100.0 + other_stats.def))).max(3.5);
                            other_stats.hp -= actual_splash;
                            other_squash.target_scale = Vec3::new(1.25, 0.75, 1.25);

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
                                &assets_3d,
                                &mut rng,
                                other_transform.translation + Vec3::new(0.0, 0.9, 0.0),
                                assets_3d.lightning.clone(),
                                6,
                            );
                        }
                    }
                }
            }

            commands.entity(proj_entity).despawn_recursive();

            turn_manager.active_attacker = None;
            turn_manager.cooldown_timer.reset();
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
