//! A loading screen during which procedural assets are prepared.

use bevy::{asset::LoadState, prelude::*, tasks::{IoTaskPool, Task}};
use futures_lite::future;

use crate::{
    gameplay::procedural_level::{spawn_procedural_level, ProceduralLevelAssets},
    generate::generate_ground::generate_ground_texture,
    menus::{generate::GenerationPrompt, Menu},
    screens::Screen,
    theme::{palette::SCREEN_BACKGROUND, prelude::*},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GroundGenerationProgress>()
        .add_systems(
            OnEnter(Screen::ProceduralLoading),
            (spawn_procedural_loading_screen, start_ground_generation),
        )
        .add_systems(
            Update,
            (track_ground_generation_tasks, advance_to_procedural_gameplay_screen)
                .run_if(in_state(Screen::ProceduralLoading)),
        )
        .add_systems(OnEnter(Screen::ProceduralGameplay), spawn_procedural_level)
        .add_systems(OnExit(Screen::ProceduralLoading), reset_ground_generation_progress);
}

fn spawn_procedural_loading_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Loading Screen"),
        BackgroundColor(SCREEN_BACKGROUND),
        StateScoped(Screen::ProceduralLoading),
        children![widget::label("Generating Procedural World...")],
    ));
}

fn advance_to_procedural_gameplay_screen(
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    progress: Res<GroundGenerationProgress>,
    procedural_assets: Option<Res<ProceduralLevelAssets>>,
    asset_server: Res<AssetServer>,
) {
    match &progress.status {
        GroundGenerationStatus::Failed(reason) => {
            warn!("ground generation failed: {reason}");
            next_screen.set(Screen::Title);
            next_menu.set(Menu::Main);
        }
        GroundGenerationStatus::Succeeded { material, texture, .. } => {
            // Wait for procedural assets to be loaded
            if let Some(assets) = procedural_assets {
                let music_state = asset_server.get_load_state(assets.music.id());
                let env_specular_state = asset_server.get_load_state(assets.env_map_specular.id());
                let env_diffuse_state = asset_server.get_load_state(assets.env_map_diffuse.id());
                let material_state = asset_server.get_load_state(material.id());
                let texture_state = asset_server.get_load_state(texture.id());

                let states = [
                    music_state,
                    env_specular_state,
                    env_diffuse_state,
                    material_state,
                    texture_state,
                ];

                if states
                    .iter()
                    .any(|state| matches!(state, Some(LoadState::Failed(_))))
                {
                    error!("generated texture failed to load");
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

fn start_ground_generation(
    mut commands: Commands,
    prompt: Res<GenerationPrompt>,
    mut progress: ResMut<GroundGenerationProgress>,
    existing_tasks: Query<Entity, With<GroundGenerationTask>>,
) {
    for entity in &existing_tasks {
        commands.entity(entity).despawn();
    }

    let prompt_text = prompt.0.clone();
    progress.status = GroundGenerationStatus::InProgress;

    info!("starting ground generation for prompt: {}", prompt_text);

    let task = IoTaskPool::get().spawn(async move { generate_ground_texture(prompt_text) });

    commands.spawn(GroundGenerationTask(task));
}

fn track_ground_generation_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut GroundGenerationTask)>,
    mut progress: ResMut<GroundGenerationProgress>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut procedural_assets: ResMut<ProceduralLevelAssets>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            commands.entity(entity).despawn();

            match result {
                Ok(relative_path) => {
                    info!("ground texture generation completed");

                    let texture_handle: Handle<Image> = asset_server.load(relative_path.clone());
                    let material = materials.add(StandardMaterial {
                        base_color_texture: Some(texture_handle.clone()),
                        perceptual_roughness: 0.9,
                        metallic: 0.0,
                        ..default()
                    });

                    procedural_assets.ground_material = material.clone();
                    progress.status = GroundGenerationStatus::Succeeded {
                        path: relative_path,
                        material,
                        texture: texture_handle,
                    };
                }
                Err(err) => {
                    error!("failed to generate ground texture: {err:?}");
                    progress.status = GroundGenerationStatus::Failed(err.to_string());
                }
            }
        }
    }
}

#[derive(Component)]
struct GroundGenerationTask(Task<anyhow::Result<String>>);

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct GroundGenerationProgress {
    pub status: GroundGenerationStatus,
}

impl Default for GroundGenerationProgress {
    fn default() -> Self {
        Self {
            status: GroundGenerationStatus::Idle,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundGenerationStatus {
    Idle,
    InProgress,
    Succeeded {
        path: String,
        material: Handle<StandardMaterial>,
        texture: Handle<Image>,
    },
    Failed(String),
}

fn reset_ground_generation_progress(mut progress: ResMut<GroundGenerationProgress>) {
    *progress = GroundGenerationProgress::default();
}
