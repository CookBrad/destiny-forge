use bevy::prelude::*;

use crate::graphics::{PLAYER_WALK_SPEED, TILE};
use crate::input::{Action, ActionState};

use super::eating::HomesteadEatCycle;
use super::sprites::{OverworldArt, PLAYER_ANIM_FRAMES};

#[derive(Resource, Clone)]
pub struct ExplorationMap {
    pub solids: Vec<Rect>,
    pub world_width: f32,
    pub world_height: f32,
    pub pixel_zoom: f32,
}

/// Prevents immediately bouncing back after a map transition.
#[derive(Resource)]
pub struct MapTransitionCooldown(pub Timer);

impl Default for MapTransitionCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(0.45, TimerMode::Once))
    }
}

pub fn tick_map_transition_cooldown(time: Res<Time>, mut cooldown: ResMut<MapTransitionCooldown>) {
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
    actions: Res<ActionState>,
    time: Res<Time>,
    map: Res<ExplorationMap>,
    mut player: Query<
        (&mut Transform, &mut OverworldVelocity, &mut Sprite),
        (With<OverworldPlayer>, Without<HomesteadEatCycle>),
    >,
) {
    let Ok((mut transform, mut velocity, mut sprite)) = player.get_single_mut() else {
        return;
    };

    let mut input = Vec2::ZERO;
    if actions.pressed(Action::MoveUp) {
        input.y += 1.0;
    }
    if actions.pressed(Action::MoveDown) {
        input.y -= 1.0;
    }
    if actions.pressed(Action::MoveLeft) {
        input.x -= 1.0;
    }
    if actions.pressed(Action::MoveRight) {
        input.x += 1.0;
    }

    if input.length_squared() > 0.0 {
        input = input.normalize();
    }

    velocity.x = input.x * PLAYER_WALK_SPEED;
    velocity.y = input.y * PLAYER_WALK_SPEED;

    let dt = time.delta_secs();
    let delta = Vec2::new(velocity.x * dt, velocity.y * dt);
    let mut next = transform.translation.truncate() + delta;
    next = resolve_collisions(next, delta, &map.solids);
    next.x = next.x.clamp(PLAYER_RADIUS, map.world_width - PLAYER_RADIUS);
    next.y = next
        .y
        .clamp(PLAYER_RADIUS, map.world_height - PLAYER_RADIUS);
    transform.translation.x = next.x;
    transform.translation.y = next.y;

    if input.x.abs() > 0.01 {
        sprite.flip_x = input.x < 0.0;
    }
}

pub fn animate_overworld_player(
    time: Res<Time>,
    art: Res<OverworldArt>,
    mut player: Query<
        (&OverworldVelocity, &mut Sprite),
        (With<OverworldPlayer>, Without<HomesteadEatCycle>),
    >,
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

#[cfg(test)]
mod movement_input_tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;

    use crate::graphics::PLAYER_WALK_SPEED;
    use crate::input::action_state_after_pressing;

    use super::{exploration_movement, ExplorationMap, OverworldPlayer, OverworldVelocity};

    fn velocity_after(keys: &[KeyCode]) -> Vec2 {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(action_state_after_pressing(keys, &[]))
            .insert_resource(ExplorationMap {
                solids: Vec::new(),
                world_width: 2_000.0,
                world_height: 2_000.0,
                pixel_zoom: 1.0,
            });
        app.world_mut().spawn((
            OverworldPlayer,
            Transform::from_xyz(400.0, 400.0, 0.0),
            OverworldVelocity::default(),
            Sprite::default(),
        ));
        app.world_mut()
            .run_system_once(exploration_movement)
            .unwrap();
        let velocity = app
            .world_mut()
            .query_filtered::<&OverworldVelocity, With<OverworldPlayer>>()
            .single(app.world());
        Vec2::new(velocity.x, velocity.y)
    }

    #[test]
    fn wasd_and_arrows_still_set_the_same_walk_velocity() {
        let west = velocity_after(&[KeyCode::KeyA]);
        let east = velocity_after(&[KeyCode::ArrowRight]);
        let north = velocity_after(&[KeyCode::KeyW]);
        let south = velocity_after(&[KeyCode::ArrowDown]);
        assert_eq!(west.x, -PLAYER_WALK_SPEED);
        assert_eq!(east.x, PLAYER_WALK_SPEED);
        assert_eq!(north.y, PLAYER_WALK_SPEED);
        assert_eq!(south.y, -PLAYER_WALK_SPEED);
        assert_eq!(velocity_after(&[KeyCode::KeyE]), Vec2::ZERO);
        assert_eq!(
            velocity_after(&[KeyCode::KeyW, KeyCode::ArrowUp]),
            velocity_after(&[KeyCode::KeyW])
        );
    }
}
