pub mod crops;
use bevy::prelude::Component;

#[derive(Component)]
pub struct DisplayInfo {
    pub name: &'static str,
    pub image_path: &'static str,
}
pub trait Item {
    fn name(&self) -> &'static str;
    fn inventory_image(&self) -> &'static str;
    fn stack_size(&self) -> usize;
    fn id(&self) -> u32;
    fn display_info(&self) -> DisplayInfo {
        DisplayInfo {
            name: self.name(),
            image_path: self.inventory_image(),
        }
    }
}

#[derive(Clone)]
pub enum ItemType {
    Corn(crops::corn::Corn),
}

impl ItemType {
    pub fn as_item(&self) -> &dyn Item {
        match self {
            ItemType::Corn(corn) => corn,
            // Add other variants here as you expand your item system
        }
    }
}

#[derive(Component, Clone)]
pub struct ItemStack {
    pub item: ItemType,
    pub count: usize,
    pub max_count: usize,
}
