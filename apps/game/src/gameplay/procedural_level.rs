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

    // Generate the simple house
    spawn_simple_house(&mut commands, &assets);

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
        Collider::cuboid(100.0, 0.1, 100.0), // Flat collision for 200x200 plane
        StateScoped(Screen::ProceduralGameplay),
    ));
}

#[cfg_attr(feature = "hot_patch", hot)]
fn spawn_simple_house(commands: &mut Commands, assets: &ProceduralLevelAssets) {
    let house_center = Vec3::new(0.0, 0.0, 0.0);

    // House dimensions
    let width = 20.0;
    let depth = 15.0;
    let height = 3.0;
    let wall_thickness = 1.0;
    let door_width = 2.0;

    // Front wall (with door opening)
    // Left part of front wall
    commands.spawn((
        Name::new("House Front Wall Left"),
        Mesh3d(assets.wall_mesh.clone()),
        MeshMaterial3d(assets.wall_material.clone()),
        Transform::from_xyz(
            house_center.x - width/2.0 + (width/2.0 - door_width/2.0)/2.0,
            house_center.y + height/2.0,
            house_center.z - depth/2.0
        ).with_scale(Vec3::new(width/2.0 - door_width/2.0, height, wall_thickness)),
        RigidBody::Static,
        Collider::cuboid(0.5, 0.5, 0.5), // Use unit cube, scaling handles size
        StateScoped(Screen::ProceduralGameplay),
    ));

    // Right part of front wall
    commands.spawn((
        Name::new("House Front Wall Right"),
        Mesh3d(assets.wall_mesh.clone()),
        MeshMaterial3d(assets.wall_material.clone()),
        Transform::from_xyz(
            house_center.x + width/2.0 - (width/2.0 - door_width/2.0)/2.0,
            house_center.y + height/2.0,
            house_center.z - depth/2.0
        ).with_scale(Vec3::new(width/2.0 - door_width/2.0, height, wall_thickness)),
        RigidBody::Static,
        Collider::cuboid(0.5, 0.5, 0.5),
        StateScoped(Screen::ProceduralGameplay),
    ));

    // Back wall
    commands.spawn((
        Name::new("House Back Wall"),
        Mesh3d(assets.wall_mesh.clone()),
        MeshMaterial3d(assets.wall_material.clone()),
        Transform::from_xyz(
            house_center.x,
            house_center.y + height/2.0,
            house_center.z + depth/2.0
        ).with_scale(Vec3::new(width, height, wall_thickness)),
        RigidBody::Static,
        Collider::cuboid(0.5, 0.5, 0.5),
        StateScoped(Screen::ProceduralGameplay),
    ));

    // Left wall
    commands.spawn((
        Name::new("House Left Wall"),
        Mesh3d(assets.wall_mesh.clone()),
        MeshMaterial3d(assets.wall_material.clone()),
        Transform::from_xyz(
            house_center.x - width/2.0,
            house_center.y + height/2.0,
            house_center.z
        ).with_scale(Vec3::new(wall_thickness, height, depth)),
        RigidBody::Static,
        Collider::cuboid(0.5, 0.5, 0.5),
        StateScoped(Screen::ProceduralGameplay),
    ));

    // Right wall
    commands.spawn((
        Name::new("House Right Wall"),
        Mesh3d(assets.wall_mesh.clone()),
        MeshMaterial3d(assets.wall_material.clone()),
        Transform::from_xyz(
            house_center.x + width/2.0,
            house_center.y + height/2.0,
            house_center.z
        ).with_scale(Vec3::new(wall_thickness, height, depth)),
        RigidBody::Static,
        Collider::cuboid(0.5, 0.5, 0.5),
        StateScoped(Screen::ProceduralGameplay),
    ));

    // Roof
    commands.spawn((
        Name::new("House Roof"),
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

        // Create procedural meshes
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let ground_mesh = meshes.add(Plane3d::default().mesh().size(200.0, 200.0));
        let wall_mesh = meshes.add(Cuboid::default());
        let roof_mesh = meshes.add(Cuboid::default());
        drop(meshes); // Release the borrow

        // Create materials
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let ground_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.7, 0.2), // Green grass color
            perceptual_roughness: 0.8,
            ..default()
        });

        let wall_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.6, 0.5), // Stone color
            perceptual_roughness: 0.9,
            ..default()
        });

        let roof_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.3, 0.2), // Wood color
            perceptual_roughness: 0.7,
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