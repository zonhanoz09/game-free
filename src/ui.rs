use crate::battle::ActionGauge;
use crate::audio::{PlaySoundEvent, SoundEffect};
use crate::board::{bench_world_pos, grid_to_world_pos, HoveredTile};
use crate::economy::{refund_amount, unit_cost, GoldDisplayText, PlayerEconomy, ShopLockToggle, ShopRerollButton, StarLevel};
use crate::stages::get_stage_def;
use crate::synergies::{SynergyContainer, SynergyCountText, SynergyRow, SynergyType};
use crate::types::*;
use crate::units::{spawn_bench_unit, spawn_unit, spawn_unit_ext, Unit};
use bevy::prelude::*;

#[derive(Component)]
pub struct StartBattleButton;

#[derive(Component)]
pub struct StartBattleText;

#[derive(Component)]
pub struct ClearBoardButton;

#[derive(Component)]
pub struct PresetButton;

#[derive(Component)]
pub struct SpeedToggleButton;

#[derive(Component)]
pub struct NextStageButton;

#[derive(Component)]
pub struct RetryButton;

#[derive(Component)]
pub struct ShopCard(pub usize);

#[derive(Component)]
pub struct ShopCardAvatar(pub usize);

#[derive(Component)]
pub struct ShopCardName(pub usize);

#[derive(Component)]
pub struct ShopCardCost(pub usize);

#[derive(Component)]
pub struct ShopLockText;

#[derive(Component)]
pub struct PlacementUiRoot;

#[derive(Component)]
pub struct ResultUiRoot;

#[derive(Component)]
pub struct StageTitleText;

#[derive(Component)]
pub struct StageDescText;

#[derive(Component)]
pub struct UnitCountText;

#[derive(Component)]
pub struct SpeedText;

#[derive(Component)]
pub struct TooltipText;

// --- Components for Hero Inspection Card ---
#[derive(Component)]
pub struct InspectHeroAvatar;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InspectStatType {
    Hp,
    Mana,
    Atk,
    Def,
    Spd,
}

#[derive(Component)]
pub struct InspectStatBar(pub InspectStatType);

#[derive(Component)]
pub struct InspectStatText(pub InspectStatType);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InspectHeaderField {
    Name,
    Faction,
    Role,
}

#[derive(Component)]
pub struct InspectHeader(pub InspectHeaderField);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InspectSkillField {
    Name,
    Type,
    Desc,
    UltName,
    UltDesc,
}

#[derive(Component)]
pub struct InspectSkill(pub InspectSkillField);

#[derive(Component)]
pub struct InspectorRoot;

