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

#[derive(Component)]
pub struct MainCamera3d;

fn setup_cameras(mut commands: Commands) {
    // Primary 3D Camera for the 3D Colosseum Arena & Champions
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        Transform::from_xyz(0.0, 14.2, 14.6).looking_at(Vec3::new(0.0, 0.4, 0.0), Vec3::Y),
        MainCamera3d,
    ));

    // Secondary 2D Camera for the 2D UI Overlay & Floating Combat Texts
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
                title: "3v3 Tactical Arena - 3D Auto-Battler".to_string(),
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
        .init_resource::<BattleTurnManager>()
        .init_resource::<HoveredTile>()
        .init_resource::<HitStopManager>()
        .init_resource::<CameraShake>()
        .init_resource::<GameTextures>()
        .init_resource::<Game3dAssets>()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.07, 0.11)))
        .insert_resource(AmbientLight {
            color: Color::srgb(0.68, 0.74, 0.88),
            brightness: 380.0,
        })
        // Setup systems
        .add_systems(
            Startup,
            (setup_cameras, setup_board, setup_ui, setup_stage_enemies).chain(),
        )
        // Group 1: Arena animations & tiles
        .add_systems(
            Update,
            (
                animate_brazier_flames,
                animate_idle_bobbing,
                animate_spinning_items,
                animate_orbiting_motes,
                update_cursor_hover,
                update_tile_visuals,
            ),
        )
        // Group 2: Game Feel (Hit Stop, Camera Shake, Dynamic Flash Lights, Combat VFX)
        .add_systems(
            Update,
            (
                update_hit_stop_system,
                update_camera_shake,
                update_flash_lights,
                update_combat_vfx,
                update_turn_spotlight,
            ),
        )
        // Group 3: HUD, Floating texts, Hero Inspection & Speed
        .add_systems(
            Update,
            (
                update_unit_health_bars,
                update_floating_text,
                update_hero_inspection_system,
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
