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
                EnemyUnitDef::normal(UnitClass::Archer, 2, 0),
                EnemyUnitDef::normal(UnitClass::Mage, 2, 2),
            ],
        },
        2 => StageDef {
            title: "Stage 2: Ambush Crossfire".to_string(),
            description: "Two armored Knights form an iron barricade protecting dual snipers.".to_string(),
            enemies: vec![
                EnemyUnitDef::normal(UnitClass::Knight, 0, 0),
                EnemyUnitDef::normal(UnitClass::Knight, 0, 2),
                EnemyUnitDef::normal(UnitClass::Archer, 2, 0),
                EnemyUnitDef::normal(UnitClass::Archer, 2, 2),
            ],
        },
        3 => StageDef {
            title: "Stage 3: Shadow Assassins".to_string(),
            description: "Knights hold the front while twin Assassins prepare to flank your backline!".to_string(),
            enemies: vec![
                EnemyUnitDef::normal(UnitClass::Knight, 0, 1),
                EnemyUnitDef::normal(UnitClass::Assassin, 1, 0),
                EnemyUnitDef::normal(UnitClass::Assassin, 1, 2),
                EnemyUnitDef::normal(UnitClass::Cleric, 2, 1),
            ],
        },
        4 => StageDef {
            title: "Stage 4: Arcane Coven".to_string(),
            description: "Twin Archmages discharge row-wide lightning empowered by a Cleric's prayers.".to_string(),
            enemies: vec![
                EnemyUnitDef::normal(UnitClass::Knight, 0, 0),
                EnemyUnitDef::normal(UnitClass::Knight, 0, 2),
                EnemyUnitDef::normal(UnitClass::Mage, 1, 1),
                EnemyUnitDef::normal(UnitClass::Mage, 2, 0),
                EnemyUnitDef::normal(UnitClass::Cleric, 2, 2),
            ],
        },
        5 => StageDef {
            title: "Stage 5: Imperial Guard (Mini-Boss)".to_string(),
            description: "Hardened 2-Star veterans led by a deadly Vanguard commander!".to_string(),
            enemies: vec![
                EnemyUnitDef::elite(UnitClass::Knight, 0, 0, 2),
                EnemyUnitDef::elite(UnitClass::Knight, 0, 2, 2),
                EnemyUnitDef::elite(UnitClass::Assassin, 1, 1, 2),
                EnemyUnitDef::elite(UnitClass::Archer, 2, 0, 2),
                EnemyUnitDef::elite(UnitClass::Mage, 2, 2, 2),
            ],
        },
        6 => StageDef {
            title: "Stage 6: Nether Stalkers".to_string(),
            description: "A lethal flock of Assassins waiting in the dark to strike at once.".to_string(),
            enemies: vec![
                EnemyUnitDef::normal(UnitClass::Knight, 0, 1),
                EnemyUnitDef::normal(UnitClass::Assassin, 1, 0),
                EnemyUnitDef::normal(UnitClass::Assassin, 1, 1),
                EnemyUnitDef::normal(UnitClass::Assassin, 1, 2),
                EnemyUnitDef::normal(UnitClass::Archer, 2, 1),
            ],
        },
        7 => StageDef {
            title: "Stage 7: Holy Phalanx".to_string(),
            description: "Double Cleric celestial wards backing up heavy plate Knights and dual snipers.".to_string(),
            enemies: vec![
                EnemyUnitDef::elite(UnitClass::Knight, 0, 0, 2),
                EnemyUnitDef::elite(UnitClass::Knight, 0, 2, 2),
                EnemyUnitDef::normal(UnitClass::Cleric, 1, 0),
                EnemyUnitDef::normal(UnitClass::Cleric, 1, 2),
                EnemyUnitDef::elite(UnitClass::Archer, 2, 1, 2),
            ],
        },
        8 => StageDef {
            title: "Stage 8: Apocalyptic Storm".to_string(),
            description: "Triple Archmages preparing to incinerate the arena with apocalyptic lightning.".to_string(),
            enemies: vec![
                EnemyUnitDef::elite(UnitClass::Knight, 0, 1, 2),
                EnemyUnitDef::normal(UnitClass::Mage, 1, 0),
                EnemyUnitDef::normal(UnitClass::Mage, 1, 2),
                EnemyUnitDef::elite(UnitClass::Mage, 2, 1, 2),
                EnemyUnitDef::normal(UnitClass::Cleric, 2, 2),
            ],
        },
        9 => StageDef {
            title: "Stage 9: Royal Vanguard Legion".to_string(),
            description: "A complete frontline of 2-Star and 3-Star champions defending the Boss Gate.".to_string(),
            enemies: vec![
                EnemyUnitDef::elite(UnitClass::Knight, 0, 0, 2),
                EnemyUnitDef::elite(UnitClass::Knight, 0, 1, 3),
                EnemyUnitDef::elite(UnitClass::Knight, 0, 2, 2),
                EnemyUnitDef::elite(UnitClass::Assassin, 1, 1, 2),
                EnemyUnitDef::elite(UnitClass::Archer, 2, 0, 2),
                EnemyUnitDef::elite(UnitClass::Mage, 2, 2, 2),
            ],
        },
        10 => StageDef {
            title: "Stage 10: Colossal Abyssal Titan (GIANT BOSS)".to_string(),
            description: "The supreme 2x2 Giant Boss Titan! Massive HP, destructive AoE slam and double holy aids!".to_string(),
            enemies: vec![
                EnemyUnitDef::boss(UnitClass::Knight, 0, 1),
                EnemyUnitDef::elite(UnitClass::Cleric, 2, 0, 2),
                EnemyUnitDef::elite(UnitClass::Cleric, 2, 2, 2),
                EnemyUnitDef::elite(UnitClass::Mage, 2, 1, 2),
            ],
        },
        wave => {
            let extra_hp_stars = ((wave - 10) / 3).min(2) as u8 + 1;
            StageDef {
                title: format!("Endless Wave {}: Abyssal Incursion", wave),
                description: format!("Endless wave scaling! Enemies have Star Level {} and increased parameters.", extra_hp_stars),
                enemies: vec![
                    EnemyUnitDef::elite(UnitClass::Knight, 0, 0, extra_hp_stars),
                    EnemyUnitDef::elite(UnitClass::Knight, 0, 2, extra_hp_stars),
                    EnemyUnitDef::elite(UnitClass::Assassin, 1, 1, extra_hp_stars),
                    EnemyUnitDef::elite(UnitClass::Archer, 2, 0, extra_hp_stars),
                    EnemyUnitDef::elite(UnitClass::Mage, 2, 2, extra_hp_stars),
                ],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_1_to_10_and_endless() {
        for s in 1..=10 {
            let def = get_stage_def(s);
            assert!(!def.title.is_empty());
            assert!(!def.enemies.is_empty());
        }
        let boss_stage = get_stage_def(10);
        assert!(boss_stage.enemies.iter().any(|e| e.is_boss));

        let endless_wave = get_stage_def(15);
        assert!(endless_wave.title.contains("Endless Wave 15"));
    }
}
