use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::combat::WeaponKind;
use crate::core::data_load::load_ron_from_assets_or_embedded;
use crate::items::{Inventory, MaterialId};
use crate::player::{ArmorKind, Loadout};

const RECIPES_PATH: &str = "assets/data/recipes.ron";
const EMBEDDED_RECIPES: &str = include_str!("../../assets/data/recipes.ron");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecipeOutput {
    Weapon(WeaponKind),
    Armor(ArmorKind),
    /// Tool or stackable material added to inventory.
    Item(MaterialId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub costs: Vec<(MaterialId, u32)>,
    pub output: RecipeOutput,
    #[serde(default)]
    pub requires_weapon: Option<WeaponKind>,
}

/// Runtime recipe list loaded from RON (disk or embedded).
#[derive(Resource, Clone, Debug)]
pub struct RecipeBook {
    pub recipes: Vec<Recipe>,
}

impl Default for RecipeBook {
    fn default() -> Self {
        Self::load()
    }
}

impl RecipeBook {
    pub fn load() -> Self {
        match load_ron_from_assets_or_embedded::<Vec<Recipe>>(
            RECIPES_PATH,
            EMBEDDED_RECIPES,
            "recipes",
        ) {
            Some(recipes) if !recipes.is_empty() => Self { recipes },
            Some(_) => {
                bevy::log::error!("Recipe list is empty; forge will have no recipes");
                Self {
                    recipes: Vec::new(),
                }
            }
            None => Self {
                recipes: Vec::new(),
            },
        }
    }

    pub fn get(&self, index: usize) -> Option<&Recipe> {
        self.recipes.get(index)
    }

    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }
}

pub fn recipe_set_bonus_hint(recipe: &Recipe) -> Option<&'static str> {
    match recipe.output {
        RecipeOutput::Armor(_) => Some(
            "Set: 2pc +10% carve & −10% special CD · 4pc 35% KB resist & +10% attack",
        ),
        RecipeOutput::Item(MaterialId::Pickaxe) => {
            Some("Homestead tool — mine iron ore at the east rocks")
        }
        RecipeOutput::Item(MaterialId::FishingRod) => {
            Some("Homestead tool — cast at the southeast dock")
        }
        RecipeOutput::Item(_) => Some("Homestead tool — equip from inventory hotbar"),
        _ => None,
    }
}

pub fn can_craft_recipe(inventory: &Inventory, loadout: &Loadout, recipe: &Recipe) -> bool {
    if !inventory.has_materials(&recipe.costs) {
        return false;
    }

    if let Some(required_weapon) = recipe.requires_weapon {
        if loadout.weapon != required_weapon {
            return false;
        }
    }

    true
}

pub fn try_craft_recipe(inventory: &mut Inventory, loadout: &mut Loadout, recipe: &Recipe) -> bool {
    if !can_craft_recipe(inventory, loadout, recipe) {
        return false;
    }

    for (material, amount) in &recipe.costs {
        if !inventory.try_remove(*material, *amount) {
            return false;
        }
    }

    match recipe.output {
        RecipeOutput::Weapon(weapon) => loadout.weapon = weapon,
        RecipeOutput::Armor(armor) => loadout.armor.set(armor),
        RecipeOutput::Item(material) => {
            let left = inventory.try_add(material, 1);
            if left > 0 {
                // Rollback costs if inventory full.
                for (m, amount) in &recipe.costs {
                    inventory.try_add(*m, *amount);
                }
                return false;
            }
        }
    }

    true
}

/// Metal-tier recipes that must be gated on mined ore (not dungeon scrap alone).
pub fn recipe_requires_mined_ore(recipe: &Recipe) -> bool {
    recipe
        .costs
        .iter()
        .any(|(m, _)| *m == MaterialId::IronOre)
}

pub fn material_name(material: MaterialId) -> &'static str {
    material.display_name()
}

pub fn recipe_costs_text(inventory: &Inventory, recipe: &Recipe) -> String {
    recipe
        .costs
        .iter()
        .map(|(material, amount)| {
            format!(
                "{} {}/{}",
                material_name(*material),
                inventory.count(*material),
                amount
            )
        })
        .collect::<Vec<_>>()
        .join("  ·  ")
}

pub fn recipe_requirement_text(loadout: &Loadout, recipe: &Recipe) -> Option<String> {
    recipe.requires_weapon.map(|weapon| {
        let label = weapon_label(weapon);
        if loadout.weapon == weapon {
            format!("Requires equipped {label} (ready)")
        } else {
            format!("Requires equipped {label}")
        }
    })
}

fn weapon_label(weapon: WeaponKind) -> &'static str {
    match weapon {
        WeaponKind::RustySword => "Rusty Sword",
        WeaponKind::RustySpear => "Rusty Spear",
        WeaponKind::IronSword => "Iron Sword",
        WeaponKind::SlimeBlade => "Slime Blade",
    }
}

