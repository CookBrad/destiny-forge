use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;

use crate::core::ProfileDirty;
use crate::forging::{can_craft_recipe, try_craft_iron_sword, IRON_SWORD_RECIPE};
use crate::items::{Inventory, MaterialId};

const PANEL_PADDING: f32 = 16.0;
const PANEL_WIDTH: f32 = 300.0;

const FRAME_BG: Color = Color::srgb(0.14, 0.09, 0.06);
const FRAME_BORDER: Color = Color::srgb(0.55, 0.4, 0.16);
const HEADER_BG: Color = Color::srgb(0.1, 0.06, 0.04);
const CLOSE_BUTTON: Color = Color::srgb(0.72, 0.14, 0.1);

#[derive(Resource, Default, Debug)]
pub struct ForgeWindowOpen(pub bool);

pub fn forge_closed(open: Res<ForgeWindowOpen>) -> bool {
    !open.0
}

pub fn forge_window_open(open: Res<ForgeWindowOpen>) -> bool {
    open.0
}

#[derive(Component)]
pub struct ForgeWindow;

#[derive(Component)]
pub struct ForgeCloseButton;

#[derive(Component)]
pub struct ForgeCraftButton;

#[derive(Component)]
pub struct ForgeStatusLabel;

pub fn open_forge_window(
    open: &mut ForgeWindowOpen,
    commands: &mut Commands,
    inventory: &Inventory,
    windows: &Query<Entity, With<ForgeWindow>>,
    time: &mut Time<Virtual>,
) {
    if open.0 || !windows.is_empty() {
        return;
    }

    open.0 = true;
    spawn_forge_window(commands, inventory);
    time.pause();
}

pub fn spawn_forge_window(commands: &mut Commands, inventory: &Inventory) {
    commands
        .spawn((
            ForgeWindow,
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
                        width: Val::Px(PANEL_WIDTH + 8.0),
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
                                row_gap: Val::Px(12.0),
                                width: Val::Px(PANEL_WIDTH),
                                padding: UiRect::all(Val::Px(PANEL_PADDING)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(FRAME_BG),
                            BorderColor(Color::srgb(0.32, 0.22, 0.12)),
                        ))
                        .with_children(|panel| {
                            spawn_header(panel);
                            spawn_recipe_panel(panel, inventory);
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
                padding: UiRect::horizontal(Val::Px(4.0)),
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(HEADER_BG),
            BorderColor(Color::srgb(0.28, 0.18, 0.1)),
        ))
        .with_children(|header| {
            header.spawn((
                Text::new("The Forge"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.94, 0.9, 0.82)),
            ));

            header
                .spawn((
                    Button,
                    ForgeCloseButton,
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

fn spawn_recipe_panel(parent: &mut ChildBuilder<'_>, inventory: &Inventory) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            align_items: AlignItems::Stretch,
            ..default()
        })
        .with_children(|panel| {
            panel.spawn((
                Text::new(recipe_description()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.68, 0.6)),
            ));

            panel
                .spawn((
                    Button,
                    ForgeCraftButton,
                    Node {
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.18, 0.14, 0.1)),
                    BorderColor(Color::srgb(0.42, 0.32, 0.18)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(format!("Craft {}", IRON_SWORD_RECIPE.name)),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.88, 0.78)),
                    ));
                });

            panel.spawn((
                ForgeStatusLabel,
                Text::new(forge_status(inventory)),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.68, 0.74, 0.82)),
            ));

            panel.spawn((
                Text::new("Esc — Close"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.52, 0.5, 0.46)),
            ));
        });
}

pub fn cleanup_forge_window(
    mut commands: Commands,
    mut open: ResMut<ForgeWindowOpen>,
    windows: Query<Entity, With<ForgeWindow>>,
    mut time: ResMut<Time<Virtual>>,
) {
    open.0 = false;
    for entity in &windows {
        commands.entity(entity).try_despawn_recursive();
    }
    if time.is_paused() {
        time.unpause();
    }
}

pub fn handle_forge_close_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    interactions: Query<&Interaction, (Changed<Interaction>, With<ForgeCloseButton>)>,
    mut commands: Commands,
    mut open: ResMut<ForgeWindowOpen>,
    windows: Query<Entity, With<ForgeWindow>>,
    mut time: ResMut<Time<Virtual>>,
) {
    if !open.0 {
        return;
    }

    let close_key = keyboard.just_pressed(KeyCode::Escape);
    let close_button = interactions
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);

    if close_key || close_button {
        close_forge_window(&mut open, &mut commands, &windows, &mut time);
    }
}

fn close_forge_window(
    open: &mut ForgeWindowOpen,
    commands: &mut Commands,
    windows: &Query<Entity, With<ForgeWindow>>,
    time: &mut Time<Virtual>,
) {
    open.0 = false;
    for entity in windows.iter() {
        commands.entity(entity).try_despawn_recursive();
    }
    time.unpause();
}

fn recipe_description() -> String {
    let costs = IRON_SWORD_RECIPE
        .costs
        .iter()
        .map(|(material, amount)| format!("{} {}", amount, material_name(*material)))
        .collect::<Vec<_>>()
        .join(", ");

    format!("{} requires: {}", IRON_SWORD_RECIPE.name, costs)
}

fn material_name(material: MaterialId) -> &'static str {
    match material {
        MaterialId::SlimeGel => "Slime Gel",
        MaterialId::SlimeCore => "Slime Core",
        MaterialId::LeatherWing => "Leather Wing",
        MaterialId::Fang => "Fang",
        MaterialId::IronScrap => "Iron Scrap",
    }
}

fn forge_status(inventory: &Inventory) -> String {
    if can_craft_recipe(inventory, &IRON_SWORD_RECIPE) {
        format!("{} is ready to craft.", IRON_SWORD_RECIPE.name)
    } else {
        "Gather the required materials from your backpack.".to_string()
    }
}

pub fn sync_forge_display(
    inventory: Res<Inventory>,
    open: Res<ForgeWindowOpen>,
    mut status: Query<&mut Text, With<ForgeStatusLabel>>,
) {
    if !open.0 || !inventory.is_changed() {
        return;
    }

    if let Ok(mut text) = status.get_single_mut() {
        text.0 = forge_status(&inventory);
    }
}

pub fn handle_forge_craft_input(
    open: Res<ForgeWindowOpen>,
    mut interactions: Query<&Interaction, (Changed<Interaction>, With<ForgeCraftButton>)>,
    mut inventory: ResMut<Inventory>,
    mut loadout: ResMut<crate::player::Loadout>,
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
                format!(
                    "Forged {} — equipped for your next dungeon run.",
                    IRON_SWORD_RECIPE.name
                )
            } else {
                forge_status(&inventory)
            };
        }
    }
}