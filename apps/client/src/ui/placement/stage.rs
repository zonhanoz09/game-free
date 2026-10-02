use super::*;

pub fn setup_stage_enemies(
    mut commands: Commands,
    textures: Res<GameTextures>,
    stage: Res<CurrentStage>,
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
            *text = Text::new(format!("⚔️ Online PvP Arena - Round #{}", pvp_mgr.round));
        }
        for mut text in desc_query.iter_mut() {
            *text = Text::new(format!(
                "Room: {} | You: {} ({} HP) vs Opponent: {} ({} HP)",
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
                let board_positions = [(2, 1), (2, 0), (1, 2), (0, 1), (1, 0)];
                for (idx, card) in player_deck.cards.iter().enumerate() {
                    let unit_class = match card.hero_class.as_str() {
                        "Archer" => UnitClass::Archer,
                        "Mage" => UnitClass::Mage,
                        "Assassin" => UnitClass::Assassin,
                        "Cleric" => UnitClass::Cleric,
                        _ => UnitClass::Knight,
                    };
                    if idx < 3 {
                        let (col, row) = board_positions[idx % board_positions.len()];
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
                        let slot = (idx - 3).min(5);
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
    for (entity, unit) in units.iter() {
        if unit.faction == Faction::Enemy {
            if let Some(e) = commands.get_entity(entity) {
                e.despawn_recursive();
            }
        }
    }

    let stage_def = get_stage_def(stage.stage_idx);
    info!(
        "[STAGE] Loaded Stage #{}: {} - {}",
        stage.stage_idx, stage_def.title, stage_def.description
    );
    for mut text in title_query.iter_mut() {
        *text = Text::new(format!("🤖 AI - {}", stage_def.title));
    }
    for mut text in desc_query.iter_mut() {
        *text = Text::new(format!(
            "{} | Đội hình đã lưu của bạn sẽ được dùng cho trận AI.",
            stage_def.description
        ));
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

    // Pre-spawn starter squad for player from owned deck
    let player_count = units
        .iter()
        .filter(|(_, u)| u.faction == Faction::Player)
        .count();
    if player_count == 0 {
        if !player_deck.cards.is_empty() {
            let board_positions = [(2, 1), (2, 0), (1, 2), (0, 1), (1, 0)];
            for (idx, card) in player_deck.cards.iter().enumerate() {
                let unit_class = match card.hero_class.as_str() {
                    "Archer" => UnitClass::Archer,
                    "Mage" => UnitClass::Mage,
                    "Assassin" => UnitClass::Assassin,
                    "Cleric" => UnitClass::Cleric,
                    _ => UnitClass::Knight,
                };
                if idx < 3 {
                    let (col, row) = board_positions[idx % board_positions.len()];
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
                    let slot = (idx - 3).min(5);
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
