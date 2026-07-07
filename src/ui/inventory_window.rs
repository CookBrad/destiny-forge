use bevy::prelude::*;

use crate::core::ProfileDirty;
use crate::forging::{can_craft_recipe, try_craft_iron_sword, IRON_SWORD_RECIPE};
use crate::items::{Inventory, MaterialId, INVENTORY_SLOT_COUNT};
use crate::player::Loadout;

#[derive(Resource, Default, Debug)]
pub struct InventoryWindowOpen(pub bool);

pub fn inventory_closed(open: Res<InventoryWindowOpen>) -> bool {
    !open.0
}

pub fn inventory_window_open(open: Res<InventoryWindowOpen>) -> bool {
    open.0
}

#[derive(Component)]
pub struct InventoryWindow;

#[derive(Component, Clone, Copy)]
pub struct InventorySlotLabel {
    pub index: usize,
}

#[derive(Component)]
pub struct ForgeCraftButton;

#[derive(Component)]
pub struct ForgeStatusLabel;

pub fn spawn_inventory_window(commands: &mut Commands, inventory: &Inventory) {
    commands
        .spawn((
            InventoryWindow,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.06, 0.55)),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(14.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.09, 0.14, 0.98)),
                    BorderColor(Color::srgba(0.38, 0.42, 0.52, 0.95)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Inventory"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.94, 0.98)),
                    ));

                    panel
                        .spawn(Node {
                            width: Val::Px(420.0),
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(6.0),
                            row_gap: Val::Px(6.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .with_children(|grid| {
                            for index in 0..INVENTORY_SLOT_COUNT {
                                let label = slot_text(inventory, index);
                                grid.spawn((
                                    InventorySlotLabel { index },
                                    Node {
                                        width: Val::Px(62.0),
                                        height: Val::Px(28.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.12, 0.13, 0.18, 0.98)),
                                    BorderColor(Color::srgba(0.32, 0.35, 0.42, 0.95)),
                                ))
                                .with_children(|slot| {
                                    slot.spawn((
                                        Text::new(label),
                                        TextFont {
                                            font_size: 11.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.8, 0.84, 0.92)),
                                    ));
                                });
                            }
                        });

                    panel
                        .spawn((
                            Button,
                            ForgeCraftButton,
                            Node {
                                min_width: Val::Px(220.0),
                                height: Val::Px(34.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(14.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.18, 0.2, 0.28, 0.98)),
                            BorderColor(Color::srgba(0.42, 0.46, 0.56, 0.9)),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new(format!("Craft {}", IRON_SWORD_RECIPE.name)),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.92, 0.94, 0.98)),
                            ));
                        });

                    panel.spawn((
                        ForgeStatusLabel,
                        Text::new(forge_status(inventory)),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.68, 0.74, 0.82)),
                    ));

                    panel.spawn((
                        Text::new("I or Esc — Close"),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.58, 0.62, 0.7)),
                    ));
                });
        });
}

pub fn cleanup_inventory_window(
    mut commands: Commands,
    mut open: ResMut<InventoryWindowOpen>,
    windows: Query<Entity, With<InventoryWindow>>,
) {
    open.0 = false;
    for entity in &windows {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn toggle_inventory_window(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<InventoryWindowOpen>,
    mut commands: Commands,
    inventory: Res<Inventory>,
    windows: Query<Entity, With<InventoryWindow>>,
    dungeon: Res<State<crate::core::DungeonPlayState>>,
    mut time: ResMut<Time<Virtual>>,
) {
    let close = open.0 && keyboard.just_pressed(KeyCode::Escape);
    let toggle = keyboard.just_pressed(KeyCode::KeyI);

    if !close && !toggle {
        return;
    }

    if close {
        open.0 = false;
    } else {
        open.0 = !open.0;
    }

    if open.0 {
        if windows.is_empty() {
            spawn_inventory_window(&mut commands, &inventory);
        }
        time.pause();
    } else {
        for entity in &windows {
            commands.entity(entity).try_despawn_recursive();
        }
        if !matches!(
            dungeon.get(),
            crate::core::DungeonPlayState::Paused
                | crate::core::DungeonPlayState::Dying
                | crate::core::DungeonPlayState::Dead
        ) {
            time.unpause();
        }
    }
}

fn slot_text(inventory: &Inventory, index: usize) -> String {
    let slot = &inventory.slots[index];
    match slot.material {
        Some(material) if slot.count > 0 => format!("{} {}", abbrev(material), slot.count),
        _ => "-".to_string(),
    }
}

fn abbrev(material: MaterialId) -> &'static str {
    match material {
        MaterialId::SlimeGel => "Gel",
        MaterialId::SlimeCore => "Core",
        MaterialId::LeatherWing => "Wing",
        MaterialId::Fang => "Fang",
        MaterialId::IronScrap => "Iron",
    }
}

fn forge_status(inventory: &Inventory) -> String {
    if can_craft_recipe(inventory, &IRON_SWORD_RECIPE) {
        format!("{} ready to craft (5 Gel, 3 Iron)", IRON_SWORD_RECIPE.name)
    } else {
        format!("Need 5 Slime Gel + 3 Iron Scrap for {}", IRON_SWORD_RECIPE.name)
    }
}

pub fn sync_inventory_display(
    inventory: Res<Inventory>,
    open: Res<InventoryWindowOpen>,
    mut slots: Query<(&InventorySlotLabel, &Children)>,
    mut slot_texts: Query<&mut Text, Without<ForgeStatusLabel>>,
    mut status: Query<&mut Text, With<ForgeStatusLabel>>,
) {
    if !open.0 || !inventory.is_changed() {
        return;
    }

    for (slot, children) in &mut slots {
        let label = slot_text(&inventory, slot.index);
        for child in children.iter() {
            if let Ok(mut text) = slot_texts.get_mut(*child) {
                text.0 = label.clone();
            }
        }
    }

    if let Ok(mut text) = status.get_single_mut() {
        text.0 = forge_status(&inventory);
    }
}

pub fn handle_forge_craft_input(
    open: Res<InventoryWindowOpen>,
    mut interactions: Query<&Interaction, (Changed<Interaction>, With<ForgeCraftButton>)>,
    mut inventory: ResMut<Inventory>,
    mut loadout: ResMut<Loadout>,
    mut profile_dirty: ResMut<ProfileDirty>,
    mut status: Query<&mut Text, With<ForgeStatusLabel>>,
) {
    if !open.0 {
        return;
    }

    for interaction in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let crafted = try_craft_iron_sword(&mut inventory, &mut loadout);
        if let Ok(mut text) = status.get_single_mut() {
            text.0 = if crafted {
                profile_dirty.mark();
                "Crafted Iron Sword — equip on next dungeon run.".to_string()
            } else {
                forge_status(&inventory)
            };
        }
    }
}