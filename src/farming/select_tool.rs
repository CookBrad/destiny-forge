//! Equip homestead tools with keys 1–4.

use bevy::prelude::*;

use super::tools::{EquippedTool, HomesteadTool};

pub fn select_homestead_tool(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut equipped: ResMut<EquippedTool>,
) {
    for key in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Numpad1,
        KeyCode::Numpad2,
        KeyCode::Numpad3,
        KeyCode::Numpad4,
    ] {
        if !keyboard.just_pressed(key) {
            continue;
        }
        let Some(tool) = HomesteadTool::from_digit_key(key) else {
            continue;
        };
        if equipped.0 != tool {
            equipped.0 = tool;
            info!("Equipped {}.", tool.label());
        }
        return;
    }
}
