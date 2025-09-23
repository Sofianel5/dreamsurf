use crate::{
    menus::Menu,
    screens::Screen,
    theme::{palette::SCREEN_BACKGROUND, prelude::*},
};
use bevy::{prelude::*, window::CursorGrabMode};
use bevy_simple_text_input::{TextInput, TextInputPlugin, TextInputSubmitEvent, TextInputSystem};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GenerationPrompt>()
        .add_systems(OnEnter(Menu::Generate), spawn_generate_menu)
        .add_plugins(TextInputPlugin)
        .add_systems(Update, listener.after(TextInputSystem));
}

fn spawn_generate_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Generate Screen"),
        BackgroundColor(SCREEN_BACKGROUND),
        StateScoped(Menu::Generate),
        GlobalZIndex(2),
        children![widget::header("Generate World"), TextInput],
    ));
}

fn listener(
    mut events: EventReader<TextInputSubmitEvent>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut prompt: ResMut<GenerationPrompt>,
    mut window: Single<&mut Window>,
) {
    for event in events.read() {
        info!("{:?} submitted: {}", event.entity, event.value);
        prompt.0 = event.value.clone();
        next_screen.set(Screen::ProceduralLoading);
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

#[derive(Resource, Default)]
pub struct GenerationPrompt(pub String);
