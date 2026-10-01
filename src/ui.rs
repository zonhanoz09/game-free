use crate::battle::ActionGauge;
use crate::audio::{PlaySoundEvent, SoundEffect};
use crate::board::HoveredTile;
use crate::economy::{unit_cost, GoldDisplayText, PlayerEconomy, ShopLockToggle, ShopRerollButton};
use crate::stages::get_stage_def;
use crate::synergies::{SynergyContainer, SynergyCountText, SynergyRow, SynergyType};
use crate::types::*;
use crate::units::{Unit, spawn_unit, spawn_unit_ext};
use bevy::prelude::*;

#[derive(Component)]
pub struct StartBattleButton;

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
pub struct BenchButton(pub UnitClass);

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

pub fn setup_ui(mut commands: Commands, textures: Res<GameTextures>) {
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
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(82.0),
                right: Val::Px(16.0),
                width: Val::Px(285.0),
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
            bottom: Val::Px(140.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_child((
            Text::new(""),
            TextFont {
                font_size: 13.5,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.95, 0.7)),
            TooltipText,
        ));

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
            // Bench row (5 classes with mini avatars)
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    let classes = [
                        UnitClass::Knight,
                        UnitClass::Archer,
                        UnitClass::Mage,
                        UnitClass::Assassin,
                        UnitClass::Cleric,
                    ];

                    for class in classes {
                        let col = class.color();
                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(142.0),
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
                            BackgroundColor(Color::srgba(
                                col.to_srgba().red * 0.35,
                                col.to_srgba().green * 0.35,
                                col.to_srgba().blue * 0.35,
                                0.95,
                            )),
                            BorderRadius::all(Val::Px(6.0)),
                            BenchButton(class),
                        ))
                        .with_children(|btn| {
                            // Avatar thumbnail
                            btn.spawn((
                                ImageNode {
                                    image: textures.get_unit_texture(class),
                                    ..default()
                                },
                                Node {
                                    width: Val::Px(44.0),
                                    height: Val::Px(44.0),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(4.0)),
                            ));

                            // Info column
                            btn.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                ..default()
                            })
                            .with_children(|txt_col| {
                                txt_col.spawn((
                                    Text::new(class.name()),
                                    TextFont {
                                        font_size: 13.5,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                                txt_col.spawn((
                                    Text::new(format!(
                                        "{}🪙 | HP:{} ATK:{}",
                                        unit_cost(class),
                                        class.base_stats().max_hp as i32,
                                        class.base_stats().atk as i32
                                    )),
                                    TextFont {
                                        font_size: 10.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgba(1.0, 0.85, 0.2, 0.9)),
                                ));
                            });
                        });
                    }
                });

            // Action Buttons Row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    // Preset Button
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(130.0),
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
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Start Battle Button
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(210.0),
                            height: Val::Px(42.0),
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
                        Text::new("BATTLE START"),
                        TextFont {
                            font_size: 15.5,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Clear Button
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(100.0),
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

                    // Reroll Shop Button (2G)
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(110.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor(Color::srgb(1.0, 0.8, 0.2)),
                        BackgroundColor(Color::srgb(0.28, 0.20, 0.12)),
                        BorderRadius::all(Val::Px(6.0)),
                        ShopRerollButton,
                    ))
                    .with_child((
                        Text::new("🎲 Roll 2G"),
                        TextFont {
                            font_size: 12.5,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.88, 0.3)),
                    ));

                    // Lock Shop Button
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(90.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.22, 0.25, 0.32)),
                        BorderRadius::all(Val::Px(6.0)),
                        ShopLockToggle,
                    ))
                    .with_child((
                        Text::new("🔒 Lock"),
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
    selected: Res<SelectedBenchUnit>,
    textures: Res<GameTextures>,
    units: Query<(&Unit, &GridPos, &UnitStats), Without<DeadUnit>>,
    mut avatar_query: Query<&mut ImageNode, With<InspectHeroAvatar>>,
    mut header_query: Query<(&InspectHeader, &mut Text, Option<&mut TextColor>)>,
    mut bar_query: Query<(&InspectStatBar, &mut Node)>,
    mut text_query: Query<(&InspectStatText, &mut Text), Without<InspectHeader>>,
    mut skill_query: Query<
        (&InspectSkill, &mut Text),
        (Without<InspectHeader>, Without<InspectStatText>),
    >,
) {
    let mut inspected_unit: Option<(UnitClass, Faction, UnitStats)> = None;

    if let Some(tile) = &hovered.tile {
        if let Some((unit, _, stats)) = units
            .iter()
            .find(|(_, g, _)| g.col == tile.col && g.row == tile.row && g.faction == tile.faction)
        {
            inspected_unit = Some((unit.class, unit.faction, *stats));
        }
    }

    if inspected_unit.is_none() {
        if let Some(class) = selected.unit_class {
            inspected_unit = Some((class, Faction::Player, class.base_stats()));
        }
    }

    let (class, faction, stats) = inspected_unit.unwrap_or_else(|| {
        (
            UnitClass::Knight,
            Faction::Player,
            UnitClass::Knight.base_stats(),
        )
    });

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

pub fn update_bench_ui(
    selected: Res<SelectedBenchUnit>,
    mut buttons: Query<(&BenchButton, &mut BorderColor, &mut BackgroundColor)>,
) {
    for (bench_btn, mut border, mut bg) in buttons.iter_mut() {
        let is_sel = selected.unit_class == Some(bench_btn.0);
        let col = bench_btn.0.color();
        if is_sel {
            *border = BorderColor(Color::srgb(1.0, 0.9, 0.2));
            *bg = BackgroundColor(Color::srgba(
                col.to_srgba().red * 0.75,
                col.to_srgba().green * 0.75,
                col.to_srgba().blue * 0.75,
                1.0,
            ));
        } else {
            *border = BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.2));
            *bg = BackgroundColor(Color::srgba(
                col.to_srgba().red * 0.35,
                col.to_srgba().green * 0.35,
                col.to_srgba().blue * 0.35,
                0.95,
            ));
        }
    }
}

