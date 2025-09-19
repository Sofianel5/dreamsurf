//! Procedural level generation.

use crate::{
    audio::MusicPool,
    screens::Screen,
    gameplay::npc::NPC_RADIUS,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_landmass::{prelude::*};
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
pub(crate) fn spawn_procedural_level(mut commands: Commands, assets: Res<ProceduralLevelAssets>) {
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
    spawn_ground(&mut commands, &assets);

    // Generate multiple houses in random locations
    spawn_multiple_houses(&mut commands, &assets);

    // Spawn player
    spawn_procedural_player(&mut commands);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn spawn_ground(commands: &mut Commands, assets: &ProceduralLevelAssets) {
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(assets.ground_mesh.clone()),
        MeshMaterial3d(assets.ground_material.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(1000.0, 0.1, 1000.0), // Very large but thin collision box
        StateScoped(Screen::ProceduralGameplay),
    ));
}

#[cfg_attr(feature = "hot_patch", hot)]
fn spawn_multiple_houses(commands: &mut Commands, assets: &ProceduralLevelAssets) {
    // Define the area where houses can spawn (avoiding the edges)
    let spawn_area = 80.0; // Keep houses within ±80 units from center (ground is 200x200)
    let min_distance = 30.0; // Minimum distance between houses

    // List of house positions
    let house_positions = vec![
        Vec3::new(0.0, 0.0, 0.0),        // Center house
        Vec3::new(40.0, 0.0, 30.0),      // Northeast
        Vec3::new(-45.0, 0.0, -20.0),    // Southwest
        Vec3::new(25.0, 0.0, -40.0),     // Southeast
        Vec3::new(-30.0, 0.0, 35.0),     // Northwest
        Vec3::new(60.0, 0.0, -10.0),     // East
        Vec3::new(-60.0, 0.0, 5.0),      // West
    ];

    // Spawn a house at each position
    for (i, position) in house_positions.iter().enumerate() {
        spawn_house_at_position(commands, assets, *position, i);
    }
}

#[cfg_attr(feature = "hot_patch", hot)]
fn spawn_house_at_position(commands: &mut Commands, assets: &ProceduralLevelAssets, house_center: Vec3, house_id: usize) {
    // House dimensions
    let width = 20.0;
    let depth = 15.0;
    let height = 3.0;
    let wall_thickness = 1.0;
    let door_width = 2.0;

    // Spawn all the visual walls (same as before)
    spawn_house_walls(commands, assets, house_center, house_id, width, depth, height, wall_thickness, door_width);

    // Single collision box for the entire house outline (hollow box)
    commands.spawn((
        Name::new(format!("House {} Collision", house_id)),
        Transform::from_xyz(house_center.x, house_center.y + height/2.0, house_center.z),
        RigidBody::Static,
        Collider::compound(vec![
            // Front wall left
            (Vec3::new(-width/2.0 + (width/2.0 - door_width/2.0)/2.0, 0.0, -depth/2.0), Quat::IDENTITY, Collider::cuboid((width/2.0 - door_width/2.0)/2.0, height/2.0, wall_thickness/2.0)),
            // Front wall right
            (Vec3::new(width/2.0 - (width/2.0 - door_width/2.0)/2.0, 0.0, -depth/2.0), Quat::IDENTITY, Collider::cuboid((width/2.0 - door_width/2.0)/2.0, height/2.0, wall_thickness/2.0)),
            // Back wall
            (Vec3::new(0.0, 0.0, depth/2.0), Quat::IDENTITY, Collider::cuboid(width/2.0, height/2.0, wall_thickness/2.0)),
            // Left wall
            (Vec3::new(-width/2.0, 0.0, 0.0), Quat::IDENTITY, Collider::cuboid(wall_thickness/2.0, height/2.0, depth/2.0)),
            // Right wall
            (Vec3::new(width/2.0, 0.0, 0.0), Quat::IDENTITY, Collider::cuboid(wall_thickness/2.0, height/2.0, depth/2.0)),
        ]),
        StateScoped(Screen::ProceduralGameplay),
    ));

    // Roof
    commands.spawn((
        Name::new(format!("House {} Roof", house_id)),
        Mesh3d(assets.roof_mesh.clone()),
        MeshMaterial3d(assets.roof_material.clone()),
        Transform::from_xyz(
            house_center.x,
            house_center.y + height + 0.5,
            house_center.z
        ).with_scale(Vec3::new(width + 2.0, 1.0, depth + 2.0)),
        RigidBody::Static,
        Collider::cuboid(0.5, 0.5, 0.5),
        StateScoped(Screen::ProceduralGameplay),
    ));
}

#[cfg_attr(feature = "hot_patch", hot)]
fn spawn_procedural_player(commands: &mut Commands) {
    // Spawn player entity at a good spawn position
    commands.spawn((
        Name::new("Procedural Player"),
        crate::gameplay::player::Player,
        Transform::from_xyz(-30.0, 2.0, 0.0), // Spawn outside the house
        Visibility::default(),
        StateScoped(Screen::ProceduralGameplay),
    ));
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct ProceduralLevel;

/// A [`Resource`] that contains all the assets needed to spawn the procedural level.
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct ProceduralLevelAssets {
    #[dependency]
    pub(crate) ground_mesh: Handle<Mesh>,
    #[dependency]
    pub(crate) wall_mesh: Handle<Mesh>,
    #[dependency]
    pub(crate) roof_mesh: Handle<Mesh>,
    #[dependency]
    pub(crate) ground_material: Handle<StandardMaterial>,
    #[dependency]
    pub(crate) wall_material: Handle<StandardMaterial>,
    #[dependency]
    pub(crate) roof_material: Handle<StandardMaterial>,
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
        let env_map_specular = assets.load("cubemaps/NightSkyHDRI001_4K-HDR_specular.ktx2");
        let env_map_diffuse = assets.load("cubemaps/NightSkyHDRI001_4K-HDR_diffuse.ktx2");

        // Load textures from darkmod assets (these exist in your project!)
        let grass_texture = assets.load("textures/darkmod/nature/dirt/dirt_002_dark.png");
        let stone_texture = assets.load("textures/darkmod/stone/cobblestones/blocks_uneven_blue.png");
        let wood_texture = assets.load("textures/darkmod/wood/boards/weathered.png");

        // Create procedural meshes with UV coordinates
        let mut meshes = world.resource_mut::<Assets<Mesh>>();

        // Very large ground plane with tiling UVs
        let mut ground_mesh = Plane3d::default().mesh().size(2000.0, 2000.0).build();
        // Scale UVs for tiling (200x200 tiles across the very large ground)
        if let Some(uvs) = ground_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
            if let bevy::render::mesh::VertexAttributeValues::Float32x2(uv_values) = uvs {
                for uv in uv_values.iter_mut() {
                    uv[0] *= 200.0; // Tile 200 times in U
                    uv[1] *= 200.0; // Tile 200 times in V
                }
            }
        }
        let ground_mesh = meshes.add(ground_mesh);

        let wall_mesh = meshes.add(Cuboid::default());
        let roof_mesh = meshes.add(Cuboid::default());
        drop(meshes); // Release the borrow

        // Create materials with textures
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();

        let ground_material = materials.add(StandardMaterial {
            base_color_texture: Some(grass_texture),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            ..default()
        });

        let wall_material = materials.add(StandardMaterial {
            base_color_texture: Some(stone_texture),
            perceptual_roughness: 0.95,
            metallic: 0.0,
            ..default()
        });

        let roof_material = materials.add(StandardMaterial {
            base_color_texture: Some(wood_texture),
            perceptual_roughness: 0.8,
            metallic: 0.0,
            ..default()
        });

        Self {
            ground_mesh,
            wall_mesh,
            roof_mesh,
            ground_material,
            wall_material,
            roof_material,
            music,
            env_map_specular,
            env_map_diffuse,
        }
    }
}

