use bevy::prelude::*;

use crate::dungeon::DungeonPlayer;
use crate::graphics::{
    camera_ortho_scale, game_camera_ortho_scale, DUNGEON_FLOOR_Y, DISPLAY_SCALE, TILE,
};

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

/// Zoom the 2D camera so native art units appear `display_scale` larger on screen.
pub fn set_camera_display_zoom(projection: &mut Projection, display_scale: f32) {
    let Projection::Orthographic(ortho) = projection else {
        return;
    };
    ortho.scale = camera_ortho_scale(display_scale);
}

/// Apply the shared higher-res 2D camera zoom used by exploration and dungeon.
pub fn apply_game_camera_zoom(projection: &mut Projection) {
    set_camera_display_zoom(projection, DISPLAY_SCALE);
}

/// Identity zoom (UI / title). Prefer [`apply_game_camera_zoom`] in gameplay scenes.
pub fn reset_camera_zoom(projection: &mut Projection) {
    set_camera_display_zoom(projection, 1.0);
}

/// Exploration entry uses the same display zoom as dungeon.
pub fn apply_exploration_camera_zoom(projection: &mut Projection) {
    apply_game_camera_zoom(projection);
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

    apply_game_camera_zoom(&mut projection);
    let half_view = viewport_half_width(window, &projection);
    camera_transform.translation.x =
        clamp_camera_x(player_transform.translation.x, bounds.width, half_view);
    camera_transform.translation.y = camera_y();
}

pub fn follow_camera(
    bounds: Res<DungeonScrollBounds>,
    player: Query<&Transform, With<DungeonPlayer>>,
    window: Query<&Window>,
    mut camera: Query<(&mut Transform, &Projection), (With<Camera2d>, Without<DungeonPlayer>)>,
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

    let half_view = viewport_half_width(window, projection);
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

/// Half the visible world width, accounting for orthographic zoom.
fn viewport_half_width(window: &Window, projection: &Projection) -> f32 {
    let zoom = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0,
    };
    window.width() * 0.5 * zoom
}

pub fn dungeon_camera_center_y() -> f32 {
    DUNGEON_FLOOR_Y + CAMERA_HEIGHT_ABOVE_FLOOR
}

/// World-space Y of the bottom edge of the visible viewport under game camera zoom.
pub fn viewport_bottom_y(window: &Window) -> f32 {
    dungeon_camera_center_y() - window.height() * 0.5 * game_camera_ortho_scale()
}

fn camera_y() -> f32 {
    dungeon_camera_center_y()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_camera_centers_when_dungeon_fits() {
        assert!((clamp_camera_x(100.0, 200.0, 150.0) - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn clamp_camera_limits_scroll() {
        assert!((clamp_camera_x(0.0, 1000.0, 100.0) - 100.0).abs() < f32::EPSILON);
        assert!((clamp_camera_x(999.0, 1000.0, 100.0) - 900.0).abs() < f32::EPSILON);
        assert!((clamp_camera_x(500.0, 1000.0, 100.0) - 500.0).abs() < f32::EPSILON);
    }

    #[test]
    fn viewport_bottom_accounts_for_display_zoom() {
        // Synthetic window height: with DISPLAY_SCALE zoom, visible half-height shrinks.
        let window_height = 600.0;
        let expected =
            dungeon_camera_center_y() - window_height * 0.5 * game_camera_ortho_scale();
        // Mirror the pure math used by viewport_bottom_y without needing a Window.
        assert!((expected - (dungeon_camera_center_y() - 100.0)).abs() < f32::EPSILON);
        assert!((game_camera_ortho_scale() - 1.0 / DISPLAY_SCALE).abs() < f32::EPSILON);
    }
}
