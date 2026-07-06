use bevy::prelude::*;

use super::movement::{ExplorationMap, OverworldPlayer};

const OVERWORLD_CAMERA_Z: f32 = 100.0;

pub fn init_exploration_camera(
    map: Res<ExplorationMap>,
    player: Query<&Transform, (With<OverworldPlayer>, Without<Camera2d>)>,
    window: Query<&Window>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<OverworldPlayer>)>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.get_single_mut() else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };

    let half_view = Vec2::new(window.width() * 0.5, window.height() * 0.5);
    let world = Vec2::new(map.world_width, map.world_height);
    let target = clamp_camera(player_transform.translation.truncate(), half_view, world);
    camera_transform.translation = target.extend(OVERWORLD_CAMERA_Z);
}

pub fn follow_exploration_camera(
    map: Res<ExplorationMap>,
    player: Query<&Transform, (With<OverworldPlayer>, Without<Camera2d>)>,
    window: Query<&Window>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<OverworldPlayer>)>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.get_single_mut() else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };

    let half_view = Vec2::new(window.width() * 0.5, window.height() * 0.5);
    let world = Vec2::new(map.world_width, map.world_height);
    let target = clamp_camera(player_transform.translation.truncate(), half_view, world);
    camera_transform.translation = target.extend(OVERWORLD_CAMERA_Z);
}

fn clamp_camera(player: Vec2, half_view: Vec2, world: Vec2) -> Vec2 {
    let min = half_view;
    let max = world - half_view;
    if max.x < min.x || max.y < min.y {
        return world * 0.5;
    }
    player.clamp(min, max)
}