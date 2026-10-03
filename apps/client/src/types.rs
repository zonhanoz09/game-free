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
    Knight,         // Triệu Vân (zhao_yun)
    Archer,         // Hoàng Trung (huang_zhong)
    Mage,           // Gia Cát Lượng (zhuge_liang)
    Assassin,       // Trương Cáp & Nhan Lương (zhang_he_yan_liang)
    Cleric,         // Hoa Đà (hua_tuo)
    CaoCao,         // Tào Tháo (cao_cao)
    DianWei,        // Điển Vi (dian_wei)
    GuoJia,         // Quách Gia (guo_jia)
    SunCe,          // Tôn Sách (sun_ce)
    LuXun,          // Lục Tốn (lu_xun)
    DaQiaoXiaoQiao, // Đại Kiều & Tiểu Kiều (da_qiao_xiao_qiao)
    JiaXu,          // Giả Hủ (jia_xu)
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
    pub const ALL: [UnitClass; 12] = [
        UnitClass::Knight,
        UnitClass::Archer,
        UnitClass::Mage,
        UnitClass::Assassin,
        UnitClass::Cleric,
        UnitClass::CaoCao,
        UnitClass::DianWei,
        UnitClass::GuoJia,
        UnitClass::SunCe,
        UnitClass::LuXun,
        UnitClass::DaQiaoXiaoQiao,
        UnitClass::JiaXu,
    ];

    pub fn id_str(&self) -> &'static str {
        match self {
            UnitClass::Knight => "zhao_yun",
            UnitClass::Archer => "huang_zhong",
            UnitClass::Mage => "zhuge_liang",
            UnitClass::Assassin => "zhang_he_yan_liang",
            UnitClass::Cleric => "hua_tuo",
            UnitClass::CaoCao => "cao_cao",
            UnitClass::DianWei => "dian_wei",
            UnitClass::GuoJia => "guo_jia",
            UnitClass::SunCe => "sun_ce",
            UnitClass::LuXun => "lu_xun",
            UnitClass::DaQiaoXiaoQiao => "da_qiao_xiao_qiao",
            UnitClass::JiaXu => "jia_xu",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            UnitClass::Knight => "Triệu Vân",
            UnitClass::Archer => "Hoàng Trung",
            UnitClass::Mage => "Gia Cát Lượng",
            UnitClass::Assassin => "Trương Cáp",
            UnitClass::Cleric => "Hoa Đà",
            UnitClass::CaoCao => "Tào Tháo",
            UnitClass::DianWei => "Điển Vi",
            UnitClass::GuoJia => "Quách Gia",
            UnitClass::SunCe => "Tôn Sách",
            UnitClass::LuXun => "Lục Tốn",
            UnitClass::DaQiaoXiaoQiao => "Đại Kiều",
            UnitClass::JiaXu => "Giả Hủ",
        }
    }

    pub fn role_title(&self) -> &'static str {
        match self {
            UnitClass::Knight => "Tiền Tuyến Thiết Vệ (Tank)",
            UnitClass::Archer => "Thần Xạ Cự Ly (DPS)",
            UnitClass::Mage => "Kỳ Môn Pháp Sư (AoE)",
            UnitClass::Assassin => "Ám Ảnh Liệp Thủ (Sát Thủ)",
            UnitClass::Cleric => "Thần Y Trị Liệu (Hỗ Trợ)",
            UnitClass::CaoCao => "Bá Vương Hiệu Lệnh (Thống Soái)",
            UnitClass::DianWei => "Cuồng Nộ Hổ Si (Đấu Sĩ)",
            UnitClass::GuoJia => "Quỷ Mưu Thần Toán (Khống Chế)",
            UnitClass::SunCe => "Tiểu Bá Vương (Tiên Phong)",
            UnitClass::LuXun => "Hỏa Thiêu Liên Doanh (Hỏa Công)",
            UnitClass::DaQiaoXiaoQiao => "Giang Đông Nhị Kiều (Trợ Thủ)",
            UnitClass::JiaXu => "Độc Sĩ Loạn Vũ (Hiểm Họa)",
        }
    }

    #[allow(dead_code)]
    pub fn icon(&self) -> &'static str {
        match self {
            UnitClass::Knight => "🐉",
            UnitClass::Archer => "🏹",
            UnitClass::Mage => "🪶",
            UnitClass::Assassin => "🗡️",
            UnitClass::Cleric => "⚕️",
            UnitClass::CaoCao => "👑",
            UnitClass::DianWei => "🛡️",
            UnitClass::GuoJia => "❄️",
            UnitClass::SunCe => "⚔️",
            UnitClass::LuXun => "🔥",
            UnitClass::DaQiaoXiaoQiao => "🌸",
            UnitClass::JiaXu => "☠️",
        }
    }

    pub fn role_abbr(&self) -> &'static str {
        match self {
            UnitClass::Knight => "TANK",
            UnitClass::Archer => "XẠ",
            UnitClass::Mage => "PHÁP",
            UnitClass::Assassin => "SÁT",
            UnitClass::Cleric => "Y",
            UnitClass::CaoCao => "CHỦ",
            UnitClass::DianWei => "THỦ",
            UnitClass::GuoJia => "BĂNG",
            UnitClass::SunCe => "ĐẤU",
            UnitClass::LuXun => "HỎA",
            UnitClass::DaQiaoXiaoQiao => "TRỢ",
            UnitClass::JiaXu => "ĐỘC",
        }
    }

    pub fn is_melee(&self) -> bool {
        matches!(
            self,
            UnitClass::Knight | UnitClass::Assassin | UnitClass::DianWei | UnitClass::SunCe
        )
    }

    pub fn is_magic(&self) -> bool {
        matches!(
            self,
            UnitClass::Mage
                | UnitClass::GuoJia
                | UnitClass::LuXun
                | UnitClass::JiaXu
                | UnitClass::CaoCao
        )
    }

    pub fn is_healer(&self) -> bool {
        matches!(self, UnitClass::Cleric | UnitClass::DaQiaoXiaoQiao)
    }

    pub fn skill_name(&self) -> &'static str {
        match self {
            UnitClass::Knight => "Bạch Mã Đột Kích",
            UnitClass::Archer => "Bách Bộ Xuyên Dương",
            UnitClass::Mage => "Cuồng Lôi Thiên Tru",
            UnitClass::Assassin => "Hắc Ám Đoạt Hồn",
            UnitClass::Cleric => "Thần Ân Trị Liệu",
            UnitClass::CaoCao => "Bá Đạo Thiên Hạ",
            UnitClass::DianWei => "Cổ Chi Ác Lai",
            UnitClass::GuoJia => "Thập Thắng Thập Bại",
            UnitClass::SunCe => "Bá Vương Khai Sơn",
            UnitClass::LuXun => "Xích Bích Liệt Hỏa",
            UnitClass::DaQiaoXiaoQiao => "Quốc Sắc Thiên Hương",
            UnitClass::JiaXu => "Vạn Độc Đoạt Mệnh",
        }
    }

    pub fn skill_type(&self) -> &'static str {
        match self {
            UnitClass::Knight => "Cận Chiến & Tăng Giáp",
            UnitClass::Archer => "Tập Kích Cự Ly & Kết Liễu",
            UnitClass::Mage => "Sấm Sét Diện Rộng",
            UnitClass::Assassin => "Ám Sát Hàng Sau & Bạo Kích",
            UnitClass::Cleric => "Trị Liệu Thần Thánh",
            UnitClass::CaoCao => "Cường Hóa Đồng Minh & Áp Đảo",
            UnitClass::DianWei => "Cuồng Kích & Phản Kích",
            UnitClass::GuoJia => "Đóng Băng & Giảm Nộ Địch",
            UnitClass::SunCe => "Cận Chiến Phá Giáp",
            UnitClass::LuXun => "Sát Thương Thiêu Đốt",
            UnitClass::DaQiaoXiaoQiao => "Hồi Phục & Tăng Tốc",
            UnitClass::JiaXu => "Trúng Độc Sát Thương Chuẩn",
        }
    }

    pub fn skill_description(&self) -> &'static str {
        match self {
            UnitClass::Knight => {
                "Lao nhanh chém kích đối phương bằng long thương, tăng cường phòng thủ vững chắc trước mọi đợt tấn công."
            }
            UnitClass::Archer => {
                "Giương cung bắn tỉa kẻ địch thấp máu nhất toàn đấu trường với tốc độ cao và tỉ lệ chí mạng vượt trội."
            }
            UnitClass::Mage => {
                "Triệu hồi cầu sấm sét tấn công mục tiêu chính và lan tỏa sóng xung kích sát thương phép lên toàn bộ hàng đối phương."
            }
            UnitClass::Assassin => {
                "Ẩn mình trong bóng đêm áp sát tiêu diệt tướng yếu máu nhất hàng sau của đối thủ bằng song đao đoạt mạng."
            }
            UnitClass::Cleric => {
                "Niệm chú ban ánh sáng thánh tích phục hồi lượng lớn sinh lực cho đồng minh bị thương nặng nhất trên bàn cờ."
            }
            UnitClass::CaoCao => {
                "Phát lệnh áp đảo toàn diện, tăng mạnh công kích và phòng thủ cho các tướng đồng minh."
            }
            UnitClass::DianWei => {
                "Vung kích sắt thị uy cuồng nộ, tăng giáp bản thân và gây sát thương cận chiến cực mạnh."
            }
            UnitClass::GuoJia => {
                "Kỳ mưu phong ấn băng sương, gây sát thương phép và làm chậm hành động quân địch."
            }
            UnitClass::SunCe => {
                "Xung phong xé toạc trận hình đối phương, gây sát thương chém lan và đẩy lùi hàng thủ địch."
            }
            UnitClass::LuXun => {
                "Hỏa thiêu liên doanh bùng cháy dữ dội, gây sát thương phép diện rộng thiêu đốt địch."
            }
            UnitClass::DaQiaoXiaoQiao => {
                "Cầm hương quyến rũ hồi máu cho đồng đội yếu nhất và tăng tốc độ hồi phục toàn đội."
            }
            UnitClass::JiaXu => {
                "Rải độc hiểm hóc lên hàng ngũ đối phương, trừ nộ khí và gây sát thương độc liên tục."
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
            UnitClass::CaoCao => "Warlord's Mandate",
            UnitClass::DianWei => "Berserker Rampage",
            UnitClass::GuoJia => "Frost Strategem",
            UnitClass::SunCe => "Conqueror's Wrath",
            UnitClass::LuXun => "Infernal Conflagration",
            UnitClass::DaQiaoXiaoQiao => "Twin Lotus Radiance",
            UnitClass::JiaXu => "Venomous Machination",
        }
    }

    pub fn ultimate_desc(&self) -> &'static str {
        match self {
            UnitClass::Knight => {
                "Tạo lá chắn long hồn cực đại, gây 220% sát thương công kích, nhận 80 Khiên và đẩy lùi thanh hành động kẻ địch."
            }
            UnitClass::Archer => {
                "Bật nhảy lùi về sau và phóng mưa 5 mũi thần tiễn xuyên phá tất cả quân địch còn sống với +50% Tỉ Lệ Bạo Kích."
            }
            UnitClass::Mage => {
                "Gọi thiên lôi diệt thế giáng xuống toàn bộ trận địa địch, gây 160% sát thương phép diện rộng lên mọi đối thủ."
            }
            UnitClass::Assassin => {
                "Dịch chuyển ra sau mục tiêu yếu nhất và tung liên hoàn 3 kích đoạt mệnh với 280% sát thương, hồi 50 Nộ khi hạ gục."
            }
            UnitClass::Cleric => {
                "Thần ân giáng thế ban phước hồi phục cho toàn bộ đồng đội bằng 160% Công + 45 Máu và tăng 25% thanh hành động."
            }
            UnitClass::CaoCao => {
                "Bá chủ thiên hạ xuất chiến, gây 180% sát thương hàng ngang và tăng 30% công cho toàn phe ta."
            }
            UnitClass::DianWei => {
                "Cuồng kích giáng xuống với 240% sát thương, tăng 60 giáp và bất tử trong 1 lượt."
            }
            UnitClass::GuoJia => {
                "Băng phong vạn lý gây 170% sát thương phép và đóng băng thanh hành động đối phương."
            }
            UnitClass::SunCe => {
                "Bá vương đột kích càn quét 220% sát thương hàng trước và làm choáng mục tiêu chính."
            }
            UnitClass::LuXun => {
                "Hỏa thiêu liên doanh giáng biển lửa gây 200% sát thương phép và đốt máu mục tiêu."
            }
            UnitClass::DaQiaoXiaoQiao => {
                "Song kiều tấu khúc trị liệu toàn đội 180% Công và tăng tốc hành động 30%."
            }
            UnitClass::JiaXu => {
                "Độc sát quần công gây 190% sát thương xuyên giáp và làm giảm 40 nộ kẻ địch."
            }
        }
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            UnitClass::Knight => {
                "Tiền tuyến với lượng Máu và Giáp cao. Hấp thụ sát thương cho đội hình."
            }
            UnitClass::Archer => {
                "Bắn tỉa mục tiêu thấp máu nhất trên bàn đấu với độ chính xác cao."
            }
            UnitClass::Mage => {
                "Phóng điện sấm sét giật toàn hàng kẻ địch với sát thương phép diện rộng."
            }
            UnitClass::Assassin => "Ẩn thân ám sát các vị trí yếu máu hàng sau của đối phương.",
            UnitClass::Cleric => "Hồi phục sinh lực và nâng đỡ đồng minh bị trọng thương.",
            UnitClass::CaoCao => {
                "Thống soái kiệt xuất, vừa có thể chống chịu vừa tăng sức mạnh đồng đội."
            }
            UnitClass::DianWei => "Đấu sĩ cự phách với sinh lực dồi dào và lực chém cực mạnh.",
            UnitClass::GuoJia => "Quân sư khống chế băng giá làm suy yếu nhịp tấn công kẻ thù.",
            UnitClass::SunCe => "Chiến tướng dũng mãnh càn quét phòng tuyến với đòn công dồn dập.",
            UnitClass::LuXun => "Pháp sư hỏa công thiêu rụi đội hình địch bằng hỏa thuật tàn khốc.",
            UnitClass::DaQiaoXiaoQiao => {
                "Song nữ tài hoa hồi phục sinh lực và tiếp sức tốc độ chiến đấu."
            }
            UnitClass::JiaXu => "Mưu sĩ độc địa bào mòn sinh lực và phá hủy nộ khí đối phương.",
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
            UnitClass::CaoCao => UnitStats {
                max_hp: 150.0,
                hp: 150.0,
                mana: 0.0,
                max_mana: 100.0,
                atk: 36.0,
                def: 30.0,
                speed: 24.0,
                crit_rate: 0.15,
                shield: 0.0,
            },
            UnitClass::DianWei => UnitStats {
                max_hp: 190.0,
                hp: 190.0,
                mana: 0.0,
                max_mana: 100.0,
                atk: 32.0,
                def: 38.0,
                speed: 16.0,
                crit_rate: 0.12,
                shield: 0.0,
            },
            UnitClass::GuoJia => UnitStats {
                max_hp: 90.0,
                hp: 90.0,
                mana: 0.0,
                max_mana: 100.0,
                atk: 44.0,
                def: 12.0,
                speed: 25.0,
                crit_rate: 0.25,
                shield: 0.0,
            },
            UnitClass::SunCe => UnitStats {
                max_hp: 165.0,
                hp: 165.0,
                mana: 0.0,
                max_mana: 100.0,
                atk: 35.0,
                def: 32.0,
                speed: 22.0,
                crit_rate: 0.20,
                shield: 0.0,
            },
            UnitClass::LuXun => UnitStats {
                max_hp: 95.0,
                hp: 95.0,
                mana: 0.0,
                max_mana: 100.0,
                atk: 48.0,
                def: 10.0,
                speed: 23.0,
                crit_rate: 0.22,
                shield: 0.0,
            },
            UnitClass::DaQiaoXiaoQiao => UnitStats {
                max_hp: 120.0,
                hp: 120.0,
                mana: 0.0,
                max_mana: 100.0,
                atk: 22.0,
                def: 20.0,
                speed: 24.0,
                crit_rate: 0.08,
                shield: 0.0,
            },
            UnitClass::JiaXu => UnitStats {
                max_hp: 105.0,
                hp: 105.0,
                mana: 0.0,
                max_mana: 100.0,
                atk: 45.0,
                def: 14.0,
                speed: 21.0,
                crit_rate: 0.18,
                shield: 0.0,
            },
        }
    }

    pub fn color(&self) -> Color {
        match self {
            UnitClass::Knight => Color::srgb(0.2, 0.45, 0.85),
            UnitClass::Archer => Color::srgb(0.2, 0.75, 0.3),
            UnitClass::Mage => Color::srgb(0.65, 0.25, 0.85),
            UnitClass::Assassin => Color::srgb(0.85, 0.2, 0.3),
            UnitClass::Cleric => Color::srgb(0.9, 0.8, 0.2),
            UnitClass::CaoCao => Color::srgb(0.75, 0.2, 0.8),
            UnitClass::DianWei => Color::srgb(0.85, 0.45, 0.15),
            UnitClass::GuoJia => Color::srgb(0.3, 0.7, 0.9),
            UnitClass::SunCe => Color::srgb(0.9, 0.5, 0.1),
            UnitClass::LuXun => Color::srgb(0.95, 0.35, 0.1),
            UnitClass::DaQiaoXiaoQiao => Color::srgb(0.95, 0.55, 0.75),
            UnitClass::JiaXu => Color::srgb(0.4, 0.8, 0.35),
        }
    }
}

