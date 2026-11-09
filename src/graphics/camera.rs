use bevy::prelude::*;

use crate::dungeon::DungeonPlayer;
use crate::graphics::{CAMERA_ORTHO_SCALE, DUNGEON_FLOOR_Y};

const CAMERA_HEIGHT_ABOVE_FLOOR: f32 = 264.0;

pub(crate) type DungeonCameraInit<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static mut OrthographicProjection),
    (With<Camera2d>, Without<DungeonPlayer>),
>;

pub(crate) type DungeonCameraFollow<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static OrthographicProjection),
    (With<Camera2d>, Without<DungeonPlayer>),
>;

#[derive(Resource, Clone, Copy)]
pub struct DungeonScrollBounds {
    pub width: f32,
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, camera_y(), 0.0)));
}

pub fn reset_camera_zoom(projection: &mut OrthographicProjection) {
    projection.scale = CAMERA_ORTHO_SCALE;
}

pub fn init_dungeon_camera(
    bounds: Res<DungeonScrollBounds>,
    player: Query<&Transform, With<DungeonPlayer>>,
    window: Query<&Window>,
    mut camera: DungeonCameraInit,
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
    mut camera: DungeonCameraFollow,
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

pub fn viewport_bottom_y(window: &Window) -> f32 {
    dungeon_camera_center_y() - window.height() * 0.5
}

fn camera_y() -> f32 {
    dungeon_camera_center_y()
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::system::RunSystemOnce;

    use crate::graphics::{DUNGEON_CEILING_Y, HUNTER_BODY_PX, TILE};

    const VIEW_HALF_HEIGHT_720P: f32 = 360.0;
    const WINDOW_WIDTH: f32 = 1280.0;
    const WINDOW_HEIGHT: f32 = 720.0;
    const DUNGEON_WIDTH: f32 = 3_200.0;

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

    #[test]
    fn init_dungeon_camera_centers_on_a_short_dungeon() {
        let mut app = dungeon_camera_app(400.0, 200.0);
        app.world_mut()
            .run_system_once(init_dungeon_camera)
            .unwrap();
        let (x, y, scale) = camera_pose(&mut app);
        assert!((x - 200.0).abs() < 1e-3, "short dungeon centers at half width");
        assert_eq!(y, dungeon_camera_center_y());
        assert_eq!(scale, CAMERA_ORTHO_SCALE);
    }

    #[test]
    fn init_dungeon_camera_clamps_to_the_player_inside_a_wide_dungeon() {
        let mut app = dungeon_camera_app(DUNGEON_WIDTH, 1_600.0);
        app.world_mut()
            .run_system_once(init_dungeon_camera)
            .unwrap();
        let (x, y, scale) = camera_pose(&mut app);
        let half_view = WINDOW_WIDTH * 0.5 * CAMERA_ORTHO_SCALE;
        assert!((x - 1_600.0).abs() < 1e-3);
        assert!(x >= half_view - 1e-3);
        assert!(x <= DUNGEON_WIDTH - half_view + 1e-3);
        assert_eq!(y, dungeon_camera_center_y());
        assert_eq!(scale, CAMERA_ORTHO_SCALE);
    }

    #[test]
    fn follow_camera_tracks_the_player_along_x() {
        let mut app = dungeon_camera_app(DUNGEON_WIDTH, 800.0);
        app.world_mut()
            .run_system_once(init_dungeon_camera)
            .unwrap();
        let hunter = app
            .world_mut()
            .query_filtered::<Entity, With<DungeonPlayer>>()
            .single(app.world());
        app.world_mut().get_mut::<Transform>(hunter).unwrap().translation.x = 2_400.0;
        app.world_mut().run_system_once(follow_camera).unwrap();
        let (x, y, _) = camera_pose(&mut app);
        assert!((x - 2_400.0).abs() < 1e-3);
        assert_eq!(y, dungeon_camera_center_y());
    }

    #[test]
    fn follow_camera_leaves_the_camera_when_no_hunter_is_present() {
        let mut app = dungeon_camera_app(DUNGEON_WIDTH, 800.0);
        app.world_mut()
            .run_system_once(init_dungeon_camera)
            .unwrap();
        let (before_x, before_y, _) = camera_pose(&mut app);
        let hunter = app
            .world_mut()
            .query_filtered::<Entity, With<DungeonPlayer>>()
            .single(app.world());
        app.world_mut().entity_mut(hunter).despawn();
        app.world_mut().run_system_once(follow_camera).unwrap();
        let (after_x, after_y, _) = camera_pose(&mut app);
        assert_eq!(after_x, before_x);
        assert_eq!(after_y, before_y);
    }

    fn dungeon_camera_app(dungeon_width: f32, player_x: f32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(DungeonScrollBounds {
                width: dungeon_width,
            });
        app.world_mut().spawn(Window {
            resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
            ..default()
        });
        app.world_mut().spawn((
            Camera2d,
            Transform::from_xyz(0.0, 0.0, 0.0),
            OrthographicProjection::default_2d(),
        ));
        app.world_mut().spawn((
            DungeonPlayer,
            Transform::from_xyz(player_x, DUNGEON_FLOOR_Y, 0.0),
        ));
        app
    }

    fn camera_pose(app: &mut App) -> (f32, f32, f32) {
        let mut cameras = app
            .world_mut()
            .query_filtered::<(&Transform, &OrthographicProjection), With<Camera2d>>();
        let (transform, projection) = cameras.single(app.world());
        (
            transform.translation.x,
            transform.translation.y,
            projection.scale,
        )
    }
}
