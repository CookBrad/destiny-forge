use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;

use crate::core::{DungeonPlayState, GameState, ProfileDirty};
use crate::forging::{can_craft_recipe, try_craft_iron_sword, IRON_SWORD_RECIPE};
use crate::items::{Inventory, MaterialId, INVENTORY_SLOT_COUNT};
use crate::player::Loadout;

const GRID_COLUMNS: usize = 4;
const SLOT_SIZE: f32 = 52.0;
const SLOT_GAP: f32 = 3.0;
const PANEL_PADDING: f32 = 10.0;

const FRAME_BG: Color = Color::srgb(0.14, 0.09, 0.06);
const FRAME_BORDER: Color = Color::srgb(0.55, 0.4, 0.16);
const HEADER_BG: Color = Color::srgb(0.1, 0.06, 0.04);
const SLOT_BG: Color = Color::srgb(0.08, 0.05, 0.04);
const SLOT_BORDER: Color = Color::srgb(0.24, 0.17, 0.11);
const SLOT_SELECTED: Color = Color::srgb(0.95, 0.48, 0.1);
const CLOSE_BUTTON: Color = Color::srgb(0.72, 0.14, 0.1);
const FOOTER_BG: Color = Color::srgb(0.09, 0.06, 0.04);

#[derive(Resource, Default, Debug)]
pub struct InventoryWindowOpen(pub bool);

#[derive(Resource, Default)]
pub struct InventorySelectedSlot(pub usize);

pub fn inventory_closed(open: Res<InventoryWindowOpen>) -> bool {
    !open.0
}

pub fn inventory_window_open(open: Res<InventoryWindowOpen>) -> bool {
    open.0
}

#[derive(Component)]
pub struct InventoryWindow;

#[derive(Component, Clone, Copy)]
pub struct InventorySlot {
    pub index: usize,
}

#[derive(Component)]
pub struct InventorySlotIcon;

#[derive(Component)]
pub struct InventorySlotStack;

#[derive(Component)]
pub struct InventorySlotStackText;

#[derive(Component)]
pub struct InventoryCloseButton;

#[derive(Component)]
pub struct ForgeCraftButton;

#[derive(Component)]
pub struct ForgeStatusLabel;

pub fn spawn_inventory_window(commands: &mut Commands, inventory: &Inventory) {
    let grid_width = GRID_COLUMNS as f32 * SLOT_SIZE + (GRID_COLUMNS as f32 - 1.0) * SLOT_GAP;
    let panel_width = grid_width + PANEL_PADDING * 2.0;

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(panel_width + 8.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(FRAME_BORDER),
                ))
                .with_children(|frame| {
                    frame
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                width: Val::Px(panel_width),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(FRAME_BG),
                            BorderColor(Color::srgb(0.32, 0.22, 0.12)),
                        ))
                        .with_children(|panel| {
                            spawn_header(panel);
                            spawn_slot_grid(panel, inventory, grid_width);
                            spawn_forge_row(panel, inventory);
                            spawn_currency_footer(panel);
                        });
                });
        });
}

fn spawn_header(parent: &mut ChildBuilder<'_>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(34.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::horizontal(Val::Px(8.0)),
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(HEADER_BG),
            BorderColor(Color::srgb(0.28, 0.18, 0.1)),
        ))
        .with_children(|header| {
            header
                .spawn((
                    Node {
                        width: Val::Px(22.0),
                        height: Val::Px(22.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.13, 0.08)),
                    BorderColor(Color::srgb(0.62, 0.46, 0.18)),
                ))
                .with_children(|emblem| {
                    emblem.spawn((
                        Text::new("◆"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.82, 0.66, 0.28)),
                    ));
                });

            header.spawn((
                Text::new("Backpack"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.94, 0.9, 0.82)),
            ));

            header
                .spawn((
                    Button,
                    InventoryCloseButton,
                    Node {
                        width: Val::Px(22.0),
                        height: Val::Px(22.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(CLOSE_BUTTON),
                    BorderColor(Color::srgb(0.42, 0.08, 0.06)),
                ))
                .with_children(|close| {
                    close.spawn((
                        Text::new("X"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.98, 0.95, 0.92)),
                    ));
                });
        });
}

fn spawn_slot_grid(parent: &mut ChildBuilder<'_>, inventory: &Inventory, grid_width: f32) {
    parent
        .spawn(Node {
            width: Val::Px(grid_width),
            margin: UiRect::all(Val::Px(PANEL_PADDING)),
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(SLOT_GAP),
            row_gap: Val::Px(SLOT_GAP),
            ..default()
        })
        .with_children(|grid| {
            for index in 0..INVENTORY_SLOT_COUNT {
                spawn_slot(grid, inventory, index);
            }
        });
}