pub fn setup_ui(mut commands: Commands, textures: Res<GameTextures>, fonts: Res<GameFonts>) {
    // 1. Top Bar UI
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            height: Val::Px(64.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(32.0)),
            ..default()
        },))
        .with_children(|parent| {
            // Left: Game Title & Stage
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new("3v3 TACTICAL ARENA - 2D AUTO-BATTLER"),
                        TextFont {
                            font_size: 19.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.96, 1.0)),
                    ));
                    col.spawn((
                        Text::new("Stage 1: Vanguard Frontline"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.98, 0.82, 0.3)),
                        StageTitleText,
                    ));
                    col.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 11.5,
                            ..default()
                        },
                        TextColor(Color::srgba(0.85, 0.85, 0.9, 0.8)),
                        StageDescText,
                    ));
                });

            // Center: Unit counter & Guide
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new("Units: 0 / 5"),
                        TextFont {
                            font_size: 16.5,
                            ..default()
                        },
                        TextColor(Color::srgb(0.35, 0.95, 0.55)),
                        UnitCountText,
                    ));
                    col.spawn((
                        Text::new("🪙 15G (+6G next)"),
                        TextFont {
                            font_size: 14.5,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.2)),
                        GoldDisplayText,
                    ));
                    col.spawn((
                        Text::new("L-Click: Place/Inspect  |  R-Click: Sell/Refund"),
                        TextFont {
                            font_size: 11.5,
                            ..default()
                        },
                        TextColor(Color::srgba(0.85, 0.88, 0.92, 0.78)),
                    ));
                });

            // Right: Battle Speed Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(115.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
                    BackgroundColor(Color::srgb(0.18, 0.22, 0.32)),
                    BorderRadius::all(Val::Px(6.0)),
                    SpeedToggleButton,
                ))
                .with_child((
                    Text::new("Speed: 1x"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    SpeedText,
                ));
        });

    // 1.5 Team Synergies Panel (Left Edge)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(82.0),
                left: Val::Px(16.0),
                width: Val::Px(175.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(7.0),
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BorderColor(Color::srgba(0.35, 0.55, 0.85, 0.5)),
            BackgroundColor(Color::srgba(0.07, 0.09, 0.14, 0.92)),
            BorderRadius::all(Val::Px(10.0)),
            SynergyContainer,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("TEAM SYNERGIES"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.3)),
            ));

            let syns = [
                SynergyType::Vanguard,
                SynergyType::Sharpshooter,
                SynergyType::Arcanist,
                SynergyType::Shadow,
                SynergyType::Divine,
            ];

            for syn in syns {
                panel
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.1)),
                        BackgroundColor(Color::srgba(0.12, 0.15, 0.22, 0.6)),
                        BorderRadius::all(Val::Px(5.0)),
                        SynergyRow(syn),
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Text::new(format!("{} {}", syn.icon(), syn.name().split(" ").next().unwrap())),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        row.spawn((
                            Text::new(format!("0/{}", syn.threshold())),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.8, 0.8, 0.8, 0.7)),
                            SynergyCountText(syn),
                        ));
                    });
            }
        });

    // 2. Modern Hero Inspection Card (Right Panel)
    commands
        .spawn((
            InspectorRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(82.0),
                right: Val::Px(16.0),
                width: Val::Px(285.0),
                display: Display::None, // Hidden by default, shows only on hover/select!
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(14.0)),
                row_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BorderColor(Color::srgba(0.35, 0.55, 0.85, 0.5)),
            BackgroundColor(Color::srgba(0.07, 0.09, 0.14, 0.92)),
            BorderRadius::all(Val::Px(10.0)),
        ))
        .with_children(|panel| {
            // Header Row: Avatar + Name + Faction Badge
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        ImageNode {
                            image: textures.knight.clone(),
                            ..default()
                        },
                        Node {
                            width: Val::Px(46.0),
                            height: Val::Px(46.0),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(6.0)),
                        InspectHeroAvatar,
                    ));

                    header
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.0),
                            ..default()
                        })
                        .with_children(|info| {
                            info.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|name_row| {
                                name_row.spawn((
                                    Text::new("Knight"),
                                    TextFont {
                                        font_size: 16.5,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    InspectHeader(InspectHeaderField::Name),
                                ));
                                name_row.spawn((
                                    Text::new("ALLY"),
                                    TextFont {
                                        font_size: 10.5,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.3, 0.7, 1.0)),
                                    InspectHeader(InspectHeaderField::Faction),
                                ));
                            });

                            info.spawn((
                                Text::new("Frontline Iron Vanguard (Tank)"),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.82, 0.4)),
                                InspectHeader(InspectHeaderField::Role),
                            ));
                        });
                });

            // Thin Divider
            panel.spawn((
                Node {
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
            ));

            // Section: Visual Stat Bars
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|stats_sec| {
                    // HP Bar
                    stats_sec.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    }).with_children(|row| {
                        row.spawn((
                            Text::new("HP (Health)"),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(Color::srgb(0.4, 0.9, 0.5)),
                        ));
                        row.spawn((
                            Text::new("180 / 180"),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(Color::WHITE),
                            InspectStatText(InspectStatType::Hp),
                        ));
                    });
                    stats_sec.spawn((
                        Node {
                            height: Val::Px(7.0),
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.16, 0.20)),
                        BorderRadius::all(Val::Px(3.0)),
                    )).with_child((
                        Node {
                            height: Val::Percent(100.0),
                            width: Val::Percent(90.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.25, 0.85, 0.45)),
                        BorderRadius::all(Val::Px(3.0)),
                        InspectStatBar(InspectStatType::Hp),
                    ));

                    // MP (Mana) Bar
                    stats_sec.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    }).with_children(|row| {
                        row.spawn((
                            Text::new("MP (Mana)"),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(Color::srgb(0.3, 0.75, 1.0)),
                        ));
                        row.spawn((
                            Text::new("0 / 100"),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(Color::WHITE),
                            InspectStatText(InspectStatType::Mana),
                        ));
                    });
                    stats_sec.spawn((
                        Node {
                            height: Val::Px(7.0),
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.16, 0.20)),
                        BorderRadius::all(Val::Px(3.0)),
                    )).with_child((
                        Node {
                            height: Val::Percent(100.0),
                            width: Val::Percent(0.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.25, 0.65, 1.0)),
                        BorderRadius::all(Val::Px(3.0)),
                        InspectStatBar(InspectStatType::Mana),
                    ));

                    // ATK Bar
                    stats_sec.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    }).with_children(|row| {
                        row.spawn((
                            Text::new("ATK (Power)"),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(Color::srgb(1.0, 0.45, 0.3)),
                        ));
                        row.spawn((
                            Text::new("25"),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(Color::WHITE),
                            InspectStatText(InspectStatType::Atk),
                        ));
                    });
                    stats_sec.spawn((
                        Node {
                            height: Val::Px(7.0),
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.16, 0.20)),
                        BorderRadius::all(Val::Px(3.0)),
                    )).with_child((
                        Node {
                            height: Val::Percent(100.0),
                            width: Val::Percent(50.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(1.0, 0.4, 0.25)),
                        BorderRadius::all(Val::Px(3.0)),
                        InspectStatBar(InspectStatType::Atk),
                    ));

                    // DEF Bar
                    stats_sec.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    }).with_children(|row| {
                        row.spawn((
                            Text::new("DEF (Armor)"),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(Color::srgb(0.4, 0.65, 1.0)),
                        ));
                        row.spawn((
                            Text::new("40"),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(Color::WHITE),
                            InspectStatText(InspectStatType::Def),
                        ));
                    });
                    stats_sec.spawn((
                        Node {
                            height: Val::Px(7.0),
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.16, 0.20)),
                        BorderRadius::all(Val::Px(3.0)),
                    )).with_child((
                        Node {
                            height: Val::Percent(100.0),
                            width: Val::Percent(80.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.35, 0.65, 1.0)),
                        BorderRadius::all(Val::Px(3.0)),
                        InspectStatBar(InspectStatType::Def),
                    ));

                    // SPD Bar
                    stats_sec.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    }).with_children(|row| {
                        row.spawn((
                            Text::new("SPD (Agility)"),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(Color::srgb(0.95, 0.85, 0.3)),
                        ));
                        row.spawn((
                            Text::new("18"),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(Color::WHITE),
                            InspectStatText(InspectStatType::Spd),
                        ));
                    });
                    stats_sec.spawn((
                        Node {
                            height: Val::Px(7.0),
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.16, 0.20)),
                        BorderRadius::all(Val::Px(3.0)),
                    )).with_child((
                        Node {
                            height: Val::Percent(100.0),
                            width: Val::Percent(45.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.95, 0.85, 0.3)),
                        BorderRadius::all(Val::Px(3.0)),
                        InspectStatBar(InspectStatType::Spd),
                    ));
                });

            // Thin Divider
            panel.spawn((
                Node {
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
            ));

            // Section: Skill Breakdown Box
            panel
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(Color::srgba(0.8, 0.7, 0.2, 0.3)),
                    BackgroundColor(Color::srgba(0.12, 0.15, 0.22, 0.8)),
                    BorderRadius::all(Val::Px(6.0)),
                ))
                .with_children(|skill_box| {
                    skill_box.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    }).with_children(|title_row| {
                        title_row.spawn((
                            Text::new("Iron Bulwark & Cleave"),
                            TextFont {
                                font_size: 12.5,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.9, 0.3)),
                            InspectSkill(InspectSkillField::Name),
                        ));
                        title_row.spawn((
                            Text::new("[Melee]"),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.8, 1.0)),
                            InspectSkill(InspectSkillField::Type),
                        ));
                    });

                    skill_box.spawn((
                        Text::new("Leaps forward with heavy shield bash, slashing with luminous steel blade. Mitigates high damage through fortified defense."),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.9, 0.92, 0.95, 0.85)),
                        InspectSkill(InspectSkillField::Desc),
                    ));
                });

            // Section: Ultimate Skill Breakdown Box
            panel
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(Color::srgba(1.0, 0.85, 0.2, 0.6)),
                    BackgroundColor(Color::srgba(0.22, 0.18, 0.08, 0.85)),
                    BorderRadius::all(Val::Px(6.0)),
                ))
                .with_children(|ult_box| {
                    ult_box.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    }).with_children(|title_row| {
                        title_row.spawn((
                            Text::new("Aegis Fortress"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.88, 0.25)),
                            InspectSkill(InspectSkillField::UltName),
                        ));
                        title_row.spawn((
                            Text::new("[ULTIMATE]"),
                            TextFont {
                                font_size: 9.5,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.85, 0.2)),
                        ));
                    });

                    ult_box.spawn((
                        Text::new("Leaps into enemy frontline with massive bash, granting +80 shield and disrupting enemy action."),
                        TextFont {
                            font_size: 10.5,
                            ..default()
                        },
                        TextColor(Color::srgba(0.98, 0.95, 0.85, 0.9)),
                        InspectSkill(InspectSkillField::UltDesc),
                    ));
                });
        });

    // 3. Tooltip Banner above the bench
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(138.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(3.0),
            ..default()
        },))
        .with_children(|b| {
            b.spawn((
                Text::new(""),
                TextFont {
                    font: fonts.bold.clone(),
                    font_size: 13.5,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.95, 0.7)),
                TooltipText,
            ));
            b.spawn((
                Text::new("[1-4] Mua | [S] Bán | [D] Đổi (2G) | [E] Khóa | [Space] Chiến | [H] Hướng Dẫn"),
                TextFont {
                    font: fonts.regular.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.8, 0.9, 0.8)),
            ));
        });

    // 4. Bottom Bench & Placement Controls
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(15.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(115.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
            PlacementUiRoot,
        ))
        .with_children(|parent| {
            // 4 Shop Cards Row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    for i in 0..4 {
                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(175.0),
                                height: Val::Px(56.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Start,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                padding: UiRect::all(Val::Px(4.0)),
                                column_gap: Val::Px(8.0),
                                ..default()
                            },
                            BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                            BackgroundColor(Color::srgba(0.12, 0.16, 0.22, 0.95)),
                            BorderRadius::all(Val::Px(6.0)),
                            ShopCard(i),
                        ))
                        .with_children(|btn| {
                            // Hotkey badge [1], [2], [3], [4]
                            btn.spawn((
                                Text::new(format!("[{}]", i + 1)),
                                TextFont {
                                    font: fonts.bold.clone(),
                                    font_size: 9.5,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.5, 0.8, 1.0, 0.85)),
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: Val::Px(3.0),
                                    right: Val::Px(5.0),
                                    ..default()
                                },
                            ));

                            // Avatar thumbnail
                            btn.spawn((
                                ImageNode {
                                    image: textures.knight.clone(),
                                    ..default()
                                },
                                Node {
                                    width: Val::Px(44.0),
                                    height: Val::Px(44.0),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(4.0)),
                                ShopCardAvatar(i),
                            ));

                            // Info column
                            btn.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                ..default()
                            })
                            .with_children(|txt_col| {
                                txt_col.spawn((
                                    Text::new("Hero"),
                                    TextFont {
                                        font_size: 13.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    ShopCardName(i),
                                ));
                                txt_col.spawn((
                                    Text::new("2G"),
                                    TextFont {
                                        font_size: 10.5,
                                        ..default()
                                    },
                                    TextColor(Color::srgba(1.0, 0.85, 0.2, 0.9)),
                                    ShopCardCost(i),
                                ));
                            });
                        });
                    }
                });

            // Action Buttons Row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(14.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    // Reroll Shop Button (2G)
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BorderColor(Color::srgb(1.0, 0.8, 0.2)),
                        BackgroundColor(Color::srgb(0.28, 0.20, 0.12)),
                        BorderRadius::all(Val::Px(6.0)),
                        ShopRerollButton,
                    ))
                    .with_child((
                        Text::new("🎲 Roll 2G [D]"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.88, 0.3)),
                    ));

                    // Lock Shop Button
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(110.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                        BackgroundColor(Color::srgb(0.20, 0.23, 0.28)),
                        BorderRadius::all(Val::Px(6.0)),
                        ShopLockToggle,
                    ))
                    .with_child((
                        Text::new("🔓 Lock [E]"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        ShopLockText,
                    ));

                    // Preset Button
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.35, 0.5)),
                        BorderRadius::all(Val::Px(6.0)),
                        PresetButton,
                    ))
                    .with_child((
                        Text::new("Preset Squad"),
                        TextFont {
                            font_size: 12.5,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Start Battle Button
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(190.0),
                            height: Val::Px(38.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(Color::srgb(0.9, 0.8, 0.2)),
                        BackgroundColor(Color::srgb(0.15, 0.65, 0.3)),
                        BorderRadius::all(Val::Px(8.0)),
                        StartBattleButton,
                    ))
                    .with_child((
                        Text::new("⚔️ BATTLE START"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        StartBattleText,
                    ));

                    // Clear Button
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(105.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.5, 0.2, 0.25)),
                        BorderRadius::all(Val::Px(6.0)),
                        ClearBoardButton,
                    ))
                    .with_child((
                        Text::new("Clear Board"),
                        TextFont {
                            font_size: 12.5,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn update_unit_count_ui(
    units: Query<&Unit, Without<DeadUnit>>,
    mut text_query: Query<&mut Text, With<UnitCountText>>,
) {
    let player_units = units
        .iter()
        .filter(|u| u.faction == Faction::Player)
        .count();
    for mut text in text_query.iter_mut() {
        *text = Text::new(format!("Units: {} / {}", player_units, MAX_PLAYER_UNITS));
    }
}

pub fn update_hero_inspection_system(
    hovered: Res<HoveredTile>,
    selected: Res<SelectedUnitState>,
    textures: Res<GameTextures>,
    board_units: Query<(&Unit, &GridPos, &UnitStats), Without<DeadUnit>>,
    bench_units: Query<(&Unit, &BenchPos, &UnitStats), Without<DeadUnit>>,
    mut inspector_root: Query<&mut Node, (With<InspectorRoot>, Without<InspectStatBar>)>,
    mut avatar_query: Query<&mut ImageNode, With<InspectHeroAvatar>>,
    mut header_query: Query<(&InspectHeader, &mut Text, Option<&mut TextColor>)>,
    mut bar_query: Query<(&InspectStatBar, &mut Node), Without<InspectorRoot>>,
    mut text_query: Query<(&InspectStatText, &mut Text), (Without<InspectHeader>, Without<InspectorRoot>)>,
    mut skill_query: Query<
        (&InspectSkill, &mut Text),
        (Without<InspectHeader>, Without<InspectStatText>, Without<InspectorRoot>),
    >,
) {
    let mut inspected: Option<(UnitClass, Faction, UnitStats)> = None;

    if let Some(tile) = &hovered.tile {
        if let Some((unit, _, stats)) = board_units
            .iter()
            .find(|(_, g, _)| g.col == tile.col && g.row == tile.row && g.faction == tile.faction)
        {
            inspected = Some((unit.class, unit.faction, *stats));
        }
    }

    if inspected.is_none() {
        if let Some(slot) = hovered.bench_slot {
            if let Some((unit, _, stats)) = bench_units
                .iter()
                .find(|(_, b, _)| b.slot == slot)
            {
                inspected = Some((unit.class, Faction::Player, *stats));
            }
        }
    }

    if inspected.is_none() {
        if let Some(sel_ent) = selected.entity {
            if let Ok((unit, _, stats)) = board_units.get(sel_ent) {
                inspected = Some((unit.class, unit.faction, *stats));
            } else if let Ok((unit, _, stats)) = bench_units.get(sel_ent) {
                inspected = Some((unit.class, Faction::Player, *stats));
            }
        }
    }

    let Ok(mut root_node) = inspector_root.get_single_mut() else {
        return;
    };

    let Some((class, faction, stats)) = inspected else {
        root_node.display = Display::None;
        return;
    };

    root_node.display = Display::Flex;

    if let Ok(mut img) = avatar_query.get_single_mut() {
        img.image = textures.get_unit_texture(class);
    }

    for (header, mut txt, text_col) in header_query.iter_mut() {
        match header.0 {
            InspectHeaderField::Name => {
                *txt = Text::new(class.name());
            }
            InspectHeaderField::Faction => {
                if let Some(mut col) = text_col {
                    match faction {
                        Faction::Player => {
                            *txt = Text::new("ALLY");
                            col.0 = Color::srgb(0.3, 0.7, 1.0);
                        }
                        Faction::Enemy => {
                            *txt = Text::new("ENEMY");
                            col.0 = Color::srgb(1.0, 0.3, 0.3);
                        }
                    }
                }
            }
            InspectHeaderField::Role => {
                *txt = Text::new(class.role_title());
            }
        }
    }

    for (stat_bar, mut node) in bar_query.iter_mut() {
        let ratio = match stat_bar.0 {
            InspectStatType::Hp => (stats.hp / stats.max_hp).clamp(0.0, 1.0),
            InspectStatType::Mana => (stats.mana / stats.max_mana).clamp(0.0, 1.0),
            InspectStatType::Atk => (stats.atk / 55.0).clamp(0.0, 1.0),
            InspectStatType::Def => (stats.def / 50.0).clamp(0.0, 1.0),
            InspectStatType::Spd => (stats.speed / 40.0).clamp(0.0, 1.0),
        };
        node.width = Val::Percent(ratio * 100.0);
    }

    for (stat_txt, mut txt) in text_query.iter_mut() {
        match stat_txt.0 {
            InspectStatType::Hp => {
                *txt = Text::new(format!("{:.0} / {:.0}", stats.hp, stats.max_hp));
            }
            InspectStatType::Mana => {
                *txt = Text::new(format!("{:.0} / {:.0}", stats.mana, stats.max_mana));
            }
            InspectStatType::Atk => {
                *txt = Text::new(format!("{:.0}", stats.atk));
            }
            InspectStatType::Def => {
                if stats.shield > 0.0 {
                    *txt = Text::new(format!("{:.0} (+{:.0} Shld)", stats.def, stats.shield));
                } else {
                    *txt = Text::new(format!("{:.0}", stats.def));
                }
            }
            InspectStatType::Spd => {
                *txt = Text::new(format!("{:.0}", stats.speed));
            }
        }
    }

    for (sk, mut txt) in skill_query.iter_mut() {
        match sk.0 {
            InspectSkillField::Name => {
                *txt = Text::new(class.skill_name());
            }
            InspectSkillField::Type => {
                *txt = Text::new(format!("[{}]", class.skill_type()));
            }
            InspectSkillField::Desc => {
                *txt = Text::new(class.skill_description());
            }
            InspectSkillField::UltName => {
                *txt = Text::new(class.ultimate_name());
            }
            InspectSkillField::UltDesc => {
                *txt = Text::new(class.ultimate_desc());
            }
        }
    }
}

pub fn update_shop_cards_ui(
    economy: Res<PlayerEconomy>,
    textures: Res<GameTextures>,
    mut card_buttons: Query<(&ShopCard, &mut BackgroundColor, &mut BorderColor)>,
    mut avatars: Query<(&ShopCardAvatar, &mut ImageNode, &mut Visibility)>,
    mut names: Query<(&ShopCardName, &mut Text, &mut TextColor)>,
    mut costs: Query<(&ShopCardCost, &mut Text), Without<ShopCardName>>,
    mut lock_button: Query<(&mut BackgroundColor, &mut BorderColor), (With<ShopLockToggle>, Without<ShopCard>)>,
    mut lock_text: Query<&mut Text, (With<ShopLockText>, Without<ShopCardName>, Without<ShopCardCost>)>,
) {
    for (card, mut bg, mut border) in card_buttons.iter_mut() {
        let idx = card.0;
        if let Some(class) = economy.shop_slots[idx] {
            let col = class.color();
            let c_rgba = col.to_srgba();
            *border = BorderColor(Color::srgba(c_rgba.red, c_rgba.green, c_rgba.blue, 0.75));
            *bg = BackgroundColor(Color::srgba(
                c_rgba.red * 0.28,
                c_rgba.green * 0.28,
                c_rgba.blue * 0.28,
                0.95,
            ));
        } else {
            *border = BorderColor(Color::srgba(0.25, 0.28, 0.35, 0.35));
            *bg = BackgroundColor(Color::srgba(0.06, 0.08, 0.11, 0.75));
        }
    }

    for (avatar, mut img, mut vis) in avatars.iter_mut() {
        let idx = avatar.0;
        if let Some(class) = economy.shop_slots[idx] {
            img.image = textures.get_unit_texture(class);
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    for (name_comp, mut txt, mut col) in names.iter_mut() {
        let idx = name_comp.0;
        if let Some(class) = economy.shop_slots[idx] {
            *txt = Text::new(class.name());
            col.0 = Color::WHITE;
        } else {
            *txt = Text::new("[PURCHASED]");
            col.0 = Color::srgb(0.45, 0.48, 0.52);
        }
    }

    for (cost_comp, mut txt) in costs.iter_mut() {
        let idx = cost_comp.0;
        if let Some(class) = economy.shop_slots[idx] {
            *txt = Text::new(format!("🪙 {}G | {}", unit_cost(class), class.role_title()));
        } else {
            *txt = Text::new("--");
        }
    }

    if let Ok((mut bg, mut border)) = lock_button.get_single_mut() {
        if economy.shop_locked {
            *bg = BackgroundColor(Color::srgb(0.45, 0.35, 0.10));
            *border = BorderColor(Color::srgb(1.0, 0.85, 0.25));
        } else {
            *bg = BackgroundColor(Color::srgb(0.20, 0.23, 0.28));
            *border = BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.15));
        }
    }

    if let Ok(mut txt) = lock_text.get_single_mut() {
        if economy.shop_locked {
            *txt = Text::new("🔒 Locked [E]");
        } else {
            *txt = Text::new("🔓 Lock [E]");
        }
    }
}

pub fn handle_shop_clicks(
    mut commands: Commands,
    textures: Res<GameTextures>,
    mut economy: ResMut<PlayerEconomy>,
    bench_units: Query<&BenchPos, (With<Unit>, Without<DeadUnit>)>,
    mut buttons: Query<(&Interaction, &ShopCard), (Changed<Interaction>, With<Button>)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut tooltip: Query<&mut Text, With<TooltipText>>,
) {
    for (interaction, shop_card) in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let slot_idx = shop_card.0;
            if let Some(class) = economy.shop_slots[slot_idx] {
                let cost = unit_cost(class);
                if economy.gold < cost {
                    info!("[SHOP] Cannot buy {}: Not enough gold (Have: {}G, Need: {}G)", class.name(), economy.gold, cost);
                    if let Ok(mut txt) = tooltip.get_single_mut() {
                        *txt = Text::new(format!("⚠️ Not enough gold! Need {}G, have {}G.", cost, economy.gold));
                    }
                    continue;
                }

                let occupied_slots: Vec<usize> = bench_units.iter().map(|b| b.slot).collect();
                let free_slot = (0..BENCH_SLOTS).find(|s| !occupied_slots.contains(s));

                if let Some(slot) = free_slot {
                    if let Some(bought_class) = economy.buy_slot(slot_idx) {
                        spawn_bench_unit(
                            &mut commands,
                            &textures,
                            bought_class,
                            slot,
                            1,
                        );
                        sound_events.send(PlaySoundEvent(SoundEffect::Click));
                        info!("[SHOP] Recruited {:?} for {}G -> placed on Reserve Bench Slot #{} (Remaining Gold: {}G)", bought_class, cost, slot + 1, economy.gold);
                        if let Ok(mut txt) = tooltip.get_single_mut() {
                            *txt = Text::new(format!("Recruited {} for {}G (Placed on Bench #{})", bought_class.name(), cost, slot + 1));
                        }
                    }
                } else {
                    info!("[SHOP] Reserve bench is full (6/6 slots occupied)!");
                    if let Ok(mut txt) = tooltip.get_single_mut() {
                        *txt = Text::new("⚠️ Reserve Bench is full (6/6)! Deploy or sell a hero first.".to_string());
                    }
                }
            }
        }
    }
}

pub fn handle_speed_toggle(
    mut speed: ResMut<BattleSpeed>,
    mut buttons: Query<&Interaction, (Changed<Interaction>, With<SpeedToggleButton>)>,
    mut text_query: Query<&mut Text, With<SpeedText>>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    for interaction in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
            if speed.multiplier == 1.0 {
                speed.multiplier = 2.0;
            } else {
                speed.multiplier = 1.0;
            }
            for mut text in text_query.iter_mut() {
                *text = Text::new(format!("Speed: {:.0}x", speed.multiplier));
            }
            info!("[UI] Battle speed toggled: {:.0}x", speed.multiplier);
        }
    }
}

