use game_logic::{
    BattleError, BattleState, BoardSlot, CombatEvent, SkillSpec, TargetRule, TeamSide, UnitState,
};
use game_protocol::{BattleResult, BattleSetup, CombatCommand, CombatReplay, PvpUnitData};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const CONFIG_VERSION: &str = "combat-v1";
const CONFIG_FINGERPRINT: &str = "release-v1";
const MAX_TURNS: u32 = 500;

pub struct AuthoritativeBattle {
    pub id: String,
    pub seed: u64,
    pub config_version: String,
    pub state: BattleState,
    accepted_commands: HashMap<String, Vec<CombatEvent>>,
    last_command_id: Option<String>,
    started_at: Instant,
    settled: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AuthorityError {
    InvalidBattleId,
    InvalidConfigVersion,
    InvalidConfigFingerprint,
    InvalidLineup(String),
    InvalidTargetRule,
    InvalidTurn,
    InvalidCommand(BattleError),
    TimedOut,
}

impl AuthoritativeBattle {
    pub fn from_setup(setup: BattleSetup) -> Result<Self, AuthorityError> {
        if setup.battle_id.trim().is_empty() {
            return Err(AuthorityError::InvalidBattleId);
        }
        if setup.config_version != CONFIG_VERSION {
            return Err(AuthorityError::InvalidConfigVersion);
        }
        if setup.config_fingerprint != CONFIG_FINGERPRINT {
            return Err(AuthorityError::InvalidConfigFingerprint);
        }
        let mut state = BattleState::default();
        add_lineup(&mut state, TeamSide::Attacker, &setup.attacker, setup.seed)?;
        add_lineup(&mut state, TeamSide::Defender, &setup.defender, setup.seed)?;
        if state.units.is_empty()
            || !state.units.iter().any(|u| u.side == TeamSide::Attacker)
            || !state.units.iter().any(|u| u.side == TeamSide::Defender)
        {
            return Err(AuthorityError::InvalidLineup(
                "Hai phe phải có ít nhất một tướng.".to_string(),
            ));
        }
        Ok(Self {
            id: setup.battle_id,
            seed: setup.seed,
            config_version: setup.config_version,
            state,
            accepted_commands: HashMap::new(),
            last_command_id: None,
            started_at: Instant::now(),
            settled: false,
        })
    }

    pub fn apply(&mut self, command: CombatCommand) -> Result<Vec<CombatEvent>, AuthorityError> {
        if self.is_timed_out() {
            return Err(AuthorityError::TimedOut);
        }
        if command.battle_id != self.id {
            return Err(AuthorityError::InvalidBattleId);
        }
        if command.command_id.trim().is_empty() {
            return Err(AuthorityError::InvalidCommand(BattleError::InvalidSkill));
        }
        if let Some(events) = self.accepted_commands.get(&command.command_id) {
            return Ok(events.clone());
        }
        if command.turn != self.state.turn {
            return Err(AuthorityError::InvalidTurn);
        }
        let target_rule = parse_target_rule(&command.target_rule)?;
        let actor = self
            .state
            .units
            .iter()
            .find(|u| u.id == command.actor_id)
            .ok_or(AuthorityError::InvalidCommand(BattleError::MissingUnit))?;

        let hero_spec = actor.hero_id.map(|h| h.as_str()).and_then(game_logic::hero_skill_spec);
        let skill = if let Some((normal, ultimate)) = hero_spec {
            let expected = if command.rage_cost == 100 {
                ultimate
            } else if command.rage_cost == 0 {
                normal
            } else {
                return Err(AuthorityError::InvalidCommand(BattleError::InvalidSkill));
            };
            if command.damage_rate_bps != expected.damage_rate_bps {
                return Err(AuthorityError::InvalidCommand(BattleError::InvalidSkill));
            }
            expected
        } else {
            if command.damage_rate_bps != 1000 || command.rage_cost != 0 {
                return Err(AuthorityError::InvalidCommand(BattleError::InvalidSkill));
            }
            SkillSpec {
                target_rule,
                damage_rate_bps: 1000,
                rage_cost: 0,
                effects: vec![game_logic::EffectKind::Damage],
            }
        };

        let events = self
            .state
            .execute_action(command.actor_id, skill, false)
            .map_err(AuthorityError::InvalidCommand)?;
        self.accepted_commands
            .insert(command.command_id.clone(), events.clone());
        self.last_command_id = Some(command.command_id);
        Ok(events)
    }

    pub fn is_timed_out(&self) -> bool {
        self.state.turn >= MAX_TURNS || self.started_at.elapsed() > Duration::from_secs(120)
    }

