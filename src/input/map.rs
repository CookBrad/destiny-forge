use bevy::prelude::*;

use super::action::Action;
use super::binding::Binding;

#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct InputMap {
    bindings: Vec<(Action, Binding)>,
}

impl Default for InputMap {
    fn default() -> Self {
        Self {
            bindings: default_bindings(),
        }
    }
}

impl InputMap {
    pub fn bindings(&self) -> &[(Action, Binding)] {
        &self.bindings
    }
}

fn default_bindings() -> Vec<(Action, Binding)> {
    let mut bindings = Vec::new();
    bindings.extend(movement_bindings());
    bindings.extend(gameplay_bindings());
    bindings.extend(menu_bindings());
    bindings.extend(tool_bindings());
    bindings.extend(profile_bindings());
    bindings.extend(skill_slot_bindings());
    bindings
}

fn movement_bindings() -> [(Action, Binding); 8] {
    [
        (Action::MoveUp, Binding::Key(KeyCode::KeyW)),
        (Action::MoveUp, Binding::Key(KeyCode::ArrowUp)),
        (Action::MoveDown, Binding::Key(KeyCode::KeyS)),
        (Action::MoveDown, Binding::Key(KeyCode::ArrowDown)),
        (Action::MoveLeft, Binding::Key(KeyCode::KeyA)),
        (Action::MoveLeft, Binding::Key(KeyCode::ArrowLeft)),
        (Action::MoveRight, Binding::Key(KeyCode::KeyD)),
        (Action::MoveRight, Binding::Key(KeyCode::ArrowRight)),
    ]
}

fn gameplay_bindings() -> [(Action, Binding); 6] {
    [
        (Action::Jump, Binding::Key(KeyCode::Space)),
        (Action::Interact, Binding::Key(KeyCode::KeyE)),
        (Action::UseTool, Binding::Key(KeyCode::Space)),
        (Action::UseTool, Binding::Mouse(MouseButton::Left)),
        (Action::PrimaryButton, Binding::Mouse(MouseButton::Left)),
        (Action::Craft, Binding::Key(KeyCode::KeyF)),
    ]
}

fn menu_bindings() -> [(Action, Binding); 9] {
    [
        (Action::Cancel, Binding::Key(KeyCode::Escape)),
        (Action::Confirm, Binding::Key(KeyCode::Enter)),
        (Action::ToggleInventory, Binding::Key(KeyCode::KeyI)),
        (Action::ReturnToHomestead, Binding::Key(KeyCode::KeyQ)),
        (Action::EraseCharacter, Binding::Key(KeyCode::Backspace)),
        (
            Action::EquipFirstStashedWeapon,
            Binding::Key(KeyCode::BracketRight),
        ),
        (
            Action::EquipLastStashedWeapon,
            Binding::Key(KeyCode::BracketLeft),
        ),
        (Action::PreviousRecipe, Binding::Key(KeyCode::ArrowUp)),
        (Action::NextRecipe, Binding::Key(KeyCode::ArrowDown)),
    ]
}

fn tool_bindings() -> [(Action, Binding); 8] {
    [
        (Action::EquipHoe, Binding::Key(KeyCode::Digit1)),
        (Action::EquipWateringCan, Binding::Key(KeyCode::Digit2)),
        (Action::EquipSeeds, Binding::Key(KeyCode::Digit3)),
        (Action::EquipHand, Binding::Key(KeyCode::Digit4)),
        (Action::EquipHoe, Binding::Key(KeyCode::Numpad1)),
        (Action::EquipWateringCan, Binding::Key(KeyCode::Numpad2)),
        (Action::EquipSeeds, Binding::Key(KeyCode::Numpad3)),
        (Action::EquipHand, Binding::Key(KeyCode::Numpad4)),
    ]
}

fn profile_bindings() -> [(Action, Binding); 3] {
    [
        (Action::SelectProfile(0), Binding::Key(KeyCode::Digit1)),
        (Action::SelectProfile(1), Binding::Key(KeyCode::Digit2)),
        (Action::SelectProfile(2), Binding::Key(KeyCode::Digit3)),
    ]
}

fn skill_slot_bindings() -> [(Action, Binding); 9] {
    [
        (Action::SkillSlot(0), Binding::Key(KeyCode::Digit1)),
        (Action::SkillSlot(1), Binding::Key(KeyCode::Digit2)),
        (Action::SkillSlot(2), Binding::Key(KeyCode::Digit3)),
        (Action::SkillSlot(3), Binding::Key(KeyCode::Digit4)),
        (Action::SkillSlot(4), Binding::Key(KeyCode::Digit5)),
        (Action::SkillSlot(5), Binding::Key(KeyCode::Digit6)),
        (Action::SkillSlot(6), Binding::Key(KeyCode::Digit7)),
        (Action::SkillSlot(7), Binding::Key(KeyCode::Digit8)),
        (Action::SkillSlot(8), Binding::Key(KeyCode::Digit9)),
    ]
}
