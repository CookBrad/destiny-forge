use crate::items::{Item, ItemCategory};
use bevy::prelude::Component;

#[derive(Component, Clone, Debug)]
pub struct CornSeed;

impl Item for CornSeed {
    fn name(&self) -> &'static str {
        "Corn"
    }

    fn inventory_image(&self) -> usize {
        121
    }

    fn stack_size(&self) -> usize {
        64
    }

    fn category(&self) -> ItemCategory {
        crate::items::ItemCategory::Crop
    }
}
