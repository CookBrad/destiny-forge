use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod player;
use player::{CollisionMap, Inventory, Player, move_player};

mod items;
use items::{Item, ItemStack, ItemType, crops::corn::Corn};

#[derive(Component)]
struct Draggable;

#[derive(Component)]
struct Dragging {
    entity: Entity,
    original_slot: usize,
}

#[derive(Resource, Default)]
struct DragState {
    dragging: Option<Dragging>,
}

#[derive(Event)]
struct InventoryDropEvent;

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

#[derive(Component)]
pub struct InventoryBar;

#[derive(Component)]
pub struct SlotTag;

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

fn add_item_to_inventory(mut inventory_query: Query<&mut Inventory, With<Player>>) {
    if let Ok(mut inventory) = inventory_query.get_single_mut() {
        let mut stack_count = 0;
        for _ in 0..66 {
            let mut found = false;
            let item_type_to_add = ItemType::Corn(Corn);
            for stack in &mut inventory.items {
                if let Some(stack) = stack {
                    if std::mem::discriminant(&stack.item)
                        == std::mem::discriminant(&item_type_to_add)
                    {
                        if stack.count < stack.max_count {
                            stack.count += 1;
                            found = true;
                            break;
                        }
                    }
                }
            }

            if !found {
                let max_count = match &item_type_to_add {
                    ItemType::Corn(corn) => corn.stack_size(),
                };
                let new_stack = ItemStack {
                    item: item_type_to_add,
                    count: 1,
                    max_count,
                };
                inventory.items[stack_count] = Some(new_stack);
                stack_count += 1;
            }
        }
    }
}

#[derive(Bundle)]
struct InventorySlotBundle {
    node: Node,
    border_color: BorderColor,
    slot_tag: SlotTag,
    draggable: Draggable,
    interaction: Interaction,
}

