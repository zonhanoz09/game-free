use crate::battle::HitStopManager;
use crate::board::{bench_world_pos, grid_to_world_pos};
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
pub struct BossUnit;

#[derive(Component)]
pub struct HealthBarRoot2d;

#[derive(Component)]
pub struct EffectIconsRoot2d;

#[derive(Component)]
#[allow(dead_code)]
pub struct EffectIconsText2d;

#[derive(Component)]
pub struct EffectBadgeSlot {
    pub index: usize,
}

#[derive(Component)]
#[allow(dead_code)]
pub struct EffectBadgeText {
    #[allow(dead_code)]
    pub index: usize,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct BadgeInfo {
    pub label: &'static str,
    pub bg_color: Color,
    pub text_color: Color,
    pub width: f32,
}

#[allow(dead_code)]
pub fn get_badge_info(effect_id: &str, name: &str) -> BadgeInfo {
    let lower_id = effect_id.to_lowercase();
    let lower_name = name.to_lowercase();

    if lower_id.contains("shield") || lower_name.contains("khiên") || lower_name.contains("thuẫn") {
        BadgeInfo {
            label: "KHIÊN",
            bg_color: Color::srgb(0.05, 0.60, 0.90),
            text_color: Color::WHITE,
            width: 32.0,
        }
    } else if lower_id.contains("burn") || lower_name.contains("hỏa") || lower_name.contains("bỏng") {
        BadgeInfo {
            label: "HỎA",
            bg_color: Color::srgb(0.92, 0.32, 0.08),
            text_color: Color::WHITE,
            width: 28.0,
        }
    } else if lower_id.contains("poison") || lower_name.contains("độc") {
        BadgeInfo {
            label: "ĐỘC",
            bg_color: Color::srgb(0.18, 0.72, 0.22),
            text_color: Color::WHITE,
            width: 28.0,
        }
    } else if lower_id.contains("freeze") || lower_name.contains("băng") {
        BadgeInfo {
            label: "BĂNG",
            bg_color: Color::srgb(0.20, 0.70, 0.95),
            text_color: Color::srgb(0.05, 0.15, 0.3),
            width: 30.0,
        }
    } else if lower_id.contains("stun") || lower_name.contains("choáng") {
        BadgeInfo {
            label: "CHOÁNG",
            bg_color: Color::srgb(0.95, 0.75, 0.10),
            text_color: Color::srgb(0.1, 0.1, 0.1),
            width: 38.0,
        }
    } else if lower_id.contains("bleed") || lower_name.contains("máu") {
        BadgeInfo {
            label: "MÁU",
            bg_color: Color::srgb(0.85, 0.12, 0.18),
            text_color: Color::WHITE,
            width: 28.0,
        }
    } else if lower_id.contains("syn_vanguard") || lower_name.contains("thiết vệ") {
        BadgeInfo {
            label: "THIẾT VỆ",
            bg_color: Color::srgb(0.12, 0.38, 0.85),
            text_color: Color::WHITE,
            width: 42.0,
        }
    } else if lower_id.contains("syn_sharpshooter") || lower_name.contains("thần xạ") {
        BadgeInfo {
            label: "THẦN XẠ",
            bg_color: Color::srgb(0.90, 0.50, 0.08),
            text_color: Color::WHITE,
            width: 42.0,
        }
    } else if lower_id.contains("syn_arcanist") || lower_name.contains("kỳ môn") {
        BadgeInfo {
            label: "KỲ MÔN",
            bg_color: Color::srgb(0.55, 0.18, 0.85),
            text_color: Color::WHITE,
            width: 40.0,
        }
    } else if lower_id.contains("syn_shadow") || lower_name.contains("ám ảnh") {
        BadgeInfo {
            label: "ÁM ẢNH",
            bg_color: Color::srgb(0.35, 0.15, 0.50),
            text_color: Color::WHITE,
            width: 40.0,
        }
    } else if lower_id.contains("syn_divine") || lower_name.contains("thần ân") {
        BadgeInfo {
            label: "THẦN ÂN",
            bg_color: Color::srgb(0.10, 0.68, 0.48),
            text_color: Color::WHITE,
            width: 42.0,
        }
    } else if lower_id.contains("weaken") || lower_name.contains("suy yếu") {
        BadgeInfo {
            label: "YẾU",
            bg_color: Color::srgb(0.55, 0.30, 0.65),
            text_color: Color::WHITE,
            width: 26.0,
        }
    } else if lower_id.contains("vulnerable") || lower_name.contains("sơ hở") {
        BadgeInfo {
            label: "HỞ",
            bg_color: Color::srgb(0.85, 0.45, 0.12),
            text_color: Color::WHITE,
            width: 26.0,
        }
    } else if lower_id.contains("heal") || lower_name.contains("hồi phục") {
        BadgeInfo {
            label: "HỒI",
            bg_color: Color::srgb(0.15, 0.80, 0.40),
            text_color: Color::WHITE,
            width: 26.0,
        }
    } else {
        BadgeInfo {
            label: "BUFF",
            bg_color: Color::srgb(0.4, 0.4, 0.5),
            text_color: Color::WHITE,
            width: 30.0,
        }
    }
}

#[derive(Component, Default, Clone, Debug)]
pub struct ActiveStatusEffects {
    pub effects: Vec<StatusEffectInstance>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatusEffectInstance {
    pub effect_id: String,
    pub icon: String,
    pub name: String,
    pub turns_remaining: i32,
}

impl ActiveStatusEffects {
    pub fn add(&mut self, effect_id: &str, icon: &str, name: &str, turns: i32) {
        if let Some(existing) = self.effects.iter_mut().find(|e| e.effect_id == effect_id) {
            existing.turns_remaining = existing.turns_remaining.max(turns);
        } else {
            self.effects.push(StatusEffectInstance {
                effect_id: effect_id.to_string(),
                icon: icon.to_string(),
                name: name.to_string(),
                turns_remaining: turns,
            });
        }
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, effect_id: &str) {
        self.effects.retain(|e| e.effect_id != effect_id);
    }

