use bevy::prelude::*;

use crate::combat::Health;
use crate::core::GameState;
use crate::forging::ForgeRecipeBook;
use crate::items::MaterialInventory;
use crate::overworld::ForgeStation;
use crate::player::{DungeonPlayer, HubPlayer, PlayerLoadout};
use crate::progression::SlimeSet;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct HudText;

pub fn setup_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
            HudRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                HudText,
            ));
        });
}

pub fn update_hud(
    state: Res<State<GameState>>,
    inventory: Res<MaterialInventory>,
    loadout: Res<PlayerLoadout>,
    recipe_book: Res<ForgeRecipeBook>,
    player_health: Query<&Health, With<DungeonPlayer>>,
    hub_player: Query<&Transform, With<HubPlayer>>,
    forge_query: Query<&Transform, With<ForgeStation>>,
    mut text_query: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    **text = match state.get() {
        GameState::Hub => hub_hud_text(
            &inventory,
            &loadout,
            &recipe_book,
            &hub_player,
            &forge_query,
        ),
        GameState::Dungeon => dungeon_hud_text(&inventory, &loadout, &player_health),
        GameState::AssetLoading => "Loading assets...".to_string(),
    };
}

fn hub_hud_text(
    inventory: &MaterialInventory,
    loadout: &PlayerLoadout,
    recipe_book: &ForgeRecipeBook,
    hub_player: &Query<&Transform, With<HubPlayer>>,
    forge_query: &Query<&Transform, With<ForgeStation>>,
) -> String {
    let mut lines = vec![
        "=== Hub ===".to_string(),
        format!("Weapon: {}", loadout.weapon.display_name()),
        format!("Armor defense: {:.0}", loadout.total_defense()),
        String::new(),
        "Materials:".to_string(),
    ];

    let mut materials: Vec<_> = inventory.iter().collect();
    materials.sort_by_key(|(material, _)| material.display_name());
    if materials.is_empty() {
        lines.push("  (none)".to_string());
    } else {
        for (material, count) in materials {
            lines.push(format!("  {} x{}", material.display_name(), count));
        }
    }

    if is_near_forge(hub_player, forge_query) {
        lines.push(String::new());
        lines.push("=== Forge ===".to_string());
        if let Some(recipe) = recipe_book.selected_recipe() {
            lines.push(format!("Recipe: {}", recipe.name));
            for (material, amount) in recipe.materials {
                let owned = inventory.count(*material);
                lines.push(format!(
                    "  {} {}/{}",
                    material.display_name(),
                    owned,
                    amount
                ));
            }
        }
        let bonuses = SlimeSet::active_bonuses(&loadout.armor);
        if !bonuses.is_empty() {
            lines.push(String::new());
            lines.push("Set bonuses:".to_string());
            for bonus in bonuses {
                lines.push(format!("  {}", bonus.description));
            }
        }
    }

    lines.join("\n")
}

fn dungeon_hud_text(
    inventory: &MaterialInventory,
    loadout: &PlayerLoadout,
    player_health: &Query<&Health, With<DungeonPlayer>>,
) -> String {
    let health_line = player_health
        .get_single()
        .map(|health| format!("HP: {:.0}/{:.0}", health.current, health.max))
        .unwrap_or_else(|_| "HP: --".to_string());

    let mut lines = vec![
        "=== Dungeon ===".to_string(),
        health_line,
        format!(
            "Weapon: {} (dmg {:.0})",
            loadout.weapon.display_name(),
            loadout.weapon_damage()
        ),
        format!("Defense: {:.0}", loadout.total_defense()),
        String::new(),
        "Materials:".to_string(),
    ];

    let mut materials: Vec<_> = inventory.iter().collect();
    materials.sort_by_key(|(material, _)| material.display_name());
    if materials.is_empty() {
        lines.push("  (none)".to_string());
    } else {
        for (material, count) in materials {
            lines.push(format!("  {} x{}", material.display_name(), count));
        }
    }

    lines.join("\n")
}

fn is_near_forge(
    hub_player: &Query<&Transform, With<HubPlayer>>,
    forge_query: &Query<&Transform, With<ForgeStation>>,
) -> bool {
    let Ok(player_transform) = hub_player.get_single() else {
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