pub fn handle_start_battle_button(
    mut commands: Commands,
    textures: Res<GameTextures>,
    keyboard: Res<ButtonInput<KeyCode>>,
    units: Query<(&Unit, &UnitStats, &GridPos, &StarLevel), Without<DeadUnit>>,
    mut buttons: Query<(&Interaction, &mut BackgroundColor), With<StartBattleButton>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut pvp_mgr: ResMut<crate::net::PvpManager>,
) {
    if *current_state.get() != GameState::Placement {
        return;
    }

    let mut clicked = false;
    for (interaction, mut bg) in buttons.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgb(0.10, 0.45, 0.20));
                clicked = true;
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.22, 0.85, 0.40));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.15, 0.65, 0.30));
            }
        }
    }

    let space_pressed = keyboard.just_pressed(KeyCode::Space);

    if clicked || space_pressed {
        sound_events.send(PlaySoundEvent(SoundEffect::Click));
        if pvp_mgr.active {
            if pvp_mgr.is_ready {
                return;
            }
            let mut lineup = Vec::new();
            for (u, _, g, s) in units.iter() {
                if u.faction == Faction::Player {
                    lineup.push(crate::net::PvpUnitData {
                        col: g.col,
                        row: g.row,
                        class: u.class,
                        star_level: s.0,
                    });
                }
            }
            if lineup.is_empty() {
                spawn_unit(&mut commands, &textures, UnitClass::Knight, Faction::Player, 2, 0);
                spawn_unit(&mut commands, &textures, UnitClass::Archer, Faction::Player, 0, 1);
                spawn_unit(&mut commands, &textures, UnitClass::Assassin, Faction::Player, 1, 2);
                lineup.push(crate::net::PvpUnitData { col: 2, row: 0, class: UnitClass::Knight, star_level: 1 });
                lineup.push(crate::net::PvpUnitData { col: 0, row: 1, class: UnitClass::Archer, star_level: 1 });
                lineup.push(crate::net::PvpUnitData { col: 1, row: 2, class: UnitClass::Assassin, star_level: 1 });
            }
            pvp_mgr.is_ready = true;
            crate::net::send_pvp_message(&crate::net::PvpMessage::PlayerReady { lineup: lineup.clone() });
            info!("[PVP] Ready & Locked In! Sent lineup of {} heroes.", lineup.len());
            return;
        }

        let player_count = units
            .iter()
            .filter(|(u, _, _, _)| u.faction == Faction::Player)
            .count();
        if player_count > 0 {
            info!("[UI] ⚔️ Battle Start triggered! (Active player heroes on board: {})", player_count);
            next_state.set(GameState::Battle);
        } else {
            info!("[UI] ⚔️ Battle Start triggered with 0 units on board: Auto-deployed starter squad (Knight, Archer, Assassin)!");
            spawn_unit(&mut commands, &textures, UnitClass::Knight, Faction::Player, 2, 0);
            spawn_unit(&mut commands, &textures, UnitClass::Archer, Faction::Player, 0, 1);
            spawn_unit(&mut commands, &textures, UnitClass::Assassin, Faction::Player, 2, 2);
            next_state.set(GameState::Battle);
        }
    }
}

