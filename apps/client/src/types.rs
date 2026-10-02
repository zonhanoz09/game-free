use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const GRID_COLS: usize = game_core::BOARD_WIDTH;
pub const GRID_ROWS: usize = game_core::BOARD_HEIGHT;
pub const MAX_PLAYER_UNITS: usize = 5;
pub const BENCH_SLOTS: usize = 6;

// 2D Pixel Layout Constants
pub const TILE_SIZE: f32 = 88.0;
pub const TILE_GAP: f32 = 14.0;
pub const ARENA_CENTER_X: f32 = -60.0;
pub const ARENA_CENTER_Y: f32 = 30.0;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Placement,
    Battle,
    Victory,
    Defeat,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Faction {
    Player,
    Enemy,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum UnitClass {
    Knight,
    Archer,
    Mage,
    Assassin,
    Cleric,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct UnitStats {
    pub max_hp: f32,
    pub hp: f32,
    pub mana: f32,
    pub max_mana: f32,
    pub atk: f32,
    pub def: f32,
    pub speed: f32,
    pub crit_rate: f32,
    pub shield: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridPos {
    pub col: usize,
    pub row: usize,
    pub faction: Faction,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BenchPos {
    pub slot: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitLocation {
    Board(GridPos),
    Bench(usize),
}

#[derive(Component)]
pub struct DeadUnit;

#[derive(Resource)]
pub struct BattleSpeed {
    pub multiplier: f32,
}

impl Default for BattleSpeed {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

#[derive(Resource)]
pub struct CurrentStage {
    pub stage_idx: usize,
}

impl Default for CurrentStage {
    fn default() -> Self {
        Self { stage_idx: 1 }
    }
}

#[allow(dead_code)]
#[derive(Resource, Default)]
pub struct SelectedBenchUnit {
    pub unit_class: Option<UnitClass>,
}

#[derive(Resource, Default, Debug)]
pub struct SelectedUnitState {
    pub entity: Option<Entity>,
    pub location: Option<UnitLocation>,
    pub class: Option<UnitClass>,
}

impl SelectedUnitState {
    pub fn clear(&mut self) {
        self.entity = None;
        self.location = None;
        self.class = None;
    }
}

#[allow(dead_code)]
#[derive(Resource, Default)]
pub struct InspectedUnitInfo {
    pub class: Option<UnitClass>,
    pub faction: Option<Faction>,
    pub stats: Option<UnitStats>,
}

impl UnitClass {
    pub fn name(&self) -> &'static str {
        match self {
            UnitClass::Knight => "Knight",
            UnitClass::Archer => "Archer",
            UnitClass::Mage => "Mage",
            UnitClass::Assassin => "Assassin",
            UnitClass::Cleric => "Cleric",
        }
    }

    pub fn role_title(&self) -> &'static str {
        match self {
            UnitClass::Knight => "Frontline Iron Vanguard (Tank)",
            UnitClass::Archer => "Long-Range Sniper (Physical Carry)",
            UnitClass::Mage => "Arcane Elementalist (Row AoE Burst)",
            UnitClass::Assassin => "Shadow Blade (Backline Infiltrator)",
            UnitClass::Cleric => "High Priestess (Divine Support)",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            UnitClass::Knight => "[Knight]",
            UnitClass::Archer => "[Archer]",
            UnitClass::Mage => "[Mage]",
            UnitClass::Assassin => "[Assassin]",
            UnitClass::Cleric => "[Cleric]",
        }
    }

    pub fn skill_name(&self) -> &'static str {
        match self {
            UnitClass::Knight => "Iron Bulwark & Cleave",
            UnitClass::Archer => "Eagle Piercing Shot",
            UnitClass::Mage => "Chain Arc Lightning",
            UnitClass::Assassin => "Shadow Void Strike",
            UnitClass::Cleric => "Divine Celestial Sanctuary",
        }
    }

    pub fn skill_type(&self) -> &'static str {
        match self {
            UnitClass::Knight => "Melee Cleave & Armor Buff",
            UnitClass::Archer => "Precision Snipe & Execute",
            UnitClass::Mage => "Row-Wide Arcane AoE",
            UnitClass::Assassin => "Backline Ambush & Crit",
            UnitClass::Cleric => "Holy Light Pillar Heal",
        }
    }

    pub fn skill_description(&self) -> &'static str {
        match self {
            UnitClass::Knight => {
                "Leaps forward with heavy shield bash, slashing with luminous steel blade. Mitigates high damage through fortified defense."
            }
            UnitClass::Archer => {
                "Snipes the lowest-health enemy across the arena with high projectile speed and critical hit chance."
            }
            UnitClass::Mage => {
                "Casts an arcane lightning orb that shocks the primary target and splashes explosive shockwave damage to the entire enemy row."
            }
            UnitClass::Assassin => {
                "Teleports through shadows directly behind enemy lines to assassinate the deepest, weakest backline unit with twin venom blades."
            }
            UnitClass::Cleric => {
                "Summons a celestial pillar of holy light upon the most wounded ally on the battlefield, restoring significant health."
            }
        }
    }

    pub fn ultimate_name(&self) -> &'static str {
        match self {
            UnitClass::Knight => "Aegis Fortress",
            UnitClass::Archer => "Arrow Barrage",
            UnitClass::Mage => "Judgment Thunderstorm",
            UnitClass::Assassin => "Shadow Execution",
            UnitClass::Cleric => "Divine Benediction",
        }
    }

    pub fn ultimate_desc(&self) -> &'static str {
        match self {
            UnitClass::Knight => {
                "Slams a giant shockwave barrier, dealing 220% ATK damage, gaining 80 Shield and disrupting target Action Gauge."
            }
            UnitClass::Archer => {
                "Leaps backwards and rains down 5 piercing arrows upon all living enemies with +50% bonus Crit Rate."
            }
            UnitClass::Mage => {
                "Summons apocalyptic lightning across the entire enemy arena, dealing 160% ATK AoE magic damage to all living foes."
            }
            UnitClass::Assassin => {
                "Teleports behind the weakest enemy and executes a lethal 3-strike flurry for 280% ATK damage, restoring 50 Mana on kill."
            }
            UnitClass::Cleric => {
                "Calls down heavenly grace, healing all allies for 160% ATK + 45 HP and accelerating their Action Gauge by +25%."
            }
        }
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            UnitClass::Knight => "Vanguard with massive HP & DEF. Absorbs frontline pressure.",
            UnitClass::Archer => "Snipes lowest HP targets across the arena with lethal precision.",
            UnitClass::Mage => "Discharges arcane lightning balls that shock entire enemy rows.",
            UnitClass::Assassin => "Teleports into shadows to strike the weakest backline target.",
            UnitClass::Cleric => "Summons holy light to heal the most severely injured ally.",
        }
    }

    pub fn base_stats(&self) -> UnitStats {
        match self {
            UnitClass::Knight => UnitStats {
                max_hp: 180.0,
                hp: 180.0,
                mana: 0.0,
                max_mana: 100.0,
                atk: 25.0,
                def: 40.0,
                speed: 18.0,
                crit_rate: 0.10,
                shield: 0.0,
            },
            UnitClass::Archer => UnitStats {
                max_hp: 110.0,
                hp: 110.0,
                mana: 0.0,
                max_mana: 100.0,
                atk: 38.0,
                def: 15.0,
                speed: 26.0,
                crit_rate: 0.35,
                shield: 0.0,
            },
            UnitClass::Mage => UnitStats {
                max_hp: 95.0,
                hp: 95.0,
                mana: 0.0,
                max_mana: 100.0,
                atk: 45.0,
                def: 10.0,
                speed: 22.0,
                crit_rate: 0.20,
                shield: 0.0,
            },
            UnitClass::Assassin => UnitStats {
                max_hp: 100.0,
                hp: 100.0,
                mana: 0.0,
                max_mana: 100.0,
                atk: 50.0,
                def: 12.0,
                speed: 34.0,
                crit_rate: 0.45,
                shield: 0.0,
            },
            UnitClass::Cleric => UnitStats {
                max_hp: 125.0,
                hp: 125.0,
                mana: 0.0,
                max_mana: 100.0,
                atk: 20.0,
                def: 22.0,
                speed: 20.0,
                crit_rate: 0.05,
                shield: 0.0,
            },
        }
    }

    pub fn color(&self) -> Color {
        match self {
            UnitClass::Knight => Color::srgb(0.2, 0.45, 0.85),
            UnitClass::Archer => Color::srgb(0.2, 0.75, 0.3),
            UnitClass::Mage => Color::srgb(0.65, 0.25, 0.85),
            UnitClass::Assassin => Color::srgb(0.85, 0.2, 0.25),
            UnitClass::Cleric => Color::srgb(0.95, 0.82, 0.25),
        }
    }
}

