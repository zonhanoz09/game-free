//! WebSocket progression and gacha message handlers.

use crate::AppState;
use axum::extract::ws::Message;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn handle_progression_message(
    msg_type: &str,
    parsed: &serde_json::Value,
    state: &Arc<AppState>,
    tx: &mpsc::UnboundedSender<Message>,
) -> bool {
    match msg_type {
        "GACHA_PULL" => {
            let count = parsed.get("count").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let username = parsed
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("guest");
            let mut prog_guard = state.player_progression.write().await;
            let player = prog_guard
                .entry(username.to_string())
                .or_insert_with(|| game_logic::gacha::PlayerProgressionState::new(5000));
            let mut rng = game_logic::gacha::GachaRng::new(rand::random());
            match player.pull(&mut rng, count, 100) {
                Ok(results) => {
                    let items: Vec<game_protocol::GachaPullItemData> = results
                        .into_iter()
                        .map(|r| game_protocol::GachaPullItemData {
                            hero_id: r.hero_id,
                            rarity: format!("{:?}", r.rarity),
                            is_duplicate: r.is_duplicate,
                            shards_granted: r.shards_granted,
                            pity_before: r.pity_before,
                            pity_after: r.pity_after,
                        })
                        .collect();
                    let msg = game_protocol::PvpMessage::GachaPullResult {
                        results: items,
                        pity_counter: player.pity_counter,
                        remaining_currency: player.currency,
                    };
                    let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                }
                Err(err) => {
                    let msg = game_protocol::PvpMessage::Error {
                        message: format!("Gacha pull failed: {:?}", err),
                    };
                    let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                }
            }
            true
        }
        "HERO_UPGRADE_STAR" => {
            let hero_id = parsed.get("hero_id").and_then(|v| v.as_str()).unwrap_or("");
            let username = parsed
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("guest");
            let mut prog_guard = state.player_progression.write().await;
            let player = prog_guard
                .entry(username.to_string())
                .or_insert_with(|| game_logic::gacha::PlayerProgressionState::new(5000));
            match player.upgrade_star(hero_id) {
                Ok(new_star) => {
                    let shards = player.heroes.get(hero_id).map(|h| h.shards).unwrap_or(0);
                    let msg = game_protocol::PvpMessage::HeroUpgradeStarResult {
                        hero_id: hero_id.to_string(),
                        new_star,
                        remaining_shards: shards,
                    };
                    let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                }
                Err(err) => {
                    let msg = game_protocol::PvpMessage::Error {
                        message: format!("Star upgrade failed: {:?}", err),
                    };
                    let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                }
            }
            true
        }
        "HERO_UPGRADE_LEVEL" => {
            let hero_id = parsed.get("hero_id").and_then(|v| v.as_str()).unwrap_or("");
            let levels = parsed.get("levels").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let username = parsed
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("guest");
            let mut prog_guard = state.player_progression.write().await;
            let player = prog_guard
                .entry(username.to_string())
                .or_insert_with(|| game_logic::gacha::PlayerProgressionState::new(5000));
            match player.upgrade_level(hero_id, levels, 50) {
                Ok(new_level) => {
                    let msg = game_protocol::PvpMessage::HeroUpgradeLevelResult {
                        hero_id: hero_id.to_string(),
                        new_level,
                        remaining_currency: player.currency,
                    };
                    let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                }
                Err(err) => {
                    let msg = game_protocol::PvpMessage::Error {
                        message: format!("Level upgrade failed: {:?}", err),
                    };
                    let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                }
            }
            true
        }
        "PROGRESSION_SYNC" => {
            let username = parsed
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("guest");
            let mut prog_guard = state.player_progression.write().await;
            let player = prog_guard
                .entry(username.to_string())
                .or_insert_with(|| game_logic::gacha::PlayerProgressionState::new(5000));
            let heroes: Vec<game_protocol::HeroProgressData> = player
                .heroes
                .values()
                .map(|h| game_protocol::HeroProgressData {
                    hero_id: h.hero_id.clone(),
                    star_level: h.star_level,
                    shards: h.shards,
                    level: h.level,
                })
                .collect();
            let msg = game_protocol::PvpMessage::ProgressionSync {
                currency: player.currency,
                pity_counter: player.pity_counter,
                heroes,
            };
            let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
            true
        }
        _ => false,
    }
}
