use bevy::prelude::*;

use crate::graphics::INTERACT_DISTANCE;

use super::enemy::DungeonProgress;
use super::movement::DungeonPlayer;
use super::setup::DungeonExit;

pub fn ladder_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Query<&Transform, With<DungeonPlayer>>,
    exits: Query<&Transform, With<DungeonExit>>,
    progress: Res<DungeonProgress>,
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

    if !near_exit || !progress.boss_defeated || !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    info!("Ladder reached — hub transition coming in the next milestone.");
}