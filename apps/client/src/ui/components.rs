use super::*;

#[derive(Component)]
pub struct StartBattleButton;

#[derive(Component)]
pub struct StartBattleText;

#[derive(Component)]
pub struct ClearBoardButton;

#[derive(Component)]
pub struct PresetButton;

#[derive(Component)]
pub struct SpeedToggleButton;

#[derive(Component)]
pub struct NextStageButton;

#[derive(Component)]
pub struct RetryButton;

#[derive(Component)]
pub struct ShopCard(pub usize);

#[derive(Component)]
pub struct ShopCardAvatar(pub usize);

#[derive(Component)]
pub struct ShopCardName(pub usize);

#[derive(Component)]
pub struct ShopCardCost(pub usize);

#[derive(Component)]
pub struct ShopLockText;

#[derive(Component)]
pub struct PlacementUiRoot;

#[derive(Component)]
pub struct ResultUiRoot;

#[derive(Component)]
pub struct StageTitleText;

#[derive(Component)]
pub struct StageDescText;

#[derive(Component)]
pub struct UnitCountText;

#[derive(Component)]
pub struct SpeedText;

#[derive(Component)]
pub struct TooltipText;

// --- Components for Hero Inspection Card ---
#[derive(Component)]
pub struct InspectHeroAvatar;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InspectStatType {
    Hp,
    Mana,
    Atk,
    Def,
    Spd,
}

#[derive(Component)]
pub struct InspectStatBar(pub InspectStatType);

#[derive(Component)]
pub struct InspectStatText(pub InspectStatType);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InspectHeaderField {
    Name,
    Faction,
    Role,
}

#[derive(Component)]
pub struct InspectHeader(pub InspectHeaderField);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InspectSkillField {
    Name,
    Type,
    Desc,
    UltName,
    UltDesc,
}

#[derive(Component)]
pub struct InspectSkill(pub InspectSkillField);

#[derive(Component)]
pub struct InspectorRoot;
