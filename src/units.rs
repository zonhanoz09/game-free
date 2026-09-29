use crate::assets_3d::Game3dAssets;
use crate::battle::{ActionGauge, HitStopManager};
use crate::board::grid_to_world_pos;
use crate::types::*;
use crate::model_loader::{CustomGltfModelRoot, GltfModelAssets};
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
pub struct StaminaBarFill3d;

#[derive(Component)]
pub struct HealthBarRoot3d;

#[derive(Component)]
pub struct UnitVisualRoot;

#[derive(Component)]
pub struct IdleBobbing {
    pub base_y: f32,
    pub phase: f32,
}

#[derive(Component, Clone)]
pub struct ChibiSquashStretch {
    pub current_scale: Vec3,
    pub target_scale: Vec3,
    pub recovery_speed: f32,
}

impl Default for ChibiSquashStretch {
    fn default() -> Self {
        Self {
            current_scale: Vec3::ONE,
            target_scale: Vec3::ONE,
            recovery_speed: 11.0,
        }
    }
}

#[derive(Component)]
pub struct SpinningItem {
    pub speed: f32,
}

#[derive(Component)]
pub struct OrbitingMote {
    pub radius: f32,
    pub speed: f32,
    pub phase: f32,
    pub base_pos: Vec3,
}

