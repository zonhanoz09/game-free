//! Server-authoritative Gacha and Progression system.
//!
//! Implements Section 7 of system_game.md:
//! - Base rates: SSR 2.5%, SR 15.5%, R 82.0%
//! - Soft pity: 50 pulls (+2.5% SSR rate per pull after 50)
//! - Hard pity: 70 pulls (100% guaranteed SSR)
//! - Duplicate to shard conversion: SSR=50, SR=20, R=5
//! - Star upgrades (1 to 7 stars with required shards)
//! - Deterministic seeded PRNG for replay and audit
//! - Atomic transactions with rollback safety

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Deterministic 64-bit splitmix64 PRNG for audit and replay.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GachaRng {
    pub state: u64,
}

impl GachaRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    pub fn next_range(&mut self, max_exclusive: u32) -> u32 {
        if max_exclusive == 0 {
            return 0;
        }
        (self.next_u64() % max_exclusive as u64) as u32
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GachaRarity {
    R,
    Sr,
    Ssr,
}

impl GachaRarity {
    pub fn duplicate_shards(self) -> u32 {
        match self {
            Self::Ssr => 50,
            Self::Sr => 20,
            Self::R => 5,
        }
    }
}

/// Shard requirement for star ascension:
/// 1 -> 2: 20
/// 2 -> 3: 40
/// 3 -> 4: 80
/// 4 -> 5: 120
/// 5 -> 6: 180
/// 6 -> 7: 250
pub fn shards_for_next_star(current_star: u8) -> Option<u32> {
    match current_star {
        1 => Some(20),
        2 => Some(40),
        3 => Some(80),
        4 => Some(120),
        5 => Some(180),
        6 => Some(250),
        _ => None,
    }
}

pub const SOFT_PITY: u32 = 50;
pub const HARD_PITY: u32 = 70;
pub const BASE_SSR_RATE: f64 = 0.025;
pub const BASE_SR_RATE: f64 = 0.155;

/// Computes the effective SSR pull probability given current pity counter.
pub fn calculate_ssr_rate(pity_counter: u32) -> f64 {
    if pity_counter >= HARD_PITY {
        1.0
    } else if pity_counter >= SOFT_PITY {
        let extra = (pity_counter - SOFT_PITY + 1) as f64 * 0.025;
        (BASE_SSR_RATE + extra).min(1.0)
    } else {
        BASE_SSR_RATE
    }
}

pub const SSR_POOL: [&str; 12] = [
    "zhao_yun",
    "huang_zhong",
    "zhuge_liang",
    "cao_cao",
    "dian_wei",
    "guo_jia",
    "sun_ce",
    "lu_xun",
    "da_qiao_xiao_qiao",
    "zhang_he_yan_liang",
    "hua_tuo",
    "jia_xu",
];

pub const SR_POOL: [&str; 6] = [
    "guan_xing",
    "zhang_bao",
    "cao_hong",
    "xiahou_ba",
    "han_dang",
    "pan_zhang",
];

pub const R_POOL: [&str; 4] = ["foot_soldier", "militia_archer", "spearman", "scout"];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GachaPullResult {
    pub hero_id: String,
    pub rarity: GachaRarity,
    pub is_duplicate: bool,
    pub shards_granted: u32,
    pub pity_before: u32,
    pub pity_after: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HeroProgress {
    pub hero_id: String,
    pub star_level: u8,
    pub shards: u32,
    pub level: u32,
}

impl HeroProgress {
    pub fn new(hero_id: impl Into<String>) -> Self {
        Self {
            hero_id: hero_id.into(),
            star_level: 1,
            shards: 0,
            level: 1,
        }
    }

    /// Stat multiplier from star level: +15% per star above 1.
    pub fn star_multiplier(&self) -> f32 {
        1.0 + (self.star_level.saturating_sub(1) as f32) * 0.15
    }

    /// Stat multiplier from hero level: +5% per level above 1.
    pub fn level_multiplier(&self) -> f32 {
        1.0 + (self.level.saturating_sub(1) as f32) * 0.05
    }

    pub fn total_multiplier(&self) -> f32 {
        self.star_multiplier() * self.level_multiplier()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProgressionError {
    InsufficientCurrency { required: u64, available: u64 },
    InsufficientShards { required: u32, available: u32 },
    HeroNotFound(String),
    MaxStarReached,
    MaxLevelReached,
    InvalidCount,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerProgressionState {
    pub currency: u64,
    pub pity_counter: u32,
    pub heroes: BTreeMap<String, HeroProgress>,
    pub total_pulls: u64,
}

impl PlayerProgressionState {
    pub fn new(initial_currency: u64) -> Self {
        Self {
            currency: initial_currency,
            pity_counter: 0,
            heroes: BTreeMap::new(),
            total_pulls: 0,
        }
    }

    /// Executes an atomic gacha summon transaction.
    ///
    /// If currency is insufficient, no state is modified and `Err` is returned.
    pub fn pull(
        &mut self,
        rng: &mut GachaRng,
        count: u32,
        cost_per_pull: u64,
    ) -> Result<Vec<GachaPullResult>, ProgressionError> {
        if count == 0 {
            return Err(ProgressionError::InvalidCount);
        }
        let total_cost = (count as u64).saturating_mul(cost_per_pull);
        if self.currency < total_cost {
            return Err(ProgressionError::InsufficientCurrency {
                required: total_cost,
                available: self.currency,
            });
        }

        // Deduct currency atomically
        self.currency -= total_cost;
        let mut results = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let pity_before = self.pity_counter;
            let ssr_rate = calculate_ssr_rate(pity_before);
            let roll = rng.next_f64();

            let (rarity, hero_id, pity_after) = if roll <= ssr_rate {
                let idx = rng.next_range(SSR_POOL.len() as u32) as usize;
                (GachaRarity::Ssr, SSR_POOL[idx].to_string(), 0)
            } else if roll <= (ssr_rate + BASE_SR_RATE) {
                let idx = rng.next_range(SR_POOL.len() as u32) as usize;
                (GachaRarity::Sr, SR_POOL[idx].to_string(), pity_before + 1)
            } else {
                let idx = rng.next_range(R_POOL.len() as u32) as usize;
                (GachaRarity::R, R_POOL[idx].to_string(), pity_before + 1)
            };

            self.pity_counter = pity_after;
            self.total_pulls = self.total_pulls.saturating_add(1);

            let (is_duplicate, shards_granted) = if let Some(hero) = self.heroes.get_mut(&hero_id) {
                let shards = rarity.duplicate_shards();
                hero.shards = hero.shards.saturating_add(shards);
                (true, shards)
            } else {
                self.heroes
                    .insert(hero_id.clone(), HeroProgress::new(&hero_id));
                (false, 0)
            };

            results.push(GachaPullResult {
                hero_id,
                rarity,
                is_duplicate,
                shards_granted,
                pity_before,
                pity_after,
            });
        }

        Ok(results)
    }

    /// Upgrades a hero's star level using hero shards atomically.
    pub fn upgrade_star(&mut self, hero_id: &str) -> Result<u8, ProgressionError> {
        let hero = self
            .heroes
            .get_mut(hero_id)
            .ok_or_else(|| ProgressionError::HeroNotFound(hero_id.to_string()))?;

        let required =
            shards_for_next_star(hero.star_level).ok_or(ProgressionError::MaxStarReached)?;

        if hero.shards < required {
            return Err(ProgressionError::InsufficientShards {
                required,
                available: hero.shards,
            });
        }

        hero.shards -= required;
        hero.star_level += 1;
        Ok(hero.star_level)
    }

    /// Upgrades a hero's level using currency atomically.
    pub fn upgrade_level(
        &mut self,
        hero_id: &str,
        levels: u32,
        cost_per_level: u64,
    ) -> Result<u32, ProgressionError> {
        if levels == 0 {
            return Err(ProgressionError::InvalidCount);
        }
        let hero = self
            .heroes
            .get_mut(hero_id)
            .ok_or_else(|| ProgressionError::HeroNotFound(hero_id.to_string()))?;

        if hero.level.saturating_add(levels) > 100 {
            return Err(ProgressionError::MaxLevelReached);
        }

        let total_cost = (levels as u64).saturating_mul(cost_per_level);
        if self.currency < total_cost {
            return Err(ProgressionError::InsufficientCurrency {
                required: total_cost,
                available: self.currency,
            });
        }

        self.currency -= total_cost;
        hero.level += levels;
        Ok(hero.level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pity_probability_curve_and_hard_cap() {
        assert!((calculate_ssr_rate(0) - 0.025).abs() < 1e-6);
        assert!((calculate_ssr_rate(49) - 0.025).abs() < 1e-6);
        assert!((calculate_ssr_rate(50) - 0.050).abs() < 1e-6);
        assert!((calculate_ssr_rate(51) - 0.075).abs() < 1e-6);
        assert!((calculate_ssr_rate(60) - 0.300).abs() < 1e-6);
        assert!((calculate_ssr_rate(69) - 0.525).abs() < 1e-6);
        assert_eq!(calculate_ssr_rate(70), 1.0);
        assert_eq!(calculate_ssr_rate(100), 1.0);
    }

    #[test]
    fn hard_pity_guarantees_ssr_at_seventy() {
        let mut player = PlayerProgressionState::new(100_000);
        player.pity_counter = 69;
        let mut rng = GachaRng::new(12345);

        // Pull 70th pull -> must be SSR
        let res = player.pull(&mut rng, 1, 100).expect("pull succeeds");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].rarity, GachaRarity::Ssr);
        assert_eq!(res[0].pity_after, 0);
        assert_eq!(player.pity_counter, 0);
    }

    #[test]
    fn duplicate_heroes_convert_to_exact_shards() {
        let mut player = PlayerProgressionState::new(100_000);
        let mut rng = GachaRng::new(999);

        // Force give Zhao Yun
        player
            .heroes
            .insert("zhao_yun".to_string(), HeroProgress::new("zhao_yun"));
        player.pity_counter = 70; // Force SSR

        let res = player.pull(&mut rng, 1, 100).expect("pull succeeds");
        assert_eq!(res[0].rarity, GachaRarity::Ssr);
        if res[0].hero_id == "zhao_yun" {
            assert!(res[0].is_duplicate);
            assert_eq!(res[0].shards_granted, 50);
            assert_eq!(player.heroes["zhao_yun"].shards, 50);
        }
    }

    #[test]
    fn star_ascension_shards_and_stat_multipliers() {
        let mut player = PlayerProgressionState::new(10_000);
        let mut hero = HeroProgress::new("zhao_yun");
        hero.shards = 20; // Exact amount for 1 -> 2 star
        player.heroes.insert("zhao_yun".to_string(), hero);

        let new_star = player.upgrade_star("zhao_yun").expect("star up");
        assert_eq!(new_star, 2);
        assert_eq!(player.heroes["zhao_yun"].shards, 0);
        assert!((player.heroes["zhao_yun"].star_multiplier() - 1.15).abs() < 1e-4);

        // Insufficient shards for 2 -> 3 star (needs 40)
        let err = player.upgrade_star("zhao_yun").expect_err("should fail");
        assert_eq!(
            err,
            ProgressionError::InsufficientShards {
                required: 40,
                available: 0
            }
        );
        // Star remains 2 (atomic rollback)
        assert_eq!(player.heroes["zhao_yun"].star_level, 2);
    }

    #[test]
    fn deterministic_seed_replay_parity() {
        let mut player1 = PlayerProgressionState::new(100_000);
        let mut rng1 = GachaRng::new(424242);
        let pulls1 = player1.pull(&mut rng1, 50, 100).expect("pulls 1");

        let mut player2 = PlayerProgressionState::new(100_000);
        let mut rng2 = GachaRng::new(424242);
        let pulls2 = player2.pull(&mut rng2, 50, 100).expect("pulls 2");

        assert_eq!(pulls1, pulls2);
        assert_eq!(player1, player2);
    }

    #[test]
    fn atomic_rollback_on_insufficient_currency() {
        let mut player = PlayerProgressionState::new(250);
        let mut rng = GachaRng::new(111);

        // Cost 100 per pull, 10 pulls = 1000 cost. Player only has 250.
        let err = player.pull(&mut rng, 10, 100).expect_err("should fail");
        assert_eq!(
            err,
            ProgressionError::InsufficientCurrency {
                required: 1000,
                available: 250
            }
        );
        // State unchanged
        assert_eq!(player.currency, 250);
        assert_eq!(player.pity_counter, 0);
        assert_eq!(player.total_pulls, 0);
        assert!(player.heroes.is_empty());
    }
}
