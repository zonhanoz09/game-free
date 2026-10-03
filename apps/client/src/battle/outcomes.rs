use super::*;

pub fn check_unit_deaths(
    mut commands: Commands,
    mut units: Query<
        (Entity, &Unit, &UnitStats, &mut Visibility, &mut Transform),
        (With<Unit>, Without<DeadUnit>),
    >,
) {
    for (entity, unit, stats, mut vis, mut transform) in units.iter_mut() {
        if stats.hp <= 0.0 {
            info!(
                "[DEATH] {:?} {:?} has fallen in battle!",
                unit.faction, unit.class
            );
            transform.translation.y = -9999.0;
            *vis = Visibility::Hidden;
            if let Some(mut e) = commands.get_entity(entity) { e.insert(DeadUnit); }
        }
    }
}

pub fn check_battle_end(
    units: Query<(&Unit, &UnitStats), (With<GridPos>, Without<DeadUnit>)>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut economy: ResMut<PlayerEconomy>,
    mut rng: ResMut<BattleRng>,
    adapter: Res<BattleSimulationAdapter>,
    pvp_mgr: Res<crate::net::PvpManager>,
    mut battle_ended: Local<bool>,
) {
    if *current_state.get() != GameState::Battle {
        *battle_ended = false;
        return;
    }

    if *battle_ended {
        return;
    }

    if let Some(winner) = adapter.settled_winner {
        let is_victory = match winner {
            Some(game_logic::TeamSide::Attacker) => true,
            _ => false,
        };
        *battle_ended = true;

        if pvp_mgr.active {
            let role_winner = if is_victory {
                pvp_mgr.role.clone()
            } else if pvp_mgr.role == "host" {
                "guest".to_string()
            } else {
                "host".to_string()
            };
            let survivors = units
                .iter()
                .filter(|(u, s)| {
                    if is_victory {
                        u.faction == Faction::Player && s.hp > 0.0
                    } else {
                        u.faction == Faction::Enemy && s.hp > 0.0
                    }
                })
                .count()
                .max(1);
            crate::net::send_pvp_message(&crate::net::PvpMessage::BattleFinished {
                winner_role: role_winner,
                player_survivors: survivors,
            });
            if is_victory {
                sound_events.send(PlaySoundEvent(SoundEffect::Victory));
                economy.apply_round_income(true, &mut rng);
            } else {
                sound_events.send(PlaySoundEvent(SoundEffect::Defeat));
                economy.apply_round_income(false, &mut rng);
            }
            return;
        }

        if is_victory {
            info!("[VICTORY] Battle resolved in victory via simulation adapter!");
            sound_events.send(PlaySoundEvent(SoundEffect::Victory));
            economy.apply_round_income(true, &mut rng);
            next_state.set(GameState::Victory);
            return;
        } else {
            info!("[DEFEAT] Battle resolved in defeat via simulation adapter!");
            sound_events.send(PlaySoundEvent(SoundEffect::Defeat));
            economy.apply_round_income(false, &mut rng);
            next_state.set(GameState::Defeat);
            return;
        }
    }

    let mut alive_player = 0;
    let mut alive_enemy = 0;

    for (unit, stats) in units.iter() {
        if stats.hp > 0.0 {
            match unit.faction {
                Faction::Player => alive_player += 1,
                Faction::Enemy => alive_enemy += 1,
            }
        }
    }

    if alive_enemy == 0 && alive_player > 0 {
        *battle_ended = true;
        info!("==================== [ROUND VICTORY] ====================");
        info!(
            "[VICTORY] All enemies defeated! Surviving player heroes: {}",
            alive_player
        );
        sound_events.send(PlaySoundEvent(SoundEffect::Victory));

        if pvp_mgr.active {
            info!(
                "[PVP ROUND VICTORY] Sending BattleFinished to server. Surviving heroes: {}",
                alive_player
            );
            crate::net::send_pvp_message(&crate::net::PvpMessage::BattleFinished {
                winner_role: pvp_mgr.role.clone(),
                player_survivors: alive_player,
            });
            economy.apply_round_income(true, &mut rng);
        } else {
            economy.apply_round_income(true, &mut rng);
            next_state.set(GameState::Victory);
        }
    } else if alive_player == 0 && alive_enemy > 0 {
        *battle_ended = true;
        info!("==================== [ROUND DEFEAT] ====================");
        info!(
            "[DEFEAT] All player heroes were eliminated! Surviving enemies: {}",
            alive_enemy
        );
        sound_events.send(PlaySoundEvent(SoundEffect::Defeat));

        if pvp_mgr.active {
            let opp_role = if pvp_mgr.role == "host" {
                "guest".to_string()
            } else {
                "host".to_string()
            };
            info!(
                "[PVP ROUND DEFEAT] Sending BattleFinished to server. Opponent won with {} survivors.",
                alive_enemy
            );
            crate::net::send_pvp_message(&crate::net::PvpMessage::BattleFinished {
                winner_role: opp_role,
                player_survivors: alive_enemy,
            });
            economy.apply_round_income(false, &mut rng);
        } else {
            economy.apply_round_income(false, &mut rng);
            next_state.set(GameState::Defeat);
        }
    } else if alive_player == 0 && alive_enemy == 0 {
        *battle_ended = true;
        info!("==================== [ROUND DRAW] ====================");
        sound_events.send(PlaySoundEvent(SoundEffect::Defeat));

        if pvp_mgr.active {
            crate::net::send_pvp_message(&crate::net::PvpMessage::BattleFinished {
                winner_role: "draw".to_string(),
                player_survivors: 0,
            });
            economy.apply_round_income(false, &mut rng);
        } else {
            economy.apply_round_income(false, &mut rng);
            next_state.set(GameState::Defeat);
        }
    }
}
