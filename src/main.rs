#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::collapsible_if
)]

mod audio;
mod battle;
mod board;
mod economy;
mod stages;
mod synergies;
mod types;
mod ui;
mod units;
mod net;

use audio::*;
use battle::*;
use bevy::prelude::*;
use board::*;
use economy::*;
use synergies::*;
use types::*;
use ui::*;
use units::*;
use net::*;

fn setup_cameras(mut commands: Commands) {
    info!("[GAME INIT] Game starting up...");
    // Single 2D Camera for 2D Arena, Champions, Combat Effects, and UI
    commands.spawn((Camera2d, MainCamera2d));
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

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
        // Setup systems
        .add_systems(
            Startup,
            (setup_cameras, setup_board, setup_ui, setup_stage_enemies).chain(),
        )
        // Group 1: Arena visual animations, cursor hover, selection halo & tile visuals
        .add_systems(
            Update,
            (
                animate_torches,
                animate_idle_bobbing,
                update_cursor_hover,
                update_tile_visuals,
                update_selection_halo,
                pvp_network_system,
            ),
        )
        // Group 2: Combat Game Feel (Hit Stop, 2D Camera Shake, VFX, Particles, Recoil, Spotlight, Audio)
        .add_systems(
            Update,
            (
                update_hit_stop_system,
                update_camera_shake,
                update_combat_vfx,
                update_spark_particles,
                update_hit_recoil,
                update_turn_spotlight,
                sound_event_listener,
            ),
        )
        // Group 3: HUD, Floating Combat Text, Hero Inspection Card, Gold & Synergies
        .add_systems(
            Update,
            (
                update_unit_health_bars,
                update_floating_text,
                update_hero_inspection_system,
                update_shop_cards_ui,
                handle_speed_toggle,
                update_gold_display_system,
                update_synergies_ui,
            ),
        )
        // Placement Phase Systems
        .add_systems(
            OnEnter(GameState::Placement),
            (
                on_exit_battle,
                show_placement_ui_on_placement,
                setup_stage_enemies,
                reset_player_units_for_placement,
                teardown_result_ui,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                handle_shop_clicks,
                handle_start_battle_button,
                handle_clear_button,
                handle_preset_button,
                handle_unit_and_tile_interaction,
                update_tooltip_system,
                update_unit_count_ui,
                handle_reroll_and_lock_buttons,
                handle_keyboard_gameplay_shortcuts,
                auto_star_fusion_system,
                update_start_button_text,
            )
                .run_if(in_state(GameState::Placement)),
        )
        // Battle Phase Systems
        .add_systems(
            OnEnter(GameState::Battle),
            (on_enter_battle, hide_placement_ui_on_battle).chain(),
        )
        .add_systems(
            Update,
            (
                battle_tick_system,
                update_dash_animations,
                update_projectiles,
                check_unit_deaths,
                check_battle_end,
            )
                .run_if(in_state(GameState::Battle)),
        )
        // Victory Phase Systems
        .add_systems(OnEnter(GameState::Victory), show_victory_ui)
        .add_systems(
            Update,
            handle_result_buttons.run_if(in_state(GameState::Victory)),
        )
        // Defeat Phase Systems
        .add_systems(OnEnter(GameState::Defeat), show_defeat_ui)
        .add_systems(
            Update,
            handle_result_buttons.run_if(in_state(GameState::Defeat)),
        )
        .run();
}

