//! Homestead hotbar — empty slots, drag items from inventory; selected = action.

use bevy::prelude::*;
use bevy::ui::widget::{ImageNode, NodeImageMode};
use bevy::window::PrimaryWindow;

use crate::farming::{HomesteadHotbar, HotbarEntry, HOTBAR_SLOT_COUNT};
use crate::items::{Inventory, ItemIconAssets, MaterialId};

#[derive(Component)]
pub struct HotbarHud;

#[derive(Component, Clone, Copy)]
pub struct HotbarSlot {
    pub index: usize,
}

#[derive(Component, Clone, Copy)]
pub struct HotbarSlotIcon {
    pub slot_index: usize,
}

#[derive(Component, Clone, Copy)]
pub struct HotbarSlotKey {
    pub slot_index: usize,
}

#[derive(Component, Clone, Copy)]
pub struct HotbarSlotCount {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct HotbarDragGhost;

#[derive(Component)]
pub struct HotbarDragGhostLabel;

/// Drag payload from backpack → inventory slot (reorganize) or hotbar.
#[derive(Resource, Default)]
pub struct InventoryHotbarDrag {
    /// Source backpack slot index.
    pub from_slot: Option<usize>,
    pub material: Option<MaterialId>,
    pub ghost: Option<Entity>,
}

const SLOT_WIDTH: f32 = 54.0;
const SLOT_HEIGHT: f32 = 68.0;
const SLOT_GAP: f32 = 6.0;
const BAR_BOTTOM: f32 = 14.0;
const GHOST_SIZE: f32 = 40.0;
const ICON_SIZE: f32 = 30.0;

pub fn setup_hotbar(
    mut commands: Commands,
    hotbar: Res<HomesteadHotbar>,
    icons: Res<ItemIconAssets>,
) {
    spawn_hotbar(&mut commands, &hotbar, &icons);
}

pub fn spawn_hotbar(commands: &mut Commands, hotbar: &HomesteadHotbar, icons: &ItemIconAssets) {
    commands
        .spawn((
            HotbarHud,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(BAR_BOTTOM),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|bar| {
            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(SLOT_GAP),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                for index in 0..HOTBAR_SLOT_COUNT {
                    spawn_hotbar_slot(
                        row,
                        index,
                        hotbar.slots[index],
                        hotbar.selected == index,
                        icons,
                    );
                }
            });
        });
}

fn spawn_hotbar_slot(
    parent: &mut ChildBuilder<'_>,
    index: usize,
    entry: HotbarEntry,
    selected: bool,
    icons: &ItemIconAssets,
) {
    let (image, visible) = entry_image(entry, icons);

    parent
        .spawn((
            HotbarSlot { index },
            Button,
            Node {
                width: Val::Px(SLOT_WIDTH),
                height: Val::Px(SLOT_HEIGHT),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(3.0)),
                border: UiRect::all(Val::Px(2.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.09, 0.08, 0.92)),
            BorderColor(if selected {
                Color::srgb(0.95, 0.72, 0.28)
            } else {
                Color::srgba(0.35, 0.42, 0.32, 0.9)
            }),
        ))
        .with_children(|slot| {
            slot.spawn((
                HotbarSlotKey { slot_index: index },
                Text::new((index + 1).to_string()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.8, 0.7)),
            ));

            slot.spawn((
                HotbarSlotIcon { slot_index: index },
                ImageNode {
                    image,
                    image_mode: NodeImageMode::Stretch,
                    color: if visible { Color::WHITE } else { Color::NONE },
                    ..default()
                },
                Node {
                    width: Val::Px(ICON_SIZE),
                    height: Val::Px(ICON_SIZE),
                    ..default()
                },
            ));

            slot.spawn((
                HotbarSlotCount { slot_index: index },
                Text::new(""),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.88, 0.8)),
            ));
        });
}

fn entry_image(entry: HotbarEntry, icons: &ItemIconAssets) -> (Handle<Image>, bool) {
    match entry {
        HotbarEntry::Item(material) => (icons.handle_for(material), true),
        HotbarEntry::Empty => (icons.slime_gel.clone(), false),
    }
}

pub fn cleanup_hotbar(
    mut commands: Commands,
    roots: Query<Entity, With<HotbarHud>>,
    mut drag: ResMut<InventoryHotbarDrag>,
) {
    clear_drag(&mut commands, &mut drag);
    for entity in &roots {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn select_hotbar_slot_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut hotbar: ResMut<HomesteadHotbar>,
) {
    let key = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(2)
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        Some(3)
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        Some(4)
    } else {
        None
    };
    if let Some(index) = key {
        hotbar.select(index);
    }
}

