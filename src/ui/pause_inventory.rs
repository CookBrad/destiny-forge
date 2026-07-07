use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;

use crate::core::ProfileDirty;
use crate::forging::{can_craft_recipe, try_craft_iron_sword, IRON_SWORD_RECIPE};
use crate::items::{Inventory, MaterialId, INVENTORY_SLOT_COUNT};
use crate::player::Loadout;

#[derive(Component)]
pub struct PauseInventoryPanel;

#[derive(Component, Clone, Copy)]
pub struct InventorySlotLabel {
    pub index: usize,
}

#[derive(Component)]
pub struct ForgeCraftButton;

#[derive(Component)]
pub struct ForgeStatusLabel;

pub fn spawn_pause_inventory_panel(parent: &mut ChildBuilder<'_>, inventory: &Inventory) {
    parent
        .spawn((
            PauseInventoryPanel,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Inventory"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.86, 0.9, 0.95)),
            ));

            panel
                .spawn(Node {
                    width: Val::Px(360.0),
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(4.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|grid| {
                    for index in 0..INVENTORY_SLOT_COUNT {
                        let label = slot_text(inventory, index);
                        grid.spawn((
                            InventorySlotLabel { index },
                            Node {
                                width: Val::Px(54.0),
                                height: Val::Px(22.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.14, 0.95)),
                            BorderColor(Color::srgba(0.28, 0.3, 0.36, 0.9)),
                        ))
                        .with_children(|slot| {
                            slot.spawn((
                                Text::new(label),
                                TextFont {
                                    font_size: 9.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.78, 0.82, 0.9)),
                            ));
                        });
                    }
                });

            panel
                .spawn((
                    Button,
                    ForgeCraftButton,
                    Node {
                        min_width: Val::Px(180.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.16, 0.18, 0.24, 0.95)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(format!("Craft {}", IRON_SWORD_RECIPE.name)),
                        TextFont {
                            font_size: 15.0,
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
        });
}

fn slot_text(inventory: &Inventory, index: usize) -> String {
    let slot = &inventory.slots[index];
    match slot.material {
        Some(material) if slot.count > 0 => {
            format!("{} {}", abbrev(material), slot.count)
        }
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

pub fn sync_pause_inventory_display(
    inventory: Res<Inventory>,
    mut slots: Query<(&InventorySlotLabel, &Children)>,
    mut slot_texts: Query<&mut Text, Without<ForgeStatusLabel>>,
    mut status: Query<&mut Text, With<ForgeStatusLabel>>,
) {
    if !inventory.is_changed() {
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
    mut interactions: Query<&Interaction, (Changed<Interaction>, With<ForgeCraftButton>)>,
    mut inventory: ResMut<Inventory>,
    mut loadout: ResMut<Loadout>,
    mut profile_dirty: ResMut<ProfileDirty>,
    mut status: Query<&mut Text, With<ForgeStatusLabel>>,
) {
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