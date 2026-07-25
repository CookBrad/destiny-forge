//! Ore nodes and mine entrance on the homestead.

use bevy::prelude::*;

use crate::graphics::{world_transform, TILE};
use crate::overworld::layout::OverworldEntity;

use super::logic::SOFT_IRON_HARDNESS;

/// Interactable ore deposit. Mined with pickaxe when hardness is met.
#[derive(Component, Clone, Debug)]
pub struct OreNode {
    pub hardness: u32,
    /// False after a successful break (node depleted until sleep/respawn).
    pub intact: bool,
}

impl Default for OreNode {
    fn default() -> Self {
        Self {
            hardness: SOFT_IRON_HARDNESS,
            intact: true,
        }
    }
}

/// Landmark marker for the mine mouth (prompt / zone).
#[derive(Component)]
pub struct MineEntrance;

/// Tile centers for soft iron nodes east of the animal pen.
pub const ORE_NODE_TILES: [(u32, u32); 4] = [(48, 21), (50, 22), (49, 24), (47, 23)];

/// Spawn mine mouth + ore nodes. Call from homestead setup.
pub fn spawn_mine_area(commands: &mut Commands, wall: Handle<Image>, path: Handle<Image>) {
    // Cave mouth / entrance landmark (visual).
    let entrance = Vec2::new(49.0 * TILE + TILE * 0.5, 26.0 * TILE + TILE * 0.5);
    commands.spawn((
        Sprite {
            image: wall.clone(),
            color: Color::srgb(0.28, 0.26, 0.3),
            custom_size: Some(Vec2::new(TILE * 3.2, TILE * 2.4)),
            ..default()
        },
        world_transform(entrance, 1.5),
        MineEntrance,
        OverworldEntity,
    ));
    // Dark mouth opening
    commands.spawn((
        Sprite {
            image: path,
            color: Color::srgb(0.08, 0.07, 0.1),
            custom_size: Some(Vec2::new(TILE * 1.4, TILE * 1.1)),
            ..default()
        },
        world_transform(entrance + Vec2::new(0.0, -TILE * 0.15), 1.6),
        MineEntrance,
        OverworldEntity,
    ));

    for (tx, ty) in ORE_NODE_TILES {
        let center = Vec2::new(tx as f32 * TILE + TILE * 0.5, ty as f32 * TILE + TILE * 0.5);
        commands.spawn((
            Sprite {
                image: wall.clone(),
                color: Color::srgb(0.42, 0.44, 0.48),
                custom_size: Some(Vec2::new(TILE * 0.85, TILE * 0.75)),
                ..default()
            },
            world_transform(center, 1.4),
            OreNode::default(),
            OverworldEntity,
        ));
        // Ore flecks
        commands.spawn((
            Sprite {
                image: wall.clone(),
                color: Color::srgb(0.55, 0.58, 0.62),
                custom_size: Some(Vec2::new(TILE * 0.35, TILE * 0.28)),
                ..default()
            },
            world_transform(center + Vec2::new(TILE * 0.08, TILE * 0.05), 1.45),
            OverworldEntity,
        ));
    }
}

/// Restore depleted nodes (e.g. on sleep).
pub fn respawn_all_ore_nodes(mut nodes: Query<(&mut OreNode, &mut Sprite)>) {
    for (mut node, mut sprite) in &mut nodes {
        if !node.intact {
            node.intact = true;
            sprite.color = Color::srgb(0.42, 0.44, 0.48);
        }
    }
}
