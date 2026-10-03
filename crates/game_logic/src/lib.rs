//! Deterministic gameplay rules shared by runtime applications.

pub mod gacha;
pub mod tactics;
pub mod formation_calc;

use game_core::{BOARD_HEIGHT, BOARD_WIDTH};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TeamSide {
    Attacker,
    Defender,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum UnitFaction {
    Wei,
    Shu,
    Wu,
    Qun,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct BoardSlot {
    pub side: TeamSide,
    pub col: u8,
    pub row: u8,
}

impl BoardSlot {
    pub fn new(side: TeamSide, col: u8, row: u8) -> Option<Self> {
        (col < BOARD_WIDTH as u8 && row < BOARD_HEIGHT as u8).then_some(Self { side, col, row })
    }

    pub fn to_slot_id(self) -> u8 {
        self.col * BOARD_HEIGHT as u8 + self.row + 1
    }

    pub fn from_slot_id(side: TeamSide, slot_id: u8) -> Option<Self> {
        if !(1..=9).contains(&slot_id) {
            return None;
        }
        let index = slot_id - 1;
        Self::new(side, index / BOARD_HEIGHT as u8, index % BOARD_HEIGHT as u8)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TargetRule {
    DirectLine,
    Backline,
    LowestHpRatio,
    HighestAttack,
    Cross,
    PierceRow,
    Column,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetCandidate {
    pub slot: BoardSlot,
    pub hp: u32,
    pub max_hp: u32,
    pub attack: u32,
    pub is_taunting: bool,
}

impl TargetCandidate {
    pub fn new(slot: BoardSlot, hp: u32, max_hp: u32, attack: u32) -> Self {
        Self {
            slot,
            hp,
            max_hp,
            attack,
            is_taunting: false,
        }
    }
}

pub fn resolve_targets(
    attacker: BoardSlot,
    candidates: &[TargetCandidate],
    rule: TargetRule,
) -> Vec<BoardSlot> {
    if candidates.is_empty() {
        return Vec::new();
    }
    let living: Vec<TargetCandidate> = candidates.iter().copied().filter(|t| t.hp > 0).collect();
    if living.is_empty() {
        return Vec::new();
    }

    // Taunt check: if any candidate has taunt active, attacks are redirected to the taunter
    if let Some(taunter) = living.iter().copied().find(|t| t.is_taunting) {
        return vec![taunter.slot];
    }

    let primary = match rule {
        TargetRule::DirectLine => living.into_iter().min_by_key(|target| {
            (
                if target.slot.row == attacker.row {
                    0
                } else {
                    1
                },
                manhattan(attacker, target.slot),
                if target.slot.row == 1 { 0 } else { 1 },
                target.slot.to_slot_id(),
            )
        }),
        TargetRule::Backline => living.into_iter().min_by_key(|target| {
            (
                if target.slot.col == 2 { 0 } else { 1 },
                manhattan(attacker, target.slot),
                target.slot.to_slot_id(),
            )
        }),
        TargetRule::LowestHpRatio => living.into_iter().min_by(|left, right| {
            ratio_cmp(*left, *right)
                .then_with(|| left.slot.to_slot_id().cmp(&right.slot.to_slot_id()))
        }),
        TargetRule::HighestAttack => living
            .into_iter()
            .max_by_key(|target| (target.attack, std::cmp::Reverse(target.slot.to_slot_id()))),
        TargetRule::Cross | TargetRule::PierceRow | TargetRule::Column => living
            .into_iter()
            .min_by_key(|target| (manhattan(attacker, target.slot), target.slot.to_slot_id())),
    };
    let Some(primary) = primary else {
        return Vec::new();
    };
    match rule {
        TargetRule::Cross => candidates
            .iter()
            .filter(|target| {
                target.hp > 0
                    && (target.slot.col == primary.slot.col || target.slot.row == primary.slot.row)
                    && manhattan(primary.slot, target.slot) <= 1
            })
            .map(|target| target.slot)
            .collect(),
        TargetRule::PierceRow => candidates
            .iter()
            .filter(|target| target.hp > 0 && target.slot.row == primary.slot.row)
            .map(|target| target.slot)
            .collect(),
        TargetRule::Column => candidates
            .iter()
            .filter(|target| target.hp > 0 && target.slot.col == primary.slot.col)
            .map(|target| target.slot)
            .collect(),
        _ => vec![primary.slot],
    }
}

fn manhattan(left: BoardSlot, right: BoardSlot) -> u8 {
    left.col.abs_diff(right.col) + left.row.abs_diff(right.row)
}

fn ratio_cmp(left: TargetCandidate, right: TargetCandidate) -> Ordering {
    (left.hp as u64 * right.max_hp as u64).cmp(&(right.hp as u64 * left.max_hp as u64))
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ActionGauge {
    pub current: u32,
}

impl ActionGauge {
    pub const MAX: u32 = 10_000;

    pub fn advance(&mut self, speed: u32) {
        self.current = self.current.saturating_add(speed);
    }

    pub fn ready(self) -> bool {
        self.current >= Self::MAX
    }

    pub fn consume_turn(&mut self) {
        self.current = self.current.saturating_sub(Self::MAX);
    }
}

pub fn clamp_board_coordinate(value: usize, limit: usize) -> usize {
    value.min(limit.saturating_sub(1))
}

pub fn mitigate_damage(raw_damage: f32, defense: f32, minimum_damage: f32) -> f32 {
    (raw_damage * (100.0 / (100.0 + defense.max(0.0)))).max(minimum_damage)
}

pub fn calculate_damage(
    attack: u32,
    skill_rate_bps: u16,
    defense: u32,
    is_critical: bool,
    type_modifier_bps: u16,
) -> u32 {
    let raw = attack as u64 * skill_rate_bps as u64;
    let mitigated = raw * 1000 / (1000 + defense as u64) / 1000;
    let crit = if is_critical { 1500 } else { 1000 };
    let typed = mitigated * crit * type_modifier_bps as u64 / 1_000_000;
    typed.max(1) as u32
}

pub fn calculate_damage_with_pierce(
    attack: u32,
    skill_rate_bps: u16,
    defense: u32,
    is_critical: bool,
    type_modifier_bps: u16,
    ignore_def_bps: u16,
) -> u32 {
    let effective_def =
        (defense as u64 * (10_000 - ignore_def_bps.min(10_000) as u64) / 10_000) as u32;
    calculate_damage(
        attack,
        skill_rate_bps,
        effective_def,
        is_critical,
        type_modifier_bps,
    )
}

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

fn default_damage_effect() -> Vec<EffectKind> {
    vec![EffectKind::Damage]
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeroId {
    ZhaoYun,
    HuangZhong,
    ZhugeLiang,
    CaoCao,
    DianWei,
    GuoJia,
    SunCe,
    LuXun,
    DaQiaoXiaoQiao,
    ZhangHeYanLiang,
    HuaTuo,
    JiaXu,
}

impl HeroId {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "zhao_yun" => Some(Self::ZhaoYun),
            "huang_zhong" => Some(Self::HuangZhong),
            "zhuge_liang" => Some(Self::ZhugeLiang),
            "cao_cao" => Some(Self::CaoCao),
            "dian_wei" => Some(Self::DianWei),
            "guo_jia" => Some(Self::GuoJia),
            "sun_ce" => Some(Self::SunCe),
            "lu_xun" => Some(Self::LuXun),
            "da_qiao_xiao_qiao" => Some(Self::DaQiaoXiaoQiao),
            "zhang_he_yan_liang" => Some(Self::ZhangHeYanLiang),
            "hua_tuo" => Some(Self::HuaTuo),
            "jia_xu" => Some(Self::JiaXu),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ZhaoYun => "zhao_yun",
            Self::HuangZhong => "huang_zhong",
            Self::ZhugeLiang => "zhuge_liang",
            Self::CaoCao => "cao_cao",
            Self::DianWei => "dian_wei",
            Self::GuoJia => "guo_jia",
            Self::SunCe => "sun_ce",
            Self::LuXun => "lu_xun",
            Self::DaQiaoXiaoQiao => "da_qiao_xiao_qiao",
            Self::ZhangHeYanLiang => "zhang_he_yan_liang",
            Self::HuaTuo => "hua_tuo",
            Self::JiaXu => "jia_xu",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnitState {
    pub id: u32,
    pub side: TeamSide,
    pub slot: BoardSlot,
    pub max_hp: u32,
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub speed: u32,
    pub rage: u16,
    pub gauge: ActionGauge,
    pub stunned_turns: u8,
    #[serde(default)]
    pub hero_id: Option<HeroId>,
    #[serde(default)]
    pub faction: Option<UnitFaction>,
    #[serde(default)]
    pub burn_turns: u8,
    #[serde(default)]
    pub burn_damage_bps: u16,
    #[serde(default)]
    pub poison_turns: u8,
    #[serde(default)]
    pub poison_damage_bps: u16,
    #[serde(default)]
    pub taunt_turns: u8,
    #[serde(default)]
    pub frozen_turns: u8,
    #[serde(default)]
    pub rage_locked_turns: u8,
    #[serde(default)]
    pub anti_heal_turns: u8,
    #[serde(default)]
    pub anti_heal_bps: u16,
    #[serde(default)]
    pub dodge_turns: u8,
}

impl UnitState {
    pub fn new(
        id: u32,
        side: TeamSide,
        slot: BoardSlot,
        max_hp: u32,
        attack: u32,
        defense: u32,
        speed: u32,
    ) -> Self {
        Self {
            id,
            side,
            slot,
            max_hp,
            hp: max_hp,
            attack,
            defense,
            speed,
            rage: 0,
            gauge: ActionGauge::default(),
            stunned_turns: 0,
            faction: None,
            burn_turns: 0,
            burn_damage_bps: 0,
            poison_turns: 0,
            poison_damage_bps: 0,
            taunt_turns: 0,
            frozen_turns: 0,
            rage_locked_turns: 0,
            anti_heal_turns: 0,
            anti_heal_bps: 0,
            dodge_turns: 0,
            hero_id: None,
        }
    }

    pub fn with_hero_id(mut self, hero_id: HeroId) -> Self {
        self.hero_id = Some(hero_id);
        self
    }

    pub fn with_faction(mut self, faction: UnitFaction) -> Self {
        self.faction = Some(faction);
        self
    }

    pub fn is_alive(self) -> bool {
        self.hp > 0
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct FactionSynergy {
    pub wei_tier: u8, // 0, 3, 5
    pub shu_tier: u8, // 0, 3, 5
    pub wu_tier: u8,  // 0, 3, 5
    pub qun_tier: u8, // 0, 3, 5
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CombatEvent {
    ActionReady {
        unit_id: u32,
    },
    Attack {
        attacker_id: u32,
        target_id: u32,
        critical: bool,
    },
    Damage {
        source_id: u32,
        target_id: u32,
        amount: u32,
        target_hp: u32,
    },
    Heal {
        source_id: u32,
        target_id: u32,
        amount: u32,
        target_hp: u32,
    },
    StatusApplied {
        source_id: u32,
        target_id: u32,
        effect: EffectKind,
    },
    UltimateTriggered {
        unit_id: u32,
    },
    Stunned {
        unit_id: u32,
    },
    RageChanged {
        unit_id: u32,
        rage: u16,
    },
    Defeated {
        unit_id: u32,
    },
    BattleEnded {
        winner: Option<TeamSide>,
    },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BattleState {
    pub units: Vec<UnitState>,
    pub events: Vec<CombatEvent>,
    pub turn: u32,
    #[serde(default)]
    pub tactics: tactics::TacticalEconomy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleError {
    DuplicateUnitId,
    InvalidSlot,
    OccupiedSlot,
    MissingUnit,
    NoTarget,
    InvalidSkill,
    NoReadyUnit,
    BattleOver,
}

impl BattleState {
    pub fn add_unit(&mut self, unit: UnitState) -> Result<(), BattleError> {
        if self.units.iter().any(|existing| existing.id == unit.id) {
            return Err(BattleError::DuplicateUnitId);
        }
        if BoardSlot::new(unit.side, unit.slot.col, unit.slot.row).is_none() {
            return Err(BattleError::InvalidSlot);
        }
        if self
            .units
            .iter()
            .any(|existing| existing.is_alive() && existing.slot == unit.slot)
        {
            return Err(BattleError::OccupiedSlot);
        }
        self.units.push(unit);
        Ok(())
    }

    pub fn synergy(&self, side: TeamSide) -> FactionSynergy {
        let mut wei = 0;
        let mut shu = 0;
        let mut wu = 0;
        let mut qun = 0;
        for u in &self.units {
            if u.side == side {
                match u.faction {
                    Some(UnitFaction::Wei) => wei += 1,
                    Some(UnitFaction::Shu) => shu += 1,
                    Some(UnitFaction::Wu) => wu += 1,
                    Some(UnitFaction::Qun) => qun += 1,
                    None => {}
                }
            }
        }
        FactionSynergy {
            wei_tier: if wei >= 5 {
                5
            } else if wei >= 3 {
                3
            } else {
                0
            },
            shu_tier: if shu >= 5 {
                5
            } else if shu >= 3 {
                3
            } else {
                0
            },
            wu_tier: if wu >= 5 {
                5
            } else if wu >= 3 {
                3
            } else {
                0
            },
            qun_tier: if qun >= 5 {
                5
            } else if qun >= 3 {
                3
            } else {
                0
            },
        }
    }

    pub fn advance_gauges(&mut self) {
        for i in 0..self.units.len() {
            if self.units[i].is_alive() {
                // 1. Process DoT: Burn
                if self.units[i].burn_turns > 0 {
                    let burn_dmg = ((self.units[i].max_hp as u64
                        * self.units[i].burn_damage_bps as u64)
                        / 10_000)
                        .max(1) as u32;
                    self.units[i].hp = self.units[i].hp.saturating_sub(burn_dmg);
                    self.units[i].burn_turns -= 1;
                    self.events.push(CombatEvent::Damage {
                        source_id: self.units[i].id,
                        target_id: self.units[i].id,
                        amount: burn_dmg,
                        target_hp: self.units[i].hp,
                    });
                    if self.units[i].hp == 0 {
                        self.events.push(CombatEvent::Defeated {
                            unit_id: self.units[i].id,
                        });
                    }
                }

                // 2. Process DoT: Poison
                if self.units[i].is_alive() && self.units[i].poison_turns > 0 {
                    let poison_dmg = ((self.units[i].max_hp as u64
                        * self.units[i].poison_damage_bps as u64)
                        / 10_000)
                        .max(1) as u32;
                    self.units[i].hp = self.units[i].hp.saturating_sub(poison_dmg);
                    self.units[i].poison_turns -= 1;
                    self.events.push(CombatEvent::Damage {
                        source_id: self.units[i].id,
                        target_id: self.units[i].id,
                        amount: poison_dmg,
                        target_hp: self.units[i].hp,
                    });
                    if self.units[i].hp == 0 {
                        self.events.push(CombatEvent::Defeated {
                            unit_id: self.units[i].id,
                        });
                    }
                }

                // 3. Decrement turn-based status effects
                if self.units[i].taunt_turns > 0 {
                    self.units[i].taunt_turns -= 1;
                }
                if self.units[i].anti_heal_turns > 0 {
                    self.units[i].anti_heal_turns -= 1;
                    if self.units[i].anti_heal_turns == 0 {
                        self.units[i].anti_heal_bps = 0;
                    }
                }
                if self.units[i].rage_locked_turns > 0 {
                    self.units[i].rage_locked_turns -= 1;
                }
                if self.units[i].dodge_turns > 0 {
                    self.units[i].dodge_turns -= 1;
                }

                // 4. Stun & Freeze vs Action Gauge
                if self.units[i].frozen_turns > 0 {
                    self.units[i].frozen_turns -= 1;
                    self.events.push(CombatEvent::StatusApplied {
                        source_id: self.units[i].id,
                        target_id: self.units[i].id,
                        effect: EffectKind::Freeze {
                            turns: self.units[i].frozen_turns,
                        },
                    });
                } else if self.units[i].stunned_turns > 0 {
                    self.units[i].stunned_turns -= 1;
                    self.events.push(CombatEvent::Stunned {
                        unit_id: self.units[i].id,
                    });
                } else {
                    let spd = self.units[i].speed;
                    self.units[i].gauge.advance(spd);
                }
            }
        }
    }

    pub fn next_ready_unit(&self) -> Option<u32> {
        self.units
            .iter()
            .filter(|unit| {
                unit.is_alive()
                    && unit.stunned_turns == 0
                    && unit.frozen_turns == 0
                    && unit.gauge.ready()
            })
            .min_by_key(|unit| (std::cmp::Reverse(unit.gauge.current), unit.id))
            .map(|unit| unit.id)
    }

    pub fn execute_action(
        &mut self,
        actor_id: u32,
        skill: SkillSpec,
        critical: bool,
    ) -> Result<Vec<CombatEvent>, BattleError> {
        if skill.damage_rate_bps == 0 || skill.rage_cost > 100 {
            return Err(BattleError::InvalidSkill);
        }
        let actor_index = self
            .units
            .iter()
            .position(|unit| unit.id == actor_id && unit.is_alive())
            .ok_or(BattleError::MissingUnit)?;
        if !self.units[actor_index].gauge.ready() {
            return Err(BattleError::MissingUnit);
        }
        if self.units[actor_index].rage < skill.rage_cost {
            return Err(BattleError::InvalidSkill);
        }
        let actor = self.units[actor_index];
        let affects_allies = skill.effects.iter().any(|effect| {
            matches!(
                effect,
                EffectKind::Heal | EffectKind::Cleanse | EffectKind::Resurrect { .. }
            )
        });
        let candidates: Vec<TargetCandidate> = self
            .units
            .iter()
            .filter(|unit| {
                unit.is_alive()
                    && if affects_allies {
                        unit.side == actor.side
                    } else {
                        unit.side != actor.side
                    }
            })
            .map(|unit| TargetCandidate {
                slot: unit.slot,
                hp: unit.hp,
                max_hp: unit.max_hp,
                attack: unit.attack,
                is_taunting: !affects_allies && unit.taunt_turns > 0,
            })
            .collect();
        let targets = resolve_targets(actor.slot, &candidates, skill.target_rule);
        if targets.is_empty() {
            return Err(BattleError::NoTarget);
        }

        self.units[actor_index].gauge.consume_turn();
        self.units[actor_index].rage -= skill.rage_cost;
        let mut emitted = vec![CombatEvent::ActionReady { unit_id: actor_id }];
        if skill.rage_cost == 100 {
            emitted.push(CombatEvent::UltimateTriggered { unit_id: actor_id });
        }

        let actor_synergy = self.synergy(actor.side);
        let target_side = if affects_allies {
            actor.side
        } else {
            match actor.side {
                TeamSide::Attacker => TeamSide::Defender,
                TeamSide::Defender => TeamSide::Attacker,
            }
        };
        let target_synergy = self.synergy(target_side);

        // Calculate defense ignore from pierce effects or Shu 5-hero synergy
        let mut base_ignore_def_bps = 0u16;
        if critical && actor_synergy.shu_tier >= 5 {
            base_ignore_def_bps = base_ignore_def_bps.max(3000); // 30% ignore def on crit
        }
        for effect in &skill.effects {
            if let EffectKind::Pierce { ignore_def_bps } = effect {
                base_ignore_def_bps = base_ignore_def_bps.max(*ignore_def_bps);
            }
        }

        for target_slot in targets {
            let Some(target_index) = self.units.iter().position(|unit| {
                unit.slot == target_slot
                    && unit.is_alive()
                    && if affects_allies {
                        unit.side == actor.side
                    } else {
                        unit.side != actor.side
                    }
            }) else {
                continue;
            };
            let target_id = self.units[target_index].id;

            // Check target dodge
            let is_dodged = self.units[target_index].dodge_turns > 0;

            for effect in &skill.effects {
                match *effect {
                    EffectKind::Damage => {
                        emitted.push(CombatEvent::Attack {
                            attacker_id: actor_id,
                            target_id,
                            critical,
                        });

                        if is_dodged {
                            emitted.push(CombatEvent::Damage {
                                source_id: actor_id,
                                target_id,
                                amount: 0,
                                target_hp: self.units[target_index].hp,
                            });
                            continue;
                        }

                        // Qun 5-hero synergy: true damage if target HP < 40%
                        let mut ignore_def = base_ignore_def_bps;
                        if actor_synergy.qun_tier >= 5
                            && (self.units[target_index].hp as u64 * 100
                                / self.units[target_index].max_hp as u64)
                                < 40
                        {
                            ignore_def = 10_000;
                        }

                        let mut amount = calculate_damage_with_pierce(
                            actor.attack,
                            skill.damage_rate_bps,
                            self.units[target_index].defense,
                            critical,
                            1000,
                            ignore_def,
                        );

                        // Wu 3-hero synergy: +20% damage if target has Burn
                        if actor_synergy.wu_tier >= 3 && self.units[target_index].burn_turns > 0 {
                            amount = ((amount as u64 * 120) / 100) as u32;
                        }

                        // Wei synergy: damage reduction
                        if target_synergy.wei_tier >= 5 {
                            amount = ((amount as u64 * 80) / 100).max(1) as u32;
                        } else if target_synergy.wei_tier >= 3 {
                            amount = ((amount as u64 * 90) / 100).max(1) as u32;
                        }

                        self.units[target_index].hp =
                            self.units[target_index].hp.saturating_sub(amount);

                        // Target rage generation (blocked if rage locked)
                        if self.units[target_index].rage_locked_turns == 0 {
                            self.units[target_index].rage =
                                self.units[target_index].rage.saturating_add(15).min(100);
                        }

                        // Wei 3-hero: 20% chance to reduce 15 rage from attacker
                        if target_synergy.wei_tier >= 3 {
                            self.units[actor_index].rage =
                                self.units[actor_index].rage.saturating_sub(15);
                        }

                        // Wei 5-hero: reflect 20% damage back to attacker
                        if target_synergy.wei_tier >= 5 {
                            let reflect = (amount as u64 * 20 / 100) as u32;
                            self.units[actor_index].hp =
                                self.units[actor_index].hp.saturating_sub(reflect);
                            emitted.push(CombatEvent::Damage {
                                source_id: target_id,
                                target_id: actor_id,
                                amount: reflect,
                                target_hp: self.units[actor_index].hp,
                            });
                        }

                        // Qun 3-hero: lifesteal 15%
                        if actor_synergy.qun_tier >= 3 {
                            let lifesteal = ((amount as u64 * 15) / 100).max(1) as u32;
                            self.units[actor_index].hp = (self.units[actor_index].hp + lifesteal)
                                .min(self.units[actor_index].max_hp);
                            emitted.push(CombatEvent::Heal {
                                source_id: actor_id,
                                target_id: actor_id,
                                amount: lifesteal,
                                target_hp: self.units[actor_index].hp,
                            });
                        }

                        emitted.push(CombatEvent::Damage {
                            source_id: actor_id,
                            target_id,
                            amount,
                            target_hp: self.units[target_index].hp,
                        });
                        emitted.push(CombatEvent::RageChanged {
                            unit_id: target_id,
                            rage: self.units[target_index].rage,
                        });

                        if self.units[target_index].hp == 0 {
                            emitted.push(CombatEvent::Defeated { unit_id: target_id });
                            self.units[actor_index].rage =
                                self.units[actor_index].rage.saturating_add(20).min(100);

                            // Shu 3-hero: on kill grant +15 rage to all living allies
                            if actor_synergy.shu_tier >= 3 {
                                for u in &mut self.units {
                                    if u.side == actor.side && u.is_alive() {
                                        u.rage = u.rage.saturating_add(15).min(100);
                                        emitted.push(CombatEvent::RageChanged {
                                            unit_id: u.id,
                                            rage: u.rage,
                                        });
                                    }
                                }
                            }
                        } else {
                            self.units[actor_index].rage =
                                self.units[actor_index].rage.saturating_add(25).min(100);
                        }
                    }
                    EffectKind::Heal => {
                        let raw = (actor.attack as u64 * skill.damage_rate_bps as u64 / 1000).max(1)
                            as u32;
                        let amount = if self.units[target_index].anti_heal_turns > 0 {
                            (raw as u64 * (10_000 - self.units[target_index].anti_heal_bps as u64)
                                / 10_000) as u32
                        } else {
                            raw
                        };
                        self.units[target_index].hp = self.units[target_index]
                            .hp
                            .saturating_add(amount)
                            .min(self.units[target_index].max_hp);
                        emitted.push(CombatEvent::Heal {
                            source_id: actor_id,
                            target_id,
                            amount,
                            target_hp: self.units[target_index].hp,
                        });
                    }
                    EffectKind::BuffAttack { bps } => {
                        self.units[target_index].attack =
                            (self.units[target_index].attack as u64 * (10_000 + bps as u64)
                                / 10_000) as u32;
                        emitted.push(CombatEvent::StatusApplied {
                            source_id: actor_id,
                            target_id,
                            effect: *effect,
                        });
                    }
                    EffectKind::DebuffDefense { bps } => {
                        self.units[target_index].defense = (self.units[target_index].defense as u64
                            * (10_000 - bps.min(10_000) as u64)
                            / 10_000)
                            as u32;
                        emitted.push(CombatEvent::StatusApplied {
                            source_id: actor_id,
                            target_id,
                            effect: *effect,
                        });
                    }
                    EffectKind::Stun { turns } => {
                        self.units[target_index].stunned_turns =
                            self.units[target_index].stunned_turns.saturating_add(turns);
                        emitted.push(CombatEvent::StatusApplied {
                            source_id: actor_id,
                            target_id,
                            effect: *effect,
                        });
                    }
                    EffectKind::Burn { damage_bps, turns } => {
                        self.units[target_index].burn_turns =
                            self.units[target_index].burn_turns.max(turns);
                        self.units[target_index].burn_damage_bps = damage_bps;
                        emitted.push(CombatEvent::StatusApplied {
                            source_id: actor_id,
                            target_id,
                            effect: *effect,
                        });
                    }
                    EffectKind::Poison { damage_bps, turns } => {
                        self.units[target_index].poison_turns =
                            self.units[target_index].poison_turns.max(turns);
                        self.units[target_index].poison_damage_bps = damage_bps;
                        emitted.push(CombatEvent::StatusApplied {
                            source_id: actor_id,
                            target_id,
                            effect: *effect,
                        });
                    }
                    EffectKind::Taunt { turns } => {
                        self.units[actor_index].taunt_turns =
                            self.units[actor_index].taunt_turns.max(turns);
                        emitted.push(CombatEvent::StatusApplied {
                            source_id: actor_id,
                            target_id: actor_id,
                            effect: *effect,
                        });
                    }
                    EffectKind::Freeze { turns } => {
                        self.units[target_index].frozen_turns =
                            self.units[target_index].frozen_turns.max(turns);
                        emitted.push(CombatEvent::StatusApplied {
                            source_id: actor_id,
                            target_id,
                            effect: *effect,
                        });
                    }
                    EffectKind::RageLock { turns } => {
                        self.units[target_index].rage_locked_turns =
                            self.units[target_index].rage_locked_turns.max(turns);
                        emitted.push(CombatEvent::StatusApplied {
                            source_id: actor_id,
                            target_id,
                            effect: *effect,
                        });
                    }
                    EffectKind::RageReduction { amount } => {
                        self.units[target_index].rage =
                            self.units[target_index].rage.saturating_sub(amount);
                        emitted.push(CombatEvent::RageChanged {
                            unit_id: target_id,
                            rage: self.units[target_index].rage,
                        });
                    }
                    EffectKind::Cleanse => {
                        self.units[target_index].stunned_turns = 0;
                        self.units[target_index].frozen_turns = 0;
                        self.units[target_index].burn_turns = 0;
                        self.units[target_index].poison_turns = 0;
                        self.units[target_index].anti_heal_turns = 0;
                        self.units[target_index].anti_heal_bps = 0;
                        emitted.push(CombatEvent::StatusApplied {
                            source_id: actor_id,
                            target_id,
                            effect: *effect,
                        });
                    }
                    EffectKind::AntiHeal { bps, turns } => {
                        self.units[target_index].anti_heal_bps = bps;
                        self.units[target_index].anti_heal_turns =
                            self.units[target_index].anti_heal_turns.max(turns);
                        emitted.push(CombatEvent::StatusApplied {
                            source_id: actor_id,
                            target_id,
                            effect: *effect,
                        });
                    }
                    EffectKind::Resurrect { hp_ratio_bps } => {
                        if let Some(dead_idx) = self
                            .units
                            .iter()
                            .position(|u| u.side == actor.side && !u.is_alive())
                        {
                            let rev_hp = ((self.units[dead_idx].max_hp as u64
                                * hp_ratio_bps as u64)
                                / 10_000)
                                .max(1) as u32;
                            self.units[dead_idx].hp = rev_hp;
                            let dead_id = self.units[dead_idx].id;
                            emitted.push(CombatEvent::Heal {
                                source_id: actor_id,
                                target_id: dead_id,
                                amount: rev_hp,
                                target_hp: rev_hp,
                            });
                        }
                    }
                    EffectKind::AttackSteal { bps } => {
                        let stolen =
                            ((self.units[target_index].attack as u64 * bps as u64) / 10_000) as u32;
                        self.units[target_index].attack =
                            self.units[target_index].attack.saturating_sub(stolen);
                        self.units[actor_index].attack =
                            self.units[actor_index].attack.saturating_add(stolen);
                        emitted.push(CombatEvent::StatusApplied {
                            source_id: actor_id,
                            target_id,
                            effect: *effect,
                        });
                    }
                    EffectKind::Dodge { turns } => {
                        self.units[actor_index].dodge_turns =
                            self.units[actor_index].dodge_turns.max(turns);
                        emitted.push(CombatEvent::StatusApplied {
                            source_id: actor_id,
                            target_id: actor_id,
                            effect: *effect,
                        });
                    }
                    EffectKind::Pierce { .. } => {}
                }
            }
        }

        emitted.push(CombatEvent::RageChanged {
            unit_id: actor_id,
            rage: self.units[actor_index].rage,
        });
        self.tactics.add_tp(actor.side, 1);
        self.turn = self.turn.saturating_add(1);
        if let Some(winner) = self.winner() {
            emitted.push(CombatEvent::BattleEnded { winner });
        }
        self.events.extend(emitted.iter().copied());
        Ok(emitted)
    }

    pub fn step(
        &mut self,
        normal: &SkillSpec,
        ultimate: &SkillSpec,
        critical: bool,
    ) -> Result<Vec<CombatEvent>, BattleError> {
        if self.winner().is_some() {
            return Err(BattleError::BattleOver);
        }
        if self.next_ready_unit().is_none() {
            self.advance_gauges();
        }
        let actor_id = self.next_ready_unit().ok_or(BattleError::NoReadyUnit)?;
        let actor = self
            .units
            .iter()
            .find(|unit| unit.id == actor_id)
            .copied()
            .ok_or(BattleError::MissingUnit)?;
        let skill = if actor.rage >= 100 { ultimate } else { normal };
        self.execute_action(actor_id, skill.clone(), critical)
    }

    pub fn execute_tactic(
        &mut self,
        side: TeamSide,
        tactic: tactics::TacticKind,
        slot_a: Option<BoardSlot>,
        slot_b: Option<BoardSlot>,
    ) -> Result<Vec<CombatEvent>, BattleError> {
        tactics::execute_tactic(
            &mut self.units,
            &mut self.events,
            &mut self.tactics,
            side,
            tactic,
            slot_a,
            slot_b,
        )
    }

    pub fn step_auto(&mut self, critical: bool) -> Result<Vec<CombatEvent>, BattleError> {
        if self.winner().is_some() {
            return Err(BattleError::BattleOver);
        }
        if self.next_ready_unit().is_none() {
            self.advance_gauges();
        }
        let actor_id = self.next_ready_unit().ok_or(BattleError::NoReadyUnit)?;
        let actor = self
            .units
            .iter()
            .find(|unit| unit.id == actor_id)
            .cloned()
            .ok_or(BattleError::MissingUnit)?;

        let (normal, ultimate) = actor.hero_id.map(|h| h.as_str()).and_then(hero_skill_spec)
            .unwrap_or_else(|| {
                (
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
                        effects: vec![EffectKind::Damage],
                    },
                )
            });

        let skill = if actor.rage >= 100 { ultimate } else { normal };
        self.execute_action(actor_id, skill, critical)
    }

    pub fn winner(&self) -> Option<Option<TeamSide>> {
        let attacker_alive = self
            .units
            .iter()
            .any(|unit| unit.side == TeamSide::Attacker && unit.is_alive());
        let defender_alive = self
            .units
            .iter()
            .any(|unit| unit.side == TeamSide::Defender && unit.is_alive());
        match (attacker_alive, defender_alive) {
            (true, false) => Some(Some(TeamSide::Attacker)),
            (false, true) => Some(Some(TeamSide::Defender)),
            (false, false) => Some(None),
            (true, true) => None,
        }
    }
}

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
