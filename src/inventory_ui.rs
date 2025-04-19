use crate::player::{Inventory, Player};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::SpriteSheetLayout;
use crate::items::{Item, ItemStack, ItemType, seeds::corn::CornSeed};

// Components
#[derive(Component)]
pub struct InventoryBar;

#[derive(Component)]
pub struct SlotTag;

#[derive(Component)]
pub struct Draggable;

// Resources
#[derive(Resource, Default)]
pub struct DragState {
    pub dragging: Option<Dragging>,
}

#[derive(Component)]
pub struct Dragging {
    original_entity: Entity,
    original_slot: usize,
    temp_entity: Option<Entity>,
}

// Events
#[derive(Resource, Default, Event)]
pub struct InventoryUpdateEvent;

// Plugin
pub struct InventoryUiPlugin;

#[derive(Resource)]
pub struct SelectedSlot(pub usize);
#[derive(Component)]
pub struct SlotBorderIndex(pub usize);

impl Plugin for InventoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .add_event::<InventoryUpdateEvent>()
            .add_systems(Startup, setup_inventory_bar)
            .add_systems(Update, (handle_drag_start, handle_drag, handle_drop))
            .add_systems(
                Update,
                update_inventory_bar.run_if(on_event::<InventoryUpdateEvent>),
            );
    }
}

// Helper Functions
#[derive(Bundle)]
pub struct InventorySlotBundle {
    node: Node,
    slot_tag: SlotTag,
    draggable: Draggable,
    interaction: Interaction,
}

fn spawn_slot(
    parent: &mut ChildBuilder,
    // asset_server: &AssetServer,
    sprite_sheet: &Res<SpriteSheetLayout>,
    item_option: Option<&ItemStack>,
    index: usize,
) {
    let bundle = InventorySlotBundle {
        node: Node {
            width: Val::Px(50.0),
            height: Val::Px(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        slot_tag: SlotTag,
        draggable: Draggable,
        interaction: Interaction::default(),
    };

    if let Some(item) = item_option {
        let display_info = item.item_type.as_item().display_info();
        // let image_handle = asset_server.load(display_info.image_path);
        let text_content = format!("{} {}", display_info.name, item.count);

        parent
            .spawn((
                ImageNode {
                    color: Color::WHITE,
                    image: sprite_sheet.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: sprite_sheet.layout.clone(),
                        index: display_info.image_path, // Start with the first sprite
                    }),
                    ..default()
                },
                Text::new(text_content),
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
                    SlotBorderIndex(index),
                ));
            });
    } else {
        parent
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
                    SlotBorderIndex(index),
                ));
            });
    };
}

pub fn setup_inventory_bar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sprite_sheet: Res<SpriteSheetLayout>,
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
        .with_children(|inventory_bar| {
            if let Ok(inventory) = inventory_query.get_single() {
                for (index, item_option) in inventory.items.iter().enumerate() {
                    spawn_slot(inventory_bar, &sprite_sheet, item_option.as_ref(), index);
                }
            }
        });
    commands.insert_resource(SelectedSlot(0));
}

fn create_drag_entity(
    commands: &mut Commands,
    item: &ItemStack,
    sprite_sheet: &Res<SpriteSheetLayout>,
) -> Entity {
    let display_info = item.item_type.as_item().display_info();
    let text_content = format!("{} {}", display_info.name, item.count);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(50.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ImageNode {
                color: Color::WHITE,
                image: sprite_sheet.texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: sprite_sheet.layout.clone(),
                    index: display_info.image_path, // Start with the first sprite
                }),
                ..default()
            },
            Text::new(text_content),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor::default(),
            TextLayout::default(),
        ))
        .id()
}

