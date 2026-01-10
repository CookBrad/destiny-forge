use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod player;
use items::{ItemCategory, ItemStack};
use player::{
    Direction, Inventory, Player, move_player, setup_player_health_bar, update_player_health_bar,
};

mod inventory_ui;
use inventory_ui::*;

mod crops;
use crops::{Crop, GrowthStage};

mod items;

mod enemy;
use enemy::{
    Health as EnemyHealth, despawn_dead_enemies, enemy_ai, spawn_enemy, update_enemy_health_bars,
};

mod combat;
use combat::{
    enemy_attack_player, handle_sword_attack, update_attack_hitboxes, update_attacking_state,
};

const SCALE: f32 = 3.0;

#[derive(Resource, Clone)]
struct SpriteSheetLayout {
    crops_layout: Handle<TextureAtlasLayout>,
    crops_texture: Handle<Image>,
    player_layout: Handle<TextureAtlasLayout>,
    player_texture: Handle<Image>,
}

#[derive(Component)]
struct HarvestedItemSprite {
    target: Vec3, // Target position (above player's head)
    speed: f32,   // Speed of movement
    timer: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_event::<InventoryUpdateEvent>()
        .insert_resource(DragState::default())
        .init_resource::<player::PlayerHitShake>()
        .add_systems(Startup, setup)
        .add_systems(Startup, add_corn_to_inventory.after(setup))
        .add_systems(Startup, setup_inventory_bar.after(add_corn_to_inventory))
        .add_systems(Startup, update_slot_borders.after(setup_inventory_bar))
        .add_systems(Startup, setup_player_health_bar.after(setup_inventory_bar))
        .add_systems(Update, move_harvested_items)
        .add_systems(Update, (move_player, player_action))
        .add_systems(Update, grow_crops)
        .add_systems(
            Update,
            (
                enemy_ai,
                enemy_attack_player,
                update_enemy_health_bars,
                despawn_dead_enemies,
                handle_sword_attack,
                update_attack_hitboxes,
                update_attacking_state,
            ),
        )
        .add_systems(
            Update,
            (
                handle_drag_start,
                handle_drag,
                handle_drop,
                handle_inventory_scroll,
                handle_right_click_selection,
                update_slot_borders,
                update_player_health_bar,
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

    let player_texture: Handle<Image> = asset_server.load("willy.png");
    let player_layout = TextureAtlasLayout::from_grid(UVec2 { x: 16, y: 32 }, 4, 4, None, None);
    let player_texture_atlas_layout = texture_atlas_layouts.add(player_layout);

    let crops_texture: Handle<Image> = asset_server.load("crops.png");
    let crops_layout = TextureAtlasLayout::from_grid(UVec2 { x: 16, y: 32 }, 16, 26, None, None);
    let crops_texture_atlas_layout = texture_atlas_layouts.add(crops_layout);

    // **Tilemap Configuration**
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_size = TilemapSize { x: 30, y: 20 };

    // Calculate the center offset to position tilemap at world (0, 0)
    let center_x = ((map_size.x - 1) as f32 * tile_size.x) / 2.0; // 192.0
    let center_y = ((map_size.y - 1) as f32 * tile_size.y) / 2.0; // 192.0
    let translation = Vec3::new(-center_x * SCALE, -center_y * SCALE, -900.0);

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
    let player_start_pos = Vec3::new(0.0, 0.0, 500.0);
    commands.spawn((
        Sprite {
            image: player_texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: player_texture_atlas_layout.clone(),
                index: 0, // Start with the first sprite
            }),
            ..Default::default()
        },
        Transform::from_translation(player_start_pos).with_scale(Vec3::new(SCALE, SCALE, 1.0)),
        Player::default(),
        Inventory {
            items: vec![None; 5], // 5 slots, initially empty
        },
        EnemyHealth::new(100.0), // Player health
    ));

    // **Spawn Enemies**
    let enemy_positions = vec![
        Vec3::new(100.0, 100.0, 500.0),
        Vec3::new(-100.0, 100.0, 500.0),
        Vec3::new(100.0, -100.0, 500.0),
    ];

    for pos in enemy_positions {
        spawn_enemy(
            &mut commands,
            pos,
            &SpriteSheetLayout {
                crops_layout: crops_texture_atlas_layout.clone(),
                crops_texture: crops_texture.clone(),
                player_layout: player_texture_atlas_layout.clone(),
                player_texture: player_texture.clone(),
            },
        );
    }

    commands.insert_resource(SpriteSheetLayout {
        crops_layout: crops_texture_atlas_layout,
        crops_texture,
        player_layout: player_texture_atlas_layout,
        player_texture,
    });

