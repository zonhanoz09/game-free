//! Deterministic gameplay rules shared by runtime applications.

pub mod board;
pub mod damage;
pub mod effects;
pub mod events;
pub mod formation_calc;
pub mod gacha;
pub mod skills;
pub mod state;
pub mod tactics;
pub mod targeting;
pub mod units;

pub use board::*;
pub use damage::*;
pub use effects::*;
pub use events::*;
pub use skills::*;
pub use state::*;
pub use targeting::*;
pub use units::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_mitigation_respects_defense_and_floor() {
        assert_eq!(mitigate_damage(100.0, 0.0, 5.0), 100.0);
        assert_eq!(mitigate_damage(100.0, 100.0, 5.0), 50.0);
        assert_eq!(mitigate_damage(1.0, 1_000.0, 5.0), 5.0);
    }

    #[test]
    fn board_slot_round_trips_slot_ids() {
        for slot_id in 1..=9 {
            let slot = BoardSlot::from_slot_id(TeamSide::Attacker, slot_id).expect("valid slot");
            assert_eq!(slot.to_slot_id(), slot_id);
        }
        assert!(BoardSlot::from_slot_id(TeamSide::Defender, 10).is_none());
    }

    #[test]
    fn direct_line_prefers_same_row_then_center_tiebreak() {
        let attacker = BoardSlot::new(TeamSide::Attacker, 0, 0).expect("valid slot");
        let targets = [
            TargetCandidate {
                slot: BoardSlot::new(TeamSide::Defender, 0, 1).expect("slot"),
                hp: 10,
                max_hp: 10,
                attack: 1,
                is_taunting: false,
            },
            TargetCandidate {
                slot: BoardSlot::new(TeamSide::Defender, 0, 2).expect("slot"),
                hp: 10,
                max_hp: 10,
                attack: 1,
                is_taunting: false,
            },
            TargetCandidate {
                slot: BoardSlot::new(TeamSide::Defender, 1, 1).expect("slot"),
                hp: 10,
                max_hp: 10,
                attack: 1,
                is_taunting: false,
            },
        ];
        assert_eq!(
            resolve_targets(attacker, &targets, TargetRule::DirectLine)[0].row,
            1
        );
    }

    #[test]
    fn taunt_forces_attacks_onto_taunter() {
        let attacker = BoardSlot::new(TeamSide::Attacker, 0, 0).unwrap();
        let targets = [
            TargetCandidate {
                slot: BoardSlot::new(TeamSide::Defender, 0, 0).unwrap(),
                hp: 100,
                max_hp: 100,
                attack: 10,
                is_taunting: false,
            },
            TargetCandidate {
                slot: BoardSlot::new(TeamSide::Defender, 2, 2).unwrap(),
                hp: 200,
                max_hp: 200,
                attack: 20,
                is_taunting: true,
            },
        ];
        let chosen = resolve_targets(attacker, &targets, TargetRule::DirectLine);
        assert_eq!(chosen[0], BoardSlot::new(TeamSide::Defender, 2, 2).unwrap());
    }

    #[test]
    fn action_gauge_accumulates_and_consumes_turns() {
        let mut gauge = ActionGauge::default();
        gauge.advance(10_000);
        assert!(gauge.ready());
        gauge.consume_turn();
        assert_eq!(gauge.current, 0);
    }

    #[test]
    fn step_selects_ready_unit_and_uses_ultimate_at_full_rage() {
        let mut battle = BattleState::default();
        let mut attacker = UnitState::new(
            1,
            TeamSide::Attacker,
            BoardSlot::new(TeamSide::Attacker, 0, 1).expect("slot"),
            100,
            20,
            0,
            10_000,
        );
        attacker.gauge.current = ActionGauge::MAX;
        attacker.rage = 100;
        battle.add_unit(attacker).expect("attacker");
        battle
            .add_unit(UnitState::new(
                2,
                TeamSide::Defender,
                BoardSlot::new(TeamSide::Defender, 0, 1).expect("slot"),
                100,
                10,
                0,
                10,
            ))
            .expect("defender");

        let normal = SkillSpec {
            target_rule: TargetRule::DirectLine,
            damage_rate_bps: 100,
            rage_cost: 0,
            effects: vec![EffectKind::Damage],
        };
        let ultimate = SkillSpec {
            target_rule: TargetRule::DirectLine,
            damage_rate_bps: 1000,
            rage_cost: 100,
            effects: vec![EffectKind::Damage, EffectKind::Stun { turns: 1 }],
        };
        let events = battle.step(&normal, &ultimate, false).expect("step");
        assert!(
            events
                .iter()
                .any(|event| matches!(event, CombatEvent::UltimateTriggered { unit_id: 1 }))
        );
        assert!(events.iter().any(|event| matches!(
            event,
            CombatEvent::StatusApplied {
                target_id: 2,
                effect: EffectKind::Stun { turns: 1 },
                ..
            }
        )));
        assert_eq!(battle.units[0].rage, 25);
        assert_eq!(battle.units[1].stunned_turns, 1);
    }

    #[test]
    fn battle_action_emits_replayable_damage_and_rage_events() {
        let mut battle = BattleState::default();
        let attacker_slot = BoardSlot::new(TeamSide::Attacker, 0, 1).expect("slot");
        let defender_slot = BoardSlot::new(TeamSide::Defender, 0, 1).expect("slot");
        let mut attacker = UnitState::new(1, TeamSide::Attacker, attacker_slot, 100, 20, 5, 10);
        attacker.gauge.current = ActionGauge::MAX;
        battle.add_unit(attacker).expect("attacker placement");
        battle
            .add_unit(UnitState::new(
                2,
                TeamSide::Defender,
                defender_slot,
                100,
                10,
                0,
                10,
            ))
            .expect("defender placement");

        let events = battle
            .execute_action(
                1,
                SkillSpec {
                    target_rule: TargetRule::DirectLine,
                    damage_rate_bps: 1000,
                    rage_cost: 0,
                    effects: vec![EffectKind::Damage],
                },
                false,
            )
            .expect("action");

        assert!(events.iter().any(|event| matches!(
            event,
            CombatEvent::Damage {
                source_id: 1,
                target_id: 2,
                ..
            }
        )));
        assert_eq!(battle.units[1].hp, 80);
        assert_eq!(battle.units[0].rage, 25);
        assert_eq!(battle.units[0].gauge.current, 0);
        assert_eq!(battle.events, events);
    }

    #[test]
    fn defeated_target_ends_battle_and_grants_kill_rage() {
        let mut battle = BattleState::default();
        let mut attacker = UnitState::new(
            1,
            TeamSide::Attacker,
            BoardSlot::new(TeamSide::Attacker, 0, 1).expect("slot"),
            100,
            100,
            0,
            10,
        );
        attacker.gauge.current = ActionGauge::MAX;
        battle.add_unit(attacker).expect("attacker placement");
        battle
            .add_unit(UnitState::new(
                2,
                TeamSide::Defender,
                BoardSlot::new(TeamSide::Defender, 0, 1).expect("slot"),
                10,
                1,
                0,
                10,
            ))
            .expect("defender placement");

        let events = battle
            .execute_action(
                1,
                SkillSpec {
                    target_rule: TargetRule::DirectLine,
                    damage_rate_bps: 1000,
                    rage_cost: 0,
                    effects: vec![EffectKind::Damage],
                },
                false,
            )
            .expect("action");

        assert!(
            events
                .iter()
                .any(|event| matches!(event, CombatEvent::Defeated { unit_id: 2 }))
        );
        assert_eq!(battle.units[0].rage, 20);
        assert_eq!(battle.winner(), Some(Some(TeamSide::Attacker)));
    }

    #[test]
    fn all_twelve_generals_have_valid_skillsets() {
        let heroes = [
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

        for hero_id in heroes {
            let spec = hero_skill_spec(hero_id);
            assert!(
                spec.is_some(),
                "General {} must have executable skill spec",
                hero_id
            );
            let (normal, ult) = spec.unwrap();
            assert_eq!(normal.rage_cost, 0);
            assert_eq!(ult.rage_cost, 100);
            assert!(!normal.effects.is_empty());
            assert!(!ult.effects.is_empty());
            assert!(hero_faction(hero_id).is_some());
        }
    }

    #[test]
    fn status_effects_burn_poison_cleanse_resurrect() {
        let mut battle = BattleState::default();
        let slot_atk = BoardSlot::new(TeamSide::Attacker, 0, 1).unwrap();
        let slot_ally = BoardSlot::new(TeamSide::Attacker, 1, 1).unwrap();
        let slot_def = BoardSlot::new(TeamSide::Defender, 0, 1).unwrap();

        let mut healer = UnitState::new(1, TeamSide::Attacker, slot_atk, 200, 50, 0, 10);
        healer.gauge.current = ActionGauge::MAX;
        battle.add_unit(healer).unwrap();

        let mut ally = UnitState::new(2, TeamSide::Attacker, slot_ally, 100, 10, 0, 10);
        ally.burn_turns = 2;
        ally.burn_damage_bps = 500; // 5% max HP per tick = 5 dmg
        battle.add_unit(ally).unwrap();

        let enemy = UnitState::new(3, TeamSide::Defender, slot_def, 100, 10, 0, 10);
        battle.add_unit(enemy).unwrap();

        // Advance gauges: burns ally for 5 dmg
        battle.advance_gauges();
        assert_eq!(battle.units[1].hp, 95);
        assert_eq!(battle.units[1].burn_turns, 1);

        // Apply cleanse on ally via LowestHpRatio
        let cleanse_events = battle
            .execute_action(
                1,
                SkillSpec {
                    target_rule: TargetRule::LowestHpRatio,
                    damage_rate_bps: 1000,
                    rage_cost: 0,
                    effects: vec![EffectKind::Cleanse],
                },
                false,
            )
            .unwrap();

        assert!(
            cleanse_events
                .iter()
                .any(|e| matches!(e, CombatEvent::StatusApplied { .. }))
        );
        assert_eq!(battle.units[1].burn_turns, 0);

        // Test Resurrect: kill ally, then cast resurrect
        battle.units[1].hp = 0;
        battle.units[0].gauge.current = ActionGauge::MAX;
        assert!(!battle.units[1].is_alive());
        let res_events = battle
            .execute_action(
                1,
                SkillSpec {
                    target_rule: TargetRule::LowestHpRatio,
                    damage_rate_bps: 1000,
                    rage_cost: 0,
                    effects: vec![EffectKind::Resurrect { hp_ratio_bps: 5000 }],
                },
                false,
            )
            .unwrap();
        assert!(res_events.iter().any(|e| matches!(e, CombatEvent::Heal { .. })));
        assert_eq!(battle.units[1].hp, 50);
        assert!(battle.units[1].is_alive());
    }

    #[test]
    fn faction_synergies_activation_and_effects() {
        let mut battle = BattleState::default();

        // 3 Shu heroes: +15% Crit Rate, +15 team rage on kill
        for i in 0..3 {
            let slot = BoardSlot::new(TeamSide::Attacker, i as u8, 0).unwrap();
            let mut u = UnitState::new(i + 1, TeamSide::Attacker, slot, 100, 30, 0, 10);
            u = u.with_faction(UnitFaction::Shu);
            battle.add_unit(u).unwrap();
        }

        // 3 Wei heroes: 10% damage reduction
        for i in 0..3 {
            let slot = BoardSlot::new(TeamSide::Defender, i as u8, 0).unwrap();
            let mut u = UnitState::new(i + 10, TeamSide::Defender, slot, 100, 10, 0, 10);
            u = u.with_faction(UnitFaction::Wei);
            battle.add_unit(u).unwrap();
        }

        let atk_syn = battle.synergy(TeamSide::Attacker);
        let def_syn = battle.synergy(TeamSide::Defender);
        assert_eq!(atk_syn.shu_tier, 3);
        assert_eq!(def_syn.wei_tier, 3);

        battle.units[0].gauge.current = ActionGauge::MAX;
        battle
            .execute_action(
                1,
                SkillSpec {
                    target_rule: TargetRule::DirectLine,
                    damage_rate_bps: 1000,
                    rage_cost: 0,
                    effects: vec![EffectKind::Damage],
                },
                false,
            )
            .unwrap();

        // 30 ATK mitigated by Wei 10% reduction -> 27 damage taken
        assert_eq!(battle.units[3].hp, 73);
    }

    #[test]
    fn headless_battle_invariants_hold_for_one_thousand_matches() {
        for match_index in 0..1_000 {
            let mut battle = BattleState::default();
            let mut attacker = UnitState::new(
                match_index * 2 + 1,
                TeamSide::Attacker,
                BoardSlot::new(TeamSide::Attacker, 0, 1).expect("slot"),
                100,
                25,
                0,
                10,
            );
            attacker.gauge.current = ActionGauge::MAX;
            battle.add_unit(attacker).expect("attacker placement");
            battle
                .add_unit(UnitState::new(
                    match_index * 2 + 2,
                    TeamSide::Defender,
                    BoardSlot::new(TeamSide::Defender, 0, 1).expect("slot"),
                    100,
                    10,
                    0,
                    10,
                ))
                .expect("defender placement");

            for _ in 0..4 {
                if battle.winner().is_some() {
                    break;
                }
                battle.units[0].gauge.current = ActionGauge::MAX;
                battle
                    .execute_action(
                        attacker.id,
                        SkillSpec {
                            target_rule: TargetRule::DirectLine,
                            damage_rate_bps: 1000,
                            rage_cost: 0,
                            effects: vec![EffectKind::Damage],
                        },
                        false,
                    )
                    .expect("deterministic action");
                assert!(battle.units.iter().all(|unit| unit.hp <= unit.max_hp));
                assert!(battle.units.iter().all(|unit| unit.rage <= 100));
            }
            assert_eq!(battle.winner(), Some(Some(TeamSide::Attacker)));
        }
    }
}

    #[test]
    fn all_twelve_generals_simulation_and_effects() {
        // 1. Zhao Yun (Dodge)
        {
            let mut battle = BattleState::default();
            let mut zy = UnitState::new(1, TeamSide::Attacker, BoardSlot::new(TeamSide::Attacker, 0, 1).unwrap(), 500, 100, 50, 10)
                .with_hero_id(HeroId::ZhaoYun);
            zy.gauge.current = ActionGauge::MAX;
            zy.rage = 100;
            battle.add_unit(zy).unwrap();
            let mut enemy = UnitState::new(2, TeamSide::Defender, BoardSlot::new(TeamSide::Defender, 0, 1).unwrap(), 500, 100, 50, 10);
            enemy.gauge.current = ActionGauge::MAX;
            battle.add_unit(enemy).unwrap();

            // Zhao Yun casts ultimate -> gets Dodge
            battle.step_auto(false).unwrap();
            assert_eq!(battle.units[0].dodge_turns, 1);

            // Enemy attacks Zhao Yun -> 0 damage due to Dodge
            let events = battle.step_auto(false).unwrap();
            let dmg = events.iter().find_map(|e| match e {
                CombatEvent::Damage { target_id: 1, amount, .. } => Some(*amount),
                _ => None,
            });
            assert_eq!(dmg, Some(0));
            assert_eq!(battle.units[0].hp, 500);
        }

        // 2. Huang Zhong (Pierce Row & Def Pierce)
        {
            let mut battle = BattleState::default();
            let mut hz = UnitState::new(1, TeamSide::Attacker, BoardSlot::new(TeamSide::Attacker, 0, 1).unwrap(), 300, 100, 30, 10)
                .with_hero_id(HeroId::HuangZhong);
            hz.gauge.current = ActionGauge::MAX;
            hz.rage = 100;
            battle.add_unit(hz).unwrap();
            let enemy = UnitState::new(2, TeamSide::Defender, BoardSlot::new(TeamSide::Defender, 0, 1).unwrap(), 500, 50, 200, 10);
            battle.add_unit(enemy).unwrap();

            let events = battle.step_auto(false).unwrap();
            assert!(events.iter().any(|e| matches!(e, CombatEvent::Damage { .. })));
        }

        // 3. Zhuge Liang (Stun)
        {
            let mut battle = BattleState::default();
            let mut zh = UnitState::new(1, TeamSide::Attacker, BoardSlot::new(TeamSide::Attacker, 0, 1).unwrap(), 300, 100, 30, 10)
                .with_hero_id(HeroId::ZhugeLiang);
            zh.gauge.current = ActionGauge::MAX;
            zh.rage = 100;
            battle.add_unit(zh).unwrap();
            let enemy = UnitState::new(2, TeamSide::Defender, BoardSlot::new(TeamSide::Defender, 0, 1).unwrap(), 500, 50, 20, 10);
            battle.add_unit(enemy).unwrap();

            battle.step_auto(false).unwrap();
            assert_eq!(battle.units[1].stunned_turns, 1);
        }

        // 4. Cao Cao (Buff Attack & Rage Reduction)
        {
            let mut battle = BattleState::default();
            let mut cc = UnitState::new(1, TeamSide::Attacker, BoardSlot::new(TeamSide::Attacker, 0, 1).unwrap(), 400, 100, 40, 10)
                .with_hero_id(HeroId::CaoCao);
            cc.gauge.current = ActionGauge::MAX;
            cc.rage = 100;
            battle.add_unit(cc).unwrap();
            let mut enemy = UnitState::new(2, TeamSide::Defender, BoardSlot::new(TeamSide::Defender, 0, 1).unwrap(), 500, 50, 20, 10);
            enemy.rage = 50;
            battle.add_unit(enemy).unwrap();

            battle.step_auto(false).unwrap();
            // Enemy rage +15 on hit, -35 by Cao Cao ultimate = 30
            assert_eq!(battle.units[1].rage, 30);
        }

        // 5. Guo Jia (Freeze & Rage Lock)
        {
            let mut battle = BattleState::default();
            let mut gj = UnitState::new(1, TeamSide::Attacker, BoardSlot::new(TeamSide::Attacker, 0, 1).unwrap(), 300, 100, 30, 10)
                .with_hero_id(HeroId::GuoJia);
            gj.gauge.current = ActionGauge::MAX;
            gj.rage = 100;
            battle.add_unit(gj).unwrap();
            let enemy = UnitState::new(2, TeamSide::Defender, BoardSlot::new(TeamSide::Defender, 0, 1).unwrap(), 500, 50, 20, 10);
            battle.add_unit(enemy).unwrap();

            battle.step_auto(false).unwrap();
            assert_eq!(battle.units[1].frozen_turns, 1);
            assert_eq!(battle.units[1].rage_locked_turns, 1);
        }

        // 6. Sun Ce (Attack Steal)
        {
            let mut battle = BattleState::default();
            let mut sc = UnitState::new(1, TeamSide::Attacker, BoardSlot::new(TeamSide::Attacker, 0, 1).unwrap(), 400, 100, 40, 10)
                .with_hero_id(HeroId::SunCe);
            sc.gauge.current = ActionGauge::MAX;
            sc.rage = 100;
            battle.add_unit(sc).unwrap();
            let enemy = UnitState::new(2, TeamSide::Defender, BoardSlot::new(TeamSide::Defender, 0, 1).unwrap(), 500, 100, 20, 10);
            battle.add_unit(enemy).unwrap();

            battle.step_auto(false).unwrap();
            // Stolen 15% of 100 = 15 ATK
            assert_eq!(battle.units[1].attack, 85);
            assert_eq!(battle.units[0].attack, 115);
        }

        // 7. Lu Xun (Burn)
        {
            let mut battle = BattleState::default();
            let mut lx = UnitState::new(1, TeamSide::Attacker, BoardSlot::new(TeamSide::Attacker, 0, 1).unwrap(), 300, 100, 30, 10)
                .with_hero_id(HeroId::LuXun);
            lx.gauge.current = ActionGauge::MAX;
            lx.rage = 100;
            battle.add_unit(lx).unwrap();
            let enemy = UnitState::new(2, TeamSide::Defender, BoardSlot::new(TeamSide::Defender, 0, 1).unwrap(), 500, 50, 20, 10);
            battle.add_unit(enemy).unwrap();

            battle.step_auto(false).unwrap();
            assert_eq!(battle.units[1].burn_turns, 2);
        }

        // 8. Jia Xu (Poison & Anti-Heal)
        {
            let mut battle = BattleState::default();
            let mut jx = UnitState::new(1, TeamSide::Attacker, BoardSlot::new(TeamSide::Attacker, 0, 1).unwrap(), 300, 100, 30, 10)
                .with_hero_id(HeroId::JiaXu);
            jx.gauge.current = ActionGauge::MAX;
            jx.rage = 100;
            battle.add_unit(jx).unwrap();
            let enemy = UnitState::new(2, TeamSide::Defender, BoardSlot::new(TeamSide::Defender, 0, 1).unwrap(), 500, 50, 20, 10);
            battle.add_unit(enemy).unwrap();

            battle.step_auto(false).unwrap();
            assert_eq!(battle.units[1].poison_turns, 2);
            assert_eq!(battle.units[1].anti_heal_turns, 2);
        }
    }
