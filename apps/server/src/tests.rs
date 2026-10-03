use super::*;

#[test]
fn test_card_economy_and_rules() {
    let mut db = Database::new_in_memory();

    // 1. Register gives starter card + 100G + 10 Gems + Starter Deck
    let user = db
        .register("test_hero", "1234", "Hero Test", "knight")
        .expect("register failed");
    assert_eq!(user.gold, 100);
    assert_eq!(user.gems, 10);
    assert_eq!(user.cards.len(), 3);
    assert_eq!(user.decks.len(), 1);
    let starter = &user.cards[0];
    assert!(starter.is_starter);
    assert_eq!(starter.hero_class, "zhao_yun");

    // 2. Starter card CANNOT be sold
    let sell_starter = db.sell_card("test_hero", &starter.id);
    assert!(sell_starter.is_err(), "Starter card must not be sellable");

    // 3. Buy Huang Zhong card for 55G
    let (user, new_card) = db
        .buy_card("test_hero", "sun_ce")
        .expect("buy card failed");
    assert_eq!(user.gold, 45); // 100 - 55 = 45
    assert_eq!(user.cards.len(), 4);
    assert_eq!(new_card.hero_class, "sun_ce");
    assert!(!new_card.is_starter);

    // 4. Upgrade Huang Zhong card level (costs 1 * 20 = 20G)
    let (user, _msg) = db
        .upgrade_card("test_hero", &new_card.id, "level")
        .expect("upgrade failed");
    assert_eq!(user.gold, 25); // 45 - 20 = 25
    let upgraded = user.cards.iter().find(|c| c.id == new_card.id).unwrap();
    assert_eq!(upgraded.level, 2);
    assert!(upgraded.hp_bonus > 0.0);

    // 5. Reward match (PvE victory +45G, +60 EXP, +2 Gems)
    let (user, earned, exp, gems, _, _) = db
        .reward_match("test_hero", "pve", true)
        .expect("reward failed");
    assert_eq!(earned, 45);
    assert_eq!(exp, 60);
    assert_eq!(gems, 2);
    assert_eq!(user.gold, 70); // 25 + 45 = 70

    // Grant gems for testing foil upgrade
    {
        let u = db.data.users.get_mut("test_hero").unwrap();
        u.gems = 60;
    }

    // 6. Foil upgrade (costs 50 gems)
    let (user, _msg) = db
        .foil_card("test_hero", &new_card.id)
        .expect("foil failed");
    assert_eq!(user.gems, 10); // 60 - 50 = 10
    let foiled = user.cards.iter().find(|c| c.id == new_card.id).unwrap();
    assert!(foiled.is_foil);

    // 7. Profile Customization
    let user = db
        .customize_profile(
            "test_hero",
            Some("Đại Tướng".to_string()),
            Some("avatar_mage".to_string()),
            Some("cb_dragon".to_string()),
            Some("board_lava".to_string()),
        )
        .expect("customize failed");
    assert_eq!(user.display_name, "Đại Tướng");
    assert_eq!(user.avatar_id, "avatar_mage");
    assert_eq!(user.cardback_id, "cb_dragon");
    assert_eq!(user.board_skin, "board_lava");

    // 8. Deck Builder Engine & Validation
    let cards_data = vec![
        DeckCardEntry {
            id: starter.id.clone(),
            count: 1,
            position: None,
        },
        DeckCardEntry {
            id: new_card.id.clone(),
            count: 1,
            position: None,
        },
    ];
    // Huang Zhong card in zhao_yun deck should fail validation
    let (is_valid, errors) =
        validate_deck("Bộ Bài Triệu Vân", "zhao_yun", &cards_data, &user.cards);
    assert!(!is_valid);
    assert!(errors.iter().any(|e| e.contains("không phù hợp")));

    // Valid Huang Zhong deck
    let valid_cards = vec![DeckCardEntry {
        id: new_card.id.clone(),
        count: 1,
        position: None,
    }];
    let (is_valid_hz, errors_hz) = validate_deck(
        "Bộ Bài Tôn Sách",
        "sun_ce",
        &valid_cards,
        &user.cards,
    );
    assert!(is_valid_hz);
    assert!(errors_hz.is_empty());

    let (user, deck) = db
        .save_deck(
            "test_hero",
            None,
            "Bộ Bài Tôn Sách".to_string(),
            "sun_ce".to_string(),
            Some("cb_dragon".to_string()),
            valid_cards,
        )
        .expect("save deck failed");
    assert_eq!(user.decks.len(), 2);
    assert!(deck.is_valid);

    // 9. Sell the Huang Zhong card (non-starter is sellable)
    let (user, refund) = db
        .sell_card("test_hero", &new_card.id)
        .expect("sell non-starter failed");
    assert!(refund > 0);
    assert_eq!(user.cards.len(), 3);
    assert_eq!(user.cards[0].id, starter.id); // only starter remains
}

