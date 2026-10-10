use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::overworld::movement::OverworldPlayer;

use super::layout::MineEntity;

pub const PICKAXE_SWING_SECONDS: f32 = 0.3;
const RAISED_ANGLE: f32 = -FRAC_PI_2 * 0.5;
const STRUCK_ANGLE: f32 = FRAC_PI_2 * 1.2;
const HANDLE_LENGTH: f32 = 16.0;
const HANDLE_COLOR: Color = Color::srgb(0.5, 0.34, 0.18);
const HEAD_COLOR: Color = Color::srgb(0.7, 0.72, 0.78);
const SWING_Z_ABOVE_PLAYER: f32 = 0.5;

#[derive(Component)]
pub struct PickaxeSwing {
    pub timer: Timer,
    pub target_node: Entity,
    pub swings_left: bool,
}

#[derive(Event)]
pub struct PickaxeSwingLanded {
    pub target_node: Entity,
}

pub fn swing_progress(swing: &PickaxeSwing) -> f32 {
    swing.timer.fraction().clamp(0.0, 1.0)
}

pub fn pickaxe_clockwise_angle_from_upright(progress: f32, swings_left: bool) -> f32 {
    let angle_when_swinging_right = RAISED_ANGLE + (STRUCK_ANGLE - RAISED_ANGLE) * progress;
    if swings_left {
        -angle_when_swinging_right
    } else {
        angle_when_swinging_right
    }
}

pub fn spawn_pickaxe_swing(
    commands: &mut Commands,
    player_transform: &Transform,
    target_node: Entity,
    swings_left: bool,
) {
    let swing = PickaxeSwing {
        timer: Timer::from_seconds(PICKAXE_SWING_SECONDS, TimerMode::Once),
        target_node,
        swings_left,
    };
    let transform = pickaxe_transform_at(player_transform, &swing);
    commands
        .spawn((swing, transform, Visibility::default(), MineEntity))
        .with_children(|pickaxe| {
            pickaxe.spawn((
                Sprite {
                    color: HANDLE_COLOR,
                    custom_size: Some(Vec2::new(3.0, HANDLE_LENGTH)),
                    ..default()
                },
                Transform::from_xyz(0.0, HANDLE_LENGTH * 0.5, 0.0),
            ));
            pickaxe.spawn((
                Sprite {
                    color: HEAD_COLOR,
                    custom_size: Some(Vec2::new(14.0, 4.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, HANDLE_LENGTH, 0.01),
            ));
        });
}

fn pickaxe_transform_at(player_transform: &Transform, swing: &PickaxeSwing) -> Transform {
    let angle = pickaxe_clockwise_angle_from_upright(swing_progress(swing), swing.swings_left);
    Transform {
        translation: player_transform.translation + Vec3::Z * SWING_Z_ABOVE_PLAYER,
        rotation: Quat::from_rotation_z(-angle),
        scale: player_transform.scale,
    }
}

pub fn animate_pickaxe_swing(
    time: Res<Time>,
    mut commands: Commands,
    mut landed_swings: EventWriter<PickaxeSwingLanded>,
    player: Query<&Transform, (With<OverworldPlayer>, Without<PickaxeSwing>)>,
    mut swings: Query<(Entity, &mut PickaxeSwing, &mut Transform)>,
) {
    let player_transform = player.get_single().ok();
    for (swing_entity, mut swing, mut transform) in &mut swings {
        swing.timer.tick(time.delta());
        if let Some(player_transform) = player_transform {
            *transform = pickaxe_transform_at(player_transform, &swing);
        }
        if swing.timer.just_finished() {
            landed_swings.send(PickaxeSwingLanded {
                target_node: swing.target_node,
            });
            commands.entity(swing_entity).try_despawn_recursive();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_close(actual: f32, expected: f32) -> bool {
        (actual - expected).abs() < 1e-5
    }

    #[test]
    fn pickaxe_sweeps_from_raised_to_struck_and_mirrors_for_left_swings() {
        assert!(is_close(
            pickaxe_clockwise_angle_from_upright(0.0, false),
            RAISED_ANGLE
        ));
        assert!(is_close(
            pickaxe_clockwise_angle_from_upright(1.0, false),
            STRUCK_ANGLE
        ));
        let halfway = pickaxe_clockwise_angle_from_upright(0.5, false);
        assert!(halfway > RAISED_ANGLE && halfway < STRUCK_ANGLE);
        assert!(is_close(
            pickaxe_clockwise_angle_from_upright(0.3, true),
            -pickaxe_clockwise_angle_from_upright(0.3, false)
        ));
    }
}