    #[allow(dead_code)]
    pub fn tick_turns(&mut self) {
        for eff in self.effects.iter_mut() {
            eff.turns_remaining = eff.turns_remaining.saturating_sub(1);
        }
        self.effects.retain(|e| e.turns_remaining > 0);
    }

    pub fn clear(&mut self) {
        self.effects.clear();
    }
}

#[derive(Component)]
pub struct UnitVisualRoot;

#[derive(Component)]
pub struct UnitSelectionRing;

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

#[derive(Clone, Copy, Debug)]
pub struct UnitSpawnParams {
    pub unit_class: UnitClass,
    pub faction: Faction,
    pub col: usize,
    pub row: usize,
    pub star_level: u8,
    pub is_boss: bool,
    pub hp_bonus: f32,
    pub atk_bonus: f32,
    pub initiative_bonus: f32,
}

impl UnitSpawnParams {
    pub fn new(unit_class: UnitClass, faction: Faction, col: usize, row: usize) -> Self {
        Self {
            unit_class,
            faction,
            col,
            row,
            star_level: 1,
            is_boss: false,
            hp_bonus: 0.0,
            atk_bonus: 0.0,
            initiative_bonus: 0.0,
        }
    }

    pub fn with_star(mut self, star_level: u8) -> Self {
        self.star_level = star_level;
        self
    }

    pub fn with_boss(mut self, is_boss: bool) -> Self {
        self.is_boss = is_boss;
        self
    }

