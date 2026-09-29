use crate::assets_3d::Game3dAssets;
use crate::types::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
pub struct BoardTile {
    pub col: usize,
    pub row: usize,
    pub faction: Faction,
    pub base_y: f32,
}

#[derive(Component)]
pub struct BrazierFlame {
    pub phase: f32,
}

#[derive(Resource, Default)]
pub struct HoveredTile {
    pub tile: Option<GridPos>,
}

pub fn grid_to_world_pos(col: usize, row: usize, faction: Faction) -> Vec3 {
    let x = match faction {
        Faction::Player => PLAYER_COL_X[col.min(2)],
        Faction::Enemy => ENEMY_COL_X[col.min(2)],
    };
    let z = ROW_Z[row.min(2)];
    Vec3::new(x, UNIT_BASE_Y, z)
}

pub fn world_to_grid_pos(hit_point: Vec3) -> Option<GridPos> {
    for faction in [Faction::Player, Faction::Enemy] {
        for col in 0..BOARD_COLS {
            for row in 0..BOARD_ROWS {
                let tile_pos = grid_to_world_pos(col, row, faction);
                let dx = (hit_point.x - tile_pos.x).abs();
                let dz = (hit_point.z - tile_pos.z).abs();

                if dx <= TILE_SIZE * 0.5 && dz <= TILE_SIZE * 0.5 {
                    return Some(GridPos { col, row, faction });
                }
            }
        }
    }
    None
}

pub fn setup_board(mut commands: Commands, assets_3d: Res<Game3dAssets>) {
    // Main 3D Directional Sunlight
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.96, 0.90),
            illuminance: 14_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 22.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Secondary fill light for soft shadows
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.4, 0.6, 0.9),
            illuminance: 5_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-10.0, 15.0, -8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Main 3D Arena Podium ("Sàn Đấu")
    commands.spawn((
        Mesh3d(assets_3d.arena_floor.clone()),
        MeshMaterial3d(assets_3d.arena_stone.clone()),
        Transform::from_xyz(0.0, -0.4, 0.0),
    ));

    // Raised decorative outer rim / beveled stone dais base
    commands.spawn((
        Mesh3d(assets_3d.arena_rim.clone()),
        MeshMaterial3d(assets_3d.arena_rim_mat.clone()),
        Transform::from_xyz(0.0, -0.6, 0.0),
    ));

    // Glowing central division line across the arena
    commands.spawn((
        Mesh3d(assets_3d.divider.clone()),
        MeshMaterial3d(assets_3d.divider_mat.clone()),
        Transform::from_xyz(0.0, 0.015, 0.0),
    ));

    // Central Arena Centerpiece / VS Emblem
    commands.spawn((
        Mesh3d(assets_3d.cylinder.clone()),
        MeshMaterial3d(assets_3d.gold_trim.clone()),
        Transform::from_xyz(0.0, 0.02, 0.0).with_scale(Vec3::new(1.2, 0.02, 1.2)),
    ));
    commands.spawn((
        Mesh3d(assets_3d.cylinder.clone()),
        MeshMaterial3d(assets_3d.arena_stone.clone()),
        Transform::from_xyz(0.0, 0.025, 0.0).with_scale(Vec3::new(1.0, 0.02, 1.0)),
    ));

    // 4 Grand Arena Corner Pillars with Braziers & Fire Orbs
    let corner_positions = [(-8.0, -4.8), (8.0, -4.8), (-8.0, 4.8), (8.0, 4.8)];

    for (idx, (cx, cz)) in corner_positions.iter().enumerate() {
        // Stone Pillar
        commands.spawn((
            Mesh3d(assets_3d.arena_pillar.clone()),
            MeshMaterial3d(assets_3d.arena_pillar_mat.clone()),
            Transform::from_xyz(*cx, 0.9, *cz),
        ));

        // Brazier Bowl
        commands.spawn((
            Mesh3d(assets_3d.arena_brazier.clone()),
            MeshMaterial3d(assets_3d.arena_rim_mat.clone()),
            Transform::from_xyz(*cx, 1.85, *cz),
        ));

        // Glowing Flame Crystal
        commands.spawn((
            Mesh3d(assets_3d.sphere.clone()),
            MeshMaterial3d(assets_3d.brazier_fire.clone()),
            Transform::from_xyz(*cx, 2.15, *cz).with_scale(Vec3::splat(0.38)),
            BrazierFlame {
                phase: idx as f32 * 1.5,
            },
        ));

        // Warm Arena Torch Point Light
        commands.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.65, 0.25),
                intensity: 65_000.0,
                range: 12.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(*cx, 2.4, *cz),
        ));
    }

    // Spawn 18 3D Grid Tiles (3x3 Player + 3x3 Enemy)
    for faction in [Faction::Player, Faction::Enemy] {
        let (slab_mat, border_mat) = match faction {
            Faction::Player => (assets_3d.tile_player.clone(), assets_3d.player_base.clone()),
            Faction::Enemy => (assets_3d.tile_enemy.clone(), assets_3d.enemy_base.clone()),
        };

        for col in 0..BOARD_COLS {
            for row in 0..BOARD_ROWS {
                let pos = grid_to_world_pos(col, row, faction);
                let tile_y = TILE_HEIGHT * 0.5;

                // Base Border Slab
                commands.spawn((
                    Mesh3d(assets_3d.tile_border.clone()),
                    MeshMaterial3d(border_mat.clone()),
                    Transform::from_xyz(pos.x, TILE_HEIGHT * 0.25, pos.z),
                ));

                // Interactive Tile Top Slab
                commands.spawn((
                    Mesh3d(assets_3d.tile_slab.clone()),
                    MeshMaterial3d(slab_mat.clone()),
                    Transform::from_xyz(pos.x, tile_y, pos.z),
                    BoardTile {
                        col,
                        row,
                        faction,
                        base_y: tile_y,
                    },
                ));
            }
        }
    }
}

