use bevy::prelude::*;

use crate::core::GameState;
use crate::items::MaterialInventory;
use crate::overworld::ForgeStation;
use crate::player::{HubPlayer, PlayerLoadout};

use super::craft::{try_craft_selected_recipe, CraftResult};
use super::recipe::ForgeRecipeBook;

pub struct ForgingPlugin;

impl Plugin for ForgingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ForgeRecipeBook>()
            .add_systems(
                Update,
                (cycle_forge_recipes, craft_at_forge)
                    .chain()
                    .run_if(in_state(GameState::Hub)),
            );
    }
}

fn cycle_forge_recipes(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut recipe_book: ResMut<ForgeRecipeBook>,
    player_query: Query<&Transform, With<HubPlayer>>,
    forge_query: Query<&Transform, With<ForgeStation>>,
) {
    if !is_player_near_forge(&player_query, &forge_query) {
        return;
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        recipe_book.select_previous();
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        recipe_book.select_next();
    }
}

fn craft_at_forge(
    keyboard: Res<ButtonInput<KeyCode>>,
    recipe_book: Res<ForgeRecipeBook>,
    mut inventory: ResMut<MaterialInventory>,
    mut loadout: ResMut<PlayerLoadout>,
    player_query: Query<&Transform, With<HubPlayer>>,
    forge_query: Query<&Transform, With<ForgeStation>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }

    if !is_player_near_forge(&player_query, &forge_query) {
        return;
    }

    let result = try_craft_selected_recipe(&recipe_book, &mut inventory, &mut loadout);
    log_craft_result(result);
}

fn is_player_near_forge(
    player_query: &Query<&Transform, With<HubPlayer>>,
    forge_query: &Query<&Transform, With<ForgeStation>>,
) -> bool {
    let Ok(player_transform) = player_query.get_single() else {
        return false;
    };

    forge_query.iter().any(|forge_transform| {
        player_transform
            .translation
            .truncate()
            .distance(forge_transform.translation.truncate())
            < 72.0
    })
}

fn log_craft_result(result: CraftResult) {
    match result {
        CraftResult::Success => info!("Craft successful"),
        CraftResult::MissingMaterials => info!("Missing materials"),
        CraftResult::MissingWeapon => info!("Required base weapon not equipped"),
        CraftResult::AlreadyOwned => info!("Recipe already crafted and equipped"),
    }
}