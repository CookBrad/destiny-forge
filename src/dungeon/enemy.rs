use bevy::prelude::*;

use crate::combat::{Health, Hurtbox};
use crate::graphics::{
    pixel_sprite, sprite_transform, DungeonSprite, GameSprites, PixelSheet, PIXEL_SCALE, TILE_SIZE,
};
use crate::items::MaterialId;
use crate::player::DungeonPlayer;

use super::carve::CarvableCorpse;
use super::setup::{DungeonEntity, DUNGEON_FLOOR_Y};

#[derive(Clone, Copy, Debug)]
pub enum EnemyKind {
    Slime,
    Bat,
}

impl EnemyKind {
    pub fn max_health(self) -> f32 {
        match self {
            Self::Slime => 30.0,
            Self::Bat => 22.0,
        }
    }

    pub fn contact_damage(self) -> f32 {
        match self {
            Self::Slime => 8.0,
            Self::Bat => 6.0,
        }
    }

    pub fn sprite(self) -> DungeonSprite {
        match self {
            Self::Slime => DungeonSprite::Slime,
            Self::Bat => DungeonSprite::Bat,
        }
    }

    pub fn carve_loot(self) -> &'static [(MaterialId, u32)] {
        match self {
            Self::Slime => &[(MaterialId::SlimeGel, 2), (MaterialId::SlimeCore, 1)],
            Self::Bat => &[
                (MaterialId::LeatherWing, 1),
                (MaterialId::Fang, 1),
                (MaterialId::IronScrap, 1),
            ],
        }
    }
}

#[derive(Component)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub attack_cooldown: Timer,
    pub patrol_direction: f32,
}

pub fn spawn_enemy(
    commands: &mut Commands,
    sprites: &GameSprites,
    kind: EnemyKind,
    position: Vec2,
) -> Entity {
    commands
        .spawn((
            pixel_sprite(sprites, PixelSheet::Dungeon, kind.sprite().atlas_index()),
            sprite_transform(position.extend(10.0)),
            Health::new(kind.max_health()),
            Hurtbox,
            Enemy {
                kind,
                attack_cooldown: Timer::from_seconds(1.0, TimerMode::Repeating),
                patrol_direction: 1.0,
            },
            DungeonEntity,
            Name::new(match kind {
                EnemyKind::Slime => "Slime",
                EnemyKind::Bat => "Bat",
            }),
        ))
        .id()
}

pub fn ground_enemy_y() -> f32 {
    let platform_half_height = 24.0;
    let enemy_half_height = TILE_SIZE as f32 * PIXEL_SCALE * 0.5;
    DUNGEON_FLOOR_Y + platform_half_height + enemy_half_height
}

pub fn enemy_patrol(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Enemy)>,
) {
    for (mut transform, mut enemy) in &mut query {
        transform.translation.x += enemy.patrol_direction * 40.0 * time.delta_secs();

        if transform.translation.x > 280.0 {
            enemy.patrol_direction = -1.0;
        } else if transform.translation.x < -280.0 {
            enemy.patrol_direction = 1.0;
        }
    }
}

pub fn enemy_chase_player(
    time: Res<Time>,
    player_query: Query<&Transform, With<DungeonPlayer>>,
    mut enemy_query: Query<(&mut Transform, &mut Enemy), Without<DungeonPlayer>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    for (mut enemy_transform, mut enemy) in &mut enemy_query {
        let delta = player_transform.translation.x - enemy_transform.translation.x;
        if delta.abs() < 180.0 {
            enemy.patrol_direction = delta.signum();
            enemy_transform.translation.x += enemy.patrol_direction * 70.0 * time.delta_secs();
        }
    }
}

pub fn enemy_contact_damage(
    time: Res<Time>,
    mut player_query: Query<(&Transform, &mut Health), With<DungeonPlayer>>,
    mut enemy_query: Query<(&Transform, &mut Enemy)>,
) {
    let Ok((player_transform, mut player_health)) = player_query.get_single_mut() else {
        return;
    };

    for (enemy_transform, mut enemy) in &mut enemy_query {
        enemy.attack_cooldown.tick(time.delta());
        if !enemy.attack_cooldown.just_finished() {
            continue;
        }

        let distance = player_transform
            .translation
            .truncate()
            .distance(enemy_transform.translation.truncate());

        if distance < 36.0 {
            player_health.take_damage(enemy.kind.contact_damage());
        }
    }
}

pub fn spawn_corpse_on_death(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    query: Query<(Entity, &Transform, &Health, &Enemy)>,
) {
    for (entity, transform, health, enemy) in &query {
        if health.is_alive() {
            continue;
        }

        commands.entity(entity).despawn();
        commands.spawn((
            pixel_sprite(
                &sprites,
                PixelSheet::Dungeon,
                DungeonSprite::Corpse.atlas_index(),
            ),
            sprite_transform(transform.translation),
            CarvableCorpse {
                loot: enemy.kind.carve_loot(),
            },
            DungeonEntity,
            Name::new("CarvableCorpse"),
        ));
    }
}