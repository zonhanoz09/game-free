use crate::types::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
pub struct MainCamera2d;

#[derive(Component)]
pub struct TileEntity {
    pub col: usize,
    pub row: usize,
    pub faction: Faction,
}

#[derive(Component)]
pub struct TileBorderVisual;

#[derive(Component)]
pub struct TileCoreVisual;

#[derive(Component)]
pub struct BenchSlotEntity {
    pub slot: usize,
}

#[derive(Component)]
pub struct BenchSlotBorderVisual;

#[derive(Component)]
pub struct BenchSlotCoreVisual;

#[derive(Component)]
pub struct TorchFlameEmitter {
    pub timer: Timer,
    pub pos: Vec2,
}

#[derive(Component)]
pub struct TorchParticle {
    pub velocity: Vec2,
    pub timer: Timer,
}

#[derive(Component)]
pub struct CenterDividerGlow;

#[derive(Resource, Default)]
pub struct HoveredTile {
    pub tile: Option<GridPos>,
    pub bench_slot: Option<usize>,
}

pub fn grid_to_world_pos(col: usize, row: usize, faction: Faction) -> Vec2 {
    let col_pitch = TILE_SIZE + TILE_GAP;
    let row_pitch = TILE_SIZE + TILE_GAP;
    let center_gap = 50.0;

    let x = match faction {
        Faction::Player => ARENA_CENTER_X - center_gap - (2.0 - col as f32) * col_pitch,
        Faction::Enemy => ARENA_CENTER_X + center_gap + (col as f32) * col_pitch,
    };

    let y = match row {
        0 => ARENA_CENTER_Y + row_pitch,
        1 => ARENA_CENTER_Y,
        _ => ARENA_CENTER_Y - row_pitch,
    };

    Vec2::new(x, y)
}

pub fn bench_world_pos(slot: usize) -> Vec2 {
    let start_x = -340.0;
    let pitch = 54.0;
    let x = start_x + (slot as f32) * pitch;
    let y = -165.0;
    Vec2::new(x, y)
}

