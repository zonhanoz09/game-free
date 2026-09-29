use bevy::prelude::*;

pub const TILE_SIZE: f32 = 1.35;
pub const TILE_GAP: f32 = 0.18;
pub const TILE_HEIGHT: f32 = 0.15;
pub const GRID_COLS: usize = 3;
pub const GRID_ROWS: usize = 3;
pub const MAX_PLAYER_UNITS: usize = 5;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Placement,
    Battle,
    Victory,
    Defeat,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Faction {
    Player,
    Enemy,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
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
    pub atk: f32,
    pub def: f32,
    pub speed: f32,
    pub crit_rate: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridPos {
    pub col: usize,
    pub row: usize,
    pub faction: Faction,
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

#[derive(Resource, Default)]
pub struct SelectedBenchUnit {
    pub unit_class: Option<UnitClass>,
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
                atk: 25.0,
                def: 40.0,
                speed: 18.0,
                crit_rate: 0.10,
            },
            UnitClass::Archer => UnitStats {
                max_hp: 110.0,
                hp: 110.0,
                atk: 38.0,
                def: 15.0,
                speed: 26.0,
                crit_rate: 0.35,
            },
            UnitClass::Mage => UnitStats {
                max_hp: 95.0,
                hp: 95.0,
                atk: 45.0,
                def: 10.0,
                speed: 22.0,
                crit_rate: 0.20,
            },
            UnitClass::Assassin => UnitStats {
                max_hp: 100.0,
                hp: 100.0,
                atk: 50.0,
                def: 12.0,
                speed: 34.0,
                crit_rate: 0.45,
            },
            UnitClass::Cleric => UnitStats {
                max_hp: 125.0,
                hp: 125.0,
                atk: 20.0,
                def: 22.0,
                speed: 20.0,
                crit_rate: 0.05,
            },
        }
    }

    pub fn color(&self) -> Color {
        match self {
            UnitClass::Knight => Color::srgb(0.2, 0.4, 0.8),
            UnitClass::Archer => Color::srgb(0.2, 0.7, 0.2),
            UnitClass::Mage => Color::srgb(0.6, 0.2, 0.8),
            UnitClass::Assassin => Color::srgb(0.8, 0.2, 0.2),
            UnitClass::Cleric => Color::srgb(0.9, 0.8, 0.2),
        }
    }
}

#[derive(Resource)]
pub struct GameTextures {
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
            knight: asset_server.load("textures/knight.png"),
            archer: asset_server.load("textures/archer.png"),
            mage: asset_server.load("textures/mage.png"),
            assassin: asset_server.load("textures/assassin.png"),
            cleric: asset_server.load("textures/cleric.png"),
        }
    }
}

impl GameTextures {
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
