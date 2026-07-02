use bevy::prelude::*;

use crate::core::GameState;

use super::camera::{follow_camera, spawn_camera};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, follow_camera.run_if(in_state(GameState::Dungeon)));
    }
}