pub fn setup_board(mut commands: Commands, textures: Res<GameTextures>) {
    // 1. Background Arena Artwork
    commands.spawn((
        Sprite {
            image: textures.background.clone(),
            custom_size: Some(Vec2::new(1280.0, 720.0)),
            color: Color::srgba(0.55, 0.58, 0.68, 1.0),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -100.0),
    ));

    // 2. Colosseum Dais Foundation (Dark Slate Stone Platform)
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(660.0, 360.0)),
            color: Color::srgba(0.06, 0.08, 0.12, 0.88),
            ..default()
        },
        Transform::from_xyz(ARENA_CENTER_X, ARENA_CENTER_Y, -30.0),
    ));

    // Dais Bronze Border Trim
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(664.0, 364.0)),
            color: Color::srgba(0.55, 0.45, 0.25, 0.50),
            ..default()
        },
        Transform::from_xyz(ARENA_CENTER_X, ARENA_CENTER_Y, -31.0),
    ));

    // Player Field Backing (Subtle Sapphire Glow)
    let p_mid = grid_to_world_pos(1, 1, Faction::Player);
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(304.0, 304.0)),
            color: Color::srgba(0.10, 0.22, 0.45, 0.35),
            ..default()
        },
        Transform::from_xyz(p_mid.x, p_mid.y, -25.0),
    ));

    // Enemy Field Backing (Subtle Crimson Glow)
    let e_mid = grid_to_world_pos(1, 1, Faction::Enemy);
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(304.0, 304.0)),
            color: Color::srgba(0.45, 0.12, 0.14, 0.35),
            ..default()
        },
        Transform::from_xyz(e_mid.x, e_mid.y, -25.0),
    ));

    // (Reserve bench removed per UI optimization requirements)

    // 4. Central Golden Energy Divider Beam
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(4.0, 330.0)),
            color: Color::srgb(1.0, 0.84, 0.30),
            ..default()
        },
        Transform::from_xyz(ARENA_CENTER_X, ARENA_CENTER_Y, -10.0),
        CenterDividerGlow,
    ));

    // Central Energy Soft Glow
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(16.0, 330.0)),
            color: Color::srgba(1.0, 0.85, 0.30, 0.18),
            ..default()
        },
        Transform::from_xyz(ARENA_CENTER_X, ARENA_CENTER_Y, -11.0),
    ));

    // Center VS Medallion
    commands
        .spawn((
            Sprite {
                custom_size: Some(Vec2::new(36.0, 36.0)),
                color: Color::srgb(0.12, 0.15, 0.22),
                ..default()
            },
            Transform::from_xyz(ARENA_CENTER_X, ARENA_CENTER_Y, -8.0),
        ))
        .with_child((
            Sprite {
                custom_size: Some(Vec2::new(28.0, 28.0)),
                color: Color::srgb(1.0, 0.82, 0.25),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));

    // 5. Four Corner Braziers / Torches
    let torch_positions = [
        Vec2::new(ARENA_CENTER_X - 325.0, ARENA_CENTER_Y + 175.0),
        Vec2::new(ARENA_CENTER_X - 325.0, ARENA_CENTER_Y - 175.0),
        Vec2::new(ARENA_CENTER_X + 325.0, ARENA_CENTER_Y + 175.0),
        Vec2::new(ARENA_CENTER_X + 325.0, ARENA_CENTER_Y - 175.0),
    ];

    for t_pos in torch_positions {
        commands.spawn((
            Sprite {
                custom_size: Some(Vec2::new(14.0, 24.0)),
                color: Color::srgb(0.35, 0.28, 0.18),
                ..default()
            },
            Transform::from_xyz(t_pos.x, t_pos.y - 8.0, -5.0),
        ));

        commands.spawn((
            Sprite {
                custom_size: Some(Vec2::new(12.0, 14.0)),
                color: Color::srgb(1.0, 0.65, 0.15),
                ..default()
            },
            Transform::from_xyz(t_pos.x, t_pos.y + 4.0, -4.0),
        ));

        commands.spawn(TorchFlameEmitter {
            timer: Timer::from_seconds(0.12, TimerMode::Repeating),
            pos: Vec2::new(t_pos.x, t_pos.y + 6.0),
        });
    }

    // 6. 18 Combat Tiles (Player 3x3 + Enemy 3x3)
    let factions = [
        (
            Faction::Player,
            Color::srgb(0.12, 0.35, 0.65), // Blue border
            Color::srgb(0.07, 0.13, 0.24), // Dark Navy core
        ),
        (
            Faction::Enemy,
            Color::srgb(0.65, 0.18, 0.22), // Crimson border
            Color::srgb(0.20, 0.08, 0.10), // Dark Crimson core
        ),
    ];

    for (faction, border_col, core_col) in factions {
        for col in 0..GRID_COLS {
            for row in 0..GRID_ROWS {
                let pos = grid_to_world_pos(col, row, faction);

                commands
                    .spawn((
                        TileEntity { col, row, faction },
                        Transform::from_xyz(pos.x, pos.y, 0.0),
                        Visibility::default(),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TileBorderVisual,
                            Sprite {
                                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                                color: border_col,
                                ..default()
                            },
                            Transform::from_xyz(0.0, 0.0, 0.0),
                        ));

                        parent.spawn((
                            TileCoreVisual,
                            Sprite {
                                custom_size: Some(Vec2::new(TILE_SIZE - 6.0, TILE_SIZE - 6.0)),
                                color: core_col,
                                ..default()
                            },
                            Transform::from_xyz(0.0, 0.0, 0.5),
                        ));
                    });
            }
        }
    }

    // 7. (Reserve Bench Pedestals removed - all units deploy directly on the 3x3 battlefield)
}