pub fn update_start_button_text(
    pvp_mgr: Res<crate::net::PvpManager>,
    mut text_query: Query<&mut Text, With<StartBattleText>>,
) {
    if !pvp_mgr.is_changed() {
        return;
    }
    for mut text in text_query.iter_mut() {
        if pvp_mgr.active {
            if pvp_mgr.is_ready {
                *text = Text::new("⏳ ĐÃ KHÓA (CHỜ ĐỐI THỦ)");
            } else if pvp_mgr.opponent_ready {
                *text = Text::new("⚡ ĐỐI THỦ ĐÃ SẴN SÀNG!");
            } else {
                *text = Text::new("⚔️ KHÓA TRẬN & SẴN SÀNG");
            }
        } else {
            *text = Text::new("⚔️ BATTLE START");
        }
    }
}

pub fn handle_clear_button(
    mut commands: Commands,
    mut buttons: Query<&Interaction, (Changed<Interaction>, With<ClearBoardButton>)>,
    board_units: Query<(Entity, &Unit, &StarLevel), (With<GridPos>, Without<DeadUnit>)>,
    bench_units: Query<(Entity, &Unit, &StarLevel), (With<BenchPos>, Without<DeadUnit>)>,
    current_state: Res<State<GameState>>,
    mut economy: ResMut<PlayerEconomy>,
    mut selected: ResMut<SelectedUnitState>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if *current_state.get() != GameState::Placement {
        return;
    }

    for interaction in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
            let mut refunded_count = 0;
            let mut total_refund = 0;

            for (entity, unit, star) in board_units.iter() {
                let refund = refund_amount(unit.class, star.0);
                economy.gold += refund;
                total_refund += refund;
                refunded_count += 1;
                commands.entity(entity).despawn_recursive();
            }

            for (entity, unit, star) in bench_units.iter() {
                let refund = refund_amount(unit.class, star.0);
                economy.gold += refund;
                total_refund += refund;
                refunded_count += 1;
                commands.entity(entity).despawn_recursive();
            }

            selected.clear();
            info!("[UI] Squad cleared: {} heroes sold for +{}G -> Total Gold: {}G", refunded_count, total_refund, economy.gold);
        }
    }
}

