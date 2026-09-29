use crate::types::*;
use bevy::prelude::*;

#[derive(Resource)]
#[allow(dead_code)]
pub struct Game3dAssets {
    // Shared primitive meshes
    pub cube: Handle<Mesh>,
    pub cylinder: Handle<Mesh>,
    pub sphere: Handle<Mesh>,
    pub cone: Handle<Mesh>,
    pub capsule: Handle<Mesh>,
    pub torus_ring: Handle<Mesh>,
    pub torus_halo: Handle<Mesh>,

    // Environment & Tile Meshes
    pub tile_slab: Handle<Mesh>,
    pub tile_border: Handle<Mesh>,
    pub pedestal: Handle<Mesh>,
    pub arena_floor: Handle<Mesh>,
    pub arena_rim: Handle<Mesh>,
    pub arena_pillar: Handle<Mesh>,
    pub arena_brazier: Handle<Mesh>,
    pub divider: Handle<Mesh>,
    pub banner_pole: Handle<Mesh>,
    pub banner_cloth: Handle<Mesh>,
    pub rune_circle: Handle<Mesh>,
    pub rune_disc: Handle<Mesh>,
    pub ember_particle: Handle<Mesh>,

    // Character Detail Meshes
    pub cape: Handle<Mesh>,
    pub small_sphere: Handle<Mesh>,
    pub shadow_disc: Handle<Mesh>,

    // Enhanced Overhead Health & Stamina Bar Meshes
    pub hp_frame: Handle<Mesh>,
    pub hp_bg: Handle<Mesh>,
    pub hp_fill: Handle<Mesh>,
    pub stamina_bg: Handle<Mesh>,
    pub stamina_fill: Handle<Mesh>,
    pub class_badge_mesh: Handle<Mesh>,

    // Combat & VFX Meshes
    pub arrow_shaft: Handle<Mesh>,
    pub arrow_head: Handle<Mesh>,
    pub magic_orb: Handle<Mesh>,
    pub divine_star: Handle<Mesh>,
    pub slash_arc: Handle<Mesh>,
    pub slash_trail_mesh: Handle<Mesh>,
    pub x_slash: Handle<Mesh>,
    pub spark_sphere: Handle<Mesh>,
    pub shockwave_ring: Handle<Mesh>,
    pub dust_puff_mesh: Handle<Mesh>,
    pub holy_pillar: Handle<Mesh>,
    pub holy_ground_ring: Handle<Mesh>,
    pub spotlight_ring: Handle<Mesh>,

    // Materials - Environment
    pub arena_stone: Handle<StandardMaterial>,
    pub arena_rim_mat: Handle<StandardMaterial>,
    pub arena_pillar_mat: Handle<StandardMaterial>,
    pub brazier_fire: Handle<StandardMaterial>,
    pub divider_mat: Handle<StandardMaterial>,
    pub banner_pole_mat: Handle<StandardMaterial>,
    pub banner_blue_mat: Handle<StandardMaterial>,
    pub banner_red_mat: Handle<StandardMaterial>,
    pub banner_gold_mat: Handle<StandardMaterial>,
    pub rune_glow_mat: Handle<StandardMaterial>,
    pub rune_base_mat: Handle<StandardMaterial>,
    pub ember_mat: Handle<StandardMaterial>,

    // Materials - Tiles & Factions
    pub tile_player: Handle<StandardMaterial>,
    pub tile_player_hover: Handle<StandardMaterial>,
    pub tile_enemy: Handle<StandardMaterial>,
    pub tile_enemy_hover: Handle<StandardMaterial>,
    pub player_ring: Handle<StandardMaterial>,
    pub enemy_ring: Handle<StandardMaterial>,
    pub player_base: Handle<StandardMaterial>,
    pub enemy_base: Handle<StandardMaterial>,
    pub player_cape: Handle<StandardMaterial>,
    pub enemy_cape: Handle<StandardMaterial>,
    pub shadow_aura: Handle<StandardMaterial>,

