use bevy::prelude::*;

use crate::core::GameState;
use crate::overworld::movement::{MapTransitionCooldown, OverworldPlayer, OverworldVelocity};
use crate::overworld::setup::{OverworldEntry, OverworldPromptLabel, OverworldZoneLabel};

use super::layout::{forest_homestead_transition, ForestLayout, ForestZone};

pub fn forest_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    layout: Res<ForestLayout>,
    cooldown: Res<MapTransitionCooldown>,
    player: Query<(&Transform, &OverworldVelocity), With<OverworldPlayer>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut zone_label: Query<&mut Text, (With<OverworldZoneLabel>, Without<OverworldPromptLabel>)>,
    mut prompt_label: Query<&mut Text, (With<OverworldPromptLabel>, Without<OverworldZoneLabel>)>,
) {
    let Ok((transform, velocity)) = player.get_single() else {
        return;
    };

    let position = transform.translation.truncate();
    let zone = layout.zone_at(position);

    if let Ok(mut text) = zone_label.get_single_mut() {
        text.0 = zone
            .map(|zone| zone.label.to_string())
            .unwrap_or_else(|| "Whispering Forest".to_string());
    }

    let mut prompt = "WASD move  ·  Esc title".to_string();
    if let Some(zone) = zone {
        match zone.zone {
            ForestZone::HomesteadReturn => {
                prompt = "Walk south down the trail to return home".to_string();
            }
            ForestZone::DeepWoods => {
                prompt = "Ancient trees loom overhead".to_string();
            }
            ForestZone::Woods => {}
        }
    }

    if cooldown.0.finished()
        && forest_homestead_transition().contains(position)
        && velocity.y < -1.0
    {
        commands.insert_resource(OverworldEntry::ForestTrail);
        next_state.set(GameState::Overworld);
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Title);
    }

    if let Ok(mut text) = prompt_label.get_single_mut() {
        text.0 = prompt;
    }
}