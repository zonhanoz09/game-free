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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthUpdate {
    pub player_hp: i32,
    pub opponent_hp: i32,
}
