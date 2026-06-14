use bevy::prelude::*;

use super::controls::{spawn_controls_help, update_controls_for_state};
use super::hud::{setup_hud, update_hud};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_hud, spawn_controls_help))
            .add_systems(Update, (update_hud, update_controls_for_state));
    }
}