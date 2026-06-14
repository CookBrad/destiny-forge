use bevy::prelude::*;

use crate::items::{ArmorId, MaterialId, WeaponId};
use crate::progression::WeaponUpgradeTree;

#[derive(Clone, Copy, Debug)]
pub enum RecipeOutput {
    Weapon(WeaponId),
    Armor(ArmorId),
}

#[derive(Clone, Copy, Debug)]
pub struct ForgeRecipe {
    pub name: &'static str,
    pub materials: &'static [(MaterialId, u32)],
    pub output: RecipeOutput,
}

impl ForgeRecipe {
    pub fn required_weapon(&self) -> Option<WeaponId> {
        match self.output {
            RecipeOutput::Weapon(weapon) => WeaponUpgradeTree::consumes_weapon(weapon),
            RecipeOutput::Armor(_) => None,
        }
    }
}

#[derive(Resource)]
pub struct ForgeRecipeBook {
    pub recipes: Vec<ForgeRecipe>,
    pub selected_index: usize,
}

impl Default for ForgeRecipeBook {
    fn default() -> Self {
        Self {
            recipes: vec![
                ForgeRecipe {
                    name: "Iron Sword",
                    materials: &[(MaterialId::SlimeGel, 5), (MaterialId::IronScrap, 3)],
                    output: RecipeOutput::Weapon(WeaponId::IronSword),
                },
                ForgeRecipe {
                    name: "Slime Blade",
                    materials: &[(MaterialId::SlimeCore, 2)],
                    output: RecipeOutput::Weapon(WeaponId::SlimeBlade),
                },
                ForgeRecipe {
                    name: "Rusty Spear",
                    materials: &[(MaterialId::SlimeGel, 3), (MaterialId::Fang, 2)],
                    output: RecipeOutput::Weapon(WeaponId::RustySpear),
                },
                ForgeRecipe {
                    name: "Slime Helm",
                    materials: &[(MaterialId::SlimeGel, 4)],
                    output: RecipeOutput::Armor(ArmorId::SlimeHelm),
                },
                ForgeRecipe {
                    name: "Slime Mail",
                    materials: &[(MaterialId::SlimeGel, 6), (MaterialId::SlimeCore, 1)],
                    output: RecipeOutput::Armor(ArmorId::SlimeMail),
                },
                ForgeRecipe {
                    name: "Slime Gauntlets",
                    materials: &[(MaterialId::SlimeGel, 3)],
                    output: RecipeOutput::Armor(ArmorId::SlimeGauntlets),
                },
                ForgeRecipe {
                    name: "Slime Greaves",
                    materials: &[(MaterialId::SlimeGel, 4)],
                    output: RecipeOutput::Armor(ArmorId::SlimeGreaves),
                },
            ],
            selected_index: 0,
        }
    }
}

impl ForgeRecipeBook {
    pub fn selected_recipe(&self) -> Option<&ForgeRecipe> {
        self.recipes.get(self.selected_index)
    }

    pub fn select_next(&mut self) {
        if self.recipes.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.recipes.len();
    }

    pub fn select_previous(&mut self) {
        if self.recipes.is_empty() {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.recipes.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }
}