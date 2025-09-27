//! The screen state for procedural gameplay.

use anyhow::Result as AnyhowResult;
use avian_pickup::prop::PreferredPickupRotation;
use avian3d::prelude::*;
use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::{
        mesh::{Mesh, MeshAabb},
        view::RenderLayers,
    },
    tasks::{IoTaskPool, Task},
    ui::Val::*,
};
use bevy_hanabi::prelude::*;
use bevy_simple_text_input::{TextInput, TextInputSubmitEvent, TextInputSystem};
use futures_lite::future;
use std::{any::TypeId, collections::VecDeque};

use crate::{
    Pause, RenderLayer,
    gameplay::{
        crosshair::CrosshairState,
        player::{Player, default_input::BlocksInput},
        procedural_level::sample_terrain_height,
    },
    generate::generate_model::{GeneratedModelPaths, generate_prop},
    menus::Menu,
    screens::Screen,
    theme::{palette::SCREEN_BACKGROUND, widget},
    third_party::avian3d::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ModelPromptUiState>();

    // Toggle pause on key press.
    app.add_systems(
        Update,
        (
            (pause, spawn_pause_overlay, open_pause_menu).run_if(
                in_state(Screen::ProceduralGameplay)
                    .and(in_state(Menu::None))
                    .and(model_prompt_closed)
                    .and(input_just_pressed(KeyCode::KeyP).or(input_just_pressed(KeyCode::Escape))),
            ),
            close_menu.run_if(
                in_state(Screen::ProceduralGameplay)
                    .and(not(in_state(Menu::None)))
                    .and(input_just_pressed(KeyCode::KeyP)),
            ),
        ),
    );

    app.add_systems(
        Update,
        (
            toggle_model_prompt,
            submit_model_prompt.after(TextInputSystem),
            monitor_model_generation_tasks,
            adjust_generated_prop_height.after(monitor_model_generation_tasks),
        )
            .run_if(in_state(Screen::ProceduralGameplay)),
    );

    app.add_systems(
        OnExit(Screen::ProceduralGameplay),
        (close_menu, unpause, cleanup_model_prompt),
    );
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(Screen::ProceduralGameplay)),
    );
}

fn model_prompt_closed(state: Res<ModelPromptUiState>) -> bool {
    !state.is_open()
}

fn unpause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause(false));
}

fn pause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause(true));
}

fn spawn_pause_overlay(mut commands: Commands) {
    commands.spawn((
        Name::new("Pause Overlay"),
        Node {
            width: Percent(100.0),
            height: Percent(100.0),
            ..default()
        },
        GlobalZIndex(1),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        StateScoped(Pause(true)),
    ));
}

fn open_pause_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Pause);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn toggle_model_prompt(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut ui_state: ResMut<ModelPromptUiState>,
    crosshair: Option<Single<&mut CrosshairState>>,
    paused: Res<State<Pause>>,
    mut next_pause: ResMut<NextState<Pause>>,
    mut blocks_input: ResMut<BlocksInput>,
) {
    let mut crosshair = crosshair.map(Single::into_inner);

    if !ui_state.is_open() && keys.just_pressed(KeyCode::KeyM) {
        open_model_prompt_ui(
            &mut commands,
            &mut ui_state,
            crosshair.as_deref_mut(),
            paused.get().0,
            &mut next_pause,
            &mut blocks_input,
        );
    } else if ui_state.is_open() && keys.just_pressed(KeyCode::Escape) {
        close_model_prompt_ui(
            &mut commands,
            &mut ui_state,
            crosshair.as_deref_mut(),
            &mut next_pause,
            &mut blocks_input,
        );
    }
}

