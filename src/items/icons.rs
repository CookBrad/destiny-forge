//! Inventory / hotbar icons for every material.

use bevy::prelude::*;

use super::MaterialId;

const ITEM_ICON_ROOT: &str = "ui/items";

#[derive(Resource, Clone)]
pub struct ItemIconAssets {
    pub slime_gel: Handle<Image>,
    pub slime_core: Handle<Image>,
    pub leather_wing: Handle<Image>,
    pub fang: Handle<Image>,
    pub iron_scrap: Handle<Image>,
    pub bone_shard: Handle<Image>,
    pub rot_flesh: Handle<Image>,
    pub royal_slime_core: Handle<Image>,
    pub turnip_seed: Handle<Image>,
    pub potato_seed: Handle<Image>,
    pub turnip: Handle<Image>,
    pub potato: Handle<Image>,
    pub hoe: Handle<Image>,
    pub watering_can: Handle<Image>,
}

impl FromWorld for ItemIconAssets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        Self::load(server)
    }
}

impl ItemIconAssets {
    pub fn load(asset_server: &AssetServer) -> Self {
        let load = |name: &str| asset_server.load(format!("{ITEM_ICON_ROOT}/{name}"));
        Self {
            slime_gel: load("slime_gel.png"),
            slime_core: load("slime_core.png"),
            leather_wing: load("leather_wing.png"),
            fang: load("fang.png"),
            iron_scrap: load("iron_scrap.png"),
            bone_shard: load("bone_shard.png"),
            rot_flesh: load("rot_flesh.png"),
            royal_slime_core: load("royal_slime_core.png"),
            turnip_seed: load("turnip_seed.png"),
            potato_seed: load("potato_seed.png"),
            turnip: load("turnip.png"),
            potato: load("potato.png"),
            hoe: load("hoe.png"),
            watering_can: load("watering_can.png"),
        }
    }

    pub fn handle_for(&self, material: MaterialId) -> Handle<Image> {
        match material {
            MaterialId::SlimeGel => self.slime_gel.clone(),
            MaterialId::SlimeCore => self.slime_core.clone(),
            MaterialId::LeatherWing => self.leather_wing.clone(),
            MaterialId::Fang => self.fang.clone(),
            MaterialId::IronScrap => self.iron_scrap.clone(),
            MaterialId::BoneShard => self.bone_shard.clone(),
            MaterialId::RotFlesh => self.rot_flesh.clone(),
            MaterialId::RoyalSlimeCore => self.royal_slime_core.clone(),
            MaterialId::TurnipSeed => self.turnip_seed.clone(),
            MaterialId::PotatoSeed => self.potato_seed.clone(),
            MaterialId::Turnip => self.turnip.clone(),
            MaterialId::Potato => self.potato.clone(),
            MaterialId::Hoe => self.hoe.clone(),
            MaterialId::WateringCan => self.watering_can.clone(),
        }
    }
}
