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
    time: Res<Time>,
    units: Query<(&Unit, &UnitStats), (With<GridPos>, Without<DeadUnit>)>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut economy: ResMut<PlayerEconomy>,
    mut rng: ResMut<BattleRng>,
    adapter: Res<BattleSimulationAdapter>,
    pvp_mgr: Res<crate::net::PvpManager>,
    mut pvp_finished_events: EventWriter<SendPvpBattleFinishedEvent>,
    mut battle_ended: Local<bool>,
    mut battle_duration: Local<f32>,
    mut pvp_wait_timer: Local<f32>,
) {
    if *current_state.get() != GameState::Battle {
        *battle_ended = false;
        *battle_duration = 0.0;
        *pvp_wait_timer = 0.0;
        return;
    }

    *battle_duration += time.delta_secs();

    if *battle_ended {
        if pvp_mgr.active {
            *pvp_wait_timer += time.delta_secs();
            if *pvp_wait_timer >= 8.0 && *pvp_wait_timer - time.delta_secs() < 8.0 {
                warn!(
                    "[PVP WATCHDOG] 8s elapsed without server round settlement. Resending BattleFinished for round {}",
                    pvp_mgr.round
                );
                pvp_finished_events.send(SendPvpBattleFinishedEvent {
                    winner_role: "draw".to_string(),
                    player_survivors: 0,
                    room_code: Some(pvp_mgr.room_code.clone()),
                    round: Some(pvp_mgr.round),
                });
            } else if *pvp_wait_timer >= 16.0 {
                warn!("[PVP WATCHDOG] 16s timeout waiting for server. Forcing transition to Placement!");
                *battle_ended = false;
                *pvp_wait_timer = 0.0;
                next_state.set(GameState::Placement);
            }
        }
        return;
    }

    // Force timeout resolve if a battle exceeds 45 seconds to prevent permanent locks
    let force_timeout_resolve = *battle_duration > 45.0;
    if force_timeout_resolve {
        warn!(
            "[BATTLE WATCHDOG] Battle duration reached {:.1}s. Forcing resolution to prevent infinite battle!",
            *battle_duration
        );
    }

    if let Some(winner) = adapter.settled_winner {
        let is_draw = winner.is_none();
        let is_victory = matches!(winner, Some(game_logic::TeamSide::Attacker));
        *battle_ended = true;

        if pvp_mgr.active {
            let role_winner = if is_draw {
                "draw".to_string()
            } else if is_victory {
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
            info!(
                "[PVP BATTLE FINISHED] Settled via sim adapter: Winner={}, Role={}, Survivors={}, Round={}",
                role_winner, pvp_mgr.role, survivors, pvp_mgr.round
            );
            pvp_finished_events.send(SendPvpBattleFinishedEvent {
                winner_role: role_winner,
                player_survivors: survivors,
                room_code: Some(pvp_mgr.room_code.clone()),
                round: Some(pvp_mgr.round),
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
    let mut player_hp_sum = 0.0;
    let mut enemy_hp_sum = 0.0;

    for (unit, stats) in units.iter() {
        if stats.hp > 0.0 {
            match unit.faction {
                Faction::Player => {
                    alive_player += 1;
                    player_hp_sum += stats.hp;
                }
                Faction::Enemy => {
                    alive_enemy += 1;
                    enemy_hp_sum += stats.hp;
                }
            }
        }
    }

    if (alive_enemy == 0 && alive_player > 0)
        || (force_timeout_resolve
            && (alive_player > alive_enemy
                || (alive_player == alive_enemy && player_hp_sum >= enemy_hp_sum)))
    {
        *battle_ended = true;
        info!("==================== [ROUND VICTORY] ====================");
        info!(
            "[VICTORY] Battle ended with victory! Surviving player heroes: {}, enemy: {} (timeout: {})",
            alive_player, alive_enemy, force_timeout_resolve
        );
        sound_events.send(PlaySoundEvent(SoundEffect::Victory));

        if pvp_mgr.active {
            info!(
                "[PVP ROUND VICTORY] Sending BattleFinished to server. Surviving heroes: {}",
                alive_player.max(1)
            );
            pvp_finished_events.send(SendPvpBattleFinishedEvent {
                winner_role: pvp_mgr.role.clone(),
                player_survivors: alive_player.max(1),
                room_code: Some(pvp_mgr.room_code.clone()),
                round: Some(pvp_mgr.round),
            });
            economy.apply_round_income(true, &mut rng);
        } else {
            economy.apply_round_income(true, &mut rng);
            next_state.set(GameState::Victory);
        }
    } else if (alive_player == 0 && alive_enemy > 0)
        || (force_timeout_resolve
            && (alive_enemy > alive_player
                || (alive_player == alive_enemy && enemy_hp_sum > player_hp_sum)))
    {
        *battle_ended = true;
        info!("==================== [ROUND DEFEAT] ====================");
        info!(
            "[DEFEAT] Battle ended with defeat! Surviving enemies: {}, player: {} (timeout: {})",
            alive_enemy, alive_player, force_timeout_resolve
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
                alive_enemy.max(1)
            );
            pvp_finished_events.send(SendPvpBattleFinishedEvent {
                winner_role: opp_role,
                player_survivors: alive_enemy.max(1),
                room_code: Some(pvp_mgr.room_code.clone()),
                round: Some(pvp_mgr.round),
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
            pvp_finished_events.send(SendPvpBattleFinishedEvent {
                winner_role: "draw".to_string(),
                player_survivors: 0,
                room_code: Some(pvp_mgr.room_code.clone()),
                round: Some(pvp_mgr.round),
            });
            economy.apply_round_income(false, &mut rng);
        } else {
            economy.apply_round_income(false, &mut rng);
            next_state.set(GameState::Defeat);
        }
    }
}
