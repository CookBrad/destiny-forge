use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod player;
use items::ItemCategory;
use player::{CollisionMap, Inventory, Player, move_player};

mod inventory_ui;
use inventory_ui::*;

mod crops;
use crops::{Crop, GrowthStage};

mod items;

const SCALE: f32 = 3.0;

#[derive(Resource, Clone)]
struct SpriteSheetLayout {
    layout: Handle<TextureAtlasLayout>,
    texture: Handle<Image>,
}

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
        .add_systems(Update, grow_crops)
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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Get window dimensions (not used for centering here, but included for reference)
    let window = windows.single();
    let _window_width = window.resolution.width();
    let _window_height = window.resolution.height();

    // **Load Assets**
    let tile_texture_handle = asset_server.load("tiles.png");
    let player_texture: Handle<Image> = asset_server.load("player.png");
    let texture: Handle<Image> = asset_server.load("crops.png");

    let layout = TextureAtlasLayout::from_grid(UVec2 { x: 16, y: 32 }, 16, 26, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // Store the handle in a resource
    commands.insert_resource(SpriteSheetLayout {
        layout: texture_atlas_layout,
        texture,
    });

    // **Tilemap Configuration**
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_size = TilemapSize { x: 30, y: 20 };

    // Calculate the center offset to position tilemap at world (0, 0)
    let center_x = ((map_size.x - 1) as f32 * tile_size.x) / 2.0; // 192.0
    let center_y = ((map_size.y - 1) as f32 * tile_size.y) / 2.0; // 192.0
    let translation = Vec3::new(-center_x * SCALE, -center_y * SCALE, 0.0);

    // **Spawn Tilemap**
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    // Populate tiles, including the dirt tile at (5, 5)
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            // Set texture index 1 for dirt at (5, 5), 0 for others
            let texture_index = if x == 5 && y == 5 { 1 } else { 0 };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(texture_index),
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    // Attach the tilemap components
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_texture_handle),
        tile_size,
        transform: Transform::from_translation(translation)
            .with_scale(Vec3::new(SCALE, SCALE, 1.0)),
        ..default()
    });

    // **Spawn Player**
    // Place player at the center of the tilemap (world coordinates (0, 0, 1))
    let player_start_pos = Vec3::new(0.0, 0.0, 1.0);
    commands.spawn((
        Sprite {
            image: player_texture,
            ..default()
        },
        Transform::from_translation(player_start_pos).with_scale(Vec3::new(SCALE, SCALE, 1.0)),
        Player { speed: 100.0 },
        Inventory {
            items: vec![None; 5], // 5 slots, initially empty
        },
    ));

    // **Spawn Camera**
    commands.spawn(Camera2d::default());

    // **Collision Map**
    // Initialize with all tiles collidable (true)
    let mut collision_data = vec![true; (map_size.x * map_size.y) as usize];
    // Dirt tile at (5, 5) is not collidable (false)
    let dirt_index = (5 * map_size.x + 5) as usize; // Row-major indexing
    collision_data[dirt_index] = false;
    commands.insert_resource(CollisionMap {
        width: map_size.x,
        height: map_size.y,
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
    sprite_sheet: Res<SpriteSheetLayout>,
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
            let mut is_empty = false;
            if let Some(item) = &mut inventory.items[selected_slot.0] {
                match item.item_type.category() {
                    ItemCategory::Crop => {
                        if let Some(crop_to_plant) = item.item_type.plant() {
                            if let Some(tile_entity) = tile_storage.get(&tile_pos_bevy) {
                                if let Ok(tile_texture) = tile_texture_query.get(tile_entity) {
                                    if tile_texture.0 == 0 {
                                        if item.count > 0 {
                                            item.count -= 1;
                                            is_empty = item.count == 0;
                                            commands.send_event(InventoryUpdateEvent);
                                        }
                                        println!("Item count: {:?}", item.count);
                                        println!("{}", crop_to_plant.growth_stage_image());
                                        commands.spawn((
                                            Sprite {
                                                image: sprite_sheet.texture.clone(),
                                                texture_atlas: Some(TextureAtlas {
                                                    layout: sprite_sheet.layout.clone(),
                                                    index: crop_to_plant.growth_stage_image(), // Start with the first sprite
                                                }),
                                                ..Default::default()
                                            },
                                            Transform {
                                                translation: tile_world_pos
                                                    + Vec3::new(0.0, 0.0, 0.5), // z=0.5 to be above tile
                                                scale: Vec3::splat(SCALE), // Match tilemap scale
                                                ..Default::default()
                                            },
                                            crop_to_plant,
                                        ));
                                        println!("Placed item: {:?}", item.item_type.category());
                                    }
                                }
                            }
                        }
                    }
                    ItemCategory::Food => {}
                    ItemCategory::Weapon => {}
                    ItemCategory::Armor => {}
                    ItemCategory::Tool => {}
                }
            }
            if is_empty {
                inventory.items[selected_slot.0] = None;
            }
        } else {
            println!("No item in selected slot");
        }
    };
}

fn grow_crops(
    time: Res<Time>,
    mut crop_query: Query<(&mut Crop, &mut Sprite)>,
    sprite_sheet: Res<SpriteSheetLayout>,
) {
    for (mut crop, mut sprite) in crop_query.iter_mut() {
        crop.timer += time.delta_secs();
        match crop.get_stage() {
            GrowthStage::Seed if crop.timer >= 5.0 && crop.timer < 15.0 => {
                crop.set_stage(GrowthStage::Sprout);
            }
            GrowthStage::Sprout if crop.timer >= 15.0 && crop.timer < 20.0 => {
                crop.set_stage(GrowthStage::Immature);
            }
            GrowthStage::Immature if crop.timer >= 20.0 && crop.timer < 25.0 => {
                crop.set_stage(GrowthStage::Mature);
            }
            GrowthStage::Mature if crop.timer >= 25.0 => {
                crop.set_stage(GrowthStage::Fruiting);
            }
            _ => {}
        }
        sprite.texture_atlas = Some(TextureAtlas {
            layout: sprite_sheet.layout.clone(),
            index: crop.growth_stage_image(), // Start with the first sprite
        });
    }
}

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
