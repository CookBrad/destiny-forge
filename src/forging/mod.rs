mod plugin;
mod recipes;

pub use plugin::ForgingPlugin;
pub use recipes::{
    can_craft_recipe, try_craft_iron_sword, try_craft_recipe, Recipe, IRON_SWORD_RECIPE,
};