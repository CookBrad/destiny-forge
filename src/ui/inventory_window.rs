use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;

use crate::core::{DungeonPlayState, GameState};
use crate::items::{Inventory, MaterialId, INVENTORY_SLOT_COUNT};
use crate::player::Loadout;

use super::eat_feedback::FoodEaten;
use super::loadout_strip::{spawn_loadout_strip, LoadoutSwapAccess};

const GRID_COLUMNS: usize = 8;
const SLOT_SIZE: f32 = 52.0;
const SLOT_GAP: f32 = 3.0;
const PANEL_PADDING: f32 = 10.0;
const PANEL_SCREEN_MARGIN: f32 = 20.0;

const FRAME_BG: Color = Color::srgb(0.14, 0.09, 0.06);
const FRAME_BORDER: Color = Color::srgb(0.55, 0.4, 0.16);
const HEADER_BG: Color = Color::srgb(0.1, 0.06, 0.04);
const SLOT_BG: Color = Color::srgb(0.08, 0.05, 0.04);
const SLOT_BORDER: Color = Color::srgb(0.24, 0.17, 0.11);
const SLOT_SELECTED: Color = Color::srgb(0.95, 0.48, 0.1);
const CLOSE_BUTTON: Color = Color::srgb(0.72, 0.14, 0.1);
const FOOTER_BG: Color = Color::srgb(0.09, 0.06, 0.04);
const ITEM_NAME_TEXT_COLOR: Color = Color::srgb(0.94, 0.9, 0.82);
const ITEM_DESCRIPTION_TEXT_COLOR: Color = Color::srgb(0.72, 0.7, 0.66);
const ITEM_BUFF_TEXT_COLOR: Color = Color::srgb(0.95, 0.72, 0.36);
const ACTION_BUTTON_BG: Color = Color::srgb(0.18, 0.14, 0.1);
const ACTION_BUTTON_BORDER: Color = Color::srgb(0.42, 0.32, 0.18);

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
pub struct InventoryIconLabel;

#[derive(Component)]
pub struct InventoryCloseButton;

#[derive(Component)]
pub struct InventoryItemName;

#[derive(Component)]
pub struct InventoryItemDescription;

#[derive(Component)]
pub struct InventoryItemBuff;

#[derive(Component)]
pub struct InventoryEatButton;

#[derive(Component)]
pub struct InventorySortButton;

pub fn spawn_inventory_window(
    commands: &mut Commands,
    inventory: &Inventory,
    loadout: &Loadout,
    access: LoadoutSwapAccess,
    selected_slot_index: usize,
) {
    let grid_width = GRID_COLUMNS as f32 * SLOT_SIZE + (GRID_COLUMNS as f32 - 1.0) * SLOT_GAP;
    let panel_width = grid_width + PANEL_PADDING * 2.0;

    commands
        .spawn((
            InventoryWindow,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::left(Val::Px(PANEL_SCREEN_MARGIN)),
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
                            spawn_slot_grid(panel, inventory, grid_width, selected_slot_index);
                            spawn_selected_item_description(
                                panel,
                                inventory,
                                loadout,
                                access,
                                selected_slot_index,
                            );
                            spawn_loadout_strip(panel, loadout, access);
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
                        Text::new("\u{25c6}"),
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

fn spawn_slot_grid(
    parent: &mut ChildBuilder<'_>,
    inventory: &Inventory,
    grid_width: f32,
    selected_slot_index: usize,
) {
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
                spawn_slot(grid, inventory, index, selected_slot_index);
            }
        });
}

