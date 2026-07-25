//! House stove: cycle recipes and cook with E/F when nearby.

use bevy::prelude::*;

use crate::core::ProfileDirty;
use crate::graphics::{world_transform, TILE};
use crate::items::Inventory;
use crate::overworld::layout::OverworldEntity;
use crate::overworld::movement::OverworldPlayer;
use crate::ui::forge_window::ForgeWindowOpen;
use crate::ui::inventory_window::InventoryWindowOpen;

use super::recipes::{can_cook, cook_recipe_at, cook_recipe_count, try_cook, COOK_RECIPES};

const STOVE_RANGE: f32 = TILE * 1.6;

#[derive(Component)]
pub struct CookStove;

#[derive(Resource, Clone, Debug, Default)]
pub struct CookSelectedRecipe(pub usize);

/// Spawn stove inside the house (south of bed area).
pub fn spawn_cook_stove(commands: &mut Commands, wall: Handle<Image>, path: Handle<Image>) {
    // House interior ~ tiles 5-12 x 31-36; stove on west wall.
    let center = Vec2::new(6.0 * TILE + TILE * 0.5, 33.0 * TILE + TILE * 0.5);
    commands.spawn((
        Sprite {
            image: wall,
            color: Color::srgb(0.35, 0.32, 0.3),
            custom_size: Some(Vec2::new(TILE * 1.1, TILE * 0.9)),
            ..default()
        },
        world_transform(center, 1.55),
        CookStove,
        OverworldEntity,
    ));
    // Ember glow
    commands.spawn((
        Sprite {
            image: path,
            color: Color::srgb(0.85, 0.4, 0.15),
            custom_size: Some(Vec2::new(TILE * 0.45, TILE * 0.3)),
            ..default()
        },
        world_transform(center + Vec2::new(0.0, -TILE * 0.15), 1.6),
        OverworldEntity,
    ));
}

pub fn near_stove(player_pos: Vec2, stoves: &Query<&Transform, With<CookStove>>) -> bool {
    stoves
        .iter()
        .any(|t| player_pos.distance(t.translation.truncate()) <= STOVE_RANGE)
}

/// Cycle cook recipe with Up/Down and craft with F while near stove.
pub fn cook_station_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory_open: Res<InventoryWindowOpen>,
    forge_open: Res<ForgeWindowOpen>,
    mut selected: ResMut<CookSelectedRecipe>,
    mut inventory: ResMut<Inventory>,
    mut dirty: ResMut<ProfileDirty>,
    player: Query<&Transform, With<OverworldPlayer>>,
    stoves: Query<&Transform, With<CookStove>>,
) {
    if inventory_open.0 || forge_open.0 {
        return;
    }
    let Ok(transform) = player.get_single() else {
        return;
    };
    if !near_stove(transform.translation.truncate(), &stoves) {
        return;
    }

    let count = cook_recipe_count();
    if count == 0 {
        return;
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        selected.0 = (selected.0 + count - 1) % count;
        if let Some(r) = cook_recipe_at(selected.0) {
            info!("Cook recipe: {} — {:?}", r.name, r.costs);
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        selected.0 = (selected.0 + 1) % count;
        if let Some(r) = cook_recipe_at(selected.0) {
            info!("Cook recipe: {} — {:?}", r.name, r.costs);
        }
    }

    if keyboard.just_pressed(KeyCode::KeyF) {
        let Some(recipe) = cook_recipe_at(selected.0) else {
            return;
        };
        if try_cook(&mut inventory, recipe) {
            dirty.mark();
            info!(
                "Cooked {} — drag to hotbar and Space to eat.",
                recipe.name
            );
        } else if can_cook(&inventory, recipe) {
            info!("Inventory full — free a slot for {}.", recipe.name);
        } else {
            info!(
                "Need materials for {}: {}",
                recipe.name,
                recipe
                    .costs
                    .iter()
                    .map(|(m, n)| format!("{}×{}", n, m.display_name()))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    // First approach: log available recipes once via E.
    if keyboard.just_pressed(KeyCode::KeyE) {
        let recipe = &COOK_RECIPES[selected.0.min(count - 1)];
        let ready = if can_cook(&inventory, recipe) {
            "ready"
        } else {
            "missing mats"
        };
        info!(
            "Stove: {} ({}/{}) — {} · F to cook · Up/Down cycle",
            recipe.name,
            selected.0 + 1,
            count,
            ready
        );
    }
}
