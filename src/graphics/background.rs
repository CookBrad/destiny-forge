use bevy::prelude::*;

use super::atlas::GameSprites;
use super::spawn::{image_sprite, sprite_transform, PIXEL_SCALE};

#[derive(Component)]
pub struct SceneBackground;

pub fn spawn_hub_background(
    commands: &mut Commands,
    sprites: &GameSprites,
    marker: impl Bundle + Copy,
) {
    commands.spawn((
        image_sprite(sprites.hub_background.clone(), Vec2::new(320.0, 180.0)),
        sprite_transform(Vec3::new(0.0, 20.0, -50.0)),
        SceneBackground,
        marker,
    ));
}

pub fn spawn_dungeon_background(
    commands: &mut Commands,
    sprites: &GameSprites,
    marker: impl Bundle + Copy,
) {
    commands.spawn((
        image_sprite(sprites.dungeon_background.clone(), Vec2::new(320.0, 180.0)),
        sprite_transform(Vec3::new(0.0, 20.0, -50.0)),
        SceneBackground,
        marker,
    ));
}