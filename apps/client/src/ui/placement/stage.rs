use super::*;

pub fn setup_stage_enemies(
    mut commands: Commands,
    textures: Res<GameTextures>,
    stage: Res<CurrentStage>,
    economy: Res<PlayerEconomy>,
    units: Query<(Entity, &Unit)>,
    mut title_query: Query<&mut Text, (With<StageTitleText>, Without<StageDescText>)>,
    mut desc_query: Query<&mut Text, (With<StageDescText>, Without<StageTitleText>)>,
    pvp_mgr: Res<crate::net::PvpManager>,
    player_deck: Res<crate::net::PlayerDeck>,
) {
    if pvp_mgr.active {
        for (entity, unit) in units.iter() {
            if unit.faction == Faction::Enemy {
                if let Some(e) = commands.get_entity(entity) {
                    e.despawn_recursive();
                }
            }
        }
        for mut text in title_query.iter_mut() {
            *text = Text::new(format!(
                "⚔️ Đấu Trường Online PvP - Hiệp #{}",
                pvp_mgr.round
            ));
        }
        for mut text in desc_query.iter_mut() {
            *text = Text::new(format!(
                "Phòng: {} | Bạn: {} ({} HP) vs Đối thủ: {} ({} HP)",
                pvp_mgr.room_code,
                pvp_mgr.player_name,
                pvp_mgr.player_hp,
                pvp_mgr.opponent_name,
                pvp_mgr.opponent_hp
            ));
        }
        let player_count = units
            .iter()
            .filter(|(_, u)| u.faction == Faction::Player)
            .count();
        if player_count == 0 {
            if !player_deck.cards.is_empty() {
                for (idx, card) in player_deck.cards.iter().enumerate() {
                    let unit_class = crate::net::general_unit_class(&card.hero_class);
                    if let Some(position) = card.position {
                        let (col, row) =
                            crate::net::formation_position_to_grid(position, Faction::Player);
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
                    } else {
                        let slot = idx.min(5);
                        crate::units::spawn_bench_unit_bonus(
                            &mut commands,
                            &textures,
                            unit_class,
                            slot,
                            card.star_level.max(1),
                            card.hp_bonus,
                            card.atk_bonus,
                        );
                    }
                }
            } else {
                spawn_unit(
                    &mut commands,
                    &textures,
                    UnitClass::Knight,
                    Faction::Player,
                    2,
                    1,
                );
            }
        }
        return;
    }

    // Single Player AI Match: Clear old enemy units
    for (entity, unit) in units.iter() {
        if unit.faction == Faction::Enemy {
            if let Some(e) = commands.get_entity(entity) {
                e.despawn_recursive();
            }
        }
    }

    // Dynamic AI opponent team derived from the cards currently available in the shop
    let mut ai_cards: Vec<UnitClass> = economy.shop_slots.iter().filter_map(|&slot| slot).collect();

    let pool = UnitClass::ALL;
    let mut pool_idx = (stage.stage_idx * 2) % pool.len();
    while ai_cards.len() < 3 {
        ai_cards.push(pool[pool_idx % pool.len()]);
        pool_idx += 1;
    }

    let is_boss_stage = stage.stage_idx % 5 == 0;
    let base_star = if stage.stage_idx >= 4 { 2 } else { 1 };

    let card_names = ai_cards
        .iter()
        .map(|c| c.name())
        .collect::<Vec<_>>()
        .join(", ");

    info!(
        "[STAGE] Loaded Dynamic Shop-Based Stage #{}: {} cards -> [{}]",
        stage.stage_idx,
        ai_cards.len(),
        card_names
    );

    for mut text in title_query.iter_mut() {
        *text = Text::new(format!("🤖 Đấu với AI - Vòng #{}", stage.stage_idx));
    }
    for mut text in desc_query.iter_mut() {
        *text = Text::new(format!(
            "AI xuất trận với {} tướng từ Cửa Hàng: {} | Đội hình 3x3",
            ai_cards.len(),
            card_names
        ));
    }

    // Tactical positioning for AI on 3x3 enemy board:
    // col 0 = Frontline (tank/warrior), col 1 = Midline, col 2 = Backline (snipers/mages)
    let mut front_row = 1;
    let mut mid_row = 0;
    let mut back_row = 0;

    for (idx, &unit_class) in ai_cards.iter().enumerate() {
        let (col, row) = if unit_class.is_melee() {
            let r = front_row;
            front_row = (front_row + 2) % 3;
            (0, r)
        } else if unit_class.is_healer() {
            let r = mid_row;
            mid_row = (mid_row + 1) % 3;
            (1, r)
        } else {
            let r = back_row;
            back_row = (back_row + 1) % 3;
            (2, r)
        };

        let is_boss = is_boss_stage && idx == 0;
        let star = if is_boss { 3 } else { base_star };

        spawn_unit_ext(
            &mut commands,
            &textures,
            unit_class,
            Faction::Enemy,
            col,
            row,
            star,
            is_boss,
        );
    }

    // Pre-spawn starter squad for player from owned deck
    let player_count = units
        .iter()
        .filter(|(_, u)| u.faction == Faction::Player)
        .count();
    if player_count == 0 {
        if !player_deck.cards.is_empty() {
            for (idx, card) in player_deck.cards.iter().enumerate() {
                let unit_class = crate::net::general_unit_class(&card.hero_class);
                if let Some(position) = card.position {
                    let (col, row) =
                        crate::net::formation_position_to_grid(position, Faction::Player);
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
                } else {
                    let slot = idx.min(5);
                    crate::units::spawn_bench_unit_bonus(
                        &mut commands,
                        &textures,
                        unit_class,
                        slot,
                        card.star_level.max(1),
                        card.hp_bonus,
                        card.atk_bonus,
                    );
                }
            }
        } else {
            spawn_unit(
                &mut commands,
                &textures,
                UnitClass::Knight,
                Faction::Player,
                2,
                1,
            );
        }
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
