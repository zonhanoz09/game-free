use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FactionKind {
    Shu,
    Wei,
    Wu,
    Qun,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CardRole {
    Vanguard,
    Warrior,
    Assassin,
    Marksman,
    Mage,
    Tactician,
    Support,
}

impl CardRole {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "VANGUARD" => Some(Self::Vanguard),
            "WARRIOR" => Some(Self::Warrior),
            "ASSASSIN" => Some(Self::Assassin),
            "MARKSMAN" => Some(Self::Marksman),
            "MAGE" => Some(Self::Mage),
            "TACTICIAN" => Some(Self::Tactician),
            "SUPPORT" => Some(Self::Support),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FormationLineType {
    Front,
    Mid,
    Back,
}

impl FormationLineType {
    /// Slot IDs: 1..=3 Front, 4..=6 Mid, 7..=9 Back
    pub fn from_slot_id(slot_id: u8) -> Option<Self> {
        match slot_id {
            1..=3 => Some(Self::Front),
            4..=6 => Some(Self::Mid),
            7..=9 => Some(Self::Back),
            _ => None,
        }
    }

    pub fn slot_indices(self) -> &'static [u8] {
        match self {
            Self::Front => &[1, 2, 3],
            Self::Mid => &[4, 5, 6],
            Self::Back => &[7, 8, 9],
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RarityConfig {
    pub rarity_id: String,
    pub display_name: String,
    pub base_stat_multiplier: f64,
    pub growth_rate_per_level: f64,
    pub max_star_rating: i16,
    pub max_skills_allowed: i16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CardTemplateRow {
    pub card_template_id: String,
    pub name: String,
    pub title: Option<String>,
    pub faction: String,
    pub rarity_id: String,
    pub recommended_role: String,

    pub base_hp: i32,
    pub base_atk: i32,
    pub base_def: i32,
    pub base_speed: i32,
    #[serde(default = "default_crit_rate")]
    pub base_crit_rate: f64,
    #[serde(default = "default_crit_dmg")]
    pub base_crit_dmg: f64,
    #[serde(default)]
    pub base_block_rate: f64,
    #[serde(default = "default_dodge_rate")]
    pub base_dodge_rate: f64,

    #[serde(default = "default_initial_morale")]
    pub initial_morale: i32,
    #[serde(default = "default_max_morale")]
    pub max_morale: i32,
}

fn default_crit_rate() -> f64 {
    0.050
}
fn default_crit_dmg() -> f64 {
    1.500
}
fn default_dodge_rate() -> f64 {
    0.050
}
fn default_initial_morale() -> i32 {
    50
}
fn default_max_morale() -> i32 {
    100
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SkillEffect {
    pub status_type: String,
    pub target: String,
    pub chance: f32,
    #[serde(default)]
    pub value: Option<f32>,
    pub duration_turns: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SkillRow {
    pub skill_id: String,
    pub name: String,
    pub skill_quality: String,
    #[serde(default)]
    pub is_skill_common: bool,
    pub trigger_type: String,
    pub execution_priority: i16,
    #[serde(default)]
    pub cost_morale: i32,
    #[serde(default)]
    pub cooldown_turns: i32,
    pub target_pattern: String,
    pub target_rule: String,
    pub damage_type: String,
    #[serde(default = "default_damage_mult")]
    pub damage_multiplier: f64,
    #[serde(default = "default_true")]
    pub can_crit: bool,
    #[serde(default)]
    pub effects_applied: Vec<SkillEffect>,
    #[serde(default)]
    pub description: Option<String>,
}

fn default_damage_mult() -> f64 {
    1.00
}
fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CardTemplateSkillRow {
    pub card_template_id: String,
    pub skill_id: String,
    #[serde(default = "default_one")]
    pub unlock_at_star: i16,
    #[serde(default = "default_slot_type")]
    pub slot_type: String, // 'NORMAL', 'ULTIMATE', 'PASSIVE'
}

fn default_one() -> i16 {
    1
}

fn default_slot_type() -> String {
    "NORMAL".to_string()
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FormationSynergyRow {
    pub synergy_id: String,
    pub name: String,
    pub synergy_type: String, // 'CLASS', 'FACTION', 'PAIR'
    pub trigger_count: usize,
    pub icon: String,
    pub description: String,
    #[serde(default)]
    pub hero_ids: Vec<String>,
    #[serde(default)]
    pub stat_buffs: serde_json::Value,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LinePenaltyStats {
    #[serde(default)]
    pub damage_taken_pct: Option<f64>,
    #[serde(default)]
    pub aggro_reduction_pct: Option<f64>,
    #[serde(default)]
    pub speed_reduction_pct: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LineBuffs {
    #[serde(default)]
    pub hp_pct: Option<f64>,
    #[serde(default)]
    pub def_pct: Option<f64>,
    #[serde(default)]
    pub crit_res_pct: Option<f64>,
    #[serde(default)]
    pub dmg_dealt_pct: Option<f64>,
    #[serde(default)]
    pub crit_rate_pct: Option<f64>,
    #[serde(default)]
    pub armor_pen_pct: Option<f64>,
    #[serde(default)]
    pub speed_flat: Option<f64>,
    #[serde(default)]
    pub initial_morale_flat: Option<i32>,
    #[serde(default)]
    pub cc_success_pct: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FormationLineConfig {
    pub line_type: FormationLineType,
    pub display_name: String,
    pub slot_indices: Vec<u8>,
    pub allowed_roles: Vec<CardRole>,
    pub penalty_roles: Vec<CardRole>,
    #[serde(default)]
    pub penalty_stats: LinePenaltyStats,
    pub line_buffs: LineBuffs,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PlayerCardRow {
    pub player_card_id: i64,
    pub user_id: i64,
    pub card_template_id: String,
    pub current_level: i32,
    pub current_exp: i64,
    pub current_star: i16,
    pub breakthrough_tier: i16,
    #[serde(default)]
    pub equipped_common_skill_id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PlayerFormationRow {
    pub formation_id: i64,
    pub user_id: i64,
    pub formation_type: String,
    pub slot_1_card_id: Option<i64>,
    pub slot_2_card_id: Option<i64>,
    pub slot_3_card_id: Option<i64>,
    pub slot_4_card_id: Option<i64>,
    pub slot_5_card_id: Option<i64>,
    pub slot_6_card_id: Option<i64>,
    pub slot_7_card_id: Option<i64>,
    pub slot_8_card_id: Option<i64>,
    pub slot_9_card_id: Option<i64>,
}

impl PlayerFormationRow {
    pub fn get_slot_card_id(&self, slot: u8) -> Option<i64> {
        match slot {
            1 => self.slot_1_card_id,
            2 => self.slot_2_card_id,
            3 => self.slot_3_card_id,
            4 => self.slot_4_card_id,
            5 => self.slot_5_card_id,
            6 => self.slot_6_card_id,
            7 => self.slot_7_card_id,
            8 => self.slot_8_card_id,
            9 => self.slot_9_card_id,
            _ => None,
        }
    }

    pub fn set_slot_card_id(&mut self, slot: u8, card_id: Option<i64>) {
        match slot {
            1 => self.slot_1_card_id = card_id,
            2 => self.slot_2_card_id = card_id,
            3 => self.slot_3_card_id = card_id,
            4 => self.slot_4_card_id = card_id,
            5 => self.slot_5_card_id = card_id,
            6 => self.slot_6_card_id = card_id,
            7 => self.slot_7_card_id = card_id,
            8 => self.slot_8_card_id = card_id,
            9 => self.slot_9_card_id = card_id,
            _ => {}
        }
    }

    pub fn slots_as_vec(&self) -> Vec<Option<i64>> {
        vec![
            self.slot_1_card_id,
            self.slot_2_card_id,
            self.slot_3_card_id,
            self.slot_4_card_id,
            self.slot_5_card_id,
            self.slot_6_card_id,
            self.slot_7_card_id,
            self.slot_8_card_id,
            self.slot_9_card_id,
        ]
    }
}

// ============================================================================
// STATUS EFFECT & BOARD CELL ROWS
// ============================================================================

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StatusEffectRow {
    pub effect_id: String,
    pub name: String,
    #[serde(default = "default_category")]
    pub effect_category: String,
    #[serde(default = "default_stat_target")]
    pub stat_target: String,
    #[serde(default = "default_calc_type")]
    pub calculation_type: String,
    #[serde(default)]
    pub base_value: f64,
    #[serde(default = "default_stack")]
    pub max_stacks: i16,
    #[serde(default = "default_duration")]
    pub duration_turns: i32,
    #[serde(default = "default_tick")]
    pub tick_trigger: String,
    #[serde(default = "default_true")]
    pub is_dispellable: bool,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub vfx_prefab: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

fn default_category() -> String {
    "BUFF".to_string()
}
fn default_stat_target() -> String {
    "ATK".to_string()
}
fn default_calc_type() -> String {
    "PERCENTAGE".to_string()
}
fn default_stack() -> i16 {
    1
}
fn default_duration() -> i32 {
    1
}
fn default_tick() -> String {
    "TURN_START".to_string()
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BoardCellConfigRow {
    pub slot_id: u8,
    pub col: u8,
    pub row: u8,
    #[serde(default = "default_line_front")]
    pub line_type: String,
    #[serde(default = "default_terrain_normal")]
    pub terrain_type: String,
    #[serde(default)]
    pub tile_buff_effect_id: Option<String>,
    #[serde(default)]
    pub hazard_damage_pct: f64,
    #[serde(default)]
    pub preferred_roles: Vec<String>,
    #[serde(default)]
    pub penalty_roles: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

fn default_line_front() -> String {
    "FRONT".to_string()
}
fn default_terrain_normal() -> String {
    "NORMAL".to_string()
}
