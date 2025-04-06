use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod player;
use player::{CollisionMap, Inventory, Player, move_player};

mod inventory_ui;
use inventory_ui::*;

mod items;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_event::<InventoryDropEvent>()
        .insert_resource(DragState::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, add_item_to_inventory.after(setup))
        .add_systems(Startup, setup_inventory_bar.after(add_item_to_inventory))
        .add_systems(Update, move_player)
        .add_systems(Update, (handle_drag_start, handle_drag, handle_drop))
        .add_systems(
            Update,
            update_inventory_bar.run_if(on_event::<InventoryDropEvent>),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tile_texture_handle = asset_server.load("tiles.png");
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
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
        Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(50.0, 0.0, 1.0)),
        GlobalTransform::default(),
        Visibility::default(),
        Player { speed: 100.0 },
        Inventory {
            items: vec![None; 5],
        },
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

// fn plant_crop(
//     mut commands: Commands,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     player_query: Query<&Transform, With<Player>>,
//     tilemap_query: Query<(&TilemapGridSize, &TileStorage)>,
//     tile_texture_query: Query<&TileTextureIndex>,
//     crop_query: Query<(&Crop, &Transform)>,
//     asset_server: Res<AssetServer>,
// ) {
//     if keyboard_input.just_pressed(KeyCode::KeyE) {
//         let player_transform = player_query.single();
//         let (grid_size, tile_storage) = tilemap_query.single();

//         let tile_pos = IVec2::new(
//             (player_transform.translation.x / grid_size.x).floor() as i32,
//             (player_transform.translation.y / grid_size.y).floor() as i32,
//         );

//         let tile_pos_bevy = TilePos {
//             x: tile_pos.x as u32,
//             y: tile_pos.y as u32,
//         };

//         let tile_world_pos = Vec2::new(
//             tile_pos.x as f32 * grid_size.x,
//             tile_pos.y as f32 * grid_size.y,
//         );

//         let mut crop_exists = false;
//         for (_, crop_transform) in crop_query.iter() {
//             let crop_pos = crop_transform.translation.truncate();
//             if crop_pos == tile_world_pos {
//                 crop_exists = true;
//                 break;
//             }
//         }

//         if !crop_exists {
//             if let Some(tile_entity) = tile_storage.get(&tile_pos_bevy) {
//                 if let Ok(tile_texture) = tile_texture_query.get(tile_entity) {
//                     if tile_texture.0 == 0 {
//                         let crop_texture = asset_server.load("crop.png");
//                         commands.spawn((
//                             Sprite {
//                                 image: crop_texture,
//                                 ..Default::default()
//                             },
//                             Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(
//                                 tile_world_pos.x,
//                                 tile_world_pos.y,
//                                 0.5,
//                             )),
//                             Crop {
//                                 stage: CropStage::Seed,
//                                 timer: 0.0,
//                             },
//                         ));
//                     }
//                 }
//             }
//         }
//     }
// }

// fn grow_crops(
//     time: Res<Time>,
//     mut crop_query: Query<(&mut Crop, &mut Sprite)>,
//     asset_server: Res<AssetServer>,
// ) {
//     for (mut crop, mut sprite) in crop_query.iter_mut() {
//         crop.timer += time.delta_secs();
//         match crop.stage {
//             CropStage::Seed if crop.timer >= 5.0 && crop.timer < 15.0 => {
//                 crop.stage = CropStage::Sprout;
//                 sprite.image = asset_server.load("crop_sprout.png"); // Fixed: texture -> image
//             }
//             CropStage::Sprout if crop.timer >= 15.0 => {
//                 crop.stage = CropStage::Mature;
//                 sprite.image = asset_server.load("crop_mature.png"); // Fixed: texture -> image
//             }
//             _ => {}
//         }
//     }
// }

// fn harvest_crop(
//     mut commands: Commands,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     player_query: Query<&Transform, With<Player>>,
//     tilemap_query: Query<&TilemapGridSize>,
//     mut crop_query: Query<(Entity, &Crop, &Transform)>,
// ) {
//     if keyboard_input.just_pressed(KeyCode::KeyE) {
//         let player_transform = player_query.single();
//         let grid_size = tilemap_query.single();

//         let player_tile_pos = IVec2::new(
//             (player_transform.translation.x / grid_size.x).floor() as i32,
//             (player_transform.translation.y / grid_size.y).floor() as i32,
//         );

//         for (entity, crop, crop_transform) in crop_query.iter_mut() {
//             let crop_tile_pos = IVec2::new(
//                 (crop_transform.translation.x / grid_size.x).floor() as i32,
//                 (crop_transform.translation.y / grid_size.y).floor() as i32,
//             );

//             if player_tile_pos == crop_tile_pos && crop.stage == CropStage::Mature {
//                 commands.entity(entity).despawn();
//                 println!("Crop harvested!");
//             }
//         }
//     }
// }
