use bevy::prelude::*;

use crate::core::GameState;
use crate::graphics::INTERACT_DISTANCE;

use super::layout::{HomesteadZone, OverworldLayout};
use super::movement::OverworldPlayer;
use super::setup::{OverworldPromptLabel, OverworldZoneLabel};

pub fn overworld_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    layout: Res<OverworldLayout>,
    player: Query<&Transform, With<OverworldPlayer>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut zone_label: Query<&mut Text, (With<OverworldZoneLabel>, Without<OverworldPromptLabel>)>,
    mut prompt_label: Query<&mut Text, (With<OverworldPromptLabel>, Without<OverworldZoneLabel>)>,
) {
    let Ok(transform) = player.get_single() else {
        return;
    };

    let position = transform.translation.truncate();
    let zone = layout.zone_at(position);

    if let Ok(mut text) = zone_label.get_single_mut() {
        text.0 = zone
            .map(|zone| zone.label.to_string())
            .unwrap_or_else(|| "Homestead Yard".to_string());
    }

    let mut prompt = "WASD move  ·  E interact  ·  Esc title".to_string();
    if let Some(zone) = zone {
        match zone.zone {
            HomesteadZone::House => prompt = "E — Enter house (soon)".to_string(),
            HomesteadZone::Forge => prompt = "E — Open forge (soon)".to_string(),
            HomesteadZone::Crops => prompt = "Crop plots ready for planting".to_string(),
            HomesteadZone::Animals => prompt = "Feed and tend your livestock".to_string(),
            HomesteadZone::DungeonGate => {
                if distance_to_zone(position, &zone.bounds) <= INTERACT_DISTANCE * 2.0 {
                    prompt = "E — Enter the dungeon".to_string();
                    if keyboard.just_pressed(KeyCode::KeyE) {
                        next_state.set(GameState::Dungeon);
                    }
                } else {
                    prompt = "Approach the gate to enter the dungeon".to_string();
                }
            }
            HomesteadZone::Yard => {}
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Title);
    }

    if let Ok(mut text) = prompt_label.get_single_mut() {
        text.0 = prompt;
    }
}

fn distance_to_zone(position: Vec2, bounds: &Rect) -> f32 {
    let center = bounds.center();
    position.distance(center)
}