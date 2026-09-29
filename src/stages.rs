use crate::types::*;

pub struct EnemyUnitDef {
    pub unit_class: UnitClass,
    pub col: usize,
    pub row: usize,
}

pub struct StageDef {
    pub title: &'static str,
    pub description: &'static str,
    pub enemies: Vec<EnemyUnitDef>,
}

pub fn get_stage_def(stage: usize) -> StageDef {
    match stage {
        1 => StageDef {
            title: "Stage 1: Vanguard Frontline",
            description: "A frontline Knight shields an Archer and a Mage behind him.",
            enemies: vec![
                EnemyUnitDef {
                    unit_class: UnitClass::Knight,
                    col: 0,
                    row: 1,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Archer,
                    col: 2,
                    row: 0,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Mage,
                    col: 2,
                    row: 2,
                },
            ],
        },
        2 => StageDef {
            title: "Stage 2: Ambush Crossfire",
            description: "Two armored Knights form an iron barricade protecting dual snipers.",
            enemies: vec![
                EnemyUnitDef {
                    unit_class: UnitClass::Knight,
                    col: 0,
                    row: 0,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Knight,
                    col: 0,
                    row: 2,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Archer,
                    col: 2,
                    row: 0,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Archer,
                    col: 2,
                    row: 2,
                },
            ],
        },
        3 => StageDef {
            title: "Stage 3: Shadow Assassins",
            description: "Knights hold the front while twin Assassins prepare to flank your backline!",
            enemies: vec![
                EnemyUnitDef {
                    unit_class: UnitClass::Knight,
                    col: 0,
                    row: 1,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Assassin,
                    col: 1,
                    row: 0,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Assassin,
                    col: 1,
                    row: 2,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Cleric,
                    col: 2,
                    row: 1,
                },
            ],
        },
        4 => StageDef {
            title: "Stage 4: Arcane Coven",
            description: "Twin Archmages discharge row-wide lightning empowered by a Cleric's prayers.",
            enemies: vec![
                EnemyUnitDef {
                    unit_class: UnitClass::Knight,
                    col: 0,
                    row: 0,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Knight,
                    col: 0,
                    row: 2,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Mage,
                    col: 1,
                    row: 1,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Mage,
                    col: 2,
                    row: 0,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Cleric,
                    col: 2,
                    row: 2,
                },
            ],
        },
        _ => StageDef {
            title: "Stage 5: The Golden Legion (Final Boss)",
            description: "An impenetrable 3-Knight vanguard supported by an elite Assassin, Mage, and Sniper!",
            enemies: vec![
                EnemyUnitDef {
                    unit_class: UnitClass::Knight,
                    col: 0,
                    row: 0,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Knight,
                    col: 0,
                    row: 1,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Knight,
                    col: 0,
                    row: 2,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Assassin,
                    col: 1,
                    row: 1,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Mage,
                    col: 2,
                    row: 0,
                },
                EnemyUnitDef {
                    unit_class: UnitClass::Archer,
                    col: 2,
                    row: 2,
                },
            ],
        },
    }
}
