//! Damage formulas, mitigation, and defense-piercing calculations.

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
