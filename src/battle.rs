use crate::audio::{PlaySoundEvent, SoundEffect};
use crate::board::grid_to_world_pos;
use crate::economy::PlayerEconomy;
use crate::types::*;
use crate::units::{BossUnit, ChibiSquashStretch, Unit};
use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Component)]
pub struct ActionGauge {
    pub current: f32,
}

#[derive(Component)]
pub struct FloatingText2d {
    pub velocity: Vec2,
    pub timer: Timer,
}

#[derive(Component)]
pub struct DashAnimation2d {
    pub origin: Vec2,
    pub target: Vec2,
    pub timer: Timer,
    pub returning: bool,
    pub damage_dealt: bool,
    pub target_entity: Entity,
    pub attacker_entity: Entity,
    pub damage: f32,
    pub is_crit: bool,
    pub is_ultimate: bool,
    pub class: UnitClass,
}

#[derive(Component)]
pub struct Projectile2d {
    pub start: Vec2,
    pub target_pos: Vec2,
    pub target_entity: Entity,
    pub timer: Timer,
    pub damage: f32,
    pub is_heal: bool,
    pub is_crit: bool,
    pub is_ultimate: bool,
    pub aoe_row: Option<(usize, Faction)>,
    pub arc_height: f32,
    pub class: UnitClass,
}

#[derive(Component)]
pub struct CombatVfx2d {
    pub timer: Timer,
    pub initial_scale: Vec2,
    pub target_scale: Vec2,
    pub rotate_speed: f32,
}

#[derive(Component)]
pub struct SparkParticle2d {
    pub velocity: Vec2,
    pub timer: Timer,
    pub initial_color: Color,
}

#[derive(Component)]
pub struct UnitHitRecoil2d {
    pub original_pos: Vec2,
    pub recoil_offset: Vec2,
    pub timer: Timer,
}

