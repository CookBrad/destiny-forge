use bevy::prelude::*;

use crate::player::{DungeonPlayer, DungeonVelocity, Facing, HubPlayer};

use super::atlas::{GameSprites, HubFacing, PlayerSprite};

pub const PLAYER_IDLE_FRAMES: &[usize] = &PlayerSprite::RIGHT_IDLE;

#[derive(Component)]
pub struct AnimatedSprite {
    pub frames: &'static [usize],
    pub frame_duration: f32,
    pub timer: f32,
    pub current_frame: usize,
}

impl AnimatedSprite {
    pub fn new(frames: &'static [usize], frame_duration: f32) -> Self {
        Self {
            frames,
            frame_duration,
            timer: 0.0,
            current_frame: 0,
        }
    }
}

#[derive(Component, Default)]
pub struct PlayerWalkAnimation;

#[derive(Component)]
pub struct HubPlayerAnimation {
    pub facing: HubFacing,
    pub walk_timer: f32,
}

impl Default for HubPlayerAnimation {
    fn default() -> Self {
        Self {
            facing: HubFacing::Down,
            walk_timer: 0.0,
        }
    }
}

pub fn animate_sprites(
    time: Res<Time>,
    sprites: Res<GameSprites>,
    mut query: Query<(&mut AnimatedSprite, &mut Sprite)>,
) {
    for (mut animation, mut sprite) in &mut query {
        if animation.frames.len() <= 1 {
            continue;
        }

        animation.timer += time.delta_secs();
        if animation.timer < animation.frame_duration {
            continue;
        }

        animation.timer = 0.0;
        animation.current_frame = (animation.current_frame + 1) % animation.frames.len();
        apply_player_frame(&mut sprite, &sprites, animation.frames[animation.current_frame]);
    }
}

pub fn update_dungeon_player_animation(
    sprites: Res<GameSprites>,
    mut query: Query<
        (
            &DungeonVelocity,
            &Facing,
            &mut AnimatedSprite,
            &mut Sprite,
        ),
        With<DungeonPlayer>,
    >,
) {
    for (velocity, facing, mut animation, mut sprite) in &mut query {
        sprite.flip_x = false;

        let is_walking = velocity.x.abs() > 10.0 && velocity.grounded;
        let walk_frames = PlayerSprite::dungeon_walk_indices(*facing);

        if is_walking {
            if animation.frames != walk_frames {
                animation.frames = walk_frames;
                animation.current_frame = 0;
                animation.timer = 0.0;
            }
            continue;
        }

        let idle_index = PlayerSprite::dungeon_idle(*facing).atlas_index();
        if animation.frames.len() != 1 || animation.frames[0] != idle_index {
            animation.frames = match facing {
                Facing::Right => &PlayerSprite::RIGHT_IDLE,
                Facing::Left => &PlayerSprite::LEFT_IDLE,
            };
            animation.current_frame = 0;
            animation.timer = 0.0;
        }
        apply_player_frame(&mut sprite, &sprites, idle_index);
    }
}

pub fn update_hub_player_sprite(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    sprites: Res<GameSprites>,
    mut query: Query<(&mut Sprite, &mut HubPlayerAnimation), With<HubPlayer>>,
) {
    let Ok((mut sprite, mut animation)) = query.get_single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        animation.facing = HubFacing::from_direction(direction);
    }

    sprite.flip_x = false;

    let frame_index = if direction == Vec2::ZERO {
        animation.walk_timer = 0.0;
        PlayerSprite::idle_for_facing(animation.facing).atlas_index()
    } else {
        animation.walk_timer += time.delta_secs();
        let walk_frames = PlayerSprite::walk_indices_for_facing(animation.facing);
        let step = (animation.walk_timer * 8.0) as usize % PlayerSprite::DOWN_WALK.len();
        walk_frames[step]
    };

    apply_player_frame(&mut sprite, &sprites, frame_index);
}

impl HubFacing {
    fn from_direction(direction: Vec2) -> Self {
        if direction.y.abs() >= direction.x.abs() {
            if direction.y > 0.0 {
                Self::Up
            } else {
                Self::Down
            }
        } else if direction.x > 0.0 {
            Self::Right
        } else {
            Self::Left
        }
    }
}

fn apply_player_frame(sprite: &mut Sprite, sprites: &GameSprites, index: usize) {
    sprite.image = sprites.player.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: sprites.player_layout.clone(),
        index,
    });
}