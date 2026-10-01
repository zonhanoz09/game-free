use crate::battle::{ActionGauge, HitStopManager};
use crate::board::grid_to_world_pos;
use crate::types::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;

#[derive(Component)]
pub struct Unit {
    pub class: UnitClass,
    pub faction: Faction,
}

#[derive(Component)]
pub struct HealthBarFill2d;

#[derive(Component)]
pub struct ManaBarFill2d;

#[derive(Component)]
pub struct StaminaBarFill2d;

#[derive(Component)]
pub struct BossUnit;

#[derive(Component)]
pub struct HealthBarRoot2d;

#[derive(Component)]
pub struct UnitVisualRoot;

#[derive(Component)]
pub struct IdleBobbing {
    pub base_y: f32,
    pub phase: f32,
}

#[derive(Component, Clone)]
pub struct ChibiSquashStretch {
    pub current_scale: Vec3,
    pub target_scale: Vec3,
    pub recovery_speed: f32,
}

impl Default for ChibiSquashStretch {
    fn default() -> Self {
        Self {
            current_scale: Vec3::ONE,
            target_scale: Vec3::ONE,
            recovery_speed: 12.0,
        }
    }
}

pub fn spawn_unit(
    commands: &mut Commands,
    textures: &GameTextures,
    unit_class: UnitClass,
    faction: Faction,
    col: usize,
    row: usize,
) -> Entity {
    spawn_unit_ext(commands, textures, unit_class, faction, col, row, 1, false)
}