    pub fn is_settled(&self) -> bool {
        self.settled
    }

    pub fn mark_settled(&mut self) {
        self.settled = true;
    }

    pub fn surviving_count(&self, side: TeamSide) -> usize {
        self.state
            .units
            .iter()
            .filter(|unit| unit.side == side && unit.is_alive())
            .count()
    }

    pub fn result(&self) -> BattleResult {
        let winner = self
            .state
            .winner()
            .flatten()
            .map(|side| match side {
                TeamSide::Attacker => "ATTACKER",
                TeamSide::Defender => "DEFENDER",
            })
            .or_else(|| {
                if self.is_timed_out() {
                    let att = self.surviving_count(TeamSide::Attacker);
                    let def = self.surviving_count(TeamSide::Defender);
                    if att > def {
                        Some("ATTACKER")
                    } else if def > att {
                        Some("DEFENDER")
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
        let events = self
            .state
            .events
            .iter()
            .map(|event| serde_json::to_value(event).expect("combat events are serializable"))
            .collect();
        BattleResult {
            battle_id: self.id.clone(),
            winner: winner.map(str::to_string),
            turn: self.state.turn,
            replay: CombatReplay {
                battle_id: self.id.clone(),
                seed: self.seed,
                config_version: self.config_version.clone(),
                config_fingerprint: CONFIG_FINGERPRINT.to_string(),
                events,
            },
            accepted_command_id: self.last_command_id.clone(),
        }
    }
}

fn add_lineup(
    state: &mut BattleState,
    side: TeamSide,
    lineup: &[PvpUnitData],
    seed: u64,
) -> Result<(), AuthorityError> {
    if lineup.is_empty() || lineup.len() > 5 {
        return Err(AuthorityError::InvalidLineup(
            "Đội hình phải có từ 1 đến 5 tướng.".to_string(),
        ));
    }
    for (index, unit) in lineup.iter().enumerate() {
        if unit.col > 2 || unit.row > 2 {
            return Err(AuthorityError::InvalidLineup(
                "Vị trí tướng phải nằm trong lưới 3x3.".to_string(),
            ));
        }
        let slot = BoardSlot::new(side, unit.col as u8, unit.row as u8).ok_or_else(|| {
            AuthorityError::InvalidLineup("Vị trí tướng không hợp lệ.".to_string())
        })?;
        let base = match unit.class.as_str() {
            "zhao_yun" | "dian_wei" | "sun_ce" | "Knight" => (180, 25, 40, 18),
            "huang_zhong" | "Archer" => (110, 38, 15, 26),
            "zhuge_liang" | "cao_cao" | "guo_jia" | "lu_xun" | "jia_xu" | "Mage" => {
                (95, 45, 10, 22)
            }
            "zhang_he_yan_liang" | "Assassin" => (100, 50, 12, 34),
            "da_qiao_xiao_qiao" | "hua_tuo" | "Cleric" => (125, 20, 22, 20),
            _ => {
                return Err(AuthorityError::InvalidLineup(format!(
                    "Tướng '{}' không được đăng ký.",
                    unit.class
                )));
            }
        };
        let side_offset = match side {
            TeamSide::Attacker => 0,
            TeamSide::Defender => 10_000,
        };
        let id = seed.wrapping_add(side_offset).wrapping_add(index as u64) as u32;
        let hero = game_logic::HeroId::parse(&unit.class);
        let faction = game_logic::hero_faction(&unit.class);
        let mut unit_state = UnitState::new(
            id, side, slot, base.0, base.1, base.2, base.3,
        );
        if let Some(h) = hero {
            unit_state = unit_state.with_hero_id(h);
        }
        if let Some(f) = faction {
            unit_state = unit_state.with_faction(f);
        }
        state
            .add_unit(unit_state)
            .map_err(|error| AuthorityError::InvalidCommand(error))?;
    }
    Ok(())
}

fn parse_target_rule(value: &str) -> Result<TargetRule, AuthorityError> {
    match value {
        "DIRECT_LINE" => Ok(TargetRule::DirectLine),
        "TARGET_BACKLINE" => Ok(TargetRule::Backline),
        "TARGET_LOWEST_HP_RATIO" => Ok(TargetRule::LowestHpRatio),
        "TARGET_HIGHEST_ATK" => Ok(TargetRule::HighestAttack),
        "TARGET_CROSS" => Ok(TargetRule::Cross),
        "TARGET_PIERCE_ROW" => Ok(TargetRule::PierceRow),
        "TARGET_COLUMN" => Ok(TargetRule::Column),
        _ => Err(AuthorityError::InvalidTargetRule),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> BattleSetup {
        BattleSetup {
            battle_id: "battle-1".to_string(),
            seed: 42,
            config_version: CONFIG_VERSION.to_string(),
            config_fingerprint: CONFIG_FINGERPRINT.to_string(),
            attacker: vec![PvpUnitData {
                col: 0,
                row: 1,
                class: "zhao_yun".to_string(),
                star_level: 1,
            }],
            defender: vec![PvpUnitData {
                col: 0,
                row: 1,
                class: "huang_zhong".to_string(),
                star_level: 1,
            }],
        }
    }

    #[test]
    fn server_builds_and_replays_authoritative_action() {
        let mut battle = AuthoritativeBattle::from_setup(setup()).expect("valid setup");
        battle.state.units[0].gauge.current = 10_000;
        let events = battle
            .apply(CombatCommand {
                command_id: "cmd-1".to_string(),
                battle_id: "battle-1".to_string(),
                turn: 0,
                actor_id: 42,
                target_rule: "DIRECT_LINE".to_string(),
                damage_rate_bps: 1000,
                rage_cost: 0,
                critical: false,
            })
            .expect("valid command");
        assert!(!events.is_empty());
        let result = battle.result();
        assert_eq!(result.replay.seed, 42);
        assert_eq!(result.replay.config_version, CONFIG_VERSION);
        assert!(!result.replay.events.is_empty());
    }

    #[test]
    fn server_rejects_invalid_client_result_inputs() {
        let mut battle = AuthoritativeBattle::from_setup(setup()).expect("valid setup");
        let result = battle.apply(CombatCommand {
            command_id: "cmd-invalid".to_string(),
            battle_id: "battle-1".to_string(),
            turn: 0,
            actor_id: 42,
            target_rule: "DIRECT_LINE".to_string(),
            damage_rate_bps: 1000,
            rage_cost: 0,
            critical: false,
        });
        assert_eq!(
            result,
            Err(AuthorityError::InvalidCommand(BattleError::MissingUnit))
        );
        assert_eq!(
            AuthoritativeBattle::from_setup(BattleSetup {
                config_version: CONFIG_VERSION.to_string(),
                config_fingerprint: CONFIG_FINGERPRINT.to_string(),
                attacker: setup().attacker,
                defender: setup().defender,
                battle_id: "bad".to_string(),
                seed: 42,
            })
            .unwrap()
            .apply(CombatCommand {
                command_id: "cmd-invalid-rule".to_string(),
                battle_id: "bad".to_string(),
                turn: 0,
                actor_id: 42,
                target_rule: "UNKNOWN".to_string(),
                damage_rate_bps: 1000,
                rage_cost: 0,
                critical: false,
            }),
            Err(AuthorityError::InvalidTargetRule)
        );
    }

    #[test]
    fn duplicate_command_is_idempotent_and_tampered_skill_is_rejected() {
        let mut battle = AuthoritativeBattle::from_setup(setup()).expect("valid setup");
        battle.state.units[0].gauge.current = 10_000;
        let command = CombatCommand {
            command_id: "retry-1".to_string(),
            battle_id: "battle-1".to_string(),
            turn: 0,
            actor_id: 42,
            target_rule: "DIRECT_LINE".to_string(),
            damage_rate_bps: 1000,
            rage_cost: 0,
            critical: true,
        };
        let first = battle.apply(command.clone()).expect("first command");
        let hp_after_first = battle.state.units[1].hp;
        let second = battle.apply(command).expect("retry command");
        assert_eq!(first, second);
        assert_eq!(battle.state.units[1].hp, hp_after_first);

        battle.state.units[0].gauge.current = 10_000;
        let error = battle
            .apply(CombatCommand {
                command_id: "tampered-1".to_string(),
                damage_rate_bps: 9000,
                ..CombatCommand {
                    battle_id: "battle-1".to_string(),
                    turn: battle.state.turn,
                    actor_id: 42,
                    target_rule: "DIRECT_LINE".to_string(),
                    damage_rate_bps: 1000,
                    rage_cost: 0,
                    critical: false,
                    command_id: "unused".to_string(),
                }
            })
            .expect_err("tampered skill");
        assert_eq!(
            error,
            AuthorityError::InvalidCommand(BattleError::InvalidSkill)
        );
    }

    #[test]
    fn config_fingerprint_is_required() {
        let mut invalid = setup();
        invalid.config_fingerprint.clear();
        assert!(matches!(
            AuthoritativeBattle::from_setup(invalid),
            Err(AuthorityError::InvalidConfigFingerprint)
        ));
    }
}
