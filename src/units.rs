use crate::assets_3d::Game3dAssets;
use crate::board::grid_to_world_pos;
use crate::types::*;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

#[derive(Component)]
pub struct Unit {
    pub class: UnitClass,
    pub faction: Faction,
}

#[derive(Component)]
pub struct HealthBarFill3d;

#[derive(Component)]
pub struct HealthBarRoot3d;

#[derive(Component)]
pub struct UnitVisualRoot;

#[derive(Component)]
pub struct IdleBobbing {
    pub base_y: f32,
    pub phase: f32,
}

#[derive(Component)]
pub struct SpinningItem {
    pub speed: f32,
}

pub fn spawn_unit(
    commands: &mut Commands,
    assets_3d: &Game3dAssets,
    unit_class: UnitClass,
    faction: Faction,
    col: usize,
    row: usize,
) -> Entity {
    let stats = unit_class.base_stats();
    let world_pos = grid_to_world_pos(col, row, faction);

    // Orientation: Player units face towards +X (enemy side), Enemy units face towards -X (player side)
    let rotation = match faction {
        Faction::Player => Quat::from_rotation_y(FRAC_PI_2),
        Faction::Enemy => Quat::from_rotation_y(-FRAC_PI_2),
    };

    let (base_mat, ring_mat) = match faction {
        Faction::Player => (assets_3d.player_base.clone(), assets_3d.player_ring.clone()),
        Faction::Enemy => (assets_3d.enemy_base.clone(), assets_3d.enemy_ring.clone()),
    };

    let phase = (col as f32 * 1.3)
        + (row as f32 * 0.7)
        + (match faction {
            Faction::Player => 0.0,
            Faction::Enemy => 2.0,
        });

    // Spawn Unit Root
    commands
        .spawn((
            Unit {
                class: unit_class,
                faction,
            },
            stats,
            GridPos { col, row, faction },
            UnitVisualRoot,
            IdleBobbing {
                base_y: world_pos.y,
                phase,
            },
            Transform::from_translation(world_pos).with_rotation(rotation),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // --- 1. Base Pedestal & Glowing Faction Rune Ring ---
            parent.spawn((
                Mesh3d(assets_3d.pedestal.clone()),
                MeshMaterial3d(base_mat),
                Transform::from_xyz(0.0, 0.04, 0.0),
            ));

            parent.spawn((
                Mesh3d(assets_3d.torus_ring.clone()),
                MeshMaterial3d(ring_mat),
                Transform::from_xyz(0.0, 0.05, 0.0),
            ));

            // --- 2. Class Specific 3D Model Anatomy ---
            match unit_class {
                UnitClass::Knight => {
                    // Armored Legs
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.knight_armor.clone()),
                        Transform::from_xyz(-0.16, 0.24, 0.0)
                            .with_scale(Vec3::new(0.24, 0.44, 0.24)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.knight_armor.clone()),
                        Transform::from_xyz(0.16, 0.24, 0.0)
                            .with_scale(Vec3::new(0.24, 0.44, 0.24)),
                    ));

                    // Gold Waist Trim Belt
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(0.0, 0.50, 0.0).with_scale(Vec3::new(0.58, 0.10, 0.38)),
                    ));

                    // Torso - Heavy Steel Plate Cuirass
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.knight_armor.clone()),
                        Transform::from_xyz(0.0, 0.85, 0.0).with_scale(Vec3::new(0.64, 0.62, 0.40)),
                    ));

                    // Golden Heraldic Cross on Chest
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(0.0, 0.88, 0.21)
                            .with_scale(Vec3::new(0.36, 0.10, 0.04)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(0.0, 0.88, 0.21)
                            .with_scale(Vec3::new(0.10, 0.42, 0.04)),
                    ));

                    // Left & Right Pauldrons (Shoulder Armor)
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.knight_armor.clone()),
                        Transform::from_xyz(-0.40, 1.10, 0.0)
                            .with_scale(Vec3::new(0.24, 0.22, 0.32)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.knight_armor.clone()),
                        Transform::from_xyz(0.40, 1.10, 0.0)
                            .with_scale(Vec3::new(0.24, 0.22, 0.32)),
                    ));

                    // Knight Greathelm
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.knight_armor.clone()),
                        Transform::from_xyz(0.0, 1.38, 0.0).with_scale(Vec3::new(0.40, 0.42, 0.40)),
                    ));

                    // Golden Visor Slit
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(0.0, 1.40, 0.21)
                            .with_scale(Vec3::new(0.28, 0.08, 0.04)),
                    ));

                    // Helmet Plume / Feather Crest
                    parent.spawn((
                        Mesh3d(assets_3d.capsule.clone()),
                        MeshMaterial3d(assets_3d.knight_plume.clone()),
                        Transform::from_xyz(0.0, 1.72, -0.05)
                            .with_scale(Vec3::new(0.18, 0.32, 0.18)),
                    ));

                    // Left Hand - Large Tower Shield
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.shield_front.clone()),
                        Transform::from_xyz(-0.46, 0.80, 0.22)
                            .with_scale(Vec3::new(0.08, 0.82, 0.52)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(-0.45, 0.80, 0.22)
                            .with_scale(Vec3::new(0.09, 0.86, 0.08)),
                    ));

                    // Right Hand - Steel Longsword
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(0.46, 0.72, 0.20)
                            .with_scale(Vec3::new(0.32, 0.06, 0.06)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.steel_blade.clone()),
                        Transform::from_xyz(0.46, 1.20, 0.20)
                            .with_scale(Vec3::new(0.07, 0.90, 0.03)),
                    ));
                }

                UnitClass::Archer => {
                    // Boots / Legs
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.archer_leather.clone()),
                        Transform::from_xyz(-0.14, 0.22, 0.0)
                            .with_scale(Vec3::new(0.20, 0.40, 0.20)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.archer_leather.clone()),
                        Transform::from_xyz(0.14, 0.22, 0.0)
                            .with_scale(Vec3::new(0.20, 0.40, 0.20)),
                    ));

                    // Forest Green Ranger Tunic
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.archer_tunic.clone()),
                        Transform::from_xyz(0.0, 0.78, 0.0).with_scale(Vec3::new(0.48, 0.60, 0.32)),
                    ));

                    // Leather Belt
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.archer_leather.clone()),
                        Transform::from_xyz(0.0, 0.52, 0.0).with_scale(Vec3::new(0.50, 0.10, 0.34)),
                    ));

                    // Hooded Head
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.archer_tunic.clone()),
                        Transform::from_xyz(0.0, 1.28, 0.0).with_scale(Vec3::splat(0.28)),
                    ));

                    // Ranger Cap Feather
                    parent.spawn((
                        Mesh3d(assets_3d.cone.clone()),
                        MeshMaterial3d(assets_3d.archer_feather.clone()),
                        Transform::from_xyz(0.12, 1.50, -0.10)
                            .with_rotation(Quat::from_rotation_z(-0.3))
                            .with_scale(Vec3::new(0.08, 0.35, 0.08)),
                    ));

                    // Back Quiver
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.archer_leather.clone()),
                        Transform::from_xyz(0.15, 0.95, -0.20)
                            .with_rotation(Quat::from_rotation_z(0.25))
                            .with_scale(Vec3::new(0.12, 0.62, 0.12)),
                    ));
                    // Glowing Arrow Fletchings in quiver
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.arrow_glow.clone()),
                        Transform::from_xyz(0.22, 1.35, -0.22).with_scale(Vec3::splat(0.08)),
                    ));

                    // Left Hand - Longbow
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.archer_wood.clone()),
                        Transform::from_xyz(-0.42, 0.85, 0.25)
                            .with_scale(Vec3::new(0.05, 0.95, 0.05)),
                    ));
                    // Nocked Arrow ready to fire
                    parent.spawn((
                        Mesh3d(assets_3d.arrow_shaft.clone()),
                        MeshMaterial3d(assets_3d.archer_wood.clone()),
                        Transform::from_xyz(-0.15, 0.85, 0.25)
                            .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.arrow_head.clone()),
                        MeshMaterial3d(assets_3d.arrow_glow.clone()),
                        Transform::from_xyz(-0.15, 0.85, 0.58)
                            .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                    ));
                }

                UnitClass::Mage => {
                    // Lower Flowing Mystic Robe
                    parent.spawn((
                        Mesh3d(assets_3d.cone.clone()),
                        MeshMaterial3d(assets_3d.mage_robe.clone()),
                        Transform::from_xyz(0.0, 0.45, 0.0).with_scale(Vec3::new(0.55, 0.85, 0.55)),
                    ));

                    // Upper Robe Torso
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.mage_robe.clone()),
                        Transform::from_xyz(0.0, 0.92, 0.0).with_scale(Vec3::new(0.48, 0.52, 0.32)),
                    ));

                    // Wizard Head
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.skin.clone()),
                        Transform::from_xyz(0.0, 1.25, 0.0).with_scale(Vec3::splat(0.22)),
                    ));

                    // Wizard Hat Brim
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.mage_hat.clone()),
                        Transform::from_xyz(0.0, 1.38, 0.0).with_scale(Vec3::new(0.52, 0.04, 0.52)),
                    ));

                    // Wizard Hat Pointed Cone
                    parent.spawn((
                        Mesh3d(assets_3d.cone.clone()),
                        MeshMaterial3d(assets_3d.mage_hat.clone()),
                        Transform::from_xyz(0.0, 1.74, 0.0).with_scale(Vec3::new(0.34, 0.72, 0.34)),
                    ));

                    // Gold Hat Band
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(0.0, 1.42, 0.0).with_scale(Vec3::new(0.36, 0.05, 0.36)),
                    ));

                    // Right Hand - Ancient Wizard Staff
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.mage_wood.clone()),
                        Transform::from_xyz(0.45, 0.85, 0.18)
                            .with_scale(Vec3::new(0.04, 1.6, 0.04)),
                    ));

                    // Floating Pulsing Arcane Crystal Orb
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.mage_crystal.clone()),
                        Transform::from_xyz(0.45, 1.72, 0.18).with_scale(Vec3::splat(0.18)),
                        SpinningItem { speed: 3.0 },
                    ));
                }

                UnitClass::Assassin => {
                    // Crouched Stealth Stance Legs
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.assassin_suit.clone()),
                        Transform::from_xyz(-0.16, 0.20, -0.05)
                            .with_scale(Vec3::new(0.20, 0.38, 0.20)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.assassin_suit.clone()),
                        Transform::from_xyz(0.16, 0.20, -0.05)
                            .with_scale(Vec3::new(0.20, 0.38, 0.20)),
                    ));

                    // Stealth Midnight Torso
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.assassin_suit.clone()),
                        Transform::from_xyz(0.0, 0.72, 0.0).with_scale(Vec3::new(0.46, 0.52, 0.28)),
                    ));

                    // Flowing Blood-Crimson Ninja Scarf
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.assassin_scarf.clone()),
                        Transform::from_xyz(0.0, 0.98, 0.0).with_scale(Vec3::new(0.48, 0.12, 0.30)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.assassin_scarf.clone()),
                        Transform::from_xyz(-0.12, 0.85, -0.28)
                            .with_rotation(Quat::from_rotation_x(0.4))
                            .with_scale(Vec3::new(0.14, 0.55, 0.06)),
                    ));

                    // Ninja Masked Head
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.assassin_suit.clone()),
                        Transform::from_xyz(0.0, 1.18, 0.0).with_scale(Vec3::new(0.34, 0.36, 0.34)),
                    ));

                    // Glowing Sinister Eye Slits
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.assassin_eyes.clone()),
                        Transform::from_xyz(-0.08, 1.20, 0.18).with_scale(Vec3::splat(0.05)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.assassin_eyes.clone()),
                        Transform::from_xyz(0.08, 1.20, 0.18).with_scale(Vec3::splat(0.05)),
                    ));

                    // Dual Reverse-Grip Poison Daggers
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.poison_blade.clone()),
                        Transform::from_xyz(-0.42, 0.62, 0.20)
                            .with_rotation(Quat::from_rotation_x(-0.4))
                            .with_scale(Vec3::new(0.04, 0.44, 0.08)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.poison_blade.clone()),
                        Transform::from_xyz(0.42, 0.62, 0.20)
                            .with_rotation(Quat::from_rotation_x(-0.4))
                            .with_scale(Vec3::new(0.04, 0.44, 0.08)),
                    ));
                }

                UnitClass::Cleric => {
                    // Lower Flowing Divine Ivory Gown
                    parent.spawn((
                        Mesh3d(assets_3d.cone.clone()),
                        MeshMaterial3d(assets_3d.cleric_robe.clone()),
                        Transform::from_xyz(0.0, 0.45, 0.0).with_scale(Vec3::new(0.55, 0.85, 0.55)),
                    ));

                    // Upper Pristine Vestments
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.cleric_robe.clone()),
                        Transform::from_xyz(0.0, 0.92, 0.0).with_scale(Vec3::new(0.48, 0.55, 0.32)),
                    ));

                    // Golden Stole Sash
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.cleric_gold.clone()),
                        Transform::from_xyz(0.0, 0.88, 0.17)
                            .with_scale(Vec3::new(0.24, 0.58, 0.04)),
                    ));

                    // Serene Priestess Head & Veil
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.cleric_robe.clone()),
                        Transform::from_xyz(0.0, 1.28, 0.0).with_scale(Vec3::splat(0.26)),
                    ));

                    // Floating Glowing Holy Halo
                    parent.spawn((
                        Mesh3d(assets_3d.torus_halo.clone()),
                        MeshMaterial3d(assets_3d.cleric_halo.clone()),
                        Transform::from_xyz(0.0, 1.68, 0.0)
                            .with_rotation(Quat::from_rotation_x(0.2)),
                        SpinningItem { speed: 1.5 },
                    ));

                    // Right Hand - Celestial Holy Scepter
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.cleric_scepter.clone()),
                        Transform::from_xyz(0.44, 0.85, 0.20)
                            .with_scale(Vec3::new(0.035, 1.4, 0.035)),
                    ));
                    // Sun Disc & Cross on top
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.heal_glow.clone()),
                        Transform::from_xyz(0.44, 1.58, 0.20).with_scale(Vec3::splat(0.16)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.cleric_gold.clone()),
                        Transform::from_xyz(0.44, 1.58, 0.20)
                            .with_scale(Vec3::new(0.30, 0.06, 0.04)),
                    ));
                }
            }

            // --- 3. 3D Overhead Floating Health Bar ---
            parent
                .spawn((
                    HealthBarRoot3d,
                    Transform::from_xyz(0.0, 2.30, 0.0).with_rotation(Quat::from_rotation_x(0.55)), // Angled facing the elevated camera
                ))
                .with_children(|bar_parent| {
                    // Dark Background Plate
                    bar_parent.spawn((
                        Mesh3d(assets_3d.hp_bg.clone()),
                        MeshMaterial3d(assets_3d.hp_bg_mat.clone()),
                        Transform::from_xyz(0.0, 0.0, -0.01),
                    ));

                    // Green/Yellow/Red Health Fill Bar
                    bar_parent.spawn((
                        HealthBarFill3d,
                        Mesh3d(assets_3d.hp_fill.clone()),
                        MeshMaterial3d(assets_3d.hp_green.clone()),
                        Transform::from_xyz(0.0, 0.0, 0.01),
                    ));
                });
        })
        .id()
}