    pub fn with_bonuses(mut self, hp_bonus: f32, atk_bonus: f32, initiative_bonus: f32) -> Self {
        self.hp_bonus = hp_bonus;
        self.atk_bonus = atk_bonus;
        self.initiative_bonus = initiative_bonus;
        self
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
    spawn_unit_from_params(commands, textures, UnitSpawnParams::new(unit_class, faction, col, row))
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
    spawn_unit_from_params(
        commands,
        textures,
        UnitSpawnParams::new(unit_class, faction, col, row)
            .with_star(star_level)
            .with_boss(is_boss),
    )
}

#[allow(dead_code)]
pub fn spawn_unit_ext_bonus(
    commands: &mut Commands,
    textures: &GameTextures,
    unit_class: UnitClass,
    faction: Faction,
    col: usize,
    row: usize,
    star_level: u8,
    is_boss: bool,
    hp_bonus: f32,
    atk_bonus: f32,
) -> Entity {
    spawn_unit_from_params(
        commands,
        textures,
        UnitSpawnParams::new(unit_class, faction, col, row)
            .with_star(star_level)
            .with_boss(is_boss)
            .with_bonuses(hp_bonus, atk_bonus, 0.0),
    )
}

pub fn spawn_unit_ext_bonus_with_initiative(
    commands: &mut Commands,
    textures: &GameTextures,
    unit_class: UnitClass,
    faction: Faction,
    col: usize,
    row: usize,
    star_level: u8,
    is_boss: bool,
    hp_bonus: f32,
    atk_bonus: f32,
    initiative_bonus: f32,
) -> Entity {
    spawn_unit_from_params(
        commands,
        textures,
        UnitSpawnParams::new(unit_class, faction, col, row)
            .with_star(star_level)
            .with_boss(is_boss)
            .with_bonuses(hp_bonus, atk_bonus, initiative_bonus),
    )
}

pub fn spawn_unit_from_params(
    commands: &mut Commands,
    textures: &GameTextures,
    params: UnitSpawnParams,
) -> Entity {
    let UnitSpawnParams {
        unit_class,
        faction,
        col,
        row,
        star_level,
        is_boss,
        hp_bonus,
        atk_bonus,
        initiative_bonus,
    } = params;

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
    stats.max_hp += hp_bonus;
    stats.hp = stats.max_hp;
    stats.atk += atk_bonus;
    stats.speed += initiative_bonus;

    let token_scale = if is_boss {
        1.55
    } else if star_level == 3 {
        1.18
    } else if star_level == 2 {
        1.08
    } else {
        1.0
    };

    let mut entity_cmds = commands.spawn((
        Unit {
            class: unit_class,
            faction,
        },
        crate::economy::StarLevel(star_level),
        stats,
        ActiveStatusEffects::default(),
        GridPos { col, row, faction },
        crate::battle::ActionGauge { current: 0.0 },
        ChibiSquashStretch::default(),
        Transform::from_xyz(world_pos.x, world_pos.y, z_depth).with_scale(Vec3::splat(token_scale)),
        Visibility::default(),
    ));

    if is_boss {
        entity_cmds.insert(BossUnit);
    }

    entity_cmds.with_children(|parent| {
        parent.spawn((
            UnitSelectionRing,
            Sprite {
                custom_size: Some(Vec2::splat(74.0)),
                color: Color::srgba(1.0, 0.85, 0.25, 0.0),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -0.2),
        ));

        // Soft drop shadow
        parent.spawn((
            Sprite {
                custom_size: Some(Vec2::new(76.0, 22.0)),
                color: Color::srgba(0.02, 0.03, 0.05, 0.55),
                ..default()
            },
            Transform::from_xyz(0.0, -32.0, -0.5),
        ));

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
                // Outer Card Border
                vis_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::splat(68.0)),
                        color: outer_border_col,
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.0),
                ));

