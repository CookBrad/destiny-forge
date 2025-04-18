use super::GrowthStage;

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

impl Corn {
    fn grow_time(&self) -> u32 {
        100
    }

    pub fn growth_stage_image(&self) -> usize {
        match self.stage {
            GrowthStage::Seed => 121,
            GrowthStage::Sprout => 122,
            GrowthStage::Immature => 123,
            GrowthStage::Mature => 125,
            GrowthStage::Fruiting => 126,
        }
    }
}