pub fn handle_drag_start(
    mut commands: Commands,
    mut drag_state: ResMut<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    query: Query<(Entity, &Interaction, &Name), With<Draggable>>,
    inventory_query: Query<&Inventory, With<Player>>,
    sprite_sheet: Res<SpriteSheetLayout>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        for (entity, interaction, name) in query.iter() {
            if *interaction == Interaction::Pressed {
                if let Some(slot_num) = name.as_str().strip_prefix("Slot_") {
                    if let Ok(slot_index) = slot_num.parse::<usize>() {
                        if let Ok(inventory) = inventory_query.get_single() {
                            if let Some(item) = &inventory.items[slot_index] {
                                let temp_entity =
                                    create_drag_entity(&mut commands, item, &sprite_sheet);

                                // Update DragState
                                drag_state.dragging = Some(Dragging {
                                    original_entity: entity,
                                    original_slot: slot_index,
                                    temp_entity: Some(temp_entity),
                                });
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_drag(
    windows: Query<&Window>,
    drag_state: Res<DragState>,
    mut query: Query<&mut Node>,
) {
    if let Some(dragging) = &drag_state.dragging {
        if let Some(temp_entity) = dragging.temp_entity {
            if let Ok(mut node) = query.get_mut(temp_entity) {
                let window = windows.single();
                if let Some(cursor_pos) = window.cursor_position() {
                    node.left = Val::Px(cursor_pos.x);
                    node.top = Val::Px(cursor_pos.y);
                }
            }
        }
    }
}

pub fn handle_drop(
    mut commands: Commands,
    mut drag_state: ResMut<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut inventory_query: Query<&mut Inventory, With<Player>>,
    query: Query<(Entity, &Interaction, &Name), With<Draggable>>,
) {
    if mouse.just_released(MouseButton::Left) {
        if let Some(dragging) = drag_state.dragging.take() {
            let mut target_slot = None;
            for (entity, interaction, name) in query.iter() {
                if *interaction == Interaction::Hovered && entity != dragging.original_entity {
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
                    if dragging.original_slot != target {
                        let original_item = inventory.items[dragging.original_slot].take();
                        let target_item = inventory.items[target].take();
                        inventory.items[target] = original_item;
                        inventory.items[dragging.original_slot] = target_item;
                    }
                }

                // Despawn the temporary entity
                if let Some(temp_entity) = dragging.temp_entity {
                    commands.entity(temp_entity).despawn();
                }

                // Trigger inventory update
                commands.send_event(InventoryUpdateEvent);
            }
        }
    }
}

fn update_slot(
    commands: &mut Commands,
    slot_entity: Entity,
    item_option: Option<&ItemStack>,
    sprite_sheet: &Res<SpriteSheetLayout>,
) {
    if let Some(item) = item_option {
        let display_info = item.item_type.as_item().display_info();
        let text_content = format!("{} {}", display_info.name, item.count);

        commands.entity(slot_entity).insert((
            ImageNode {
                color: Color::WHITE,
                image: sprite_sheet.texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: sprite_sheet.layout.clone(),
                    index: display_info.image_path, // Start with the first sprite
                }),
                ..default()
            },
            Text::new(text_content),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor::default(),
            TextLayout::default(),
        ));
    } else {
        commands.entity(slot_entity).insert(Text::new(""));
        commands.entity(slot_entity).remove::<ImageNode>();
    }
}

pub fn update_inventory_bar(
    mut commands: Commands,
    inventory_query: Query<&Inventory, With<Player>>,
    bar_query: Query<&Children, With<InventoryBar>>,
    slot_query: Query<Entity, With<SlotTag>>,
    sprite_sheet: Res<SpriteSheetLayout>,
) {
    if let Ok(inventory) = inventory_query.get_single() {
        if let Ok(children) = bar_query.get_single() {
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

            for (index, &slot_entity) in slot_entities.iter().enumerate() {
                if index < inventory.items.len() {
                    let item_option = inventory.items[index].as_ref();
                    update_slot(&mut commands, slot_entity, item_option, &sprite_sheet);
                }
            }
        }
    }
}

pub fn add_item_to_inventory(
    commands: &mut Commands,
    mut inventory: Mut<'_, Inventory>,
    mut item_stack: ItemStack,
) {
    let mut found = false;
    for current_stack in &mut inventory.items {
        if let Some(inventory_stack) = current_stack {
            if std::mem::discriminant(&inventory_stack.item_type)
                == std::mem::discriminant(&item_stack.item_type)
            {
                if inventory_stack.count + item_stack.count <= inventory_stack.max_count {
                    inventory_stack.count += item_stack.count;
                    println!("{}", inventory_stack.count);
                    found = true;
                    break;
                }
                if inventory_stack.count < inventory_stack.max_count {
                    item_stack.count =
                        item_stack.count - (inventory_stack.max_count - inventory_stack.count);
                    inventory_stack.count = inventory_stack.max_count;
                }
            }
        }
    }

    if !found {
        if let Some(k) = inventory.items.iter().position(|x| x.is_none()) {
            inventory.items[k] = Some(item_stack);
        } else {
            println!("Inventory full");
        }
    }

    commands.send_event(InventoryUpdateEvent);
}

pub fn add_corn_to_inventory(mut inventory_query: Query<&mut Inventory, With<Player>>) {
    if let Ok(mut inventory) = inventory_query.get_single_mut() {
        let mut stack_count = 0;
        for _ in 0..66 {
            let mut found = false;
            let item_type_to_add = ItemType::CornSeed(CornSeed);
            for stack in &mut inventory.items {
                if let Some(stack) = stack {
                    if std::mem::discriminant(&stack.item_type)
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
                    ItemType::CornSeed(corn) => corn.stack_size(),
                };
                let new_stack = ItemStack {
                    item_type: item_type_to_add,
                    count: 1,
                    max_count,
                };
                inventory.items[stack_count] = Some(new_stack);
                stack_count += 1;
            }
        }
    }
}

pub fn handle_inventory_scroll(
    mut selected_slot: ResMut<SelectedSlot>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    slot_query: Query<&Interaction, With<SlotTag>>,
    slots: Query<(), With<SlotTag>>,
) {
    let num_slots = slots.iter().count();
    for event in mouse_wheel_events.read() {
        if slot_query
            .iter()
            .any(|&interaction| interaction == Interaction::Hovered)
        {
            let scroll_direction = event.y;
            if scroll_direction > 0.0 {
                selected_slot.0 = (selected_slot.0 + 1) % num_slots;
            } else if scroll_direction < 0.0 {
                selected_slot.0 = (selected_slot.0 + num_slots - 1) % num_slots;
            }
        }
    }
}

pub fn update_slot_borders(
    selected_slot: Res<SelectedSlot>,
    mut query: Query<(&SlotBorderIndex, &mut BorderColor)>,
) {
    for (slot_index, mut border_color) in query.iter_mut() {
        if slot_index.0 == selected_slot.0 {
            *border_color = Color::Srgba(Srgba {
                red: 10.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            })
            .into();
        } else {
            *border_color = Color::WHITE.into();
        }
    }
}