pub fn handle_bench_clicks(
    mut selected: ResMut<SelectedBenchUnit>,
    mut buttons: Query<(&Interaction, &BenchButton), (Changed<Interaction>, With<Button>)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    for (interaction, bench_btn) in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
            if selected.unit_class == Some(bench_btn.0) {
                selected.unit_class = None;
                info!("[UI] Bench selection deselected");
            } else {
                selected.unit_class = Some(bench_btn.0);
                info!("[UI] Bench hero selected: {:?} (Cost: {}G)", bench_btn.0, unit_cost(bench_btn.0));
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
    units: Query<(&Unit, &UnitStats), Without<DeadUnit>>,
    mut buttons: Query<(&Interaction, &mut BackgroundColor), With<StartBattleButton>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut sound_events: EventWriter<PlaySoundEvent>,
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
        let player_count = units
            .iter()
            .filter(|(u, _)| u.faction == Faction::Player)
            .count();
        if player_count > 0 {
            info!("[UI] ⚔️ Battle Start triggered! (Active player heroes: {})", player_count);
            next_state.set(GameState::Battle);
        } else {
            info!("[UI] ⚔️ Battle Start triggered with 0 units: Auto-deployed starter squad (Knight, Archer, Assassin)!");
            spawn_unit(&mut commands, &textures, UnitClass::Knight, Faction::Player, 2, 0);
            spawn_unit(&mut commands, &textures, UnitClass::Archer, Faction::Player, 0, 1);
            spawn_unit(&mut commands, &textures, UnitClass::Assassin, Faction::Player, 2, 2);
            next_state.set(GameState::Battle);
        }
    }
}

