//! Combat skill specs and effect primitives.

use crate::targeting::TargetRule;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum EffectKind {
    Damage,
    Heal,
    BuffAttack { bps: u16 },
    DebuffDefense { bps: u16 },
    Stun { turns: u8 },
    Burn { damage_bps: u16, turns: u8 },
    Poison { damage_bps: u16, turns: u8 },
    Taunt { turns: u8 },
    Freeze { turns: u8 },
    RageLock { turns: u8 },
    RageReduction { amount: u16 },
    Cleanse,
    AntiHeal { bps: u16, turns: u8 },
    Resurrect { hp_ratio_bps: u16 },
    AttackSteal { bps: u16 },
    Dodge { turns: u8 },
    Pierce { ignore_def_bps: u16 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SkillSpec {
    pub target_rule: TargetRule,
    pub damage_rate_bps: u16,
    pub rage_cost: u16,
    #[serde(default = "default_damage_effect")]
    pub effects: Vec<EffectKind>,
}

pub(crate) fn default_damage_effect() -> Vec<EffectKind> {
    vec![EffectKind::Damage]
}
