pub trait Crop {
    fn grow_time(&self) -> u32;
    fn growth_stage_image(&self) -> &str;
}

pub enum GrowthStage {
    Seed,
    Sprout,
    Mature,
    Fruiting,
}

pub mod corn;
