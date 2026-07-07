use bevy::prelude::*;

use crate::core::GameState;
use crate::ui::forge_window::{open_forge_window, ForgeWindowOpen};
use crate::ui::inventory_window::InventoryWindowOpen;
use crate::graphics::INTERACT_DISTANCE;

use super::layout::{homestead_forest_transition, HomesteadZone, OverworldLayout};
use super::movement::{MapTransitionCooldown, OverworldPlayer, OverworldVelocity};

pub fn overworld_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<InventoryWindowOpen>,
    mut commands: Commands,
    game_inventory: Res<crate::items::Inventory>,
    forge_windows: Query<Entity, With<crate::ui::forge_window::ForgeWindow>>,
    mut time: ResMut<Time<Virtual>>,
    mut forge: ResMut<ForgeWindowOpen>,
    layout: Res<OverworldLayout>,
    cooldown: Res<MapTransitionCooldown>,
    player: Query<(&Transform, &OverworldVelocity), With<OverworldPlayer>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok((transform, velocity)) = player.get_single() else {
        return;
    };

    if inventory.0 || forge.0 {
        return;
    }

    let position = transform.translation.truncate();

    if let Some(zone) = layout.zone_at(position) {
        let near = distance_to_zone(position, &zone.bounds) <= INTERACT_DISTANCE * 2.0;
        match zone.zone {
            HomesteadZone::Forge if near && keyboard.just_pressed(KeyCode::KeyE) => {
                open_forge_window(
                    &mut forge,
                    &mut commands,
                    &game_inventory,
                    &forge_windows,
                    &mut time,
                );
            }
            HomesteadZone::DungeonGate if near && keyboard.just_pressed(KeyCode::KeyE) => {
                next_state.set(GameState::Dungeon);
            }
            _ => {}
        }
    }

    if cooldown.0.finished()
        && homestead_forest_transition().contains(position)
        && velocity.y > 1.0
    {
        next_state.set(GameState::Forest);
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Title);
    }
}

fn distance_to_zone(position: Vec2, bounds: &Rect) -> f32 {
    position.distance(bounds.center())
}