pub fn handle_clear_button(
    mut commands: Commands,
    mut buttons: Query<&Interaction, (Changed<Interaction>, With<ClearBoardButton>)>,
    units: Query<(Entity, &Unit)>,
    current_state: Res<State<GameState>>,
    mut economy: ResMut<PlayerEconomy>,
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
            for (entity, unit) in units.iter() {
                if unit.faction == Faction::Player {
                    let cost = unit_cost(unit.class);
                    economy.gold += cost;
                    total_refund += cost;
                    refunded_count += 1;
                    commands.entity(entity).despawn_recursive();
                }
            }
            info!("[UI] Board cleared: {} heroes sold for +{}G -> Total Gold: {}G", refunded_count, total_refund, economy.gold);
        }
    }
}

pub fn handle_preset_button(
    mut commands: Commands,
    textures: Res<GameTextures>,
    mut buttons: Query<&Interaction, (Changed<Interaction>, With<PresetButton>)>,
    units: Query<(Entity, &Unit)>,
    current_state: Res<State<GameState>>,
    mut economy: ResMut<PlayerEconomy>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if *current_state.get() != GameState::Placement {
        return;
    }

    for interaction in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
            for (entity, unit) in units.iter() {
                if unit.faction == Faction::Player {
                    economy.gold += unit_cost(unit.class);
                    commands.entity(entity).despawn_recursive();
                }
            }

            if economy.gold >= 7 {
                economy.gold -= 7;
                spawn_unit(&mut commands, &textures, UnitClass::Knight, Faction::Player, 2, 0);
                spawn_unit(&mut commands, &textures, UnitClass::Knight, Faction::Player, 2, 2);
                spawn_unit(&mut commands, &textures, UnitClass::Assassin, Faction::Player, 2, 1);
                info!("[UI] Preset squad deployed (Cost: 7G) -> Remaining Gold: {}G", economy.gold);
            } else {
                info!("[UI] Cannot deploy preset squad: Need 7G (Current: {}G)", economy.gold);
            }
        }
    }
}

pub fn handle_tile_mouse_placement(
    mut commands: Commands,
    textures: Res<GameTextures>,
    mouse: Res<ButtonInput<MouseButton>>,
    hovered: Res<HoveredTile>,
    selected: Res<SelectedBenchUnit>,
    units: Query<(Entity, &Unit, &GridPos), Without<DeadUnit>>,
    current_state: Res<State<GameState>>,
    mut economy: ResMut<PlayerEconomy>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    if *current_state.get() != GameState::Placement {
        return;
    }

    let Some(hovered_tile) = &hovered.tile else {
        return;
    };
    if hovered_tile.faction != Faction::Player {
        return;
    };

    let target_col = hovered_tile.col;
    let target_row = hovered_tile.row;

    let existing_on_tile = units.iter().find(|(_, u, g)| {
        u.faction == Faction::Player && g.col == target_col && g.row == target_row
    });

    if mouse.just_pressed(MouseButton::Left) {
        if let Some(unit_class) = selected.unit_class {
            let cost = unit_cost(unit_class);
            if let Some((old_ent, old_unit, _)) = existing_on_tile {
                let old_cost = unit_cost(old_unit.class);
                if economy.gold + old_cost >= cost {
                    economy.gold = economy.gold + old_cost - cost;
                    commands.entity(old_ent).despawn_recursive();
                    spawn_unit(
                        &mut commands,
                        &textures,
                        unit_class,
                        Faction::Player,
                        target_col,
                        target_row,
                    );
                    info!("[PLACEMENT] Replaced {:?} with {:?} at ({}, {}) -> Remaining Gold: {}G", old_unit.class, unit_class, target_col, target_row, economy.gold);
                    sound_events.send(PlaySoundEvent(SoundEffect::Click));
                }
            } else {
                let current_count = units
                    .iter()
                    .filter(|(_, u, _)| u.faction == Faction::Player)
                    .count();
                if current_count < MAX_PLAYER_UNITS && economy.gold >= cost {
                    economy.gold -= cost;
                    spawn_unit(
                        &mut commands,
                        &textures,
                        unit_class,
                        Faction::Player,
                        target_col,
                        target_row,
                    );
                    info!("[PLACEMENT] Deployed {:?} to ({}, {}) (Cost: {}G) -> Remaining Gold: {}G", unit_class, target_col, target_row, cost, economy.gold);
                    sound_events.send(PlaySoundEvent(SoundEffect::Click));
                }
            }
        }
    }

    if mouse.just_pressed(MouseButton::Right) {
        if let Some((old_ent, old_unit, _)) = existing_on_tile {
            let refund = unit_cost(old_unit.class);
            economy.gold += refund;
            commands.entity(old_ent).despawn_recursive();
            info!("[PLACEMENT] Sold {:?} at ({}, {}) (Refund: +{}G) -> Total Gold: {}G", old_unit.class, target_col, target_row, refund, economy.gold);
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
        }
    }
}

