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
            title: "MÃ n 1: Äá»™i TiÃªn Phong",
            description: "1 Hiá»‡p sÄ© hÃ ng Ä‘áº§u báº£o vá»‡ 1 Cung thá»§ vÃ  1 PhÃ¡p sÆ° phÃ­a sau.",
            enemies: vec![
                EnemyUnitDef { unit_class: UnitClass::Knight, col: 0, row: 1 },
                EnemyUnitDef { unit_class: UnitClass::Archer, col: 2, row: 0 },
                EnemyUnitDef { unit_class: UnitClass::Mage,   col: 2, row: 2 },
            ],
        },
        2 => StageDef {
            title: "MÃ n 2: Cung Tiá»…n Mai Phá»¥c",
            description: "2 Hiá»‡p sÄ© táº¡o bá»©c tÆ°á»ng thÃ©p che cháº¯n cho 2 Cung thá»§ báº¯n tá»‰a hiá»ƒm hÃ³c.",
            enemies: vec![
                EnemyUnitDef { unit_class: UnitClass::Knight, col: 0, row: 0 },
                EnemyUnitDef { unit_class: UnitClass::Knight, col: 0, row: 2 },
                EnemyUnitDef { unit_class: UnitClass::Archer, col: 2, row: 0 },
                EnemyUnitDef { unit_class: UnitClass::Archer, col: 2, row: 2 },
            ],
        },
        3 => StageDef {
            title: "MÃ n 3: BÃ³ng ÄÃªm SÃ¡t Thá»§",
            description: "Hiá»‡p sÄ© giáº±ng co hÃ ng trÆ°á»›c trong khi 2 SÃ¡t thá»§ chá»±c chá» láº»n vÃ o Ã¡m sÃ¡t hÃ ng sau cá»§a báº¡n!",
            enemies: vec![
                EnemyUnitDef { unit_class: UnitClass::Knight,   col: 0, row: 1 },
                EnemyUnitDef { unit_class: UnitClass::Assassin, col: 1, row: 0 },
                EnemyUnitDef { unit_class: UnitClass::Assassin, col: 1, row: 2 },
                EnemyUnitDef { unit_class: UnitClass::Cleric,   col: 2, row: 1 },
            ],
        },
        4 => StageDef {
            title: "MÃ n 4: Há»™i Äá»“ng Ma PhÃ¡p",
            description: "CÃ¡c PhÃ¡p sÆ° ma thuáº­t sáº¥m sÃ©t lan tá»a cÃ¹ng Má»¥c sÆ° liÃªn tá»¥c há»“i mÃ¡u.",
            enemies: vec![
                EnemyUnitDef { unit_class: UnitClass::Knight, col: 0, row: 0 },
                EnemyUnitDef { unit_class: UnitClass::Knight, col: 0, row: 2 },
                EnemyUnitDef { unit_class: UnitClass::Mage,   col: 1, row: 1 },
                EnemyUnitDef { unit_class: UnitClass::Mage,   col: 2, row: 0 },
                EnemyUnitDef { unit_class: UnitClass::Cleric, col: 2, row: 2 },
            ],
        },
        _ => StageDef {
            title: "MÃ n 5: QuÃ¢n ÄoÃ n HoÃ ng Kim (TrÃ¹m Cuá»‘i)",
            description: "HÃ ng thá»§ 3 Hiá»‡p sÄ© báº¥t kháº£ xÃ¢m pháº¡m, há»— trá»£ bá»Ÿi SÃ¡t thá»§, PhÃ¡p sÆ° vÃ  Cung thá»§ thiá»‡n xáº¡!",
            enemies: vec![
                EnemyUnitDef { unit_class: UnitClass::Knight,   col: 0, row: 0 },
                EnemyUnitDef { unit_class: UnitClass::Knight,   col: 0, row: 1 },
                EnemyUnitDef { unit_class: UnitClass::Knight,   col: 0, row: 2 },
                EnemyUnitDef { unit_class: UnitClass::Assassin, col: 1, row: 1 },
                EnemyUnitDef { unit_class: UnitClass::Mage,     col: 2, row: 0 },
                EnemyUnitDef { unit_class: UnitClass::Archer,   col: 2, row: 2 },
            ],
        },
    }
}