fn spawn_slot(
    parent: &mut ChildBuilder<'_>,
    inventory: &Inventory,
    index: usize,
    selected_slot_index: usize,
) {
    let (icon_color, icon_label, stack) = slot_visuals(inventory, index);
    let (border_width, border_color) = slot_border(index, selected_slot_index);

    parent
        .spawn((
            Button,
            InventorySlot { index },
            Node {
                width: Val::Px(SLOT_SIZE),
                height: Val::Px(SLOT_SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(border_width)),
                ..default()
            },
            BackgroundColor(SLOT_BG),
            BorderColor(border_color),
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
                    InventoryIconLabel,
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

fn slot_border(index: usize, selected_slot_index: usize) -> (f32, Color) {
    if index == selected_slot_index {
        (2.0, SLOT_SELECTED)
    } else {
        (1.0, SLOT_BORDER)
    }
}

fn spawn_selected_item_description(
    parent: &mut ChildBuilder<'_>,
    inventory: &Inventory,
    loadout: &Loadout,
    access: LoadoutSwapAccess,
    selected_slot_index: usize,
) {
    let item_description =
        selected_item_description(inventory, loadout, access, selected_slot_index);

    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(8.0),
            padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(0.0), Val::Px(8.0)),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|text| {
                text.spawn((
                    InventoryItemName,
                    Text::new(item_description.name_line),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(ITEM_NAME_TEXT_COLOR),
                ));
                text.spawn((
                    InventoryItemDescription,
                    Text::new(item_description.description),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(ITEM_DESCRIPTION_TEXT_COLOR),
                ));
                text.spawn((
                    InventoryItemBuff,
                    Text::new(item_description.buff_line),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(ITEM_BUFF_TEXT_COLOR),
                ));
            });

            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|actions| {
                if access == LoadoutSwapAccess::Hub {
                    spawn_action_button(actions, InventoryEatButton, "Eat [E]");
                }
                spawn_action_button(actions, InventorySortButton, "Sort");
            });
        });
}

fn spawn_action_button(parent: &mut ChildBuilder<'_>, marker: impl Component, label: &str) {
    parent
        .spawn((
            Button,
            marker,
            Node {
                min_width: Val::Px(64.0),
                height: Val::Px(22.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(ACTION_BUTTON_BG),
            BorderColor(ACTION_BUTTON_BORDER),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(ITEM_NAME_TEXT_COLOR),
            ));
        });
}

struct SelectedItemDescription {
    name_line: String,
    description: String,
    buff_line: String,
}

fn selected_item_description(
    inventory: &Inventory,
    loadout: &Loadout,
    access: LoadoutSwapAccess,
    selected_slot_index: usize,
) -> SelectedItemDescription {
    let active_buff_line = active_buff_line(loadout);
    let slot = inventory
        .slots
        .get(selected_slot_index)
        .copied()
        .unwrap_or_default();
    let Some(material) = slot.material.filter(|_| slot.count > 0) else {
        return SelectedItemDescription {
            name_line: "Empty slot".to_string(),
            description: String::new(),
            buff_line: active_buff_line,
        };
    };

    SelectedItemDescription {
        name_line: item_name_line(material, slot.count),
        description: material.description().to_string(),
        buff_line: item_buff_line(material, access, active_buff_line),
    }
}

fn active_buff_line(loadout: &Loadout) -> String {
    loadout
        .food_buff
        .map(|food| {
            format!(
                "Active: {} ({})",
                food.display_name(),
                loadout.active_food_buff().bonus_summary()
            )
        })
        .unwrap_or_default()
}

fn item_name_line(material: MaterialId, count: u32) -> String {
    format!(
        "{} \u{00d7}{}  \u{00b7}  {}",
        material.display_name(),
        count,
        material.category().label()
    )
}

fn item_buff_line(
    material: MaterialId,
    access: LoadoutSwapAccess,
    active_buff_line: String,
) -> String {
    let Some(buff) = material.food_buff() else {
        return active_buff_line;
    };
    let food_line = format!("{}  \u{00b7}  {}", buff.bonus_summary(), eat_hint(access));
    if active_buff_line.is_empty() {
        food_line
    } else {
        format!("{food_line}\n{active_buff_line}")
    }
}

fn eat_hint(access: LoadoutSwapAccess) -> &'static str {
    if access == LoadoutSwapAccess::Hub {
        "E to eat"
    } else {
        "Eat on the homestead"
    }
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
    forge: Res<crate::ui::forge_window::ForgeWindowOpen>,
    mut open: ResMut<InventoryWindowOpen>,
    mut commands: Commands,
    inventory: Res<Inventory>,
    loadout: Res<Loadout>,
    selected: Res<InventorySelectedSlot>,
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

    if toggle && forge.0 {
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
        spawn_inventory_window(
            &mut commands,
            &inventory,
            &loadout,
            LoadoutSwapAccess::from_game_state(game.get()),
            selected.0,
        );
        time.pause();
    }
}

