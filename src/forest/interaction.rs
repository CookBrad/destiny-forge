use bevy::prelude::*;

use crate::core::GameState;
use crate::exploration::{
    set_exploration_prompt, set_exploration_zone_label, EXPLORATION_PROMPT_MOVE,
};
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

    set_exploration_zone_label(
        &mut zone_label,
        zone.map(|zone| zone.label)
            .unwrap_or("Whispering Forest"),
    );

    let mut prompt = EXPLORATION_PROMPT_MOVE;
    if let Some(zone) = zone {
        prompt = match zone.zone {
            ForestZone::HomesteadReturn => "Walk south down the trail to return home",
            ForestZone::DeepWoods => "Ancient trees loom overhead",
            ForestZone::Woods => EXPLORATION_PROMPT_MOVE,
        };
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

    set_exploration_prompt(&mut prompt_label, prompt);
}