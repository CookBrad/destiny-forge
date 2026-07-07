use bevy::prelude::*;

use crate::graphics::{center_on_surface, world_transform, TILE};

use super::layout::{spawn_homestead, tile_center, OverworldLayout, WORLD_WIDTH};
use super::movement::{ExplorationMap, MapTransitionCooldown, OverworldPlayer};
use super::sprites::{OverworldArt, PLAYER_SPRITE_HEIGHT};

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq)]
pub enum OverworldEntry {
    #[default]
    Yard,
    ForestTrail,
}

pub fn setup_overworld(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    entry: Option<Res<OverworldEntry>>,
) {
    let art = OverworldArt::load(&asset_server, &mut atlas_layouts);
    let layout = OverworldLayout::homestead();
    let spawn = entry.map(|entry| *entry).unwrap_or_default();

    spawn_homestead(&mut commands, &art);
    spawn_overworld_player(&mut commands, &art, spawn);

    commands.insert_resource(ExplorationMap {
        solids: layout.solids.clone(),
        world_width: super::layout::WORLD_WIDTH,
        world_height: super::layout::WORLD_HEIGHT,
    });
    commands.insert_resource(MapTransitionCooldown::default());
    commands.insert_resource(art);
    commands.insert_resource(layout);
    commands.remove_resource::<OverworldEntry>();
}

fn spawn_overworld_player(commands: &mut Commands, art: &OverworldArt, entry: OverworldEntry) {
    let start = match entry {
        OverworldEntry::Yard => Vec2::new(WORLD_WIDTH * 0.5, TILE * 12.0),
        OverworldEntry::ForestTrail => tile_center(3, 36),
    };
    let y = center_on_surface(start.y, PLAYER_SPRITE_HEIGHT);

    commands.spawn((
        Sprite {
            image: art.player.idle[0].clone(),
            ..default()
        },
        world_transform(Vec2::new(start.x, y), 5.0),
        OverworldPlayer,
        super::movement::OverworldVelocity::default(),
    ));
}

pub fn cleanup_overworld(
    mut commands: Commands,
    entities: Query<Entity, With<super::layout::OverworldEntity>>,
    players: Query<Entity, With<OverworldPlayer>>,
) {
    for entity in entities.iter().chain(players.iter()) {
        commands.entity(entity).try_despawn_recursive();
    }
    commands.remove_resource::<OverworldArt>();
    commands.remove_resource::<OverworldLayout>();
    commands.remove_resource::<ExplorationMap>();
    commands.remove_resource::<MapTransitionCooldown>();
}