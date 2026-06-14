use bevy::prelude::*;

#[derive(Component)]
pub struct DungeonVelocity {
    pub x: f32,
    pub y: f32,
    pub grounded: bool,
}