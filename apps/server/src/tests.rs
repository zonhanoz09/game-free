use super::*;

#[test]
fn test_card_economy_and_rules() {
    let temp_dir = std::env::temp_dir();
    let test_db_path = temp_dir.join(format!("test_db_{}.json", chrono_now()));
    let mut db = Database::new(&test_db_path);

    // 1. Register gives starter card + 100G + 10 Gems + Starter Deck
    let user = db
        .register("test_hero", "1234", "Hero Test", "knight")
        .expect("register failed");
    assert_eq!(user.gold, 100);
    assert_eq!(user.gems, 10);
    assert_eq!(user.cards.len(), 1);
    assert_eq!(user.decks.len(), 1);
    let starter = &user.cards[0];
    assert!(starter.is_starter);
    assert_eq!(starter.hero_class, "Knight");

    // 2. Starter card CANNOT be sold
    let sell_starter = db.sell_card("test_hero", &starter.id);
    assert!(sell_starter.is_err(), "Starter card must not be sellable");

    // 3. Buy Archer card for 50G
    let (user, new_card) = db.buy_card("test_hero", "Archer").expect("buy card failed");
    assert_eq!(user.gold, 50); // 100 - 50 = 50
    assert_eq!(user.cards.len(), 2);
    assert_eq!(new_card.hero_class, "Archer");
    assert!(!new_card.is_starter);

    // 4. Upgrade Archer card level (costs 1 * 20 = 20G)
    let (user, _msg) = db
        .upgrade_card("test_hero", &new_card.id, "level")
        .expect("upgrade failed");
    assert_eq!(user.gold, 30); // 50 - 20 = 30
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
    assert_eq!(user.gold, 75);

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
    // Archer card in Knight deck should fail validation
    let (is_valid, errors) = validate_deck("Bộ Bài Chiến Binh", "Knight", &cards_data, &user.cards);
    assert!(!is_valid);
    assert!(errors.iter().any(|e| e.contains("không phù hợp")));

    // Valid Archer deck
    let valid_cards = vec![DeckCardEntry {
        id: new_card.id.clone(),
        count: 1,
        position: None,
    }];
    let (is_valid_archer, errors_archer) =
        validate_deck("Bộ Bài Xạ Thủ", "Archer", &valid_cards, &user.cards);
    assert!(is_valid_archer);
    assert!(errors_archer.is_empty());

    let (user, deck) = db
        .save_deck(
            "test_hero",
            None,
            "Bộ Bài Xạ Thủ".to_string(),
            "Archer".to_string(),
            Some("cb_dragon".to_string()),
            valid_cards,
        )
        .expect("save deck failed");
    assert_eq!(user.decks.len(), 2);
    assert!(deck.is_valid);

    // 9. Sell the Archer card (non-starter is sellable)
    let (user, refund) = db
        .sell_card("test_hero", &new_card.id)
        .expect("sell non-starter failed");
    assert!(refund > 0);
    assert_eq!(user.cards.len(), 1);
    assert_eq!(user.cards[0].id, starter.id); // only starter remains

    let _ = std::fs::remove_file(test_db_path);
}
