use bevy::prelude::*;

use crate::items::{MaterialId, MaterialInventory};
use crate::player::{DungeonPlayer, PlayerLoadout};

#[derive(Component)]
pub struct CarvableCorpse {
    pub loot: &'static [(MaterialId, u32)],
}

pub fn carve_nearby_corpses(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    player_query: Query<&Transform, With<DungeonPlayer>>,
    corpse_query: Query<(Entity, &Transform, &CarvableCorpse)>,
    mut inventory: ResMut<MaterialInventory>,
    loadout: Res<PlayerLoadout>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let carve_multiplier = loadout.carve_speed_multiplier();

    for (entity, corpse_transform, corpse) in &corpse_query {
        let distance = player_transform
            .translation
            .truncate()
            .distance(corpse_transform.translation.truncate());

        if distance > 48.0 {
            continue;
        }

        for (material, amount) in corpse.loot {
            let bonus_amount = if carve_multiplier > 1.0 && *material == MaterialId::SlimeGel {
                1
            } else {
                0
            };
            inventory.add(*material, amount + bonus_amount);
        }

        commands.entity(entity).despawn();
    }
}