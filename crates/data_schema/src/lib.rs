//! Serializable schemas for balance and runtime game data.

pub mod models;
pub mod seeds;

pub use models::*;
pub use seeds::*;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Faction {
    Wei,
    Shu,
    Wu,
    Qun,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Rarity {
    R,
    Sr,
    Ssr,
    Ur,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeroRole {
    Tanker,
    Warrior,
    Assassin,
    MageMarksman,
    SupportHealer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HeroStats {
    pub hp: u32,
    pub atk: u32,
    pub def: u32,
    pub spd: u32,
    pub crit_rate_bps: u16,
    pub crit_damage_bps: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillData {
    pub id: String,
    pub name: String,
    pub target_rule: String,
    pub damage_rate_bps: u16,
    #[serde(default)]
    pub effects: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HeroData {
    pub id: String,
    pub name: String,
    pub faction: Faction,
    pub rarity: Rarity,
    pub role: HeroRole,
    pub optimal_slots: Vec<u8>,
    pub stats: HeroStats,
    pub normal_skill: SkillData,
    pub ultimate_skill: SkillData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContentCatalog {
    pub schema_version: u32,
    pub config_version: String,
    pub heroes: Vec<HeroData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogError {
    UnsupportedSchema(u32),
    MissingConfigVersion,
    EmptyHeroes,
    DuplicateHeroId(String),
    MissingSkillId(String),
    InvalidSlot { hero_id: String, slot: u8 },
    InvalidStats(String),
    InvalidSkill(String),
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CatalogError {}

impl ContentCatalog {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn from_json(input: &str) -> Result<Self, String> {
        let catalog: Self = serde_json::from_str(input).map_err(|error| error.to_string())?;
        catalog.validate().map_err(|error| error.to_string())?;
        Ok(catalog)
    }

    pub fn validate(&self) -> Result<(), CatalogError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(CatalogError::UnsupportedSchema(self.schema_version));
        }
        if self.config_version.trim().is_empty() {
            return Err(CatalogError::MissingConfigVersion);
        }
        if self.heroes.is_empty() {
            return Err(CatalogError::EmptyHeroes);
        }

        let mut hero_ids = HashSet::new();
        let mut skill_ids = HashSet::new();
        for hero in &self.heroes {
            if !hero_ids.insert(hero.id.as_str()) {
                return Err(CatalogError::DuplicateHeroId(hero.id.clone()));
            }
            validate_stats(hero)?;
            if hero.optimal_slots.is_empty() {
                return Err(CatalogError::InvalidSlot {
                    hero_id: hero.id.clone(),
                    slot: 0,
                });
            }
            for &slot in &hero.optimal_slots {
                if !(1..=9).contains(&slot) {
                    return Err(CatalogError::InvalidSlot {
                        hero_id: hero.id.clone(),
                        slot,
                    });
                }
            }
            for skill in [&hero.normal_skill, &hero.ultimate_skill] {
                if skill.id.trim().is_empty() || skill.name.trim().is_empty() {
                    return Err(CatalogError::InvalidSkill(hero.id.clone()));
                }
                if !skill_ids.insert(skill.id.as_str()) {
                    return Err(CatalogError::MissingSkillId(skill.id.clone()));
                }
                if skill.damage_rate_bps == 0 || skill.damage_rate_bps > 10_000 {
                    return Err(CatalogError::InvalidSkill(hero.id.clone()));
                }
            }
        }
        Ok(())
    }

    pub fn hero(&self, id: &str) -> Option<&HeroData> {
        self.heroes.iter().find(|hero| hero.id == id)
    }

    /// Stable FNV-1a fingerprint for replay/config compatibility checks.
    pub fn fingerprint(&self) -> Result<String, String> {
        let bytes = serde_json::to_vec(self).map_err(|error| error.to_string())?;
        let hash = bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
        Ok(format!("{hash:016x}"))
    }
}

fn validate_stats(hero: &HeroData) -> Result<(), CatalogError> {
    let stats = &hero.stats;
    if stats.hp == 0
        || stats.atk == 0
        || stats.spd == 0
        || stats.crit_rate_bps > 10_000
        || stats.crit_damage_bps < 1_000
    {
        return Err(CatalogError::InvalidStats(hero.id.clone()));
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BalanceEntry {
    pub id: String,
    pub value: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> ContentCatalog {
        ContentCatalog {
            schema_version: 1,
            config_version: "combat-v1".to_string(),
            heroes: vec![HeroData {
                id: "zhao_yun".to_string(),
                name: "Triệu Vân".to_string(),
                faction: Faction::Shu,
                rarity: Rarity::Ssr,
                role: HeroRole::Warrior,
                optimal_slots: vec![2, 4],
                stats: HeroStats {
                    hp: 3800,
                    atk: 550,
                    def: 300,
                    spd: 105,
                    crit_rate_bps: 1000,
                    crit_damage_bps: 1500,
                },
                normal_skill: SkillData {
                    id: "zhao_yun_normal".to_string(),
                    name: "Thất Tiến Thất Xuất".to_string(),
                    target_rule: "DIRECT_LINE".to_string(),
                    damage_rate_bps: 1000,
                    effects: vec![],
                },
                ultimate_skill: SkillData {
                    id: "zhao_yun_ultimate".to_string(),
                    name: "Thất Tiến Thất Xuất".to_string(),
                    target_rule: "TARGET_LOWEST_HP_RATIO".to_string(),
                    damage_rate_bps: 1800,
                    effects: vec!["DODGE".to_string()],
                },
            }],
        }
    }

    #[test]
    fn catalog_round_trips_and_finds_hero() {
        let source = serde_json::to_string(&catalog()).expect("encode catalog");
        let decoded = ContentCatalog::from_json(&source).expect("valid catalog");
        assert_eq!(decoded.hero("zhao_yun").expect("hero").name, "Triệu Vân");
        assert_eq!(decoded.fingerprint().expect("fingerprint").len(), 16);
    }

    #[test]
    fn catalog_rejects_duplicate_hero_ids() {
        let mut value = catalog();
        value.heroes.push(value.heroes[0].clone());
        assert_eq!(
            value.validate(),
            Err(CatalogError::DuplicateHeroId("zhao_yun".to_string()))
        );
    }

    #[test]
    fn catalog_rejects_invalid_slots() {
        let mut value = catalog();
        value.heroes[0].optimal_slots = vec![10];
        assert!(matches!(
            value.validate(),
            Err(CatalogError::InvalidSlot { slot: 10, .. })
        ));
    }

    #[test]
    fn release_hero_catalog_loads_all_twelve_generals() {
        let source = include_str!("../../../assets/configs/heroes.json");
        let catalog = ContentCatalog::from_json(source).expect("release config");
        assert_eq!(catalog.config_version, "combat-v1");
        assert_eq!(catalog.heroes.len(), 12);
        assert!(catalog.hero("zhao_yun").is_some());
        assert!(catalog.hero("jia_xu").is_some());
    }

    #[test]
    fn test_schema_game_models_round_trip() {
        let rarities = default_rarity_configs();
        assert_eq!(rarities.len(), 4);
        assert_eq!(rarities["SSR"].base_stat_multiplier, 1.50);

        let lines = default_formation_line_configs();
        assert_eq!(lines.len(), 3);
        assert_eq!(
            FormationLineType::from_slot_id(1),
            Some(FormationLineType::Front)
        );
        assert_eq!(
            FormationLineType::from_slot_id(5),
            Some(FormationLineType::Mid)
        );
        assert_eq!(
            FormationLineType::from_slot_id(9),
            Some(FormationLineType::Back)
        );
        assert_eq!(FormationLineType::from_slot_id(10), None);

        let templates = default_master_templates();
        assert_eq!(templates.len(), 13);
        assert_eq!(templates[0].card_template_id, "hero_guan_yu");

        let skills = default_master_skills();
        assert_eq!(skills.len(), 14);
        let json_skill = serde_json::to_string(&skills[0]).expect("serialize skill");
        let restored: SkillRow = serde_json::from_str(&json_skill).expect("deserialize skill");
        assert_eq!(restored.skill_id, "skill_guanyu_ult");
        assert_eq!(restored.effects_applied.len(), 2);

        let mut formation = PlayerFormationRow {
            formation_id: 100,
            user_id: 1,
            formation_type: "ARENA_DEFENSE".to_string(),
            ..Default::default()
        };
        formation.set_slot_card_id(2, Some(42));
        formation.set_slot_card_id(7, Some(84));
        assert_eq!(formation.get_slot_card_id(2), Some(42));
        assert_eq!(formation.get_slot_card_id(7), Some(84));
        assert_eq!(formation.get_slot_card_id(1), None);
        assert_eq!(formation.slots_as_vec().len(), 9);
    }
}
