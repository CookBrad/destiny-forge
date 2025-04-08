use crate::items::{Item, ItemCategory};
use bevy::prelude::Component;

#[derive(Component, Clone, Debug)]
pub struct Corn;

impl Item for Corn {
    fn name(&self) -> &'static str {
        "Corn"
    }

    fn inventory_image(&self) -> &'static str {
        "crop.png"
    }

    fn stack_size(&self) -> usize {
        64
    }

    fn category(&self) -> ItemCategory {
        crate::items::ItemCategory::Crop
    }
}
