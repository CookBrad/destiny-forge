mod audio;
mod combat;
mod core;
mod dungeon;
mod exploration;
mod farming;
mod forest;
mod forging;
mod graphics;
mod items;
mod overworld;
mod player;
mod ui;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(Color::srgb(0.08, 0.07, 0.1)))
        .add_plugins(core::CorePlugin)
        .add_plugins(audio::GameAudioPlugin)
        .add_plugins(graphics::GraphicsPlugin)
        .add_plugins(dungeon::DungeonPlugin)
        .add_plugins(overworld::OverworldPlugin)
        .add_plugins(farming::FarmingPlugin)
        .add_plugins(forest::ForestPlugin)
        .add_plugins(ui::UiPlugin)
        .run();
}