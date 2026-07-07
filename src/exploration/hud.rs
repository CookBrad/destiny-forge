use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;

pub const EXPLORATION_PROMPT_MOVE: &str = "WASD move  ·  I inventory  ·  Esc title";
pub const EXPLORATION_PROMPT_MOVE_INTERACT: &str =
    "WASD move  ·  I inventory  ·  E interact  ·  Esc title";

pub fn set_exploration_zone_label<F: QueryFilter>(
    zone_label: &mut Query<'_, '_, &mut Text, F>,
    label: &str,
) {
    if let Ok(mut text) = zone_label.get_single_mut() {
        text.0 = label.to_string();
    }
}

pub fn set_exploration_prompt<F: QueryFilter>(
    prompt_label: &mut Query<'_, '_, &mut Text, F>,
    prompt: &str,
) {
    if let Ok(mut text) = prompt_label.get_single_mut() {
        text.0 = prompt.to_string();
    }
}