fn submit_model_prompt(
    mut events: EventReader<TextInputSubmitEvent>,
    mut commands: Commands,
    mut ui_state: ResMut<ModelPromptUiState>,
    crosshair: Option<Single<&mut CrosshairState>>,
    mut next_pause: ResMut<NextState<Pause>>,
    mut blocks_input: ResMut<BlocksInput>,
    mut effects: ResMut<Assets<EffectAsset>>,
    players: Query<&GlobalTransform, With<Player>>,
) {
    if !ui_state.is_open() {
        for _ in events.read() {}
        return;
    }

    let mut crosshair = crosshair.map(Single::into_inner);

    for event in events.read() {
        let prompt = event.value.trim();
        if prompt.is_empty() {
            continue;
        }

        info!(prompt, "Starting in-game Meshy generation task");

        let task_prompt = prompt.to_string();
        let spawn_location = predicted_spawn_location(&players);
        let placeholder = spawn_placeholder(
            &mut commands,
            &mut effects,
            spawn_location.transform.clone(),
        );

        let task_location = spawn_location.clone();
        let task = IoTaskPool::get().spawn({
            let prompt = task_prompt.clone();
            async move { generate_prop(prompt) }
        });
        commands.spawn(ModelGenerationTask {
            prompt: task_prompt,
            task,
            placeholder,
            spawn_location: task_location,
        });

        close_model_prompt_ui(
            &mut commands,
            &mut ui_state,
            crosshair.as_deref_mut(),
            &mut next_pause,
            &mut blocks_input,
        );
        break;
    }
}

fn monitor_model_generation_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ModelGenerationTask)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut task.task)) {
            let prompt = task.prompt.clone();
            let spawn_location = task.spawn_location.clone();
            commands.entity(task.placeholder).despawn();
            commands.entity(entity).despawn();

            match result {
                Ok(paths) => spawn_generated_model(
                    &mut commands,
                    &asset_server,
                    spawn_location,
                    &paths,
                    &prompt,
                ),
                Err(err) => error!(prompt, ?err, "Failed to generate Meshy model"),
            }
        }
    }
}

fn spawn_generated_model(
    commands: &mut Commands,
    asset_server: &AssetServer,
    spawn_location: SpawnLocation,
    paths: &GeneratedModelPaths,
    prompt: &str,
) {
    let scene_path = format!("{}#Scene0", paths.refined_path);
    let scene_handle: Handle<Scene> = asset_server.load(scene_path);

    let generation_folder = paths.refined_path.split('/').nth(2).unwrap_or("generated");

    commands.spawn((
        Name::new(format!("Generated Prop {generation_folder}")),
        SceneRoot(scene_handle),
        spawn_location.transform,
        GlobalTransform::default(),
        // Add physics components to make the model collidable and pickupable
        RigidBody::Dynamic, // Dynamic so it can be picked up and moved
        // Use convex hull collision with proper layers for pickup interaction
        ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh)
            .with_default_layers(CollisionLayers::new(CollisionLayer::Prop, LayerMask::ALL))
            .with_default_density(500.0), // Reasonable weight for generated objects
        // Enable pickup interaction
        PreferredPickupRotation(Quat::IDENTITY), // Keep upright when picked up
        StateScoped(Screen::ProceduralGameplay),
        GeneratedPropRoot {
            ground_height: spawn_location.ground_height,
            adjusted: false,
        },
    ));

    info!(
        prompt,
        refined = %paths.refined_path,
        preview = %paths.preview_path,
        "Spawned Meshy-generated prop in procedural gameplay"
    );
}

fn cleanup_model_prompt(
    mut commands: Commands,
    mut ui_state: ResMut<ModelPromptUiState>,
    crosshair: Option<Single<&mut CrosshairState>>,
    mut next_pause: ResMut<NextState<Pause>>,
    mut blocks_input: ResMut<BlocksInput>,
) {
    let mut crosshair = crosshair.map(Single::into_inner);
    close_model_prompt_ui(
        &mut commands,
        &mut ui_state,
        crosshair.as_deref_mut(),
        &mut next_pause,
        &mut blocks_input,
    );
}

