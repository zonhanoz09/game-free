use super::*;

#[derive(Component)]
pub struct ActionGauge {
    pub current: f32,
}

#[derive(Component)]
pub struct FloatingText2d {
    pub velocity: Vec2,
    pub timer: Timer,
}

#[derive(Component)]
pub struct DashAnimation2d {
    pub origin: Vec2,
    pub target: Vec2,
    pub timer: Timer,
    pub returning: bool,
    pub damage_dealt: bool,
    pub target_entity: Entity,
    #[allow(dead_code)] pub attacker_entity: Entity,
    pub damage: f32,
    pub is_crit: bool,
    pub is_ultimate: bool,
    pub class: UnitClass,
}

#[derive(Component)]
pub struct Projectile2d {
    pub start: Vec2,
    pub target_pos: Vec2,
    pub target_entity: Entity,
    pub timer: Timer,
    pub damage: f32,
    pub is_heal: bool,
    pub is_crit: bool,
    pub is_ultimate: bool,
    pub aoe_row: Option<(usize, Faction)>,
    pub arc_height: f32,
    pub class: UnitClass,
}

#[derive(Component)]
pub struct CombatVfx2d {
    pub timer: Timer,
    pub initial_scale: Vec2,
    pub target_scale: Vec2,
    pub rotate_speed: f32,
}

#[derive(Component)]
pub struct SparkParticle2d {
    pub velocity: Vec2,
    pub timer: Timer,
    pub initial_color: Color,
}

#[derive(Component)]
pub struct UnitHitRecoil2d {
    pub original_pos: Vec2,
    pub recoil_offset: Vec2,
    pub timer: Timer,
}

#[derive(Component)]
pub struct ActiveTurnSpotlight2d {
    #[allow(dead_code)]
    #[allow(dead_code)] pub attacker_entity: Entity,
}

#[derive(Resource, Default)]
pub struct HitStopManager {
    pub timer: Timer,
    pub active: bool,
}

impl HitStopManager {
    pub fn trigger(&mut self, duration: f32) {
        self.timer = Timer::from_seconds(duration, TimerMode::Once);
        self.active = true;
    }

    pub fn update(&mut self, dt: f32) {
        if self.active {
            self.timer.tick(std::time::Duration::from_secs_f32(dt));
            if self.timer.finished() {
                self.active = false;
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct CameraShake2d {
    pub trauma: f32,
}

impl CameraShake2d {
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }
}

#[derive(Resource)]
pub struct BattleTurnManager {
    pub active_attacker: Option<Entity>,
    pub cooldown_timer: Timer,
    pub acted_this_cycle: std::collections::HashSet<Entity>,
    pub cycle_turn_count: u32,
    pub animation_stall_timer: f32,
}

impl Default for BattleTurnManager {
    fn default() -> Self {
        Self {
            active_attacker: None,
            cooldown_timer: Timer::from_seconds(0.35, TimerMode::Once),
            acted_this_cycle: std::collections::HashSet::new(),
            cycle_turn_count: 0,
            animation_stall_timer: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct BattleRng {
    pub state: u64,
}

impl Default for BattleRng {
    fn default() -> Self {
        Self { state: 123456789 }
    }
}

impl BattleRng {
    pub fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 32) as u32 as f32) / (u32::MAX as f32)
    }

    pub fn random_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}
