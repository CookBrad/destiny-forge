use crate::items::{Item, ItemCategory};
use bevy::prelude::Component;

#[derive(Component, Clone, Debug)]
pub struct Sword;

impl Item for Sword {
    fn name(&self) -> &'static str {
        "Sword"
    }

    fn inventory_image(&self) -> usize {
        0 // You can change this to a sword sprite index
    }

    fn stack_size(&self) -> usize {
        1 // Weapons don't stack
    }

    fn category(&self) -> ItemCategory {
        ItemCategory::Weapon
    }
}
