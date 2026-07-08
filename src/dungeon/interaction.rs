use bevy::prelude::*;

use crate::core::ProfileDirty;
use crate::core::GameState;
use crate::graphics::INTERACT_DISTANCE;
use crate::overworld::setup::OverworldEntry;

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

    let near_exit = exits.iter().any(|exit| {
        player_transform
            .translation
            .distance(exit.translation)
            <= INTERACT_DISTANCE
    });

    if !near_exit || !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    profile_dirty.mark();
    commands.insert_resource(OverworldEntry::DungeonReturn);
    next_game.set(GameState::Overworld);
}