//! Network-facing schemas shared by game clients and servers.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthUpdate {
    pub player_hp: i32,
    pub opponent_hp: i32,
}
