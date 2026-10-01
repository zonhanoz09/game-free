use crate::units::Unit;
use crate::battle::BattleRng;
use crate::types::*;
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StarLevel(pub u8);

impl Default for StarLevel {
    fn default() -> Self {
        StarLevel(1)
    }
}

impl StarLevel {
    pub fn badge(&self) -> &'static str {
        match self.0 {
            1 => "★",
            2 => "★★",
            _ => "★★★",
        }
    }

    pub fn color(&self) -> Color {
        match self.0 {
            1 => Color::srgb(0.8, 0.8, 0.85),
            2 => Color::srgb(1.0, 0.85, 0.2),
            _ => Color::srgb(0.9, 0.4, 1.0),
        }
    }

    #[allow(dead_code)]
    pub fn stat_multiplier(&self) -> (f32, f32) {
        // (HP multiplier, ATK multiplier)
        match self.0 {
            1 => (1.0, 1.0),
            2 => (1.8, 1.6),
            _ => (2.8, 2.5),
        }
    }
}

pub fn unit_cost(class: UnitClass) -> i32 {
    match class {
        UnitClass::Knight => 2,
        UnitClass::Archer => 2,
        UnitClass::Mage => 3,
        UnitClass::Assassin => 3,
        UnitClass::Cleric => 3,
    }
}

#[derive(Resource)]
pub struct PlayerEconomy {
    pub gold: i32,
    pub interest: i32,
    pub win_streak: i32,
    pub shop_locked: bool,
    pub shop_slots: [Option<UnitClass>; 4],
}

impl Default for PlayerEconomy {
    fn default() -> Self {
        Self {
            gold: 15,
            interest: 1,
            win_streak: 0,
            shop_locked: false,
            shop_slots: [
                Some(UnitClass::Knight),
                Some(UnitClass::Archer),
                Some(UnitClass::Mage),
                Some(UnitClass::Assassin),
            ],
        }
    }
}

impl PlayerEconomy {
    pub fn reroll(&mut self, rng: &mut BattleRng) -> bool {
        if self.gold < 2 {
            info!("[SHOP] Reroll failed: Not enough gold (Current: {}G, Needed: 2G)", self.gold);
            return false;
        }
        self.gold -= 2;
        self.generate_shop(rng);
        info!("[SHOP] Rerolled shop bench for 2G -> Remaining Gold: {}G", self.gold);
        true
    }

    pub fn generate_shop(&mut self, rng: &mut BattleRng) {
        let pool = [
            UnitClass::Knight,
            UnitClass::Archer,
            UnitClass::Mage,
            UnitClass::Assassin,
            UnitClass::Cleric,
        ];
        for slot in self.shop_slots.iter_mut() {
            let idx = (rng.next_f32() * pool.len() as f32) as usize % pool.len();
            *slot = Some(pool[idx]);
        }
    }

    #[allow(dead_code)]
    pub fn buy_slot(&mut self, slot_idx: usize) -> Option<UnitClass> {
        if slot_idx >= 4 {
            return None;
        }
        if let Some(class) = self.shop_slots[slot_idx] {
            let cost = unit_cost(class);
            if self.gold >= cost {
                self.gold -= cost;
                self.shop_slots[slot_idx] = None;
                return Some(class);
            }
        }
        None
    }

    pub fn apply_round_income(&mut self, victory: bool, rng: &mut BattleRng) {
        let base_income = 5;
        let streak_bonus = if victory {
            self.win_streak += 1;
            1
        } else {
            self.win_streak = 0;
            0
        };
        let interest = (self.gold / 10).clamp(0, 5);
        self.interest = interest;
        let total = base_income + streak_bonus + interest;
        self.gold += total;
        info!(
            "[ECONOMY] Round Income: +{}G (Base: 5G, Streak Bonus: {}G, Interest: {}G) -> Current Gold: {}G (Streak: {})",
            total, streak_bonus, interest, self.gold, self.win_streak
        );

        if !self.shop_locked {
            self.generate_shop(rng);
            info!("[SHOP] Shop bench refreshed with new heroes");
        } else {
            info!("[SHOP] Shop bench preserved (Shop is locked)");
        }
    }
}

