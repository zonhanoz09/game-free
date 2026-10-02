use crate::{
    audio::{self, PlaySoundEvent, SoundManager},
    battle::{self, BattleRng, BattleTurnManager, CameraShake2d, HitStopManager},
    board::{self, HoveredTile, MainCamera2d},
    economy::{self, PlayerEconomy},
    net::{self, PvpManager},
    synergies,
    types::{
        BattleSpeed, CurrentStage, GameFonts, GameState, GameTextures, SelectedBenchUnit,
        SelectedUnitState,
    },
    ui, units,
};
use bevy::prelude::*;

fn setup_cameras(mut commands: Commands) {
    info!("[GAME INIT] Game starting up...");
    commands.spawn((Camera2d, MainCamera2d));
}

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "3v3 Tactical Arena - 2D Auto-Battler".to_string(),
                resolution: (1280.0_f32, 720.0_f32).into(),
                resizable: true,
                canvas: Some("#bevy-canvas".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<BattleSpeed>()
        .init_resource::<CurrentStage>()
        .init_resource::<SelectedBenchUnit>()
        .init_resource::<SelectedUnitState>()
        .init_resource::<PvpManager>()
        .init_resource::<net::PlayerDeck>()
        .init_resource::<SoundManager>()
        .add_event::<PlaySoundEvent>()
        .init_resource::<PlayerEconomy>()
        .init_resource::<BattleRng>()
        .init_resource::<BattleTurnManager>()
        .init_resource::<HoveredTile>()
        .init_resource::<HitStopManager>()
        .init_resource::<CameraShake2d>()
        .init_resource::<GameTextures>()
        .init_resource::<GameFonts>()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.06, 0.09)))
        .add_systems(
            Startup,
            (
                setup_cameras,
                board::setup_board,
                ui::setup_ui,
                ui::setup_stage_enemies,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                board::animate_torches,
                units::animate_idle_bobbing,
                board::update_cursor_hover,
                board::update_tile_visuals,
                units::update_selection_halo,
                net::pvp_network_system,
            ),
        )
        .add_systems(
            Update,
            (
                battle::update_hit_stop_system,
                battle::update_camera_shake,
                battle::update_combat_vfx,
                battle::update_spark_particles,
                battle::update_hit_recoil,
                battle::update_turn_spotlight,
                audio::sound_event_listener,
            ),
        )
        .add_systems(
            Update,
            (
                units::update_unit_health_bars,
                battle::update_floating_text,
                ui::update_hero_inspection_system,
                ui::update_shop_cards_ui,
                ui::handle_speed_toggle,
                ui::update_gold_display_system,
                synergies::update_synergies_ui,
            ),
        )
        .add_systems(
            OnEnter(GameState::Placement),
            (
                battle::on_exit_battle,
                ui::show_placement_ui_on_placement,
                ui::setup_stage_enemies,
                ui::reset_player_units_for_placement,
                ui::teardown_result_ui,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                ui::handle_shop_clicks,
                ui::handle_start_battle_button,
                ui::handle_clear_button,
                ui::handle_preset_button,
                ui::handle_unit_and_tile_interaction,
                ui::update_tooltip_system,
                ui::update_unit_count_ui,
                ui::handle_reroll_and_lock_buttons,
                ui::handle_keyboard_gameplay_shortcuts,
                economy::auto_star_fusion_system,
                ui::update_start_button_text,
            )
                .run_if(in_state(GameState::Placement)),
        )
        .add_systems(
            OnEnter(GameState::Battle),
            (battle::on_enter_battle, ui::hide_placement_ui_on_battle).chain(),
        )
        .add_systems(
            Update,
            (
                battle::battle_tick_system,
                battle::update_dash_animations,
                battle::update_projectiles,
                battle::check_unit_deaths,
                battle::check_battle_end,
            )
                .run_if(in_state(GameState::Battle)),
        )
        .add_systems(OnEnter(GameState::Victory), ui::show_victory_ui)
        .add_systems(
            Update,
            ui::handle_result_buttons.run_if(in_state(GameState::Victory)),
        )
        .add_systems(OnEnter(GameState::Defeat), ui::show_defeat_ui)
        .add_systems(
            Update,
            ui::handle_result_buttons.run_if(in_state(GameState::Defeat)),
        )
        .run();
}
