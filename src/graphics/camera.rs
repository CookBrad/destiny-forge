use bevy::prelude::*;

use crate::dungeon::DungeonPlayer;
use crate::graphics::{CAMERA_ORTHO_SCALE, DUNGEON_FLOOR_Y};

/// Pinned in pixels, still 1x. In a 720 px view this frames the whole room:
/// three rows of ground under the floor, the ceiling beam and some rock over it.
const CAMERA_HEIGHT_ABOVE_FLOOR: f32 = 264.0;

/// Horizontal span of the current dungeon floor in native world pixels.
#[derive(Resource, Clone, Copy)]
pub struct DungeonScrollBounds {
    pub width: f32,
}

pub fn spawn_camera(mut commands: Commands) {
    // Camera2d already requires OrthographicProjection::default_2d at scale 1.
    // A second Projection would fight it for Camera.clip_from_view every frame.
    commands.spawn((Camera2d, Transform::from_xyz(0.0, camera_y(), 0.0)));
}

pub fn reset_camera_zoom(projection: &mut OrthographicProjection) {
    projection.scale = CAMERA_ORTHO_SCALE;
}

pub fn init_dungeon_camera(
    bounds: Res<DungeonScrollBounds>,
    player: Query<&Transform, With<DungeonPlayer>>,
    window: Query<&Window>,
    mut camera: Query<
        (&mut Transform, &mut OrthographicProjection),
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
    let half_view = viewport_half_width(window, projection.scale);
    camera_transform.translation.x =
        clamp_camera_x(player_transform.translation.x, bounds.width, half_view);
    camera_transform.translation.y = camera_y();
}

pub fn follow_camera(
    bounds: Res<DungeonScrollBounds>,
    player: Query<&Transform, With<DungeonPlayer>>,
    window: Query<&Window>,
    mut camera: Query<
        (&mut Transform, &OrthographicProjection),
        (With<Camera2d>, Without<DungeonPlayer>),
    >,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let Ok((mut camera_transform, projection)) = camera.get_single_mut() else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };

    let half_view = viewport_half_width(window, projection.scale);
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

fn viewport_half_width(window: &Window, ortho_scale: f32) -> f32 {
    window.width() * 0.5 * ortho_scale
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::graphics::{DUNGEON_CEILING_Y, HUNTER_BODY_PX, TILE};

    const VIEW_HALF_HEIGHT_720P: f32 = 360.0;

    #[test]
    fn camera_frames_the_ground_the_hunter_and_the_ceiling_in_a_720p_view() {
        assert_eq!(CAMERA_HEIGHT_ABOVE_FLOOR, 264.0);
        let center = dungeon_camera_center_y();
        assert_eq!(center, DUNGEON_FLOOR_Y + 264.0);
        let bottom = center - VIEW_HALF_HEIGHT_720P;
        let top = center + VIEW_HALF_HEIGHT_720P;
        assert_eq!(DUNGEON_FLOOR_Y - bottom, 3.0 * TILE, "three ground rows show");
        assert!(DUNGEON_FLOOR_Y + HUNTER_BODY_PX.y < top);
        assert!(DUNGEON_CEILING_Y + TILE < top, "the ceiling beam shows");
    }
}
