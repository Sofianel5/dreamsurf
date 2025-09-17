//! Input handling for the player.

use std::any::TypeId;

use bevy::{platform::collections::HashSet, prelude::*};
use bevy_enhanced_input::prelude::*;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

use super::Player;

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<DefaultInputContext>();
    app.add_observer(bind_default_inputs);

    app.init_resource::<BlocksInput>();
    app.register_type::<BlocksInput>();
    app.add_systems(
        PreUpdate,
        update_player_input_binding.run_if(resource_changed::<BlocksInput>),
    );
}

#[derive(Debug, InputAction)]
#[action_output(Vec3)]
pub(crate) struct Move;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(crate) struct Jump;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(crate) struct Interact;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub(crate) struct Rotate;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(crate) struct PickupProp;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub(crate) struct DropProp;

#[derive(Debug, Component, Default)]
pub(crate) struct DefaultInputContext;

#[derive(Resource, Default, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub(crate) struct BlocksInput(HashSet<TypeId>);

fn bind_default_inputs(trigger: Trigger<OnAdd, DefaultInputContext>, mut commands: Commands) {
    const DEFAULT_SENSITIVITY: f32 = 0.002;
    const DEFAULT_SPEED: f32 = 8.0;
    commands
        .entity(trigger.target())
        .insert(actions!(DefaultInputContext[
            (
                Action::<Move>::new(), DeadZone::default(),
                SmoothNudge::default(),
                Scale::splat(DEFAULT_SPEED),
                Negate::y(),
                SwizzleAxis::XZY,
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Axial::left_stick()
                ))
            ),
            (Action::<Jump>::new(), bindings![KeyCode::Space, GamepadButton::South]),
            (Action::<Interact>::new(), bindings![KeyCode::KeyE, GamepadButton::South]),
            (Action::<Rotate>::new(),Negate::all(), Scale::splat(DEFAULT_SENSITIVITY),
                Bindings::spawn((Spawn(Binding::mouse_motion()), Axial::right_stick()))),
            (Action::<PickupProp>::new(), bindings![MouseButton::Left, GamepadButton::East]),
            (Action::<DropProp>::new(), bindings![MouseButton::Right, GamepadButton::East]),
        ]));
}

#[cfg_attr(feature = "hot_patch", hot)]
fn update_player_input_binding(
    player: Single<Entity, With<Player>>,
    blocks_input: Res<BlocksInput>,
    mut commands: Commands,
) {
    if blocks_input.is_empty() {
        commands.entity(*player).insert(DefaultInputContext);
    } else {
        commands
            .entity(*player)
            .remove_with_requires::<DefaultInputContext>()
            .despawn_related::<Actions<DefaultInputContext>>();
    }
}
