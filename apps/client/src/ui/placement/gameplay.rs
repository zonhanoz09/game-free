use super::*;

pub fn update_unit_count_ui(
    units: Query<&Unit, Without<DeadUnit>>,
    mut text_query: Query<&mut Text, With<UnitCountText>>,
) {
    let player_units = units
        .iter()
        .filter(|u| u.faction == Faction::Player)
        .count();
    for mut text in text_query.iter_mut() {
        *text = Text::new(format!("Tướng: {} / {}", player_units, MAX_PLAYER_UNITS));
    }
}

pub fn update_hero_inspection_system(
    textures: Res<GameTextures>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<crate::board::MainCamera2d>>,
    hovered: Res<HoveredTile>,
    selected: Res<SelectedUnitState>,
    board_units: Query<(&Unit, &GridPos, &UnitStats, &Transform), Without<DeadUnit>>,
    bench_units: Query<(&Unit, &BenchPos, &UnitStats, &Transform), Without<DeadUnit>>,
    mut inspector_root: Query<&mut Node, (With<InspectorRoot>, Without<InspectStatBar>)>,
    mut bar_query: Query<(&InspectStatBar, &mut Node), Without<InspectorRoot>>,
    mut avatar_img: Query<&mut ImageNode, With<InspectHeroAvatar>>,
    mut text_queries: ParamSet<(
        Query<(&InspectHeader, &mut Text, Option<&mut TextColor>)>,
        Query<(&InspectStatText, &mut Text)>,
        Query<(&InspectSkill, &mut Text)>,
    )>,
) {
    let mut inspected: Option<(UnitClass, Faction, UnitStats)> = None;
    let mut inspected_world_pos: Option<Vec2> = None;

    if let Some(tile) = &hovered.tile {
        if let Some((unit, _, stats, transform)) = board_units.iter().find(|(_, g, _, _)| {
            g.col == tile.col && g.row == tile.row && g.faction == tile.faction
        }) {
            inspected = Some((unit.class, unit.faction, *stats));
            inspected_world_pos = Some(transform.translation.truncate());
        }
    }

    if inspected.is_none() {
        if let Some(slot) = hovered.bench_slot {
            if let Some((unit, _, stats, transform)) =
                bench_units.iter().find(|(_, b, _, _)| b.slot == slot)
            {
                inspected = Some((unit.class, Faction::Player, *stats));
                inspected_world_pos = Some(transform.translation.truncate());
            }
        }
    }

    if inspected.is_none() {
        if let Some(sel_ent) = selected.entity {
            if let Ok((unit, _, stats, transform)) = board_units.get(sel_ent) {
                inspected = Some((unit.class, unit.faction, *stats));
                inspected_world_pos = Some(transform.translation.truncate());
            } else if let Ok((unit, _, stats, transform)) = bench_units.get(sel_ent) {
                inspected = Some((unit.class, Faction::Player, *stats));
                inspected_world_pos = Some(transform.translation.truncate());
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

    // Dynamically position the card inspector BESIDE the hovered unit!
    if let (Some(world_pos), Ok((camera, cam_transform)), Ok(window)) = (
        inspected_world_pos,
        camera_q.get_single(),
        windows.get_single(),
    ) {
        if let Ok(viewport_pos) = camera.world_to_viewport(cam_transform, world_pos.extend(0.0)) {
            let win_w = window.width();
            let win_h = window.height();

            if viewport_pos.x < win_w * 0.5 {
                // Unit is on the left side -> show inspector panel to the right of the unit
                root_node.left = Val::Px((viewport_pos.x + 50.0).clamp(10.0, win_w - 300.0));
                root_node.right = Val::Auto;
            } else {
                // Unit is on the right side -> show inspector panel to the left of the unit
                root_node.left = Val::Px((viewport_pos.x - 300.0).clamp(10.0, win_w - 300.0));
                root_node.right = Val::Auto;
            }

            root_node.top = Val::Px((viewport_pos.y - 120.0).clamp(65.0, win_h - 380.0));
            root_node.bottom = Val::Auto;
        }
    }

    if let Ok(mut img) = avatar_img.get_single_mut() {
        img.image = textures.get_unit_texture(class);
    }

    for (header, mut txt, text_col) in text_queries.p0().iter_mut() {
        match header.0 {
            InspectHeaderField::Name => {
                *txt = Text::new(class.name());
            }
            InspectHeaderField::Faction => {
                if let Some(mut col) = text_col {
                    match faction {
                        Faction::Player => {
                            *txt = Text::new("QUÂN TA");
                            col.0 = Color::srgb(0.3, 0.7, 1.0);
                        }
                        Faction::Enemy => {
                            *txt = Text::new("QUÂN ĐỊCH");
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

    for (stat_txt, mut txt) in text_queries.p1().iter_mut() {
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
                *txt = Text::new(format!("{:.0}", stats.def));
            }
            InspectStatType::Spd => {
                *txt = Text::new(format!("{:.0}", stats.speed));
            }
        }
    }

    for (sk, mut txt) in text_queries.p2().iter_mut() {
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
    textures: Res<GameTextures>,
    economy: Res<PlayerEconomy>,
    mut card_buttons: Query<(&ShopCard, &mut BackgroundColor, &mut BorderColor)>,
    mut avatar_images: Query<(&ShopCardAvatar, &mut ImageNode, &mut Visibility)>,
    mut lock_button: Query<
        (&mut BackgroundColor, &mut BorderColor),
        (With<ShopLockToggle>, Without<ShopCard>),
    >,
    mut text_queries: ParamSet<(
        Query<(&ShopCardName, &mut Text, &mut TextColor)>,
        Query<(&ShopCardCost, &mut Text)>,
        Query<&mut Text, With<ShopLockText>>,
    )>,
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

    for (avatar, mut img, mut vis) in avatar_images.iter_mut() {
        let idx = avatar.0;
        if let Some(class) = economy.shop_slots[idx] {
            img.image = textures.get_unit_texture(class);
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    for (name_comp, mut txt, mut col) in text_queries.p0().iter_mut() {
        let idx = name_comp.0;
        if let Some(class) = economy.shop_slots[idx] {
            *txt = Text::new(class.name());
            col.0 = Color::WHITE;
        } else {
            *txt = Text::new("[ĐÃ MUA]");
            col.0 = Color::srgb(0.45, 0.48, 0.52);
        }
    }

    for (cost_comp, mut txt) in text_queries.p1().iter_mut() {
        let idx = cost_comp.0;
        if let Some(class) = economy.shop_slots[idx] {
            *txt = Text::new(format!("{} Vàng | {}", unit_cost(class), class.role_abbr()));
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

    if let Ok(mut txt) = text_queries.p2().get_single_mut() {
        if economy.shop_locked {
            *txt = Text::new("🔒 Đã Khóa [E]");
        } else {
            *txt = Text::new("🔓 Khóa [E]");
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
                    info!(
                        "[SHOP] Cannot buy {}: Not enough gold (Have: {}G, Need: {}G)",
                        class.name(),
                        economy.gold,
                        cost
                    );
                    if let Ok(mut txt) = tooltip.get_single_mut() {
                        *txt = Text::new(format!(
                            "⚠️ Không đủ vàng! Cần {} Vàng, hiện có {} Vàng.",
                            cost, economy.gold
                        ));
                    }
                    continue;
                }

                let occupied_slots: Vec<usize> = bench_units.iter().map(|b| b.slot).collect();
                let free_slot = (0..BENCH_SLOTS).find(|s| !occupied_slots.contains(s));

                if let Some(slot) = free_slot {
                    if let Some(bought_class) = economy.buy_slot(slot_idx) {
                        spawn_bench_unit(&mut commands, &textures, bought_class, slot, 1);
                        sound_events.send(PlaySoundEvent(SoundEffect::Click));
                        info!(
                            "[SHOP] Recruited {:?} for {}G -> placed on Reserve Bench Slot #{} (Remaining Gold: {}G)",
                            bought_class,
                            cost,
                            slot + 1,
                            economy.gold
                        );
                        if let Ok(mut txt) = tooltip.get_single_mut() {
                            *txt = Text::new(format!(
                                "Đã chiêu mộ {} ({} Vàng) -> Hàng Chờ #{}",
                                bought_class.name(),
                                cost,
                                slot + 1
                            ));
                        }
                    }
                } else {
                    info!("[SHOP] Reserve bench is full (6/6 slots occupied)!");
                    if let Ok(mut txt) = tooltip.get_single_mut() {
                        *txt = Text::new(
                            "⚠️ Hàng chờ đã đầy (6/6)! Hãy xuất trận hoặc bán bớt tướng."
                                .to_string(),
                        );
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
                *text = Text::new(format!("Tốc độ: {:.0}x", speed.multiplier));
            }
            info!("[UI] Battle speed toggled: {:.0}x", speed.multiplier);
        }
    }
}

pub fn handle_start_battle_button(
    mut commands: Commands,
    textures: Res<GameTextures>,
    player_deck: Res<crate::net::PlayerDeck>,
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
                        class: u.class.id_str().to_string(),
                        star_level: s.0,
                    });
                }
            }
            if lineup.is_empty() {
                spawn_unit(
                    &mut commands,
                    &textures,
                    UnitClass::Knight,
                    Faction::Player,
                    2,
                    0,
                );
                lineup.push(crate::net::PvpUnitData {
                    col: 2,
                    row: 0,
                    class: UnitClass::Knight.id_str().to_string(),
                    star_level: 1,
                });
            }
            pvp_mgr.is_ready = true;
            crate::net::send_pvp_message(&crate::net::PvpMessage::PlayerReady {
                lineup: lineup.clone(),
            });
            info!(
                "[PVP] Ready & Locked In! Sent lineup of {} heroes.",
                lineup.len()
            );
            return;
        }

        let player_count = units
            .iter()
            .filter(|(u, _, _, _)| u.faction == Faction::Player)
            .count();
        if player_count > 0 {
            info!(
                "[UI] ⚔️ Battle Start triggered! (Active player heroes on board: {})",
                player_count
            );
            next_state.set(GameState::Battle);
        } else {
            info!(
                "[UI] ⚔️ AI battle started with 0 units on board: restoring the saved formation first"
            );
            if player_deck.cards.is_empty() {
                spawn_unit(
                    &mut commands,
                    &textures,
                    UnitClass::Knight,
                    Faction::Player,
                    2,
                    0,
                );
                spawn_unit(
                    &mut commands,
                    &textures,
                    UnitClass::Archer,
                    Faction::Player,
                    0,
                    1,
                );
                spawn_unit(
                    &mut commands,
                    &textures,
                    UnitClass::Assassin,
                    Faction::Player,
                    2,
                    2,
                );
            } else {
                let board_positions = [(2, 1), (2, 0), (1, 2)];
                for (idx, card) in player_deck.cards.iter().take(3).enumerate() {
                    let (col, row) = board_positions[idx];
                    let unit_class = crate::net::general_unit_class(&card.hero_class);
                    crate::units::spawn_unit_ext_bonus_with_initiative(
                        &mut commands,
                        &textures,
                        unit_class,
                        Faction::Player,
                        col,
                        row,
                        card.star_level.max(1),
                        false,
                        card.hp_bonus,
                        card.atk_bonus,
                        card.initiative_bonus,
                    );
                }
            }
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
            *text = Text::new("⚔️ Xuất Trận [Space]");
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
    mut tooltip: Query<&mut Text, With<TooltipText>>,
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
            info!(
                "[UI] Squad cleared: {} heroes sold for +{}G -> Total Gold: {}G",
                refunded_count, total_refund, economy.gold
            );
            if let Ok(mut txt) = tooltip.get_single_mut() {
                *txt = Text::new(format!(
                    "🧹 Đã thu hồi toàn bộ {} tướng, hoàn lại +{} Vàng!",
                    refunded_count, total_refund
                ));
            }
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
    mut tooltip: Query<&mut Text, With<TooltipText>>,
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
                spawn_unit(
                    &mut commands,
                    &textures,
                    UnitClass::Knight,
                    Faction::Player,
                    2,
                    0,
                );
                spawn_unit(
                    &mut commands,
                    &textures,
                    UnitClass::Archer,
                    Faction::Player,
                    0,
                    1,
                );
                spawn_unit(
                    &mut commands,
                    &textures,
                    UnitClass::Assassin,
                    Faction::Player,
                    2,
                    2,
                );
                info!(
                    "[UI] Preset squad deployed (Cost: 7G) -> Remaining Gold: {}G",
                    economy.gold
                );
                if let Ok(mut txt) = tooltip.get_single_mut() {
                    *txt = Text::new("📋 Đã triển khai Đội Hình Mẫu (Chi phí: 7 Vàng)".to_string());
                }
            } else {
                info!(
                    "[UI] Cannot deploy preset squad: Need 7G (Current: {}G)",
                    economy.gold
                );
                if let Ok(mut txt) = tooltip.get_single_mut() {
                    *txt = Text::new(format!(
                        "⚠️ Không đủ vàng triển khai Đội Hình Mẫu! Cần 7 Vàng, hiện có {} Vàng.",
                        economy.gold
                    ));
                }
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
                if let Some((ent, unit, _, star)) = board_units.iter().find(|(_, _, g, _)| {
                    g.col == tile.col && g.row == tile.row && g.faction == Faction::Player
                }) {
                    let refund = refund_amount(unit.class, star.0);
                    economy.gold += refund;
                    if let Some(e_cmd) = commands.get_entity(ent) {
                        e_cmd.despawn_recursive();
                    }
                    if selected.entity == Some(ent) {
                        selected.clear();
                    }
                    sound_events.send(PlaySoundEvent(SoundEffect::Click));
                    info!(
                        "[SELL] Sold {}★ {} from Board for +{}G -> Total: {}G",
                        star.0,
                        unit.class.name(),
                        refund,
                        economy.gold
                    );
                    if let Ok(mut txt) = tooltip.get_single_mut() {
                        *txt = Text::new(format!(
                            "Đã bán {}★ {} nhận +{} Vàng!",
                            star.0,
                            unit.class.name(),
                            refund
                        ));
                    }
                    return;
                }
            }
        }

        if let Some(slot) = hovered.bench_slot {
            if let Some((ent, unit, _, star)) =
                bench_units.iter().find(|(_, _, b, _)| b.slot == slot)
            {
                let refund = refund_amount(unit.class, star.0);
                economy.gold += refund;
                if let Some(e_cmd) = commands.get_entity(ent) {
                    e_cmd.despawn_recursive();
                }
                if selected.entity == Some(ent) {
                    selected.clear();
                }
                sound_events.send(PlaySoundEvent(SoundEffect::Click));
                info!(
                    "[SELL] Sold {}★ {} from Bench #{} for +{}G -> Total: {}G",
                    star.0,
                    unit.class.name(),
                    slot + 1,
                    refund,
                    economy.gold
                );
                if let Ok(mut txt) = tooltip.get_single_mut() {
                    *txt = Text::new(format!(
                        "Đã bán {}★ {} nhận +{} Vàng!",
                        star.0,
                        unit.class.name(),
                        refund
                    ));
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
                        .find(|(_, _, g, _)| {
                            g.col == tile.col && g.row == tile.row && g.faction == Faction::Player
                        })
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
                                let p1 =
                                    grid_to_world_pos(target_g.col, target_g.row, Faction::Player);
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
                                let p_board =
                                    grid_to_world_pos(board_g.col, board_g.row, Faction::Player);
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
                                let p_board =
                                    grid_to_world_pos(board_g.col, board_g.row, Faction::Player);
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
                                    info!(
                                        "[DEPLOY] Board is full (5/5)! Cannot deploy another hero."
                                    );
                                    if let Ok(mut txt) = tooltip.get_single_mut() {
                                        *txt = Text::new("⚠️ Bàn cờ đã đầy (5/5 tướng)! Hãy hoán đổi vị trí với tướng đang xuất trận.".to_string());
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
                        info!(
                            "[MOVE] Placed hero on Board ({}, {})",
                            target_g.col, target_g.row
                        );
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
            "Đang chọn: {}★ {} ({}) - Chuột trái vào ô trống để đặt/đổi chỗ, Chuột phải để bán.",
            star,
            class.name(),
            if is_bench { "Hàng Chờ" } else { "Bàn Cờ" }
        ));
        return;
    }

    if let Some(tile) = &hovered.tile {
        if let Some((unit, _, stats, star)) = board_units.iter().find(|(_, g, _, _)| {
            g.col == tile.col && g.row == tile.row && g.faction == tile.faction
        }) {
            *text = Text::new(format!(
                "Tướng: {}★ {} [{}] - HP: {:.0}/{:.0} | ATK: {:.0} | DEF: {:.0} | SPD: {:.0} (Chuột phải để bán)",
                star.0,
                unit.class.name(),
                if unit.faction == Faction::Player {
                    "Quân Ta"
                } else {
                    "Quân Địch"
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

    if let Some(slot) = hovered.bench_slot {
        if let Some((unit, _, stats, star)) = bench_units.iter().find(|(_, b, _, _)| b.slot == slot)
        {
            *text = Text::new(format!(
                "Hàng Chờ #{}: {}★ {} - HP: {:.0}/{:.0} | ATK: {:.0} | DEF: {:.0} (Chuột trái để đặt/đổi, Chuột phải để bán)",
                slot + 1,
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

    *text = Text::new("💡 Nhấn vào thẻ tướng bên dưới để chiêu mộ. Chuột trái để điều động/hoán đổi vị trí. Chuột phải để bán tướng lấy vàng.".to_string());
}
