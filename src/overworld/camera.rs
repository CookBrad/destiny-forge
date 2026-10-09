use bevy::prelude::*;

use crate::graphics::CAMERA_ORTHO_SCALE;

use super::movement::{ExplorationMap, OverworldPlayer};

const OVERWORLD_CAMERA_Z: f32 = 100.0;

/// Brad (2026-10): the home area was too small to read at 1×, so the homestead camera
/// draws one world unit as three screen pixels again. Art and transforms stay 1×.
pub const HOMESTEAD_PIXEL_ZOOM: f32 = 3.0;
/// The forest keeps the shared 1× camera; only the homestead was re-locked to 3×.
pub const FOREST_PIXEL_ZOOM: f32 = 1.0;

pub fn init_exploration_camera(
    map: Res<ExplorationMap>,
    player: Query<&Transform, (With<OverworldPlayer>, Without<Camera2d>)>,
    window: Query<&Window>,
    mut camera: Query<
        (&mut Transform, &mut OrthographicProjection),
        (With<Camera2d>, Without<OverworldPlayer>),
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

    apply_pixel_zoom(&mut projection, map.pixel_zoom);
    let half_view = exploration_half_view(window, &projection);
    let world = Vec2::new(map.world_width, map.world_height);
    let target = clamp_camera(player_transform.translation.truncate(), half_view, world);
    camera_transform.translation = target.extend(OVERWORLD_CAMERA_Z);
}

pub fn follow_exploration_camera(
    map: Res<ExplorationMap>,
    player: Query<&Transform, (With<OverworldPlayer>, Without<Camera2d>)>,
    window: Query<&Window>,
    mut camera: Query<
        (&mut Transform, &OrthographicProjection),
        (With<Camera2d>, Without<OverworldPlayer>),
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

    let half_view = exploration_half_view(window, projection);
    let world = Vec2::new(map.world_width, map.world_height);
    let target = clamp_camera(player_transform.translation.truncate(), half_view, world);
    camera_transform.translation = target.extend(OVERWORLD_CAMERA_Z);
}

/// A zoom below 1× would shrink the map further, so it clamps to the shared 1× camera.
fn apply_pixel_zoom(projection: &mut OrthographicProjection, pixel_zoom: f32) {
    projection.scale = CAMERA_ORTHO_SCALE / pixel_zoom.max(1.0);
}

fn exploration_half_view(window: &Window, projection: &OrthographicProjection) -> Vec2 {
    Vec2::new(window.width() * 0.5, window.height() * 0.5) * projection.scale
}

fn clamp_camera(player: Vec2, half_view: Vec2, world: Vec2) -> Vec2 {
    let min = half_view;
    let max = world - half_view;
    if max.x < min.x || max.y < min.y {
        return world * 0.5;
    }
    player.clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn homestead_zoom_draws_a_tile_at_three_times_its_size() {
        let mut projection = OrthographicProjection::default_2d();
        apply_pixel_zoom(&mut projection, HOMESTEAD_PIXEL_ZOOM);
        assert_eq!(projection.scale, 1.0 / 3.0);
        assert_eq!(crate::graphics::TILE / projection.scale, 96.0);
    }

    #[test]
    fn forest_zoom_stays_on_the_shared_one_x_camera() {
        let mut projection = OrthographicProjection::default_2d();
        apply_pixel_zoom(&mut projection, FOREST_PIXEL_ZOOM);
        assert_eq!(projection.scale, CAMERA_ORTHO_SCALE);
    }

    #[test]
    fn zoom_below_one_x_clamps_to_the_shared_camera() {
        let mut projection = OrthographicProjection::default_2d();
        apply_pixel_zoom(&mut projection, 0.5);
        assert_eq!(projection.scale, CAMERA_ORTHO_SCALE);
    }

    #[test]
    fn homestead_half_view_is_a_third_of_the_window() {
        let mut projection = OrthographicProjection::default_2d();
        apply_pixel_zoom(&mut projection, HOMESTEAD_PIXEL_ZOOM);
        let window = Window {
            resolution: (1280.0_f32, 720.0_f32).into(),
            ..default()
        };
        let half_view = exploration_half_view(&window, &projection);
        assert!((half_view.x - 1280.0 / 6.0).abs() < 1e-3);
        assert!((half_view.y - 120.0).abs() < 1e-3);
    }
}
