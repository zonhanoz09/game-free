#![allow(dead_code)]
//! Segregated repository interfaces for server data access (Interface Segregation Principle).

use super::data::{DeckCardEntry, PlayerDeck, User, UserCard};
use game_data_schema::PlayerFormationRow;

pub trait AuthRepository {
    fn register(
        &mut self,
        username: &str,
        password: &str,
        display_name: &str,
        avatar: &str,
    ) -> Result<User, String>;

    fn login(&mut self, username: &str, password: &str) -> Result<User, String>;
    fn get_user(&self, username: &str) -> Option<User>;
    fn customize_profile(
        &mut self,
        username: &str,
        display_name: Option<String>,
        avatar_id: Option<String>,
        cardback_id: Option<String>,
        board_skin: Option<String>,
    ) -> Result<User, String>;
}

pub trait DeckRepository {
    fn save_deck(
        &mut self,
        username: &str,
        deck_id: Option<String>,
        deck_name: String,
        hero_class: String,
        cardback_id: Option<String>,
        cards: Vec<DeckCardEntry>,
    ) -> Result<(User, PlayerDeck), String>;

    fn delete_deck(&mut self, username: &str, deck_id: &str) -> Result<User, String>;
}

pub trait EconomyRepository {
    fn buy_card(
        &mut self,
        username: &str,
        hero_class: &str,
    ) -> Result<(User, UserCard), String>;

    fn buy_battle_slot(&mut self, username: &str) -> Result<User, String>;

    fn upgrade_card(
        &mut self,
        username: &str,
        card_id: &str,
        upgrade_type: &str,
    ) -> Result<(User, String), String>;

    fn foil_card(&mut self, username: &str, card_id: &str) -> Result<(User, String), String>;
    fn sell_card(&mut self, username: &str, card_id: &str) -> Result<(User, u32), String>;
}

pub trait MatchRepository {
    fn reward_match(
        &mut self,
        username: &str,
        mode: &str,
        win: bool,
    ) -> Result<(User, u32, u64, u32, bool, String), String>;

    fn record_match(
        &mut self,
        match_id: &str,
        host_user: &str,
        guest_user: &str,
        winner_user: Option<&str>,
        rounds: usize,
    );

    fn get_leaderboard(&self, limit: usize) -> Vec<User>;
}

pub trait FormationRepository {
    fn get_user_formation(&self, username: &str, formation_type: &str) -> Option<PlayerFormationRow>;
    fn save_user_formation(
        &mut self,
        username: &str,
        formation: PlayerFormationRow,
    ) -> Result<(), String>;
}
