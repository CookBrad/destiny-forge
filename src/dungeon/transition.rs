use bevy::prelude::*;

use crate::core::GameState;

use super::setup::DungeonExit;
use crate::player::DungeonPlayer;

pub fn leave_dungeon_via_exit(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<&Transform, With<DungeonPlayer>>,
    exit_query: Query<&Transform, With<DungeonExit>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    for exit_transform in &exit_query {
        let distance = player_transform
            .translation
            .truncate()
            .distance(exit_transform.translation.truncate());

        if distance < 56.0 {
            next_state.set(GameState::Hub);
            return;
        }
    }
}