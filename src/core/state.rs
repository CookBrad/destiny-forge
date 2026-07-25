use bevy::prelude::*;

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    #[default]
    Title,
    Overworld,
    Forest,
    /// Lakeside exploration + Stardew-style fishing pier.
    Lake,
    Dungeon,
}

#[derive(SubStates, Default, Clone, Copy, Eq, PartialEq, Hash, Debug)]
#[source(GameState = GameState::Dungeon)]
pub enum DungeonPlayState {
    #[default]
    Running,
    Paused,
    Dying,
    Dead,
}