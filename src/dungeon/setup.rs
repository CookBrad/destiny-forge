use bevy::prelude::*;

use crate::graphics::{
    dungeon_tile_sprite, pixel_sprite, spawn_dungeon_background, spawn_dungeon_platform_tiles,
    spawn_dungeon_tile_floor, sprite_transform, DungeonSprite, GameSprites, PixelSheet,
    PlatformBounds,
};

use super::enemy::{spawn_enemy, EnemyKind};
use super::movement::spawn_dungeon_player;

pub const DUNGEON_FLOOR_Y: f32 = -220.0;

#[derive(Component, Copy, Clone)]
pub struct DungeonEntity;

#[derive(Component)]
pub struct GroundPlatform;

#[derive(Component)]
pub struct DungeonExit;

pub fn setup_dungeon(mut commands: Commands, sprites: Res<GameSprites>) {
    spawn_dungeon_background(&mut commands, &sprites, DungeonEntity);

    spawn_dungeon_tile_floor(
        &mut commands,
        &sprites,
        Vec2::new(0.0, DUNGEON_FLOOR_Y - 8.0),
        19,
        1,
        DungeonSprite::CaveFloorA,
        DungeonEntity,
    );

    spawn_platform(
        &mut commands,
        &sprites,
        Vec2::new(0.0, DUNGEON_FLOOR_Y),
        Vec2::new(900.0, 48.0),
    );
    spawn_platform(
        &mut commands,
        &sprites,
        Vec2::new(-260.0, -120.0),
        Vec2::new(220.0, 48.0),
    );
    spawn_platform(
        &mut commands,
        &sprites,
        Vec2::new(220.0, -60.0),
        Vec2::new(260.0, 48.0),
    );

    spawn_torch(&mut commands, &sprites, Vec2::new(-360.0, DUNGEON_FLOOR_Y + 30.0));
    spawn_torch(&mut commands, &sprites, Vec2::new(60.0, -90.0));

    spawn_dungeon_player(&mut commands, &sprites);
    spawn_enemy(
        &mut commands,
        &sprites,
        EnemyKind::Slime,
        Vec2::new(-120.0, super::enemy::ground_enemy_y()),
    );
    spawn_enemy(
        &mut commands,
        &sprites,
        EnemyKind::Bat,
        Vec2::new(180.0, -20.0),
    );
    spawn_enemy(
        &mut commands,
        &sprites,
        EnemyKind::Slime,
        Vec2::new(320.0, super::enemy::ground_enemy_y()),
    );

    let exit_y = DUNGEON_FLOOR_Y + 24.0;
    commands.spawn((
        pixel_sprite(
            &sprites,
            PixelSheet::Dungeon,
            DungeonSprite::LadderExit.atlas_index(),
        ),
        sprite_transform(Vec3::new(420.0, exit_y + 20.0, 5.0)),
        DungeonExit,
        DungeonEntity,
        Name::new("DungeonExit"),
    ));
}

fn spawn_platform(
    commands: &mut Commands,
    sprites: &GameSprites,
    center: Vec2,
    size: Vec2,
) {
    spawn_dungeon_platform_tiles(commands, sprites, center, size);
    commands.spawn((
        Transform::from_translation(center.extend(0.0)),
        GroundPlatform,
        PlatformBounds {
            half_size: size * 0.5,
        },
        DungeonEntity,
    ));
}

fn spawn_torch(commands: &mut Commands, sprites: &GameSprites, position: Vec2) {
    commands.spawn((
        dungeon_tile_sprite(sprites, DungeonSprite::Torch),
        sprite_transform(position.extend(6.0)),
        DungeonEntity,
    ));
}

pub fn cleanup_dungeon(mut commands: Commands, query: Query<Entity, With<DungeonEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}