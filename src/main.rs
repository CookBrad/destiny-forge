use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_player,
                camera_follow,
                plant_crop,
                grow_crops,
                harvest_crop,
            ),
        )
        .run();
}

#[derive(Component)]
struct Player {
    speed: f32,
}

#[derive(Component)]
struct Crop {
    stage: CropStage,
    timer: f32,
}

#[derive(PartialEq)]
enum CropStage {
    Seed,
    Sprout,
    Mature,
}

#[derive(Resource)]
struct CollisionMap {
    width: u32,
    height: u32,
    data: Vec<bool>,
}

impl CollisionMap {
    fn get(&self, pos: IVec2) -> Option<bool> {
        if pos.x >= 0 && pos.x < self.width as i32 && pos.y >= 0 && pos.y < self.height as i32 {
            Some(self.data[(pos.y * self.width as i32 + pos.x) as usize])
        } else {
            None
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tile_texture_handle = asset_server.load("tiles.png");
    let tile_size: TilemapTileSize = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_size = TilemapSize { x: 50, y: 50 };

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(0),
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let dirt_pos = TilePos { x: 5, y: 5 };
    if let Some(tile_entity) = tile_storage.get(&dirt_pos) {
        commands.entity(tile_entity).insert(TileTextureIndex(1));
    }

    let tilemap_bundle = TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_texture_handle),
        tile_size,
        transform: Transform::from_scale(Vec3::splat(6.0))
            .with_translation(Vec3::new(-150.0, -150.0, 0.0)),
        ..default()
    };
    commands.entity(tilemap_entity).insert(tilemap_bundle);

    let player_texture = asset_server.load("player.png");
    commands.spawn((
        Sprite {
            image: player_texture,
            ..default()
        },
        Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(50.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Visibility::default(),
        Player { speed: 100.0 },
    ));

    commands.spawn(Camera2d { ..default() });

    let mut collision_data = vec![true; (map_size.x * map_size.y) as usize];
    let dirt_index = (dirt_pos.y * map_size.x + dirt_pos.x) as usize;
    collision_data[dirt_index] = false;
    commands.insert_resource(CollisionMap {
        width: map_size.x,
        height: map_size.y,
        data: collision_data,
    });
}

fn move_player(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    collision_map: Res<CollisionMap>,
    tilemap_query: Query<&TilemapGridSize>,
) {
    let grid_size = tilemap_query.single();
    let mut player_transform = player_query.single_mut();
    let speed = 100.0;

    let mut velocity = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        velocity.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        velocity.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        velocity.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        velocity.x += 1.0;
    }

    if velocity.length() > 0.0 {
        velocity = velocity.normalize() * speed;
    }

    let dt = time.delta_secs();
    let new_position =
        player_transform.translation + Vec3::new(velocity.x * dt, velocity.y * dt, 1.0);

    let tile_pos = IVec2::new(
        (new_position.x / grid_size.x).floor() as i32,
        (new_position.y / grid_size.y).floor() as i32,
    );

    if let Some(is_walkable) = collision_map.get(tile_pos) {
        if is_walkable {
            player_transform.translation = new_position;
        }
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for mut camera_transform in camera_query.iter_mut() {
            camera_transform.translation = player_transform.translation;
        }
    }
}

fn plant_crop(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    tilemap_query: Query<(&TilemapGridSize, &TileStorage)>,
    tile_texture_query: Query<&TileTextureIndex>,
    crop_query: Query<(&Crop, &Transform)>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        let player_transform = player_query.single();
        let (grid_size, tile_storage) = tilemap_query.single();

        let tile_pos = IVec2::new(
            (player_transform.translation.x / grid_size.x).floor() as i32,
            (player_transform.translation.y / grid_size.y).floor() as i32,
        );

        let tile_pos_bevy = TilePos {
            x: tile_pos.x as u32,
            y: tile_pos.y as u32,
        };

        let tile_world_pos = Vec2::new(
            tile_pos.x as f32 * grid_size.x,
            tile_pos.y as f32 * grid_size.y,
        );

        let mut crop_exists = false;
        for (_, crop_transform) in crop_query.iter() {
            let crop_pos = crop_transform.translation.truncate();
            if crop_pos == tile_world_pos {
                crop_exists = true;
                break;
            }
        }

        if !crop_exists {
            if let Some(tile_entity) = tile_storage.get(&tile_pos_bevy) {
                if let Ok(tile_texture) = tile_texture_query.get(tile_entity) {
                    if tile_texture.0 == 0 {
                        let crop_texture = asset_server.load("crop.png");
                        commands.spawn((
                            Sprite {
                                image: crop_texture,
                                ..Default::default()
                            },
                            Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(
                                tile_world_pos.x,
                                tile_world_pos.y,
                                0.5,
                            )),
                            Crop {
                                stage: CropStage::Seed,
                                timer: 0.0,
                            },
                        ));
                    }
                }
            }
        }
    }
}

fn grow_crops(
    time: Res<Time>,
    mut crop_query: Query<(&mut Crop, &mut Sprite)>,
    asset_server: Res<AssetServer>,
) {
    for (mut crop, mut sprite) in crop_query.iter_mut() {
        crop.timer += time.delta_secs();
        match crop.stage {
            CropStage::Seed if crop.timer >= 5.0 && crop.timer < 15.0 => {
                crop.stage = CropStage::Sprout;
                sprite.image = asset_server.load("crop_sprout.png"); // Fixed: texture -> image
            }
            CropStage::Sprout if crop.timer >= 15.0 => {
                crop.stage = CropStage::Mature;
                sprite.image = asset_server.load("crop_mature.png"); // Fixed: texture -> image
            }
            _ => {}
        }
    }
}

fn harvest_crop(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    tilemap_query: Query<&TilemapGridSize>,
    mut crop_query: Query<(Entity, &Crop, &Transform)>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        let player_transform = player_query.single();
        let grid_size = tilemap_query.single();

        let player_tile_pos = IVec2::new(
            (player_transform.translation.x / grid_size.x).floor() as i32,
            (player_transform.translation.y / grid_size.y).floor() as i32,
        );

        for (entity, crop, crop_transform) in crop_query.iter_mut() {
            let crop_tile_pos = IVec2::new(
                (crop_transform.translation.x / grid_size.x).floor() as i32,
                (crop_transform.translation.y / grid_size.y).floor() as i32,
            );

            if player_tile_pos == crop_tile_pos && crop.stage == CropStage::Mature {
                commands.entity(entity).despawn();
                println!("Crop harvested!");
            }
        }
    }
}