pub fn spawn_unit_ext(
    commands: &mut Commands,
    textures: &GameTextures,
    unit_class: UnitClass,
    faction: Faction,
    col: usize,
    row: usize,
    star_level: u8,
    is_boss: bool,
) -> Entity {
    let world_pos = grid_to_world_pos(col, row, faction);
    let z_depth = 10.0 + (row as f32 * -0.5);

    let (outer_border_col, inner_border_col) = match faction {
        Faction::Player => (Color::srgb(0.25, 0.65, 1.0), Color::srgb(0.10, 0.25, 0.55)),
        Faction::Enemy => (Color::srgb(1.0, 0.35, 0.35), Color::srgb(0.60, 0.15, 0.18)),
    };

    let mut stats = unit_class.base_stats();
    if is_boss {
        stats.max_hp = 1500.0;
        stats.hp = 1500.0;
        stats.atk = 80.0;
        stats.def = 60.0;
        stats.speed = 22.0;
    } else if star_level > 1 {
        let (hp_mult, atk_mult) = match star_level {
            2 => (1.8, 1.6),
            _ => (2.8, 2.5),
        };
        stats.max_hp = (stats.max_hp * hp_mult).round();
        stats.hp = stats.max_hp;
        stats.atk = (stats.atk * atk_mult).round();
    }

    let token_scale = if is_boss { 1.55 } else if star_level == 3 { 1.18 } else if star_level == 2 { 1.08 } else { 1.0 };

    let mut entity_cmds = commands.spawn((
        Unit {
            class: unit_class,
            faction,
        },
        crate::economy::StarLevel(star_level),
        stats,
        GridPos { col, row, faction },
        ActionGauge { current: 0.0 },
        ChibiSquashStretch::default(),
        Transform::from_xyz(world_pos.x, world_pos.y, z_depth).with_scale(Vec3::splat(token_scale)),
        Visibility::default(),
    ));

    if is_boss {
        entity_cmds.insert(BossUnit);
    }

    entity_cmds
        .with_children(|parent| {
            // 1. Under-Token Ambient Shadow
            parent.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(76.0, 22.0)),
                    color: Color::srgba(0.02, 0.03, 0.05, 0.55),
                    ..default()
                },
                Transform::from_xyz(0.0, -32.0, -0.5),
            ));

            // 2. Animated Character Visual Root (Handles Idle Bobbing & Squash/Stretch)
            parent
                .spawn((
                    UnitVisualRoot,
                    IdleBobbing {
                        base_y: 0.0,
                        phase: (col * 3 + row) as f32 * 1.15,
                    },
                    ChibiSquashStretch::default(),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                    Visibility::default(),
                ))
                .with_children(|vis_parent| {
                    // Outer Token Base Ring
                    vis_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::splat(68.0)),
                            color: outer_border_col,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.0),
                    ));

                    // Inner Token Accent Rim
                    vis_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::splat(64.0)),
                            color: inner_border_col,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.1),
                    ));

                    // Token Dark Portrait Core Matting
                    vis_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::splat(58.0)),
                            color: Color::srgb(0.08, 0.10, 0.15),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.2),
                    ));

                    // 2D Character Portrait Artwork Sprite
                    let portrait_tex = textures.get_unit_texture(unit_class);
                    let flip_x = faction == Faction::Enemy;

                    vis_parent.spawn((
                        Sprite {
                            image: portrait_tex,
                            custom_size: Some(Vec2::splat(56.0)),
                            flip_x,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.3),
                    ));

                    // Subtle Character Class Corner Badge
                    vis_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::splat(16.0)),
                            color: unit_class.color(),
                            ..default()
                        },
                        Transform::from_xyz(-22.0, 22.0, 0.4),
                    ));
                });

            // 3. Role Name Plate Banner below token
            parent
                .spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(60.0, 14.0)),
                        color: Color::srgba(0.05, 0.07, 0.10, 0.85),
                        ..default()
                    },
                    Transform::from_xyz(0.0, -31.0, 0.5),
                ))
                .with_child((
                    Text2d::new(if is_boss { "[BOSS] TITAN".to_string() } else if star_level > 1 { format!("{} {}", unit_class.name().to_uppercase(), crate::economy::StarLevel(star_level).badge()) } else { unit_class.name().to_uppercase() }),
                    TextFont {
                        font_size: if is_boss { 9.5 } else { 9.0 },
                        ..default()
                    },
                    TextColor(if is_boss { Color::srgb(1.0, 0.25, 0.25) } else if star_level > 1 { crate::economy::StarLevel(star_level).color() } else { unit_class.color() }),
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));

            // 4. Overhead Floating Triple Bars: Health (HP), Mana (MP), & Action Gauge (ATB)
            parent
                .spawn((
                    HealthBarRoot2d,
                    Transform::from_xyz(0.0, 48.0, 1.0),
                    Visibility::default(),
                ))
                .with_children(|bar_parent| {
                    // HP Bar Outer Frame
                    bar_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(64.0, 7.0)),
                            color: Color::srgb(0.06, 0.08, 0.12),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.0),
                    ));

                    // Dynamic HP Fill Bar (Anchored at CenterLeft)
                    bar_parent.spawn((
                        HealthBarFill2d,
                        Sprite {
                            custom_size: Some(Vec2::new(62.0, 5.0)),
                            color: Color::srgb(0.2, 0.85, 0.3),
                            anchor: Anchor::CenterLeft,
                            ..default()
                        },
                        Transform::from_xyz(-31.0, 0.0, 0.1),
                    ));

                    // Mana Bar Outer Frame
                    bar_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(64.0, 4.5)),
                            color: Color::srgb(0.04, 0.05, 0.09),
                            ..default()
                        },
                        Transform::from_xyz(0.0, -5.5, 0.0),
                    ));

                    // Dynamic Mana Fill Bar (Arcane Blue / Ready Gold)
                    bar_parent.spawn((
                        ManaBarFill2d,
                        Sprite {
                            custom_size: Some(Vec2::new(62.0, 3.5)),
                            color: Color::srgb(0.22, 0.55, 1.0),
                            anchor: Anchor::CenterLeft,
                            ..default()
                        },
                        Transform::from_xyz(-31.0, -5.5, 0.1),
                    ));

                    // Stamina / Action Gauge Bar Frame
                    bar_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(64.0, 3.5)),
                            color: Color::srgb(0.04, 0.05, 0.08),
                            ..default()
                        },
                        Transform::from_xyz(0.0, -10.0, 0.0),
                    ));

                    // Dynamic Stamina / Action Gauge Fill Bar (Cyan Energy)
                    bar_parent.spawn((
                        StaminaBarFill2d,
                        Sprite {
                            custom_size: Some(Vec2::new(62.0, 2.5)),
                            color: Color::srgb(0.2, 0.85, 1.0),
                            anchor: Anchor::CenterLeft,
                            ..default()
                        },
                        Transform::from_xyz(-31.0, -10.0, 0.1),
                    ));
                });
        })
        .id()
}

