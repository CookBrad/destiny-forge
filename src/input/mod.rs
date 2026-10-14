mod action;
mod binding;
mod map;
mod plugin;
mod state;
mod text;

#[cfg(test)]
use bevy::prelude::*;

pub use action::Action;
pub use plugin::InputPlugin;
pub use state::ActionState;

#[cfg(test)]
pub use map::InputMap;

#[cfg(test)]
pub fn action_state_after_pressing(keys: &[KeyCode], buttons: &[MouseButton]) -> ActionState {
    let mut keyboard = ButtonInput::default();
    for key in keys {
        keyboard.press(*key);
    }
    let mut mouse = ButtonInput::default();
    for button in buttons {
        mouse.press(*button);
    }
    ActionState::from_devices(&InputMap::default(), &keyboard, &mouse)
}

#[cfg(test)]
mod tests;
