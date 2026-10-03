use super::*;

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
            } else {
                let occupied_slots: Vec<usize> =
                    bench_units.iter().map(|(_, _, b, _)| b.slot).collect();
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
                                "Đã chiêu mộ {} ({} Vàng) -> Hàng Chờ #{} [Phím {}]",
                                bought_class.name(),
                                cost,
                                slot + 1,
                                slot_idx + 1
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

    // 2. Sell unit with S, Delete, or Backspace
    if keyboard.just_pressed(KeyCode::KeyS)
        || keyboard.just_pressed(KeyCode::Delete)
        || keyboard.just_pressed(KeyCode::Backspace)
    {
        let mut target_to_sell: Option<(Entity, UnitClass, i32, u8)> = None;

        // A. Hovered board tile
        if let Some(tile) = &hovered.tile {
            if tile.faction == Faction::Player {
                if let Some((ent, unit, _, star)) = board_units.iter().find(|(_, _, g, _)| {
                    g.col == tile.col && g.row == tile.row && g.faction == Faction::Player
                }) {
                    target_to_sell =
                        Some((ent, unit.class, refund_amount(unit.class, star.0), star.0));
                }
            }
        }

        // B. Hovered bench slot
        if target_to_sell.is_none() {
            if let Some(slot) = hovered.bench_slot {
                if let Some((ent, unit, _, star)) =
                    bench_units.iter().find(|(_, _, b, _)| b.slot == slot)
                {
                    target_to_sell =
                        Some((ent, unit.class, refund_amount(unit.class, star.0), star.0));
                }
            }
        }

        // C. Currently selected unit
        if target_to_sell.is_none() {
            if let Some(sel_ent) = selected.entity {
                if let Some((ent, unit, _, star)) =
                    board_units.iter().find(|(e, _, _, _)| *e == sel_ent)
                {
                    target_to_sell =
                        Some((ent, unit.class, refund_amount(unit.class, star.0), star.0));
                } else if let Some((ent, unit, _, star)) =
                    bench_units.iter().find(|(e, _, _, _)| *e == sel_ent)
                {
                    target_to_sell =
                        Some((ent, unit.class, refund_amount(unit.class, star.0), star.0));
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
            info!(
                "[SELL] Hotkey sold {}★ {} for +{}G -> Total: {}G",
                star,
                class.name(),
                refund,
                economy.gold
            );
            if let Ok(mut txt) = tooltip.get_single_mut() {
                *txt = Text::new(format!(
                    "Đã bán {}★ {} nhận +{} Vàng! [Phím: S]",
                    star,
                    class.name(),
                    refund
                ));
            }
        }
    }
}