pub fn forge_status(inventory: &Inventory, loadout: &Loadout, recipe: &Recipe) -> String {
    if can_craft_recipe(inventory, loadout, recipe) {
        format!("{} is ready to craft.", recipe.name)
    } else if let Some(requirement) = recipe_requirement_text(loadout, recipe) {
        if loadout.weapon != recipe.requires_weapon.unwrap_or(loadout.weapon) {
            requirement
        } else {
            "Gather the required materials.".to_string()
        }
    } else {
        "Gather the required materials.".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> RecipeBook {
        RecipeBook::load()
    }

    fn recipe_named(book: &RecipeBook, name: &str) -> Recipe {
        book.recipes
            .iter()
            .find(|r| r.name == name)
            .cloned()
            .unwrap_or_else(|| panic!("missing recipe {name}"))
    }

    #[test]
    fn loads_recipes_from_embedded_ron() {
        let book = book();
        assert!(
            book.len() >= 10,
            "expected weapon + armor + tool recipes, got {}",
            book.len()
        );
    }

    #[test]
    fn iron_sword_requires_mined_ore_not_scrap_alone() {
        let iron = recipe_named(&book(), "Iron Sword");
        assert!(recipe_requires_mined_ore(&iron));
        assert!(
            iron.costs.iter().any(|(m, _)| *m == MaterialId::IronOre),
            "Iron Sword must cost IronOre"
        );
        assert!(
            !iron.costs.iter().any(|(m, _)| *m == MaterialId::IronScrap),
            "Iron Sword should not depend on dungeon IronScrap"
        );

        let mut inventory = Inventory::default();
        let mut loadout = Loadout::default();
        inventory.try_add(MaterialId::SlimeGel, 5);
        inventory.try_add(MaterialId::IronScrap, 3);
        assert!(
            !can_craft_recipe(&inventory, &loadout, &iron),
            "scrap alone must not craft iron sword"
        );

        inventory.try_add(MaterialId::IronOre, 3);
        assert!(try_craft_recipe(&mut inventory, &mut loadout, &iron));
        assert_eq!(loadout.weapon, WeaponKind::IronSword);
        assert_eq!(inventory.count(MaterialId::IronOre), 0);
    }

    #[test]
    fn crafts_pickaxe_into_inventory() {
        let pick = recipe_named(&book(), "Pickaxe");
        let mut inventory = Inventory::default();
        let mut loadout = Loadout::default();
        inventory.try_add(MaterialId::IronScrap, 2);
        inventory.try_add(MaterialId::SlimeGel, 1);
        assert!(try_craft_recipe(&mut inventory, &mut loadout, &pick));
        assert_eq!(inventory.count(MaterialId::Pickaxe), 1);
    }

    #[test]
    fn crafts_fishing_rod_into_inventory() {
        let rod = recipe_named(&book(), "Fishing Rod");
        let mut inventory = Inventory::default();
        let mut loadout = Loadout::default();
        inventory.try_add(MaterialId::BoneShard, 2);
        inventory.try_add(MaterialId::SlimeGel, 1);
        assert!(try_craft_recipe(&mut inventory, &mut loadout, &rod));
        assert_eq!(inventory.count(MaterialId::FishingRod), 1);
    }

    #[test]
    fn slime_blade_requires_iron_sword_and_royal_core() {
        let blade = recipe_named(&book(), "Slime Blade");
        let mut inventory = Inventory::default();
        let mut loadout = Loadout::default();
        inventory.try_add(MaterialId::SlimeCore, 2);
        inventory.try_add(MaterialId::RoyalSlimeCore, 1);

        assert!(!try_craft_recipe(
            &mut inventory,
            &mut loadout,
            &blade
        ));

        loadout.weapon = WeaponKind::IronSword;
        assert!(try_craft_recipe(
            &mut inventory,
            &mut loadout,
            &blade
        ));
        assert_eq!(loadout.weapon, WeaponKind::SlimeBlade);
        assert_eq!(inventory.count(MaterialId::RoyalSlimeCore), 0);
    }

    #[test]
    fn armor_piece_equips_into_slot() {
        let helm = recipe_named(&book(), "Slime Helm");
        let mut inventory = Inventory::default();
        let mut loadout = Loadout::default();
        inventory.try_add(MaterialId::SlimeGel, 4);

        assert!(try_craft_recipe(&mut inventory, &mut loadout, &helm));
        assert_eq!(loadout.armor.head, Some(ArmorKind::SlimeHelm));
    }

    #[test]
    fn invalid_ron_returns_none_from_loader() {
        use crate::core::data_load::load_ron_from_assets_or_embedded;
        let result: Option<Vec<Recipe>> =
            load_ron_from_assets_or_embedded("assets/data/__missing_recipes.ron", "not valid", "t");
        assert!(result.is_none());
    }
}
