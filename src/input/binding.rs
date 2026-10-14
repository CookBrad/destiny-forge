use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Binding {
    Key(KeyCode),
    Mouse(MouseButton),
}

impl Binding {
    pub(super) fn is_pressed(
        self,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Self::Key(key) => keyboard.pressed(key),
            Self::Mouse(button) => mouse.pressed(button),
        }
    }

    pub(super) fn just_pressed(
        self,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Self::Key(key) => keyboard.just_pressed(key),
            Self::Mouse(button) => mouse.just_pressed(button),
        }
    }

    pub(super) fn just_released(
        self,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self {
            Self::Key(key) => keyboard.just_released(key),
            Self::Mouse(button) => mouse.just_released(button),
        }
    }
}
