use bevy::prelude::*;

use crate::graphics::{center_on_surface, world_transform};
use crate::overworld::layout::tile_center;
use crate::overworld::movement::{
    ExplorationMap, MapTransitionCooldown, OverworldPlayer, OverworldVelocity,
};
use crate::overworld::sprites::{OverworldArt, PLAYER_SPRITE_HEIGHT};

use super::layout::{spawn_lake, LakeLayout, WORLD_HEIGHT, WORLD_WIDTH};

pub fn setup_lake(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let overworld_art = OverworldArt::load(&asset_server, &mut atlas_layouts);
    let layout = LakeLayout::generate();

    spawn_lake(
        &mut commands,
        overworld_art.grass.clone(),
        overworld_art.path.clone(),
        overworld_art.wall.clone(),
    );
    spawn_lake_player(&mut commands, &overworld_art);

    commands.insert_resource(ExplorationMap {
        solids: layout.solids(),
        world_width: WORLD_WIDTH,
        world_height: WORLD_HEIGHT,
    });
    commands.insert_resource(MapTransitionCooldown::default());
    commands.insert_resource(overworld_art);
    commands.insert_resource(layout);
}

fn spawn_lake_player(commands: &mut Commands, art: &OverworldArt) {
    // Enter from west trail
    let start = tile_center(4, 12);
    let y = center_on_surface(start.y, PLAYER_SPRITE_HEIGHT);

    commands.spawn((
        Sprite {
            image: art.player.idle[0].clone(),
            ..default()
        },
        world_transform(Vec2::new(start.x, y), 5.0),
        OverworldPlayer,
        OverworldVelocity::default(),
        crate::farming::PlayerFacing::default(),
    ));
}

pub fn cleanup_lake(
    mut commands: Commands,
    entities: Query<Entity, With<super::layout::LakeEntity>>,
    players: Query<Entity, With<OverworldPlayer>>,
    bobbers: Query<Entity, With<crate::fishing::FishingBobber>>,
) {
    for entity in entities.iter().chain(players.iter()).chain(bobbers.iter()) {
        commands.entity(entity).try_despawn_recursive();
    }
    commands.remove_resource::<OverworldArt>();
    commands.remove_resource::<LakeLayout>();
    commands.remove_resource::<ExplorationMap>();
    commands.remove_resource::<MapTransitionCooldown>();
}