pub fn handle_preset_button(
    mut commands: Commands,
    textures: Res<GameTextures>,
    mut buttons: Query<&Interaction, (Changed<Interaction>, With<PresetButton>)>,
    board_units: Query<(Entity, &Unit, &StarLevel), (With<GridPos>, Without<DeadUnit>)>,
    current_state: Res<State<GameState>>,
    mut economy: ResMut<PlayerEconomy>,
    mut selected: ResMut<SelectedUnitState>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if *current_state.get() != GameState::Placement {
        return;
    }

    for interaction in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
            for (entity, unit, star) in board_units.iter() {
                economy.gold += refund_amount(unit.class, star.0);
                commands.entity(entity).despawn_recursive();
            }
            selected.clear();

            if economy.gold >= 7 {
                economy.gold -= 7;
                spawn_unit(&mut commands, &textures, UnitClass::Knight, Faction::Player, 2, 0);
                spawn_unit(&mut commands, &textures, UnitClass::Archer, Faction::Player, 0, 1);
                spawn_unit(&mut commands, &textures, UnitClass::Assassin, Faction::Player, 2, 2);
                info!("[UI] Preset squad deployed (Cost: 7G) -> Remaining Gold: {}G", economy.gold);
            } else {
                info!("[UI] Cannot deploy preset squad: Need 7G (Current: {}G)", economy.gold);
            }
        }
    }
}

