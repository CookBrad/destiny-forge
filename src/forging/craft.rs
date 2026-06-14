use crate::items::MaterialInventory;
use crate::player::PlayerLoadout;

use super::recipe::{ForgeRecipeBook, RecipeOutput};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CraftResult {
    Success,
    MissingMaterials,
    MissingWeapon,
    AlreadyOwned,
}

pub fn try_craft_selected_recipe(
    recipe_book: &ForgeRecipeBook,
    inventory: &mut MaterialInventory,
    loadout: &mut PlayerLoadout,
) -> CraftResult {
    let Some(recipe) = recipe_book.selected_recipe() else {
        return CraftResult::MissingMaterials;
    };

    if recipe_already_owned(recipe.output, loadout) {
        return CraftResult::AlreadyOwned;
    }

    if let Some(required_weapon) = recipe.required_weapon() {
        if loadout.weapon != required_weapon {
            return CraftResult::MissingWeapon;
        }
    }

    if !inventory.can_remove(recipe.materials) {
        return CraftResult::MissingMaterials;
    }

    inventory.remove(recipe.materials);
    apply_recipe_output(recipe.output, loadout);
    CraftResult::Success
}

fn recipe_already_owned(output: RecipeOutput, loadout: &PlayerLoadout) -> bool {
    match output {
        RecipeOutput::Weapon(weapon) => loadout.owns_weapon(weapon),
        RecipeOutput::Armor(armor) => loadout.owns_armor(armor),
    }
}

fn apply_recipe_output(output: RecipeOutput, loadout: &mut PlayerLoadout) {
    match output {
        RecipeOutput::Weapon(weapon) => loadout.equip_weapon(weapon),
        RecipeOutput::Armor(armor) => loadout.equip_armor(armor),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forging::recipe::ForgeRecipe;
    use crate::items::MaterialId;

    #[test]
    fn iron_sword_requires_materials() {
        let recipe_book = ForgeRecipeBook {
            recipes: vec![ForgeRecipe {
                name: "Iron Sword",
                materials: &[(MaterialId::SlimeGel, 5), (MaterialId::IronScrap, 3)],
                output: RecipeOutput::Weapon(WeaponId::IronSword),
            }],
            selected_index: 0,
        };
        let mut inventory = MaterialInventory::default();
        let mut loadout = PlayerLoadout::default();

        assert_eq!(
            try_craft_selected_recipe(&recipe_book, &mut inventory, &mut loadout),
            CraftResult::MissingMaterials
        );

        inventory.add(MaterialId::SlimeGel, 5);
        inventory.add(MaterialId::IronScrap, 3);

        assert_eq!(
            try_craft_selected_recipe(&recipe_book, &mut inventory, &mut loadout),
            CraftResult::Success
        );
        assert_eq!(loadout.weapon, WeaponId::IronSword);
    }
}