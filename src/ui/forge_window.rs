use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;

use crate::core::ProfileDirty;
use crate::forging::{
    forge_status, recipe_costs_text, recipe_requirement_text, recipe_set_bonus_hint, try_craft_recipe,
    ALL_RECIPES,
};
use crate::items::Inventory;
use crate::player::Loadout;

const PANEL_PADDING: f32 = 16.0;
const PANEL_WIDTH: f32 = 340.0;

const FRAME_BG: Color = Color::srgb(0.14, 0.09, 0.06);
const FRAME_BORDER: Color = Color::srgb(0.55, 0.4, 0.16);
const HEADER_BG: Color = Color::srgb(0.1, 0.06, 0.04);
const CLOSE_BUTTON: Color = Color::srgb(0.72, 0.14, 0.1);

#[derive(Resource, Default, Debug)]
pub struct ForgeWindowOpen(pub bool);

#[derive(Resource, Default, Debug)]
pub struct ForgeSelectedRecipe(pub usize);

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
pub struct ForgeRecipeNameLabel;

#[derive(Component)]
pub struct ForgeCostsLabel;

#[derive(Component)]
pub struct ForgeRequirementLabel;

#[derive(Component)]
pub struct ForgeSetBonusLabel;

#[derive(Component)]
pub struct ForgeStatusLabel;

pub fn open_forge_window(
    open: &mut ForgeWindowOpen,
    selected: &mut ForgeSelectedRecipe,
    commands: &mut Commands,
    inventory: &Inventory,
    loadout: &Loadout,
    windows: &Query<Entity, With<ForgeWindow>>,
    time: &mut Time<Virtual>,
) {
    if open.0 || !windows.is_empty() {
        return;
    }

    open.0 = true;
    selected.0 = 0;
    spawn_forge_window(commands, inventory, loadout, selected.0);
    time.pause();
}

pub fn spawn_forge_window(commands: &mut Commands, inventory: &Inventory, loadout: &Loadout, index: usize) {
    let recipe = ALL_RECIPES[index.min(ALL_RECIPES.len().saturating_sub(1))];

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
                                row_gap: Val::Px(10.0),
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
                            spawn_recipe_panel(panel, inventory, loadout, recipe);
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

fn spawn_recipe_panel(
    parent: &mut ChildBuilder<'_>,
    inventory: &Inventory,
    loadout: &Loadout,
    recipe: &crate::forging::Recipe,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            align_items: AlignItems::Stretch,
            ..default()
        })
        .with_children(|panel| {
            panel.spawn((
                ForgeRecipeNameLabel,
                Text::new(format!("{} ({}/{})", recipe.name, 1, ALL_RECIPES.len())),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.88, 0.78)),
            ));

            panel.spawn((
                ForgeCostsLabel,
                Text::new(recipe_costs_text(inventory, recipe)),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.68, 0.6)),
            ));

            panel.spawn((
                ForgeRequirementLabel,
                Text::new(
                    recipe_requirement_text(loadout, recipe)
                        .unwrap_or_else(|| " ".to_string()),
                ),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.62, 0.66, 0.74)),
            ));

            panel.spawn((
                ForgeSetBonusLabel,
                Text::new(
                    recipe_set_bonus_hint(recipe)
                        .unwrap_or(" ")
                        .to_string(),
                ),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.58, 0.72, 0.62)),
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
                        Text::new(format!("Craft {}", recipe.name)),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.88, 0.78)),
                    ));
                });

            panel.spawn((
                ForgeStatusLabel,
                Text::new(forge_status(inventory, loadout, recipe)),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.68, 0.74, 0.82)),
            ));

            panel.spawn((
                Text::new("Up/Down — cycle recipe  ·  F — craft  ·  Esc — close"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.52, 0.5, 0.46)),
            ));
        });
}

pub fn cleanup_forge_window(
    mut commands: Commands,
    mut open: ResMut<ForgeWindowOpen>,
    mut selected: ResMut<ForgeSelectedRecipe>,
    windows: Query<Entity, With<ForgeWindow>>,
    mut time: ResMut<Time<Virtual>>,
) {
    open.0 = false;
    selected.0 = 0;
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
    mut selected: ResMut<ForgeSelectedRecipe>,
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
        close_forge_window(&mut open, &mut selected, &mut commands, &windows, &mut time);
    }
}

