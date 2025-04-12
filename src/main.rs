use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod player;
use player::{CollisionMap, Inventory, Player, move_player};

mod inventory_ui;
use inventory_ui::*;

mod items;

const SCALE: f32 = 3.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_event::<InventoryUpdateEvent>()
        .insert_resource(DragState::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, add_item_to_inventory.after(setup))
        .add_systems(Startup, setup_inventory_bar.after(add_item_to_inventory))
        .add_systems(Startup, update_slot_borders.after(setup_inventory_bar))
        .add_systems(Update, (move_player, player_action))
        .add_systems(
            Update,
            (
                handle_drag_start,
                handle_drag,
                handle_drop,
                handle_inventory_scroll,
                update_slot_borders,
            ),
        )
        .add_systems(
            Update,
            update_inventory_bar.run_if(on_event::<InventoryUpdateEvent>),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, windows: Query<&Window>) {
    let window = windows.single();
    let window_width = window.resolution.width();
    let window_height = window.resolution.height();

    let tile_texture_handle = asset_server.load("tiles.png");
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_size = TilemapSize { x: 25, y: 25 };

    let tilemap_width = map_size.x as f32 * tile_size.x; // 50 * 16.0 = 800.0
    let tilemap_height = map_size.y as f32 * tile_size.y; // 50 * 16.0 = 800.0
    let scale_x = window_width / tilemap_width;
    let scale_y = window_height / tilemap_height;

    let tx = -window_width / 2.0 + (tile_size.x / 2.0) * scale_x;
    let ty = -window_height / 2.0 + (tile_size.y / 2.0) * scale_y;

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
        transform: Transform::from_scale(Vec3::splat(SCALE))
            .with_translation(Vec3::new(tx, ty, 0.0)),
        ..default()
    };
    commands.entity(tilemap_entity).insert(tilemap_bundle);

    let player_texture = asset_server.load("player.png");
    commands.spawn((
        Sprite {
            image: player_texture,
            ..default()
        },
        Transform::from_scale(Vec3::splat(SCALE)).with_translation(Vec3::new(50.0, 0.0, 1.0)),
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
        width: tilemap_width as u32,
        height: tilemap_height as u32,
        data: collision_data,
    });
}

fn player_action(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    tilemap_query: Query<(&TilemapGridSize, &TileStorage, &Transform)>,
    tile_texture_query: Query<&TileTextureIndex>,
    mut inventory_query: Query<&mut Inventory>,
    selected_slot: Res<SelectedSlot>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        let player_transform = player_query.single();
        let (grid_size, tile_storage, tilemap_transform) = tilemap_query.single();

        let world_pos = player_transform.translation;
        let local_pos = (world_pos - tilemap_transform.translation) / tilemap_transform.scale.x;

        // Step 2: Calculate the tile position (for tiles centered in local space)
        let tile_x = ((local_pos.x + grid_size.x / 2.0) / grid_size.x).floor() as u32;
        let tile_y = ((local_pos.y + grid_size.y / 2.0) / grid_size.y).floor() as u32;
        let tile_pos_bevy = TilePos {
            x: tile_x,
            y: tile_y,
        };

        // Step 3: Compute the tile's center in local space
        let tile_local_pos = Vec3::new(
            tile_pos_bevy.x as f32 * grid_size.x,
            tile_pos_bevy.y as f32 * grid_size.y,
            0.0,
        );

        // Step 4: Convert the tile's local position to world space
        let tile_world_pos = tilemap_transform.transform_point(tile_local_pos);

        if let Ok(mut inventory) = inventory_query.get_single_mut() {
            if let Some(item) = &mut inventory.items[selected_slot.0] {
                if let Some(tile_entity) = tile_storage.get(&tile_pos_bevy) {
                    if let Ok(tile_texture) = tile_texture_query.get(tile_entity) {
                        if tile_texture.0 == 0 {
                            if item.count > 0 {
                                item.count -= 1;
                                commands.send_event(InventoryUpdateEvent);
                            }
                            println!("Item count: {:?}", item.count);
                            let crop_texture = asset_server.load("crop.png");
                            commands.spawn((
                                Sprite {
                                    image: crop_texture,
                                    ..Default::default()
                                },
                                Transform {
                                    translation: tile_world_pos + Vec3::new(0.0, 0.0, 0.5), // z=0.5 to be above tile
                                    scale: Vec3::splat(SCALE), // Match tilemap scale
                                    ..Default::default()
                                },
                            ));
                            println!("Placed item: {:?}", item.item.category());
                        }
                    }
                }
            }
        } else {
            println!("No item in selected slot");
        }
    };

    // let mut crop_exists = false;
    // for (_, crop_transform) in crop_query.iter() {
    //     let crop_pos = crop_transform.translation.truncate();
    //     if crop_pos == tile_world_pos {
    //         crop_exists = true;
    //         break;
    //     }
    // }

    // if !crop_exists {
    //     if let Some(tile_entity) = tile_storage.get(&tile_pos_bevy) {
    //         if let Ok(tile_texture) = tile_texture_query.get(tile_entity) {
    //             if tile_texture.0 == 0 {
    //                 let crop_texture = asset_server.load("crop.png");
    //                 commands.spawn((
    //                     Sprite {
    //                         image: crop_texture,
    //                         ..Default::default()
    //                     },
    //                     Transform::from_scale(Vec3::splat(SCALE)).with_translation(Vec3::new(
    //                         tile_world_pos.x,
    //                         tile_world_pos.y,
    //                         0.5,
    //                     )),
    //                 ));
    //             }
    //         }
    //     }
    // }
}
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
