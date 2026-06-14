use bevy::prelude::*;

use crate::combat::{AttackCooldown, Health};
use crate::graphics::{
    pixel_sprite, sprite_transform, AnimatedSprite, GameSprites, PlayerSprite, PlayerWalkAnimation,
    PixelSheet, PLAYER_FRAME_HEIGHT, PLAYER_FRAME_WIDTH, PLAYER_IDLE_FRAMES, PIXEL_SCALE,
};
use crate::player::{DungeonPlayer, DungeonVelocity, Facing};

use crate::graphics::PlatformBounds;

use super::setup::{DungeonEntity, GroundPlatform, DUNGEON_FLOOR_Y};

const GRAVITY: f32 = -900.0;
const JUMP_SPEED: f32 = 360.0;
const MOVE_SPEED: f32 = 220.0;

const PLAYER_TILE_SIZE: Vec2 =
    Vec2::new(PLAYER_FRAME_WIDTH as f32, PLAYER_FRAME_HEIGHT as f32);

pub fn spawn_dungeon_player(commands: &mut Commands, sprites: &GameSprites) -> Entity {
    let ground_top = DUNGEON_FLOOR_Y + 24.0;
    let player_half_height = PLAYER_TILE_SIZE.y * PIXEL_SCALE * 0.5;
    let spawn_y = ground_top + player_half_height;

    commands
        .spawn((
            pixel_sprite(
                sprites,
                PixelSheet::Player,
                PlayerSprite::Right0.atlas_index(),
            ),
            sprite_transform(Vec3::new(-420.0, spawn_y, 15.0)),
            AnimatedSprite::new(PLAYER_IDLE_FRAMES, 0.1),
            PlayerWalkAnimation,
            DungeonVelocity {
                x: 0.0,
                y: 0.0,
                grounded: false,
            },
            Health::new(100.0),
            AttackCooldown::default(),
            Facing::Right,
            DungeonPlayer,
            DungeonEntity,
            Name::new("DungeonPlayer"),
        ))
        .id()
}

pub fn dungeon_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut DungeonVelocity, &mut Facing), With<DungeonPlayer>>,
) {
    let Ok((mut transform, mut velocity, mut facing)) = query.get_single_mut() else {
        return;
    };

    let mut move_input = 0.0;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        move_input -= 1.0;
        *facing = Facing::Left;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        move_input += 1.0;
        *facing = Facing::Right;
    }

    velocity.x = move_input * MOVE_SPEED;

    if keyboard.just_pressed(KeyCode::Space) && velocity.grounded {
        velocity.y = JUMP_SPEED;
        velocity.grounded = false;
    }

    velocity.y += GRAVITY * time.delta_secs();
    transform.translation.x += velocity.x * time.delta_secs();
    transform.translation.y += velocity.y * time.delta_secs();
}

pub fn apply_platform_collisions(
    mut player_query: Query<
        (&mut Transform, &mut DungeonVelocity),
        (With<DungeonPlayer>, Without<GroundPlatform>),
    >,
    platform_query: Query<
        (&Transform, &PlatformBounds),
        (With<GroundPlatform>, Without<DungeonPlayer>),
    >,
) {
    let Ok((mut player_transform, mut velocity)) = player_query.get_single_mut() else {
        return;
    };

    let player_half_size = PLAYER_TILE_SIZE * PIXEL_SCALE * 0.5;
    velocity.grounded = false;

    for (platform_transform, platform_bounds) in &platform_query {
        if !is_player_above_platform(
            &player_transform.translation,
            player_half_size,
            platform_transform,
            platform_bounds.half_size,
        ) {
            continue;
        }

        snap_player_to_platform(
            &mut player_transform,
            &mut velocity,
            player_half_size.y,
            platform_transform.translation.y + platform_bounds.half_size.y,
        );
    }
}

fn is_player_above_platform(
    player_position: &Vec3,
    player_half_size: Vec2,
    platform_transform: &Transform,
    platform_half_size: Vec2,
) -> bool {
    let player_bottom = player_position.y - player_half_size.y;
    let platform_top = platform_transform.translation.y + platform_half_size.y;

    let horizontal_overlap = (player_position.x - platform_transform.translation.x).abs()
        < platform_half_size.x + player_half_size.x;

    horizontal_overlap
        && player_bottom <= platform_top
        && player_bottom >= platform_top - 24.0
}

fn snap_player_to_platform(
    player_transform: &mut Transform,
    velocity: &mut DungeonVelocity,
    player_half_height: f32,
    platform_top: f32,
) {
    if velocity.y > 0.0 {
        return;
    }

    player_transform.translation.y = platform_top + player_half_height;
    velocity.y = 0.0;
    velocity.grounded = true;
}

