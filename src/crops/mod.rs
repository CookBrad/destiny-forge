use bevy::prelude::Component;

pub mod corn;
pub use corn::Corn;
#[derive(Component)]
pub struct Crop {
    pub crop_type: CropType,
    pub timer: f32,
}

pub enum GrowthStage {
    Seed,
    Sprout,
    Mature,
    Fruiting,
}

pub enum CropType {
    Corn(Corn),
    // Wheat,
    // Carrot,
    // Tomato,
}

impl Crop {
    pub fn growth_stage_image(&self) -> &str {
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
