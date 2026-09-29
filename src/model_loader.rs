use crate::types::*;
use bevy::prelude::*;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CharacterAnimation {
    Idle,
    Attack,
    Run,
    Hit,
    Die,
}

#[derive(Component)]
pub struct CustomGltfModelRoot {
    pub class: UnitClass,
}

#[derive(Resource, Default)]
pub struct GltfModelAssets {
    pub models: HashMap<UnitClass, Handle<Scene>>,
    pub graphs: HashMap<UnitClass, Handle<AnimationGraph>>,
    pub node_indices: HashMap<UnitClass, HashMap<CharacterAnimation, AnimationNodeIndex>>,
    pub available_classes: Vec<UnitClass>,
}

impl GltfModelAssets {
    pub fn has_model(&self, class: UnitClass) -> bool {
        self.available_classes.contains(&class)
    }
}

pub fn setup_gltf_models(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut model_assets = GltfModelAssets::default();

    let class_files = [
        (UnitClass::Knight, "models/knight.glb", "knight.glb"),
        (UnitClass::Archer, "models/archer.glb", "archer.glb"),
        (UnitClass::Mage, "models/mage.glb", "mage.glb"),
        (UnitClass::Assassin, "models/assassin.glb", "assassin.glb"),
        (UnitClass::Cleric, "models/cleric.glb", "cleric.glb"),
    ];

    for (class, rel_path, file_name) in class_files {
        let os_path = Path::new("assets/models").join(file_name);
        if os_path.exists() {
            info!("Found custom 3D model for {:?} at {:?}", class, os_path);
            let scene_handle = asset_server.load(format!("{}#Scene0", rel_path));
            model_assets.models.insert(class, scene_handle);

            // Create animation graph with standard animations
            let mut graph = AnimationGraph::new();
            let mut indices = HashMap::new();

            // Load animations by index (0: Idle, 1: Attack, 2: Run, 3: Hit, 4: Die)
            let idle_clip = asset_server.load(format!("{}#Animation0", rel_path));
            let idle_node = graph.add_clip(idle_clip, 1.0, graph.root);
            indices.insert(CharacterAnimation::Idle, idle_node);

            let attack_clip = asset_server.load(format!("{}#Animation1", rel_path));
            let attack_node = graph.add_clip(attack_clip, 1.0, graph.root);
            indices.insert(CharacterAnimation::Attack, attack_node);

            let run_clip = asset_server.load(format!("{}#Animation2", rel_path));
            let run_node = graph.add_clip(run_clip, 1.0, graph.root);
            indices.insert(CharacterAnimation::Run, run_node);

            let hit_clip = asset_server.load(format!("{}#Animation3", rel_path));
            let hit_node = graph.add_clip(hit_clip, 1.0, graph.root);
            indices.insert(CharacterAnimation::Hit, hit_node);

            let die_clip = asset_server.load(format!("{}#Animation4", rel_path));
            let die_node = graph.add_clip(die_clip, 1.0, graph.root);
            indices.insert(CharacterAnimation::Die, die_node);

            let graph_handle = animation_graphs.add(graph);
            model_assets.graphs.insert(class, graph_handle);
            model_assets.node_indices.insert(class, indices);
            model_assets.available_classes.push(class);
        }
    }

    commands.insert_resource(model_assets);
}

/// Automatically binds AnimationGraph and starts playing Idle animation when a glTF AnimationPlayer spawns
pub fn auto_bind_gltf_animations(
    mut commands: Commands,
    gltf_assets: Res<GltfModelAssets>,
    roots: Query<(Entity, &CustomGltfModelRoot)>,
    children_query: Query<&Children>,
    players: Query<Entity, (With<AnimationPlayer>, Without<AnimationGraphHandle>)>,
    mut player_query: Query<&mut AnimationPlayer>,
) {
    for (root_entity, root_model) in roots.iter() {
        if let Some(graph_handle) = gltf_assets.graphs.get(&root_model.class) {
            // Find AnimationPlayer inside children
            for child in children_query.iter_descendants(root_entity) {
                if let Ok(player_ent) = players.get(child) {
                    let mut cmd = commands.entity(player_ent);
                    cmd.insert(AnimationGraphHandle(graph_handle.clone()));
                    cmd.insert(AnimationTransitions::new());

                    if let Some(nodes) = gltf_assets.node_indices.get(&root_model.class) {
                        if let Some(&idle_node) = nodes.get(&CharacterAnimation::Idle) {
                            if let Ok(mut player) = player_query.get_mut(player_ent) {
                                player.play(idle_node).repeat();
                            }
                        }
                    }
                }
            }
        }
    }
}
