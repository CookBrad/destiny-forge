use std::collections::HashSet;

use bevy::prelude::*;

use super::action::Action;
use super::binding::Binding;
use super::map::InputMap;
use super::text::character_for_key;

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct ActionState {
    pressed: HashSet<Action>,
    just_pressed: HashSet<Action>,
    just_released: HashSet<Action>,
    just_pressed_in_binding_order: Vec<Action>,
    typed: String,
}

impl ActionState {
    pub fn from_devices(
        map: &InputMap,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> Self {
        let mut state = Self::default();
        state.gather_bindings(map, keyboard, mouse);
        state.gather_typed_characters(keyboard);
        state
    }

    pub fn pressed(&self, action: Action) -> bool {
        self.pressed.contains(&action)
    }

    pub fn just_pressed(&self, action: Action) -> bool {
        self.just_pressed.contains(&action)
    }

    pub fn just_released(&self, action: Action) -> bool {
        self.just_released.contains(&action)
    }

    pub fn just_pressed_actions(&self) -> &[Action] {
        &self.just_pressed_in_binding_order
    }

    pub fn typed_characters(&self) -> &str {
        &self.typed
    }

    fn gather_bindings(
        &mut self,
        map: &InputMap,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) {
        for (action, binding) in map.bindings() {
            self.record_binding(*action, *binding, keyboard, mouse);
        }
    }

    fn record_binding(
        &mut self,
        action: Action,
        binding: Binding,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) {
        if binding.is_pressed(keyboard, mouse) {
            self.pressed.insert(action);
        }
        if binding.just_pressed(keyboard, mouse) {
            self.just_pressed.insert(action);
            self.just_pressed_in_binding_order.push(action);
        }
        if binding.just_released(keyboard, mouse) {
            self.just_released.insert(action);
        }
    }

    fn gather_typed_characters(&mut self, keyboard: &ButtonInput<KeyCode>) {
        for key in keyboard.get_just_pressed() {
            if let Some(ch) = character_for_key(*key) {
                self.typed.push(ch);
            }
        }
    }
}
