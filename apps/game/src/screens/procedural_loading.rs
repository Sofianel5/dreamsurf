//! A loading screen during which procedural assets are prepared.

use bevy::{
    asset::LoadState,
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use futures_lite::future;

use crate::{
    gameplay::procedural_level::{spawn_procedural_level, ProceduralLevelAssets},
    generate::{generate_ground::generate_ground_texture, generate_sky::generate_sky_texture},
    menus::{generate::GenerationPrompt, Menu},
    screens::Screen,
    theme::{palette::SCREEN_BACKGROUND, prelude::*},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GenerationProgress>()
        .add_systems(
            OnEnter(Screen::ProceduralLoading),
            (spawn_procedural_loading_screen, start_generation_tasks),
        )
        .add_systems(
            Update,
            (monitor_generation_tasks, advance_to_procedural_gameplay_screen)
                .run_if(in_state(Screen::ProceduralLoading)),
        )
        .add_systems(OnEnter(Screen::ProceduralGameplay), spawn_procedural_level)
        .add_systems(OnExit(Screen::ProceduralLoading), reset_generation_progress);
}

fn spawn_procedural_loading_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Loading Screen"),
        BackgroundColor(SCREEN_BACKGROUND),
        StateScoped(Screen::ProceduralLoading),
        children![widget::label("Generating Procedural World...")],
    ));
}

fn start_generation_tasks(
    mut commands: Commands,
    prompt: Res<GenerationPrompt>,
    mut progress: ResMut<GenerationProgress>,
    existing_tasks: Query<Entity, With<GenerationTask>>,
) {
    for entity in &existing_tasks {
        commands.entity(entity).despawn();
    }

    let base_prompt = prompt.0.clone();

    progress.ground = GenerationStatus::InProgress;
    progress.sky = GenerationStatus::InProgress;

    info!("starting ground generation for prompt: {}", base_prompt);
    let ground_task = IoTaskPool::get().spawn({
        let prompt = base_prompt.clone();
        async move { generate_ground_texture(prompt) }
    });
    commands.spawn(GenerationTask::new(GenerationKind::Ground, ground_task));

    info!("starting sky generation for prompt: {}", base_prompt);
    let sky_task = IoTaskPool::get().spawn({
        let prompt = base_prompt.clone();
        async move { generate_sky_texture(prompt) }
    });
    commands.spawn(GenerationTask::new(GenerationKind::Sky, sky_task));
}

fn monitor_generation_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut GenerationTask)>,
    mut progress: ResMut<GenerationProgress>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut procedural_assets: ResMut<ProceduralLevelAssets>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut task.task)) {
            let kind = task.kind;
            commands.entity(entity).despawn();

            match (kind, result) {
                (GenerationKind::Ground, Ok(path)) => {
                    let texture: Handle<Image> = asset_server.load(path.clone());
                    let material = materials.add(StandardMaterial {
                        base_color_texture: Some(texture.clone()),
                        perceptual_roughness: 0.9,
                        metallic: 0.0,
                        ..default()
                    });

                    procedural_assets.ground_material = material.clone();
                    progress.ground = GenerationStatus::Succeeded(GeneratedGround {
                        material,
                        texture,
                    });
                }
                (GenerationKind::Sky, Ok(path)) => {
                    let texture: Handle<Image> = asset_server.load(path.clone());

                    procedural_assets.env_map_specular = texture.clone();
                    procedural_assets.env_map_diffuse = texture.clone();
                    info!(
                        "Sky generation succeeded; updated env map handles (specular: {:?})",
                        texture
                    );
                    progress.sky = GenerationStatus::Succeeded(GeneratedSky { texture });
                }
                (GenerationKind::Ground, Err(err)) => {
                    error!("failed to generate ground texture: {err:?}");
                    progress.ground = GenerationStatus::Failed(err.to_string());
                }
                (GenerationKind::Sky, Err(err)) => {
                    error!("failed to generate sky texture: {err:?}");
                    progress.sky = GenerationStatus::Failed(err.to_string());
                }
            }
        }
    }
}

fn advance_to_procedural_gameplay_screen(
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    progress: Res<GenerationProgress>,
    procedural_assets: Option<Res<ProceduralLevelAssets>>,
    asset_server: Res<AssetServer>,
) {
    match (&progress.ground, &progress.sky) {
        (GenerationStatus::Failed(reason), _) | (_, GenerationStatus::Failed(reason)) => {
            warn!("generation failed: {reason}");
            next_screen.set(Screen::Title);
            next_menu.set(Menu::Main);
        }
        (
            GenerationStatus::Succeeded(ground),
            GenerationStatus::Succeeded(sky),
        ) => {
            if let Some(assets) = procedural_assets {
                let states = [
                    asset_server.get_load_state(assets.music.id()),
                    asset_server.get_load_state(assets.env_map_specular.id()),
                    asset_server.get_load_state(assets.env_map_diffuse.id()),
                    asset_server.get_load_state(ground.material.id()),
                    asset_server.get_load_state(ground.texture.id()),
                    asset_server.get_load_state(sky.texture.id()),
                ];

                if states
                    .iter()
                    .any(|state| matches!(state, Some(LoadState::Failed(_))))
                {
                    error!("generated assets failed to load; returning to title");
                    next_screen.set(Screen::Title);
                    next_menu.set(Menu::Main);
                    return;
                }

                let all_loaded = states
                    .iter()
                    .all(|state| matches!(state, Some(LoadState::Loaded)) || state.is_none());

                if all_loaded {
                    next_screen.set(Screen::ProceduralGameplay);
                }
            }
        }
        _ => {}
    }
}

fn reset_generation_progress(mut progress: ResMut<GenerationProgress>) {
    *progress = GenerationProgress::default();
}

#[derive(Component)]
struct GenerationTask {
    kind: GenerationKind,
    task: Task<anyhow::Result<String>>,
}

impl GenerationTask {
    fn new(kind: GenerationKind, task: Task<anyhow::Result<String>>) -> Self {
        Self { kind, task }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum GenerationKind {
    Ground,
    Sky,
}

#[derive(Resource, Debug, Clone)]
struct GenerationProgress {
    ground: GenerationStatus<GeneratedGround>,
    sky: GenerationStatus<GeneratedSky>,
}

impl Default for GenerationProgress {
    fn default() -> Self {
        Self {
            ground: GenerationStatus::Pending,
            sky: GenerationStatus::Pending,
        }
    }
}

#[derive(Debug, Clone)]
struct GeneratedGround {
    material: Handle<StandardMaterial>,
    texture: Handle<Image>,
}

#[derive(Debug, Clone)]
struct GeneratedSky {
    texture: Handle<Image>,
}

#[derive(Debug, Clone)]
enum GenerationStatus<T> {
    Pending,
    InProgress,
    Succeeded(T),
    Failed(String),
}
