use crate::items::seeds::corn::CornSeed;
use crate::items::{ItemStack, ItemType};
use bevy::prelude::Component;
pub mod corn;
pub use corn::Corn;
#[derive(Component)]
pub struct Crop {
    pub crop_type: CropType,
    pub timer: f32,
}

#[derive(PartialEq)]
pub enum GrowthStage {
    Seed,
    Sprout,
    Immature,
    Mature,
    Fruiting,
}

pub enum CropType {
    Corn(Corn),
    // Wheat,
    // Carrot,
    // Tomato,
}
impl CropType {
    pub fn harvested(&self) -> ItemStack {
        match self {
            CropType::Corn(_) => ItemStack {
                item_type: ItemType::CornSeed(CornSeed),
                count: 33,
                max_count: 64,
            },
        }
    }
}

impl Crop {
    pub fn growth_stage_image(&self) -> usize {
        match &self.crop_type {
            CropType::Corn(corn) => corn.growth_stage_image(),
        }
    }
    pub fn get_stage(&self) -> &GrowthStage {
        match &self.crop_type {
            CropType::Corn(corn) => &corn.stage,
        }
    }
    pub fn set_stage(&mut self, growth_stage: GrowthStage) {
        match &mut self.crop_type {
            CropType::Corn(corn) => corn.stage = growth_stage,
        }
    }
}