pub fn handle_unit_and_tile_interaction(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    hovered: Res<HoveredTile>,
    mut selected: ResMut<SelectedUnitState>,
    mut economy: ResMut<PlayerEconomy>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut tooltip: Query<&mut Text, With<TooltipText>>,
    board_units: Query<(Entity, &Unit, &GridPos, &StarLevel), Without<DeadUnit>>,
    bench_units: Query<(Entity, &Unit, &BenchPos, &StarLevel), Without<DeadUnit>>,
    mut transforms: Query<&mut Transform, With<Unit>>,
) {
    // 0. Auto-prune stale selected entity if despawned
    if let Some(sel_ent) = selected.entity {
        if transforms.get(sel_ent).is_err() {
            selected.clear();
        }
    }

    // 1. Right Click -> Sell Unit
    if mouse.just_pressed(MouseButton::Right) {
        if let Some(tile) = &hovered.tile {
            if tile.faction == Faction::Player {
                if let Some((ent, unit, _, star)) = board_units.iter().find(|(_, _, g, _)| g.col == tile.col && g.row == tile.row && g.faction == Faction::Player) {
                    let refund = refund_amount(unit.class, star.0);
                    economy.gold += refund;
                    if let Some(e_cmd) = commands.get_entity(ent) { e_cmd.despawn_recursive(); }
                    if selected.entity == Some(ent) {
                        selected.clear();
                    }
                    sound_events.send(PlaySoundEvent(SoundEffect::Click));
                    info!("[SELL] Sold {}★ {} from Board for +{}G -> Total: {}G", star.0, unit.class.name(), refund, economy.gold);
                    if let Ok(mut txt) = tooltip.get_single_mut() {
                        *txt = Text::new(format!("Sold {}★ {} for +{}G!", star.0, unit.class.name(), refund));
                    }
                    return;
                }
            }
        }

        if let Some(slot) = hovered.bench_slot {
            if let Some((ent, unit, _, star)) = bench_units.iter().find(|(_, _, b, _)| b.slot == slot) {
                let refund = refund_amount(unit.class, star.0);
                economy.gold += refund;
                if let Some(e_cmd) = commands.get_entity(ent) { e_cmd.despawn_recursive(); }
                if selected.entity == Some(ent) {
                    selected.clear();
                }
                sound_events.send(PlaySoundEvent(SoundEffect::Click));
                info!("[SELL] Sold {}★ {} from Bench #{} for +{}G -> Total: {}G", star.0, unit.class.name(), slot + 1, refund, economy.gold);
                if let Ok(mut txt) = tooltip.get_single_mut() {
                    *txt = Text::new(format!("Sold {}★ {} for +{}G!", star.0, unit.class.name(), refund));
                }
                return;
            }
        }
    }

    // 2. Left Click -> Select, Move, or Swap
    if mouse.just_pressed(MouseButton::Left) {
        let unit_at_cursor: Option<(Entity, UnitClass, UnitLocation)> = {
            if let Some(tile) = &hovered.tile {
                if tile.faction == Faction::Player {
                    board_units
                        .iter()
                        .find(|(_, _, g, _)| g.col == tile.col && g.row == tile.row && g.faction == Faction::Player)
                        .map(|(e, u, g, _)| (e, u.class, UnitLocation::Board(*g)))
                } else {
                    None
                }
            } else if let Some(slot) = hovered.bench_slot {
                bench_units
                    .iter()
                    .find(|(_, _, b, _)| b.slot == slot)
                    .map(|(e, u, b, _)| (e, u.class, UnitLocation::Bench(b.slot)))
            } else {
                None
            }
        };

        if let Some((target_ent, target_class, target_loc)) = unit_at_cursor {
            if let Some(sel_ent) = selected.entity {
                if sel_ent == target_ent {
                    selected.clear();
                    sound_events.send(PlaySoundEvent(SoundEffect::Click));
                } else {
                    let sel_loc = selected.location.unwrap();
                    match (sel_loc, target_loc) {
                        (UnitLocation::Board(sel_g), UnitLocation::Board(target_g)) => {
                            if let Some(mut c1) = commands.get_entity(sel_ent) {
                                c1.insert(target_g);
                            }
                            if let Some(mut c2) = commands.get_entity(target_ent) {
                                c2.insert(sel_g);
                            }
                            if true {

                                let p1 = grid_to_world_pos(target_g.col, target_g.row, Faction::Player);
                                let p2 = grid_to_world_pos(sel_g.col, sel_g.row, Faction::Player);
                                if let Ok(mut t) = transforms.get_mut(sel_ent) {
                                    t.translation.x = p1.x;
                                    t.translation.y = p1.y;
                                    t.translation.z = 10.0 + (target_g.row as f32 * -0.5);
                                }
                                if let Ok(mut t) = transforms.get_mut(target_ent) {
                                    t.translation.x = p2.x;
                                    t.translation.y = p2.y;
                                    t.translation.z = 10.0 + (sel_g.row as f32 * -0.5);
                                }
                            }
                        }
                        (UnitLocation::Bench(sel_s), UnitLocation::Bench(target_s)) => {
                            if let Some(mut c1) = commands.get_entity(sel_ent) {
                                c1.insert(BenchPos { slot: target_s });
                            }
                            if let Some(mut c2) = commands.get_entity(target_ent) {
                                c2.insert(BenchPos { slot: sel_s });
                            }
                            if true {

                                let p1 = bench_world_pos(target_s);
                                let p2 = bench_world_pos(sel_s);
                                if let Ok(mut t) = transforms.get_mut(sel_ent) {
                                    t.translation.x = p1.x;
                                    t.translation.y = p1.y;
                                }
                                if let Ok(mut t) = transforms.get_mut(target_ent) {
                                    t.translation.x = p2.x;
                                    t.translation.y = p2.y;
                                }
                            }
                        }
                        (UnitLocation::Board(board_g), UnitLocation::Bench(bench_s)) => {
                            if let Some(mut c1) = commands.get_entity(sel_ent) {
                                c1.remove::<GridPos>().insert(BenchPos { slot: bench_s });
                            }
                            if let Some(mut c2) = commands.get_entity(target_ent) {
                                c2.remove::<BenchPos>().insert(board_g);
                            }
                            if true {

                                let p_bench = bench_world_pos(bench_s);
                                let p_board = grid_to_world_pos(board_g.col, board_g.row, Faction::Player);
                                if let Ok(mut t) = transforms.get_mut(sel_ent) {
                                    t.translation.x = p_bench.x;
                                    t.translation.y = p_bench.y;
                                    t.translation.z = 10.0;
                                }
                                if let Ok(mut t) = transforms.get_mut(target_ent) {
                                    t.translation.x = p_board.x;
                                    t.translation.y = p_board.y;
                                    t.translation.z = 10.0 + (board_g.row as f32 * -0.5);
                                }
                            }
                        }
                        (UnitLocation::Bench(bench_s), UnitLocation::Board(board_g)) => {
                            if let Some(mut c1) = commands.get_entity(sel_ent) {
                                c1.remove::<BenchPos>().insert(board_g);
                            }
                            if let Some(mut c2) = commands.get_entity(target_ent) {
                                c2.remove::<GridPos>().insert(BenchPos { slot: bench_s });
                            }
                            if true {

                                let p_board = grid_to_world_pos(board_g.col, board_g.row, Faction::Player);
                                let p_bench = bench_world_pos(bench_s);
                                if let Ok(mut t) = transforms.get_mut(sel_ent) {
                                    t.translation.x = p_board.x;
                                    t.translation.y = p_board.y;
                                    t.translation.z = 10.0 + (board_g.row as f32 * -0.5);
                                }
                                if let Ok(mut t) = transforms.get_mut(target_ent) {
                                    t.translation.x = p_bench.x;
                                    t.translation.y = p_bench.y;
                                    t.translation.z = 10.0;
                                }
                            }
                        }
                    }
                    selected.clear();
                    sound_events.send(PlaySoundEvent(SoundEffect::Click));
                    info!("[SWAP] Swapped heroes positions!");
                }
            } else {
                selected.entity = Some(target_ent);
                selected.location = Some(target_loc);
                selected.class = Some(target_class);
                sound_events.send(PlaySoundEvent(SoundEffect::Click));
                info!("[SELECT] Selected {:?} at {:?}", target_class, target_loc);
            }
        } else {
            if let Some(sel_ent) = selected.entity {
                let sel_loc = selected.location.unwrap();

                if let Some(tile) = &hovered.tile {
                    if tile.faction == Faction::Player {
                        let target_g = GridPos {
                            col: tile.col,
                            row: tile.row,
                            faction: Faction::Player,
                        };

                        if let Some(mut c) = commands.get_entity(sel_ent) {
                            if let UnitLocation::Bench(_) = sel_loc {
                                let active_count = board_units.iter().count();
                                if active_count >= MAX_PLAYER_UNITS {
                                    info!("[DEPLOY] Board is full (5/5)! Cannot deploy another hero.");
                                    if let Ok(mut txt) = tooltip.get_single_mut() {
                                        *txt = Text::new("⚠️ Board squad is full (5/5)! Swap with an active hero instead.".to_string());
                                    }
                                    return;
                                }
                                c.remove::<BenchPos>().insert(target_g);
                            } else {
                                c.insert(target_g);
                            }
                        }

                        let p = grid_to_world_pos(target_g.col, target_g.row, Faction::Player);
                        if let Ok(mut t) = transforms.get_mut(sel_ent) {
                            t.translation.x = p.x;
                            t.translation.y = p.y;
                            t.translation.z = 10.0 + (target_g.row as f32 * -0.5);
                        }
                        selected.clear();
                        sound_events.send(PlaySoundEvent(SoundEffect::Click));
                        info!("[MOVE] Placed hero on Board ({}, {})", target_g.col, target_g.row);
                    }
                } else if let Some(slot) = hovered.bench_slot {
                    if let Some(mut c) = commands.get_entity(sel_ent) {
                        if let UnitLocation::Board(_) = sel_loc {
                            c.remove::<GridPos>().insert(BenchPos { slot });
                        } else {
                            c.insert(BenchPos { slot });
                        }
                    }

                    let p = bench_world_pos(slot);
                    if let Ok(mut t) = transforms.get_mut(sel_ent) {
                        t.translation.x = p.x;
                        t.translation.y = p.y;
                        t.translation.z = 10.0;
                    }
                    selected.clear();
                    sound_events.send(PlaySoundEvent(SoundEffect::Click));
                    info!("[MOVE] Placed hero on Bench Slot #{}", slot + 1);
                }
            }
        }
    }
}

