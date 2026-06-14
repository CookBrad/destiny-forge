use bevy::prelude::*;

mod combat;
mod core;
mod dungeon;
mod graphics;
mod items;
mod overworld;
mod forging;
mod player;
mod progression;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Destiny Forge".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            core::CorePlugin,
            graphics::GraphicsPlugin,
            combat::CombatPlugin,
            items::ItemsPlugin,
            player::PlayerPlugin,
            progression::ProgressionPlugin,
            dungeon::DungeonPlugin,
            forging::ForgingPlugin,
            overworld::OverworldPlugin,
            ui::UiPlugin,
        ))
        .run();
}