    // **Spawn Camera**
    commands.spawn(Camera2d::default());
}

fn player_action(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&Transform, &Player)>,
    tilemap_query: Query<(&TilemapGridSize, &TileStorage, &Transform)>,
    tile_texture_query: Query<&TileTextureIndex>,
    mut inventory_query: Query<&mut Inventory>,
    mut query_crops: Query<(&mut Crop, &Transform)>,
    selected_slot: Res<SelectedSlot>,
    sprite_sheet: Res<SpriteSheetLayout>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        let (player_transform, player) = player_query.single();
        let (grid_size, tile_storage, tilemap_transform) = tilemap_query.single();

        let world_pos = player_transform.translation.truncate();

        // Player's tile position
        let player_tile = world_to_tile(world_pos, tilemap_transform, grid_size);

        // Compute the faced tile position
        let mut player_tile_faced = player_tile;
        match player.direction {
            Direction::Up => {
                player_tile_faced.y = (player_tile_faced.y + 1).min(19);
                player_tile_faced.x = player_tile_faced.x.saturating_sub(1);
            }
            Direction::Down => {
                player_tile_faced.y = player_tile_faced.y.saturating_sub(1);
                player_tile_faced.x = player_tile_faced.x.saturating_sub(1)
            }
            Direction::Left => player_tile_faced.x = player_tile_faced.x.saturating_sub(1),
            Direction::Right => player_tile_faced.x = player_tile_faced.x + 1,
        }

        let mut crops_to_harvest = Vec::new();
        for (crop, crop_transform) in query_crops.iter_mut() {
            let crop_tile = world_to_tile(
                crop_transform.translation.truncate(),
                tilemap_transform,
                grid_size,
            );

            let crop_x = crop_tile.x as i32;
            let crop_y = crop_tile.y as i32;
            let faced_x = player_tile.x as i32; // Use player's tile, not faced tile, per original logic
            let faced_y = player_tile.y as i32;

            if faced_x >= crop_x - 1
                && faced_x <= crop_x + 1
                && faced_y >= crop_y - 1
                && faced_y <= crop_y + 1
                && *crop.get_stage() == GrowthStage::Fruiting
            {
                crops_to_harvest.push((crop, crop_transform));
            }
        }

        if let Some((crop, crop_transform)) = crops_to_harvest.first_mut() {
            // Harvesting logic (unchanged)
            let harvested_item_stack = crop.crop_type.harvested();
            if let Ok(inventory) = inventory_query.get_single_mut() {
                add_item_to_inventory(&mut commands, inventory, harvested_item_stack);
            }
            commands
                .spawn((
                    Sprite {
                        image: sprite_sheet.crops_texture.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: sprite_sheet.crops_layout.clone(),
                            index: 102,
                        }),
                        ..Default::default()
                    },
                    Transform::from_translation(Vec3::new(
                        crop_transform.translation.x,
                        crop_transform.translation.y + 10.0,
                        4.0,
                    ))
                    .with_scale(Vec3::splat(3.0)),
                ))
                .insert(HarvestedItemSprite {
                    target: player_transform.translation + Vec3::new(0.0, 64.0, 0.0),
                    speed: 100.0,
                    timer: 0.0,
                });
            crop.timer = 20.0;
            crop.set_stage(GrowthStage::Mature);
        } else {
            // Planting logic
            if let Ok(mut inventory) = inventory_query.get_single_mut() {
                if let Some(item) = &mut inventory.items[selected_slot.0] {
                    match item.item_type.category() {
                        ItemCategory::Crop => {
                            let mut can_plant = true;
                            for (_, crop_transform) in query_crops.iter() {
                                let crop_tile = world_to_tile(
                                    crop_transform.translation.truncate(),
                                    tilemap_transform,
                                    grid_size,
                                );

                                if (TilePos {
                                    x: crop_tile.x - 1, // 1 offset for center placement
                                    y: crop_tile.y,
                                }) == player_tile_faced
                                {
                                    // Use player_tile_faced for planting
                                    can_plant = false;
                                    break;
                                }
                            }
                            if can_plant {
                                let tile_local_pos = Vec3::new(
                                    player_tile_faced.x as f32 * grid_size.x + grid_size.x,
                                    player_tile_faced.y as f32 * grid_size.y + grid_size.y / 2.0,
                                    0.0,
                                );
                                let tile_world_pos =
                                    tilemap_transform.transform_point(tile_local_pos);
                                let is_empty = plant_crop(
                                    item,
                                    tile_storage,
                                    tile_texture_query,
                                    player_tile_faced,
                                    commands,
                                    sprite_sheet,
                                    tile_world_pos,
                                );
                                if is_empty {
                                    inventory.items[selected_slot.0] = None;
                                }
                            }
                        }
                        ItemCategory::Food => {}
                        ItemCategory::Weapon => {}
                        ItemCategory::Armor => {}
                        ItemCategory::Tool => {}
                    }
                }
            }
        }
    }
}

