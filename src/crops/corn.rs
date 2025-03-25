use super::{Crop, GrowthStage};

pub struct Corn {
    pub stage: GrowthStage,
}

impl Default for Corn {
    fn default() -> Self {
        Self {
            stage: GrowthStage::Seed, // Default speed set to 100.0
        }
    }
}

impl Crop for Corn {
    fn grow_time(&self) -> u32 {
        100
    }

    fn growth_stage_image(&self) -> &str {
        match self.stage {
            GrowthStage::Seed => "corn_seed.png",
            GrowthStage::Sprout => "corn_sprout.png",
            GrowthStage::Mature => "corn_mature.png",
            GrowthStage::Fruiting => "corn_fruiting.png",
        }
    }
}
