//! Pure carve loot resolution — unit-testable without Bevy systems.
//! Tables load from `assets/data/carve_loot.ron`.

use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::core::data_load::load_ron_from_assets_or_embedded;
use crate::items::MaterialId;

use super::enemy::EnemyKind;

const LOOT_PATH: &str = "assets/data/carve_loot.ron";
const EMBEDDED_LOOT: &str = include_str!("../../assets/data/carve_loot.ron");

/// What is being carved: trash pack or boss.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CarveTarget {
    Pack(EnemyKind),
    KingSlime,
}

impl CarveTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pack(EnemyKind::Slime) => "slime",
            Self::Pack(EnemyKind::Bat) => "bat",
            Self::Pack(EnemyKind::Goblin) => "goblin",
            Self::Pack(EnemyKind::Skeleton) => "skeleton",
            Self::Pack(EnemyKind::Zombie) => "zombie",
            Self::KingSlime => "king slime",
        }
    }
}

/// Guaranteed drops always granted; bonus entries roll independently by chance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LootTable {
    pub target: CarveTarget,
    pub guaranteed: Vec<(MaterialId, u32)>,
    /// (material, amount, chance in 0.0..=1.0)
    pub bonus: Vec<(MaterialId, u32, f32)>,
}

/// Runtime loot tables loaded from RON.
#[derive(Resource, Clone, Debug)]
pub struct CarveLootBook {
    tables: Vec<LootTable>,
}

impl Default for CarveLootBook {
    fn default() -> Self {
        Self::load()
    }
}

impl CarveLootBook {
    pub fn load() -> Self {
        match load_ron_from_assets_or_embedded::<Vec<LootTable>>(
            LOOT_PATH,
            EMBEDDED_LOOT,
            "carve loot",
        ) {
            Some(tables) if !tables.is_empty() => Self { tables },
            Some(_) => {
                bevy::log::error!("Carve loot tables empty");
                Self {
                    tables: Vec::new(),
                }
            }
            None => Self {
                tables: Vec::new(),
            },
        }
    }

    pub fn table_for(&self, target: CarveTarget) -> Option<&LootTable> {
        self.tables.iter().find(|table| table.target == target)
    }

    /// Roll carve yields for a target. Always includes guaranteed parts when a table exists.
    pub fn roll(&self, target: CarveTarget, rng: &mut impl Rng) -> Vec<(MaterialId, u32)> {
        let Some(table) = self.table_for(target) else {
            bevy::log::warn!("No carve loot table for {target:?}");
            return Vec::new();
        };
        roll_table(table, rng)
    }

    /// Deterministic loot for tests / debugging (all bonus rolls succeed).
    pub fn max_loot(&self, target: CarveTarget) -> Vec<(MaterialId, u32)> {
        let Some(table) = self.table_for(target) else {
            return Vec::new();
        };
        let mut drops = Vec::new();
        for &(material, amount) in &table.guaranteed {
            push_or_stack(&mut drops, material, amount);
        }
        for &(material, amount, _) in &table.bonus {
            push_or_stack(&mut drops, material, amount);
        }
        drops
    }
}

/// Convenience for call sites that hold a book resource.
pub fn roll_carve_loot(
    book: &CarveLootBook,
    target: CarveTarget,
    rng: &mut impl Rng,
) -> Vec<(MaterialId, u32)> {
    book.roll(target, rng)
}

fn roll_table(table: &LootTable, rng: &mut impl Rng) -> Vec<(MaterialId, u32)> {
    let mut drops = Vec::with_capacity(table.guaranteed.len() + table.bonus.len());

    for &(material, amount) in &table.guaranteed {
        push_or_stack(&mut drops, material, amount);
    }

    for &(material, amount, chance) in &table.bonus {
        if rng.gen::<f32>() < chance {
            push_or_stack(&mut drops, material, amount);
        }
    }

    drops
}

fn push_or_stack(drops: &mut Vec<(MaterialId, u32)>, material: MaterialId, amount: u32) {
    if let Some((_, total)) = drops.iter_mut().find(|(id, _)| *id == material) {
        *total += amount;
    } else {
        drops.push((material, amount));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn book() -> CarveLootBook {
        CarveLootBook::load()
    }

    #[test]
    fn each_pack_species_has_distinct_guaranteed_part() {
        let book = book();
        let slime = book.table_for(CarveTarget::Pack(EnemyKind::Slime)).unwrap();
        let bat = book.table_for(CarveTarget::Pack(EnemyKind::Bat)).unwrap();
        let goblin = book.table_for(CarveTarget::Pack(EnemyKind::Goblin)).unwrap();
        let skeleton = book
            .table_for(CarveTarget::Pack(EnemyKind::Skeleton))
            .unwrap();
        let zombie = book.table_for(CarveTarget::Pack(EnemyKind::Zombie)).unwrap();

        assert!(slime
            .guaranteed
            .iter()
            .any(|(m, _)| *m == MaterialId::SlimeGel));
        assert!(bat
            .guaranteed
            .iter()
            .any(|(m, _)| *m == MaterialId::LeatherWing));
        assert!(goblin
            .guaranteed
            .iter()
            .any(|(m, _)| *m == MaterialId::Fang));
        assert!(skeleton
            .guaranteed
            .iter()
            .any(|(m, _)| *m == MaterialId::BoneShard));
        assert!(zombie
            .guaranteed
            .iter()
            .any(|(m, _)| *m == MaterialId::RotFlesh));
    }

    #[test]
    fn king_slime_always_drops_royal_core() {
        let book = book();
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..20 {
            let drops = book.roll(CarveTarget::KingSlime, &mut rng);
            let royal: u32 = drops
                .iter()
                .filter(|(m, _)| *m == MaterialId::RoyalSlimeCore)
                .map(|(_, n)| *n)
                .sum();
            assert!(
                royal >= 1,
                "boss must always yield at least one Royal Slime Core"
            );
        }
    }

    #[test]
    fn roll_never_empty_for_packs() {
        let book = book();
        let mut rng = StdRng::seed_from_u64(42);
        for kind in [
            EnemyKind::Slime,
            EnemyKind::Bat,
            EnemyKind::Goblin,
            EnemyKind::Skeleton,
            EnemyKind::Zombie,
        ] {
            let drops = book.roll(CarveTarget::Pack(kind), &mut rng);
            assert!(!drops.is_empty(), "{kind:?} should always yield something");
        }
    }

    #[test]
    fn max_loot_stacks_same_materials() {
        let book = book();
        let max = book.max_loot(CarveTarget::Pack(EnemyKind::Slime));
        let gel_entries = max
            .iter()
            .filter(|(m, _)| *m == MaterialId::SlimeGel)
            .count();
        assert_eq!(gel_entries, 1, "same materials should stack into one entry");
    }
}
