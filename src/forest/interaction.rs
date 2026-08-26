use bevy::prelude::*;

use crate::core::GameState;
use crate::ui::inventory_window::InventoryWindowOpen;
use crate::overworld::movement::{MapTransitionCooldown, OverworldPlayer, OverworldVelocity};
use crate::overworld::setup::OverworldEntry;

use super::layout::forest_homestead_transition;

pub fn forest_interaction(
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
        // Spawn on homestead at this exit (aligned with where we left).
        commands.insert_resource(OverworldEntry::from_forest_return(position));
        next_state.set(GameState::Overworld);
    }

    // Escape never returns to title from the forest.
}