use bevy::prelude::*;

use crate::core::{
    perform_sleep, DayClock, GameState, PlayerProfile, ProfileDirty, ToolEnergy,
};
use crate::player::Loadout;
use crate::ui::forge_window::{open_forge_window, ForgeSelectedRecipe, ForgeWindowOpen};
use crate::ui::inventory_window::InventoryWindowOpen;
use crate::graphics::INTERACT_DISTANCE;

use super::layout::{homestead_forest_transition, Bed, HomesteadZone, OverworldLayout};
use super::movement::{MapTransitionCooldown, OverworldPlayer, OverworldVelocity};

pub fn overworld_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<InventoryWindowOpen>,
    mut commands: Commands,
    game_inventory: Res<crate::items::Inventory>,
    forge_windows: Query<Entity, With<crate::ui::forge_window::ForgeWindow>>,
    mut time: ResMut<Time<Virtual>>,
    mut forge: ResMut<ForgeWindowOpen>,
    mut forge_recipe: ResMut<ForgeSelectedRecipe>,
    loadout: Res<Loadout>,
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
                    &mut forge_recipe,
                    &mut commands,
                    &game_inventory,
                    &loadout,
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

/// Hold E near the house bed to end the day.
pub fn try_sleep_at_bed(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<InventoryWindowOpen>,
    forge: Res<ForgeWindowOpen>,
    player: Query<&Transform, With<OverworldPlayer>>,
    beds: Query<&Transform, With<Bed>>,
    mut day_clock: ResMut<DayClock>,
    mut tool_energy: ResMut<ToolEnergy>,
    mut profile: ResMut<PlayerProfile>,
    mut profile_dirty: ResMut<ProfileDirty>,
    mut clear: ResMut<ClearColor>,
) {
    if inventory.0 || forge.0 || !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let position = player_transform.translation.truncate();
    if !beds.iter().any(|bed| {
        position.distance(bed.translation.truncate()) <= INTERACT_DISTANCE * 1.5
    }) {
        return;
    }

    let day = perform_sleep(&mut day_clock, &mut tool_energy);
    profile.calendar_day = day_clock.calendar_day;
    profile.day_phase = day_clock.phase;
    profile_dirty.mark();
    clear.0 = day_clock.phase.ambient_clear_color();
    info!("Slept — morning of day {day}. Tool energy restored.");
}

fn distance_to_zone(position: Vec2, bounds: &Rect) -> f32 {
    position.distance(bounds.center())
}
