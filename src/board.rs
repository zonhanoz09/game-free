use crate::assets_3d::Game3dAssets;
use crate::types::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
pub struct TileEntity {
    pub col: usize,
    pub row: usize,
    pub faction: Faction,
}

#[derive(Component)]
pub struct TileBorderVisual;

#[derive(Component)]
pub struct BrazierLight {
    pub base_intensity: f32,
    pub phase: f32,
}

#[derive(Component)]
pub struct BrazierEmberEmitter {
    pub timer: Timer,
    pub pos: Vec3,
}

#[derive(Component)]
pub struct FloatingEmber {
    pub velocity: Vec3,
    pub timer: Timer,
}

#[derive(Component)]
pub struct CenterRuneMedallion;

#[derive(Resource, Default)]
pub struct HoveredTile {
    pub tile: Option<GridPos>,
}

pub fn grid_to_world_pos(col: usize, row: usize, faction: Faction) -> Vec3 {
    let col_pitch = TILE_SIZE + TILE_GAP;
    let row_pitch = TILE_SIZE + TILE_GAP;

    let z = (row as f32 - 1.0) * row_pitch;
    let x = match faction {
        Faction::Player => -(2.5 - col as f32) * col_pitch - 0.7,
        Faction::Enemy => (0.5 + col as f32) * col_pitch + 0.7,
    };

    Vec3::new(x, TILE_HEIGHT * 0.5 + 0.04, z)
}

pub fn setup_board(mut commands: Commands, assets_3d: Res<Game3dAssets>) {
    // 1. Colosseum Stone Ground Foundation & Outer Rim
    commands.spawn((
        Mesh3d(assets_3d.arena_rim.clone()),
        MeshMaterial3d(assets_3d.arena_rim_mat.clone()),
        Transform::from_xyz(0.0, -0.40, 0.0),
    ));

    commands.spawn((
        Mesh3d(assets_3d.arena_floor.clone()),
        MeshMaterial3d(assets_3d.arena_stone.clone()),
        Transform::from_xyz(0.0, -0.20, 0.0),
    ));

    // 2. Central Golden Arena Divider Line
    commands.spawn((
        Mesh3d(assets_3d.divider.clone()),
        MeshMaterial3d(assets_3d.divider_mat.clone()),
        Transform::from_xyz(0.0, 0.02, 0.0),
    ));

    // 3. Central Ethereal Runic Medallion
    commands.spawn((
        Mesh3d(assets_3d.rune_disc.clone()),
        MeshMaterial3d(assets_3d.rune_base_mat.clone()),
        Transform::from_xyz(0.0, 0.025, 0.0),
    ));
    commands.spawn((
        Mesh3d(assets_3d.rune_circle.clone()),
        MeshMaterial3d(assets_3d.rune_glow_mat.clone()),
        Transform::from_xyz(0.0, 0.04, 0.0),
        CenterRuneMedallion,
    ));

    // 4. Four Grand Corner Pillars & Hanging Banners
    let pillar_coords = [
        (-7.8, -4.6, Faction::Player),
        (-7.8, 4.6, Faction::Player),
        (7.8, -4.6, Faction::Enemy),
        (7.8, 4.6, Faction::Enemy),
    ];

    for (px, pz, faction) in pillar_coords {
        // Stone Pillar
        commands.spawn((
            Mesh3d(assets_3d.arena_pillar.clone()),
            MeshMaterial3d(assets_3d.arena_pillar_mat.clone()),
            Transform::from_xyz(px, 1.1, pz),
        ));

        // Brazier Basin
        commands.spawn((
            Mesh3d(assets_3d.arena_brazier.clone()),
            MeshMaterial3d(assets_3d.arena_rim_mat.clone()),
            Transform::from_xyz(px, 2.3, pz),
        ));

        // Blazing Fire Crystal Core
        commands.spawn((
            Mesh3d(assets_3d.sphere.clone()),
            MeshMaterial3d(assets_3d.brazier_fire.clone()),
            Transform::from_xyz(px, 2.65, pz).with_scale(Vec3::splat(0.26)),
        ));

        // Point Light for dramatic fire glow
        let phase = px * 1.5 + pz * 0.7;
        commands.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.65, 0.2),
                intensity: 4500.0,
                range: 8.5,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(px, 2.9, pz),
            BrazierLight {
                base_intensity: 4500.0,
                phase,
            },
        ));

        // Floating Ember Emitter
        commands.spawn((BrazierEmberEmitter {
            timer: Timer::from_seconds(0.18, TimerMode::Repeating),
            pos: Vec3::new(px, 2.7, pz),
        },));

        // Royal Hanging War Banners on Pillars
        let banner_mat = match faction {
            Faction::Player => assets_3d.banner_blue_mat.clone(),
            Faction::Enemy => assets_3d.banner_red_mat.clone(),
        };

        // Banner Pole Crossbar
        commands.spawn((
            Mesh3d(assets_3d.banner_pole.clone()),
            MeshMaterial3d(assets_3d.banner_pole_mat.clone()),
            Transform::from_xyz(px, 1.9, pz).with_rotation(Quat::from_rotation_z(1.5708)),
        ));

        // Banner Cloth
        commands.spawn((
            Mesh3d(assets_3d.banner_cloth.clone()),
            MeshMaterial3d(banner_mat),
            Transform::from_xyz(px, 1.15, pz + if pz > 0.0 { -0.15 } else { 0.15 }),
        ));
    }

    // 5. Directional Arena Sunlight & Fill Lighting
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.96, 0.90),
            illuminance: 12000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(9.0, 18.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.55, 0.65, 0.90),
            illuminance: 3200.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-9.0, 12.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 6. Interactive 3D Combat Tiles (Player 3x3 + Enemy 3x3)
    let factions = [
        (Faction::Player, assets_3d.tile_player.clone()),
        (Faction::Enemy, assets_3d.tile_enemy.clone()),
    ];

    for (faction, base_mat) in factions {
        for col in 0..GRID_COLS {
            for row in 0..GRID_ROWS {
                let pos = grid_to_world_pos(col, row, faction);

                // Slab Root
                commands
                    .spawn((
                        TileEntity { col, row, faction },
                        Mesh3d(assets_3d.tile_slab.clone()),
                        MeshMaterial3d(base_mat.clone()),
                        Transform::from_translation(pos),
                    ))
                    .with_children(|parent| {
                        // Outer Border Frame
                        parent.spawn((
                            TileBorderVisual,
                            Mesh3d(assets_3d.tile_border.clone()),
                            MeshMaterial3d(assets_3d.arena_stone.clone()),
                            Transform::from_xyz(0.0, -0.015, 0.0),
                        ));
                    });
            }
        }
    }
}

