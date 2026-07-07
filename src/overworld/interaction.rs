use bevy::prelude::*;

use crate::core::GameState;
use crate::ui::inventory_window::InventoryWindowOpen;
use crate::exploration::{
    set_exploration_prompt, set_exploration_zone_label, EXPLORATION_PROMPT_MOVE_INTERACT,
};
use crate::graphics::INTERACT_DISTANCE;

use super::layout::{homestead_forest_transition, HomesteadZone, OverworldLayout};
use super::movement::{MapTransitionCooldown, OverworldPlayer, OverworldVelocity};
use super::setup::{OverworldPromptLabel, OverworldZoneLabel};

pub fn overworld_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<InventoryWindowOpen>,
    layout: Res<OverworldLayout>,
    cooldown: Res<MapTransitionCooldown>,
    player: Query<(&Transform, &OverworldVelocity), With<OverworldPlayer>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut zone_label: Query<&mut Text, (With<OverworldZoneLabel>, Without<OverworldPromptLabel>)>,
    mut prompt_label: Query<&mut Text, (With<OverworldPromptLabel>, Without<OverworldZoneLabel>)>,
) {
    let Ok((transform, velocity)) = player.get_single() else {
        return;
    };

    if inventory.0 {
        return;
    }

    let position = transform.translation.truncate();
    let zone = layout.zone_at(position);

    set_exploration_zone_label(
        &mut zone_label,
        zone.map(|zone| zone.label)
            .unwrap_or("Homestead Yard"),
    );

    let mut prompt = EXPLORATION_PROMPT_MOVE_INTERACT;
    if let Some(zone) = zone {
        prompt = match zone.zone {
            HomesteadZone::House => "E — Enter house (soon)",
            HomesteadZone::Forge => "E — Open forge (soon)",
            HomesteadZone::Crops => "Crop plots ready for planting",
            HomesteadZone::Animals => "Feed and tend your livestock",
            HomesteadZone::ForestTrail => "Walk north up the trail to enter the woods",
            HomesteadZone::DungeonGate => {
                if distance_to_zone(position, &zone.bounds) <= INTERACT_DISTANCE * 2.0 {
                    if keyboard.just_pressed(KeyCode::KeyE) {
                        next_state.set(GameState::Dungeon);
                    }
                    "E — Enter the dungeon"
                } else {
                    "Approach the gate to enter the dungeon"
                }
            }
        };
    }

    if cooldown.0.finished()
        && homestead_forest_transition().contains(position)
        && velocity.y > 1.0
    {
        next_state.set(GameState::Forest);
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Title);
    }

    set_exploration_prompt(&mut prompt_label, prompt);
}

fn distance_to_zone(position: Vec2, bounds: &Rect) -> f32 {
    position.distance(bounds.center())
}