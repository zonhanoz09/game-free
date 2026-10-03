//! Battle state machine and turn simulation.

use crate::board::{BoardSlot, TeamSide, UnitFaction};
use crate::damage::calculate_damage_with_pierce;
use crate::effects::{EffectKind, SkillSpec};
use crate::events::CombatEvent;
use crate::skills::hero_skill_spec;
use crate::tactics;
use crate::targeting::{resolve_targets, TargetCandidate, TargetRule};
use crate::units::{FactionSynergy, UnitState};
use serde::{Deserialize, Serialize};

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
    pub const MAX_TURNS: u32 = 100;
    pub const SUDDEN_DEATH_TURN: u32 = 40;

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

    pub fn advance_gauges(&mut self) -> Vec<CombatEvent> {
        let mut advance_events = Vec::new();
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
                    advance_events.push(CombatEvent::Damage {
                        source_id: self.units[i].id,
                        target_id: self.units[i].id,
                        amount: burn_dmg,
                        target_hp: self.units[i].hp,
                    });
                    if self.units[i].hp == 0 {
                        advance_events.push(CombatEvent::Defeated {
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
                    advance_events.push(CombatEvent::Damage {
                        source_id: self.units[i].id,
                        target_id: self.units[i].id,
                        amount: poison_dmg,
                        target_hp: self.units[i].hp,
                    });
                    if self.units[i].hp == 0 {
                        advance_events.push(CombatEvent::Defeated {
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
                    advance_events.push(CombatEvent::StatusApplied {
                        source_id: self.units[i].id,
                        target_id: self.units[i].id,
                        effect: EffectKind::Freeze {
                            turns: self.units[i].frozen_turns,
                        },
                    });
                } else if self.units[i].stunned_turns > 0 {
                    self.units[i].stunned_turns -= 1;
                    advance_events.push(CombatEvent::Stunned {
                        unit_id: self.units[i].id,
                    });
                } else {
                    let spd = self.units[i].speed.max(1);
                    self.units[i].gauge.advance(spd);
                }
            }
        }
        self.events.extend(advance_events.iter().copied());
        advance_events
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
                        let mut amount = if self.units[target_index].anti_heal_turns > 0 {
                            (raw as u64 * (10_000 - self.units[target_index].anti_heal_bps as u64)
                                / 10_000) as u32
                        } else {
                            raw
                        };
                        if self.turn >= Self::SUDDEN_DEATH_TURN {
                            let heal_decay_bps =
                                ((self.turn - Self::SUDDEN_DEATH_TURN + 1) as u64 * 400).min(10_000);
                            amount =
                                (amount as u64 * (10_000 - heal_decay_bps) / 10_000) as u32;
                        }
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

        if self.turn >= Self::SUDDEN_DEATH_TURN {
            let fatigue_turns = (self.turn - Self::SUDDEN_DEATH_TURN + 1) as u64;
            let fatigue_bps = (fatigue_turns * 250).min(5000);
            for i in 0..self.units.len() {
                if self.units[i].is_alive() {
                    let fatigue_dmg =
                        ((self.units[i].max_hp as u64 * fatigue_bps) / 10_000).max(1) as u32;
                    self.units[i].hp = self.units[i].hp.saturating_sub(fatigue_dmg);
                    emitted.push(CombatEvent::Damage {
                        source_id: self.units[i].id,
                        target_id: self.units[i].id,
                        amount: fatigue_dmg,
                        target_hp: self.units[i].hp,
                    });
                    if self.units[i].hp == 0 {
                        emitted.push(CombatEvent::Defeated {
                            unit_id: self.units[i].id,
                        });
                    }
                }
            }
        }

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
        let mut advance_events = Vec::new();
        let mut advance_limit = 0;
        while self.next_ready_unit().is_none() && advance_limit < 1000 {
            advance_events.extend(self.advance_gauges());
            advance_limit += 1;
            if self.winner().is_some() {
                break;
            }
        }
        if let Some(winner) = self.winner() {
            let mut end_events = advance_events;
            end_events.push(CombatEvent::BattleEnded { winner });
            self.events.push(CombatEvent::BattleEnded { winner });
            return Ok(end_events);
        }
        let actor_id = self.next_ready_unit().ok_or(BattleError::NoReadyUnit)?;
        let actor = self
            .units
            .iter()
            .find(|unit| unit.id == actor_id)
            .copied()
            .ok_or(BattleError::MissingUnit)?;
        let skill = if actor.rage >= 100 { ultimate } else { normal };
        let mut action_events = self.execute_action(actor_id, skill.clone(), critical)?;
        advance_events.append(&mut action_events);
        Ok(advance_events)
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
        let mut advance_events = Vec::new();
        let mut advance_limit = 0;
        while self.next_ready_unit().is_none() && advance_limit < 1000 {
            advance_events.extend(self.advance_gauges());
            advance_limit += 1;
            if self.winner().is_some() {
                break;
            }
        }
        if let Some(winner) = self.winner() {
            let mut end_events = advance_events;
            end_events.push(CombatEvent::BattleEnded { winner });
            self.events.push(CombatEvent::BattleEnded { winner });
            return Ok(end_events);
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
        let mut action_events = self.execute_action(actor_id, skill, critical)?;
        advance_events.append(&mut action_events);
        Ok(advance_events)
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
            (true, true) => {
                if self.turn >= Self::MAX_TURNS {
                    let attacker_count = self
                        .units
                        .iter()
                        .filter(|u| u.side == TeamSide::Attacker && u.is_alive())
                        .count();
                    let defender_count = self
                        .units
                        .iter()
                        .filter(|u| u.side == TeamSide::Defender && u.is_alive())
                        .count();
                    match attacker_count.cmp(&defender_count) {
                        std::cmp::Ordering::Greater => Some(Some(TeamSide::Attacker)),
                        std::cmp::Ordering::Less => Some(Some(TeamSide::Defender)),
                        std::cmp::Ordering::Equal => {
                            let attacker_hp: u64 = self
                                .units
                                .iter()
                                .filter(|u| u.side == TeamSide::Attacker && u.is_alive())
                                .map(|u| u.hp as u64)
                                .sum();
                            let defender_hp: u64 = self
                                .units
                                .iter()
                                .filter(|u| u.side == TeamSide::Defender && u.is_alive())
                                .map(|u| u.hp as u64)
                                .sum();
                            match attacker_hp.cmp(&defender_hp) {
                                std::cmp::Ordering::Greater => Some(Some(TeamSide::Attacker)),
                                std::cmp::Ordering::Less => Some(Some(TeamSide::Defender)),
                                std::cmp::Ordering::Equal => Some(None),
                            }
                        }
                    }
                } else {
                    None
                }
            }
        }
    }
}