pub fn animate_idle_bobbing(
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    hit_stop: Res<HitStopManager>,
    mut bob_query: Query<
        (&mut Transform, &IdleBobbing),
        (With<UnitVisualRoot>, Without<DeadUnit>),
    >,
    mut root_query: Query<
        (
            &mut Transform,
            &mut ChibiSquashStretch,
            Option<&BossUnit>,
            Option<&crate::economy::StarLevel>,
        ),
        (With<Unit>, Without<DeadUnit>, Without<UnitVisualRoot>),
    >,
) {
    if hit_stop.active {
        return;
    }
    let dt = time.delta_secs() * speed.multiplier;
    let t = time.elapsed_secs() * speed.multiplier;
    for (mut transform, bob) in bob_query.iter_mut() {
        let offset = (t * 3.2 + bob.phase).sin() * 2.2;
        transform.translation.y = bob.base_y + offset;
    }

    for (mut transform, mut squash, is_boss, star_level) in root_query.iter_mut() {
        let base_scale = if is_boss.is_some() {
            1.55
        } else if let Some(s) = star_level {
            if s.0 == 3 {
                1.18
            } else if s.0 == 2 {
                1.08
            } else {
                1.0
            }
        } else {
            1.0
        };

        squash.target_scale = squash
            .target_scale
            .lerp(Vec3::splat(base_scale), (dt * 8.0).min(1.0));
        squash.current_scale = squash
            .current_scale
            .lerp(squash.target_scale, (dt * squash.recovery_speed).min(1.0));
        transform.scale = squash.current_scale;
    }
}

pub fn update_unit_health_bars(
    units: Query<(&UnitStats, &Children, Option<&ActionGauge>), (With<Unit>, Without<DeadUnit>)>,
    mut hp_fill_query: Query<
        (&mut Transform, &mut Sprite),
        (
            With<HealthBarFill2d>,
            Without<ManaBarFill2d>,
            Without<StaminaBarFill2d>,
        ),
    >,
    mut mana_fill_query: Query<
        (&mut Transform, &mut Sprite),
        (
            With<ManaBarFill2d>,
            Without<HealthBarFill2d>,
            Without<StaminaBarFill2d>,
        ),
    >,
    mut stamina_fill_query: Query<
        &mut Transform,
        (
            With<StaminaBarFill2d>,
            Without<HealthBarFill2d>,
            Without<ManaBarFill2d>,
        ),
    >,
    roots: Query<&Children, With<HealthBarRoot2d>>,
) {
    for (stats, unit_children, maybe_gauge) in units.iter() {
        let hp_ratio = (stats.hp / stats.max_hp).clamp(0.0, 1.0);
        let mana_ratio = (stats.mana / stats.max_mana).clamp(0.0, 1.0);
        let stamina_ratio = maybe_gauge.map_or(0.0, |g| (g.current / 100.0).clamp(0.0, 1.0));

        for child in unit_children.iter() {
            if let Ok(bar_children) = roots.get(*child) {
                for bar_child in bar_children.iter() {
                    // Update HP Fill Bar
                    if let Ok((mut transform, mut sprite)) = hp_fill_query.get_mut(*bar_child) {
                        transform.scale.x = hp_ratio;

                        if stats.shield > 0.0 {
                            sprite.color = Color::srgb(0.35, 0.75, 1.0); // Shielded Cyan
                        } else if hp_ratio > 0.55 {
                            sprite.color = Color::srgb(0.20, 0.85, 0.35); // Healthy Green
                        } else if hp_ratio > 0.25 {
                            sprite.color = Color::srgb(0.95, 0.75, 0.15); // Caution Yellow
                        } else {
                            sprite.color = Color::srgb(0.95, 0.20, 0.20); // Critical Red
                        }
                    }

                    // Update Mana Fill Bar
                    if let Ok((mut transform, mut sprite)) = mana_fill_query.get_mut(*bar_child) {
                        transform.scale.x = mana_ratio;
                        if mana_ratio >= 1.0 {
                            sprite.color = Color::srgb(1.0, 0.86, 0.25); // Ultimate Ready Gold!
                        } else {
                            sprite.color = Color::srgb(0.22, 0.55, 1.0); // Arcane Mana Blue
                        }
                    }

                    // Update Stamina / Action Gauge Bar
                    if let Ok(mut transform) = stamina_fill_query.get_mut(*bar_child) {
                        transform.scale.x = stamina_ratio;
                    }
                }
            }
        }
    }
}
