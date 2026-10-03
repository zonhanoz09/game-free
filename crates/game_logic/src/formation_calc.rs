//! Runtime computation pipeline for 3x3 tactical card engine.
//!
//! Implements Section 5 of schema_game.md:
//! Stat_final = ( BaseStat × RarityMult × (1 + GrowthRate × (Level - 1)) × (1 + StarBonus) + LineBuffFlat ) × (1 + LineBuffPct)
//!
//! Line assignments:
//! - Slots 1, 2, 3: Frontline
//! - Slots 4, 5, 6: Midline
//! - Slots 7, 8, 9: Backline

use game_data_schema::{
    CardRole, CardTemplateRow, FormationLineConfig, FormationLineType, RarityConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComputedUnitStats {
    pub max_hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub speed: u32,
    pub crit_rate: f64,
    pub crit_dmg: f64,
    pub block_rate: f64,
    pub dodge_rate: f64,
    pub initial_morale: i32,
    pub max_morale: i32,
    pub crit_res_pct: f64,
    pub armor_pen_pct: f64,
    pub damage_dealt_mult: f64,
    pub damage_taken_mult: f64,
    pub aggro_mult: f64,
    pub cc_success_pct: f64,
    pub is_role_penalized: bool,
    pub line_type: FormationLineType,
    pub slot_id: u8,
}

/// Computes the star bonus multiplier: 0.10 (10%) per star rating above 1.
pub fn calculate_star_bonus(star: u8) -> f64 {
    if star <= 1 {
        0.0
    } else {
        (star - 1) as f64 * 0.10
    }
}

/// Applies the Section 5 Final Stat formula:
/// Stat_final = ( BaseStat × RarityMult × (1 + GrowthRate × (Level - 1)) × (1 + StarBonus) + LineBuffFlat ) × (1 + LineBuffPct)
pub fn compute_single_stat(
    base_stat: f64,
    rarity_mult: f64,
    growth_rate: f64,
    level: u32,
    star_bonus: f64,
    line_buff_flat: f64,
    line_buff_pct: f64,
) -> f64 {
    let level_factor = 1.0 + growth_rate * (level.saturating_sub(1) as f64);
    let star_factor = 1.0 + star_bonus;
    let pre_buff = base_stat * rarity_mult * level_factor * star_factor + line_buff_flat;
    pre_buff * (1.0 + line_buff_pct)
}

/// Pipeline function: evaluates card template, rarity, level, star, slot ID, and line configs
/// into a fully computed `ComputedUnitStats`.
pub fn compute_unit_final_stats(
    template: &CardTemplateRow,
    rarity: &RarityConfig,
    level: u32,
    star: u8,
    slot_id: u8,
    line_configs: &[FormationLineConfig],
) -> Result<ComputedUnitStats, String> {
    let line_type = FormationLineType::from_slot_id(slot_id)
        .ok_or_else(|| format!("Invalid slot_id {slot_id}: must be 1..=9"))?;

    let line_cfg = line_configs
        .iter()
        .find(|c| c.line_type == line_type)
        .ok_or_else(|| format!("Missing configuration for line type {line_type:?}"))?;

    let role = CardRole::from_str_loose(&template.recommended_role).unwrap_or(CardRole::Warrior);

    let is_role_penalized = line_cfg.penalty_roles.contains(&role);
    let star_bonus = calculate_star_bonus(star);

    // Flat buffs
    let speed_flat = line_cfg.line_buffs.speed_flat.unwrap_or(0.0);
    let morale_flat = line_cfg.line_buffs.initial_morale_flat.unwrap_or(0);

    // Pct buffs
    let hp_pct = line_cfg.line_buffs.hp_pct.unwrap_or(0.0);
    let def_pct = line_cfg.line_buffs.def_pct.unwrap_or(0.0);

    // Compute Base core stats
    let final_hp = compute_single_stat(
        template.base_hp as f64,
        rarity.base_stat_multiplier,
        rarity.growth_rate_per_level,
        level,
        star_bonus,
        0.0,
        hp_pct,
    )
    .round()
    .max(1.0) as u32;

    let final_atk = compute_single_stat(
        template.base_atk as f64,
        rarity.base_stat_multiplier,
        rarity.growth_rate_per_level,
        level,
        star_bonus,
        0.0,
        0.0,
    )
    .round()
    .max(1.0) as u32;

    let final_def = compute_single_stat(
        template.base_def as f64,
        rarity.base_stat_multiplier,
        rarity.growth_rate_per_level,
        level,
        star_bonus,
        0.0,
        def_pct,
    )
    .round()
    .max(0.0) as u32;

    let raw_speed = compute_single_stat(
        template.base_speed as f64,
        rarity.base_stat_multiplier,
        rarity.growth_rate_per_level,
        level,
        star_bonus,
        speed_flat,
        0.0,
    );

    // Line & Penalty Modifiers
    let mut speed_penalty_factor = 1.0;
    let mut aggro_mult = 1.0;
    let mut damage_taken_mult = 1.0;

    if is_role_penalized {
        if let Some(dmg_taken) = line_cfg.penalty_stats.damage_taken_pct {
            damage_taken_mult += dmg_taken;
        }
        if let Some(aggro_red) = line_cfg.penalty_stats.aggro_reduction_pct {
            aggro_mult = (aggro_mult - aggro_red).max(0.1);
        }
        if let Some(spd_red) = line_cfg.penalty_stats.speed_reduction_pct {
            speed_penalty_factor = (1.0 - spd_red).max(0.1);
        }
    }

    let final_speed = (raw_speed * speed_penalty_factor).round().max(1.0) as u32;

    // Line extra buffs
    let crit_rate_bonus = line_cfg.line_buffs.crit_rate_pct.unwrap_or(0.0);
    let final_crit_rate = (template.base_crit_rate + crit_rate_bonus).clamp(0.0, 1.0);
    let crit_res_pct = line_cfg.line_buffs.crit_res_pct.unwrap_or(0.0);
    let armor_pen_pct = line_cfg.line_buffs.armor_pen_pct.unwrap_or(0.0);
    let damage_dealt_mult = 1.0 + line_cfg.line_buffs.dmg_dealt_pct.unwrap_or(0.0);
    let cc_success_pct = line_cfg.line_buffs.cc_success_pct.unwrap_or(0.0);

    let final_initial_morale = template.initial_morale + morale_flat;

    Ok(ComputedUnitStats {
        max_hp: final_hp,
        attack: final_atk,
        defense: final_def,
        speed: final_speed,
        crit_rate: final_crit_rate,
        crit_dmg: template.base_crit_dmg,
        block_rate: template.base_block_rate,
        dodge_rate: template.base_dodge_rate,
        initial_morale: final_initial_morale,
        max_morale: template.max_morale,
        crit_res_pct,
        armor_pen_pct,
        damage_dealt_mult,
        damage_taken_mult,
        aggro_mult,
        cc_success_pct,
        is_role_penalized,
        line_type,
        slot_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_data_schema::{
        default_formation_line_configs, default_master_templates, default_rarity_configs,
    };

    #[test]
    fn test_compute_guan_yu_frontline_level_one_star_one() {
        let rarities = default_rarity_configs();
        let lines = default_formation_line_configs();
        let templates = default_master_templates();

        let guan_yu = &templates[0]; // SSR Warrior
        let ssr = &rarities["SSR"]; // mult = 1.50, growth = 0.10

        // Slot 2: Frontline (slots 1, 2, 3)
        // Guan Yu is WARRIOR, allowed on Frontline -> NOT penalized
        let stats = compute_unit_final_stats(guan_yu, ssr, 1, 1, 2, &lines).expect("stats");
        assert_eq!(stats.line_type, FormationLineType::Front);
        assert!(!stats.is_role_penalized);
        assert_eq!(stats.damage_taken_mult, 1.0);

        // Expected HP: 4200 * 1.50 * (1 + 0) * (1 + 0) = 6300. Frontline +15% HP -> 6300 * 1.15 = 7245
        assert_eq!(stats.max_hp, 7245);
        // Expected ATK: 780 * 1.50 = 1170
        assert_eq!(stats.attack, 1170);
        // Expected DEF: 360 * 1.50 = 540. Frontline +12% DEF -> 540 * 1.12 = 604.8 -> 605
        assert_eq!(stats.defense, 605);
        // Frontline crit res: +5%
        assert_eq!(stats.crit_res_pct, 0.05);
    }

    #[test]
    fn test_compute_level_and_star_scaling() {
        let rarities = default_rarity_configs();
        let lines = default_formation_line_configs();
        let templates = default_master_templates();

        let guan_yu = &templates[0];
        let ssr = &rarities["SSR"];

        // Level 11 (Lv-1 = 10 -> growth factor = 1 + 0.10 * 10 = 2.0)
        // Star 3 (Star-1 = 2 -> star factor = 1 + 0.20 = 1.20)
        // Midline (slot 5): +8% DMG dealt, +10% Crit Rate, +5% Armor Pen
        let stats = compute_unit_final_stats(guan_yu, ssr, 11, 3, 5, &lines).expect("stats");
        assert_eq!(stats.line_type, FormationLineType::Mid);
        assert!(!stats.is_role_penalized);
        assert_eq!(stats.damage_dealt_mult, 1.08);
        assert_eq!(stats.armor_pen_pct, 0.05);
        // Base crit rate 0.15 + Midline 0.10 = 0.25
        assert!((stats.crit_rate - 0.25).abs() < 1e-6);

        // ATK = 780 * 1.50 * 2.0 * 1.20 = 2808
        assert_eq!(stats.attack, 2808);
    }

    #[test]
    fn test_frontline_penalty_for_marksman_support() {
        let rarities = default_rarity_configs();
        let lines = default_formation_line_configs();
        let mut template = default_master_templates()[0].clone();
        template.recommended_role = "MARKSMAN".to_string();

        let ssr = &rarities["SSR"];
        // Marksman on Frontline (slot 1) should suffer +20% damage taken penalty
        let stats = compute_unit_final_stats(&template, ssr, 1, 1, 1, &lines).expect("stats");
        assert!(stats.is_role_penalized);
        assert_eq!(stats.damage_taken_mult, 1.20);
    }

    #[test]
    fn test_backline_buffs_and_vanguard_penalty() {
        let rarities = default_rarity_configs();
        let lines = default_formation_line_configs();
        let mut template = default_master_templates()[0].clone();
        template.recommended_role = "VANGUARD".to_string();

        let ssr = &rarities["SSR"];
        // Vanguard on Backline (slot 8) has penalty: -30% aggro, -15% speed
        // Backline buffs: +10 speed flat, +25 initial morale, +15% CC success
        let stats = compute_unit_final_stats(&template, ssr, 1, 1, 8, &lines).expect("stats");
        assert!(stats.is_role_penalized);
        assert!((stats.aggro_mult - 0.70).abs() < 1e-6);
        // Base speed = 105 * 1.50 = 157.5 + 10 flat = 167.5. Penalty 15% -> 167.5 * 0.85 = 142.375 -> 142
        assert_eq!(stats.speed, 142);
        // Initial morale: 50 + 25 = 75
        assert_eq!(stats.initial_morale, 75);
        assert_eq!(stats.cc_success_pct, 0.15);
    }
}
