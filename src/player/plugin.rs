use bevy::prelude::*;

use super::PlayerLoadout;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerLoadout>();
    }
}