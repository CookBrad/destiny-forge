mod audio;
mod combat;
mod core;
mod dungeon;
mod forging;
mod graphics;
mod items;
mod player;
mod ui;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(Color::srgb(0.08, 0.07, 0.1)))
        .add_plugins(core::CorePlugin)
        .add_plugins(items::ItemsPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(forging::ForgingPlugin)
        .add_plugins(audio::GameAudioPlugin)
        .add_plugins(combat::CombatPlugin)
        .add_plugins(graphics::GraphicsPlugin)
        .add_plugins(dungeon::DungeonPlugin)
        .add_plugins(ui::UiPlugin)
        .run();
}