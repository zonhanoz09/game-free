use crate::assets_3d::Game3dAssets;
use crate::board::HoveredTile;
use crate::stages::get_stage_def;
use crate::types::*;
use crate::units::{Unit, spawn_unit};
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

pub fn setup_ui(mut commands: Commands, textures: Res<GameTextures>) {
    // Top Bar UI
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            height: Val::Px(64.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(30.0)),
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
                        Text::new("CHIẾN THUẬT 3x3 - ĐẤU TRƯỜNG 3D"),
                        TextFont {
                            font_size: 19.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.95, 1.0)),
                    ));
                    col.spawn((
                        Text::new("Màn 1: Đội Tiên Phong"),
                        TextFont {
                            font_size: 13.5,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.8, 0.3)),
                        StageTitleText,
                    ));
                    col.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 11.5,
                            ..default()
                        },
                        TextColor(Color::srgba(0.85, 0.85, 0.85, 0.8)),
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
                        Text::new("Số quân: 0 / 5"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.3, 0.9, 0.5)),
                        UnitCountText,
                    ));
                    col.spawn((
                        Text::new("Chuột trái sàn đấu: Đặt quân | Chuột phải: Gỡ bỏ"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.85, 0.85, 0.85, 0.75)),
                    ));
                });

            // Right: Battle Speed Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(110.0),
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
                    Text::new("Tốc độ: 1x"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    SpeedText,
                ));
        });

    // Tooltip Banner above the bench
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
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.95, 0.7)),
            TooltipText,
        ));

    // Bottom Bench & Placement Controls
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
                                width: Val::Px(136.0),
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
                                    Text::new(class.name_vi()),
                                    TextFont {
                                        font_size: 13.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                                txt_col.spawn((
                                    Text::new(format!(
                                        "HP:{} C:{}",
                                        class.base_stats().max_hp as i32,
                                        class.base_stats().atk as i32
                                    )),
                                    TextFont {
                                        font_size: 10.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgba(0.9, 0.9, 0.9, 0.8)),
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
                        Text::new("Đội hình mẫu"),
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
                        Text::new("BẮT ĐẦU CHIẾN ĐẤU"),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Clear Button
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(110.0),
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
                        Text::new("Xóa hết"),
                        TextFont {
                            font_size: 13.0,
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
        *text = Text::new(format!("Số quân: {} / {}", player_units, MAX_PLAYER_UNITS));
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
) {
    for (interaction, bench_btn) in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            if selected.unit_class == Some(bench_btn.0) {
                selected.unit_class = None;
            } else {
                selected.unit_class = Some(bench_btn.0);
            }
        }
    }
}

pub fn handle_speed_toggle(
    mut speed: ResMut<BattleSpeed>,
    mut buttons: Query<&Interaction, (Changed<Interaction>, With<SpeedToggleButton>)>,
    mut text_query: Query<&mut Text, With<SpeedText>>,
) {
    for interaction in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            if speed.multiplier == 1.0 {
                speed.multiplier = 2.0;
            } else {
                speed.multiplier = 1.0;
            }
            for mut text in text_query.iter_mut() {
                *text = Text::new(format!("Tốc độ: {:.0}x", speed.multiplier));
            }
        }
    }
}

pub fn handle_start_battle_button(
    units: Query<(&Unit, &UnitStats), Without<DeadUnit>>,
    mut buttons: Query<&Interaction, (Changed<Interaction>, With<StartBattleButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if *current_state.get() != GameState::Placement {
        return;
    }

    for interaction in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            let player_count = units
                .iter()
                .filter(|(u, _)| u.faction == Faction::Player)
                .count();
            if player_count > 0 {
                next_state.set(GameState::Battle);
            }
        }
    }
}