#[derive(Component)]
pub struct ActiveTurnSpotlight2d {
    #[allow(dead_code)]
    pub attacker_entity: Entity,
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

#[derive(Resource, Default)]
pub struct CameraShake2d {
    pub trauma: f32,
}

impl CameraShake2d {
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
    mut shake: ResMut<CameraShake2d>,
    mut rng: ResMut<BattleRng>,
    mut cam_query: Query<&mut Transform, With<crate::MainCamera2d>>,
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
    units: Query<Entity, With<Unit>>,
    mut turn_manager: ResMut<BattleTurnManager>,
    mut player_units: Query<(&Unit, &mut UnitStats), Without<DeadUnit>>,
) {
    turn_manager.active_attacker = None;
    turn_manager.cooldown_timer.reset();

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
                stats.def += if unit.class == UnitClass::Knight { 35.0 } else { 15.0 };
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

pub fn spawn_floating_text(
    commands: &mut Commands,
    rng: &mut BattleRng,
    pos: Vec2,
    text: &str,
    color: Color,
    font_size: f32,
) {
    let offset_x = rng.random_range(-14.0, 14.0);
    let offset_y = rng.random_range(16.0, 28.0);
    let vx = rng.random_range(-22.0, 22.0);
    let vy = rng.random_range(52.0, 78.0);

    commands.spawn((
        Text2d::new(text),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(pos.x + offset_x, pos.y + offset_y, 80.0),
        FloatingText2d {
            velocity: Vec2::new(vx, vy),
            timer: Timer::from_seconds(0.75, TimerMode::Once),
        },
    ));
}

pub fn update_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut FloatingText2d)>,
) {
    let dt = time.delta_secs() * speed.multiplier;
    for (entity, mut transform, mut text_color, mut float) in query.iter_mut() {
        float.timer.tick(std::time::Duration::from_secs_f32(dt));

        transform.translation.x += float.velocity.x * dt;
        transform.translation.y += float.velocity.y * dt;
        float.velocity.y -= 75.0 * dt;

        let progress = float.timer.fraction();

        let scale = if progress < 0.20 {
            0.6 + (progress / 0.20) * 0.5
        } else {
            1.1 - (progress - 0.20) * 0.25
        };
        transform.scale = Vec3::splat(scale);

        if progress > 0.60 {
            let alpha = 1.0 - (progress - 0.60) / 0.40;
            let current = text_color.0.to_srgba();
            text_color.0 =
                Color::srgba(current.red, current.green, current.blue, alpha.clamp(0.0, 1.0));
        }

        if float.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn spawn_impact_sparks(
    commands: &mut Commands,
    rng: &mut BattleRng,
    pos: Vec2,
    color: Color,
    count: usize,
) {
    for _ in 0..count {
        let angle = rng.random_range(0.0, PI * 2.0);
        let speed = rng.random_range(40.0, 120.0);
        let vx = angle.cos() * speed;
        let vy = angle.sin() * speed;

        commands.spawn((
            Sprite {
                custom_size: Some(Vec2::splat(rng.random_range(3.0, 6.0))),
                color,
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 60.0),
            SparkParticle2d {
                velocity: Vec2::new(vx, vy),
                timer: Timer::from_seconds(rng.random_range(0.20, 0.38), TimerMode::Once),
                initial_color: color,
            },
        ));
    }
}

pub fn update_spark_particles(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut SparkParticle2d)>,
) {
    let dt = time.delta_secs() * speed.multiplier;
    for (entity, mut transform, mut sprite, mut spark) in query.iter_mut() {
        spark.timer.tick(std::time::Duration::from_secs_f32(dt));

        transform.translation.x += spark.velocity.x * dt;
        transform.translation.y += spark.velocity.y * dt;
        spark.velocity.y -= 140.0 * dt;
        spark.velocity.x *= 0.94;

        let progress = spark.timer.fraction();
        let alpha = (1.0 - progress).clamp(0.0, 1.0);
        let c = spark.initial_color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);

        if spark.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn update_combat_vfx(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut CombatVfx2d)>,
) {
    let dt = time.delta_secs() * speed.multiplier;
    for (entity, mut transform, mut sprite, mut vfx) in query.iter_mut() {
        vfx.timer.tick(std::time::Duration::from_secs_f32(dt));
        let progress = vfx.timer.fraction();

        let cur_scale = vfx.initial_scale.lerp(vfx.target_scale, progress);
        transform.scale = Vec3::new(cur_scale.x, cur_scale.y, 1.0);

        transform.rotate_z(vfx.rotate_speed * dt);

        let alpha = (1.0 - progress).clamp(0.0, 1.0);
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);

        if vfx.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn update_hit_recoil(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(Entity, &mut Transform, &mut UnitHitRecoil2d)>,
) {
    let dt = time.delta_secs() * speed.multiplier;
    for (entity, mut transform, mut recoil) in query.iter_mut() {
        recoil.timer.tick(std::time::Duration::from_secs_f32(dt));
        let progress = recoil.timer.fraction();

        let t = progress * PI * 2.0;
        let damping = 1.0 - progress;
        let offset = recoil.recoil_offset * (t.sin() * damping * 0.7);

        transform.translation.x = recoil.original_pos.x + offset.x;
        transform.translation.y = recoil.original_pos.y + offset.y;

        if recoil.timer.finished() {
            transform.translation.x = recoil.original_pos.x;
            transform.translation.y = recoil.original_pos.y;
            commands.entity(entity).remove::<UnitHitRecoil2d>();
        }
    }
}

pub fn update_turn_spotlight(
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut query: Query<(&mut Transform, &mut Sprite), With<ActiveTurnSpotlight2d>>,
) {
    let dt = time.delta_secs() * speed.multiplier;
    let t = time.elapsed_secs() * speed.multiplier;
    for (mut transform, mut sprite) in query.iter_mut() {
        transform.rotate_z(2.0 * dt);
        let pulse = (t * 5.0).sin() * 0.12 + 1.0;
        transform.scale = Vec3::splat(pulse);

        let alpha = (t * 4.0).sin() * 0.2 + 0.65;
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

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
        info!("[ACTION] [ULTIMATE] {:?} {:?} cast ULTIMATE: {}!", faction, class, class.ultimate_name());
    } else {
        info!("[ACTION] {:?} {:?} took turn (Speed: {:.0}, ATB full)", faction, class, stats.speed);
    }

    if let Ok((_, _, mut actor_stats, _, _, mut gauge, maybe_squash)) = units.get_mut(actor_entity) {
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
        ),
        (Without<DashAnimation2d>, Without<UnitHitRecoil2d>),
    >,
    mut attacker_stats_query: Query<&mut UnitStats, With<DashAnimation2d>>,
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
                    )) = target_query.get_mut(dash.target_entity)
                    {
                        let raw_dmg =
                            (dash.damage * (100.0 / (100.0 + target_stats.def))).max(5.0);
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
                        target_stats.mana = (target_stats.mana + 12.0).min(target_stats.max_mana);
                        info!(
                            "[COMBAT MELEE] {:?} struck target for {:.1} dmg (Crit: {}, Ult: {}) -> Target HP: {:.1}/{:.1}",
                            dash.class, actual_dmg, dash.is_crit, dash.is_ultimate, target_stats.hp, target_stats.max_hp
                        );

                        if dash.is_ultimate && dash.class == UnitClass::Knight {
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
                            if let Ok(mut atk_stats) =
                                attacker_stats_query.get_mut(dash.attacker_entity)
                            {
                                atk_stats.mana = (atk_stats.mana + 50.0).min(atk_stats.max_mana);
                                spawn_floating_text(
                                    &mut commands,
                                    &mut rng,
                                    dash.origin + Vec2::new(0.0, 30.0),
                                    "+50 MANA EXECUTE!",
                                    Color::srgb(1.0, 0.88, 0.2),
                                    15.0,
                                );
                            }
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

                        if dash.class == UnitClass::Knight {
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
                                    target_scale: Vec2::splat(if dash.is_ultimate { 2.4 } else { 1.8 }),
                                    rotate_speed: 6.0,
                                },
                            ));

                            commands.spawn((
                                Sprite {
                                    custom_size: Some(Vec2::splat(if dash.is_ultimate { 44.0 } else { 28.0 })),
                                    color: Color::srgba(1.0, 0.9, 0.4, 0.8),
                                    ..default()
                                },
                                Transform::from_xyz(hit_pos.x, hit_pos.y, 38.0),
                                CombatVfx2d {
                                    timer: Timer::from_seconds(0.25, TimerMode::Once),
                                    initial_scale: Vec2::splat(0.5),
                                    target_scale: Vec2::splat(if dash.is_ultimate { 3.0 } else { 2.2 }),
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
                                    target_scale: Vec2::splat(if dash.is_ultimate { 2.2 } else { 1.65 }),
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
                                    target_scale: Vec2::splat(if dash.is_ultimate { 2.2 } else { 1.65 }),
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

                        let recoil_dir = (dash.target - dash.origin).normalize_or_zero() * 16.0;
                        commands.entity(target_ent).insert(UnitHitRecoil2d {
                            original_pos: target_transform.translation.xy(),
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
                if let Ok((_, mut stats, _, target_transform, mut maybe_target_squash)) =
                    target_query.get_mut(proj.target_entity)
                {
                    stats.hp = (stats.hp + proj.damage).min(stats.max_hp);
                    info!("[COMBAT HEAL] Holy Grace healed target for +{:.1} HP -> New HP: {:.1}/{:.1}", proj.damage, stats.hp, stats.max_hp);
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
                            custom_size: Some(Vec2::splat(if proj.is_ultimate { 48.0 } else { 32.0 })),
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

                if let Ok((target_ent, mut stats, _, target_transform, mut maybe_target_squash)) =
                    target_query.get_mut(proj.target_entity)
                {
                    let raw_dmg = (proj.damage * (100.0 / (100.0 + stats.def))).max(5.0);
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
                    stats.mana = (stats.mana + 12.0).min(stats.max_mana);
                    info!(
                        "[COMBAT RANGED] {:?} projectile hit for {:.1} dmg (Crit: {}, Ult: {}) -> Target HP: {:.1}/{:.1}",
                        proj.class, actual_dmg, proj.is_crit, proj.is_ultimate, stats.hp, stats.max_hp
                    );
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
                    } else if proj.class == UnitClass::Mage {
                        camera_shake.add_trauma(if proj.is_ultimate { 0.65 } else { 0.50 });

                        commands.spawn((
                            Sprite {
                                custom_size: Some(Vec2::splat(if proj.is_ultimate { 52.0 } else { 36.0 })),
                                color: Color::srgba(0.85, 0.35, 1.0, 0.8),
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
                            Color::srgb(0.8, 0.3, 1.0),
                            if proj.is_ultimate { 15 } else { 10 },
                        );
                    }

                    let recoil_dir = (proj.target_pos - proj.start).normalize_or_zero() * 14.0;
                    commands.entity(target_ent).insert(UnitHitRecoil2d {
                        original_pos: target_transform.translation.xy(),
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
                    ) in target_query.iter_mut()
                    {
                        if other_ent != proj.target_entity
                            && other_grid.row == row
                            && other_grid.faction == target_faction
                            && other_stats.hp > 0.0
                        {
                            let actual_splash_raw =
                                (splash_dmg * (100.0 / (100.0 + other_stats.def))).max(3.5);
                            let mut actual_splash = actual_splash_raw;
                            if other_stats.shield > 0.0 {
                                let absorbed = actual_splash.min(other_stats.shield);
                                other_stats.shield -= absorbed;
                                actual_splash -= absorbed;
                            }
                            other_stats.hp -= actual_splash;
                            other_stats.mana = (other_stats.mana + 12.0).min(other_stats.max_mana);
                            if let Some(ref mut s) = maybe_other_squash {
                                s.target_scale = Vec3::new(1.25, 0.75, 1.0);
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

pub fn check_unit_deaths(
    mut commands: Commands,
    mut units: Query<
        (Entity, &Unit, &UnitStats, &mut Visibility, &mut Transform),
        (With<Unit>, Without<DeadUnit>),
    >,
) {
    for (entity, unit, stats, mut vis, mut transform) in units.iter_mut() {
        if stats.hp <= 0.0 {
            info!("[DEATH] {:?} {:?} has fallen in battle!", unit.faction, unit.class);
            transform.translation.y = -9999.0;
            *vis = Visibility::Hidden;
            commands.entity(entity).insert(DeadUnit);
        }
    }
}

pub fn check_battle_end(
    units: Query<(&Unit, &UnitStats), Without<DeadUnit>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut economy: ResMut<PlayerEconomy>,
    mut rng: ResMut<BattleRng>,
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
        info!("==================== [ROUND VICTORY] ====================");
        info!("[VICTORY] All enemies defeated! Surviving player heroes: {}", alive_player);
        sound_events.send(PlaySoundEvent(SoundEffect::Victory));
        economy.apply_round_income(true, &mut rng);
        next_state.set(GameState::Victory);
    } else if alive_player == 0 {
        info!("==================== [ROUND DEFEAT] ====================");
        info!("[DEFEAT] All player heroes were eliminated! Surviving enemies: {}", alive_enemy);
        sound_events.send(PlaySoundEvent(SoundEffect::Defeat));
        economy.apply_round_income(false, &mut rng);
        next_state.set(GameState::Defeat);
    }
}


#[cfg(test)]
mod tests {
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
        let player_ent = app.world_mut().spawn((
            Unit { class: UnitClass::Knight, faction: Faction::Player },
            UnitClass::Knight.base_stats(),
            GridPos { col: 2, row: 1, faction: Faction::Player },
            ActionGauge { current: 100.0 }, // near full
            Transform::from_xyz(-100.0, 0.0, 10.0),
        )).id();

        // Spawn an enemy Knight
        let _enemy_ent = app.world_mut().spawn((
            Unit { class: UnitClass::Knight, faction: Faction::Enemy },
            UnitClass::Knight.base_stats(),
            GridPos { col: 0, row: 1, faction: Faction::Enemy },
            ActionGauge { current: 10.0 },
            Transform::from_xyz(100.0, 0.0, 10.0),
        )).id();

        app.add_systems(Update, battle_tick_system);

        // Finish cooldown timer and set dt
        app.world_mut().resource_mut::<BattleTurnManager>().cooldown_timer.tick(std::time::Duration::from_secs(1));
        // Also tick ActionGauge manually or let Time run
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(500));
        app.update();

        // Check if ActionGauge increased or attacker was activated
        let turn_mgr = app.world().resource::<BattleTurnManager>();
        assert_eq!(turn_mgr.active_attacker, Some(player_ent), "Player knight should have triggered an attack!");
    }
}
