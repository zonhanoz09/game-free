use crate::types::*;
use crate::units::Unit;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SynergyType {
    Vanguard,     // Tank / Warrior
    Sharpshooter, // DPS / Ranged
    Arcanist,     // Mage / Burst AoE
    Shadow,       // Assassin / Crit
    Divine,       // Healer / Support
}

impl SynergyType {
    pub fn name(&self) -> &'static str {
        match self {
            SynergyType::Vanguard => "Thiết Vệ",
            SynergyType::Sharpshooter => "Thần Xạ",
            SynergyType::Arcanist => "Kỳ Môn",
            SynergyType::Shadow => "Ám Ảnh",
            SynergyType::Divine => "Thần Ân",
        }
    }

    pub fn threshold(&self) -> usize {
        match self {
            SynergyType::Vanguard => 2,
            SynergyType::Sharpshooter => 2,
            SynergyType::Arcanist => 2,
            SynergyType::Shadow => 2,
            SynergyType::Divine => 1,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SynergyType::Vanguard => "🛡️",
            SynergyType::Sharpshooter => "🏹",
            SynergyType::Arcanist => "⚡",
            SynergyType::Shadow => "🗡️",
            SynergyType::Divine => "⚕️",
        }
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            SynergyType::Vanguard => "(2) +35 Giáp cho Tiền Tuyến, +15 Giáp cho toàn đội",
            SynergyType::Sharpshooter => "(2) +25% Công, +15% Tỉ Lệ Bạo Kích cho Thần Xạ",
            SynergyType::Arcanist => "(2) +30% Công Phép, bắt đầu trận với 30 Nộ Khí",
            SynergyType::Shadow => "(2) +25% Tỉ Lệ Bạo Kích, ưu tiên áp sát hàng sau",
            SynergyType::Divine => "(1) Hồi 20 HP mỗi lượt cho toàn đội hình",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            SynergyType::Vanguard => Color::srgb(0.3, 0.6, 1.0),
            SynergyType::Sharpshooter => Color::srgb(0.3, 0.85, 0.4),
            SynergyType::Arcanist => Color::srgb(0.75, 0.35, 1.0),
            SynergyType::Shadow => Color::srgb(0.95, 0.3, 0.35),
            SynergyType::Divine => Color::srgb(1.0, 0.85, 0.25),
        }
    }
}

pub fn class_to_synergy(class: UnitClass) -> SynergyType {
    match class {
        UnitClass::Knight | UnitClass::DianWei | UnitClass::SunCe | UnitClass::CaoCao => {
            SynergyType::Vanguard
        }
        UnitClass::Archer => SynergyType::Sharpshooter,
        UnitClass::Mage | UnitClass::GuoJia | UnitClass::LuXun | UnitClass::JiaXu => {
            SynergyType::Arcanist
        }
        UnitClass::Assassin => SynergyType::Shadow,
        UnitClass::Cleric | UnitClass::DaQiaoXiaoQiao => SynergyType::Divine,
    }
}

#[derive(Component)]
pub struct SynergyRow(pub SynergyType);

#[derive(Component)]
pub struct SynergyCountText(pub SynergyType);

#[derive(Component)]
pub struct SynergyContainer;

pub fn update_synergies_ui(
    units: Query<(&Unit, &GridPos), Without<DeadUnit>>,
    mut text_query: Query<(&SynergyCountText, &mut Text, &mut TextColor)>,
    mut row_query: Query<(&SynergyRow, &mut BorderColor, &mut BackgroundColor)>,
) {
    let mut counts = std::collections::HashMap::new();
    for (unit, _) in units.iter() {
        if unit.faction == Faction::Player {
            let syn = class_to_synergy(unit.class);
            *counts.entry(syn).or_insert(0) += 1;
        }
    }

    for (syn_text, mut txt, mut color) in text_query.iter_mut() {
        let count = counts.get(&syn_text.0).copied().unwrap_or(0);
        let thresh = syn_text.0.threshold();
        *txt = Text::new(format!("{}/{}", count, thresh));
        if count >= thresh {
            color.0 = Color::srgb(1.0, 0.9, 0.2);
        } else {
            color.0 = Color::srgba(0.8, 0.8, 0.8, 0.7);
        }
    }

    for (syn_row, mut border, mut bg) in row_query.iter_mut() {
        let count = counts.get(&syn_row.0).copied().unwrap_or(0);
        let thresh = syn_row.0.threshold();
        let syn_col = syn_row.0.color();
        if count >= thresh {
            border.0 = syn_col;
            bg.0 = Color::srgba(
                syn_col.to_srgba().red * 0.45,
                syn_col.to_srgba().green * 0.45,
                syn_col.to_srgba().blue * 0.45,
                0.90,
            );
        } else {
            border.0 = Color::srgba(1.0, 1.0, 1.0, 0.1);
            bg.0 = Color::srgba(0.12, 0.15, 0.22, 0.6);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_classes_mapped_to_synergies() {
        for c in UnitClass::ALL {
            let syn = class_to_synergy(c);
            assert!(!syn.name().is_empty());
            assert!(!syn.icon().is_empty());
            assert!(!syn.description().is_empty());
        }
    }
}
