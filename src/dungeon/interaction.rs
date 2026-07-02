use bevy::prelude::*;

use crate::graphics::INTERACT_DISTANCE;

use super::enemy::DungeonProgress;
use super::movement::DungeonPlayer;
use super::setup::DungeonExit;

#[derive(Resource, Default)]
pub struct LadderPrompt {
    pub near_exit: bool,
    pub exit_unlocked: bool,
}

pub fn update_ladder_prompt(
    player: Query<&Transform, With<DungeonPlayer>>,
    exits: Query<&Transform, With<DungeonExit>>,
    progress: Res<DungeonProgress>,
    mut prompt: ResMut<LadderPrompt>,
) {
    let Ok(player_transform) = player.get_single() else {
        *prompt = LadderPrompt::default();
        return;
    };

    prompt.near_exit = exits.iter().any(|exit| {
        player_transform
            .translation
            .distance(exit.translation)
            <= INTERACT_DISTANCE
    });
    prompt.exit_unlocked = progress.boss_defeated;
}

pub fn ladder_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    prompt: Res<LadderPrompt>,
) {
    if !prompt.near_exit || !prompt.exit_unlocked || !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    info!("Ladder reached — hub transition coming in the next milestone.");
}