//! Homestead inventory/action bar (overworld) — mirrors combat skill bar layout.

use bevy::prelude::*;

use crate::farming::{HomesteadHotbar, HotbarEntry, HOTBAR_SLOT_COUNT};
use crate::items::Inventory;

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

const SLOT_WIDTH: f32 = 54.0;
const SLOT_HEIGHT: f32 = 68.0;
const SLOT_GAP: f32 = 6.0;
const BAR_BOTTOM: f32 = 14.0;

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
                border: UiRect::all(Val::Px(if selected { 2.0 } else { 2.0 })),
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

pub fn cleanup_hotbar(mut commands: Commands, roots: Query<Entity, With<HotbarHud>>) {
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

pub fn handle_hotbar_slot_clicks(
    mut hotbar: ResMut<HomesteadHotbar>,
    interactions: Query<(&Interaction, &HotbarSlot), Changed<Interaction>>,
) {
    for (interaction, slot) in &interactions {
        if *interaction == Interaction::Pressed {
            hotbar.select(slot.index);
        }
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
                if n > 0 {
                    n.to_string()
                } else {
                    "0".to_string()
                }
            }
            HotbarEntry::Tool(_) => String::new(),
            HotbarEntry::Empty => String::new(),
        };
    }
}