pub fn animate_idle_bobbing(
    time: Res<Time>,
    mut bob_query: Query<(&mut Transform, &IdleBobbing), (With<UnitVisualRoot>, Without<DeadUnit>)>,
    mut spin_query: Query<(&mut Transform, &SpinningItem), Without<UnitVisualRoot>>,
) {
    let t = time.elapsed_secs();
    let dt = time.delta_secs();

    // Bobbing breath
    for (mut transform, bob) in bob_query.iter_mut() {
        let offset = (t * 3.2 + bob.phase).sin() * 0.025;
        transform.translation.y = bob.base_y + offset;
    }

    // Spinning halos & arcane crystals
    for (mut transform, spin) in spin_query.iter_mut() {
        transform.rotate_y(spin.speed * dt);
    }
}

pub fn update_unit_health_bars(
    units: Query<(&UnitStats, &Children), Changed<UnitStats>>,
    assets_3d: Res<Game3dAssets>,
    mut fill_query: Query<
        (&mut Transform, &mut MeshMaterial3d<StandardMaterial>),
        With<HealthBarFill3d>,
    >,
    roots: Query<&Children, With<HealthBarRoot3d>>,
) {
    for (stats, unit_children) in units.iter() {
        let hp_ratio = (stats.hp / stats.max_hp).clamp(0.0, 1.0);

        for child in unit_children.iter() {
            if let Ok(bar_children) = roots.get(*child) {
                for bar_child in bar_children.iter() {
                    if let Ok((mut transform, mut mat)) = fill_query.get_mut(*bar_child) {
                        transform.scale.x = hp_ratio;
                        transform.translation.x = -(1.0 - hp_ratio) * 0.5 * 1.16;

                        if hp_ratio > 0.55 {
                            mat.0 = assets_3d.hp_green.clone();
                        } else if hp_ratio > 0.25 {
                            mat.0 = assets_3d.hp_yellow.clone();
                        } else {
                            mat.0 = assets_3d.hp_red.clone();
                        }
                    }
                }
            }
        }
    }
}