#[derive(Resource)]
pub struct GameTextures {
    pub background: Handle<Image>,
    pub knight: Handle<Image>,
    pub archer: Handle<Image>,
    pub mage: Handle<Image>,
    pub assassin: Handle<Image>,
    pub cleric: Handle<Image>,
}

impl FromWorld for GameTextures {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            background: asset_server.load("textures/background.png"),
            knight: asset_server.load("textures/knight.png"),
            archer: asset_server.load("textures/archer.png"),
            mage: asset_server.load("textures/mage.png"),
            assassin: asset_server.load("textures/assassin.png"),
            cleric: asset_server.load("textures/cleric.png"),
        }
    }
}

#[derive(Resource, Clone)]
pub struct GameFonts {
    pub regular: Handle<Font>,
    pub bold: Handle<Font>,
}

impl FromWorld for GameFonts {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            regular: asset_server.load("fonts/font.ttf"),
            bold: asset_server.load("fonts/font_bold.ttf"),
        }
    }
}

impl GameFonts {
    #[allow(dead_code)]
    pub fn dummy() -> Self {
        Self {
            regular: Handle::default(),
            bold: Handle::default(),
        }
    }
}

impl GameTextures {
    #[allow(dead_code)]
    pub fn dummy() -> Self {
        Self {
            background: Handle::default(),
            knight: Handle::default(),
            archer: Handle::default(),
            mage: Handle::default(),
            assassin: Handle::default(),
            cleric: Handle::default(),
        }
    }

