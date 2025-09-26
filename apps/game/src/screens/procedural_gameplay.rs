//! The screen state for procedural gameplay.

use anyhow::Result as AnyhowResult;
use avian_pickup::prop::PreferredPickupRotation;
use avian3d::prelude::*;
use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    tasks::{IoTaskPool, Task},
    ui::Val::*,
};
use bevy_simple_text_input::{TextInput, TextInputSubmitEvent, TextInputSystem};
use futures_lite::future;
use std::any::TypeId;

use crate::{
    Pause,
    gameplay::{crosshair::CrosshairState, player::Player},
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
) {
    let mut crosshair = crosshair.map(Single::into_inner);

    if !ui_state.is_open() && keys.just_pressed(KeyCode::KeyM) {
        open_model_prompt_ui(
            &mut commands,
            &mut ui_state,
            crosshair.as_deref_mut(),
            paused.get().0,
            &mut next_pause,
        );
    } else if ui_state.is_open() && keys.just_pressed(KeyCode::Escape) {
        close_model_prompt_ui(
            &mut commands,
            &mut ui_state,
            crosshair.as_deref_mut(),
            &mut next_pause,
        );
    }
}

fn submit_model_prompt(
    mut events: EventReader<TextInputSubmitEvent>,
    mut commands: Commands,
    mut ui_state: ResMut<ModelPromptUiState>,
    crosshair: Option<Single<&mut CrosshairState>>,
    mut next_pause: ResMut<NextState<Pause>>,
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
        let task = IoTaskPool::get().spawn({
            let prompt = task_prompt.clone();
            async move { generate_prop(prompt) }
        });
        commands.spawn(ModelGenerationTask {
            prompt: task_prompt,
            task,
        });

        close_model_prompt_ui(
            &mut commands,
            &mut ui_state,
            crosshair.as_deref_mut(),
            &mut next_pause,
        );
        break;
    }
}

fn monitor_model_generation_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ModelGenerationTask)>,
    asset_server: Res<AssetServer>,
    players: Query<&GlobalTransform, With<Player>>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut task.task)) {
            let prompt = task.prompt.clone();
            commands.entity(entity).despawn();

            match result {
                Ok(paths) => {
                    spawn_generated_model(&mut commands, &asset_server, &players, &paths, &prompt)
                }
                Err(err) => error!(prompt, ?err, "Failed to generate Meshy model"),
            }
        }
    }
}

fn spawn_generated_model(
    commands: &mut Commands,
    asset_server: &AssetServer,
    players: &Query<&GlobalTransform, With<Player>>,
    paths: &GeneratedModelPaths,
    prompt: &str,
) {
    let scene_path = format!("{}#Scene0", paths.refined_path);
    let scene_handle: Handle<Scene> = asset_server.load(scene_path);

    let transform = players
        .iter()
        .next()
        .map(|player_transform| {
            let origin = player_transform.translation();
            let forward = player_transform.forward();
            Transform::from_translation(origin + forward * 2.0 + Vec3::Y)
        })
        .unwrap_or_else(|| Transform::from_xyz(0.0, 1.0, 0.0));

    let generation_folder = paths.refined_path.split('/').nth(2).unwrap_or("generated");

    commands.spawn((
        Name::new(format!("Generated Prop {generation_folder}")),
        SceneRoot(scene_handle.clone()),
        transform,
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
        GeneratedModel, // Mark as generated model
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
) {
    let mut crosshair = crosshair.map(Single::into_inner);
    close_model_prompt_ui(
        &mut commands,
        &mut ui_state,
        crosshair.as_deref_mut(),
        &mut next_pause,
    );
}

fn open_model_prompt_ui(
    commands: &mut Commands,
    ui_state: &mut ModelPromptUiState,
    crosshair: Option<&mut CrosshairState>,
    was_paused: bool,
    next_pause: &mut NextState<Pause>,
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

    if !was_paused {
        next_pause.set(Pause(true));
        ui_state.paused_game = true;
    }

    if let Some(crosshair) = crosshair {
        crosshair
            .wants_free_cursor
            .insert(TypeId::of::<ModelPromptOverlay>());
    }
}

fn close_model_prompt_ui(
    commands: &mut Commands,
    ui_state: &mut ModelPromptUiState,
    crosshair: Option<&mut CrosshairState>,
    next_pause: &mut NextState<Pause>,
) {
    if let Some(root) = ui_state.root.take() {
        commands.entity(root).despawn();
    }

    if ui_state.paused_game {
        next_pause.set(Pause(false));
        ui_state.paused_game = false;
    }

    if let Some(crosshair) = crosshair {
        crosshair
            .wants_free_cursor
            .remove(&TypeId::of::<ModelPromptOverlay>());
    }
}

#[derive(Resource, Default)]
struct ModelPromptUiState {
    root: Option<Entity>,
    paused_game: bool,
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
}

/// Marker component for generated models that need collision setup
#[derive(Component)]
struct GeneratedModel;
