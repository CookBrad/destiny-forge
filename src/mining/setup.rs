use bevy::prelude::*;

use crate::core::PlayerProfile;
use crate::farming::PlayerFacing;
use crate::graphics::{center_on_surface, world_transform};
use crate::overworld::camera::HOMESTEAD_PIXEL_ZOOM;
use crate::overworld::layout::tile_center;
use crate::overworld::movement::{
    ExplorationMap, MapTransitionCooldown, OverworldPlayer, OverworldVelocity,
};
use crate::overworld::sprites::{OverworldArt, PLAYER_SPRITE_HEIGHT};

use super::layout::{mine_solids, spawn_mine, MineEntity, ARRIVAL_TILE, WORLD_HEIGHT, WORLD_WIDTH};
use super::swing::DwarfPickaxeSwingFrames;

pub fn set_mine_clear_color(mut clear: ResMut<ClearColor>) {
    clear.0 = Color::srgb(0.06, 0.05, 0.06);
}

pub fn setup_mine(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut images: ResMut<Assets<Image>>,
    profile: Res<PlayerProfile>,
) {
    let art = OverworldArt::load(&asset_server, &mut atlas_layouts, &mut images);
    spawn_mine(&mut commands, &art, &profile.depleted_ore_node_ids);

    let start = tile_center(ARRIVAL_TILE.0, ARRIVAL_TILE.1);
    commands.spawn((
        Sprite {
            image: art.player.idle[0].clone(),
            ..default()
        },
        world_transform(
            Vec2::new(start.x, center_on_surface(start.y, PLAYER_SPRITE_HEIGHT)),
            5.0,
        ),
        OverworldPlayer,
        OverworldVelocity::default(),
        PlayerFacing::default(),
    ));

    commands.insert_resource(ExplorationMap {
        solids: mine_solids(),
        world_width: WORLD_WIDTH,
        world_height: WORLD_HEIGHT,
        pixel_zoom: HOMESTEAD_PIXEL_ZOOM,
    });
    commands.insert_resource(MapTransitionCooldown::default());
    commands.insert_resource(art);
    commands.insert_resource(DwarfPickaxeSwingFrames::load(&asset_server));
}

pub fn cleanup_mine(
    mut commands: Commands,
    entities: Query<Entity, With<MineEntity>>,
    players: Query<Entity, With<OverworldPlayer>>,
) {
    for entity in entities.iter().chain(players.iter()) {
        commands.entity(entity).try_despawn_recursive();
    }
    commands.remove_resource::<OverworldArt>();
    commands.remove_resource::<DwarfPickaxeSwingFrames>();
    commands.remove_resource::<ExplorationMap>();
    commands.remove_resource::<MapTransitionCooldown>();
}
