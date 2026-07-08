//! Homestead hotbar — empty slots, drag items from inventory; selected = action.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::farming::{HomesteadHotbar, HotbarEntry, HOTBAR_SLOT_COUNT};
use crate::items::{Inventory, MaterialId};

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
pub struct HotbarSlotLabel {
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

/// Drag payload from backpack → hotbar.
#[derive(Resource, Default)]
pub struct InventoryHotbarDrag {
    pub material: Option<MaterialId>,
    pub ghost: Option<Entity>,
}

const SLOT_WIDTH: f32 = 54.0;
const SLOT_HEIGHT: f32 = 68.0;
const SLOT_GAP: f32 = 6.0;
const BAR_BOTTOM: f32 = 14.0;
const GHOST_SIZE: f32 = 40.0;

pub fn setup_hotbar(mut commands: Commands, hotbar: Res<HomesteadHotbar>) {
    spawn_hotbar(&mut commands, &hotbar);
}

pub fn spawn_hotbar(commands: &mut Commands, hotbar: &HomesteadHotbar) {
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
                    spawn_hotbar_slot(row, index, hotbar.slots[index], hotbar.selected == index);
                }
            });
        });
}

fn spawn_hotbar_slot(
    parent: &mut ChildBuilder<'_>,
    index: usize,
    entry: HotbarEntry,
    selected: bool,
) {
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
                Node {
                    width: Val::Px(30.0),
                    height: Val::Px(30.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(entry.icon_color()),
                BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
            ))
            .with_children(|icon| {
                icon.spawn((
                    HotbarSlotLabel { slot_index: index },
                    Text::new(entry.short_label()),
                    TextFont {
                        font_size: 9.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.96, 0.9)),
                ));
            });

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

        if let Some(material) = drag.material.take() {
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
    window: Query<&Window, With<PrimaryWindow>>,
    mut ghosts: Query<&mut Node, With<HotbarDragGhost>>,
    mut labels: Query<&mut Text, With<HotbarDragGhostLabel>>,
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
                BackgroundColor(HotbarEntry::Item(material).icon_color()),
                BorderColor(Color::srgb(0.95, 0.85, 0.4)),
                GlobalZIndex(250),
            ))
            .with_children(|g| {
                g.spawn((
                    HotbarDragGhostLabel,
                    Text::new(material.short_label()),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
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
    if let Ok(mut text) = labels.get_single_mut() {
        let label = material.short_label();
        if text.as_str() != label {
            text.0 = label.to_string();
        }
    }
}

fn clear_drag(commands: &mut Commands, drag: &mut InventoryHotbarDrag) {
    drag.material = None;
    if let Some(entity) = drag.ghost.take() {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn sync_hotbar_ui(
    hotbar: Res<HomesteadHotbar>,
    inventory: Res<Inventory>,
    mut slots: Query<(&HotbarSlot, &mut BorderColor, &mut BackgroundColor)>,
    mut icons: Query<(&HotbarSlotIcon, &mut BackgroundColor), Without<HotbarSlot>>,
    mut labels: Query<(&HotbarSlotLabel, &mut Text)>,
    mut counts: Query<(&HotbarSlotCount, &mut Text), Without<HotbarSlotLabel>>,
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

    for (icon, mut bg) in &mut icons {
        let entry = hotbar.slots[icon.slot_index];
        bg.0 = entry.icon_color();
    }

    for (label, mut text) in &mut labels {
        let entry = hotbar.slots[label.slot_index];
        text.0 = entry.short_label().to_string();
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
