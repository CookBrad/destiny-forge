use bevy::prelude::*;

#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    #[default]
    Right,
    Left,
}