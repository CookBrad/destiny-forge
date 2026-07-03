use bevy::prelude::*;

/// Hold `2` to raise the sword and deflect incoming projectiles.
#[derive(Component, Default)]
pub struct PlayerBlock {
    pub active: bool,
}

impl PlayerBlock {
    pub fn is_active(&self) -> bool {
        self.active
    }
}