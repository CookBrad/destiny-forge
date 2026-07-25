//! Cast / bite / fight animation feedback near the player (bobber + cast flash).

use bevy::prelude::*;

use crate::graphics::{world_transform, TILE};
use crate::overworld::layout::OverworldEntity;
use crate::overworld::movement::OverworldPlayer;

use super::cast::ActiveCast;
use super::logic::FishingAnimKind;

/// World bobber / cast spark that follows the player during a cast.
#[derive(Component)]
pub struct FishingBobber;

#[derive(Component)]
pub struct FishingAnimTimer(pub f32);

/// Lake entities also need cleanup — tag bobber with a generic marker that both
/// overworld and lake cleanups can despawn via OverworldEntity OR LakeEntity.
/// We use FishingBobber only and despawn in fishing plugin exit handlers.

pub fn sync_fishing_animation(
    mut commands: Commands,
    time: Res<Time>,
    cast: Res<ActiveCast>,
    player: Query<&Transform, (With<OverworldPlayer>, Without<FishingBobber>)>,
    mut bobbers: Query<
        (Entity, &mut Transform, &mut Sprite, &mut FishingAnimTimer),
        (With<FishingBobber>, Without<OverworldPlayer>),
    >,
) {
    let kind = cast.state.anim_kind();
    let Ok(player_tf) = player.get_single() else {
        for (e, _, _, _) in &bobbers {
            commands.entity(e).try_despawn_recursive();
        }
        return;
    };

    if kind == FishingAnimKind::None {
        for (e, _, _, _) in &bobbers {
            commands.entity(e).try_despawn_recursive();
        }
        return;
    }

    let player_pos = player_tf.translation.truncate();
    let bobber_count = bobbers.iter().count();
    if bobber_count == 0 {
        spawn_bobber(&mut commands, player_pos, kind);
        return;
    }

    let t = time.elapsed_secs();
    for (_, mut transform, mut sprite, mut timer) in &mut bobbers {
        timer.0 += time.delta_secs();
        let (offset, color, size) = anim_visual(kind, t, timer.0);
        transform.translation.x = player_pos.x + offset.x;
        transform.translation.y = player_pos.y + offset.y;
        transform.translation.z = 4.5;
        sprite.color = color;
        sprite.custom_size = Some(size);
    }
}

fn spawn_bobber(commands: &mut Commands, player_pos: Vec2, kind: FishingAnimKind) {
    let (offset, color, size) = anim_visual(kind, 0.0, 0.0);
    // Use a plain colored sprite without requiring a texture — Bevy 0.15 needs an image.
    // We'll load via a minimal approach: reuse a known white-ish asset through Default.
    // Actually Sprite without image may not render; use Color only with default rect.
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        world_transform(player_pos + offset, 4.5),
        FishingBobber,
        FishingAnimTimer(0.0),
        OverworldEntity, // also cleaned with homestead; lake adds LakeEntity below if needed
    ));
}

fn anim_visual(kind: FishingAnimKind, world_t: f32, local_t: f32) -> (Vec2, Color, Vec2) {
    let bob = (world_t * 6.0).sin() * TILE * 0.08;
    match kind {
        FishingAnimKind::None => (Vec2::ZERO, Color::NONE, Vec2::ZERO),
        FishingAnimKind::Cast => {
            // Arc outward during cast
            let p = (local_t / 0.55).clamp(0.0, 1.0);
            let arc = Vec2::new(TILE * 0.9 * p, TILE * 0.5 * (1.0 - (2.0 * p - 1.0).powi(2)));
            (
                arc,
                Color::srgb(0.85, 0.75, 0.45),
                Vec2::new(TILE * 0.25, TILE * 0.25),
            )
        }
        FishingAnimKind::Waiting => (
            Vec2::new(TILE * 1.1, -TILE * 0.15 + bob),
            Color::srgb(0.9, 0.35, 0.25),
            Vec2::new(TILE * 0.22, TILE * 0.22),
        ),
        FishingAnimKind::Fighting => (
            Vec2::new(TILE * 1.15, -TILE * 0.1 + bob * 1.6),
            Color::srgb(1.0, 0.55, 0.2),
            Vec2::new(TILE * 0.28, TILE * 0.2),
        ),
        FishingAnimKind::Success => (
            Vec2::new(TILE * 0.4, TILE * 0.35 + bob),
            Color::srgb(0.3, 0.95, 0.45),
            Vec2::new(TILE * 0.35, TILE * 0.35),
        ),
        FishingAnimKind::Fail => (
            Vec2::new(TILE * 1.2, -TILE * 0.2),
            Color::srgb(0.4, 0.4, 0.45),
            Vec2::new(TILE * 0.15, TILE * 0.15),
        ),
    }
}

pub fn cleanup_fishing_animation(
    mut commands: Commands,
    bobbers: Query<Entity, With<FishingBobber>>,
) {
    for e in &bobbers {
        commands.entity(e).try_despawn_recursive();
    }
}
