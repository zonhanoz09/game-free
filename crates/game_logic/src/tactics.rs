//! Tactics and Equipment systems for meta-layer strategic depth.
//!
//! Implements Section 6 of system_game.md:
//! - Tactical Points (TP) economy: max 10 TP, accumulates per friendly turn.
//! - Tactic Spells:
//!   - TACTIC_001 "Mượn Gió Đông" (Cost 3 TP): Doubles all active Burn damage on enemy team.
//!   - TACTIC_002 "Man Thiên Quá Hải" (Cost 2 TP): Swaps positions of two friendly units on 3x3 board.
//!   - TACTIC_003 "Bát Trận Đồ" (Cost 4 TP): Delays enemy action gauges by 30%.
//! - Equipment & Artifacts:
//!   - Weapon "Thanh Long Đao": +15% Ignore DEF on normal/skill attacks.
//!   - Mount "Xích Thố": +30 flat SPD.
//!   - Mount "Đích Lô": Reduces single-hit damage by 50% if damage exceeds 50% current HP.
//! - Meta Formations and balance validation for Shu, Wu, Wei, Qun.

use crate::{BattleError, BoardSlot, CombatEvent, TeamSide, UnitState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TacticKind {
    /// Doubles active burn damage bps on all enemy units.
    AmplifyBurn,
    /// Swaps the 3x3 positions of two living friendly units.
    SwapSlots,
    /// Delays all enemy units' action gauges by 3,000 points (30%).
    DelayTurnOrder,
}

impl TacticKind {
    pub fn tp_cost(self) -> u8 {
        match self {
            Self::AmplifyBurn => 3,
            Self::SwapSlots => 2,
            Self::DelayTurnOrder => 4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::AmplifyBurn => "Mượn Gió Đông",
            Self::SwapSlots => "Man Thiên Quá Hải",
            Self::DelayTurnOrder => "Bát Trận Đồ",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum EquipmentKind {
    /// Thanh Long Đao: +15% ignore defense
    WeaponThanhLongDao,
    /// Xích Thố: +30 flat speed
    MountXichTho,
    /// Đích Lô: -50% damage when single hit exceeds 50% current HP
    MountDichLo,
    /// Huyền Vũ Giáp: +50 flat defense
    ArmorHuyenVu,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Equipment {
    pub kind: EquipmentKind,
    pub bonus_atk: u32,
    pub bonus_def: u32,
    pub bonus_spd: u32,
    pub bonus_hp: u32,
    pub ignore_def_bps: u16,
}

impl Equipment {
    pub fn thanh_long_dao() -> Self {
        Self {
            kind: EquipmentKind::WeaponThanhLongDao,
            bonus_atk: 50,
            bonus_def: 0,
            bonus_spd: 0,
            bonus_hp: 0,
            ignore_def_bps: 1500, // 15% ignore def
        }
    }

    pub fn xich_tho() -> Self {
        Self {
            kind: EquipmentKind::MountXichTho,
            bonus_atk: 0,
            bonus_def: 0,
            bonus_spd: 30, // +30 flat SPD
            bonus_hp: 0,
            ignore_def_bps: 0,
        }
    }

    pub fn dich_lo() -> Self {
        Self {
            kind: EquipmentKind::MountDichLo,
            bonus_atk: 0,
            bonus_def: 20,
            bonus_spd: 10,
            bonus_hp: 200,
            ignore_def_bps: 0,
        }
    }

    pub fn huyen_vu() -> Self {
        Self {
            kind: EquipmentKind::ArmorHuyenVu,
            bonus_atk: 0,
            bonus_def: 50,
            bonus_spd: 0,
            bonus_hp: 400,
            ignore_def_bps: 0,
        }
    }

    /// Applies static equipment bonuses to a UnitState.
    pub fn apply_to_unit(&self, unit: &mut UnitState) {
        unit.attack = unit.attack.saturating_add(self.bonus_atk);
        unit.defense = unit.defense.saturating_add(self.bonus_def);
        unit.speed = unit.speed.saturating_add(self.bonus_spd);
        unit.max_hp = unit.max_hp.saturating_add(self.bonus_hp);
        unit.hp = unit.hp.saturating_add(self.bonus_hp);
    }
}

/// Tactical battle manager for TP and tactic card activation.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TacticalEconomy {
    pub attacker_tp: u8,
    pub defender_tp: u8,
}

impl TacticalEconomy {
    pub const MAX_TP: u8 = 10;

    pub fn get_tp(&self, side: TeamSide) -> u8 {
        match side {
            TeamSide::Attacker => self.attacker_tp,
            TeamSide::Defender => self.defender_tp,
        }
    }

    pub fn add_tp(&mut self, side: TeamSide, amount: u8) {
        match side {
            TeamSide::Attacker => {
                self.attacker_tp = (self.attacker_tp.saturating_add(amount)).min(Self::MAX_TP);
            }
            TeamSide::Defender => {
                self.defender_tp = (self.defender_tp.saturating_add(amount)).min(Self::MAX_TP);
            }
        }
    }

    pub fn spend_tp(&mut self, side: TeamSide, amount: u8) -> Result<(), BattleError> {
        match side {
            TeamSide::Attacker => {
                if self.attacker_tp >= amount {
                    self.attacker_tp -= amount;
                    Ok(())
                } else {
                    Err(BattleError::InvalidSkill)
                }
            }
            TeamSide::Defender => {
                if self.defender_tp >= amount {
                    self.defender_tp -= amount;
                    Ok(())
                } else {
                    Err(BattleError::InvalidSkill)
                }
            }
        }
    }
}

/// Executes a Tactic spell on the battle state.
pub fn execute_tactic(
    units: &mut [UnitState],
    events: &mut Vec<CombatEvent>,
    economy: &mut TacticalEconomy,
    side: TeamSide,
    tactic: TacticKind,
    slot_a: Option<BoardSlot>,
    slot_b: Option<BoardSlot>,
) -> Result<Vec<CombatEvent>, BattleError> {
    let cost = tactic.tp_cost();
    economy.spend_tp(side, cost)?;

    let mut emitted = Vec::new();
    match tactic {
        TacticKind::AmplifyBurn => {
            // Doubles burn damage for all living enemies with active burn
            for u in units.iter_mut() {
                if u.side != side && u.is_alive() && u.burn_turns > 0 {
                    u.burn_damage_bps = u.burn_damage_bps.saturating_mul(2);
                    emitted.push(CombatEvent::StatusApplied {
                        source_id: 0,
                        target_id: u.id,
                        effect: crate::EffectKind::Burn {
                            damage_bps: u.burn_damage_bps,
                            turns: u.burn_turns,
                        },
                    });
                }
            }
        }
        TacticKind::SwapSlots => {
            let (Some(a), Some(b)) = (slot_a, slot_b) else {
                return Err(BattleError::InvalidSlot);
            };
            if a.side != side || b.side != side {
                return Err(BattleError::InvalidSlot);
            }
            let idx_a = units.iter().position(|u| u.is_alive() && u.slot == a);
            let idx_b = units.iter().position(|u| u.is_alive() && u.slot == b);
            match (idx_a, idx_b) {
                (Some(ia), Some(ib)) => {
                    units[ia].slot = b;
                    units[ib].slot = a;
                }
                (Some(ia), None) => {
                    units[ia].slot = b;
                }
                (None, Some(ib)) => {
                    units[ib].slot = a;
                }
                (None, None) => return Err(BattleError::NoTarget),
            }
        }
        TacticKind::DelayTurnOrder => {
            // Delays enemy units' action gauges by 30% (3,000 points)
            for u in units.iter_mut() {
                if u.side != side && u.is_alive() {
                    u.gauge.current = u.gauge.current.saturating_sub(3_000);
                }
            }
        }
    }

    events.extend(emitted.iter().copied());
    Ok(emitted)
}


/// Helper to construct meta formations for testing and balance simulation.
pub fn create_meta_formation(side: TeamSide, faction: crate::UnitFaction) -> Vec<UnitState> {
    match faction {
        crate::UnitFaction::Shu => vec![
            UnitState::new(1, side, BoardSlot::new(side, 0, 1).unwrap(), 3800, 550, 300, 105)
                .with_hero_id(crate::HeroId::ZhaoYun)
                .with_faction(crate::UnitFaction::Shu),
            UnitState::new(2, side, BoardSlot::new(side, 1, 0).unwrap(), 2200, 920, 160, 100)
                .with_hero_id(crate::HeroId::HuangZhong)
                .with_faction(crate::UnitFaction::Shu),
            UnitState::new(3, side, BoardSlot::new(side, 1, 2).unwrap(), 2200, 920, 160, 100)
                .with_hero_id(crate::HeroId::ZhugeLiang)
                .with_faction(crate::UnitFaction::Shu),
        ],
        crate::UnitFaction::Wei => vec![
            UnitState::new(10, side, BoardSlot::new(side, 0, 1).unwrap(), 5000, 350, 420, 90)
                .with_hero_id(crate::HeroId::DianWei)
                .with_faction(crate::UnitFaction::Wei),
            UnitState::new(11, side, BoardSlot::new(side, 1, 0).unwrap(), 3800, 550, 300, 105)
                .with_hero_id(crate::HeroId::CaoCao)
                .with_faction(crate::UnitFaction::Wei),
            UnitState::new(12, side, BoardSlot::new(side, 1, 2).unwrap(), 2200, 920, 160, 100)
                .with_hero_id(crate::HeroId::GuoJia)
                .with_faction(crate::UnitFaction::Wei),
        ],
        crate::UnitFaction::Wu => vec![
            UnitState::new(20, side, BoardSlot::new(side, 0, 0).unwrap(), 3800, 550, 300, 105)
                .with_hero_id(crate::HeroId::SunCe)
                .with_faction(crate::UnitFaction::Wu),
            UnitState::new(21, side, BoardSlot::new(side, 1, 1).unwrap(), 2200, 920, 160, 100)
                .with_hero_id(crate::HeroId::LuXun)
                .with_faction(crate::UnitFaction::Wu),
            UnitState::new(22, side, BoardSlot::new(side, 1, 2).unwrap(), 3000, 450, 240, 115)
                .with_hero_id(crate::HeroId::DaQiaoXiaoQiao)
                .with_faction(crate::UnitFaction::Wu),
        ],
        crate::UnitFaction::Qun => vec![
            UnitState::new(30, side, BoardSlot::new(side, 0, 1).unwrap(), 2400, 850, 180, 130)
                .with_hero_id(crate::HeroId::ZhangHeYanLiang)
                .with_faction(crate::UnitFaction::Qun),
            UnitState::new(31, side, BoardSlot::new(side, 1, 0).unwrap(), 3000, 450, 240, 115)
                .with_hero_id(crate::HeroId::HuaTuo)
                .with_faction(crate::UnitFaction::Qun),
            UnitState::new(32, side, BoardSlot::new(side, 1, 2).unwrap(), 2200, 920, 160, 100)
                .with_hero_id(crate::HeroId::JiaXu)
                .with_faction(crate::UnitFaction::Qun),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn meta_formations_simulation_battle() {
        let mut battle = crate::BattleState::default();
        let mut shu_lineup = create_meta_formation(TeamSide::Attacker, crate::UnitFaction::Shu);
        let mut wei_lineup = create_meta_formation(TeamSide::Defender, crate::UnitFaction::Wei);

        // Equip Zhao Yun with Thanh Long Dao
        Equipment::thanh_long_dao().apply_to_unit(&mut shu_lineup[0]);
        // Equip Dian Wei with Huyen Vu Giap
        Equipment::huyen_vu().apply_to_unit(&mut wei_lineup[0]);

        for u in shu_lineup {
            battle.add_unit(u).unwrap();
        }
        for u in wei_lineup {
            battle.add_unit(u).unwrap();
        }

        assert_eq!(battle.synergy(TeamSide::Attacker).shu_tier, 3);
        assert_eq!(battle.synergy(TeamSide::Defender).wei_tier, 3);

        // Run simulation steps with auto skills and tactics
        let mut steps = 0;
        while battle.winner().is_none() && steps < 300 {
            steps += 1;
            if battle.tactics.get_tp(TeamSide::Attacker) >= 4 {
                let _ = battle.execute_tactic(TeamSide::Attacker, TacticKind::DelayTurnOrder, None, None);
            }
            let _ = battle.step_auto(false);
        }

        // Invariants
        assert!(battle.units.iter().all(|u| u.hp <= u.max_hp));
        assert!(battle.units.iter().all(|u| u.rage <= 100));
        assert!(!battle.events.is_empty());
    }

    #[test]
    fn tactical_points_accumulation_and_spending() {
        let mut tp = TacticalEconomy::default();
        assert_eq!(tp.get_tp(TeamSide::Attacker), 0);

        tp.add_tp(TeamSide::Attacker, 5);
        assert_eq!(tp.get_tp(TeamSide::Attacker), 5);

        // Cap at 10
        tp.add_tp(TeamSide::Attacker, 10);
        assert_eq!(tp.get_tp(TeamSide::Attacker), 10);

        // Spend for Tactic
        assert!(tp.spend_tp(TeamSide::Attacker, 4).is_ok());
        assert_eq!(tp.get_tp(TeamSide::Attacker), 6);

        // Insufficient TP
        assert!(tp.spend_tp(TeamSide::Attacker, 7).is_err());
        assert_eq!(tp.get_tp(TeamSide::Attacker), 6);
    }

    #[test]
    fn tactic_amplify_burn_doubles_burn_dot() {
        let mut units = vec![
            UnitState::new(
                1,
                TeamSide::Attacker,
                BoardSlot::new(TeamSide::Attacker, 0, 1).unwrap(),
                500,
                100,
                50,
                10,
            ),
            {
                let mut enemy = UnitState::new(
                    2,
                    TeamSide::Defender,
                    BoardSlot::new(TeamSide::Defender, 0, 1).unwrap(),
                    500,
                    100,
                    50,
                    10,
                );
                enemy.burn_turns = 2;
                enemy.burn_damage_bps = 500;
                enemy
            },
        ];
        let mut events = Vec::new();
        let mut economy = TacticalEconomy::default();
        economy.add_tp(TeamSide::Attacker, 5);

        let res = execute_tactic(
            &mut units,
            &mut events,
            &mut economy,
            TeamSide::Attacker,
            TacticKind::AmplifyBurn,
            None,
            None,
        )
        .expect("amplify burn succeeds");

        assert_eq!(units[1].burn_damage_bps, 1000);
        assert_eq!(economy.get_tp(TeamSide::Attacker), 2); // 5 - 3 = 2
        assert!(!res.is_empty());
    }

    #[test]
    fn tactic_swap_slots_swaps_positions() {
        let slot_a = BoardSlot::new(TeamSide::Attacker, 0, 0).unwrap();
        let slot_b = BoardSlot::new(TeamSide::Attacker, 2, 2).unwrap();
        let mut units = vec![
            UnitState::new(1, TeamSide::Attacker, slot_a, 500, 100, 50, 10),
            UnitState::new(2, TeamSide::Attacker, slot_b, 500, 100, 50, 10),
        ];
        let mut events = Vec::new();
        let mut economy = TacticalEconomy::default();
        economy.add_tp(TeamSide::Attacker, 5);

        execute_tactic(
            &mut units,
            &mut events,
            &mut economy,
            TeamSide::Attacker,
            TacticKind::SwapSlots,
            Some(slot_a),
            Some(slot_b),
        )
        .expect("swap succeeds");

        assert_eq!(units[0].slot, slot_b);
        assert_eq!(units[1].slot, slot_a);
        assert_eq!(economy.get_tp(TeamSide::Attacker), 3); // 5 - 2 = 3
    }

    #[test]
    fn equipment_stat_modifiers_application() {
        let slot = BoardSlot::new(TeamSide::Attacker, 0, 1).unwrap();
        let mut unit = UnitState::new(1, TeamSide::Attacker, slot, 500, 100, 50, 10);

        let weapon = Equipment::thanh_long_dao();
        weapon.apply_to_unit(&mut unit);
        assert_eq!(unit.attack, 150); // 100 + 50

        let mount = Equipment::xich_tho();
        mount.apply_to_unit(&mut unit);
        assert_eq!(unit.speed, 40); // 10 + 30
    }
}