#[allow(dead_code)]
#[derive(Resource)]
pub struct GameTextures {
    pub background: Handle<Image>,
    pub knight: Handle<Image>,
    pub archer: Handle<Image>,
    pub mage: Handle<Image>,
    pub assassin: Handle<Image>,
    pub cleric: Handle<Image>,
    pub cao_cao: Handle<Image>,
    pub dian_wei: Handle<Image>,
    pub guo_jia: Handle<Image>,
    pub sun_ce: Handle<Image>,
    pub lu_xun: Handle<Image>,
    pub da_qiao_xiao_qiao: Handle<Image>,
    pub jia_xu: Handle<Image>,
}

impl FromWorld for GameTextures {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            background: asset_server.load("textures/background.png"),
            knight: asset_server.load("textures/zhao_yun.png"),
            archer: asset_server.load("textures/huang_zhong.png"),
            mage: asset_server.load("textures/zhuge_liang.png"),
            assassin: asset_server.load("textures/zhang_he.png"),
            cleric: asset_server.load("textures/hua_tuo.png"),
            cao_cao: asset_server.load("textures/cao_cao.png"),
            dian_wei: asset_server.load("textures/dian_wei.png"),
            guo_jia: asset_server.load("textures/guo_jia.png"),
            sun_ce: asset_server.load("textures/sun_ce.png"),
            lu_xun: asset_server.load("textures/lu_xun.png"),
            da_qiao_xiao_qiao: asset_server.load("textures/da_qiao_xiao_qiao.png"),
            jia_xu: asset_server.load("textures/jia_xu.png"),
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
            cao_cao: Handle::default(),
            dian_wei: Handle::default(),
            guo_jia: Handle::default(),
            sun_ce: Handle::default(),
            lu_xun: Handle::default(),
            da_qiao_xiao_qiao: Handle::default(),
            jia_xu: Handle::default(),
        }
    }

    #[allow(dead_code)]
    pub fn get_unit_texture(&self, class: UnitClass) -> Handle<Image> {
        match class {
            UnitClass::Knight => self.knight.clone(),
            UnitClass::Archer => self.archer.clone(),
            UnitClass::Mage => self.mage.clone(),
            UnitClass::Assassin => self.assassin.clone(),
            UnitClass::Cleric => self.cleric.clone(),
            UnitClass::CaoCao => self.cao_cao.clone(),
            UnitClass::DianWei => self.dian_wei.clone(),
            UnitClass::GuoJia => self.guo_jia.clone(),
            UnitClass::SunCe => self.sun_ce.clone(),
            UnitClass::LuXun => self.lu_xun.clone(),
            UnitClass::DaQiaoXiaoQiao => self.da_qiao_xiao_qiao.clone(),
            UnitClass::JiaXu => self.jia_xu.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hero_classes_stats_and_mana() {
        for class in UnitClass::ALL {
            let stats = class.base_stats();
            assert!(stats.max_hp > 0.0);
            assert_eq!(stats.hp, stats.max_hp);
            assert_eq!(stats.mana, 0.0);
            assert_eq!(stats.max_mana, 100.0);
            assert_eq!(stats.shield, 0.0);
            assert!(stats.atk > 0.0);
            assert!(stats.def > 0.0);
            assert!(stats.speed > 0.0);
            assert!(stats.crit_rate >= 0.0);
        }
    }

    #[test]
    fn test_ultimate_skills_distinct() {
        for class in UnitClass::ALL {
            assert!(!class.ultimate_name().is_empty());
            assert!(!class.ultimate_desc().is_empty());
            assert!(!class.id_str().is_empty());
            assert!(!class.name().is_empty());
        }
    }
}
