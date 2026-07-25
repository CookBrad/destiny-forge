mod audio;
mod combat;
mod cooking;
mod core;
mod dungeon;
mod exploration;
mod farming;
mod fishing;
mod forest;
mod forging;
mod graphics;
mod items;
mod mining;
mod overworld;
mod player;
mod ui;

use bevy::prelude::*;

fn main() {
    App::new()
        // Default linear sampling for higher-resolution 2D (not forced nearest/pixel look).
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.08, 0.07, 0.1)))
        .add_plugins(core::CorePlugin)
        .add_plugins(audio::GameAudioPlugin)
        .add_plugins(graphics::GraphicsPlugin)
        .add_plugins(dungeon::DungeonPlugin)
        .add_plugins(overworld::OverworldPlugin)
        .add_plugins(farming::FarmingPlugin)
        .add_plugins(mining::MiningPlugin)
        .add_plugins(fishing::FishingPlugin)
        .add_plugins(cooking::CookingPlugin)
        .add_plugins(forest::ForestPlugin)
        .add_plugins(ui::UiPlugin)
        .run();
}