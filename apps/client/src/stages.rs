#![allow(dead_code)]

use crate::types::*;

pub struct EnemyUnitDef {
    pub unit_class: UnitClass,
    pub col: usize,
    pub row: usize,
    pub is_boss: bool,
    pub star_level: u8,
}

impl EnemyUnitDef {
    pub const fn normal(unit_class: UnitClass, col: usize, row: usize) -> Self {
        Self {
            unit_class,
            col,
            row,
            is_boss: false,
            star_level: 1,
        }
    }

    pub const fn elite(unit_class: UnitClass, col: usize, row: usize, star_level: u8) -> Self {
        Self {
            unit_class,
            col,
            row,
            is_boss: false,
            star_level,
        }
    }

    pub const fn boss(unit_class: UnitClass, col: usize, row: usize) -> Self {
        Self {
            unit_class,
            col,
            row,
            is_boss: true,
            star_level: 3,
        }
    }
}

pub struct StageDef {
    pub title: String,
    pub description: String,
    pub enemies: Vec<EnemyUnitDef>,
}

pub fn get_stage_def(stage: usize) -> StageDef {
    match stage {
        1 => StageDef {
            title: "Stage 1: Vanguard Frontline".to_string(),
            description: "A frontline Knight shields an Archer and a Mage behind him.".to_string(),
            enemies: vec![
                EnemyUnitDef::normal(UnitClass::Knight, 0, 1),
                EnemyUnitDef::normal(UnitClass::Archer, 1, 0),
                EnemyUnitDef::normal(UnitClass::Mage, 1, 2),
            ],
        },
        2 => StageDef {
            title: "Stage 2: Twin Lancers & Cleric".to_string(),
            description: "Two Knights supported by a backline Cleric's persistent healing."
                .to_string(),
            enemies: vec![
                EnemyUnitDef::normal(UnitClass::Knight, 0, 0),
                EnemyUnitDef::normal(UnitClass::Knight, 0, 2),
                EnemyUnitDef::normal(UnitClass::Cleric, 1, 1),
            ],
        },
        3 => StageDef {
            title: "Stage 3: Shadow Ambush".to_string(),
            description: "Two Assassins flanking an Archer - watch your backline!".to_string(),
            enemies: vec![
                EnemyUnitDef::normal(UnitClass::Assassin, 0, 0),
                EnemyUnitDef::normal(UnitClass::Assassin, 0, 2),
                EnemyUnitDef::normal(UnitClass::Archer, 1, 1),
            ],
        },
        4 => StageDef {
            title: "Stage 4: Arcane Storm".to_string(),
            description: "A 2★ Knight protects two high-damage Mages. Spread out!".to_string(),
            enemies: vec![
                EnemyUnitDef::elite(UnitClass::Knight, 0, 1, 2),
                EnemyUnitDef::normal(UnitClass::Mage, 1, 0),
                EnemyUnitDef::normal(UnitClass::Mage, 1, 2),
            ],
        },
        5 => StageDef {
            title: "Stage 5: BOSS - Grand Warlord".to_string(),
            description: "A 3★ Boss Knight with an Archer and Cleric. Brace yourself!".to_string(),
            enemies: vec![
                EnemyUnitDef::boss(UnitClass::Knight, 0, 1),
                EnemyUnitDef::normal(UnitClass::Archer, 1, 0),
                EnemyUnitDef::normal(UnitClass::Cleric, 1, 2),
            ],
        },
        _ => {
            let cycle = (stage - 1) % 5 + 1;
            let star_boost = ((stage - 1) / 5) as u8;
            let mut def = get_stage_def(cycle);
            def.title = format!("Stage {} (Ascension {})", stage, star_boost + 1);
            for e in &mut def.enemies {
                e.star_level = (e.star_level + star_boost).min(3);
            }
            def
        }
    }
}