fn plant_crop(
    item: &mut ItemStack,
    tile_storage: &TileStorage,
    tile_texture_query: Query<&TileTextureIndex>,
    target_tile_pos: TilePos,
    mut commands: Commands,
    sprite_sheet: Res<SpriteSheetLayout>,
    tile_world_pos: Vec3,
) -> bool {
    let mut is_empty = false;
    if let Some(crop_to_plant) = item.item_type.plant() {
        if let Some(tile_entity) = tile_storage.get(&target_tile_pos) {
            if let Ok(tile_texture) = tile_texture_query.get(tile_entity) {
                if tile_texture.0 == 0 {
                    if item.count > 0 {
                        item.count -= 1;
                        is_empty = item.count == 0;
                        commands.send_event(InventoryUpdateEvent);

                        commands.spawn((
                            Sprite {
                                image: sprite_sheet.crops_texture.clone(),
                                texture_atlas: Some(TextureAtlas {
                                    layout: sprite_sheet.crops_layout.clone(),
                                    index: crop_to_plant.growth_stage_image(), // Start with the first sprite
                                }),
                                ..Default::default()
                            },
                            Transform {
                                translation: Vec3::new(tile_world_pos.x, tile_world_pos.y, -700.),
                                scale: Vec3::splat(SCALE), // Match tilemap scale
                                ..Default::default()
                            },
                            crop_to_plant,
                        ));
                        return is_empty;
                    }
                }
            }
        }
    }
    return is_empty;
}

// System to move harvested item sprites
fn move_harvested_items(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut HarvestedItemSprite)>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut harvested) in query.iter_mut() {
        let direction = (harvested.target - transform.translation).normalize_or_zero();
        let distance = transform.translation.distance(harvested.target);
        harvested.timer += time.delta_secs();
        if harvested.timer > 1.0 {
            commands.entity(entity).despawn();
        }
        if distance > 0.1 {
            let move_amount = direction * harvested.speed * time.delta_secs();
            transform.translation += move_amount;
        } else {
            commands.entity(entity).despawn();
        }
    }
}

// Helper function to convert world position to tile grid position
fn world_to_tile(
    world_pos: Vec2,
    tilemap_transform: &Transform,
    grid_size: &TilemapGridSize,
) -> TilePos {
    // Convert world position to tilemap local space
    let local_pos =
        (world_pos - tilemap_transform.translation.truncate()) / tilemap_transform.scale.truncate();
    // Convert local position to tile indices
    let tile_x = (local_pos.x / grid_size.x).floor() as i32;
    let tile_y = (local_pos.y / grid_size.y).floor() as i32;
    TilePos {
        x: tile_x.max(0) as u32,
        y: tile_y.max(0) as u32,
    }
}

fn grow_crops(
    time: Res<Time>,
    mut crop_query: Query<(&mut Crop, &mut Sprite, &mut Transform)>,
    sprite_sheet: Res<SpriteSheetLayout>,
) {
    for (mut crop, mut sprite, mut transform) in crop_query.iter_mut() {
        crop.timer += time.delta_secs();
        let old_stage = (*crop.get_stage()).clone();
        match old_stage {
            GrowthStage::Seed if crop.timer >= 5.0 => {
                crop.set_stage(GrowthStage::Sprout);
            }
            GrowthStage::Sprout if crop.timer >= 10.0 => {
                crop.set_stage(GrowthStage::Immature);
            }
            GrowthStage::Immature if crop.timer >= 15.0 => {
                crop.set_stage(GrowthStage::Mature);
            }
            GrowthStage::Mature if crop.timer >= 25.0 => {
                crop.set_stage(GrowthStage::Fruiting);
            }
            _ => {}
        }
        if *crop.get_stage() != old_stage {
            transform.translation.z = match crop.get_stage() {
                GrowthStage::Seed => 0.0,
                _ => 500.0 - transform.translation.y,
            };
        }
        sprite.texture_atlas = Some(TextureAtlas {
            layout: sprite_sheet.crops_layout.clone(),
            index: crop.growth_stage_image(),
        });
    }
}
