//! Deterministic gameplay rules shared by runtime applications.

pub fn clamp_board_coordinate(value: usize, limit: usize) -> usize {
    value.min(limit.saturating_sub(1))
}

/// Applies the arena's defense curve in one place for all runtimes.
///
/// Keeping this rule outside Bevy means a server simulation and a client
/// prediction cannot silently diverge.
pub fn mitigate_damage(raw_damage: f32, defense: f32, minimum_damage: f32) -> f32 {
    (raw_damage * (100.0 / (100.0 + defense.max(0.0)))).max(minimum_damage)
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
}
