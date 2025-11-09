pub mod seeds;
pub mod weapons;
use super::crops::{Corn, Crop, CropType};

use bevy::prelude::Component;

#[derive(Component)]
pub struct DisplayInfo {
    pub name: &'static str,
    pub image_path: usize,
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
    fn inventory_image(&self) -> usize;
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
    Sword(weapons::sword::Sword),
}

impl ItemType {
    pub fn as_item(&self) -> &dyn Item {
        match self {
            ItemType::CornSeed(corn) => corn,
            ItemType::Sword(sword) => sword,
        }
    }
    pub fn category(&self) -> ItemCategory {
        self.as_item().category()
    }
    pub fn plant(&self) -> Option<Crop> {
        match self {
            ItemType::CornSeed(_) => Some(Crop {
                crop_type: CropType::Corn(Corn::default()),
                timer: 0.0,
            }),
            ItemType::Sword(_) => None,
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub count: usize,
    pub max_count: usize,
}
