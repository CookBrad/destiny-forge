use bevy::prelude::*;

use super::map::InputMap;
use super::state::ActionState;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputMap>()
            .init_resource::<ActionState>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(
                PreUpdate,
                sync_action_state.after(bevy::input::InputSystem),
            );
    }
}

fn sync_action_state(
    map: Res<InputMap>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut actions: ResMut<ActionState>,
) {
    *actions = ActionState::from_devices(&map, &keyboard, &mouse);
}