fn spawn_slot(parent: &mut ChildBuilder<'_>, inventory: &Inventory, index: usize) {
    let (icon_color, icon_label, stack) = slot_visuals(inventory, index);
    let selected = index == 0;

    parent
        .spawn((
            Button,
            InventorySlot { index },
            Node {
                width: Val::Px(SLOT_SIZE),
                height: Val::Px(SLOT_SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(if selected { 2.0 } else { 1.0 })),
                ..default()
            },
            BackgroundColor(SLOT_BG),
            BorderColor(if selected {
                SLOT_SELECTED
            } else {
                SLOT_BORDER
            }),
        ))
        .with_children(|slot| {
            slot.spawn((
                InventorySlotIcon,
                Node {
                    width: Val::Px(38.0),
                    height: Val::Px(38.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(icon_color),
                BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
            ))
            .with_children(|icon| {
                icon.spawn((
                    Text::new(icon_label),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.95, 0.98)),
                ));
            });

            slot.spawn((
                InventorySlotStack,
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(3.0),
                    bottom: Val::Px(1.0),
                    ..default()
                },
            ))
            .with_children(|stack_node| {
                stack_node.spawn((
                    InventorySlotStackText,
                    Text::new(stack),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.98, 0.98, 1.0)),
                ));
            });
        });
}

fn spawn_forge_row(parent: &mut ChildBuilder<'_>, inventory: &Inventory) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(4.0),
            padding: UiRect::horizontal(Val::Px(PANEL_PADDING)),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Button,
                ForgeCraftButton,
                Node {
                    min_width: Val::Px(140.0),
                    height: Val::Px(24.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.1, 0.08)),
                BorderColor(Color::srgb(0.38, 0.3, 0.18)),
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new(format!("Craft {}", IRON_SWORD_RECIPE.name)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.86, 0.82, 0.72)),
                ));
            });

            row.spawn((
                ForgeStatusLabel,
                Text::new(forge_status(inventory)),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(0.58, 0.54, 0.48)),
            ));
        });
}

fn spawn_currency_footer(parent: &mut ChildBuilder<'_>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(28.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(14.0),
                padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(6.0), Val::Px(6.0)),
                border: UiRect::top(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(FOOTER_BG),
            BorderColor(Color::srgb(0.28, 0.18, 0.1)),
        ))
        .with_children(|footer| {
            spawn_coin_display(footer, Color::srgb(0.92, 0.76, 0.18), "0");
            spawn_coin_display(footer, Color::srgb(0.72, 0.74, 0.78), "0");
            spawn_coin_display(footer, Color::srgb(0.78, 0.48, 0.28), "0");
        });
}

fn spawn_coin_display(parent: &mut ChildBuilder<'_>, coin_color: Color, amount: &str) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|group| {
            group
                .spawn((
                    Node {
                        width: Val::Px(14.0),
                        height: Val::Px(14.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(coin_color),
                    BorderColor(Color::srgb(0.18, 0.12, 0.08)),
                ));
            group.spawn((
                Text::new(amount),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.88, 0.84)),
            ));
        });
}

pub fn cleanup_inventory_window(
    mut commands: Commands,
    mut open: ResMut<InventoryWindowOpen>,
    mut selected: ResMut<InventorySelectedSlot>,
    windows: Query<Entity, With<InventoryWindow>>,
) {
    open.0 = false;
    selected.0 = 0;
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
    game: Res<State<GameState>>,
    dungeon: Option<Res<State<DungeonPlayState>>>,
    mut time: ResMut<Time<Virtual>>,
) {
    let close = open.0 && keyboard.just_pressed(KeyCode::Escape);
    let toggle = keyboard.just_pressed(KeyCode::KeyI);

    if !close && !toggle {
        return;
    }

    if open.0 {
        close_inventory(
            &mut open,
            &mut commands,
            &windows,
            game.get(),
            dungeon.as_deref(),
            &mut time,
        );
    } else {
        open.0 = true;
        spawn_inventory_window(&mut commands, &inventory);
        time.pause();
    }
}

pub fn handle_inventory_close_button(
    interactions: Query<&Interaction, (Changed<Interaction>, With<InventoryCloseButton>)>,
    mut commands: Commands,
    windows: Query<Entity, With<InventoryWindow>>,
    game: Res<State<GameState>>,
    dungeon: Option<Res<State<DungeonPlayState>>>,
    mut time: ResMut<Time<Virtual>>,
    mut open: ResMut<InventoryWindowOpen>,
) {
    if !open.0 {
        return;
    }

    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            close_inventory(
                &mut open,
                &mut commands,
                &windows,
                game.get(),
                dungeon.as_deref(),
                &mut time,
            );
            return;
        }
    }
}

