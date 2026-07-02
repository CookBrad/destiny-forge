use bevy::prelude::*;

use crate::dungeon::DungeonPlayer;
use crate::graphics::{DUNGEON_FLOOR_Y, TILE};

const CAMERA_HEIGHT_ABOVE_FLOOR: f32 = 5.5 * TILE;

/// Horizontal span of the current dungeon floor in native world pixels.
#[derive(Resource, Clone, Copy)]
pub struct DungeonScrollBounds {
    pub width: f32,
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, camera_y(), 0.0),
    ));
}

pub fn init_dungeon_camera(
    bounds: Res<DungeonScrollBounds>,
    player: Query<&Transform, With<DungeonPlayer>>,
    window: Query<&Window>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<DungeonPlayer>)>,
    projections: Query<&Projection, With<Camera2d>>,
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
    let Ok(projection) = projections.get_single() else {
        return;
    };

    let half_view = viewport_half_width(window, projection);
    camera_transform.translation.x =
        clamp_camera_x(player_transform.translation.x, bounds.width, half_view);
    camera_transform.translation.y = camera_y();
}

pub fn follow_camera(
    bounds: Res<DungeonScrollBounds>,
    player: Query<&Transform, With<DungeonPlayer>>,
    window: Query<&Window>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<DungeonPlayer>)>,
    projections: Query<&Projection, With<Camera2d>>,
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
    let Ok(projection) = projections.get_single() else {
        return;
    };

    let half_view = viewport_half_width(window, projection);
    let target_x = clamp_camera_x(player_transform.translation.x, bounds.width, half_view);
    let desired = Vec3::new(target_x, camera_y(), camera_transform.translation.z);
    camera_transform.translation = camera_transform.translation.lerp(desired, 0.12);
}

fn clamp_camera_x(player_x: f32, dungeon_width: f32, half_viewport: f32) -> f32 {
    if dungeon_width <= half_viewport * 2.0 {
        return dungeon_width * 0.5;
    }

    let min_x = half_viewport;
    let max_x = dungeon_width - half_viewport;
    player_x.clamp(min_x, max_x)
}

fn viewport_half_width(window: &Window, projection: &Projection) -> f32 {
    let Projection::Orthographic(ortho) = projection else {
        return window.width() * 0.5;
    };

    if ortho.area.width() > 0.0 {
        return ortho.area.width() / (2.0 * ortho.scale);
    }

    let world_height = ortho.area.height() / ortho.scale;
    let aspect = window.width() / window.height();
    world_height * aspect * 0.5
}

fn camera_y() -> f32 {
    DUNGEON_FLOOR_Y + CAMERA_HEIGHT_ABOVE_FLOOR
}