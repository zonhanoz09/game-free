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

    // Character Detail Meshes
    pub cape: Handle<Mesh>,
    pub small_sphere: Handle<Mesh>,
    pub shadow_disc: Handle<Mesh>,

    // Combat & VFX Meshes
    pub hp_bg: Handle<Mesh>,
    pub hp_fill: Handle<Mesh>,
    pub arrow_shaft: Handle<Mesh>,
    pub arrow_head: Handle<Mesh>,
    pub magic_orb: Handle<Mesh>,
    pub divine_star: Handle<Mesh>,
    pub slash_arc: Handle<Mesh>,
    pub x_slash: Handle<Mesh>,
    pub spark_sphere: Handle<Mesh>,
    pub shockwave_ring: Handle<Mesh>,
    pub holy_pillar: Handle<Mesh>,
    pub spotlight_ring: Handle<Mesh>,

    // Materials - Environment
    pub arena_stone: Handle<StandardMaterial>,
    pub arena_rim_mat: Handle<StandardMaterial>,
    pub arena_pillar_mat: Handle<StandardMaterial>,
    pub brazier_fire: Handle<StandardMaterial>,
    pub divider_mat: Handle<StandardMaterial>,

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
    pub x_slash_mat: Handle<StandardMaterial>,
    pub spark_mat: Handle<StandardMaterial>,
    pub shockwave_mat: Handle<StandardMaterial>,
    pub holy_pillar_mat: Handle<StandardMaterial>,
    pub spotlight_mat: Handle<StandardMaterial>,
    pub hit_flash_mat: Handle<StandardMaterial>,

    // Common & UI Materials
    pub skin: Handle<StandardMaterial>,
    pub hp_bg_mat: Handle<StandardMaterial>,
    pub hp_green: Handle<StandardMaterial>,
    pub hp_yellow: Handle<StandardMaterial>,
    pub hp_red: Handle<StandardMaterial>,
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
        let arena_floor = meshes.add(Cuboid::new(16.5, 0.8, 10.5));
        let arena_rim = meshes.add(Cuboid::new(17.5, 0.4, 11.5));
        let arena_pillar = meshes.add(Cylinder::new(0.35, 1.8));
        let arena_brazier = meshes.add(Cylinder::new(0.55, 0.35));
        let divider = meshes.add(Cuboid::new(0.12, 0.03, 8.5));

        let cape = meshes.add(Cuboid::new(0.48, 0.85, 0.04));
        let small_sphere = meshes.add(Sphere::new(0.06));
        let shadow_disc = meshes.add(Cylinder::new(0.72, 0.02));

        let hp_bg = meshes.add(Cuboid::new(1.2, 0.10, 0.03));
        let hp_fill = meshes.add(Cuboid::new(1.16, 0.07, 0.04));
        let arrow_shaft = meshes.add(Cylinder::new(0.02, 0.6));
        let arrow_head = meshes.add(Cone::new(0.06, 0.12));
        let magic_orb = meshes.add(Sphere::new(0.24));
        let divine_star = meshes.add(Sphere::new(0.22));
        let slash_arc = meshes.add(Torus::new(0.04, 0.65));
        let x_slash = meshes.add(Cuboid::new(0.06, 0.95, 0.02));
        let spark_sphere = meshes.add(Sphere::new(0.08));
        let shockwave_ring = meshes.add(Torus::new(0.05, 1.1));
        let holy_pillar = meshes.add(Cylinder::new(0.65, 3.2));
        let spotlight_ring = meshes.add(Torus::new(0.05, 0.75));

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();

        let arena_stone = materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.14, 0.18),
            perceptual_roughness: 0.8,
            ..default()
        });
        let arena_rim_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.24, 0.20, 0.15),
            metallic: 0.4,
            perceptual_roughness: 0.5,
            ..default()
        });
        let arena_pillar_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.20, 0.24),
            perceptual_roughness: 0.7,
            ..default()
        });
        let brazier_fire = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.6, 0.1),
            emissive: LinearRgba::rgb(3.0, 1.5, 0.3),
            ..default()
        });
        let divider_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.85, 0.3),
            emissive: LinearRgba::rgb(1.5, 1.2, 0.3),
            ..default()
        });

        let tile_player = materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.14, 0.26),
            perceptual_roughness: 0.6,
            ..default()
        });
        let tile_player_hover = materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.40, 0.75),
            emissive: LinearRgba::rgb(0.4, 1.2, 2.2),
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
            emissive: LinearRgba::rgb(2.2, 0.4, 0.5),
            perceptual_roughness: 0.3,
            ..default()
        });

        let player_ring = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 1.0),
            emissive: LinearRgba::rgb(0.4, 1.2, 2.5),
            ..default()
        });
        let enemy_ring = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.2, 0.3),
            emissive: LinearRgba::rgb(2.5, 0.3, 0.4),
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
            base_color: Color::srgba(0.05, 0.05, 0.08, 0.7),
            emissive: LinearRgba::rgb(0.1, 0.0, 0.2),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        // Knight materials
        let knight_armor = materials.add(StandardMaterial {
            base_color: Color::srgb(0.76, 0.80, 0.88),
            metallic: 0.90,
            perceptual_roughness: 0.20,
            ..default()
        });
        let knight_plume = materials.add(StandardMaterial {
            base_color: Color::srgb(0.90, 0.18, 0.18),
            emissive: LinearRgba::rgb(0.5, 0.05, 0.05),
            ..default()
        });
        let gold_trim = materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.82, 0.22),
            metallic: 0.85,
            perceptual_roughness: 0.25,
            ..default()
        });
        let shield_front = materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.36, 0.75),
            metallic: 0.6,
            perceptual_roughness: 0.3,
            ..default()
        });
        let steel_blade = materials.add(StandardMaterial {
            base_color: Color::srgb(0.92, 0.95, 1.0),
            metallic: 0.98,
            perceptual_roughness: 0.12,
            ..default()
        });

        // Archer materials
        let archer_tunic = materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.48, 0.24),
            perceptual_roughness: 0.65,
            ..default()
        });
        let archer_leather = materials.add(StandardMaterial {
            base_color: Color::srgb(0.38, 0.24, 0.14),
            perceptual_roughness: 0.55,
            ..default()
        });
        let archer_wood = materials.add(StandardMaterial {
            base_color: Color::srgb(0.50, 0.30, 0.16),
            perceptual_roughness: 0.5,
            ..default()
        });
        let archer_feather = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.85, 0.4),
            emissive: LinearRgba::rgb(0.3, 1.4, 0.5),
            ..default()
        });
        let arrow_glow = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.95, 0.45),
            emissive: LinearRgba::rgb(0.5, 2.2, 0.7),
            ..default()
        });

        // Mage materials
        let mage_robe = materials.add(StandardMaterial {
            base_color: Color::srgb(0.38, 0.15, 0.62),
            perceptual_roughness: 0.65,
            ..default()
        });
        let mage_hat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.08, 0.40),
            perceptual_roughness: 0.7,
            ..default()
        });
        let mage_wood = materials.add(StandardMaterial {
            base_color: Color::srgb(0.26, 0.18, 0.12),
            perceptual_roughness: 0.6,
            ..default()
        });
        let mage_crystal = materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.3, 0.95),
            emissive: LinearRgba::rgb(2.5, 0.9, 3.2),
            ..default()
        });
        let lightning = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.4, 1.0),
            emissive: LinearRgba::rgb(3.0, 1.2, 3.8),
            ..default()
        });

        // Assassin materials
        let assassin_suit = materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.12, 0.15),
            perceptual_roughness: 0.75,
            ..default()
        });
        let assassin_scarf = materials.add(StandardMaterial {
            base_color: Color::srgb(0.80, 0.10, 0.18),
            emissive: LinearRgba::rgb(0.4, 0.05, 0.08),
            ..default()
        });
        let assassin_eyes = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.15, 0.15),
            emissive: LinearRgba::rgb(3.5, 0.3, 0.3),
            ..default()
        });
        let poison_blade = materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.8, 0.35),
            emissive: LinearRgba::rgb(0.3, 2.5, 0.6),
            metallic: 0.85,
            ..default()
        });

        // Cleric materials
        let cleric_robe = materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.95, 0.98),
            perceptual_roughness: 0.55,
            ..default()
        });
        let cleric_gold = materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.84, 0.25),
            metallic: 0.75,
            perceptual_roughness: 0.25,
            ..default()
        });
        let cleric_halo = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.92, 0.4),
            emissive: LinearRgba::rgb(3.2, 2.6, 0.7),
            ..default()
        });
        let cleric_scepter = materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.85, 0.3),
            metallic: 0.85,
            ..default()
        });
        let heal_glow = materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.9, 0.3),
            emissive: LinearRgba::rgb(2.8, 2.4, 0.6),
            ..default()
        });

        // Combat VFX Materials
        let slash_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.95, 1.0),
            emissive: LinearRgba::rgb(2.8, 3.4, 4.5),
            ..default()
        });
        let x_slash_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.2, 0.3),
            emissive: LinearRgba::rgb(4.0, 0.5, 0.7),
            ..default()
        });
        let spark_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.8, 0.2),
            emissive: LinearRgba::rgb(3.8, 2.8, 0.8),
            ..default()
        });
        let shockwave_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.4, 1.0),
            emissive: LinearRgba::rgb(3.2, 1.2, 4.2),
            ..default()
        });
        let holy_pillar_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.95, 0.6, 0.65),
            emissive: LinearRgba::rgb(3.5, 3.0, 0.9),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let spotlight_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.88, 0.2),
            emissive: LinearRgba::rgb(2.8, 2.2, 0.4),
            ..default()
        });
        let hit_flash_mat = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            emissive: LinearRgba::rgb(5.0, 5.0, 5.0),
            ..default()
        });

        // Common & UI materials
        let skin = materials.add(StandardMaterial {
            base_color: Color::srgb(0.92, 0.74, 0.62),
            perceptual_roughness: 0.7,
            ..default()
        });
        let hp_bg_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.08, 0.10),
            unlit: true,
            ..default()
        });
        let hp_green = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.85, 0.35),
            emissive: LinearRgba::rgb(0.4, 1.8, 0.6),
            unlit: true,
            ..default()
        });
        let hp_yellow = materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.8, 0.2),
            emissive: LinearRgba::rgb(1.8, 1.5, 0.3),
            unlit: true,
            ..default()
        });
        let hp_red = materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.25, 0.25),
            emissive: LinearRgba::rgb(2.0, 0.4, 0.4),
            unlit: true,
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
            cape,
            small_sphere,
            shadow_disc,
            hp_bg,
            hp_fill,
            arrow_shaft,
            arrow_head,
            magic_orb,
            divine_star,
            slash_arc,
            x_slash,
            spark_sphere,
            shockwave_ring,
            holy_pillar,
            spotlight_ring,
            arena_stone,
            arena_rim_mat,
            arena_pillar_mat,
            brazier_fire,
            divider_mat,
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
            x_slash_mat,
            spark_mat,
            shockwave_mat,
            holy_pillar_mat,
            spotlight_mat,
            hit_flash_mat,
            skin,
            hp_bg_mat,
            hp_green,
            hp_yellow,
            hp_red,
        }
    }
}
