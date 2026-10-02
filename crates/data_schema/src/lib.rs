//! Serializable schemas for balance and runtime game data.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BalanceEntry {
    pub id: String,
    pub value: f32,
}