                // Inner Bevel
                vis_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::splat(64.0)),
                        color: inner_border_col,
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));

                // Dark Portrait Plate
                vis_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::splat(58.0)),
                        color: Color::srgb(0.08, 0.10, 0.15),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.2),
                ));

                // Hero Portrait Texture Plate
                vis_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::splat(44.0)),
                        color: Color::srgba(0.04, 0.06, 0.10, 0.90),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 2.0, 0.25),
                ));

                vis_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::splat(40.0)),
                        color: unit_class.color(),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 2.0, 0.28),
                ));

                // Real unit card texture
                vis_parent.spawn((
                    Sprite {
                        image: textures.get_unit_texture(unit_class),
                        custom_size: Some(Vec2::splat(38.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 2.0, 0.32),
                ));

                vis_parent.spawn((
                    Text2d::new(unit_class.name()),
                    TextFont {
                        font: textures.font_bold.clone(),
                        font_size: 8.5,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, -18.0, 0.38),
                ));

                // Top-Left Class Role Badge
                vis_parent
                    .spawn((
                        Sprite {
                            custom_size: Some(Vec2::splat(20.0)),
                            color: Color::srgba(0.06, 0.08, 0.14, 0.95),
                            ..default()
                        },
                        Transform::from_xyz(-23.0, 23.0, 0.4),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Sprite {
                                custom_size: Some(Vec2::splat(18.0)),
                                color: unit_class.color(),
                                ..default()
                            },
                            Transform::from_xyz(0.0, 0.0, 0.05),
                        ));
                        badge.spawn((
                            Sprite {
                                custom_size: Some(Vec2::splat(15.0)),
                                color: Color::srgb(0.08, 0.10, 0.16),
                                ..default()
                            },
                            Transform::from_xyz(0.0, 0.0, 0.1),
                        ));
                        badge.spawn((
                            Text2d::new(unit_class.role_abbr()),
                            TextFont {
                                font: textures.font_bold.clone(),
                                font_size: 7.5,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Transform::from_xyz(0.0, 0.0, 0.2),
                        ));
                    });

                // Top-Right Star Rating / Boss Badge
                if is_boss {
                    vis_parent
                        .spawn((
                            Sprite {
                                custom_size: Some(Vec2::new(34.0, 15.0)),
                                color: Color::srgba(0.5, 0.05, 0.1, 0.92),
                                ..default()
                            },
                            Transform::from_xyz(18.0, 23.0, 0.4),
                        ))
                        .with_child((
                            Text2d::new("👑 BOSS"),
                            TextFont {
                                font: textures.font_bold.clone(),
                                font_size: 8.5,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.3, 0.3)),
                            Transform::from_xyz(0.0, 0.0, 0.1),
                        ));
                } else if star_level > 1 {
                    vis_parent
                        .spawn((
                            Sprite {
                                custom_size: Some(Vec2::new(
                                    if star_level >= 3 { 32.0 } else { 24.0 },
                                    15.0,
                                )),
                                color: Color::srgba(0.05, 0.07, 0.12, 0.92),
                                ..default()
                            },
                            Transform::from_xyz(20.0, 23.0, 0.4),
                        ))
                        .with_children(|star_badge| {
                            star_badge.spawn((
                                Text2d::new(crate::economy::StarLevel(star_level).badge()),
                                TextFont {
                                    font: textures.font_bold.clone(),
                                    font_size: 9.5,
                                    ..default()
                                },
                                TextColor(crate::economy::StarLevel(star_level).color()),
                                Transform::from_xyz(0.0, 0.0, 0.1),
                            ));
                        });
                }
            });

        // Bottom Name Plate Pill
        parent
            .spawn((
                Sprite {
                    custom_size: Some(Vec2::new(64.0, 14.0)),
                    color: Color::srgba(0.05, 0.07, 0.10, 0.88),
                    ..default()
                },
                Transform::from_xyz(0.0, -31.0, 0.5),
            ))
            .with_child((
                Text2d::new(if is_boss {
                    "TITAN".to_string()
                } else if star_level > 1 {
                    format!(
                        "{} {}",
                        unit_class.name(),
                        crate::economy::StarLevel(star_level).badge()
                    )
                } else {
                    unit_class.name().to_string()
                }),
                TextFont {
                    font: textures.font_bold.clone(),
                    font_size: if is_boss { 9.5 } else { 9.0 },
                    ..default()
                },
                TextColor(if is_boss {
                    Color::srgb(1.0, 0.25, 0.25)
                } else if star_level > 1 {
                    crate::economy::StarLevel(star_level).color()
                } else {
                    unit_class.color()
                }),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));

        // Overhead Status Effects Container (Pure Icon Sprites)
        parent
            .spawn((
                EffectIconsRoot2d,
                Transform::from_xyz(0.0, 59.0, 1.2),
                Visibility::default(),
            ))
            .with_children(|fx_parent| {
                for idx in 0..3 {
                    fx_parent.spawn((
                        EffectBadgeSlot { index: idx },
                        Sprite {
                            image: textures.fx_shield.clone(),
                            custom_size: Some(Vec2::splat(16.0)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.1),
                        Visibility::Hidden,
                    ));
                }
            });

        // Overhead Dual Status Bars (HP + Mana)
        parent
            .spawn((
                HealthBarRoot2d,
                Transform::from_xyz(0.0, 46.0, 1.0),
                Visibility::default(),
            ))
            .with_children(|bar_parent| {
                // Background Frame
                bar_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(66.0, 12.0)),
                        color: Color::srgba(0.04, 0.05, 0.08, 0.95),
                        ..default()
                    },
                    Transform::from_xyz(0.0, -2.5, 0.0),
                ));

                // HP Bar BG
                bar_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(64.0, 6.0)),
                        color: Color::srgb(0.06, 0.08, 0.12),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.05),
                ));

                // HP Bar Fill
                bar_parent.spawn((
                    HealthBarFill2d,
                    Sprite {
                        custom_size: Some(Vec2::new(62.0, 5.0)),
                        color: Color::srgb(0.2, 0.85, 0.35),
                        anchor: Anchor::CenterLeft,
                        ..default()
                    },
                    Transform::from_xyz(-31.0, 0.0, 0.1),
                ));

                // Mana Bar BG
                bar_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(64.0, 4.0)),
                        color: Color::srgb(0.04, 0.05, 0.09),
                        ..default()
                    },
                    Transform::from_xyz(0.0, -5.5, 0.05),
                ));

                // Mana Bar Fill
                bar_parent.spawn((
                    ManaBarFill2d,
                    Sprite {
                        custom_size: Some(Vec2::new(62.0, 3.2)),
                        color: Color::srgb(0.22, 0.55, 1.0),
                        anchor: Anchor::CenterLeft,
                        ..default()
                    },
                    Transform::from_xyz(-31.0, -5.5, 0.1),
                ));
            });
    });

    entity_cmds.id()
}