#[derive(Component)]
#[allow(dead_code)]
pub struct ShopCardButton(pub usize);

#[derive(Component)]
pub struct ShopRerollButton;

#[derive(Component)]
pub struct ShopLockToggle;

#[derive(Component)]
pub struct GoldDisplayText;


pub fn auto_star_fusion_system(
    mut commands: Commands,
    textures: Res<GameTextures>,
    units: Query<(Entity, &Unit, &GridPos, &StarLevel), Without<DeadUnit>>,
    mut sound_events: EventWriter<crate::audio::PlaySoundEvent>,
) {
    let classes = [
        UnitClass::Knight,
        UnitClass::Archer,
        UnitClass::Mage,
        UnitClass::Assassin,
        UnitClass::Cleric,
    ];

    for class in classes {
        let star1_units: Vec<(Entity, GridPos)> = units
            .iter()
            .filter(|(_, u, _, s)| u.class == class && u.faction == Faction::Player && s.0 == 1)
            .map(|(e, _, g, _)| (e, *g))
            .collect();

        if star1_units.len() >= 3 {
            let keep_pos = star1_units[0].1;
            for (e, _) in star1_units.iter().take(3) {
                commands.entity(*e).despawn_recursive();
            }
            info!("[STAR FUSION] ⭐⭐ Merged 3x 1★ {:?} into 2★ {:?} at ({}, {})!", class, class, keep_pos.col, keep_pos.row);
            crate::units::spawn_unit_ext(
                &mut commands,
                &textures,
                class,
                Faction::Player,
                keep_pos.col,
                keep_pos.row,
                2,
                false,
            );
            sound_events.send(crate::audio::PlaySoundEvent(crate::audio::SoundEffect::Ultimate));
            return;
        }

        let star2_units: Vec<(Entity, GridPos)> = units
            .iter()
            .filter(|(_, u, _, s)| u.class == class && u.faction == Faction::Player && s.0 == 2)
            .map(|(e, _, g, _)| (e, *g))
            .collect();

        if star2_units.len() >= 3 {
            let keep_pos = star2_units[0].1;
            for (e, _) in star2_units.iter().take(3) {
                commands.entity(*e).despawn_recursive();
            }
            info!("[STAR FUSION] ⭐⭐⭐ Merged 3x 2★ {:?} into 3★ {:?} at ({}, {})!", class, class, keep_pos.col, keep_pos.row);
            crate::units::spawn_unit_ext(
                &mut commands,
                &textures,
                class,
                Faction::Player,
                keep_pos.col,
                keep_pos.row,
                3,
                false,
            );
            sound_events.send(crate::audio::PlaySoundEvent(crate::audio::SoundEffect::Ultimate));
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_star_level_multipliers() {
        let (hp1, atk1) = StarLevel(1).stat_multiplier();
        let (hp2, atk2) = StarLevel(2).stat_multiplier();
        let (hp3, atk3) = StarLevel(3).stat_multiplier();

        assert_eq!(hp1, 1.0);
        assert_eq!(atk1, 1.0);
        assert!(hp2 > hp1 && atk2 > atk1);
        assert!(hp3 > hp2 && atk3 > atk2);
    }

    #[test]
    fn test_player_economy_reroll_and_buy() {
        let mut eco = PlayerEconomy {
            gold: 10,
            ..default()
        };
        let mut rng = BattleRng::default();

        assert!(eco.reroll(&mut rng));
        assert_eq!(eco.gold, 8);

        // Buy first slot
        if let Some(class) = eco.shop_slots[0] {
            let cost = unit_cost(class);
            let bought = eco.buy_slot(0);
            assert_eq!(bought, Some(class));
            assert_eq!(eco.gold, 8 - cost);
            assert_eq!(eco.shop_slots[0], None);
        }
    }
}
