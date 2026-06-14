use bevy::prelude::*;
use bevy::image::ImageSampler;

use crate::core::GameState;

use super::atlas::{
    GameSprites, PLAYER_FRAME_HEIGHT, PLAYER_FRAME_WIDTH, PLAYER_SHEET_COLUMNS, PLAYER_SHEET_ROWS,
    TILE_SIZE,
};
use super::spawn::PIXEL_SCALE;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_game_sprites)
            .add_systems(PostStartup, enter_hub_after_assets_load)
            .add_systems(Update, configure_pixel_art_filtering);
    }
}

fn enter_hub_after_assets_load(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Hub);
}

fn load_game_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let hub_tiles = asset_server.load("sprites/hub_tiles.png");
    let dungeon_sheet = asset_server.load("sprites/dungeon_sheet.png");
    let player = asset_server.load("sprites/player.png");
    let forge_building = asset_server.load("sprites/forge.png");
    let mine_entrance = asset_server.load("sprites/mine_entrance.png");
    let hub_background = asset_server.load("sprites/hub_background.png");
    let dungeon_background = asset_server.load("sprites/dungeon_background.png");

    let hub_tiles_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(TILE_SIZE),
        8,
        4,
        None,
        None,
    ));

    let dungeon_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(TILE_SIZE),
        12,
        4,
        None,
        None,
    ));

    let player_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(PLAYER_FRAME_WIDTH, PLAYER_FRAME_HEIGHT),
        PLAYER_SHEET_COLUMNS,
        PLAYER_SHEET_ROWS,
        None,
        None,
    ));

    commands.insert_resource(GameSprites {
        hub_tiles,
        hub_tiles_layout,
        dungeon_sheet,
        dungeon_layout,
        player,
        player_layout,
        forge_building,
        mine_entrance,
        hub_background,
        dungeon_background,
    });

    commands.insert_resource(ClearColor(Color::srgb(0.45, 0.68, 0.88)));
    info!(
        "Loaded sprites (player frames: {}x{}) at {}x scale",
        PLAYER_FRAME_WIDTH, PLAYER_FRAME_HEIGHT, PIXEL_SCALE
    );
}

fn configure_pixel_art_filtering(
    sprites: Res<GameSprites>,
    mut images: ResMut<Assets<Image>>,
    mut configured: Local<bool>,
) {
    if *configured {
        return;
    }

    let handles = [
        sprites.hub_tiles.clone(),
        sprites.dungeon_sheet.clone(),
        sprites.player.clone(),
        sprites.forge_building.clone(),
        sprites.mine_entrance.clone(),
        sprites.hub_background.clone(),
        sprites.dungeon_background.clone(),
    ];

    if !handles.iter().all(|handle| images.get(handle).is_some()) {
        return;
    }

    for handle in handles {
        apply_nearest_neighbor_filter(&mut images, &handle);
    }

    *configured = true;
}

fn apply_nearest_neighbor_filter(images: &mut Assets<Image>, handle: &Handle<Image>) {
    let Some(image) = images.get_mut(handle) else {
        return;
    };

    image.sampler = ImageSampler::nearest();
}