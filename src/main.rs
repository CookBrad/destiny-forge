mod combat;
mod core;
mod dungeon;
mod graphics;
mod ui;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(Color::srgb(0.08, 0.07, 0.1)))
        .add_plugins(core::CorePlugin)
        .add_plugins(combat::CombatPlugin)
        .add_plugins(graphics::GraphicsPlugin)
        .add_plugins(dungeon::DungeonPlugin)
        .add_plugins(ui::UiPlugin)
        .run();
}