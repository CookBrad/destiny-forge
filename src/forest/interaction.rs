use bevy::prelude::*;

use crate::core::GameState;
use crate::ui::inventory_window::InventoryWindowOpen;
use crate::overworld::movement::{MapTransitionCooldown, OverworldPlayer, OverworldVelocity};
use crate::overworld::setup::OverworldEntry;

use super::layout::forest_homestead_transition;

pub fn forest_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<InventoryWindowOpen>,
    cooldown: Res<MapTransitionCooldown>,
    player: Query<(&Transform, &OverworldVelocity), With<OverworldPlayer>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    let Ok((transform, velocity)) = player.get_single() else {
        return;
    };

    if inventory.0 {
        return;
    }

    let position = transform.translation.truncate();

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
}