pub fn spawn_unit(
    commands: &mut Commands,
    assets_3d: &Game3dAssets,
    gltf_assets: &GltfModelAssets,
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

    let (base_mat, ring_mat, cape_mat, badge_mat) = match faction {
        Faction::Player => (
            assets_3d.player_base.clone(),
            assets_3d.player_ring.clone(),
            assets_3d.player_cape.clone(),
            assets_3d.badge_player_mat.clone(),
        ),
        Faction::Enemy => (
            assets_3d.enemy_base.clone(),
            assets_3d.enemy_ring.clone(),
            assets_3d.enemy_cape.clone(),
            assets_3d.badge_enemy_mat.clone(),
        ),
    };

    let phase = (col as f32 * 1.3)
        + (row as f32 * 0.7)
        + (match faction {
            Faction::Player => 0.0,
            Faction::Enemy => 2.0,
        });

    // Spawn Unit Root with full Visibility Hierarchy (ensuring all children render properly)
    let mut unit_cmd = commands.spawn((
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
        ChibiSquashStretch::default(),
        Transform::from_translation(world_pos).with_rotation(rotation),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));

    if gltf_assets.has_model(unit_class) {
        unit_cmd.insert(CustomGltfModelRoot { class: unit_class });
    }

    unit_cmd
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

            // --- 2. Class Specific 3D Model Anatomy & Gear ---
            if gltf_assets.has_model(unit_class) {
                // High-fidelity rigged 3D glTF model (VRoid / Quaternius / Mixamo / Kenney)
                parent.spawn((
                    SceneRoot(gltf_assets.models[&unit_class].clone()),
                    Transform::from_xyz(0.0, 0.05, 0.0).with_scale(Vec3::splat(1.0)),
                ));
            } else {
                match unit_class {
                UnitClass::Knight => {
                    // Armored Greaves / Legs
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
                            .with_scale(Vec3::new(0.32, 0.08, 0.04)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(0.0, 0.88, 0.21)
                            .with_scale(Vec3::new(0.08, 0.32, 0.04)),
                    ));

                    // Shoulders - Pauldrons
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.knight_armor.clone()),
                        Transform::from_xyz(-0.42, 1.10, 0.0).with_scale(Vec3::splat(0.26)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.knight_armor.clone()),
                        Transform::from_xyz(0.42, 1.10, 0.0).with_scale(Vec3::splat(0.26)),
                    ));

                    // Head - Knight Greathelm with Eye Visor Slit
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.knight_armor.clone()),
                        Transform::from_xyz(0.0, 1.34, 0.0).with_scale(Vec3::new(0.42, 0.42, 0.38)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.hp_bg_mat.clone()),
                        Transform::from_xyz(0.0, 1.35, 0.20)
                            .with_scale(Vec3::new(0.28, 0.06, 0.04)),
                    ));

                    // Red Knight Feather Plume on Helmet Crest
                    parent.spawn((
                        Mesh3d(assets_3d.cone.clone()),
                        MeshMaterial3d(assets_3d.knight_plume.clone()),
                        Transform::from_xyz(0.0, 1.65, -0.05)
                            .with_rotation(Quat::from_rotation_x(-0.35))
                            .with_scale(Vec3::new(0.18, 0.42, 0.18)),
                    ));

                    // Left Hand - Royal Tower Kite Shield
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.shield_front.clone()),
                        Transform::from_xyz(-0.46, 0.82, 0.22)
                            .with_scale(Vec3::new(0.12, 0.72, 0.48)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(-0.47, 0.82, 0.22)
                            .with_scale(Vec3::new(0.14, 0.74, 0.12)),
                    ));

                    // Right Hand - Polished Steel Longsword
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(0.46, 0.60, 0.20)
                            .with_scale(Vec3::new(0.04, 0.26, 0.04)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(0.46, 0.74, 0.20)
                            .with_scale(Vec3::new(0.24, 0.04, 0.06)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.steel_blade.clone()),
                        Transform::from_xyz(0.46, 1.25, 0.20)
                            .with_scale(Vec3::new(0.06, 0.98, 0.02)),
                    ));

                    // Back Cape
                    parent.spawn((
                        Mesh3d(assets_3d.cape.clone()),
                        MeshMaterial3d(cape_mat),
                        Transform::from_xyz(0.0, 0.75, -0.22)
                            .with_rotation(Quat::from_rotation_x(0.18)),
                    ));
                }

                UnitClass::Archer => {
                    // Leather Boots / Legs
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.archer_leather.clone()),
                        Transform::from_xyz(-0.15, 0.22, 0.0)
                            .with_scale(Vec3::new(0.20, 0.44, 0.20)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.archer_leather.clone()),
                        Transform::from_xyz(0.15, 0.22, 0.0)
                            .with_scale(Vec3::new(0.20, 0.44, 0.20)),
                    ));

                    // Forest Green Tunic Body
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.archer_tunic.clone()),
                        Transform::from_xyz(0.0, 0.78, 0.0).with_scale(Vec3::new(0.50, 0.65, 0.34)),
                    ));

                    // Leather Chest Cross-Strap
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.archer_leather.clone()),
                        Transform::from_xyz(0.0, 0.82, 0.18)
                            .with_rotation(Quat::from_rotation_z(0.65))
                            .with_scale(Vec3::new(0.10, 0.65, 0.03)),
                    ));

                    // Quiver on the Back with Arrows
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.archer_leather.clone()),
                        Transform::from_xyz(0.18, 0.95, -0.22)
                            .with_rotation(Quat::from_rotation_z(-0.35))
                            .with_scale(Vec3::new(0.10, 0.60, 0.10)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.arrow_glow.clone()),
                        Transform::from_xyz(0.24, 1.25, -0.22)
                            .with_rotation(Quat::from_rotation_z(-0.35))
                            .with_scale(Vec3::new(0.03, 0.35, 0.03)),
                    ));

                    // Head & Forest Archer Robin Hood Hat with Green Plume Feather
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.skin.clone()),
                        Transform::from_xyz(0.0, 1.25, 0.0).with_scale(Vec3::splat(0.28)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cone.clone()),
                        MeshMaterial3d(assets_3d.archer_tunic.clone()),
                        Transform::from_xyz(0.0, 1.44, -0.06)
                            .with_rotation(Quat::from_rotation_x(-0.45))
                            .with_scale(Vec3::new(0.36, 0.35, 0.36)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cone.clone()),
                        MeshMaterial3d(assets_3d.archer_feather.clone()),
                        Transform::from_xyz(-0.16, 1.55, -0.05)
                            .with_rotation(Quat::from_rotation_z(0.65))
                            .with_scale(Vec3::new(0.08, 0.40, 0.08)),
                    ));

                    // Left Hand - Longbow Frame with Bowstring
                    parent.spawn((
                        Mesh3d(assets_3d.torus_ring.clone()),
                        MeshMaterial3d(assets_3d.archer_wood.clone()),
                        Transform::from_xyz(-0.44, 0.85, 0.16)
                            .with_rotation(Quat::from_rotation_y(FRAC_PI_2))
                            .with_scale(Vec3::new(0.45, 1.2, 0.45)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.arrow_glow.clone()),
                        Transform::from_xyz(-0.44, 0.85, 0.16)
                            .with_scale(Vec3::new(0.015, 1.15, 0.015)),
                    ));
                }

                UnitClass::Mage => {
                    // Long Arcane Robe (Conical gown + Torso)
                    parent.spawn((
                        Mesh3d(assets_3d.cone.clone()),
                        MeshMaterial3d(assets_3d.mage_robe.clone()),
                        Transform::from_xyz(0.0, 0.42, 0.0).with_scale(Vec3::new(0.55, 0.80, 0.55)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.mage_robe.clone()),
                        Transform::from_xyz(0.0, 0.85, 0.0).with_scale(Vec3::new(0.48, 0.52, 0.34)),
                    ));

                    // Head & Classic Wizard Pointy Hat with Wide Brim
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.skin.clone()),
                        Transform::from_xyz(0.0, 1.22, 0.0).with_scale(Vec3::splat(0.28)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.mage_hat.clone()),
                        Transform::from_xyz(0.0, 1.35, 0.0).with_scale(Vec3::new(0.55, 0.04, 0.55)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cone.clone()),
                        MeshMaterial3d(assets_3d.mage_hat.clone()),
                        Transform::from_xyz(0.0, 1.68, -0.06)
                            .with_rotation(Quat::from_rotation_x(-0.20))
                            .with_scale(Vec3::new(0.34, 0.65, 0.34)),
                    ));

                    // Right Hand - Ancient Gnarled Oak Staff
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.mage_wood.clone()),
                        Transform::from_xyz(0.45, 0.85, 0.18)
                            .with_scale(Vec3::new(0.04, 1.6, 0.04)),
                    ));

                    // Floating Pulsing Arcane Crystal Orb with Soft Point Light
                    let staff_orb_pos = Vec3::new(0.45, 1.72, 0.18);
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.mage_crystal.clone()),
                        Transform::from_translation(staff_orb_pos).with_scale(Vec3::splat(0.18)),
                        SpinningItem { speed: 3.0 },
                    ));
                    parent.spawn((
                        PointLight {
                            color: Color::srgb(0.75, 0.3, 1.0),
                            intensity: 2200.0,
                            range: 4.0,
                            shadows_enabled: false,
                            ..default()
                        },
                        Transform::from_translation(staff_orb_pos + Vec3::new(0.0, 0.1, 0.0)),
                    ));

                    // 2 Orbiting Magical Satellite Motes
                    parent.spawn((
                        Mesh3d(assets_3d.small_sphere.clone()),
                        MeshMaterial3d(assets_3d.lightning.clone()),
                        Transform::from_translation(staff_orb_pos + Vec3::new(0.24, 0.0, 0.0)),
                        OrbitingMote {
                            radius: 0.26,
                            speed: 4.0,
                            phase: 0.0,
                            base_pos: staff_orb_pos,
                        },
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.small_sphere.clone()),
                        MeshMaterial3d(assets_3d.lightning.clone()),
                        Transform::from_translation(staff_orb_pos + Vec3::new(-0.24, 0.0, 0.0)),
                        OrbitingMote {
                            radius: 0.26,
                            speed: 4.0,
                            phase: 3.1415,
                            base_pos: staff_orb_pos,
                        },
                    ));
                }

                UnitClass::Assassin => {
                    // Sleek Dark Shinobi Suit (Legs & Torso)
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.assassin_suit.clone()),
                        Transform::from_xyz(-0.14, 0.22, 0.0)
                            .with_scale(Vec3::new(0.18, 0.44, 0.18)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.assassin_suit.clone()),
                        Transform::from_xyz(0.14, 0.22, 0.0)
                            .with_scale(Vec3::new(0.18, 0.44, 0.18)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.assassin_suit.clone()),
                        Transform::from_xyz(0.0, 0.74, 0.0).with_scale(Vec3::new(0.44, 0.58, 0.28)),
                    ));

                    // Crimson Trailing Scarf
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.assassin_scarf.clone()),
                        Transform::from_xyz(0.0, 1.02, 0.12)
                            .with_scale(Vec3::new(0.42, 0.10, 0.20)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cape.clone()),
                        MeshMaterial3d(assets_3d.assassin_scarf.clone()),
                        Transform::from_xyz(-0.15, 0.72, -0.22)
                            .with_rotation(Quat::from_rotation_x(0.35))
                            .with_scale(Vec3::new(0.4, 0.8, 0.5)),
                    ));

                    // Cowled Shadow Hood
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

                    // Dual Reverse-Grip Poison Daggers with Crossguards
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(-0.42, 0.72, 0.20)
                            .with_scale(Vec3::new(0.16, 0.04, 0.06)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.poison_blade.clone()),
                        Transform::from_xyz(-0.42, 0.50, 0.20)
                            .with_rotation(Quat::from_rotation_x(-0.35))
                            .with_scale(Vec3::new(0.04, 0.46, 0.08)),
                    ));

                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.gold_trim.clone()),
                        Transform::from_xyz(0.42, 0.72, 0.20)
                            .with_scale(Vec3::new(0.16, 0.04, 0.06)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.poison_blade.clone()),
                        Transform::from_xyz(0.42, 0.50, 0.20)
                            .with_rotation(Quat::from_rotation_x(-0.35))
                            .with_scale(Vec3::new(0.04, 0.46, 0.08)),
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

                    // Golden Stole Sash across Chest
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

                    // Floating Glowing Holy Halo with Soft Spin
                    parent.spawn((
                        Mesh3d(assets_3d.torus_halo.clone()),
                        MeshMaterial3d(assets_3d.cleric_halo.clone()),
                        Transform::from_xyz(0.0, 1.68, 0.0)
                            .with_rotation(Quat::from_rotation_x(0.2)),
                        SpinningItem { speed: 1.5 },
                    ));

                    // Right Hand - Celestial Holy Scepter with Sun Disc & Warm Light
                    let scepter_top = Vec3::new(0.44, 1.58, 0.20);
                    parent.spawn((
                        Mesh3d(assets_3d.cylinder.clone()),
                        MeshMaterial3d(assets_3d.cleric_scepter.clone()),
                        Transform::from_xyz(0.44, 0.85, 0.20)
                            .with_scale(Vec3::new(0.035, 1.4, 0.035)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.sphere.clone()),
                        MeshMaterial3d(assets_3d.heal_glow.clone()),
                        Transform::from_translation(scepter_top).with_scale(Vec3::splat(0.16)),
                    ));
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.cleric_gold.clone()),
                        Transform::from_translation(scepter_top)
                            .with_scale(Vec3::new(0.30, 0.06, 0.04)),
                    ));
                    parent.spawn((
                        PointLight {
                            color: Color::srgb(1.0, 0.9, 0.4),
                            intensity: 2200.0,
                            range: 4.0,
                            shadows_enabled: false,
                            ..default()
                        },
                        Transform::from_translation(scepter_top + Vec3::new(0.0, 0.1, 0.0)),
                    ));

                    // Left Hand - Sacred Prayer Scripture Tome
                    parent.spawn((
                        Mesh3d(assets_3d.cube.clone()),
                        MeshMaterial3d(assets_3d.cleric_gold.clone()),
                        Transform::from_xyz(-0.38, 0.85, 0.20)
                            .with_scale(Vec3::new(0.12, 0.22, 0.16)),
                    ));
                }
            }
        }

            // --- 3. Enhanced 3D Overhead Floating Dual Health & Stamina HUD ---
            parent
                .spawn((
                    HealthBarRoot3d,
                    Transform::from_xyz(0.0, 2.36, 0.0).with_rotation(Quat::from_rotation_x(0.55)),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .with_children(|bar_parent| {
                    // Dark Metallic Frame Enclosure
                    bar_parent.spawn((
                        Mesh3d(assets_3d.hp_frame.clone()),
                        MeshMaterial3d(assets_3d.hp_frame_mat.clone()),
                        Transform::from_xyz(0.0, 0.0, -0.015),
                    ));

                    // Faction Emblem Sphere on left of bar
                    bar_parent.spawn((
                        Mesh3d(assets_3d.class_badge_mesh.clone()),
                        MeshMaterial3d(badge_mat),
                        Transform::from_xyz(-0.68, 0.02, 0.02),
                    ));

                    // Health Bar Background Plate (Top)
                    bar_parent.spawn((
                        Mesh3d(assets_3d.hp_bg.clone()),
                        MeshMaterial3d(assets_3d.hp_bg_mat.clone()),
                        Transform::from_xyz(0.05, 0.045, -0.005),
                    ));

                    // Dynamic Health Fill Bar
                    bar_parent.spawn((
                        HealthBarFill3d,
                        Mesh3d(assets_3d.hp_fill.clone()),
                        MeshMaterial3d(assets_3d.hp_green.clone()),
                        Transform::from_xyz(0.05, 0.045, 0.01),
                    ));

                    // Stamina / Action Gauge Background Plate (Bottom)
                    bar_parent.spawn((
                        Mesh3d(assets_3d.stamina_bg.clone()),
                        MeshMaterial3d(assets_3d.stamina_bg_mat.clone()),
                        Transform::from_xyz(0.05, -0.045, -0.005),
                    ));

                    // Dynamic Stamina / Action Gauge Fill Bar (Cyan Energy)
                    bar_parent.spawn((
                        StaminaBarFill3d,
                        Mesh3d(assets_3d.stamina_fill.clone()),
                        MeshMaterial3d(assets_3d.stamina_mat.clone()),
                        Transform::from_xyz(0.05, -0.045, 0.01),
                    ));
                });
        })
        .id()
}

