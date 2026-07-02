use bevy::prelude::*;

use crate::graphics::INTERACT_DISTANCE;

use super::movement::DungeonPlayer;
use super::setup::DungeonExit;

#[derive(Resource, Default)]
pub struct LadderPrompt {
    pub visible: bool,
}

pub fn update_ladder_prompt(
    player: Query<&Transform, With<DungeonPlayer>>,
    exits: Query<&Transform, With<DungeonExit>>,
    mut prompt: ResMut<LadderPrompt>,
) {
    let Ok(player_transform) = player.get_single() else {
        prompt.visible = false;
        return;
    };

    prompt.visible = exits.iter().any(|exit| {
        player_transform
            .translation
            .distance(exit.translation)
            <= INTERACT_DISTANCE
    });
}

pub fn ladder_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    prompt: Res<LadderPrompt>,
) {
    if !prompt.visible || !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    info!("Ladder reached — hub transition coming in the next milestone.");
}