pub fn update_tooltip_system(
    hovered: Res<HoveredTile>,
    selected: Res<SelectedUnitState>,
    board_units: Query<(&Unit, &GridPos, &UnitStats, &StarLevel), Without<DeadUnit>>,
    bench_units: Query<(&Unit, &BenchPos, &UnitStats, &StarLevel), Without<DeadUnit>>,
    mut tooltip: Query<&mut Text, With<TooltipText>>,
) {
    let Ok(mut text) = tooltip.get_single_mut() else {
        return;
    };

    if let Some(sel_ent) = selected.entity {
        let (class, star, is_bench) = if let Ok((u, _, _, s)) = board_units.get(sel_ent) {
            (u.class, s.0, false)
        } else if let Ok((u, _, _, s)) = bench_units.get(sel_ent) {
            (u.class, s.0, true)
        } else {
            (UnitClass::Knight, 1, false)
        };

        *text = Text::new(format!(
            "Selected: {}★ {} ({}) - Left-Click empty slot to place, click another hero to swap, Right-Click to sell.",
            star,
            class.name(),
            if is_bench { "Reserve Bench" } else { "Active Board" }
        ));
        return;
    }

    if let Some(tile) = &hovered.tile {
        if let Some((unit, _, stats, star)) = board_units
            .iter()
            .find(|(_, g, _, _)| g.col == tile.col && g.row == tile.row && g.faction == tile.faction)
        {
            *text = Text::new(format!(
                "Hovering: {} {}★ {} [{}] - HP: {:.0}/{:.0} | ATK: {:.0} | DEF: {:.0} | SPD: {:.0} (Right-Click to sell)",
                unit.class.icon(),
                star.0,
                unit.class.name(),
                if unit.faction == Faction::Player { "Ally" } else { "Enemy" },
                stats.hp,
                stats.max_hp,
                stats.atk,
                stats.def,
                stats.speed,
            ));
            return;
        }
    }

    if let Some(slot) = hovered.bench_slot {
        if let Some((unit, _, stats, star)) = bench_units
            .iter()
            .find(|(_, b, _, _)| b.slot == slot)
        {
            *text = Text::new(format!(
                "Reserve Bench #{}: {} {}★ {} - HP: {:.0}/{:.0} | ATK: {:.0} | DEF: {:.0} (Click to deploy/swap, Right-Click to sell)",
                slot + 1,
                unit.class.icon(),
                star.0,
                unit.class.name(),
                stats.hp,
                stats.max_hp,
                stats.atk,
                stats.def,
            ));
            return;
        }
    }

    *text = Text::new("💡 Click a shop card below to recruit. Left-Click heroes to position/swap. Right-Click to sell for gold.".to_string());
}

pub fn setup_stage_enemies(
    mut commands: Commands,
    textures: Res<GameTextures>,
    stage: Res<CurrentStage>,
    units: Query<(Entity, &Unit)>,
    mut title_query: Query<&mut Text, (With<StageTitleText>, Without<StageDescText>)>,
    mut desc_query: Query<&mut Text, (With<StageDescText>, Without<StageTitleText>)>,
    pvp_mgr: Res<crate::net::PvpManager>,
) {
    if pvp_mgr.active {
        for (entity, unit) in units.iter() {
            if unit.faction == Faction::Enemy {
                if let Some(e) = commands.get_entity(entity) { e.despawn_recursive(); }
            }
        }
        for mut text in title_query.iter_mut() {
            *text = Text::new(format!("⚔️ Online PvP Arena - Round #{}", pvp_mgr.round));
        }
        for mut text in desc_query.iter_mut() {
            *text = Text::new(format!("Room: {} | You: {} ({} HP) vs Opponent: {} ({} HP)", pvp_mgr.room_code, pvp_mgr.player_name, pvp_mgr.player_hp, pvp_mgr.opponent_name, pvp_mgr.opponent_hp));
        }
        let player_count = units.iter().filter(|(_, u)| u.faction == Faction::Player).count();
        if player_count == 0 {
            spawn_unit(&mut commands, &textures, UnitClass::Knight, Faction::Player, 2, 0);
            spawn_unit(&mut commands, &textures, UnitClass::Archer, Faction::Player, 0, 1);
            spawn_unit(&mut commands, &textures, UnitClass::Assassin, Faction::Player, 1, 2);
        }
        return;
    }
    for (entity, unit) in units.iter() {
        if unit.faction == Faction::Enemy {
            if let Some(e) = commands.get_entity(entity) { e.despawn_recursive(); }
        }
    }

    let stage_def = get_stage_def(stage.stage_idx);
    info!("[STAGE] Loaded Stage #{}: {} - {}", stage.stage_idx, stage_def.title, stage_def.description);
    for mut text in title_query.iter_mut() {
        *text = Text::new(&stage_def.title);
    }
    for mut text in desc_query.iter_mut() {
        *text = Text::new(&stage_def.description);
    }

    for enemy in stage_def.enemies {
        spawn_unit_ext(
            &mut commands,
            &textures,
            enemy.unit_class,
            Faction::Enemy,
            enemy.col,
            enemy.row,
            enemy.star_level,
            enemy.is_boss,
        );
    }

    // Pre-spawn starter squad for player if no units exist yet
    let player_count = units.iter().filter(|(_, u)| u.faction == Faction::Player).count();
    if player_count == 0 {
        spawn_unit(&mut commands, &textures, UnitClass::Knight, Faction::Player, 2, 0);
        spawn_unit(&mut commands, &textures, UnitClass::Archer, Faction::Player, 0, 1);
        spawn_unit(&mut commands, &textures, UnitClass::Assassin, Faction::Player, 2, 2);
    }
}

pub fn reset_player_units_for_placement(
    mut commands: Commands,
    textures: Res<GameTextures>,
    mut units: Query<
        (
            Entity,
            &Unit,
            &GridPos,
            &mut Transform,
            &mut Visibility,
            &mut UnitStats,
            Option<&mut ActionGauge>,
        ),
        Without<DeadUnit>,
    >,
    dead_units: Query<(Entity, &Unit, &GridPos), With<DeadUnit>>,
) {
    use crate::board::grid_to_world_pos;
    for (_, unit, grid, mut transform, mut vis, mut stats, maybe_gauge) in units.iter_mut() {
        if unit.faction == Faction::Player {
            let pos = grid_to_world_pos(grid.col, grid.row, Faction::Player);
            transform.translation = Vec3::new(pos.x, pos.y, 10.0 + (grid.row as f32 * -0.5));
            *vis = Visibility::Inherited;
            stats.hp = stats.max_hp;
            stats.shield = 0.0;
            stats.mana = 0.0;
            if let Some(mut gauge) = maybe_gauge {
                gauge.current = 0.0;
            }
        }
    }

    for (entity, unit, grid) in dead_units.iter() {
        if unit.faction == Faction::Player {
            commands.entity(entity).despawn_recursive();
            spawn_unit(
                &mut commands,
                &textures,
                unit.class,
                Faction::Player,
                grid.col,
                grid.row,
            );
        }
    }
}

pub fn show_victory_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.14, 0.07, 0.90)),
            ResultUiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("VICTORY!"),
                TextFont {
                    font_size: 46.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.88, 0.2)),
            ));

            parent.spawn((
                Text::new("You have vanquished the entire enemy force upon the arena!"),
                TextFont {
                    font_size: 16.5,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(15.0)),
                    ..default()
                })
                .with_children(|btn_row| {
                    btn_row
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(180.0),
                                height: Val::Px(48.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.7, 0.3)),
                            BorderRadius::all(Val::Px(8.0)),
                            NextStageButton,
                        ))
                        .with_child((
                            Text::new("NEXT STAGE >>"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                    btn_row
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(150.0),
                                height: Val::Px(48.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.4, 0.5)),
                            BorderRadius::all(Val::Px(8.0)),
                            RetryButton,
                        ))
                        .with_child((
                            Text::new("Retry Stage"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                });
        });
}

pub fn show_defeat_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.04, 0.07, 0.90)),
            ResultUiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("DEFEAT!"),
                TextFont {
                    font_size: 46.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
            ));

            parent.spawn((
                Text::new("Your champions have fallen! Adjust your positioning and try again."),
                TextFont {
                    font_size: 16.5,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(15.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.75, 0.25, 0.25)),
                    BorderRadius::all(Val::Px(8.0)),
                    RetryButton,
                ))
                .with_child((
                    Text::new("RETRY <<"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
        });
}

