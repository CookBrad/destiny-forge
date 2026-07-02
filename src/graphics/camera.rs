use bevy::prelude::*;

use crate::graphics::{DUNGEON_FLOOR_Y, TILE};

const CAMERA_HEIGHT_ABOVE_FLOOR: f32 = 5.5 * TILE;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, camera_y(), 0.0),
    ));
}

pub fn follow_camera(
    player: Query<&Transform, With<crate::dungeon::DungeonPlayer>>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<crate::dungeon::DungeonPlayer>)>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.get_single_mut() else {
        return;
    };

    let desired = Vec3::new(
        player_transform.translation.x,
        camera_y(),
        camera_transform.translation.z,
    );
    camera_transform.translation = camera_transform.translation.lerp(desired, 0.12);
}

fn camera_y() -> f32 {
    DUNGEON_FLOOR_Y + CAMERA_HEIGHT_ABOVE_FLOOR
}