fn setup_inventory_bar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    inventory_query: Query<&Inventory, With<Player>>,
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            InventoryBar,
        ))
        .with_children(|inventory_bar| {
            let background_image = asset_server.load("inventory_bar.png");
            inventory_bar.spawn((
                Node {
                    width: Val::Px(5.0 * 50.0),
                    min_width: Val::Px(5.0 * 50.0),
                    height: Val::Px(50.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                ImageNode {
                    image: background_image,
                    ..default()
                },
                BorderColor(Color::BLACK),
            ));
        })
        .with_children(|inventory_slot| {
            if let Ok(inventory) = inventory_query.get_single() {
                for (index, item_option) in inventory.items.iter().enumerate() {
                    let bundle = InventorySlotBundle {
                        node: Node {
                            width: Val::Px(50.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        border_color: BorderColor(Color::WHITE),
                        slot_tag: SlotTag,
                        draggable: Draggable,
                        interaction: Interaction::default(),
                    };

                    if let Some(item_entity) = item_option {
                        let display_info = item_entity.item.as_item().display_info();
                        let name = display_info.name;
                        let image_path = display_info.image_path;
                        let item_image: Handle<Image> = asset_server.load(image_path);

                        inventory_slot
                            .spawn((
                                ImageNode {
                                    image: item_image,
                                    ..default()
                                },
                                Text::new(format!("{name} {count}", count = item_entity.count)),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor::default(),
                                TextLayout::default(),
                                bundle,
                            ))
                            .insert(Name::new(format!("Slot_{}", index)))
                            .with_children(|slot| {
                                slot.spawn((
                                    Node {
                                        width: Val::Px(50.0),
                                        height: Val::Px(50.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BorderColor(Color::WHITE),
                                ));
                            });
                    } else {
                        inventory_slot
                            .spawn(bundle)
                            .insert(Name::new(format!("Slot_{}", index)))
                            .with_children(|slot| {
                                slot.spawn((
                                    Node {
                                        width: Val::Px(50.0),
                                        height: Val::Px(50.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BorderColor(Color::WHITE),
                                ));
                            });
                    }
                }
            }
        });
}
fn handle_drag_start(
    mut commands: Commands,
    mut drag_state: ResMut<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    query: Query<(Entity, &Interaction, &Name), With<Draggable>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        for (entity, interaction, name) in query.iter() {
            if *interaction == Interaction::Pressed {
                if let Some(slot_num) = name.as_str().strip_prefix("Slot_") {
                    println!("Dragging item from slot: {}", slot_num);
                    if let Ok(slot_index) = slot_num.parse::<usize>() {
                        drag_state.dragging = Some(Dragging {
                            entity,
                            original_slot: slot_index,
                        });
                        commands
                            .entity(entity)
                            .insert(Transform::from_xyz(0.0, 0.0, 1.0));
                        break;
                    }
                }
            }
        }
    }
}

fn handle_drag(
    windows: Query<&Window>,
    drag_state: Res<DragState>,
    mut query: Query<&mut Transform, With<Draggable>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    if let Some(dragging) = &drag_state.dragging {
        if let Ok(mut transform) = query.get_mut(dragging.entity) {
            let window = windows.single();
            if let Some(cursor_pos) = window.cursor_position() {
                let (camera, camera_transform) = camera_q.single();
                if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    transform.translation = Vec3::new(world_pos.x, world_pos.y, 1.0);
                }
            }
        }
    }
}

fn handle_drop(
    mut commands: Commands,
    mut drag_state: ResMut<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut inventory_query: Query<&mut Inventory, With<Player>>,
    query: Query<(Entity, &Interaction, &Name), With<Draggable>>,
) {
    if mouse.just_released(MouseButton::Left) {
        if let Some(dragging) = drag_state.dragging.take() {
            let mut target_slot = None;
            println!("released item");
            commands.send_event(InventoryDropEvent);

            // Find which slot we're dropping onto
            for (entity, interaction, name) in query.iter() {
                if *interaction == Interaction::Hovered && entity != dragging.entity {
                    if let Some(slot_num) = name.as_str().strip_prefix("Slot_") {
                        if let Ok(slot_index) = slot_num.parse::<usize>() {
                            target_slot = Some(slot_index);
                            break;
                        }
                    }
                }
            }

            if let Ok(mut inventory) = inventory_query.get_single_mut() {
                if let Some(target) = target_slot {
                    // Swap or move items
                    println!("Dropping item onto slot: {}", target);
                    if dragging.original_slot != target {
                        let original_item = inventory.items[dragging.original_slot].take();
                        let target_item = inventory.items[target].take();

                        inventory.items[target] = original_item;
                        inventory.items[dragging.original_slot] = target_item;
                        println!(
                            "Moved item from slot {} to slot {}",
                            dragging.original_slot, target
                        );
                        println!("{:?}", inventory.items);
                    }
                }

                // Reset position
                commands.entity(dragging.entity).remove::<Transform>();
            }
        }
    }
}

use bevy::hierarchy::Children;

fn update_inventory_bar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    inventory_query: Query<&Inventory, With<Player>>,
    bar_query: Query<&Children, With<InventoryBar>>,
    slot_query: Query<Entity, With<SlotTag>>,
) {
    // Get the player's inventory
    if let Ok(inventory) = inventory_query.get_single() {
        // Get the children of the inventory bar
        if let Ok(children) = bar_query.get_single() {
            // Filter children to only include slot entities
            let slot_entities: Vec<Entity> = children
                .iter()
                .filter_map(|&child| {
                    if slot_query.get(child).is_ok() {
                        Some(child)
                    } else {
                        None
                    }
                })
                .collect();

            // Update each slot based on the inventory
            for (index, &slot_entity) in slot_entities.iter().enumerate() {
                if index < inventory.items.len() {
                    match &inventory.items[index] {
                        Some(item) => {
                            // Get item display information
                            let display_info = item.item.as_item().display_info();
                            // Load the image directly using asset_server
                            let image_handle = asset_server.load(display_info.image_path);
                            let text_content = format!("{} {}", display_info.name, item.count);

                            // Update the slot with the image and text
                            commands.entity(slot_entity).insert((
                                ImageNode {
                                    image: image_handle,
                                    ..Default::default()
                                },
                                Text::new(text_content),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor::default(),
                                TextLayout::default(),
                            ));
                            // .with_children(|slot| {
                            //     slot.spawn((
                            //         Node {
                            //             width: Val::Px(50.0),
                            //             height: Val::Px(50.0),
                            //             border: UiRect::all(Val::Px(2.0)),
                            //             ..default()
                            //         },
                            //         BorderColor(Color::WHITE),
                            //     ));
                            // });
                        }
                        None => {
                            // Clear the slot if no item exists
                            commands.entity(slot_entity).insert(Text::new(""));
                            commands.entity(slot_entity).remove::<ImageNode>();
                        }
                    }
                }
            }
        }
    }
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