pub fn spawn_bench_unit(
    commands: &mut Commands,
    textures: &GameTextures,
    unit_class: UnitClass,
    slot: usize,
    star_level: u8,
) -> Entity {
    spawn_bench_unit_bonus(commands, textures, unit_class, slot, star_level, 0.0, 0.0)
}

pub fn spawn_bench_unit_bonus(
    commands: &mut Commands,
    textures: &GameTextures,
    unit_class: UnitClass,
    slot: usize,
    star_level: u8,
    hp_bonus: f32,
    atk_bonus: f32,
) -> Entity {
    let world_pos = bench_world_pos(slot);
    let z_depth = 10.0;

    let outer_border_col = Color::srgb(0.25, 0.65, 1.0);
    let inner_border_col = Color::srgb(0.10, 0.25, 0.55);

    let mut stats = unit_class.base_stats();
    if star_level > 1 {
        let (hp_mult, atk_mult) = match star_level {
            2 => (1.8, 1.6),
            _ => (2.8, 2.5),
        };
        stats.max_hp = (stats.max_hp * hp_mult).round();
        stats.hp = stats.max_hp;
        stats.atk = (stats.atk * atk_mult).round();
    }
    stats.max_hp += hp_bonus;
    stats.hp = stats.max_hp;
    stats.atk += atk_bonus;

    let token_scale = if star_level == 3 {
        1.18
    } else if star_level == 2 {
        1.08
    } else {
        1.0
    };

    let mut entity_cmds = commands.spawn((
        Unit {
            class: unit_class,
            faction: Faction::Player,
        },
        crate::economy::StarLevel(star_level),
        stats,
        ActiveStatusEffects::default(),
        BenchPos { slot },
        ChibiSquashStretch::default(),
        Transform::from_xyz(world_pos.x, world_pos.y, z_depth).with_scale(Vec3::splat(token_scale)),
        Visibility::default(),
    ));

    entity_cmds.with_children(|parent| {
        parent.spawn((
            UnitSelectionRing,
            Sprite {
                custom_size: Some(Vec2::splat(74.0)),
                color: Color::srgba(1.0, 0.85, 0.25, 0.0),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -0.2),
        ));

        // Soft drop shadow
        parent.spawn((
            Sprite {
                custom_size: Some(Vec2::new(76.0, 22.0)),
                color: Color::srgba(0.02, 0.03, 0.05, 0.55),
                ..default()
            },
            Transform::from_xyz(0.0, -32.0, -0.5),
        ));

        parent
            .spawn((
                UnitVisualRoot,
                IdleBobbing {
                    base_y: 0.0,
                    phase: slot as f32 * 1.5,
                },
                ChibiSquashStretch::default(),
                Transform::from_xyz(0.0, 0.0, 0.0),
                Visibility::default(),
            ))
            .with_children(|vis_parent| {
                // Outer Card Border
                vis_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::splat(68.0)),
                        color: outer_border_col,
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.0),
                ));

                // Inner Bevel
                vis_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::splat(64.0)),
                        color: inner_border_col,
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));

                // Dark Portrait Plate
                vis_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::splat(58.0)),
                        color: Color::srgb(0.08, 0.10, 0.15),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.2),
                ));

                // Central Hero Icon Emblem
                vis_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::splat(44.0)),
                        color: Color::srgba(0.04, 0.06, 0.10, 0.90),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 2.0, 0.25),
                ));

                vis_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::splat(40.0)),
                        color: unit_class.color(),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 2.0, 0.28),
                ));

                // Real unit card texture
                vis_parent.spawn((
                    Sprite {
                        image: textures.get_unit_texture(unit_class),
                        custom_size: Some(Vec2::splat(38.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 2.0, 0.32),
                ));

                vis_parent.spawn((
                    Text2d::new(unit_class.name()),
                    TextFont {
                        font: textures.font_bold.clone(),
                        font_size: 8.5,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, -18.0, 0.38),
                ));

                // Top-Left Class Role Badge
                vis_parent
                    .spawn((
                        Sprite {
                            custom_size: Some(Vec2::splat(20.0)),
                            color: Color::srgba(0.06, 0.08, 0.14, 0.95),
                            ..default()
                        },
                        Transform::from_xyz(-23.0, 23.0, 0.4),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Sprite {
                                custom_size: Some(Vec2::splat(18.0)),
                                color: unit_class.color(),
                                ..default()
                            },
                            Transform::from_xyz(0.0, 0.0, 0.05),
                        ));
                        badge.spawn((
                            Sprite {
                                custom_size: Some(Vec2::splat(15.0)),
                                color: Color::srgb(0.08, 0.10, 0.16),
                                ..default()
                            },
                            Transform::from_xyz(0.0, 0.0, 0.1),
                        ));
                        badge.spawn((
                            Text2d::new(unit_class.role_abbr()),
                            TextFont {
                                font: textures.font_bold.clone(),
                                font_size: 7.5,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Transform::from_xyz(0.0, 0.0, 0.2),
                        ));
                    });

                // Top-Right Star Rating Badge
                if star_level > 1 {
                    vis_parent
                        .spawn((
                            Sprite {
                                custom_size: Some(Vec2::new(
                                    if star_level >= 3 { 32.0 } else { 24.0 },
                                    15.0,
                                )),
                                color: Color::srgba(0.05, 0.07, 0.12, 0.92),
                                ..default()
                            },
                            Transform::from_xyz(20.0, 23.0, 0.4),
                        ))
                        .with_children(|star_badge| {
                            star_badge.spawn((
                                Text2d::new(crate::economy::StarLevel(star_level).badge()),
                                TextFont {
                                    font: textures.font_bold.clone(),
                                    font_size: 9.5,
                                    ..default()
                                },
                                TextColor(crate::economy::StarLevel(star_level).color()),
                                Transform::from_xyz(0.0, 0.0, 0.1),
                            ));
                        });
                }
            });

        // Bottom Name Plate Pill
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
                Text2d::new(if star_level > 1 {
                    format!(
                        "{} {}",
                        unit_class.name(),
                        crate::economy::StarLevel(star_level).badge()
                    )
                } else {
                    unit_class.name().to_string()
                }),
                TextFont {
                    font: textures.font_bold.clone(),
                    font_size: 9.0,
                    ..default()
                },
                TextColor(if star_level > 1 {
                    crate::economy::StarLevel(star_level).color()
                } else {
                    unit_class.color()
                }),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));

        // Overhead Status Effects Container (Pure Icon Sprites)
        parent
            .spawn((
                EffectIconsRoot2d,
                Transform::from_xyz(0.0, 59.0, 1.2),
                Visibility::default(),
            ))
            .with_children(|fx_parent| {
                for idx in 0..3 {
                    fx_parent.spawn((
                        EffectBadgeSlot { index: idx },
                        Sprite {
                            image: textures.fx_shield.clone(),
                            custom_size: Some(Vec2::splat(16.0)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.1),
                        Visibility::Hidden,
                    ));
                }
            });

        // Overhead HP Bar
        parent
            .spawn((
                HealthBarRoot2d,
                Transform::from_xyz(0.0, 46.0, 1.0),
                Visibility::default(),
            ))
            .with_children(|bar_parent| {
                bar_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(64.0, 7.0)),
                        color: Color::srgb(0.06, 0.08, 0.12),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.0),
                ));

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
            });
    });

    entity_cmds.id()
}

