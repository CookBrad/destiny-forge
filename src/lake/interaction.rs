use bevy::prelude::*;

use crate::core::GameState;
use crate::fishing::ActiveCast;
use crate::overworld::movement::{MapTransitionCooldown, OverworldPlayer, OverworldVelocity};
use crate::overworld::setup::OverworldEntry;
use crate::ui::interaction_prompt::{best_prompt, InteractionPrompt, PromptKind};
use crate::ui::inventory_window::InventoryWindowOpen;

use super::layout::lake_homestead_transition;

pub fn lake_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<InventoryWindowOpen>,
    fishing: Res<ActiveCast>,
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

    // Esc cancels fishing first (handled by fishing system).
    if fishing.minigame_active() {
        return;
    }

    let position = transform.translation.truncate();

    if cooldown.0.finished()
        && lake_homestead_transition().contains(position)
        && velocity.x < -1.0
    {
        commands.insert_resource(OverworldEntry::LakeReturn);
        next_state.set(GameState::Overworld);
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Title);
    }
}

pub fn update_lake_interaction_prompt(
    inventory: Res<InventoryWindowOpen>,
    fishing: Res<ActiveCast>,
    player: Query<&Transform, With<OverworldPlayer>>,
    spots: Query<&Transform, With<crate::fishing::FishingSpot>>,
    mut prompt: ResMut<InteractionPrompt>,
) {
    if inventory.0 || fishing.minigame_active() {
        prompt.clear();
        return;
    }
    let Ok(tf) = player.get_single() else {
        prompt.clear();
        return;
    };
    let pos = tf.translation.truncate();
    let near_spot = spots.iter().any(|s| {
        pos.distance(s.translation.truncate()) <= crate::graphics::TILE * 2.2
    });
    let near_exit = lake_homestead_transition().contains(pos);

    let mut candidates = Vec::new();
    if near_spot {
        candidates.push(PromptKind::Fish);
    }
    if near_exit {
        candidates.push(PromptKind::LeaveLake);
    }
    prompt.set(best_prompt(&candidates));
}
