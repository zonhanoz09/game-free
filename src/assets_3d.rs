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

    // Combat & UI Meshes
    pub hp_bg: Handle<Mesh>,
    pub hp_fill: Handle<Mesh>,
    pub arrow_shaft: Handle<Mesh>,
    pub arrow_head: Handle<Mesh>,
    pub magic_orb: Handle<Mesh>,
    pub divine_star: Handle<Mesh>,

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

        let hp_bg = meshes.add(Cuboid::new(1.2, 0.10, 0.03));
        let hp_fill = meshes.add(Cuboid::new(1.16, 0.07, 0.04));
        let arrow_shaft = meshes.add(Cylinder::new(0.02, 0.6));
        let arrow_head = meshes.add(Cone::new(0.06, 0.12));
        let magic_orb = meshes.add(Sphere::new(0.22));
        let divine_star = meshes.add(Sphere::new(0.20));

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

        // Knight materials
        let knight_armor = materials.add(StandardMaterial {
            base_color: Color::srgb(0.72, 0.76, 0.84),
            metallic: 0.85,
            perceptual_roughness: 0.25,
            ..default()
        });
        let knight_plume = materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.18, 0.18),
            ..default()
        });
        let gold_trim = materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.78, 0.22),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        });
        let shield_front = materials.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.38, 0.75),
            metallic: 0.5,
            ..default()
        });
        let steel_blade = materials.add(StandardMaterial {
            base_color: Color::srgb(0.88, 0.92, 0.98),
            metallic: 0.95,
            perceptual_roughness: 0.15,
            ..default()
        });

        // Archer materials
        let archer_tunic = materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.46, 0.22),
            perceptual_roughness: 0.7,
            ..default()
        });
        let archer_leather = materials.add(StandardMaterial {
            base_color: Color::srgb(0.36, 0.22, 0.12),
            perceptual_roughness: 0.6,
            ..default()
        });
        let archer_wood = materials.add(StandardMaterial {
            base_color: Color::srgb(0.48, 0.28, 0.14),
            perceptual_roughness: 0.5,
            ..default()
        });
        let archer_feather = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.4),
            emissive: LinearRgba::rgb(0.2, 1.2, 0.4),
            ..default()
        });
        let arrow_glow = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.9, 0.4),
            emissive: LinearRgba::rgb(0.4, 2.0, 0.6),
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
            emissive: LinearRgba::rgb(2.2, 0.8, 3.0),
            ..default()
        });
        let lightning = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.4, 1.0),
            emissive: LinearRgba::rgb(2.5, 1.0, 3.2),
            ..default()
        });

        // Assassin materials
        let assassin_suit = materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.12, 0.15),
            perceptual_roughness: 0.75,
            ..default()
        });
        let assassin_scarf = materials.add(StandardMaterial {
            base_color: Color::srgb(0.75, 0.10, 0.18),
            ..default()
        });
        let assassin_eyes = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.2, 0.2),
            emissive: LinearRgba::rgb(3.0, 0.3, 0.3),
            ..default()
        });
        let poison_blade = materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.7, 0.3),
            emissive: LinearRgba::rgb(0.2, 2.2, 0.5),
            metallic: 0.8,
            ..default()
        });

        // Cleric materials
        let cleric_robe = materials.add(StandardMaterial {
            base_color: Color::srgb(0.94, 0.94, 0.96),
            perceptual_roughness: 0.6,
            ..default()
        });
        let cleric_gold = materials.add(StandardMaterial {
            base_color: Color::srgb(0.96, 0.82, 0.25),
            metallic: 0.7,
            perceptual_roughness: 0.3,
            ..default()
        });
        let cleric_halo = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.9, 0.4),
            emissive: LinearRgba::rgb(3.0, 2.4, 0.6),
            ..default()
        });
        let cleric_scepter = materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.85, 0.3),
            metallic: 0.8,
            ..default()
        });
        let heal_glow = materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.9, 0.3),
            emissive: LinearRgba::rgb(2.5, 2.2, 0.5),
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
            hp_bg,
            hp_fill,
            arrow_shaft,
            arrow_head,
            magic_orb,
            divine_star,
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
            skin,
            hp_bg_mat,
            hp_green,
            hp_yellow,
            hp_red,
        }
    }
}