    // Materials - Knight
    pub knight_armor: Handle<StandardMaterial>,
    pub knight_plume: Handle<StandardMaterial>,
    pub gold_trim: Handle<StandardMaterial>,
    pub shield_front: Handle<StandardMaterial>,
    pub steel_blade: Handle<StandardMaterial>,

    // Materials - Archer
    pub archer_tunic: Handle<StandardMaterial>,
    pub archer_leather: Handle<StandardMaterial>,
    pub archer_wood: Handle<StandardMaterial>,
    pub archer_feather: Handle<StandardMaterial>,
    pub arrow_glow: Handle<StandardMaterial>,

    // Materials - Mage
    pub mage_robe: Handle<StandardMaterial>,
    pub mage_hat: Handle<StandardMaterial>,
    pub mage_wood: Handle<StandardMaterial>,
    pub mage_crystal: Handle<StandardMaterial>,
    pub lightning: Handle<StandardMaterial>,

    // Materials - Assassin
    pub assassin_suit: Handle<StandardMaterial>,
    pub assassin_scarf: Handle<StandardMaterial>,
    pub assassin_eyes: Handle<StandardMaterial>,
    pub poison_blade: Handle<StandardMaterial>,

    // Materials - Cleric
    pub cleric_robe: Handle<StandardMaterial>,
    pub cleric_gold: Handle<StandardMaterial>,
    pub cleric_halo: Handle<StandardMaterial>,
    pub cleric_scepter: Handle<StandardMaterial>,
    pub heal_glow: Handle<StandardMaterial>,

    // Combat VFX Materials
    pub slash_mat: Handle<StandardMaterial>,
    pub slash_trail_mat: Handle<StandardMaterial>,
    pub x_slash_mat: Handle<StandardMaterial>,
    pub spark_mat: Handle<StandardMaterial>,
    pub shockwave_mat: Handle<StandardMaterial>,
    pub dust_mat: Handle<StandardMaterial>,
    pub holy_pillar_mat: Handle<StandardMaterial>,
    pub holy_ground_mat: Handle<StandardMaterial>,
    pub spotlight_mat: Handle<StandardMaterial>,
    pub hit_flash_mat: Handle<StandardMaterial>,

    // Common & UI Materials
    pub skin: Handle<StandardMaterial>,
    pub hp_frame_mat: Handle<StandardMaterial>,
    pub hp_bg_mat: Handle<StandardMaterial>,
    pub hp_green: Handle<StandardMaterial>,
    pub hp_yellow: Handle<StandardMaterial>,
    pub hp_red: Handle<StandardMaterial>,
    pub stamina_mat: Handle<StandardMaterial>,
    pub stamina_bg_mat: Handle<StandardMaterial>,
    pub badge_player_mat: Handle<StandardMaterial>,
    pub badge_enemy_mat: Handle<StandardMaterial>,
}

