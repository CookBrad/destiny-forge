pub mod seeds;
use bevy::prelude::Component;

#[derive(Component)]
pub struct DisplayInfo {
    pub name: &'static str,
    pub image_path: &'static str,
}

#[derive(Debug)]
pub enum ItemCategory {
    Crop,
    Tool,
    Food,
    Weapon,
    Armor,
}
pub trait Item {
    fn name(&self) -> &'static str;
    fn inventory_image(&self) -> &'static str;
    fn stack_size(&self) -> usize;
    fn category(&self) -> ItemCategory;
    fn display_info(&self) -> DisplayInfo {
        DisplayInfo {
            name: self.name(),
            image_path: self.inventory_image(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ItemType {
    CornSeed(seeds::corn::CornSeed),
}

impl ItemType {
    pub fn as_item(&self) -> &dyn Item {
        match self {
            ItemType::CornSeed(corn) => corn,
            // Add other variants here as you expand your item system
        }
    }
    pub fn category(&self) -> ItemCategory {
        self.as_item().category()
    }
}

#[derive(Component, Clone, Debug)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub count: usize,
    pub max_count: usize,
}
