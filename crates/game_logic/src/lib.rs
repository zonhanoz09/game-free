//! Deterministic gameplay rules shared by runtime applications.

pub fn clamp_board_coordinate(value: usize, limit: usize) -> usize {
    value.min(limit.saturating_sub(1))
}
