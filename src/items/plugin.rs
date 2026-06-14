use bevy::prelude::*;

use super::MaterialInventory;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MaterialInventory>();
    }
}