pub fn animate_idle_bobbing(
    time: Res<Time>,
    speed: Res<BattleSpeed>,
    hit_stop: Res<HitStopManager>,
    mut bob_query: Query<
        (&mut Transform, &IdleBobbing, &mut ChibiSquashStretch),
        (With<UnitVisualRoot>, Without<DeadUnit>),
    >,
) {
    if hit_stop.active {
        return;
    }
    let dt = time.delta_secs() * speed.multiplier;
    let t = time.elapsed_secs() * speed.multiplier;
    for (mut transform, bob, mut squash) in bob_query.iter_mut() {
        let offset = (t * 3.2 + bob.phase).sin() * 0.025;
        transform.translation.y = bob.base_y + offset;

        // Smooth anime chibi squash and stretch relaxation
        squash.target_scale = squash.target_scale.lerp(Vec3::ONE, (dt * 8.0).min(1.0));
        squash.current_scale = squash
            .current_scale
            .lerp(squash.target_scale, (dt * squash.recovery_speed).min(1.0));
        transform.scale = squash.current_scale;
    }
}

pub fn animate_spinning_items(
    time: Res<Time>,
    hit_stop: Res<HitStopManager>,
    mut spin_query: Query<(&mut Transform, &SpinningItem), Without<UnitVisualRoot>>,
) {
    if hit_stop.active {
        return;
    }
    let dt = time.delta_secs();
    for (mut transform, spin) in spin_query.iter_mut() {
        transform.rotate_y(spin.speed * dt);
    }
}