/// Click empty/assigned slot to select (highlighted = active action).
/// Dropping a dragged inventory item assigns it.
/// Right-click clears a slot.
pub fn handle_hotbar_slot_clicks(
    mouse: Res<ButtonInput<MouseButton>>,
    mut hotbar: ResMut<HomesteadHotbar>,
    mut drag: ResMut<InventoryHotbarDrag>,
    mut commands: Commands,
    interactions: Query<(&Interaction, &HotbarSlot), Changed<Interaction>>,
) {
    for (interaction, slot) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if mouse.pressed(MouseButton::Right) {
            hotbar.clear_slot(slot.index);
            clear_drag(&mut commands, &mut drag);
            continue;
        }

        if let Some(material) = drag.material {
            hotbar.assign(slot.index, material);
            clear_drag(&mut commands, &mut drag);
        } else {
            hotbar.select(slot.index);
        }
    }
}

pub fn cancel_hotbar_drag_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<InventoryHotbarDrag>,
    mut commands: Commands,
) {
    if drag.material.is_none() {
        return;
    }
    if keyboard.just_pressed(KeyCode::Escape) || mouse.just_released(MouseButton::Right) {
        clear_drag(&mut commands, &mut drag);
    }
}

pub fn update_hotbar_drag_ghost(
    mut commands: Commands,
    mut drag: ResMut<InventoryHotbarDrag>,
    icons: Res<ItemIconAssets>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut ghosts: Query<&mut Node, With<HotbarDragGhost>>,
    mut ghost_images: Query<&mut ImageNode, With<HotbarDragGhostLabel>>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };

    let Some(material) = drag.material else {
        clear_drag(&mut commands, &mut drag);
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let left = cursor.x - GHOST_SIZE * 0.5;
    let top = cursor.y - GHOST_SIZE * 0.5;

    if drag.ghost.is_none() {
        let ghost = commands
            .spawn((
                HotbarDragGhost,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(GHOST_SIZE),
                    height: Val::Px(GHOST_SIZE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    left: Val::Px(left),
                    top: Val::Px(top),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.85)),
                BorderColor(Color::srgb(0.95, 0.85, 0.4)),
                GlobalZIndex(250),
            ))
            .with_children(|g| {
                g.spawn((
                    HotbarDragGhostLabel,
                    ImageNode {
                        image: icons.handle_for(material),
                        image_mode: NodeImageMode::Stretch,
                        color: Color::WHITE,
                        ..default()
                    },
                    Node {
                        width: Val::Px(ICON_SIZE),
                        height: Val::Px(ICON_SIZE),
                        ..default()
                    },
                ));
            })
            .id();
        drag.ghost = Some(ghost);
        return;
    }

    if let Ok(mut node) = ghosts.get_single_mut() {
        node.left = Val::Px(left);
        node.top = Val::Px(top);
    }
    if let Ok(mut image) = ghost_images.get_single_mut() {
        image.image = icons.handle_for(material);
    }
}

pub fn clear_inventory_drag(commands: &mut Commands, drag: &mut InventoryHotbarDrag) {
    drag.from_slot = None;
    drag.material = None;
    if let Some(entity) = drag.ghost.take() {
        commands.entity(entity).try_despawn_recursive();
    }
}

fn clear_drag(commands: &mut Commands, drag: &mut InventoryHotbarDrag) {
    clear_inventory_drag(commands, drag);
}

pub fn sync_hotbar_ui(
    hotbar: Res<HomesteadHotbar>,
    inventory: Res<Inventory>,
    icons: Res<ItemIconAssets>,
    mut slots: Query<(&HotbarSlot, &mut BorderColor, &mut BackgroundColor)>,
    mut icon_images: Query<(&HotbarSlotIcon, &mut ImageNode)>,
    mut counts: Query<(&HotbarSlotCount, &mut Text)>,
) {
    if !hotbar.is_changed() && !inventory.is_changed() {
        return;
    }

    for (slot, mut border, mut bg) in &mut slots {
        let selected = slot.index == hotbar.selected;
        border.0 = if selected {
            Color::srgb(0.95, 0.72, 0.28)
        } else {
            Color::srgba(0.35, 0.42, 0.32, 0.9)
        };
        bg.0 = if selected {
            Color::srgba(0.14, 0.14, 0.1, 0.95)
        } else {
            Color::srgba(0.08, 0.09, 0.08, 0.92)
        };
    }

    for (icon, mut image_node) in &mut icon_images {
        let entry = hotbar.slots[icon.slot_index];
        let (handle, visible) = entry_image(entry, &icons);
        image_node.image = handle;
        image_node.color = if visible { Color::WHITE } else { Color::NONE };
    }

    for (count, mut text) in &mut counts {
        let entry = hotbar.slots[count.slot_index];
        text.0 = match entry {
            HotbarEntry::Item(material) => {
                let n = inventory.count(material);
                if material.is_tool() {
                    String::new()
                } else if n > 0 {
                    n.to_string()
                } else {
                    "0".to_string()
                }
            }
            HotbarEntry::Empty => String::new(),
        };
    }
}