pub fn animate_brazier_flames(time: Res<Time>, mut flames: Query<(&mut Transform, &BrazierFlame)>) {
    let t = time.elapsed_secs();
    for (mut transform, flame) in flames.iter_mut() {
        let pulse = (t * 4.0 + flame.phase).sin() * 0.06;
        let flicker_y = (t * 6.0 + flame.phase).cos() * 0.04;
        transform.scale = Vec3::splat(0.38 + pulse);
        transform.translation.y = 2.15 + flicker_y;
    }
}

pub fn update_cursor_hover(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut hovered: ResMut<HoveredTile>,
    state: Res<State<GameState>>,
) {
    if *state.get() != GameState::Placement {
        hovered.tile = None;
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.get_single() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            if ray.direction.y.abs() > 0.0001 {
                let t = (TILE_SURFACE_Y - ray.origin.y) / ray.direction.y;
                if t > 0.0 {
                    let hit_point = ray.origin + *ray.direction * t;
                    hovered.tile = world_to_grid_pos(hit_point);
                    return;
                }
            }
        }
    }

    hovered.tile = None;
}

pub fn update_tile_visuals(
    hovered: Res<HoveredTile>,
    assets_3d: Res<Game3dAssets>,
    mut tiles: Query<(
        &BoardTile,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (tile, mut transform, mut material) in tiles.iter_mut() {
        let is_hovered = hovered.tile.as_ref().map_or(false, |h| {
            h.col == tile.col && h.row == tile.row && h.faction == tile.faction
        });

        let target_y = if is_hovered {
            tile.base_y + 0.12
        } else {
            tile.base_y
        };

        // Smooth elevation transition
        transform.translation.y += (target_y - transform.translation.y) * (18.0 * dt).min(1.0);

        match tile.faction {
            Faction::Player => {
                if is_hovered {
                    material.0 = assets_3d.tile_player_hover.clone();
                } else {
                    material.0 = assets_3d.tile_player.clone();
                }
            }
            Faction::Enemy => {
                if is_hovered {
                    material.0 = assets_3d.tile_enemy_hover.clone();
                } else {
                    material.0 = assets_3d.tile_enemy.clone();
                }
            }
        }
    }
}
