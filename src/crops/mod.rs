pub mod corn;
use corn::Corn;
pub struct Crop {
    pub crop_type: CropType,
}

pub enum GrowthStage {
    Seed,
    Sprout,
    Mature,
    Fruiting,
}

pub enum CropType {
    Corn(Corn),
    Wheat,
    Carrot,
    Tomato,
}