pub fn rebuild_inventory_on_loadout_change(
    loadout: Res<Loadout>,
    inventory: Res<Inventory>,
    open: Res<InventoryWindowOpen>,
    selected: Res<InventorySelectedSlot>,
    game: Res<State<GameState>>,
    mut commands: Commands,
    windows: Query<Entity, With<InventoryWindow>>,
) {
    if !open.0 || !loadout.is_changed() || windows.is_empty() {
        return;
    }

    for entity in &windows {
        commands.entity(entity).try_despawn_recursive();
    }
    spawn_inventory_window(
        &mut commands,
        &inventory,
        &loadout,
        LoadoutSwapAccess::from_game_state(game.get()),
        selected.0,
    );
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

pub fn handle_inventory_sort_button(
    sort_buttons: Query<&Interaction, (Changed<Interaction>, With<InventorySortButton>)>,
    mut selected: ResMut<InventorySelectedSlot>,
    mut inventory: ResMut<Inventory>,
) {
    if !sort_buttons.iter().any(is_pressed) {
        return;
    }
    inventory.merge_stacks_and_sort_by_category();
    selected.0 = 0;
}

pub fn handle_inventory_eat_request(
    keyboard: Res<ButtonInput<KeyCode>>,
    eat_buttons: Query<&Interaction, (Changed<Interaction>, With<InventoryEatButton>)>,
    game: Res<State<GameState>>,
    selected: Res<InventorySelectedSlot>,
    mut inventory: ResMut<Inventory>,
    mut loadout: ResMut<Loadout>,
    mut food_eaten: EventWriter<FoodEaten>,
) {
    let eat_requested = keyboard.just_pressed(KeyCode::KeyE) || eat_buttons.iter().any(is_pressed);
    if !eat_requested || LoadoutSwapAccess::from_game_state(game.get()) != LoadoutSwapAccess::Hub {
        return;
    }

    let Some(food) = food_in_slot(&inventory, selected.0) else {
        return;
    };

    if loadout.eat_food_replacing_active_buff(&mut inventory, food) {
        food_eaten.send(FoodEaten { food });
        info!(
            "Ate {} — {} until the next hunt ends or you sleep.",
            food.display_name(),
            loadout.active_food_buff().bonus_summary()
        );
    }
}

fn is_pressed(interaction: &Interaction) -> bool {
    *interaction == Interaction::Pressed
}

fn food_in_slot(inventory: &Inventory, slot_index: usize) -> Option<MaterialId> {
    inventory
        .slots
        .get(slot_index)
        .and_then(|slot| slot.material)
        .filter(|material| material.food_buff().is_some())
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
        MaterialId::BoneShard => (Color::srgb(0.78, 0.76, 0.7), "Bone"),
        MaterialId::RotFlesh => (Color::srgb(0.55, 0.32, 0.28), "Rot"),
        MaterialId::RoyalSlimeCore => (Color::srgb(0.95, 0.75, 0.2), "Royal"),
        MaterialId::TurnipSeed => (Color::srgb(0.45, 0.55, 0.28), "T.Sd"),
        MaterialId::PotatoSeed => (Color::srgb(0.55, 0.42, 0.22), "P.Sd"),
        MaterialId::Turnip => (Color::srgb(0.72, 0.55, 0.78), "Trnp"),
        MaterialId::Potato => (Color::srgb(0.78, 0.68, 0.42), "Pota"),
        MaterialId::Hoe => (Color::srgb(0.55, 0.4, 0.22), "Hoe"),
        MaterialId::WateringCan => (Color::srgb(0.28, 0.48, 0.72), "Water"),
        MaterialId::RoastTurnip => (Color::srgb(0.82, 0.46, 0.22), "Roast"),
        MaterialId::PotatoStew => (Color::srgb(0.7, 0.5, 0.26), "Stew"),
    }
}