pub fn teardown_result_ui(mut commands: Commands, query: Query<Entity, With<ResultUiRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_result_buttons(
    mut next_state: ResMut<NextState<GameState>>,
    mut stage: ResMut<CurrentStage>,
    next_btn: Query<&Interaction, (Changed<Interaction>, With<NextStageButton>)>,
    retry_btn: Query<&Interaction, (Changed<Interaction>, With<RetryButton>)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    for interaction in next_btn.iter() {
        if *interaction == Interaction::Pressed {
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
            stage.stage_idx += 1;
            info!("[STAGE] Advancing to Next Stage (Index: {})", stage.stage_idx);
            next_state.set(GameState::Placement);
        }
    }

    for interaction in retry_btn.iter() {
        if *interaction == Interaction::Pressed {
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
            info!("[STAGE] Retrying Stage (Index: {})", stage.stage_idx);
            next_state.set(GameState::Placement);
        }
    }
}

pub fn update_gold_display_system(
    economy: Res<PlayerEconomy>,
    mut text_query: Query<&mut Text, With<GoldDisplayText>>,
) {
    let interest = (economy.gold / 10).clamp(0, 5);
    for mut text in text_query.iter_mut() {
        *text = Text::new(format!("🪙 {}G (+{}G next)", economy.gold, 5 + interest));
    }
}

pub fn handle_reroll_and_lock_buttons(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut economy: ResMut<PlayerEconomy>,
    mut rng: ResMut<crate::battle::BattleRng>,
    mut reroll_buttons: Query<&Interaction, (Changed<Interaction>, With<ShopRerollButton>)>,
    mut lock_buttons: Query<&Interaction, (Changed<Interaction>, With<ShopLockToggle>)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut tooltip: Query<&mut Text, With<TooltipText>>,
) {
    let mut do_reroll = false;
    for interaction in reroll_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            do_reroll = true;
        }
    }
    if keyboard.just_pressed(KeyCode::KeyD) {
        do_reroll = true;
    }

    if do_reroll {
        if economy.reroll(&mut rng) {
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
        } else if let Ok(mut txt) = tooltip.get_single_mut() {
            *txt = Text::new("⚠️ Need at least 2G to roll the shop!".to_string());
        }
    }

    let mut toggle_lock = false;
    for interaction in lock_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            toggle_lock = true;
        }
    }
    if keyboard.just_pressed(KeyCode::KeyE) {
        toggle_lock = true;
    }

    if toggle_lock {
        economy.shop_locked = !economy.shop_locked;
        info!("[SHOP] Shop lock toggled -> Locked: {}", economy.shop_locked);
        sound_events.send(PlaySoundEvent(SoundEffect::Click));
    }
}

pub fn hide_placement_ui_on_battle(
    mut query: Query<&mut Visibility, With<PlacementUiRoot>>,
) {
    for mut vis in query.iter_mut() {
        *vis = Visibility::Hidden;
    }
}

pub fn show_placement_ui_on_placement(
    mut query: Query<&mut Visibility, With<PlacementUiRoot>>,
) {
    for mut vis in query.iter_mut() {
        *vis = Visibility::Inherited;
    }
}

pub fn handle_keyboard_gameplay_shortcuts(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    textures: Res<GameTextures>,
    mut economy: ResMut<PlayerEconomy>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut tooltip: Query<&mut Text, With<TooltipText>>,
    hovered: Res<HoveredTile>,
    mut selected: ResMut<SelectedUnitState>,
    board_units: Query<(Entity, &Unit, &GridPos, &StarLevel), Without<DeadUnit>>,
    bench_units: Query<(Entity, &Unit, &BenchPos, &StarLevel), Without<DeadUnit>>,
) {
    // 1. Buy cards with 1, 2, 3, 4 (and Numpad 1-4)
    let mut buy_slot: Option<usize> = None;
    if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::Numpad1) {
        buy_slot = Some(0);
    } else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) {
        buy_slot = Some(1);
    } else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) {
        buy_slot = Some(2);
    } else if keyboard.just_pressed(KeyCode::Digit4) || keyboard.just_pressed(KeyCode::Numpad4) {
        buy_slot = Some(3);
    }

    if let Some(slot_idx) = buy_slot {
        if let Some(class) = economy.shop_slots[slot_idx] {
            let cost = unit_cost(class);
            if economy.gold < cost {
                info!("[SHOP] Cannot buy {}: Not enough gold (Have: {}G, Need: {}G)", class.name(), economy.gold, cost);
                if let Ok(mut txt) = tooltip.get_single_mut() {
                    *txt = Text::new(format!("⚠️ Not enough gold! Need {}G, have {}G.", cost, economy.gold));
                }
            } else {
                let occupied_slots: Vec<usize> = bench_units.iter().map(|(_, _, b, _)| b.slot).collect();
                let free_slot = (0..BENCH_SLOTS).find(|s| !occupied_slots.contains(s));

                if let Some(slot) = free_slot {
                    if let Some(bought_class) = economy.buy_slot(slot_idx) {
                        spawn_bench_unit(
                            &mut commands,
                            &textures,
                            bought_class,
                            slot,
                            1,
                        );
                        sound_events.send(PlaySoundEvent(SoundEffect::Click));
                        info!("[SHOP] Recruited {:?} for {}G -> placed on Reserve Bench Slot #{} (Remaining Gold: {}G)", bought_class, cost, slot + 1, economy.gold);
                        if let Ok(mut txt) = tooltip.get_single_mut() {
                            *txt = Text::new(format!("Recruited {} for {}G (Placed on Bench #{}) [HotKey #{}]", bought_class.name(), cost, slot + 1, slot_idx + 1));
                        }
                    }
                } else {
                    info!("[SHOP] Reserve bench is full (6/6 slots occupied)!");
                    if let Ok(mut txt) = tooltip.get_single_mut() {
                        *txt = Text::new("⚠️ Reserve Bench is full (6/6)! Deploy or sell a hero first.".to_string());
                    }
                }
            }
        }
    }

    // 2. Sell unit with S, Delete, or Backspace
    if keyboard.just_pressed(KeyCode::KeyS) || keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
        let mut target_to_sell: Option<(Entity, UnitClass, i32, u8)> = None;

        // A. Hovered board tile
        if let Some(tile) = &hovered.tile {
            if tile.faction == Faction::Player {
                if let Some((ent, unit, _, star)) = board_units.iter().find(|(_, _, g, _)| g.col == tile.col && g.row == tile.row && g.faction == Faction::Player) {
                    target_to_sell = Some((ent, unit.class, refund_amount(unit.class, star.0), star.0));
                }
            }
        }

        // B. Hovered bench slot
        if target_to_sell.is_none() {
            if let Some(slot) = hovered.bench_slot {
                if let Some((ent, unit, _, star)) = bench_units.iter().find(|(_, _, b, _)| b.slot == slot) {
                    target_to_sell = Some((ent, unit.class, refund_amount(unit.class, star.0), star.0));
                }
            }
        }

        // C. Currently selected unit
        if target_to_sell.is_none() {
            if let Some(sel_ent) = selected.entity {
                if let Some((ent, unit, _, star)) = board_units.iter().find(|(e, _, _, _)| *e == sel_ent) {
                    target_to_sell = Some((ent, unit.class, refund_amount(unit.class, star.0), star.0));
                } else if let Some((ent, unit, _, star)) = bench_units.iter().find(|(e, _, _, _)| *e == sel_ent) {
                    target_to_sell = Some((ent, unit.class, refund_amount(unit.class, star.0), star.0));
                }
            }
        }

        if let Some((ent, class, refund, star)) = target_to_sell {
            economy.gold += refund;
            if let Some(e_cmd) = commands.get_entity(ent) {
                e_cmd.despawn_recursive();
            }
            if selected.entity == Some(ent) {
                selected.clear();
            }
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
            info!("[SELL] Hotkey sold {}★ {} for +{}G -> Total: {}G", star, class.name(), refund, economy.gold);
            if let Ok(mut txt) = tooltip.get_single_mut() {
                *txt = Text::new(format!("Sold {}★ {} for +{}G! [Key: S]", star, class.name(), refund));
            }
        }
    }
}
