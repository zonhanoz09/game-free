use bevy::prelude::*;

pub const BOARD_COLS: usize = 3;
pub const BOARD_ROWS: usize = 3;

// 3D Board Layout Constants
pub const TILE_SIZE: f32 = 1.7;
pub const TILE_HEIGHT: f32 = 0.14;

pub const PLAYER_COL_X: [f32; 3] = [-5.4, -3.4, -1.4]; // Hau (0), Trung (1), Tien (2)
pub const ENEMY_COL_X: [f32; 3] = [1.4, 3.4, 5.4]; // Tien (0), Trung (1), Hau (2)
pub const ROW_Z: [f32; 3] = [-2.3, 0.0, 2.3];

pub const TILE_SURFACE_Y: f32 = 0.14;
pub const UNIT_BASE_Y: f32 = 0.14;

pub const MAX_PLAYER_UNITS: usize = 5;

#[derive(States, Clone, Copy, Eq, PartialEq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Placement,
    Battle,
    Victory,
    Defeat,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Component, Reflect)]
pub enum Faction {
    Player,
    Enemy,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Component, Reflect)]
pub enum UnitClass {
    Knight,   // Hiep Si: Tanker, can chien, giap day
    Archer,   // Cung Thu: Ban xa, tia muc tieu yeu nhat
    Mage,     // Phap Su: Sat thuong phep lan toan hang
    Assassin, // Sat Thu: Nhay ra sau am sat, bao kich cao
    Cleric,   // Muc Su: Hoi phuc dong minh nguy cap
}

impl UnitClass {
    pub fn name_vi(&self) -> &'static str {
        match self {
            UnitClass::Knight => "Hiệp Sĩ",
            UnitClass::Archer => "Cung Thủ",
            UnitClass::Mage => "Pháp Sư",
            UnitClass::Assassin => "Sát Thủ",
            UnitClass::Cleric => "Mục Sư",
        }
    }

    #[allow(dead_code)]
    pub fn code(&self) -> &'static str {
        match self {
            UnitClass::Knight => "KNG",
            UnitClass::Archer => "ARC",
            UnitClass::Mage => "MAG",
            UnitClass::Assassin => "ASN",
            UnitClass::Cleric => "CLR",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            UnitClass::Knight => "[Kiếm-Khiên]",
            UnitClass::Archer => "[Cung-Tên]",
            UnitClass::Mage => "[Phép-Thuật]",
            UnitClass::Assassin => "[Song-Đao]",
            UnitClass::Cleric => "[Thánh-Điện]",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            UnitClass::Knight => Color::srgb(0.23, 0.51, 0.96), // Blue
            UnitClass::Archer => Color::srgb(0.13, 0.77, 0.37), // Green
            UnitClass::Mage => Color::srgb(0.66, 0.33, 0.97),   // Violet
            UnitClass::Assassin => Color::srgb(0.94, 0.27, 0.27), // Crimson
            UnitClass::Cleric => Color::srgb(0.96, 0.72, 0.15), // Amber/Gold
        }
    }

    pub fn base_stats(&self) -> UnitStats {
        match self {
            UnitClass::Knight => UnitStats {
                max_hp: 170.0,
                hp: 170.0,
                atk: 24.0,
                def: 9.0,
                speed: 10.0,
                range: 1,
                crit_rate: 0.05,
            },
            UnitClass::Archer => UnitStats {
                max_hp: 80.0,
                hp: 80.0,
                atk: 34.0,
                def: 2.0,
                speed: 14.0,
                range: 9,
                crit_rate: 0.22,
            },
            UnitClass::Mage => UnitStats {
                max_hp: 70.0,
                hp: 70.0,
                atk: 30.0,
                def: 1.0,
                speed: 9.5,
                range: 9,
                crit_rate: 0.10,
            },
            UnitClass::Assassin => UnitStats {
                max_hp: 85.0,
                hp: 85.0,
                atk: 42.0,
                def: 3.0,
                speed: 18.0,
                range: 1,
                crit_rate: 0.38,
            },
            UnitClass::Cleric => UnitStats {
                max_hp: 75.0,
                hp: 75.0,
                atk: 18.0,
                def: 3.0,
                speed: 11.5,
                range: 9,
                crit_rate: 0.05,
            },
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            UnitClass::Knight => "Hàng trước kiên cố, giáp dày và khiên lớn thu hút hỏa lực địch.",
            UnitClass::Archer => "Bắn tên chính xác tầm xa, ưu tiên hạ mục tiêu ít máu nhất.",
            UnitClass::Mage => "Cầu ma pháp nổ sét gây sát thương lan cho toàn bộ hàng địch.",
            UnitClass::Assassin => "Tốc độ chớp nhoáng, nhảy thẳng ra hậu phương ám sát hàng sau.",
            UnitClass::Cleric => "Cầu nguyện ánh sáng hồi phục sinh lực cho đồng đội yếu nhất.",
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct UnitStats {
    pub max_hp: f32,
    pub hp: f32,
    pub atk: f32,
    pub def: f32,
    pub speed: f32,
    pub range: usize,
    pub crit_rate: f32,
}

#[derive(Component, Clone, Copy, Eq, PartialEq, Hash, Debug, Reflect)]
pub struct GridPos {
    pub col: usize, // 0..3
    pub row: usize, // 0..3
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

#[derive(Resource, Clone)]
pub struct GameTextures {
    #[allow(dead_code)]
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
        GameTextures {
            background: asset_server.load("textures/background.png"),
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
