mod assets_3d;
mod battle;
mod board;
mod stages;
mod types;
mod ui;
mod units;

use assets_3d::Game3dAssets;
use battle::*;
use bevy::prelude::*;
use bevy::render::camera::ClearColorConfig;
use board::*;
use types::*;
use ui::*;
use units::*;

fn setup_cameras(mut commands: Commands) {
    // Primary 3D Camera for the 3D Colosseum Arena & Champions
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        Transform::from_xyz(0.0, 14.2, 14.6).looking_at(Vec3::new(0.0, 0.4, 0.0), Vec3::Y),
    ));

    // Secondary 2D Camera for the 2D UI Overlay (Top Bar, Bottom Bench, Results)
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Chiến Thuật 3x3 - Đấu Trường 3D (Auto-Battler)".to_string(),
                resolution: (1280.0, 720.0).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<BattleSpeed>()
        .init_resource::<CurrentStage>()
        .init_resource::<SelectedBenchUnit>()
        .init_resource::<BattleRng>()
        .init_resource::<HoveredTile>()
        .init_resource::<GameTextures>()
        .init_resource::<Game3dAssets>()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.07, 0.11)))
        .insert_resource(AmbientLight {
            color: Color::srgb(0.65, 0.72, 0.85),
            brightness: 320.0,
        })
        // Setup systems
        .add_systems(
            Startup,
            (setup_cameras, setup_board, setup_ui, setup_stage_enemies).chain(),
        )
        // Always active systems (animations, visual updates, floating texts)
        .add_systems(
            Update,
            (
                animate_brazier_flames,
                animate_idle_bobbing,
                update_tile_visuals,
                update_unit_health_bars,
                update_floating_text,
                handle_speed_toggle,
            ),
        )
        // Placement Phase Systems
        .add_systems(
            OnEnter(GameState::Placement),
            (
                on_exit_battle,
                setup_stage_enemies,
                reset_player_units_for_placement,
                teardown_result_ui,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                update_cursor_hover,
                update_bench_ui,
                handle_bench_clicks,
                handle_start_battle_button,
                handle_clear_button,
                handle_preset_button,
                handle_tile_mouse_placement,
                update_tooltip_system,
                update_unit_count_ui,
            )
                .run_if(in_state(GameState::Placement)),
        )
        // Battle Phase Systems
        .add_systems(OnEnter(GameState::Battle), on_enter_battle)
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
