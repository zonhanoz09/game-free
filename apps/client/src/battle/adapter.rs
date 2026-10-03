use super::*;
use game_logic::{BattleState, CombatEvent, EffectKind, SkillSpec, TargetRule, TeamSide};
use std::collections::VecDeque;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BattleUnitId(pub u32);

#[derive(Resource)]
pub struct BattleSimulationAdapter {
    pub battle_state: Option<BattleState>,
    pub pending_events: VecDeque<CombatEvent>,
    pub battle_id: Option<String>,
    pub seed: u64,
    pub is_pvp: bool,
    pub authoritative_turn: u32,
    pub settled_winner: Option<Option<TeamSide>>,
    pub last_dispatched_command_id: Option<String>,
}

impl Default for BattleSimulationAdapter {
    fn default() -> Self {
        Self {
            battle_state: None,
            pending_events: VecDeque::new(),
            battle_id: None,
            seed: 42,
            is_pvp: false,
            authoritative_turn: 0,
            settled_winner: None,
            last_dispatched_command_id: None,
        }
    }
}

impl BattleSimulationAdapter {
    pub fn reset(&mut self, is_pvp: bool, seed: u64, battle_id: Option<String>) {
        self.battle_state = None;
        self.pending_events.clear();
        self.battle_id = battle_id;
        self.seed = seed;
        self.is_pvp = is_pvp;
        self.authoritative_turn = 0;
        self.settled_winner = None;
        self.last_dispatched_command_id = None;
    }
}

pub fn skill_specs_for_class(class: UnitClass) -> (SkillSpec, SkillSpec) {
    match class {
        UnitClass::Knight | UnitClass::DianWei | UnitClass::SunCe => (
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 1000,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 1500,
                rage_cost: 100,
                effects: vec![EffectKind::Damage, EffectKind::BuffAttack { bps: 2000 }],
            },
        ),
        UnitClass::CaoCao => (
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 1000,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::Cross,
                damage_rate_bps: 1400,
                rage_cost: 100,
                effects: vec![EffectKind::Damage, EffectKind::BuffAttack { bps: 2500 }],
            },
        ),
        UnitClass::Archer => (
            SkillSpec {
                target_rule: TargetRule::LowestHpRatio,
                damage_rate_bps: 1000,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::Cross,
                damage_rate_bps: 1350,
                rage_cost: 100,
                effects: vec![EffectKind::Damage],
            },
        ),
        UnitClass::Mage | UnitClass::GuoJia | UnitClass::LuXun => (
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 1000,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::PierceRow,
                damage_rate_bps: 1600,
                rage_cost: 100,
                effects: vec![EffectKind::Damage],
            },
        ),
        UnitClass::JiaXu => (
            SkillSpec {
                target_rule: TargetRule::LowestHpRatio,
                damage_rate_bps: 1000,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::PierceRow,
                damage_rate_bps: 1600,
                rage_cost: 100,
                effects: vec![EffectKind::Damage],
            },
        ),
        UnitClass::Assassin => (
            SkillSpec {
                target_rule: TargetRule::Backline,
                damage_rate_bps: 1000,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::Backline,
                damage_rate_bps: 1800,
                rage_cost: 100,
                effects: vec![EffectKind::Damage],
            },
        ),
        UnitClass::Cleric | UnitClass::DaQiaoXiaoQiao => (
            SkillSpec {
                target_rule: TargetRule::LowestHpRatio,
                damage_rate_bps: 500,
                rage_cost: 0,
                effects: vec![EffectKind::Heal],
            },
            SkillSpec {
                target_rule: TargetRule::Cross,
                damage_rate_bps: 1200,
                rage_cost: 100,
                effects: vec![EffectKind::Heal],
            },
        ),
    }
}

#[allow(dead_code)]
pub fn target_rule_name(rule: TargetRule) -> &'static str {
    match rule {
        TargetRule::DirectLine => "DIRECT_LINE",
        TargetRule::Backline => "TARGET_BACKLINE",
        TargetRule::LowestHpRatio => "TARGET_LOWEST_HP_RATIO",
        TargetRule::HighestAttack => "TARGET_HIGHEST_ATK",
        TargetRule::Cross => "TARGET_CROSS",
        TargetRule::PierceRow => "TARGET_PIERCE_ROW",
        TargetRule::Column => "TARGET_COLUMN",
    }
}

pub fn calculate_unit_id(side: TeamSide, index: usize, seed: u64) -> u32 {
    let side_offset = match side {
        TeamSide::Attacker => 0u64,
        TeamSide::Defender => 10_000u64,
    };
    seed.wrapping_add(side_offset).wrapping_add(index as u64) as u32
}
