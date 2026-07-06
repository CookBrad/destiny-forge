use bevy::prelude::*;

use crate::dungeon::DungeonPlayer;
use crate::graphics::{DUNGEON_FLOOR_Y, PIXEL_SCALE, TILE};

const CAMERA_HEIGHT_ABOVE_FLOOR: f32 = 5.5 * TILE;

/// Horizontal span of the current dungeon floor in native world pixels.
#[derive(Resource, Clone, Copy)]
pub struct DungeonScrollBounds {
    pub width: f32,
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection::default_2d()),
        Transform::from_xyz(0.0, camera_y(), 0.0),
    ));
}

/// Zoom the 2D camera so one native art pixel renders `pixel_scale` screen pixels.
pub fn set_camera_pixel_zoom(projection: &mut Projection, pixel_scale: f32) {
    let Projection::Orthographic(ortho) = projection else {
        return;
    };
    ortho.scale = 1.0 / pixel_scale.max(1.0);
}

pub fn reset_camera_zoom(projection: &mut Projection) {
    set_camera_pixel_zoom(projection, 1.0);
}

pub fn apply_exploration_camera_zoom(projection: &mut Projection) {
    set_camera_pixel_zoom(projection, PIXEL_SCALE);
}

pub fn init_dungeon_camera(
    bounds: Res<DungeonScrollBounds>,
    player: Query<&Transform, With<DungeonPlayer>>,
    window: Query<&Window>,
    mut camera: Query<
        (&mut Transform, &mut Projection),
        (With<Camera2d>, Without<DungeonPlayer>),
    >,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let Ok((mut camera_transform, mut projection)) = camera.get_single_mut() else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };

    reset_camera_zoom(&mut projection);
    let half_view = viewport_half_width(window);
    camera_transform.translation.x =
        clamp_camera_x(player_transform.translation.x, bounds.width, half_view);
    camera_transform.translation.y = camera_y();
}

pub fn follow_camera(
    bounds: Res<DungeonScrollBounds>,
    player: Query<&Transform, With<DungeonPlayer>>,
    window: Query<&Window>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<DungeonPlayer>)>,
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

    let half_view = viewport_half_width(window);
    let target_x = clamp_camera_x(player_transform.translation.x, bounds.width, half_view);
    camera_transform.translation.x = target_x;
    camera_transform.translation.y = camera_y();
}

fn clamp_camera_x(player_x: f32, dungeon_width: f32, half_viewport: f32) -> f32 {
    if dungeon_width <= half_viewport * 2.0 {
        return dungeon_width * 0.5;
    }

    let min_x = half_viewport;
    let max_x = dungeon_width - half_viewport;
    player_x.clamp(min_x, max_x)
}

/// Default Bevy 2D maps window logical pixels 1:1 to world units.
fn viewport_half_width(window: &Window) -> f32 {
    window.width() * 0.5
}

pub fn dungeon_camera_center_y() -> f32 {
    DUNGEON_FLOOR_Y + CAMERA_HEIGHT_ABOVE_FLOOR
}

/// World-space Y of the bottom edge of the visible viewport.
pub fn viewport_bottom_y(window: &Window) -> f32 {
    dungeon_camera_center_y() - window.height() * 0.5
}

fn camera_y() -> f32 {
    dungeon_camera_center_y()
}