pub fn handle_inventory_slot_click(
    mut interactions: Query<
        (&Interaction, &InventorySlot),
        (Changed<Interaction>, With<Button>),
    >,
    mut selected: ResMut<InventorySelectedSlot>,
    mut slots: Query<(&InventorySlot, &mut BorderColor)>,
) {
    for (interaction, slot) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        selected.0 = slot.index;
        for (entry, mut border) in &mut slots {
            if entry.index == selected.0 {
                *border = BorderColor(SLOT_SELECTED);
            } else {
                *border = BorderColor(SLOT_BORDER);
            }
        }
    }
}

fn close_inventory(
    open: &mut InventoryWindowOpen,
    commands: &mut Commands,
    windows: &Query<Entity, With<InventoryWindow>>,
    game: &GameState,
    dungeon: Option<&State<DungeonPlayState>>,
    time: &mut Time<Virtual>,
) {
    open.0 = false;
    for entity in windows.iter() {
        commands.entity(entity).try_despawn_recursive();
    }
    if should_resume_time(game, dungeon) {
        time.unpause();
    }
}

fn should_resume_time(game: &GameState, dungeon: Option<&State<DungeonPlayState>>) -> bool {
    if !matches!(game, GameState::Dungeon) {
        return true;
    }

    let Some(dungeon) = dungeon else {
        return true;
    };

    !matches!(
        dungeon.get(),
        DungeonPlayState::Paused | DungeonPlayState::Dying | DungeonPlayState::Dead
    )
}

fn slot_visuals(inventory: &Inventory, index: usize) -> (Color, String, String) {
    let slot = &inventory.slots[index];
    match slot.material {
        Some(material) if slot.count > 0 => {
            let (color, label) = material_visual(material);
            let stack = if slot.count > 1 {
                slot.count.to_string()
            } else {
                String::new()
            };
            (color, label.to_string(), stack)
        }
        _ => (
            Color::srgba(0.0, 0.0, 0.0, 0.0),
            String::new(),
            String::new(),
        ),
    }
}

fn material_visual(material: MaterialId) -> (Color, &'static str) {
    match material {
        MaterialId::SlimeGel => (Color::srgb(0.2, 0.45, 0.82), "Gel"),
        MaterialId::SlimeCore => (Color::srgb(0.28, 0.72, 0.34), "Core"),
        MaterialId::LeatherWing => (Color::srgb(0.52, 0.28, 0.62), "Wing"),
        MaterialId::Fang => (Color::srgb(0.86, 0.84, 0.78), "Fang"),
        MaterialId::IronScrap => (Color::srgb(0.48, 0.5, 0.54), "Iron"),
    }
}

fn forge_status(inventory: &Inventory) -> String {
    if can_craft_recipe(inventory, &IRON_SWORD_RECIPE) {
        format!("{} ready (5 Gel, 3 Iron)", IRON_SWORD_RECIPE.name)
    } else {
        String::new()
    }
}

pub fn sync_inventory_display(
    inventory: Res<Inventory>,
    open: Res<InventoryWindowOpen>,
    selected: Res<InventorySelectedSlot>,
    mut slots: Query<(&InventorySlot, &Children, &mut BorderColor)>,
    mut icons: Query<(&mut BackgroundColor, &Children), With<InventorySlotIcon>>,
    mut icon_labels: Query<
        &mut Text,
        (Without<InventorySlotStackText>, Without<ForgeStatusLabel>),
    >,
    stacks: Query<&Children, With<InventorySlotStack>>,
    mut stack_texts: Query<&mut Text, With<InventorySlotStackText>>,
    mut status: Query<&mut Text, With<ForgeStatusLabel>>,
) {
    if !open.0 {
        return;
    }

    let inventory_changed = inventory.is_changed();
    let selection_changed = selected.is_changed();

    if !inventory_changed && !selection_changed {
        return;
    }

    for (slot, children, mut border) in &mut slots {
        *border = if slot.index == selected.0 {
            BorderColor(SLOT_SELECTED)
        } else {
            BorderColor(SLOT_BORDER)
        };

        if !inventory_changed {
            continue;
        }

        let (icon_color, icon_label, stack_label) = slot_visuals(&inventory, slot.index);
        for child in children.iter() {
            if let Ok((mut bg, icon_children)) = icons.get_mut(*child) {
                *bg = BackgroundColor(icon_color);
                for icon_child in icon_children.iter() {
                    if let Ok(mut text) = icon_labels.get_mut(*icon_child) {
                        text.0 = icon_label.clone();
                    }
                }
            }

            if let Ok(stack_children) = stacks.get(*child) {
                for stack_child in stack_children.iter() {
                    if let Ok(mut text) = stack_texts.get_mut(*stack_child) {
                        text.0 = stack_label.clone();
                    }
                }
            }
        }
    }

    if inventory_changed {
        if let Ok(mut text) = status.get_single_mut() {
            text.0 = forge_status(&inventory);
        }
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