fn open_model_prompt_ui(
    commands: &mut Commands,
    ui_state: &mut ModelPromptUiState,
    crosshair: Option<&mut CrosshairState>,
    was_paused: bool,
    next_pause: &mut NextState<Pause>,
    blocks_input: &mut BlocksInput,
) {
    if ui_state.is_open() {
        return;
    }

    let root = commands
        .spawn((
            widget::ui_root("In-Game Model Prompt"),
            BackgroundColor(SCREEN_BACKGROUND.with_alpha(0.9)),
            GlobalZIndex(3),
            StateScoped(Screen::ProceduralGameplay),
            ModelPromptOverlay,
            children![
                widget::header("Generate Prop"),
                widget::label("Describe the object to generate"),
                (
                    Name::new("Model Prompt Input"),
                    Node {
                        width: Px(520.0),
                        ..default()
                    },
                    children![(TextInput, ModelPromptInput)],
                ),
                widget::label_small("Press Enter to submit. Press M or Esc to close."),
            ],
        ))
        .id();

    ui_state.root = Some(root);
    ui_state.paused_game = false;
    ui_state.blocked_input = false;

    if !was_paused {
        next_pause.set(Pause(true));
        ui_state.paused_game = true;
    }

    let overlay_id = TypeId::of::<ModelPromptOverlay>();

    if blocks_input.insert(overlay_id) {
        ui_state.blocked_input = true;
    }

    if let Some(crosshair) = crosshair {
        crosshair.wants_free_cursor.insert(overlay_id);
    }
}

fn close_model_prompt_ui(
    commands: &mut Commands,
    ui_state: &mut ModelPromptUiState,
    crosshair: Option<&mut CrosshairState>,
    next_pause: &mut NextState<Pause>,
    blocks_input: &mut BlocksInput,
) {
    if let Some(root) = ui_state.root.take() {
        commands.entity(root).despawn();
    }

    if ui_state.paused_game {
        next_pause.set(Pause(false));
        ui_state.paused_game = false;
    }

    if ui_state.blocked_input {
        let overlay_id = TypeId::of::<ModelPromptOverlay>();
        blocks_input.remove(&overlay_id);
        ui_state.blocked_input = false;
    }

    if let Some(crosshair) = crosshair {
        let overlay_id = TypeId::of::<ModelPromptOverlay>();
        crosshair.wants_free_cursor.remove(&overlay_id);
    }
}

#[derive(Resource, Default)]
struct ModelPromptUiState {
    root: Option<Entity>,
    paused_game: bool,
    blocked_input: bool,
}

impl ModelPromptUiState {
    fn is_open(&self) -> bool {
        self.root.is_some()
    }
}

#[derive(Component)]
struct ModelPromptOverlay;

#[derive(Component)]
struct ModelPromptInput;

#[derive(Component)]
struct ModelGenerationTask {
    prompt: String,
    task: Task<AnyhowResult<GeneratedModelPaths>>,
    placeholder: Entity,
    spawn_location: SpawnLocation,
}

#[derive(Clone)]
struct SpawnLocation {
    transform: Transform,
    ground_height: f32,
}

fn predicted_spawn_location(players: &Query<&GlobalTransform, With<Player>>) -> SpawnLocation {
    let target_position = players
        .iter()
        .next()
        .map(|player_transform| {
            let origin = player_transform.translation();
            let forward = player_transform.forward();
            Vec3::new(origin.x + forward.x * 2.0, 0.0, origin.z + forward.z * 2.0)
        })
        .unwrap_or(Vec3::ZERO);

    let ground_height = sample_terrain_height(target_position.x, target_position.z);
    SpawnLocation {
        transform: Transform::from_translation(Vec3::new(
            target_position.x,
            ground_height,
            target_position.z,
        )),
        ground_height,
    }
}

#[derive(Component)]
struct GeneratedPropRoot {
    ground_height: f32,
    adjusted: bool,
}

