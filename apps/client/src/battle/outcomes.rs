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
            commands.entity(entity).insert(DeadUnit);
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
    pvp_mgr: Res<crate::net::PvpManager>,
) {
    if *current_state.get() != GameState::Battle {
        return;
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
        info!("==================== [ROUND VICTORY] ====================");
        info!(
            "[VICTORY] All enemies defeated! Surviving player heroes: {}",
            alive_player
        );
        sound_events.send(PlaySoundEvent(SoundEffect::Victory));
        if pvp_mgr.active {
            crate::net::send_pvp_message(&crate::net::PvpMessage::BattleFinished {
                winner_role: pvp_mgr.role.clone(),
                player_survivors: alive_player,
            });
        }
        economy.apply_round_income(true, &mut rng);
        next_state.set(GameState::Victory);
    } else if alive_player == 0 {
        info!("==================== [ROUND DEFEAT] ====================");
        info!(
            "[DEFEAT] All player heroes were eliminated! Surviving enemies: {}",
            alive_enemy
        );
        sound_events.send(PlaySoundEvent(SoundEffect::Defeat));
        if pvp_mgr.active {
            let opp_role = if pvp_mgr.role == "host" {
                "guest"
            } else {
                "host"
            };
            crate::net::send_pvp_message(&crate::net::PvpMessage::BattleFinished {
                winner_role: opp_role.to_string(),
                player_survivors: 0,
            });
        }
        economy.apply_round_income(false, &mut rng);
        next_state.set(GameState::Defeat);
    }
}
