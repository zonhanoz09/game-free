//! Network-facing schemas shared by game clients and servers.

use serde::{Deserialize, Serialize};

/// A unit placement sent between the game client and match server.
///
/// This intentionally contains only serializable game data. Rendering
/// components, Bevy entities, and client-only state stay in the applications.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PvpUnitData {
    pub col: usize,
    pub row: usize,
    pub class: String,
    pub star_level: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeckCardData {
    pub id: String,
    pub hero_class: String,
    pub star_level: u8,
    pub level: u32,
    pub hp_bonus: f32,
    pub atk_bonus: f32,
    #[serde(default)]
    pub initiative_bonus: f32,
    pub is_starter: bool,
    /// Formation slot 0..8. `None` means the card stays on the bench.
    #[serde(default)]
    pub position: Option<usize>,
}

/// Messages exchanged by the browser client and the multiplayer gateway.
///
/// Keeping the discriminator and aliases here prevents client/server drift
/// when a message is renamed or a new transport is added.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PvpMessage {
    #[serde(alias = "CreateRoom", alias = "CREATE_ROOM")]
    CreateRoom { room_code: String },
    #[serde(alias = "JoinRoom", alias = "JOIN_ROOM")]
    JoinRoom { room_code: String },
    #[serde(alias = "RoomJoined", alias = "ROOM_JOINED")]
    RoomJoined {
        room_code: String,
        role: String,
        player_name: String,
        opponent_name: String,
    },
    #[serde(alias = "PlayerReady", alias = "PLAYER_READY")]
    PlayerReady { lineup: Vec<PvpUnitData> },
    #[serde(alias = "OpponentReady", alias = "OPPONENT_READY")]
    OpponentReady,
    #[serde(alias = "StartRound", alias = "START_ROUND")]
    StartRound {
        round: usize,
        opponent_lineup: Vec<PvpUnitData>,
        player_hp: i32,
        opponent_hp: i32,
    },
    #[serde(alias = "BattleFinished", alias = "BATTLE_FINISHED")]
    BattleFinished {
        winner_role: String,
        player_survivors: usize,
        #[serde(default)]
        room_code: Option<String>,
        #[serde(default)]
        round: Option<usize>,
    },
    #[serde(alias = "UpdateMatchHp", alias = "UPDATE_MATCH_HP")]
    UpdateMatchHp {
        player_hp: i32,
        opponent_hp: i32,
        damage_dealt: i32,
    },
    #[serde(alias = "MatchEnd", alias = "MATCH_END")]
    MatchEnd { winner: String },
    #[serde(alias = "SetDeck", alias = "SET_DECK")]
    SetDeck { cards: Vec<DeckCardData> },
    #[serde(alias = "Error", alias = "ERROR")]
    Error { message: String },
    #[serde(alias = "SetSpeed", alias = "SET_SPEED")]
    SetSpeed { speed: f32 },
    #[serde(alias = "ExitMatch", alias = "EXIT_MATCH")]
    ExitMatch,
    #[serde(alias = "StartBattle", alias = "START_BATTLE")]
    StartBattle,
    #[serde(alias = "BattleCommand", alias = "BATTLE_COMMAND")]
    BattleCommand { command: CombatCommand },
    #[serde(alias = "BattleResult", alias = "BATTLE_RESULT")]
    BattleResult { result: BattleResult },
    #[serde(alias = "GachaPull", alias = "GACHA_PULL")]
    GachaPull { count: u32 },
    #[serde(alias = "GachaPullResult", alias = "GACHA_PULL_RESULT")]
    GachaPullResult {
        results: Vec<GachaPullItemData>,
        pity_counter: u32,
        remaining_currency: u64,
    },
    #[serde(alias = "HeroUpgradeStar", alias = "HERO_UPGRADE_STAR")]
    HeroUpgradeStar { hero_id: String },
    #[serde(alias = "HeroUpgradeStarResult", alias = "HERO_UPGRADE_STAR_RESULT")]
    HeroUpgradeStarResult {
        hero_id: String,
        new_star: u8,
        remaining_shards: u32,
    },
    #[serde(alias = "HeroUpgradeLevel", alias = "HERO_UPGRADE_LEVEL")]
    HeroUpgradeLevel { hero_id: String, levels: u32 },
    #[serde(alias = "HeroUpgradeLevelResult", alias = "HERO_UPGRADE_LEVEL_RESULT")]
    HeroUpgradeLevelResult {
        hero_id: String,
        new_level: u32,
        remaining_currency: u64,
    },
    #[serde(alias = "ProgressionSync", alias = "PROGRESSION_SYNC")]
    ProgressionSync {
        currency: u64,
        pity_counter: u32,
        heroes: Vec<HeroProgressData>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthUpdate {
    pub player_hp: i32,
    pub opponent_hp: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BattleSetup {
    pub battle_id: String,
    pub seed: u64,
    pub config_version: String,
    #[serde(default)]
    pub config_fingerprint: String,
    pub attacker: Vec<PvpUnitData>,
    pub defender: Vec<PvpUnitData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatCommand {
    #[serde(default)]
    pub command_id: String,
    pub battle_id: String,
    pub turn: u32,
    pub actor_id: u32,
    pub target_rule: String,
    pub damage_rate_bps: u16,
    pub rage_cost: u16,
    pub critical: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatReplay {
    pub battle_id: String,
    pub seed: u64,
    pub config_version: String,
    #[serde(default)]
    pub config_fingerprint: String,
    pub events: Vec<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BattleResult {
    pub battle_id: String,
    pub winner: Option<String>,
    pub turn: u32,
    pub replay: CombatReplay,
    pub accepted_command_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gacha_messages_round_trip_json() {
        let msg = PvpMessage::GachaPullResult {
            results: vec![GachaPullItemData {
                hero_id: "zhao_yun".to_string(),
                rarity: "SSR".to_string(),
                is_duplicate: false,
                shards_granted: 0,
                pity_before: 49,
                pity_after: 0,
            }],
            pity_counter: 0,
            remaining_currency: 9900,
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: PvpMessage = serde_json::from_str(&serialized).unwrap();
        match deserialized {
            PvpMessage::GachaPullResult {
                results,
                pity_counter,
                remaining_currency,
            } => {
                assert_eq!(results.len(), 1);
                assert_eq!(results[0].hero_id, "zhao_yun");
                assert_eq!(pity_counter, 0);
                assert_eq!(remaining_currency, 9900);
            }
            _ => panic!("unexpected message variant"),
        }
    }

    #[test]
    fn battle_command_round_trips_json() {
        let command = CombatCommand {
            command_id: "cmd-1".to_string(),
            battle_id: "room-1-1".to_string(),
            turn: 4,
            actor_id: 42,
            target_rule: "DIRECT_LINE".to_string(),
            damage_rate_bps: 1000,
            rage_cost: 0,
            critical: false,
        };
        let encoded = serde_json::to_string(&command).expect("encode command");
        let decoded: CombatCommand = serde_json::from_str(&encoded).expect("decode command");
        assert_eq!(decoded.battle_id, "room-1-1");
        assert_eq!(decoded.actor_id, 42);
        assert_eq!(decoded.turn, 4);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GachaPullItemData {
    pub hero_id: String,
    pub rarity: String,
    pub is_duplicate: bool,
    pub shards_granted: u32,
    pub pity_before: u32,
    pub pity_after: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeroProgressData {
    pub hero_id: String,
    pub star_level: u8,
    pub shards: u32,
    pub level: u32,
}
