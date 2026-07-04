use bevy::prelude::*;

use super::{DungeonPlayState, GameState};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_sub_state::<DungeonPlayState>();
    }
}