fn close_forge_window(
    open: &mut ForgeWindowOpen,
    selected: &mut ForgeSelectedRecipe,
    commands: &mut Commands,
    windows: &Query<Entity, With<ForgeWindow>>,
    time: &mut Time<Virtual>,
) {
    open.0 = false;
    selected.0 = 0;
    for entity in windows.iter() {
        commands.entity(entity).try_despawn_recursive();
    }
    time.unpause();
}

pub fn handle_forge_recipe_cycle(
    keyboard: Res<ButtonInput<KeyCode>>,
    open: Res<ForgeWindowOpen>,
    mut selected: ResMut<ForgeSelectedRecipe>,
    mut commands: Commands,
    windows: Query<Entity, With<ForgeWindow>>,
    inventory: Res<Inventory>,
    loadout: Res<Loadout>,
) {
    if !open.0 || ALL_RECIPES.is_empty() {
        return;
    }

    let mut changed = false;
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        selected.0 = selected.0.checked_sub(1).unwrap_or(ALL_RECIPES.len() - 1);
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        selected.0 = (selected.0 + 1) % ALL_RECIPES.len();
        changed = true;
    }

    if changed {
        for entity in &windows {
            commands.entity(entity).try_despawn_recursive();
        }
        spawn_forge_window(&mut commands, &inventory, &loadout, selected.0);
    }
}

pub fn sync_forge_display(
    inventory: Res<Inventory>,
    loadout: Res<Loadout>,
    open: Res<ForgeWindowOpen>,
    selected: Res<ForgeSelectedRecipe>,
    // ParamSet: multiple &mut Text queries conflict without full Without chains (B0001).
    mut texts: ParamSet<(
        Query<&mut Text, With<ForgeRecipeNameLabel>>,
        Query<&mut Text, With<ForgeCostsLabel>>,
        Query<&mut Text, With<ForgeRequirementLabel>>,
        Query<&mut Text, With<ForgeSetBonusLabel>>,
        Query<&mut Text, With<ForgeStatusLabel>>,
    )>,
) {
    if !open.0 {
        return;
    }

    if !inventory.is_changed() && !loadout.is_changed() {
        return;
    }

    let recipe = ALL_RECIPES[selected.0.min(ALL_RECIPES.len().saturating_sub(1))];

    if let Ok(mut text) = texts.p0().get_single_mut() {
        text.0 = format!(
            "{} ({}/{})",
            recipe.name,
            selected.0 + 1,
            ALL_RECIPES.len()
        );
    }
    if let Ok(mut text) = texts.p1().get_single_mut() {
        text.0 = recipe_costs_text(&inventory, recipe);
    }
    if let Ok(mut text) = texts.p2().get_single_mut() {
        text.0 = recipe_requirement_text(&loadout, recipe).unwrap_or_else(|| " ".to_string());
    }
    if let Ok(mut text) = texts.p3().get_single_mut() {
        text.0 = recipe_set_bonus_hint(recipe)
            .unwrap_or(" ")
            .to_string();
    }
    if let Ok(mut text) = texts.p4().get_single_mut() {
        text.0 = forge_status(&inventory, &loadout, recipe);
    }
}

fn craft_selected_recipe(
    inventory: &mut Inventory,
    loadout: &mut Loadout,
    selected: usize,
) -> Option<String> {
    let recipe = ALL_RECIPES.get(selected)?;
    if try_craft_recipe(inventory, loadout, recipe) {
        Some(format!(
            "Forged {} — equipped for your next dungeon run.",
            recipe.name
        ))
    } else {
        None
    }
}

pub fn handle_forge_craft_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    open: Res<ForgeWindowOpen>,
    selected: Res<ForgeSelectedRecipe>,
    mut interactions: Query<&Interaction, (Changed<Interaction>, With<ForgeCraftButton>)>,
    mut inventory: ResMut<Inventory>,
    mut loadout: ResMut<Loadout>,
    mut profile_dirty: ResMut<ProfileDirty>,
    mut status: Query<&mut Text, With<ForgeStatusLabel>>,
) {
    if !open.0 {
        return;
    }

    let craft_key = keyboard.just_pressed(KeyCode::KeyF);
    let craft_button = interactions
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);

    if !craft_key && !craft_button {
        return;
    }

    if let Some(message) = craft_selected_recipe(&mut inventory, &mut loadout, selected.0) {
        profile_dirty.mark();
        if let Ok(mut text) = status.get_single_mut() {
            text.0 = message;
        }
    } else if let Ok(mut text) = status.get_single_mut() {
        let recipe = ALL_RECIPES[selected.0.min(ALL_RECIPES.len().saturating_sub(1))];
        text.0 = forge_status(&inventory, &loadout, recipe);
    }
}