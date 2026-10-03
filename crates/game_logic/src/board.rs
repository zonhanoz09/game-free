//! Board coordinates, slot placement, and factions.

use game_core::{BOARD_HEIGHT, BOARD_WIDTH};
use serde::{Deserialize, Serialize};

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

pub fn clamp_board_coordinate(value: usize, limit: usize) -> usize {
    value.min(limit.saturating_sub(1))
}

pub(crate) fn manhattan(left: BoardSlot, right: BoardSlot) -> u8 {
    left.col.abs_diff(right.col) + left.row.abs_diff(right.row)
}
