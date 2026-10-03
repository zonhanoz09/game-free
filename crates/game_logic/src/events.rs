//! Combat simulation events emitted during battle progression.

use crate::board::TeamSide;
use crate::effects::EffectKind;
use serde::{Deserialize, Serialize};

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