    pub fn get_unit_texture(&self, class: UnitClass) -> Handle<Image> {
        match class {
            UnitClass::Knight => self.knight.clone(),
            UnitClass::Archer => self.archer.clone(),
            UnitClass::Mage => self.mage.clone(),
            UnitClass::Assassin => self.assassin.clone(),
            UnitClass::Cleric => self.cleric.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hero_classes_stats_and_mana() {
        let classes = [
            UnitClass::Knight,
            UnitClass::Archer,
            UnitClass::Mage,
            UnitClass::Assassin,
            UnitClass::Cleric,
        ];

        for class in classes {
            let stats = class.base_stats();
            assert!(stats.max_hp > 0.0);
            assert_eq!(stats.hp, stats.max_hp);
            assert_eq!(stats.mana, 0.0);
            assert_eq!(stats.max_mana, 100.0);
            assert_eq!(stats.shield, 0.0);
            assert!(stats.atk > 0.0);
            assert!(stats.speed > 0.0);
            assert!(!class.ultimate_name().is_empty());
            assert!(!class.ultimate_desc().is_empty());
            assert!(!class.name().is_empty());
        }
    }

    #[test]
    fn test_ultimate_skills_distinct() {
        assert_eq!(UnitClass::Knight.ultimate_name(), "Aegis Fortress");
        assert_eq!(UnitClass::Archer.ultimate_name(), "Arrow Barrage");
        assert_eq!(UnitClass::Mage.ultimate_name(), "Judgment Thunderstorm");
        assert_eq!(UnitClass::Assassin.ultimate_name(), "Shadow Execution");
        assert_eq!(UnitClass::Cleric.ultimate_name(), "Divine Benediction");
    }
}
