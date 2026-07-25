//! Fishing dock / pond landmark on the homestead.

use bevy::prelude::*;

use crate::graphics::{world_transform, TILE};
use crate::overworld::layout::OverworldEntity;

/// Player must be near this to cast.
#[derive(Component)]
pub struct FishingSpot;

/// Pond + dock tiles (southeast, below animal pen / east of dungeon path).
pub const POND_TILES: [(u32, u32); 6] = [
    (45, 2),
    (46, 2),
    (47, 2),
    (45, 3),
    (46, 3),
    (47, 3),
];

pub const DOCK_TILE: (u32, u32) = (44, 3);

pub fn spawn_fishing_spot(commands: &mut Commands, path: Handle<Image>, grass: Handle<Image>) {
    let water = Color::srgb(0.18, 0.38, 0.55);
    for (tx, ty) in POND_TILES {
        let center = Vec2::new(tx as f32 * TILE + TILE * 0.5, ty as f32 * TILE + TILE * 0.5);
        commands.spawn((
            Sprite {
                image: grass.clone(),
                color: water,
                custom_size: Some(Vec2::new(TILE * 1.02, TILE * 1.02)),
                ..default()
            },
            world_transform(center, 0.3),
            OverworldEntity,
        ));
    }

    // Dock planks
    let (dtx, dty) = DOCK_TILE;
    let dock_center = Vec2::new(
        dtx as f32 * TILE + TILE * 0.5,
        dty as f32 * TILE + TILE * 0.5,
    );
    commands.spawn((
        Sprite {
            image: path,
            color: Color::srgb(0.48, 0.36, 0.22),
            custom_size: Some(Vec2::new(TILE * 1.1, TILE * 0.7)),
            ..default()
        },
        world_transform(dock_center, 1.2),
        FishingSpot,
        OverworldEntity,
    ));
    // Pole marker
    commands.spawn((
        Sprite {
            image: grass,
            color: Color::srgb(0.55, 0.42, 0.28),
            custom_size: Some(Vec2::new(TILE * 0.18, TILE * 1.1)),
            ..default()
        },
        world_transform(dock_center + Vec2::new(TILE * 0.35, TILE * 0.35), 1.3),
        FishingSpot,
        OverworldEntity,
    ));
}

/// Distance from player to nearest fishing spot.
pub fn nearest_spot_distance(
    player_pos: Vec2,
    spots: &Query<&Transform, With<FishingSpot>>,
) -> Option<f32> {
    spots
        .iter()
        .map(|t| player_pos.distance(t.translation.truncate()))
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
}

