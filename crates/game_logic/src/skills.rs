//! Hero skill specifications and hero-to-faction mappings.

use crate::board::UnitFaction;
use crate::effects::{EffectKind, SkillSpec};
use crate::targeting::TargetRule;

/// Returns the exact (Normal, Ultimate) SkillSpec for each of the 12 Tam Quoc generals.
pub fn hero_skill_spec(hero_id: &str) -> Option<(SkillSpec, SkillSpec)> {
    match hero_id {
        "zhao_yun" => Some((
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 1000,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::LowestHpRatio,
                damage_rate_bps: 1800,
                rage_cost: 100,
                effects: vec![EffectKind::Damage, EffectKind::Dodge { turns: 1 }],
            },
        )),
        "huang_zhong" => Some((
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 1000,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::PierceRow,
                damage_rate_bps: 2200,
                rage_cost: 100,
                effects: vec![
                    EffectKind::Damage,
                    EffectKind::Pierce {
                        ignore_def_bps: 3000,
                    },
                ],
            },
        )),
        "zhuge_liang" => Some((
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 900,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::Column,
                damage_rate_bps: 1600,
                rage_cost: 100,
                effects: vec![EffectKind::Damage, EffectKind::Stun { turns: 1 }],
            },
        )),
        "cao_cao" => Some((
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 1000,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::HighestAttack,
                damage_rate_bps: 1200,
                rage_cost: 100,
                effects: vec![
                    EffectKind::Damage,
                    EffectKind::BuffAttack { bps: 2000 },
                    EffectKind::RageReduction { amount: 35 },
                ],
            },
        )),
        "dian_wei" => Some((
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 800,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::Column,
                damage_rate_bps: 1400,
                rage_cost: 100,
                effects: vec![EffectKind::Damage, EffectKind::Taunt { turns: 1 }],
            },
        )),
        "guo_jia" => Some((
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 900,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::HighestAttack,
                damage_rate_bps: 1300,
                rage_cost: 100,
                effects: vec![
                    EffectKind::Damage,
                    EffectKind::Freeze { turns: 1 },
                    EffectKind::RageLock { turns: 1 },
                ],
            },
        )),
        "sun_ce" => Some((
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 1000,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::Cross,
                damage_rate_bps: 1700,
                rage_cost: 100,
                effects: vec![EffectKind::Damage, EffectKind::AttackSteal { bps: 1500 }],
            },
        )),
        "lu_xun" => Some((
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 900,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::PierceRow,
                damage_rate_bps: 1600,
                rage_cost: 100,
                effects: vec![
                    EffectKind::Damage,
                    EffectKind::Burn {
                        damage_bps: 300,
                        turns: 2,
                    },
                ],
            },
        )),
        "da_qiao_xiao_qiao" => Some((
            SkillSpec {
                target_rule: TargetRule::LowestHpRatio,
                damage_rate_bps: 500,
                rage_cost: 0,
                effects: vec![EffectKind::Heal],
            },
            SkillSpec {
                target_rule: TargetRule::Column,
                damage_rate_bps: 1000,
                rage_cost: 100,
                effects: vec![EffectKind::Heal, EffectKind::Cleanse],
            },
        )),
        "zhang_he_yan_liang" => Some((
            SkillSpec {
                target_rule: TargetRule::Backline,
                damage_rate_bps: 1200,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::Backline,
                damage_rate_bps: 2200,
                rage_cost: 100,
                effects: vec![
                    EffectKind::Damage,
                    EffectKind::Pierce {
                        ignore_def_bps: 4000,
                    },
                ],
            },
        )),
        "hua_tuo" => Some((
            SkillSpec {
                target_rule: TargetRule::LowestHpRatio,
                damage_rate_bps: 500,
                rage_cost: 0,
                effects: vec![EffectKind::Heal],
            },
            SkillSpec {
                target_rule: TargetRule::Column,
                damage_rate_bps: 1000,
                rage_cost: 100,
                effects: vec![
                    EffectKind::Heal,
                    EffectKind::Resurrect { hp_ratio_bps: 3000 },
                ],
            },
        )),
        "jia_xu" => Some((
            SkillSpec {
                target_rule: TargetRule::DirectLine,
                damage_rate_bps: 900,
                rage_cost: 0,
                effects: vec![EffectKind::Damage],
            },
            SkillSpec {
                target_rule: TargetRule::Column,
                damage_rate_bps: 1500,
                rage_cost: 100,
                effects: vec![
                    EffectKind::Damage,
                    EffectKind::Poison {
                        damage_bps: 350,
                        turns: 2,
                    },
                    EffectKind::AntiHeal {
                        bps: 5000,
                        turns: 2,
                    },
                ],
            },
        )),
        _ => None,
    }
}

pub fn hero_faction(hero_id: &str) -> Option<UnitFaction> {
    match hero_id {
        "zhao_yun" | "huang_zhong" | "zhuge_liang" => Some(UnitFaction::Shu),
        "cao_cao" | "dian_wei" | "guo_jia" => Some(UnitFaction::Wei),
        "sun_ce" | "lu_xun" | "da_qiao_xiao_qiao" => Some(UnitFaction::Wu),
        "zhang_he_yan_liang" | "hua_tuo" | "jia_xu" => Some(UnitFaction::Qun),
        _ => None,
    }
}
