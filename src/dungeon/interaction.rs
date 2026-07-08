use bevy::prelude::*;

use crate::combat::EnemyCorpse;
use crate::core::ProfileDirty;
use crate::core::GameState;
use crate::graphics::INTERACT_DISTANCE;
use crate::overworld::setup::OverworldEntry;
use crate::ui::interaction_prompt::{best_prompt, InteractionPrompt, PromptKind};
use crate::ui::inventory_window::InventoryWindowOpen;

use super::movement::DungeonPlayer;
use super::setup::DungeonExit;

pub fn ladder_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Query<&Transform, With<DungeonPlayer>>,
    exits: Query<&Transform, With<DungeonExit>>,
    mut commands: Commands,
    mut next_game: ResMut<NextState<GameState>>,
    mut profile_dirty: ResMut<ProfileDirty>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    if !near_exit(player_transform, &exits) || !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    profile_dirty.mark();
    commands.insert_resource(OverworldEntry::DungeonReturn);
    next_game.set(GameState::Overworld);
}

/// Tooltip for ladder exit and carvable corpses.
pub fn update_dungeon_interaction_prompt(
    inventory: Res<InventoryWindowOpen>,
    player: Query<&Transform, With<DungeonPlayer>>,
    exits: Query<&Transform, With<DungeonExit>>,
    corpses: Query<&Transform, With<EnemyCorpse>>,
    mut prompt: ResMut<InteractionPrompt>,
) {
    if inventory.0 {
        prompt.clear();
        return;
    }

    let Ok(player_transform) = player.get_single() else {
        prompt.clear();
        return;
    };

    let mut candidates = Vec::with_capacity(2);
    if near_exit(player_transform, &exits) {
        candidates.push(PromptKind::ClimbLadder);
    }
    if near_corpse(player_transform, &corpses) {
        candidates.push(PromptKind::Carve);
    }

    prompt.set(best_prompt(&candidates));
}

fn near_exit(player: &Transform, exits: &Query<&Transform, With<DungeonExit>>) -> bool {
    exits.iter().any(|exit| {
        player.translation.distance(exit.translation) <= INTERACT_DISTANCE
    })
}

fn near_corpse(player: &Transform, corpses: &Query<&Transform, With<EnemyCorpse>>) -> bool {
    corpses.iter().any(|corpse| {
        player.translation.distance(corpse.translation) <= INTERACT_DISTANCE
    })
}
