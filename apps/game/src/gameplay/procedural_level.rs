//! Procedural level generation.

use crate::{audio::MusicPool, gameplay::npc::NPC_RADIUS, screens::Screen};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_landmass::prelude::*;
use bevy_seedling::prelude::*;
use bevy_seedling::sample::Sample;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ProceduralLevelAssets>();
    app.register_type::<ProceduralLevel>();
}

/// A system that spawns a procedural level.
#[cfg_attr(feature = "hot_patch", hot)]
pub(crate) fn spawn_procedural_level(
    mut commands: Commands,
    assets: Res<ProceduralLevelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Spawn level container
    commands.spawn((
        Name::new("Procedural Level"),
        StateScoped(Screen::ProceduralGameplay),
        ProceduralLevel,
        children![(
            Name::new("Level Music"),
            SamplePlayer::new(assets.music.clone()).looping(),
            MusicPool
        )],
    ));

    // Set ambient light to none (like Volta level)
    commands.insert_resource(AmbientLight::NONE);

    // Create archipelago for navigation
    let _archipelago = commands
        .spawn((
            Name::new("Procedural Level Archipelago"),
            StateScoped(Screen::ProceduralGameplay),
            Archipelago3d::new(ArchipelagoOptions::from_agent_radius(NPC_RADIUS)),
        ))
        .id();

    // Generate the ground plane
    spawn_ground(&mut commands, &assets, &mut meshes);

    // Spawn player
    spawn_player(&mut commands);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn spawn_ground(
    commands: &mut Commands,
    assets: &ProceduralLevelAssets,
    meshes: &mut Assets<Mesh>,
) {
    // Generate hilly terrain using a heightfield collider
    let terrain_size = 200;
    let terrain_scale = 2.0;
    let mut heights = vec![vec![0.0; terrain_size]; terrain_size];

    // Generate hills using simple noise-like patterns
    for x in 0..terrain_size {
        for z in 0..terrain_size {
            let fx = x as f32 / terrain_size as f32;
            let fz = z as f32 / terrain_size as f32;

            // Combine multiple sine waves for rolling hills
            let height = (fx * 8.0).sin() * (fz * 8.0).sin() * 3.0 +  // Main hills
                (fx * 16.0).sin() * (fz * 16.0).sin() * 1.5 + // Smaller variations
                (fx * 4.0).sin() * (fz * 4.0).sin() * 5.0; // Large rolling hills

            heights[x][z] = height;
        }
    }

    // Create a mesh from the height data
    let terrain_mesh = create_terrain_mesh(&heights, terrain_size, terrain_scale);
    let terrain_mesh_handle = meshes.add(terrain_mesh);

    commands.spawn((
        Name::new("Ground"),
        Mesh3d(terrain_mesh_handle),
        MeshMaterial3d(assets.ground_material.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::heightfield(
            heights,
            Vec3::new(
                terrain_size as f32 * terrain_scale,
                1.0,
                terrain_size as f32 * terrain_scale,
            ),
        ),
        StateScoped(Screen::ProceduralGameplay),
    ));
}

fn create_terrain_mesh(heights: &Vec<Vec<f32>>, terrain_size: usize, terrain_scale: f32) -> Mesh {
    let num_vertices = terrain_size * terrain_size;
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = vec![[0.0, 1.0, 0.0]; num_vertices];
    let mut uvs = Vec::with_capacity(num_vertices);
    let mut indices = Vec::with_capacity((terrain_size - 1) * (terrain_size - 1) * 6);

    let total_width = terrain_size as f32 * terrain_scale;
    let total_depth = terrain_size as f32 * terrain_scale;

    let cell_width = total_width / (terrain_size - 1) as f32;
    let cell_depth = total_depth / (terrain_size - 1) as f32;

    // Tiling factor - how many times the texture repeats across the terrain
    let texture_tile_factor = 20.0; // Texture will repeat 20 times across the terrain

    for z in 0..terrain_size {
        for x in 0..terrain_size {
            let px = x as f32 * cell_width - total_width / 2.0;
            let pz = z as f32 * cell_depth - total_depth / 2.0;
            let py = heights[x][z];

            positions.push([px, py, pz]);
            // Tile the texture by multiplying UV coordinates
            uvs.push([
                (x as f32 / (terrain_size - 1) as f32) * texture_tile_factor,
                (z as f32 / (terrain_size - 1) as f32) * texture_tile_factor,
            ]);
        }
    }

    for z in 0..(terrain_size - 1) {
        for x in 0..(terrain_size - 1) {
            let top_left = (z * terrain_size + x) as u32;
            let top_right = top_left + 1;
            let bottom_left = top_left + terrain_size as u32;
            let bottom_right = bottom_left + 1;

            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);

            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    // Calculate normals
    for z in 1..(terrain_size - 1) {
        for x in 1..(terrain_size - 1) {
            let h_l = heights[x - 1][z];
            let h_r = heights[x + 1][z];
            let h_d = heights[x][z - 1];
            let h_u = heights[x][z + 1];

            let normal = Vec3::new(h_l - h_r, 2.0 * cell_width, h_d - h_u).normalize_or(Vec3::Y);
            normals[z * terrain_size + x] = normal.into();
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

#[cfg_attr(feature = "hot_patch", hot)]
fn spawn_player(commands: &mut Commands) {
    // Calculate terrain height at spawn position
    let spawn_x = -30.0;
    let spawn_z = 0.0;
    let terrain_height = get_terrain_height_at(spawn_x, spawn_z);

    // Spawn player entity at a good spawn position
    commands.spawn((
        Name::new("Procedural Player"),
        crate::gameplay::player::Player,
        Transform::from_xyz(spawn_x, terrain_height + 2.0, spawn_z), // Spawn above terrain
        Visibility::default(),
        StateScoped(Screen::ProceduralGameplay),
    ));
}

// Helper function to calculate terrain height at a given position
fn get_terrain_height_at(x: f32, z: f32) -> f32 {
    let terrain_size = 200.0;
    let terrain_scale = 2.0;

    // Convert world coordinates to normalized coordinates
    let fx = (x / (terrain_size * terrain_scale) + 0.5).clamp(0.0, 1.0);
    let fz = (z / (terrain_size * terrain_scale) + 0.5).clamp(0.0, 1.0);

    // Calculate height using the same formula as terrain generation
    let height = (fx * 8.0).sin() * (fz * 8.0).sin() * 3.0 +  // Main hills
        (fx * 16.0).sin() * (fz * 16.0).sin() * 1.5 + // Smaller variations
        (fx * 4.0).sin() * (fz * 4.0).sin() * 5.0; // Large rolling hills

    height
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct ProceduralLevel;

/// A [`Resource`] that contains all the assets needed to spawn the procedural level.
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct ProceduralLevelAssets {
    #[dependency]
    pub(crate) ground_material: Handle<StandardMaterial>,
    #[dependency]
    pub(crate) music: Handle<Sample>,
    #[dependency]
    pub(crate) env_map_specular: Handle<Image>,
    #[dependency]
    pub(crate) env_map_diffuse: Handle<Image>,
}

impl FromWorld for ProceduralLevelAssets {
    fn from_world(world: &mut World) -> Self {
        // Get immutable reference to assets first
        let assets = world.resource::<AssetServer>();
        let music = assets.load("audio/music/Ambiance_Rain_Calm_Loop_Stereo.ogg");

        // Create placeholder material/handles until procedural generation completes
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let ground_material = materials.add(StandardMaterial::default());

        Self {
            ground_material,
            music,
            env_map_specular: Handle::default(),
            env_map_diffuse: Handle::default(),
        }
    }
}