impl FromWorld for Game3dAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();

        let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let cylinder = meshes.add(Cylinder::new(1.0, 1.0));
        let sphere = meshes.add(Sphere::new(1.0));
        let cone = meshes.add(Cone::new(1.0, 1.0));
        let capsule = meshes.add(Capsule3d::new(0.5, 1.0));
        let torus_ring = meshes.add(Torus::new(0.04, 0.65));
        let torus_halo = meshes.add(Torus::new(0.035, 0.22));

        let tile_slab = meshes.add(Cuboid::new(TILE_SIZE, TILE_HEIGHT, TILE_SIZE));
        let tile_border = meshes.add(Cuboid::new(
            TILE_SIZE + 0.12,
            TILE_HEIGHT * 0.5,
            TILE_SIZE + 0.12,
        ));
        let pedestal = meshes.add(Cylinder::new(0.65, 0.08));
        let arena_floor = meshes.add(Cuboid::new(16.8, 0.8, 10.8));
        let arena_rim = meshes.add(Cuboid::new(18.2, 0.4, 12.2));
        let arena_pillar = meshes.add(Cylinder::new(0.38, 2.2));
        let arena_brazier = meshes.add(Cylinder::new(0.55, 0.35));
        let divider = meshes.add(Cuboid::new(0.12, 0.03, 8.8));

        let banner_pole = meshes.add(Cylinder::new(0.04, 2.6));
        let banner_cloth = meshes.add(Cuboid::new(0.72, 1.45, 0.02));
        let rune_circle = meshes.add(Torus::new(0.06, 1.85));
        let rune_disc = meshes.add(Cylinder::new(1.9, 0.02));
        let ember_particle = meshes.add(Sphere::new(0.05));

        let cape = meshes.add(Cuboid::new(0.48, 0.85, 0.04));
        let small_sphere = meshes.add(Sphere::new(0.06));
        let shadow_disc = meshes.add(Cylinder::new(0.72, 0.02));

        // Overhead Bars
        let hp_frame = meshes.add(Cuboid::new(1.36, 0.26, 0.02));
        let hp_bg = meshes.add(Cuboid::new(1.18, 0.10, 0.03));
        let hp_fill = meshes.add(Cuboid::new(1.16, 0.08, 0.04));
        let stamina_bg = meshes.add(Cuboid::new(1.18, 0.04, 0.03));
        let stamina_fill = meshes.add(Cuboid::new(1.16, 0.035, 0.04));
        let class_badge_mesh = meshes.add(Sphere::new(0.11));

        let arrow_shaft = meshes.add(Cylinder::new(0.02, 0.6));
        let arrow_head = meshes.add(Cone::new(0.06, 0.12));
        let magic_orb = meshes.add(Sphere::new(0.24));
        let divine_star = meshes.add(Sphere::new(0.22));
        let slash_arc = meshes.add(Torus::new(0.04, 0.65));
        let slash_trail_mesh = meshes.add(Torus::new(0.08, 0.95));
        let x_slash = meshes.add(Cuboid::new(0.06, 0.95, 0.02));
        let spark_sphere = meshes.add(Sphere::new(0.08));
        let shockwave_ring = meshes.add(Torus::new(0.05, 1.1));
        let dust_puff_mesh = meshes.add(Torus::new(0.06, 0.5));
        let holy_pillar = meshes.add(Cylinder::new(0.65, 3.2));
        let holy_ground_ring = meshes.add(Torus::new(0.06, 0.85));
        let spotlight_ring = meshes.add(Torus::new(0.05, 0.75));

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();

        let arena_stone = materials.add(StandardMaterial {
            base_color: Color::srgb(0.11, 0.13, 0.17),
            perceptual_roughness: 0.85,
            ..default()
        });
        let arena_rim_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.26, 0.22, 0.16),
            metallic: 0.5,
            perceptual_roughness: 0.45,
            ..default()
        });
        let arena_pillar_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.22, 0.28),
            perceptual_roughness: 0.65,
            ..default()
        });
        let brazier_fire = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.65, 0.12),
            emissive: LinearRgba::rgb(5.5, 2.5, 0.6),
            ..default()
        });
        let divider_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.85, 0.3),
            emissive: LinearRgba::rgb(2.2, 1.6, 0.4),
            ..default()
        });

        // Banner materials
        let banner_pole_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 0.25, 0.15),
            metallic: 0.6,
            perceptual_roughness: 0.4,
            ..default()
        });
        let banner_blue_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.32, 0.75),
            emissive: LinearRgba::rgb(0.4, 1.0, 2.5),
            perceptual_roughness: 0.6,
            ..default()
        });
        let banner_red_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.75, 0.12, 0.18),
            emissive: LinearRgba::rgb(2.5, 0.4, 0.5),
            perceptual_roughness: 0.6,
            ..default()
        });
        let banner_gold_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.82, 0.25),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        });

        // Center Arena Runic Inlay
        let rune_glow_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.85, 1.0),
            emissive: LinearRgba::rgb(2.5, 4.5, 6.0),
            ..default()
        });
        let rune_base_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.14, 0.16, 0.22),
            perceptual_roughness: 0.7,
            ..default()
        });

        // Embers
        let ember_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.6, 0.1),
            emissive: LinearRgba::rgb(6.0, 3.2, 0.8),
            ..default()
        });

        let tile_player = materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.14, 0.26),
            perceptual_roughness: 0.6,
            ..default()
        });
        let tile_player_hover = materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.40, 0.75),
            emissive: LinearRgba::rgb(0.8, 2.0, 3.8),
            perceptual_roughness: 0.3,
            ..default()
        });
        let tile_enemy = materials.add(StandardMaterial {
            base_color: Color::srgb(0.24, 0.08, 0.12),
            perceptual_roughness: 0.6,
            ..default()
        });
        let tile_enemy_hover = materials.add(StandardMaterial {
            base_color: Color::srgb(0.65, 0.15, 0.22),
            emissive: LinearRgba::rgb(3.5, 0.6, 0.8),
            perceptual_roughness: 0.3,
            ..default()
        });

        let player_ring = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 1.0),
            emissive: LinearRgba::rgb(0.8, 2.2, 4.5),
            ..default()
        });
        let enemy_ring = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.2, 0.3),
            emissive: LinearRgba::rgb(4.2, 0.6, 0.8),
            ..default()
        });

        let player_base = materials.add(StandardMaterial {
            base_color: Color::srgb(0.06, 0.12, 0.22),
            metallic: 0.2,
            ..default()
        });
        let enemy_base = materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.06, 0.08),
            metallic: 0.2,
            ..default()
        });

        let player_cape = materials.add(StandardMaterial {
            base_color: Color::srgb(0.14, 0.32, 0.72),
            perceptual_roughness: 0.6,
            ..default()
        });
        let enemy_cape = materials.add(StandardMaterial {
            base_color: Color::srgb(0.70, 0.12, 0.16),
            perceptual_roughness: 0.6,
            ..default()
        });
        let shadow_aura = materials.add(StandardMaterial {
            base_color: Color::srgba(0.04, 0.04, 0.07, 0.85),
            emissive: LinearRgba::rgb(0.3, 0.05, 0.4),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        // Knight materials
        let knight_armor = materials.add(StandardMaterial {
            base_color: Color::srgb(0.82, 0.86, 0.94),
            metallic: 0.95,
            perceptual_roughness: 0.15,
            ..default()
        });
        let knight_plume = materials.add(StandardMaterial {
            base_color: Color::srgb(0.92, 0.18, 0.18),
            emissive: LinearRgba::rgb(1.2, 0.15, 0.15),
            ..default()
        });
        let gold_trim = materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.84, 0.22),
            metallic: 0.88,
            perceptual_roughness: 0.22,
            ..default()
        });
        let shield_front = materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.38, 0.80),
            metallic: 0.65,
            perceptual_roughness: 0.28,
            ..default()
        });
        let steel_blade = materials.add(StandardMaterial {
            base_color: Color::srgb(0.96, 0.98, 1.0),
            metallic: 0.98,
            perceptual_roughness: 0.08,
            emissive: LinearRgba::rgb(0.3, 0.4, 0.6),
            ..default()
        });

        // Archer materials
        let archer_tunic = materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.50, 0.25),
            perceptual_roughness: 0.65,
            ..default()
        });
        let archer_leather = materials.add(StandardMaterial {
            base_color: Color::srgb(0.38, 0.24, 0.14),
            perceptual_roughness: 0.55,
            ..default()
        });
        let archer_wood = materials.add(StandardMaterial {
            base_color: Color::srgb(0.52, 0.30, 0.16),
            perceptual_roughness: 0.5,
            ..default()
        });
        let archer_feather = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.88, 0.4),
            emissive: LinearRgba::rgb(0.8, 2.5, 1.2),
            ..default()
        });
        let arrow_glow = materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 1.0, 0.55),
            emissive: LinearRgba::rgb(1.5, 4.8, 2.2),
            ..default()
        });

        // Mage materials
        let mage_robe = materials.add(StandardMaterial {
            base_color: Color::srgb(0.38, 0.15, 0.65),
            perceptual_roughness: 0.65,
            ..default()
        });
        let mage_hat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.08, 0.42),
            perceptual_roughness: 0.7,
            ..default()
        });
        let mage_wood = materials.add(StandardMaterial {
            base_color: Color::srgb(0.26, 0.18, 0.12),
            perceptual_roughness: 0.6,
            ..default()
        });
        let mage_crystal = materials.add(StandardMaterial {
            base_color: Color::srgb(0.75, 0.35, 1.0),
            emissive: LinearRgba::rgb(4.5, 2.0, 5.5),
            ..default()
        });
        let lightning = materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.45, 1.0),
            emissive: LinearRgba::rgb(4.8, 2.2, 6.0),
            ..default()
        });

        // Assassin materials
        let assassin_suit = materials.add(StandardMaterial {
            base_color: Color::srgb(0.11, 0.11, 0.14),
            perceptual_roughness: 0.75,
            ..default()
        });
        let assassin_scarf = materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.10, 0.18),
            emissive: LinearRgba::rgb(1.2, 0.1, 0.2),
            ..default()
        });
        let assassin_eyes = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.15, 0.15),
            emissive: LinearRgba::rgb(5.0, 0.6, 0.6),
            ..default()
        });
        let poison_blade = materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 1.0, 0.45),
            emissive: LinearRgba::rgb(0.8, 4.5, 1.5),
            metallic: 0.88,
            ..default()
        });

        // Cleric materials
        let cleric_robe = materials.add(StandardMaterial {
            base_color: Color::srgb(0.96, 0.96, 0.98),
            perceptual_roughness: 0.55,
            ..default()
        });
        let cleric_gold = materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.85, 0.25),
            metallic: 0.80,
            perceptual_roughness: 0.22,
            ..default()
        });
        let cleric_halo = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.94, 0.45),
            emissive: LinearRgba::rgb(5.0, 4.2, 1.2),
            ..default()
        });
        let cleric_scepter = materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.86, 0.3),
            metallic: 0.85,
            ..default()
        });
        let heal_glow = materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.92, 0.35),
            emissive: LinearRgba::rgb(4.8, 4.2, 1.2),
            ..default()
        });

        // Combat VFX Materials with HDR Bloom
        let slash_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.95, 1.0),
            emissive: LinearRgba::rgb(4.8, 5.5, 7.0),
            ..default()
        });
        let slash_trail_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.85, 0.3),
            emissive: LinearRgba::rgb(5.8, 3.8, 0.8),
            ..default()
        });
        let x_slash_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.2, 0.3),
            emissive: LinearRgba::rgb(6.0, 0.8, 1.0),
            ..default()
        });
        let spark_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.9, 0.3),
            emissive: LinearRgba::rgb(5.5, 4.2, 1.2),
            ..default()
        });
        let shockwave_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.45, 1.0),
            emissive: LinearRgba::rgb(5.0, 2.0, 6.5),
            ..default()
        });
        let dust_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.82, 0.84, 0.90, 0.55),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let holy_pillar_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.96, 0.7, 0.75),
            emissive: LinearRgba::rgb(5.2, 4.5, 1.5),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let holy_ground_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.9, 0.4),
            emissive: LinearRgba::rgb(3.8, 3.0, 0.9),
            ..default()
        });
        let spotlight_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.90, 0.25),
            emissive: LinearRgba::rgb(4.5, 3.5, 0.8),
            ..default()
        });
        let hit_flash_mat = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            emissive: LinearRgba::rgb(7.0, 7.0, 7.0),
            ..default()
        });

        // Common & UI Materials
        let skin = materials.add(StandardMaterial {
            base_color: Color::srgb(0.92, 0.74, 0.62),
            perceptual_roughness: 0.7,
            ..default()
        });
        let hp_frame_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.18, 0.22),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        });
        let hp_bg_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.06, 0.06, 0.08),
            unlit: true,
            ..default()
        });
        let hp_green = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.88, 0.38),
            emissive: LinearRgba::rgb(0.8, 2.5, 1.0),
            unlit: true,
            ..default()
        });
        let hp_yellow = materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.82, 0.2),
            emissive: LinearRgba::rgb(2.5, 2.0, 0.4),
            unlit: true,
            ..default()
        });
        let hp_red = materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.22, 0.22),
            emissive: LinearRgba::rgb(2.8, 0.5, 0.5),
            unlit: true,
            ..default()
        });
        let stamina_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.75, 1.0),
            emissive: LinearRgba::rgb(0.6, 2.0, 4.0),
            unlit: true,
            ..default()
        });
        let stamina_bg_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.04, 0.08, 0.14),
            unlit: true,
            ..default()
        });
        let badge_player_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 1.0),
            emissive: LinearRgba::rgb(0.8, 2.0, 3.5),
            ..default()
        });
        let badge_enemy_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.25, 0.25),
            emissive: LinearRgba::rgb(3.5, 0.6, 0.6),
            ..default()
        });

        Game3dAssets {
            cube,
            cylinder,
            sphere,
            cone,
            capsule,
            torus_ring,
            torus_halo,
            tile_slab,
            tile_border,
            pedestal,
            arena_floor,
            arena_rim,
            arena_pillar,
            arena_brazier,
            divider,
            banner_pole,
            banner_cloth,
            rune_circle,
            rune_disc,
            ember_particle,
            cape,
            small_sphere,
            shadow_disc,
            hp_frame,
            hp_bg,
            hp_fill,
            stamina_bg,
            stamina_fill,
            class_badge_mesh,
            arrow_shaft,
            arrow_head,
            magic_orb,
            divine_star,
            slash_arc,
            slash_trail_mesh,
            x_slash,
            spark_sphere,
            shockwave_ring,
            dust_puff_mesh,
            holy_pillar,
            holy_ground_ring,
            spotlight_ring,
            arena_stone,
            arena_rim_mat,
            arena_pillar_mat,
            brazier_fire,
            divider_mat,
            banner_pole_mat,
            banner_blue_mat,
            banner_red_mat,
            banner_gold_mat,
            rune_glow_mat,
            rune_base_mat,
            ember_mat,
            tile_player,
            tile_player_hover,
            tile_enemy,
            tile_enemy_hover,
            player_ring,
            enemy_ring,
            player_base,
            enemy_base,
            player_cape,
            enemy_cape,
            shadow_aura,
            knight_armor,
            knight_plume,
            gold_trim,
            shield_front,
            steel_blade,
            archer_tunic,
            archer_leather,
            archer_wood,
            archer_feather,
            arrow_glow,
            mage_robe,
            mage_hat,
            mage_wood,
            mage_crystal,
            lightning,
            assassin_suit,
            assassin_scarf,
            assassin_eyes,
            poison_blade,
            cleric_robe,
            cleric_gold,
            cleric_halo,
            cleric_scepter,
            heal_glow,
            slash_mat,
            slash_trail_mat,
            x_slash_mat,
            spark_mat,
            shockwave_mat,
            dust_mat,
            holy_pillar_mat,
            holy_ground_mat,
            spotlight_mat,
            hit_flash_mat,
            skin,
            hp_frame_mat,
            hp_bg_mat,
            hp_green,
            hp_yellow,
            hp_red,
            stamina_mat,
            stamina_bg_mat,
            badge_player_mat,
            badge_enemy_mat,
        }
    }
}