pub fn handle_clear_button(
    mut commands: Commands,
    mut buttons: Query<&Interaction, (Changed<Interaction>, With<ClearBoardButton>)>,
    units: Query<(Entity, &Unit)>,
    current_state: Res<State<GameState>>,
) {
    if *current_state.get() != GameState::Placement {
        return;
    }

    for interaction in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            for (entity, unit) in units.iter() {
                if unit.faction == Faction::Player {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

pub fn handle_preset_button(
    mut commands: Commands,
    assets_3d: Res<Game3dAssets>,
    mut buttons: Query<&Interaction, (Changed<Interaction>, With<PresetButton>)>,
    units: Query<(Entity, &Unit)>,
    current_state: Res<State<GameState>>,
) {
    if *current_state.get() != GameState::Placement {
        return;
    }

    for interaction in buttons.iter_mut() {
        if *interaction == Interaction::Pressed {
            for (entity, unit) in units.iter() {
                if unit.faction == Faction::Player {
                    commands.entity(entity).despawn_recursive();
                }
            }

            // 2 Knights in frontline (col 2, row 0 and 2)
            spawn_unit(
                &mut commands,
                &assets_3d,
                UnitClass::Knight,
                Faction::Player,
                2,
                0,
            );
            spawn_unit(
                &mut commands,
                &assets_3d,
                UnitClass::Knight,
                Faction::Player,
                2,
                2,
            );
            // 1 Assassin in frontline center (col 2, row 1)
            spawn_unit(
                &mut commands,
                &assets_3d,
                UnitClass::Assassin,
                Faction::Player,
                2,
                1,
            );
            // 1 Archer in backline (col 0, row 0)
            spawn_unit(
                &mut commands,
                &assets_3d,
                UnitClass::Archer,
                Faction::Player,
                0,
                0,
            );
            // 1 Cleric in backline (col 0, row 2)
            spawn_unit(
                &mut commands,
                &assets_3d,
                UnitClass::Cleric,
                Faction::Player,
                0,
                2,
            );
        }
    }
}

pub fn handle_tile_mouse_placement(
    mut commands: Commands,
    assets_3d: Res<Game3dAssets>,
    mouse: Res<ButtonInput<MouseButton>>,
    hovered: Res<HoveredTile>,
    selected: Res<SelectedBenchUnit>,
    units: Query<(Entity, &Unit, &GridPos), Without<DeadUnit>>,
    current_state: Res<State<GameState>>,
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
            if let Some((old_ent, _, _)) = existing_on_tile {
                commands.entity(old_ent).despawn_recursive();
                spawn_unit(
                    &mut commands,
                    &assets_3d,
                    unit_class,
                    Faction::Player,
                    target_col,
                    target_row,
                );
            } else {
                let current_count = units
                    .iter()
                    .filter(|(_, u, _)| u.faction == Faction::Player)
                    .count();
                if current_count < MAX_PLAYER_UNITS {
                    spawn_unit(
                        &mut commands,
                        &assets_3d,
                        unit_class,
                        Faction::Player,
                        target_col,
                        target_row,
                    );
                }
            }
        }
    }

    if mouse.just_pressed(MouseButton::Right) {
        if let Some((old_ent, _, _)) = existing_on_tile {
            commands.entity(old_ent).despawn_recursive();
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
                "{} {} [{}] - HP: {:.0}/{:.0} | Công: {:.0} | Giáp: {:.0} | Tốc: {:.0} — {}",
                unit.class.icon(),
                unit.class.name_vi(),
                if unit.faction == Faction::Player {
                    "Phe Ta"
                } else {
                    "Phe Địch"
                },
                stats.hp,
                stats.max_hp,
                stats.atk,
                stats.def,
                stats.speed,
                unit.class.description()
            ));
            return;
        }
    }

    if let Some(class) = selected.unit_class {
        let stats = class.base_stats();
        *text = Text::new(format!(
            "Đang chọn: {} {} - HP: {:.0} | Công: {:.0} | Giáp: {:.0} | Tốc: {:.0} — {}",
            class.icon(),
            class.name_vi(),
            stats.hp,
            stats.atk,
            stats.def,
            stats.speed,
            class.description()
        ));
        return;
    }

    *text = Text::new("");
}

pub fn setup_stage_enemies(
    mut commands: Commands,
    assets_3d: Res<Game3dAssets>,
    stage: Res<CurrentStage>,
    units: Query<(Entity, &Unit)>,
    mut title_query: Query<&mut Text, (With<StageTitleText>, Without<StageDescText>)>,
    mut desc_query: Query<&mut Text, With<StageDescText>>,
) {
    for (entity, unit) in units.iter() {
        if unit.faction == Faction::Enemy {
            commands.entity(entity).despawn_recursive();
        }
    }

    let stage_def = get_stage_def(stage.stage_idx);
    for mut text in title_query.iter_mut() {
        *text = Text::new(stage_def.title);
    }
    for mut text in desc_query.iter_mut() {
        *text = Text::new(stage_def.description);
    }

    for enemy in stage_def.enemies {
        spawn_unit(
            &mut commands,
            &assets_3d,
            enemy.unit_class,
            Faction::Enemy,
            enemy.col,
            enemy.row,
        );
    }
}

pub fn reset_player_units_for_placement(
    mut commands: Commands,
    assets_3d: Res<Game3dAssets>,
    mut units: Query<(Entity, &Unit, &GridPos, &mut Transform, &mut Visibility), Without<DeadUnit>>,
    dead_units: Query<(Entity, &Unit, &GridPos), With<DeadUnit>>,
) {
    use crate::board::grid_to_world_pos;
    for (_, unit, grid, mut transform, mut vis) in units.iter_mut() {
        if unit.faction == Faction::Player {
            let pos = grid_to_world_pos(grid.col, grid.row, Faction::Player);
            transform.translation = pos;
            *vis = Visibility::Inherited;
        }
    }

    for (entity, unit, grid) in dead_units.iter() {
        if unit.faction == Faction::Player {
            commands.entity(entity).despawn_recursive();
            spawn_unit(
                &mut commands,
                &assets_3d,
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
            BackgroundColor(Color::srgba(0.05, 0.15, 0.08, 0.88)),
            ResultUiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("CHIẾN THẮNG!"),
                TextFont {
                    font_size: 44.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.88, 0.2)),
            ));

            parent.spawn((
                Text::new("Bạn đã tiêu diệt toàn bộ đội hình đối phương trên sàn đấu!"),
                TextFont {
                    font_size: 16.0,
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
                            Text::new("MÀN TIẾP THEO >>"),
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
                            Text::new("Chơi lại màn này"),
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
            BackgroundColor(Color::srgba(0.2, 0.05, 0.08, 0.88)),
            ResultUiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("THẤT BẠI!"),
                TextFont {
                    font_size: 44.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
            ));

            parent.spawn((
                Text::new(
                    "Toàn bộ anh hùng đã ngã xuống! Hãy điều chỉnh vị trí hoặc dàn trận lại.",
                ),
                TextFont {
                    font_size: 16.0,
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
                    Text::new("THỬ LẠI <<"),
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
) {
    for interaction in next_btn.iter() {
        if *interaction == Interaction::Pressed {
            stage.stage_idx += 1;
            next_state.set(GameState::Placement);
        }
    }

    for interaction in retry_btn.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Placement);
        }
    }
}