fn adjust_generated_prop_height(
    mut roots: Query<(&mut Transform, &Children, &mut GeneratedPropRoot)>,
    children_query: Query<&Children>,
    mesh_query: Query<(&GlobalTransform, &Mesh3d)>,
    meshes: Res<Assets<Mesh>>,
) {
    for (mut transform, children, mut prop) in roots.iter_mut() {
        if prop.adjusted {
            continue;
        }

        let mut stack: VecDeque<Entity> = VecDeque::new();
        for child in children.iter() {
            stack.push_back(child);
        }
        let mut min_y: Option<f32> = None;

        while let Some(entity) = stack.pop_front() {
            if let Ok((global_transform, mesh)) = mesh_query.get(entity) {
                if let Some(mesh_asset) = meshes.get(&mesh.0) {
                    if let Some(aabb) = mesh_asset.compute_aabb() {
                        let center: Vec3 = aabb.center.into();
                        let half: Vec3 = aabb.half_extents.into();
                        for sx in [-1.0, 1.0] {
                            for sy in [-1.0, 1.0] {
                                for sz in [-1.0, 1.0] {
                                    let local =
                                        center + Vec3::new(sx * half.x, sy * half.y, sz * half.z);
                                    let world = global_transform.transform_point(local);
                                    min_y = Some(match min_y {
                                        Some(current) => current.min(world.y),
                                        None => world.y,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    stack.push_back(child);
                }
            }
        }

        if let Some(min_y) = min_y {
            let delta = prop.ground_height - min_y;
            transform.translation.y += delta;
            prop.adjusted = true;
        }
    }
}

fn spawn_placeholder(
    commands: &mut Commands,
    effects: &mut Assets<EffectAsset>,
    transform: Transform,
) -> Entity {
    let writer = ExprWriter::new();
    let mean_velocity = writer.lit(Vec3::new(0.0, 2.5, 0.0));
    let sd_velocity = writer.lit(Vec3::new(0.3, 1.2, 0.3));
    let velocity = SetAttributeModifier::new(
        Attribute::VELOCITY,
        mean_velocity.normal(sd_velocity).expr(),
    );

    let mut module = writer.finish();

    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::new(0.0, 0.5, 0.0)),
        radius: module.lit(0.6),
        dimension: ShapeDimension::Volume,
    };

    let lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(1.2));
    let accel = module.lit(Vec3::new(0.0, 0.8, 0.0));
    let update_accel = AccelModifier::new(accel);

    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(0.2, 0.5, 0.9, 0.0));
    color_gradient.add_key(0.2, Vec4::new(0.35, 0.75, 1.0, 0.55));
    color_gradient.add_key(0.7, Vec4::new(0.65, 0.9, 1.0, 0.4));
    color_gradient.add_key(1.0, Vec4::new(0.9, 1.0, 1.0, 0.0));
    let color_over_lifetime = ColorOverLifetimeModifier {
        gradient: color_gradient,
        ..default()
    };

    let mut size_curve = Gradient::new();
    size_curve.add_key(0.0, Vec3::splat(0.2));
    size_curve.add_key(0.4, Vec3::splat(0.45));
    size_curve.add_key(1.0, Vec3::splat(0.1));
    let size_over_lifetime = SizeOverLifetimeModifier {
        gradient: size_curve,
        screen_space_size: false,
    };

    const MAX_PARTICLES: u32 = 4096;
    let effect = EffectAsset::new(MAX_PARTICLES, SpawnerSettings::rate(220.0.into()), module)
        .with_name("meshy_transporter_placeholder")
        .init(init_pos)
        .init(velocity)
        .init(lifetime)
        .update(update_accel)
        .render(color_over_lifetime)
        .render(size_over_lifetime);

    let effect_handle = effects.add(effect);

    commands
        .spawn((
            Name::new("Meshy Transporter Placeholder"),
            transform,
            GlobalTransform::default(),
            ParticleEffect::new(effect_handle),
            RenderLayers::from(RenderLayer::PARTICLES),
            StateScoped(Screen::ProceduralGameplay),
        ))
        .id()
}

/// Marker component for generated models that need collision setup
#[derive(Component)]
struct GeneratedModel;