pub fn update_selection_halo(
    time: Res<Time>,
    selected: Res<SelectedUnitState>,
    units: Query<(Entity, &Children), With<Unit>>,
    mut halos: Query<&mut Sprite, With<UnitSelectionRing>>,
) {
    let t = time.elapsed_secs();
    for (unit_entity, children) in units.iter() {
        let is_selected = selected.entity == Some(unit_entity);
        for &child in children.iter() {
            if let Ok(mut sprite) = halos.get_mut(child) {
                if is_selected {
                    let pulse = 0.70 + (t * 6.0).sin() * 0.30;
                    sprite.color = Color::srgba(1.0, 0.88, 0.20, pulse);
                } else {
                    sprite.color = Color::srgba(1.0, 0.85, 0.25, 0.0);
                }
            }
        }
    }
}

pub fn animate_idle_bobbing(
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    hit_stop: Res<HitStopManager>,
    mut bob_query: Query<(&mut Transform, &IdleBobbing), (With<UnitVisualRoot>, Without<DeadUnit>)>,
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
    units: Query<(&UnitStats, &Children), (With<Unit>, Without<DeadUnit>)>,
    mut hp_fill_query: Query<
        (&mut Transform, &mut Sprite),
        (With<HealthBarFill2d>, Without<ManaBarFill2d>),
    >,
    mut mana_fill_query: Query<
        (&mut Transform, &mut Sprite),
        (With<ManaBarFill2d>, Without<HealthBarFill2d>),
    >,
    roots: Query<&Children, With<HealthBarRoot2d>>,
) {
    for (stats, unit_children) in units.iter() {
        let hp_ratio = (stats.hp / stats.max_hp).clamp(0.0, 1.0);
        let mana_ratio = (stats.mana / stats.max_mana).clamp(0.0, 1.0);

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
                }
            }
        }
    }
}