pub fn sync_inventory_selected_item_description(
    inventory: Res<Inventory>,
    loadout: Res<Loadout>,
    open: Res<InventoryWindowOpen>,
    selected: Res<InventorySelectedSlot>,
    game: Res<State<GameState>>,
    mut selected_item_description_texts: ParamSet<(
        Query<&mut Text, With<InventoryItemName>>,
        Query<&mut Text, With<InventoryItemDescription>>,
        Query<&mut Text, With<InventoryItemBuff>>,
    )>,
) {
    if !open.0 || !(inventory.is_changed() || selected.is_changed() || loadout.is_changed()) {
        return;
    }

    let item_description = selected_item_description(
        &inventory,
        &loadout,
        LoadoutSwapAccess::from_game_state(game.get()),
        selected.0,
    );
    for mut text in &mut selected_item_description_texts.p0() {
        text.0 = item_description.name_line.clone();
    }
    for mut text in &mut selected_item_description_texts.p1() {
        text.0 = item_description.description.clone();
    }
    for mut text in &mut selected_item_description_texts.p2() {
        text.0 = item_description.buff_line.clone();
    }
}

pub fn sync_inventory_display(
    inventory: Res<Inventory>,
    open: Res<InventoryWindowOpen>,
    selected: Res<InventorySelectedSlot>,
    mut slots: Query<(&InventorySlot, &Children, &mut BorderColor)>,
    mut icons: Query<(&mut BackgroundColor, &Children), With<InventorySlotIcon>>,
    stacks: Query<&Children, With<InventorySlotStack>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<InventoryIconLabel>>,
        Query<&mut Text, With<InventorySlotStackText>>,
    )>,
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
                    if let Ok(mut text) = texts.p0().get_mut(*icon_child) {
                        text.0 = icon_label.clone();
                    }
                }
            }

            if let Ok(stack_children) = stacks.get(*child) {
                for stack_child in stack_children.iter() {
                    if let Ok(mut text) = texts.p1().get_mut(*stack_child) {
                        text.0 = stack_label.clone();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn app_with_two_stews_and_e_pressed(game: GameState) -> App {
        let mut app = App::new();
        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(KeyCode::KeyE);
        let mut inventory = Inventory::default();
        inventory.try_add(MaterialId::PotatoStew, 2);
        app.insert_resource(keyboard)
            .insert_resource(State::new(game))
            .insert_resource(inventory)
            .init_resource::<Loadout>()
            .init_resource::<InventorySelectedSlot>()
            .add_event::<FoodEaten>();
        app
    }

    #[test]
    fn e_eats_selected_food_on_the_homestead() {
        let mut app = app_with_two_stews_and_e_pressed(GameState::Overworld);
        app.world_mut()
            .run_system_once(handle_inventory_eat_request)
            .unwrap();

        let world = app.world();
        let stew = world.resource::<Inventory>().count(MaterialId::PotatoStew);
        assert_eq!(stew, 1);
        assert_eq!(
            world.resource::<Loadout>().food_buff,
            Some(MaterialId::PotatoStew)
        );
        let food_eaten: Vec<_> = world
            .resource::<Events<FoodEaten>>()
            .iter_current_update_events()
            .copied()
            .collect();
        assert_eq!(
            food_eaten,
            vec![FoodEaten {
                food: MaterialId::PotatoStew
            }]
        );
    }

    #[test]
    fn eating_is_locked_in_the_dungeon() {
        let mut app = app_with_two_stews_and_e_pressed(GameState::Dungeon);
        app.world_mut()
            .run_system_once(handle_inventory_eat_request)
            .unwrap();

        let world = app.world();
        let stew = world.resource::<Inventory>().count(MaterialId::PotatoStew);
        assert_eq!(stew, 2);
        assert_eq!(world.resource::<Loadout>().food_buff, None);
    }

    #[test]
    fn selected_item_description_names_item_and_buff() {
        let mut inventory = Inventory::default();
        inventory.try_add(MaterialId::RoastTurnip, 3);
        let loadout = Loadout::default();

        let hub = selected_item_description(&inventory, &loadout, LoadoutSwapAccess::Hub, 0);
        assert_eq!(hub.name_line, "Roast Turnip \u{00d7}3  \u{00b7}  Food");
        assert_eq!(hub.buff_line, "+8% attack  \u{00b7}  E to eat");

        let locked = selected_item_description(&inventory, &loadout, LoadoutSwapAccess::Locked, 0);
        assert_eq!(
            locked.buff_line,
            "+8% attack  \u{00b7}  Eat on the homestead"
        );

        let empty = selected_item_description(&inventory, &loadout, LoadoutSwapAccess::Hub, 5);
        assert_eq!(empty.name_line, "Empty slot");
    }
}