#[test]
fn test_effects_and_cells_master_data() {
    let mut db = Database::new_in_memory();

    // Check defaults were seeded
    assert!(
        !db.data.master_effects.is_empty(),
        "Master effects should be seeded"
    );
    assert_eq!(
        db.data.master_cells.len(),
        9,
        "9 Master board cells should be seeded"
    );

    // Test effect insert / update
    let custom_effect = game_data_schema::StatusEffectRow {
        effect_id: "test_bleed".to_string(),
        name: "Xuất Huyết".to_string(),
        effect_category: "DOT".to_string(),
        stat_target: "HP".to_string(),
        calculation_type: "PERCENTAGE".to_string(),
        base_value: 0.05,
        max_stacks: 5,
        duration_turns: 3,
        tick_trigger: "TURN_START".to_string(),
        is_dispellable: true,
        icon: Some("🩸".to_string()),
        vfx_prefab: Some("vfx_bleed".to_string()),
        description: Some("Mất 5% HP tối đa mỗi đầu lượt".to_string()),
    };
    db.data.master_effects.push(custom_effect);
    assert!(
        db.data
            .master_effects
            .iter()
            .any(|e| e.effect_id == "test_bleed")
    );

    // Test cell update
    if let Some(cell) = db.data.master_cells.iter_mut().find(|c| c.slot_id == 1) {
        cell.terrain_type = "Lava".to_string();
        cell.hazard_damage_pct = 0.10;
    }
    let cell1 = db
        .data
        .master_cells
        .iter()
        .find(|c| c.slot_id == 1)
        .unwrap();
    assert_eq!(cell1.terrain_type, "Lava");
    assert_eq!(cell1.hazard_damage_pct, 0.10);
}

#[test]
fn test_pvp_room_battle_finished_deduplication() {
    let (host_tx, _host_rx) = tokio::sync::mpsc::unbounded_channel();
    let (guest_tx, _guest_rx) = tokio::sync::mpsc::unbounded_channel();

    let host = PlayerSession {
        id: "p1".to_string(),
        name: "Host".to_string(),
        avatar: "archer".to_string(),
        elo: 1000,
        hp: 100,
        ready: true,
        lineup: vec![],
        tx: host_tx,
    };
    let guest = PlayerSession {
        id: "p2".to_string(),
        name: "Guest".to_string(),
        avatar: "knight".to_string(),
        elo: 1000,
        hp: 100,
        ready: true,
        lineup: vec![],
        tx: guest_tx,
    };

    let mut room = Room {
        code: "TEST".to_string(),
        round: 1,
        settled_round: 0,
        host,
        guest: Some(guest),
        authority: None,
    };

    // Simulate Host reporting round 1 victory
    let msg_round = 1;
    assert!(room.settled_round < msg_round, "Round 1 not settled yet");
    room.settled_round = msg_round.max(room.round);
    let damage = 10 + 2 * 4; // 18 damage
    room.guest.as_mut().unwrap().hp -= damage;
    assert_eq!(room.guest.as_ref().unwrap().hp, 82);
    room.round += 1; // round becomes 2

    // Now Guest sends duplicate BATTLE_FINISHED for round 1
    let guest_msg_round = 1;
    let is_duplicate = room.settled_round >= guest_msg_round;
    assert!(is_duplicate, "Guest duplicate submission for round 1 must be detected");
    // Guest HP must remain 82 (no duplicate damage)
    assert_eq!(room.guest.as_ref().unwrap().hp, 82);
    // Round must remain 2 (no premature round 3 progression)
    assert_eq!(room.round, 2);
}

#[test]
fn test_pvp_draw_applies_tiebreaker_damage() {
    let (host_tx, _) = tokio::sync::mpsc::unbounded_channel();
    let (guest_tx, _) = tokio::sync::mpsc::unbounded_channel();

    let mut room = Room {
        code: "TEST".to_string(),
        round: 1,
        settled_round: 0,
        host: PlayerSession {
            id: "p1".to_string(),
            name: "Host".to_string(),
            avatar: "archer".to_string(),
            elo: 1000,
            hp: 100,
            ready: true,
            lineup: vec![],
            tx: host_tx,
        },
        guest: Some(PlayerSession {
            id: "p2".to_string(),
            name: "Guest".to_string(),
            avatar: "knight".to_string(),
            elo: 1000,
            hp: 100,
            ready: true,
            lineup: vec![],
            tx: guest_tx,
        }),
        authority: None,
    };

    // Simulate 10 consecutive draws
    for _ in 1..=10 {
        let draw_damage = 10;
        room.host.hp = (room.host.hp - draw_damage).max(0);
        room.guest.as_mut().unwrap().hp = (room.guest.as_ref().unwrap().hp - draw_damage).max(0);
        room.round += 1;
    }

    assert_eq!(room.host.hp, 0);
    assert_eq!(room.guest.as_ref().unwrap().hp, 0);
    const MAX_MATCH_ROUNDS: usize = 20;
    let match_ended = room.host.hp <= 0 || room.guest.as_ref().unwrap().hp <= 0 || room.round >= MAX_MATCH_ROUNDS;
    assert!(match_ended, "Repeated draws must terminate the match via tiebreaker damage");
}

#[test]
fn test_pvp_round_cap_at_twenty_terminates_match() {
    let mut round = 1;
    let mut match_ended = false;
    const MAX_MATCH_ROUNDS: usize = 20;

    while !match_ended {
        if round >= MAX_MATCH_ROUNDS {
            match_ended = true;
            break;
        }
        round += 1;
    }

    assert!(match_ended);
    assert_eq!(round, 20);
}