pub fn update_unit_effect_icons(
    units: Query<(&UnitStats, &ActiveStatusEffects, &Children), (With<Unit>, Without<DeadUnit>)>,
    roots: Query<&Children, With<EffectIconsRoot2d>>,
    mut slots: Query<(&EffectBadgeSlot, &mut Transform, &mut Sprite, &mut Visibility)>,
    textures: Res<GameTextures>,
) {
    for (stats, active_fx, children) in units.iter() {
        let mut effect_ids: Vec<String> = Vec::new();

        // 1. Shield active
        if stats.shield > 0.0 {
            effect_ids.push("shield".to_string());
        }

        // 2. Active status effects
        for eff in &active_fx.effects {
            if effect_ids.len() >= 3 {
                break;
            }
            if !effect_ids.iter().any(|id| id == &eff.effect_id) {
                effect_ids.push(eff.effect_id.clone());
            }
        }

        let count = effect_ids.len();

        for child in children.iter() {
            if let Ok(slot_entities) = roots.get(*child) {
                for &slot_entity in slot_entities.iter() {
                    if let Ok((slot, mut transform, mut sprite, mut vis)) = slots.get_mut(slot_entity) {
                        if slot.index < count {
                            let eff_id = &effect_ids[slot.index];

                            let offset_x = match count {
                                1 => 0.0,
                                2 => if slot.index == 0 { -9.0 } else { 9.0 },
                                _ => match slot.index {
                                    0 => -18.0,
                                    1 => 0.0,
                                    _ => 18.0,
                                },
                            };

                            transform.translation.x = offset_x;
                            sprite.custom_size = Some(Vec2::splat(16.0));
                            sprite.color = Color::WHITE;
                            sprite.image = textures.get_effect_texture(eff_id);
                            *vis = Visibility::Inherited;
                        } else {
                            *vis = Visibility::Hidden;
                        }
                    }
                }
            }
        }
    }
}
