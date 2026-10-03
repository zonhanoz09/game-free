use super::*;

pub fn show_victory_ui(mut commands: Commands) {
    #[cfg(target_arch = "wasm32")]
    crate::net::rust_to_js_pvp(r#"{"type":"PVE_VICTORY","gold":40}"#);
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
                Text::new("CHIẾN THẮNG!"),
                TextFont {
                    font_size: 46.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.88, 0.2)),
            ));

            parent.spawn((
                Text::new("Toàn bộ quân địch trên sa trường đã bị tiêu diệt!"),
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
                            Text::new("ẢI KẾ TIẾP >>"),
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
                            Text::new("Đấu Lại"),
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
    #[cfg(target_arch = "wasm32")]
    crate::net::rust_to_js_pvp(r#"{"type":"PVE_DEFEAT","gold":10}"#);
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
                Text::new("THẤT BẠI!"),
                TextFont {
                    font_size: 46.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
            ));

            parent.spawn((
                Text::new("Đội hình của bạn đã gục ngã! Hãy sắp xếp lại trận pháp và thử lại."),
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
    mut sound_events: EventWriter<PlaySoundEvent>,
) {
    for interaction in next_btn.iter() {
        if *interaction == Interaction::Pressed {
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
            stage.stage_idx += 1;
            info!(
                "[STAGE] Advancing to Next Stage (Index: {})",
                stage.stage_idx
            );
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
        *text = Text::new(format!("💰 {} Vàng (+{} kế)", economy.gold, 5 + interest));
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
            *txt = Text::new("⚠️ Cần ít nhất 2 Vàng để đổi thẻ tướng mới!".to_string());
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
        info!(
            "[SHOP] Shop lock toggled -> Locked: {}",
            economy.shop_locked
        );
        sound_events.send(PlaySoundEvent(SoundEffect::Click));
    }
}

pub fn hide_placement_ui_on_battle(mut query: Query<&mut Visibility, With<PlacementUiRoot>>) {
    for mut vis in query.iter_mut() {
        *vis = Visibility::Hidden;
    }
}

pub fn show_placement_ui_on_placement(mut query: Query<&mut Visibility, With<PlacementUiRoot>>) {
    for mut vis in query.iter_mut() {
        *vis = Visibility::Inherited;
    }
}
