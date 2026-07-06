use bevy::prelude::*;

use crate::graphics::{DUNGEON_MOVE_SPEED, TILE};

use super::sprites::{OverworldArt, PLAYER_ANIM_FRAMES};

#[derive(Resource, Clone)]
pub struct ExplorationMap {
    pub solids: Vec<Rect>,
    pub world_width: f32,
    pub world_height: f32,
}

/// Prevents immediately bouncing back after a map transition.
#[derive(Resource)]
pub struct MapTransitionCooldown(pub Timer);

impl Default for MapTransitionCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(0.45, TimerMode::Once))
    }
}

pub fn tick_map_transition_cooldown(
    time: Res<Time>,
    mut cooldown: ResMut<MapTransitionCooldown>,
) {
    cooldown.0.tick(time.delta());
}

#[derive(Component)]
pub struct OverworldPlayer;

#[derive(Component, Default)]
pub struct OverworldVelocity {
    pub x: f32,
    pub y: f32,
}

const PLAYER_RADIUS: f32 = TILE * 0.42;

pub fn exploration_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    map: Res<ExplorationMap>,
    mut player: Query<(&mut Transform, &mut OverworldVelocity), With<OverworldPlayer>>,
) {
    let Ok((mut transform, mut velocity)) = player.get_single_mut() else {
        return;
    };

    let mut input = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        input.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        input.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        input.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        input.x += 1.0;
    }

    if input.length_squared() > 0.0 {
        input = input.normalize();
    }

    velocity.x = input.x * DUNGEON_MOVE_SPEED;
    velocity.y = input.y * DUNGEON_MOVE_SPEED;

    let dt = time.delta_secs();
    let delta = Vec2::new(velocity.x * dt, velocity.y * dt);
    let mut next = transform.translation.truncate() + delta;
    next = resolve_collisions(next, delta, &map.solids);
    next.x = next
        .x
        .clamp(PLAYER_RADIUS, map.world_width - PLAYER_RADIUS);
    next.y = next
        .y
        .clamp(PLAYER_RADIUS, map.world_height - PLAYER_RADIUS);
    transform.translation.x = next.x;
    transform.translation.y = next.y;

    if input.x.abs() > 0.01 {
        let scale = transform.scale.x.abs();
        transform.scale.x = if input.x < 0.0 { -scale } else { scale };
    }
}

pub fn animate_overworld_player(
    time: Res<Time>,
    art: Res<OverworldArt>,
    mut player: Query<(&OverworldVelocity, &mut Sprite), With<OverworldPlayer>>,
) {
    let Ok((velocity, mut sprite)) = player.get_single_mut() else {
        return;
    };

    let moving = velocity.x.abs() + velocity.y.abs() >= 1.0;
    let frame = if moving {
        ((time.elapsed_secs() * 8.0) as usize) % PLAYER_ANIM_FRAMES
    } else {
        0
    };

    sprite.image = art.player.frame_handle(moving, frame);
    sprite.rect = None;
}

fn resolve_collisions(position: Vec2, delta: Vec2, solids: &[Rect]) -> Vec2 {
    let mut resolved = position;
    if delta.x.abs() > f32::EPSILON {
        resolved.x = position.x;
        if collides(resolved, solids) {
            resolved.x = position.x - delta.x;
        }
    }
    if delta.y.abs() > f32::EPSILON {
        resolved.y = position.y;
        if collides(resolved, solids) {
            resolved.y = position.y - delta.y;
        }
    }
    resolved
}

fn collides(center: Vec2, solids: &[Rect]) -> bool {
    let player = player_bounds(center);
    solids
        .iter()
        .any(|solid| !player.intersect(*solid).is_empty())
}

fn player_bounds(center: Vec2) -> Rect {
    Rect {
        min: center - Vec2::splat(PLAYER_RADIUS),
        max: center + Vec2::splat(PLAYER_RADIUS),
    }
}