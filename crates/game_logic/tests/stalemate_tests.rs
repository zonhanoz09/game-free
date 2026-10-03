use game_logic::{BattleState, BoardSlot, CombatEvent, HeroId, TeamSide, UnitState};

#[test]
fn test_high_defense_tank_stalemate_resolves() {
    let mut battle = BattleState::default();
    // High HP, very high defense, low attack -> without sudden death this would loop forever
    let tank_a = UnitState::new(
        1,
        TeamSide::Attacker,
        BoardSlot::new(TeamSide::Attacker, 1, 1).unwrap(),
        3000,
        5,
        2000,
        100,
    );
    let tank_d = UnitState::new(
        2,
        TeamSide::Defender,
        BoardSlot::new(TeamSide::Defender, 1, 1).unwrap(),
        3000,
        5,
        2000,
        100,
    );
    battle.add_unit(tank_a).unwrap();
    battle.add_unit(tank_d).unwrap();

    let mut turns = 0;
    while battle.winner().is_none() && turns < 200 {
        let events = battle.step_auto(false).expect("step must succeed");
        turns += 1;
        if events.iter().any(|e| matches!(e, CombatEvent::BattleEnded { .. })) {
            break;
        }
    }

    assert!(
        battle.winner().is_some(),
        "Tank stalemate must definitively end via sudden death or max turns"
    );
    assert!(
        battle.turn <= BattleState::MAX_TURNS,
        "Battle must conclude within MAX_TURNS ({})",
        BattleState::MAX_TURNS
    );
}

#[test]
fn test_cleric_healer_stalemate_resolves() {
    let mut battle = BattleState::default();
    // Two healers with skills that heal themselves, overcoming standard attack damage
    let mut healer_a = UnitState::new(
        1,
        TeamSide::Attacker,
        BoardSlot::new(TeamSide::Attacker, 1, 1).unwrap(),
        2000,
        50,
        100,
        100,
    );
    healer_a.hero_id = Some(HeroId::HuaTuo);

    let mut healer_d = UnitState::new(
        2,
        TeamSide::Defender,
        BoardSlot::new(TeamSide::Defender, 1, 1).unwrap(),
        2000,
        50,
        100,
        100,
    );
    healer_d.hero_id = Some(HeroId::DaQiaoXiaoQiao);

    battle.add_unit(healer_a).unwrap();
    battle.add_unit(healer_d).unwrap();

    let mut turns = 0;
    while battle.winner().is_none() && turns < 200 {
        let _ = battle.step_auto(false).expect("step must succeed");
        turns += 1;
    }

    assert!(
        battle.winner().is_some(),
        "Healer match must conclude without infinite loop"
    );
    assert!(
        battle.turn <= BattleState::MAX_TURNS,
        "Healer match must not exceed MAX_TURNS"
    );
}

#[test]
fn test_exact_mirror_draw_resolves_at_max_turns() {
    let mut battle = BattleState::default();
    // 0 attack units that deal 0 damage to each other
    let dummy_a = UnitState::new(
        1,
        TeamSide::Attacker,
        BoardSlot::new(TeamSide::Attacker, 1, 1).unwrap(),
        1000,
        0,
        100,
        100,
    );
    let dummy_d = UnitState::new(
        2,
        TeamSide::Defender,
        BoardSlot::new(TeamSide::Defender, 1, 1).unwrap(),
        1000,
        0,
        100,
        100,
    );
    battle.add_unit(dummy_a).unwrap();
    battle.add_unit(dummy_d).unwrap();

    // Force turn to MAX_TURNS to test tiebreaker resolution
    battle.turn = BattleState::MAX_TURNS;
    let winner = battle.winner();
    assert_eq!(
        winner,
        Some(None),
        "Equal alive count and equal HP at MAX_TURNS must resolve as a Draw (Some(None))"
    );
}

#[test]
fn test_thousand_random_matches_all_terminate() {
    let hero_candidates = [
        "zhao_yun",
        "huang_zhong",
        "zhuge_liang",
        "cao_cao",
        "dian_wei",
        "guo_jia",
        "sun_ce",
        "lu_xun",
        "da_qiao_xiao_qiao",
        "zhang_he_yan_liang",
        "hua_tuo",
        "jia_xu",
    ];

    let mut rng_state: u64 = 0x9E3779B97F4A7C15;
    let mut next_rand = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        rng_state
    };

    let total_matches = 1000;
    let mut completed_matches = 0;

    for match_idx in 0..total_matches {
        let mut battle = BattleState::default();

        let attacker_count = 1 + (next_rand() % 5) as usize;
        let defender_count = 1 + (next_rand() % 5) as usize;

        let mut unit_id = 1;
        for i in 0..attacker_count {
            let col = (i % 3) as u8;
            let row = ((i / 3) % 3) as u8;
            let slot = BoardSlot::new(TeamSide::Attacker, col, row).unwrap();
            let hero_name = hero_candidates[(next_rand() % hero_candidates.len() as u64) as usize];
            let hp = 500 + (next_rand() % 2500) as u32;
            let atk = 20 + (next_rand() % 150) as u32;
            let def = 10 + (next_rand() % 100) as u32;
            let spd = 50 + (next_rand() % 100) as u32;

            let mut u = UnitState::new(unit_id, TeamSide::Attacker, slot, hp, atk, def, spd);
            u.hero_id = HeroId::parse(hero_name);
            let _ = battle.add_unit(u);
            unit_id += 1;
        }

        for i in 0..defender_count {
            let col = (i % 3) as u8;
            let row = ((i / 3) % 3) as u8;
            let slot = BoardSlot::new(TeamSide::Defender, col, row).unwrap();
            let hero_name = hero_candidates[(next_rand() % hero_candidates.len() as u64) as usize];
            let hp = 500 + (next_rand() % 2500) as u32;
            let atk = 20 + (next_rand() % 150) as u32;
            let def = 10 + (next_rand() % 100) as u32;
            let spd = 50 + (next_rand() % 100) as u32;

            let mut u = UnitState::new(unit_id, TeamSide::Defender, slot, hp, atk, def, spd);
            u.hero_id = HeroId::parse(hero_name);
            let _ = battle.add_unit(u);
            unit_id += 1;
        }

        let mut loop_count = 0;
        while battle.winner().is_none() && loop_count < 150 {
            let res = battle.step_auto(false);
            if res.is_err() {
                break;
            }
            loop_count += 1;
        }

        assert!(
            battle.winner().is_some(),
            "Match #{} failed to terminate within {} steps! (Turn: {})",
            match_idx,
            loop_count,
            battle.turn
        );
        completed_matches += 1;
    }

    assert_eq!(
        completed_matches, total_matches,
        "All 1,000 randomized matches must terminate cleanly"
    );
}
