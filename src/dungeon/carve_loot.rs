//! Pure carve loot resolution — unit-testable without Bevy systems.

use rand::Rng;

use crate::items::MaterialId;

use super::enemy::EnemyKind;

/// What is being carved: trash pack or boss.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug)]
struct LootTable {
    guaranteed: &'static [(MaterialId, u32)],
    /// (material, amount, chance in 0.0..=1.0)
    bonus: &'static [(MaterialId, u32, f32)],
}

const SLIME: LootTable = LootTable {
    guaranteed: &[(MaterialId::SlimeGel, 2)],
    bonus: &[
        (MaterialId::SlimeGel, 2, 0.55),
        (MaterialId::SlimeCore, 1, 0.35),
        (MaterialId::IronScrap, 1, 0.25),
    ],
};

const BAT: LootTable = LootTable {
    guaranteed: &[(MaterialId::LeatherWing, 1)],
    bonus: &[
        (MaterialId::Fang, 1, 0.65),
        (MaterialId::LeatherWing, 1, 0.3),
    ],
};

const GOBLIN: LootTable = LootTable {
    guaranteed: &[(MaterialId::Fang, 1)],
    bonus: &[
        (MaterialId::IronScrap, 1, 0.5),
        (MaterialId::Fang, 1, 0.25),
    ],
};

const SKELETON: LootTable = LootTable {
    guaranteed: &[(MaterialId::BoneShard, 2)],
    bonus: &[
        (MaterialId::BoneShard, 1, 0.4),
        (MaterialId::IronScrap, 1, 0.45),
    ],
};

const ZOMBIE: LootTable = LootTable {
    guaranteed: &[(MaterialId::RotFlesh, 1)],
    bonus: &[
        (MaterialId::RotFlesh, 1, 0.4),
        (MaterialId::IronScrap, 1, 0.35),
        (MaterialId::SlimeGel, 1, 0.15),
    ],
};

const KING_SLIME: LootTable = LootTable {
    guaranteed: &[
        (MaterialId::SlimeGel, 5),
        (MaterialId::SlimeCore, 2),
        (MaterialId::RoyalSlimeCore, 1),
    ],
    bonus: &[
        (MaterialId::SlimeGel, 3, 0.5),
        (MaterialId::RoyalSlimeCore, 1, 0.15),
        (MaterialId::IronScrap, 2, 0.4),
    ],
};

fn table_for(target: CarveTarget) -> LootTable {
    match target {
        CarveTarget::Pack(EnemyKind::Slime) => SLIME,
        CarveTarget::Pack(EnemyKind::Bat) => BAT,
        CarveTarget::Pack(EnemyKind::Goblin) => GOBLIN,
        CarveTarget::Pack(EnemyKind::Skeleton) => SKELETON,
        CarveTarget::Pack(EnemyKind::Zombie) => ZOMBIE,
        CarveTarget::KingSlime => KING_SLIME,
    }
}

/// Roll carve yields for a target. Always includes guaranteed parts.
pub fn roll_carve_loot(target: CarveTarget, rng: &mut impl Rng) -> Vec<(MaterialId, u32)> {
    let table = table_for(target);
    let mut drops = Vec::with_capacity(table.guaranteed.len() + table.bonus.len());

    for &(material, amount) in table.guaranteed {
        push_or_stack(&mut drops, material, amount);
    }

    for &(material, amount, chance) in table.bonus {
        if rng.gen::<f32>() < chance {
            push_or_stack(&mut drops, material, amount);
        }
    }

    drops
}

/// Deterministic loot for tests / debugging (all bonus rolls succeed).
pub fn max_carve_loot(target: CarveTarget) -> Vec<(MaterialId, u32)> {
    let table = table_for(target);
    let mut drops = Vec::new();
    for &(material, amount) in table.guaranteed {
        push_or_stack(&mut drops, material, amount);
    }
    for &(material, amount, _) in table.bonus {
        push_or_stack(&mut drops, material, amount);
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

    #[test]
    fn each_pack_species_has_distinct_guaranteed_part() {
        let slime = table_for(CarveTarget::Pack(EnemyKind::Slime));
        let bat = table_for(CarveTarget::Pack(EnemyKind::Bat));
        let goblin = table_for(CarveTarget::Pack(EnemyKind::Goblin));
        let skeleton = table_for(CarveTarget::Pack(EnemyKind::Skeleton));
        let zombie = table_for(CarveTarget::Pack(EnemyKind::Zombie));

        assert!(slime.guaranteed.iter().any(|(m, _)| *m == MaterialId::SlimeGel));
        assert!(bat.guaranteed.iter().any(|(m, _)| *m == MaterialId::LeatherWing));
        assert!(goblin.guaranteed.iter().any(|(m, _)| *m == MaterialId::Fang));
        assert!(skeleton
            .guaranteed
            .iter()
            .any(|(m, _)| *m == MaterialId::BoneShard));
        assert!(zombie.guaranteed.iter().any(|(m, _)| *m == MaterialId::RotFlesh));
    }

    #[test]
    fn king_slime_always_drops_royal_core() {
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..20 {
            let drops = roll_carve_loot(CarveTarget::KingSlime, &mut rng);
            let royal: u32 = drops
                .iter()
                .filter(|(m, _)| *m == MaterialId::RoyalSlimeCore)
                .map(|(_, n)| *n)
                .sum();
            assert!(royal >= 1, "boss must always yield at least one Royal Slime Core");
        }
    }

    #[test]
    fn roll_never_empty_for_packs() {
        let mut rng = StdRng::seed_from_u64(42);
        for kind in [
            EnemyKind::Slime,
            EnemyKind::Bat,
            EnemyKind::Goblin,
            EnemyKind::Skeleton,
            EnemyKind::Zombie,
        ] {
            let drops = roll_carve_loot(CarveTarget::Pack(kind), &mut rng);
            assert!(!drops.is_empty(), "{kind:?} should always yield something");
        }
    }

    #[test]
    fn max_loot_stacks_same_materials() {
        let max = max_carve_loot(CarveTarget::Pack(EnemyKind::Slime));
        let gel_entries = max
            .iter()
            .filter(|(m, _)| *m == MaterialId::SlimeGel)
            .count();
        assert_eq!(gel_entries, 1, "same materials should stack into one entry");
    }
}