pub fn animate_torches(
    mut commands: Commands,
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    mut divider_query: Query<&mut Sprite, With<CenterDividerGlow>>,
    mut emitter_query: Query<&mut TorchFlameEmitter>,
    mut particle_query: Query<
        (Entity, &mut Transform, &mut Sprite, &mut TorchParticle),
        Without<CenterDividerGlow>,
    >,
) {
    let dt = time.delta_secs() * speed.multiplier;
    let t = time.elapsed_secs() * speed.multiplier;

    // 1. Divider subtle pulse
    for mut sprite in divider_query.iter_mut() {
        let alpha = 0.85 + (t * 4.0).sin() * 0.15;
        sprite.color = Color::srgba(1.0, 0.84, 0.30, alpha);
    }

    // 2. Torch Ember Emitter
    for mut emitter in emitter_query.iter_mut() {
        emitter.timer.tick(std::time::Duration::from_secs_f32(dt));
        if emitter.timer.just_finished() {
            let vx = (t * 11.0).sin() * 12.0;
            let vy = 26.0 + (t * 7.0).cos().abs() * 22.0;

            commands.spawn((
                Sprite {
                    custom_size: Some(Vec2::splat(4.5)),
                    color: Color::srgb(1.0, 0.75, 0.25),
                    ..default()
                },
                Transform::from_xyz(emitter.pos.x, emitter.pos.y, -3.0),
                TorchParticle {
                    velocity: Vec2::new(vx, vy),
                    timer: Timer::from_seconds(0.85, TimerMode::Once),
                },
            ));
        }
    }

    // 3. Update torch particles
    for (entity, mut transform, mut sprite, mut p) in particle_query.iter_mut() {
        p.timer.tick(std::time::Duration::from_secs_f32(dt));
        transform.translation.x += p.velocity.x * dt;
        transform.translation.y += p.velocity.y * dt;

        let frac = p.timer.fraction();
        let scale = (1.0 - frac).max(0.01);
        transform.scale = Vec3::splat(scale);
        sprite.color = Color::srgba(1.0, 0.5 + 0.3 * (1.0 - frac), 0.1, 1.0 - frac);

        if p.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_cursor_hover(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera2d>>,
    mut hovered: ResMut<HoveredTile>,
) {
    let Ok(window) = windows.get_single() else {
        hovered.tile = None;
        hovered.bench_slot = None;
        return;
    };
    let Ok((camera, camera_transform)) = cameras.get_single() else {
        hovered.tile = None;
        hovered.bench_slot = None;
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        hovered.tile = None;
        hovered.bench_slot = None;
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        hovered.tile = None;
        hovered.bench_slot = None;
        return;
    };

    let half = TILE_SIZE * 0.5;
    let mut found_tile = None;

    for &faction in &[Faction::Player, Faction::Enemy] {
        for col in 0..GRID_COLS {
            for row in 0..GRID_ROWS {
                let center = grid_to_world_pos(col, row, faction);
                if (world_pos.x - center.x).abs() <= half && (world_pos.y - center.y).abs() <= half
                {
                    found_tile = Some(GridPos { col, row, faction });
                    break;
                }
            }
            if found_tile.is_some() {
                break;
            }
        }
        if found_tile.is_some() {
            break;
        }
    }

    hovered.tile = found_tile;
    hovered.bench_slot = None;
}

pub fn update_tile_visuals(
    hovered: Res<HoveredTile>,
    mut tiles: Query<(&TileEntity, &mut Transform, &Children)>,
    mut bench_tiles: Query<(&BenchSlotEntity, &mut Transform, &Children), Without<TileEntity>>,
    tile_borders: Query<(), With<TileBorderVisual>>,
    tile_cores: Query<(), With<TileCoreVisual>>,
    bench_borders: Query<(), With<BenchSlotBorderVisual>>,
    bench_cores: Query<(), With<BenchSlotCoreVisual>>,
    mut sprites: Query<&mut Sprite>,
) {
    // 1. Grid Tiles
    for (tile, mut transform, children) in tiles.iter_mut() {
        let is_hovered = hovered
            .tile
            .as_ref()
            .is_some_and(|h| h.col == tile.col && h.row == tile.row && h.faction == tile.faction);

        if is_hovered {
            transform.scale = Vec3::splat(1.05);
            for &child in children.iter() {
                if tile_borders.contains(child) {
                    if let Ok(mut border) = sprites.get_mut(child) {
                        border.color = match tile.faction {
                            Faction::Player => Color::srgb(0.35, 0.85, 1.0),
                            Faction::Enemy => Color::srgb(1.0, 0.45, 0.35),
                        };
                    }
                } else if tile_cores.contains(child) {
                    if let Ok(mut core) = sprites.get_mut(child) {
                        core.color = match tile.faction {
                            Faction::Player => Color::srgb(0.12, 0.22, 0.42),
                            Faction::Enemy => Color::srgb(0.35, 0.12, 0.15),
                        };
                    }
                }
            }
        } else {
            transform.scale = Vec3::splat(1.0);
            for &child in children.iter() {
                if tile_borders.contains(child) {
                    if let Ok(mut border) = sprites.get_mut(child) {
                        border.color = match tile.faction {
                            Faction::Player => Color::srgb(0.12, 0.35, 0.65),
                            Faction::Enemy => Color::srgb(0.65, 0.18, 0.22),
                        };
                    }
                } else if tile_cores.contains(child) {
                    if let Ok(mut core) = sprites.get_mut(child) {
                        core.color = match tile.faction {
                            Faction::Player => Color::srgb(0.07, 0.13, 0.24),
                            Faction::Enemy => Color::srgb(0.20, 0.08, 0.10),
                        };
                    }
                }
            }
        }
    }

    // 2. Bench Slot Pedestals
    for (bench, mut transform, children) in bench_tiles.iter_mut() {
        let is_hovered = hovered.bench_slot == Some(bench.slot);
        if is_hovered {
            transform.scale = Vec3::splat(1.08);
            for &child in children.iter() {
                if bench_borders.contains(child) {
                    if let Ok(mut border) = sprites.get_mut(child) {
                        border.color = Color::srgb(1.0, 0.85, 0.25); // Gold glow
                    }
                } else if bench_cores.contains(child) {
                    if let Ok(mut core) = sprites.get_mut(child) {
                        core.color = Color::srgb(0.15, 0.25, 0.40);
                    }
                }
            }
        } else {
            transform.scale = Vec3::splat(1.0);
            for &child in children.iter() {
                if bench_borders.contains(child) {
                    if let Ok(mut border) = sprites.get_mut(child) {
                        border.color = Color::srgba(0.25, 0.42, 0.65, 0.80);
                    }
                } else if bench_cores.contains(child) {
                    if let Ok(mut core) = sprites.get_mut(child) {
                        core.color = Color::srgba(0.09, 0.14, 0.22, 0.90);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_positions_within_arena() {
        for &faction in &[Faction::Player, Faction::Enemy] {
            for col in 0..GRID_COLS {
                for row in 0..GRID_ROWS {
                    let pos = grid_to_world_pos(col, row, faction);
                    // Check bounds within 1280x720 window
                    assert!(pos.x > -640.0 && pos.x < 640.0);
                    assert!(pos.y > -360.0 && pos.y < 360.0);
                }
            }
        }
    }

    #[test]
    fn test_player_and_enemy_separation() {
        let player_front = grid_to_world_pos(2, 1, Faction::Player);
        let enemy_front = grid_to_world_pos(0, 1, Faction::Enemy);
        // Player frontline must be strictly to the left of enemy frontline
        assert!(player_front.x < enemy_front.x);
        // Ensure there is at least TILE_SIZE gap between the centers
        assert!(enemy_front.x - player_front.x >= TILE_SIZE);
    }

    #[test]
    fn test_bench_positions_within_arena() {
        for slot in 0..BENCH_SLOTS {
            let pos = bench_world_pos(slot);
            assert!(pos.x > -640.0 && pos.x < 640.0);
            assert!(pos.y > -360.0 && pos.y < 360.0);
            // All bench slots should be on the player side
            assert!(pos.x < ARENA_CENTER_X);
        }
    }

    #[test]
    fn test_reserve_bench_slot_spacing_and_ordering() {
        for slot in 0..BENCH_SLOTS - 1 {
            let pos_a = bench_world_pos(slot);
            let pos_b = bench_world_pos(slot + 1);
            assert!(pos_b.x > pos_a.x);
            assert_eq!(pos_a.y, pos_b.y);
            assert!((pos_b.x - pos_a.x - 54.0).abs() < 0.001);
        }
    }
}