pub fn update_tooltip_system(
    hovered: Res<HoveredTile>,
    selected: Res<SelectedBenchUnit>,
    units: Query<(&Unit, &GridPos, &UnitStats), Without<DeadUnit>>,
    mut tooltip: Query<&mut Text, With<TooltipText>>,
) {
    let Ok(mut text) = tooltip.get_single_mut() else {
        return;
    };

    if let Some(tile) = &hovered.tile {
        if let Some((unit, _, stats)) = units
            .iter()
            .find(|(_, g, _)| g.col == tile.col && g.row == tile.row && g.faction == tile.faction)
        {
            *text = Text::new(format!(
                "Hovering: {} {} [{}] - HP: {:.0}/{:.0} | ATK: {:.0} | DEF: {:.0} | SPD: {:.0}",
                unit.class.icon(),
                unit.class.name(),
                if unit.faction == Faction::Player {
                    "Ally"
                } else {
                    "Enemy"
                },
                stats.hp,
                stats.max_hp,
                stats.atk,
                stats.def,
                stats.speed,
            ));
            return;
        }
    }

    if let Some(class) = selected.unit_class {
        let stats = class.base_stats();
        *text = Text::new(format!(
            "Selected: {} {} - HP: {:.0} | ATK: {:.0} | DEF: {:.0} | SPD: {:.0} (Click on blue grid to place)",
            class.icon(),
            class.name(),
            stats.hp,
            stats.atk,
            stats.def,
            stats.speed,
        ));
        return;
    }

    *text = Text::new("");
}

pub fn setup_stage_enemies(
    mut commands: Commands,
    textures: Res<GameTextures>,
    stage: Res<CurrentStage>,
    units: Query<(Entity, &Unit)>,
    mut title_query: Query<&mut Text, (With<StageTitleText>, Without<StageDescText>)>,
    mut desc_query: Query<&mut Text, (With<StageDescText>, Without<StageTitleText>)>,
) {
    for (entity, unit) in units.iter() {
        if unit.faction == Faction::Enemy {
            commands.entity(entity).despawn_recursive();
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
    mut economy: ResMut<PlayerEconomy>,
    mut rng: ResMut<crate::battle::BattleRng>,
    mut reroll_buttons: Query<&Interaction, (Changed<Interaction>, With<ShopRerollButton>)>,
    mut lock_buttons: Query<(&Interaction, &mut BorderColor), (Changed<Interaction>, With<ShopLockToggle>)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    for interaction in reroll_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            if economy.reroll(&mut rng) {
                sound_events.send(PlaySoundEvent(SoundEffect::Click));
            }
        }
    }

    for (interaction, mut border) in lock_buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            economy.shop_locked = !economy.shop_locked;
            if economy.shop_locked {
                border.0 = Color::srgb(1.0, 0.85, 0.2);
            } else {
                border.0 = Color::srgba(1.0, 1.0, 1.0, 0.1);
            }
            info!("[SHOP] Shop Lock toggled -> Locked: {}", economy.shop_locked);
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
        }
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
