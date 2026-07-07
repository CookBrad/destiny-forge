use bevy::prelude::*;

use crate::graphics::{center_on_surface, world_transform};
use crate::overworld::layout::tile_center;
use crate::overworld::movement::{
    ExplorationMap, MapTransitionCooldown, OverworldPlayer, OverworldVelocity,
};
use crate::overworld::sprites::{OverworldArt, PLAYER_SPRITE_HEIGHT};

use super::layout::{spawn_forest, ForestLayout};

pub fn setup_forest(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let forest_art = super::sprites::ForestArt::load(&asset_server);
    let overworld_art = OverworldArt::load(&asset_server, &mut atlas_layouts);
    let layout = ForestLayout::generate();

    spawn_forest(&mut commands, &forest_art, &layout);
    spawn_forest_player(&mut commands, &overworld_art);

    commands.insert_resource(ExplorationMap {
        solids: layout.solids(),
        world_width: super::layout::WORLD_WIDTH,
        world_height: super::layout::WORLD_HEIGHT,
    });
    commands.insert_resource(MapTransitionCooldown::default());
    commands.insert_resource(forest_art);
    commands.insert_resource(overworld_art);
    commands.insert_resource(layout);
}

fn spawn_forest_player(commands: &mut Commands, art: &OverworldArt) {
    let start = tile_center(3, 4);
    let y = center_on_surface(start.y, PLAYER_SPRITE_HEIGHT);

    commands.spawn((
        Sprite {
            image: art.player.idle[0].clone(),
            ..default()
        },
        world_transform(Vec2::new(start.x, y), 5.0),
        OverworldPlayer,
        OverworldVelocity::default(),
    ));
}

pub fn cleanup_forest(
    mut commands: Commands,
    entities: Query<Entity, With<super::layout::ForestEntity>>,
    players: Query<Entity, With<OverworldPlayer>>,
) {
    for entity in entities.iter().chain(players.iter()) {
        commands.entity(entity).try_despawn_recursive();
    }
    commands.remove_resource::<super::sprites::ForestArt>();
    commands.remove_resource::<OverworldArt>();
    commands.remove_resource::<ForestLayout>();
    commands.remove_resource::<ExplorationMap>();
    commands.remove_resource::<MapTransitionCooldown>();
}