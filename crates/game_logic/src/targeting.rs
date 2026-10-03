//! Unit targeting rules and selection algorithms.

use crate::board::{manhattan, BoardSlot};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

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

fn ratio_cmp(left: TargetCandidate, right: TargetCandidate) -> Ordering {
    (left.hp as u64 * right.max_hp as u64).cmp(&(right.hp as u64 * left.max_hp as u64))
}
