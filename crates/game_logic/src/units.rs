//! Unit combat state, action gauge, and faction synergies.

use crate::board::{BoardSlot, TeamSide, UnitFaction};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ActionGauge {
    pub current: u32,
}

impl ActionGauge {
    pub const MAX: u32 = 10_000;

    pub fn advance(&mut self, speed: u32) {
        self.current = self.current.saturating_add(speed);
    }

    pub fn ready(self) -> bool {
        self.current >= Self::MAX
    }

    pub fn consume_turn(&mut self) {
        self.current = self.current.saturating_sub(Self::MAX);
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeroId {
    ZhaoYun,
    HuangZhong,
    ZhugeLiang,
    CaoCao,
    DianWei,
    GuoJia,
    SunCe,
    LuXun,
    DaQiaoXiaoQiao,
    ZhangHeYanLiang,
    HuaTuo,
    JiaXu,
}

impl HeroId {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "zhao_yun" => Some(Self::ZhaoYun),
            "huang_zhong" => Some(Self::HuangZhong),
            "zhuge_liang" => Some(Self::ZhugeLiang),
            "cao_cao" => Some(Self::CaoCao),
            "dian_wei" => Some(Self::DianWei),
            "guo_jia" => Some(Self::GuoJia),
            "sun_ce" => Some(Self::SunCe),
            "lu_xun" => Some(Self::LuXun),
            "da_qiao_xiao_qiao" => Some(Self::DaQiaoXiaoQiao),
            "zhang_he_yan_liang" => Some(Self::ZhangHeYanLiang),
            "hua_tuo" => Some(Self::HuaTuo),
            "jia_xu" => Some(Self::JiaXu),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ZhaoYun => "zhao_yun",
            Self::HuangZhong => "huang_zhong",
            Self::ZhugeLiang => "zhuge_liang",
            Self::CaoCao => "cao_cao",
            Self::DianWei => "dian_wei",
            Self::GuoJia => "guo_jia",
            Self::SunCe => "sun_ce",
            Self::LuXun => "lu_xun",
            Self::DaQiaoXiaoQiao => "da_qiao_xiao_qiao",
            Self::ZhangHeYanLiang => "zhang_he_yan_liang",
            Self::HuaTuo => "hua_tuo",
            Self::JiaXu => "jia_xu",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnitState {
    pub id: u32,
    pub side: TeamSide,
    pub slot: BoardSlot,
    pub max_hp: u32,
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub speed: u32,
    pub rage: u16,
    pub gauge: ActionGauge,
    pub stunned_turns: u8,
    #[serde(default)]
    pub hero_id: Option<HeroId>,
    #[serde(default)]
    pub faction: Option<UnitFaction>,
    #[serde(default)]
    pub burn_turns: u8,
    #[serde(default)]
    pub burn_damage_bps: u16,
    #[serde(default)]
    pub poison_turns: u8,
    #[serde(default)]
    pub poison_damage_bps: u16,
    #[serde(default)]
    pub taunt_turns: u8,
    #[serde(default)]
    pub frozen_turns: u8,
    #[serde(default)]
    pub rage_locked_turns: u8,
    #[serde(default)]
    pub anti_heal_turns: u8,
    #[serde(default)]
    pub anti_heal_bps: u16,
    #[serde(default)]
    pub dodge_turns: u8,
}

impl UnitState {
    pub fn new(
        id: u32,
        side: TeamSide,
        slot: BoardSlot,
        max_hp: u32,
        attack: u32,
        defense: u32,
        speed: u32,
    ) -> Self {
        Self {
            id,
            side,
            slot,
            max_hp,
            hp: max_hp,
            attack,
            defense,
            speed,
            rage: 0,
            gauge: ActionGauge::default(),
            stunned_turns: 0,
            faction: None,
            burn_turns: 0,
            burn_damage_bps: 0,
            poison_turns: 0,
            poison_damage_bps: 0,
            taunt_turns: 0,
            frozen_turns: 0,
            rage_locked_turns: 0,
            anti_heal_turns: 0,
            anti_heal_bps: 0,
            dodge_turns: 0,
            hero_id: None,
        }
    }

    pub fn with_hero_id(mut self, hero_id: HeroId) -> Self {
        self.hero_id = Some(hero_id);
        self
    }

    pub fn with_faction(mut self, faction: UnitFaction) -> Self {
        self.faction = Some(faction);
        self
    }

    pub fn is_alive(self) -> bool {
        self.hp > 0
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct FactionSynergy {
    pub wei_tier: u8, // 0, 3, 5
    pub shu_tier: u8, // 0, 3, 5
    pub wu_tier: u8,  // 0, 3, 5
    pub qun_tier: u8, // 0, 3, 5
}