pub fn animate_brazier_flames(
    mut commands: Commands,
    time: Res<Time>,
    assets_3d: Res<Game3dAssets>,
    mut light_query: Query<(&mut PointLight, &BrazierLight)>,
    mut rune_query: Query<&mut Transform, With<CenterRuneMedallion>>,
    mut emitter_query: Query<&mut BrazierEmberEmitter>,
    mut ember_query: Query<
        (Entity, &mut Transform, &mut FloatingEmber),
        Without<CenterRuneMedallion>,
    >,
) {
    let t = time.elapsed_secs();
    let dt = time.delta_secs();

    // 1. Flicker brazier lights
    for (mut light, brazier) in light_query.iter_mut() {
        let flicker =
            (t * 7.0 + brazier.phase).sin() * 0.15 + (t * 19.0 + brazier.phase * 2.0).cos() * 0.08;
        light.intensity = brazier.base_intensity * (1.0 + flicker);
    }

    // 2. Pulse center rune medallion
    for mut rune_tf in rune_query.iter_mut() {
        let pulse = 1.0 + (t * 2.5).sin() * 0.03;
        rune_tf.scale = Vec3::new(pulse, 1.0, pulse);
    }

    // 3. Spawn rising embers from emitters
    for mut emitter in emitter_query.iter_mut() {
        emitter.timer.tick(std::time::Duration::from_secs_f32(dt));
        if emitter.timer.just_finished() {
            let vx = (t * 13.0).sin() * 0.45;
            let vy = 1.2 + (t * 5.0).cos().abs() * 0.8;
            let vz = (t * 17.0).cos() * 0.45;

            commands.spawn((
                Mesh3d(assets_3d.ember_particle.clone()),
                MeshMaterial3d(assets_3d.ember_mat.clone()),
                Transform::from_translation(emitter.pos).with_scale(Vec3::splat(0.8)),
                FloatingEmber {
                    velocity: Vec3::new(vx, vy, vz),
                    timer: Timer::from_seconds(1.1, TimerMode::Once),
                },
            ));
        }
    }

    // 4. Update floating embers
    for (entity, mut transform, mut ember) in ember_query.iter_mut() {
        ember.timer.tick(std::time::Duration::from_secs_f32(dt));
        transform.translation += ember.velocity * dt;
        let scale = (1.0 - ember.timer.fraction()).max(0.01);
        transform.scale = Vec3::splat(scale * 0.8);

        if ember.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_cursor_hover(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut hovered: ResMut<HoveredTile>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        hovered.tile = None;
        return;
    };

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        hovered.tile = None;
        return;
    };

    let plane_y = TILE_HEIGHT * 0.5;
    if ray.direction.y.abs() < 1e-5 {
        hovered.tile = None;
        return;
    }

    let t = (plane_y - ray.origin.y) / ray.direction.y;
    if t < 0.0 {
        hovered.tile = None;
        return;
    }

    let hit_world = ray.origin + ray.direction * t;

    let half = TILE_SIZE * 0.5 + 0.06;
    let mut found = None;

    for &faction in &[Faction::Player, Faction::Enemy] {
        for col in 0..GRID_COLS {
            for row in 0..GRID_ROWS {
                let center = grid_to_world_pos(col, row, faction);
                if (hit_world.x - center.x).abs() <= half && (hit_world.z - center.z).abs() <= half
                {
                    found = Some(GridPos { col, row, faction });
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        if found.is_some() {
            break;
        }
    }

    hovered.tile = found;
}

pub fn update_tile_visuals(
    hovered: Res<HoveredTile>,
    assets_3d: Res<Game3dAssets>,
    mut tiles: Query<(
        &TileEntity,
        &mut MeshMaterial3d<StandardMaterial>,
        &mut Transform,
    )>,
) {
    for (tile, mut mat, mut transform) in tiles.iter_mut() {
        let is_hovered = hovered.tile.as_ref().map_or(false, |h| {
            h.col == tile.col && h.row == tile.row && h.faction == tile.faction
        });

        let base_y = TILE_HEIGHT * 0.5 + 0.04;

        if is_hovered {
            transform.translation.y = base_y + 0.06;
            mat.0 = match tile.faction {
                Faction::Player => assets_3d.tile_player_hover.clone(),
                Faction::Enemy => assets_3d.tile_enemy_hover.clone(),
            };
        } else {
            transform.translation.y = base_y;
            mat.0 = match tile.faction {
                Faction::Player => assets_3d.tile_player.clone(),
                Faction::Enemy => assets_3d.tile_enemy.clone(),
            };
        }
    }
}
