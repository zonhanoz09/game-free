use super::*;

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
                            Text::new(format!(
                                "{} {}",
                                syn.icon(),
                                syn.name().split(" ").next().unwrap()
                            )),
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
                Text::new(
                    "[1-4] Mua | [S] Bán | [D] Đổi (2G) | [E] Khóa | [Space] Chiến | [H] Hướng Dẫn",
                ),
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
                        Text::new("⚔️ ĐẤU AI VỚI ĐỘI HÌNH LƯU"),
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