pub fn animate_orbiting_motes(
    time: Res<Time>,
    hit_stop: Res<HitStopManager>,
    mut mote_query: Query<(&mut Transform, &OrbitingMote), Without<UnitVisualRoot>>,
) {
    if hit_stop.active {
        return;
    }
    let t = time.elapsed_secs();
    for (mut transform, mote) in mote_query.iter_mut() {
        let angle = t * mote.speed + mote.phase;
        let x = angle.cos() * mote.radius;
        let z = angle.sin() * mote.radius;
        transform.translation = mote.base_pos + Vec3::new(x, 0.0, z);
    }
}

pub fn update_unit_health_bars(
    units: Query<(&UnitStats, &Children, Option<&ActionGauge>), (With<Unit>, Without<DeadUnit>)>,
    assets_3d: Res<Game3dAssets>,
    mut hp_fill_query: Query<
        (&mut Transform, &mut MeshMaterial3d<StandardMaterial>),
        (With<HealthBarFill3d>, Without<StaminaBarFill3d>),
    >,
    mut stamina_fill_query: Query<
        &mut Transform,
        (With<StaminaBarFill3d>, Without<HealthBarFill3d>),
    >,
    roots: Query<&Children, With<HealthBarRoot3d>>,
) {
    for (stats, unit_children, maybe_gauge) in units.iter() {
        let hp_ratio = (stats.hp / stats.max_hp).clamp(0.0, 1.0);
        let stamina_ratio = maybe_gauge.map_or(0.0, |g| (g.current / 100.0).clamp(0.0, 1.0));

        for child in unit_children.iter() {
            if let Ok(bar_children) = roots.get(*child) {
                for bar_child in bar_children.iter() {
                    // Update HP Fill Bar
                    if let Ok((mut transform, mut mat)) = hp_fill_query.get_mut(*bar_child) {
                        transform.scale.x = hp_ratio;
                        transform.translation.x = 0.05 - (1.0 - hp_ratio) * 0.5 * 1.16;

                        if hp_ratio > 0.55 {
                            mat.0 = assets_3d.hp_green.clone();
                        } else if hp_ratio > 0.25 {
                            mat.0 = assets_3d.hp_yellow.clone();
                        } else {
                            mat.0 = assets_3d.hp_red.clone();
                        }
                    }

                    // Update Stamina / Action Gauge Bar
                    if let Ok(mut transform) = stamina_fill_query.get_mut(*bar_child) {
                        transform.scale.x = stamina_ratio;
                        transform.translation.x = 0.05 - (1.0 - stamina_ratio) * 0.5 * 1.16;
                    }
                }
            }
        }
    }
}
