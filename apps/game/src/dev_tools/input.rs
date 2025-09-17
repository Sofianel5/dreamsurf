//! Input for the dev tools.

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_dev_tools_input);
    app.add_input_context::<DevToolsInputContext>();
}

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(crate) struct ToggleDebugUi;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(crate) struct ForceFreeCursor;

#[derive(Debug, Component, Default)]
struct DevToolsInputContext;

#[cfg_attr(feature = "hot_patch", hot)]
fn setup_dev_tools_input(mut commands: Commands) {
    commands.spawn((
        Name::new("DevToolsInput"),
        DevToolsInputContext,
        actions!(DevToolsInputContext[
            (Action::<ToggleDebugUi>::new(), bindings![KeyCode::F3]),
            (Action::<ForceFreeCursor>::new(), bindings![KeyCode::Backquote]),
        ]),
    ));
}