fn spawn_house_walls(commands: &mut Commands, assets: &ProceduralLevelAssets, house_center: Vec3, house_id: usize, width: f32, depth: f32, height: f32, wall_thickness: f32, door_width: f32) {
    // Front wall (with door opening) - Left part
    commands.spawn((
        Name::new(format!("House {} Front Wall Left", house_id)),
        Mesh3d(assets.wall_mesh.clone()),
        MeshMaterial3d(assets.wall_material.clone()),
        Transform::from_xyz(
            house_center.x - width/2.0 + (width/2.0 - door_width/2.0)/2.0,
            house_center.y + height/2.0,
            house_center.z - depth/2.0
        ).with_scale(Vec3::new(width/2.0 - door_width/2.0, height, wall_thickness)),
        StateScoped(Screen::ProceduralGameplay),
    ));

    // Front wall (with door opening) - Right part
    commands.spawn((
        Name::new(format!("House {} Front Wall Right", house_id)),
        Mesh3d(assets.wall_mesh.clone()),
        MeshMaterial3d(assets.wall_material.clone()),
        Transform::from_xyz(
            house_center.x + width/2.0 - (width/2.0 - door_width/2.0)/2.0,
            house_center.y + height/2.0,
            house_center.z - depth/2.0
        ).with_scale(Vec3::new(width/2.0 - door_width/2.0, height, wall_thickness)),
        StateScoped(Screen::ProceduralGameplay),
    ));

    // Back wall
    commands.spawn((
        Name::new(format!("House {} Back Wall", house_id)),
        Mesh3d(assets.wall_mesh.clone()),
        MeshMaterial3d(assets.wall_material.clone()),
        Transform::from_xyz(
            house_center.x,
            house_center.y + height/2.0,
            house_center.z + depth/2.0
        ).with_scale(Vec3::new(width, height, wall_thickness)),
        StateScoped(Screen::ProceduralGameplay),
    ));

    // Left wall
    commands.spawn((
        Name::new(format!("House {} Left Wall", house_id)),
        Mesh3d(assets.wall_mesh.clone()),
        MeshMaterial3d(assets.wall_material.clone()),
        Transform::from_xyz(
            house_center.x - width/2.0,
            house_center.y + height/2.0,
            house_center.z
        ).with_scale(Vec3::new(wall_thickness, height, depth)),
        StateScoped(Screen::ProceduralGameplay),
    ));

    // Right wall
    commands.spawn((
        Name::new(format!("House {} Right Wall", house_id)),
        Mesh3d(assets.wall_mesh.clone()),
        MeshMaterial3d(assets.wall_material.clone()),
        Transform::from_xyz(
            house_center.x + width/2.0,
            house_center.y + height/2.0,
            house_center.z
        ).with_scale(Vec3::new(wall_thickness, height, depth)),
        StateScoped(Screen::ProceduralGameplay),
    ));
}