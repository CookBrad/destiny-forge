use bevy::prelude::*;

use crate::core::GameState;
use crate::graphics::{
    image_sprite, pixel_sprite, spawn_decorative_hub_props, spawn_hub_background,
    spawn_hub_grass_field, sprite_transform, GameSprites, HubPlayerAnimation, HubTile, PixelSheet,
    PlayerSprite,
};
use crate::player::HubPlayer;

#[derive(Component, Copy, Clone)]
pub struct HubEntity;

#[derive(Component)]
pub struct ForgeStation;

#[derive(Component)]
pub struct DungeonEntrance;

const HUB_MOVE_SPEED: f32 = 160.0;

pub fn setup_hub(mut commands: Commands, sprites: Res<GameSprites>) {
    spawn_hub_background(&mut commands, &sprites, HubEntity);
    spawn_hub_grass_field(
        &mut commands,
        &sprites,
        Vec2::new(0.0, -120.0),
        11,
        7,
        HubEntity,
    );
    spawn_decorative_hub_props(&mut commands, &sprites, HubEntity);
    spawn_dirt_path(&mut commands, &sprites);
    spawn_hub_player(&mut commands, &sprites);
    spawn_forge(&mut commands, &sprites, Vec2::new(-150.0, -20.0));
    spawn_dungeon_entrance(&mut commands, &sprites, Vec2::new(170.0, -10.0));
}

fn spawn_dirt_path(commands: &mut Commands, sprites: &GameSprites) {
    for offset in -2..=2 {
        let position = Vec3::new(offset as f32 * 48.0, -40.0, 1.0);
        commands.spawn((
            pixel_sprite(sprites, PixelSheet::Hub, HubTile::DirtA.atlas_index()),
            sprite_transform(position),
            HubEntity,
        ));
    }
}

fn spawn_hub_player(commands: &mut Commands, sprites: &GameSprites) {
    commands.spawn((
        pixel_sprite(
            sprites,
            PixelSheet::Player,
            PlayerSprite::Down0.atlas_index(),
        ),
        sprite_transform(Vec3::new(0.0, 0.0, 10.0)),
        HubPlayerAnimation::default(),
        HubPlayer,
        HubEntity,
        Name::new("HubPlayer"),
    ));
}

fn spawn_forge(commands: &mut Commands, sprites: &GameSprites, position: Vec2) {
    commands.spawn((
        image_sprite(sprites.forge_building.clone(), Vec2::new(64.0, 48.0)),
        sprite_transform(position.extend(8.0)),
        ForgeStation,
        HubEntity,
        Name::new("Forge"),
    ));
}

fn spawn_dungeon_entrance(commands: &mut Commands, sprites: &GameSprites, position: Vec2) {
    commands.spawn((
        image_sprite(sprites.mine_entrance.clone(), Vec2::new(48.0, 64.0)),
        sprite_transform(position.extend(8.0)),
        DungeonEntrance,
        HubEntity,
        Name::new("DungeonEntrance"),
    ));
}

pub fn hub_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<HubPlayer>>,
) {
    let Ok(mut transform) = query.get_single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        transform.translation += (direction.normalize() * HUB_MOVE_SPEED * time.delta_secs())
            .extend(0.0);
    }
}

pub fn enter_dungeon(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<&Transform, With<HubPlayer>>,
    entrance_query: Query<&Transform, With<DungeonEntrance>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    for entrance_transform in &entrance_query {
        let distance = player_transform
            .translation
            .truncate()
            .distance(entrance_transform.translation.truncate());

        if distance < 90.0 {
            next_state.set(GameState::Dungeon);
            return;
        }
    }
}

pub fn cleanup_hub(mut commands: Commands, query: Query<Entity, With<HubEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}