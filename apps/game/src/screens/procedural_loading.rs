//! A loading screen during which procedural assets are prepared.

use bevy::prelude::*;

use crate::{
    gameplay::procedural_level::spawn_procedural_level,
    screens::Screen,
    theme::{palette::SCREEN_BACKGROUND, prelude::*},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::ProceduralLoading),
        (spawn_procedural_level, spawn_procedural_loading_screen),
    );
    app.add_systems(
        Update,
        advance_to_procedural_gameplay_screen.run_if(in_state(Screen::ProceduralLoading)),
    );
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
    procedural_assets: Option<Res<crate::gameplay::procedural_level::ProceduralLevelAssets>>,
    asset_server: Res<AssetServer>,
) {
    // Wait for procedural assets to be loaded
    if let Some(assets) = procedural_assets {
        let music_loaded = asset_server.is_loaded_with_dependencies(&assets.music);
        let env_specular_loaded = asset_server.is_loaded_with_dependencies(&assets.env_map_specular);
        let env_diffuse_loaded = asset_server.is_loaded_with_dependencies(&assets.env_map_diffuse);

        if music_loaded && env_specular_loaded && env_diffuse_loaded {
            next_screen.set(Screen::ProceduralGameplay);
        }
    }
}