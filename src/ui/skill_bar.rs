use bevy::prelude::*;
use bevy::ui::widget::{ImageNode, NodeImageMode};
use bevy::window::PrimaryWindow;

use crate::combat::{SkillBindings, SkillIconAssets, SKILL_SLOT_COUNT};

#[derive(Component)]
pub struct SkillBarHud;

#[derive(Component, Clone, Copy)]
pub struct SkillSlot {
    pub index: usize,
}

#[derive(Component, Clone, Copy)]
pub struct SkillSlotImage {
    pub slot_index: usize,
}

#[derive(Component, Clone, Copy)]
pub struct SkillSlotNameLabel {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct SkillSlotKeyLabel;

#[derive(Component)]
pub struct SkillBarDragGhost;

#[derive(Component)]
pub(crate) struct SkillBarDragGhostName;

#[derive(Resource, Default)]
pub struct SkillBarDrag {
    pub from_slot: Option<usize>,
    pub ghost: Option<Entity>,
}

const SLOT_WIDTH: f32 = 54.0;
const SLOT_HEIGHT: f32 = 68.0;
const SLOT_GAP: f32 = 6.0;
const BAR_BOTTOM: f32 = 14.0;
const ICON_SIZE: f32 = 30.0;
const NAME_FONT_SIZE: f32 = 9.0;
const GHOST_WIDTH: f32 = SLOT_WIDTH;
const GHOST_HEIGHT: f32 = ICON_SIZE + NAME_FONT_SIZE + 6.0;

pub fn spawn_skill_bar(mut commands: Commands) {
    commands
        .spawn((
            SkillBarHud,
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
                for index in 0..SKILL_SLOT_COUNT {
                    spawn_skill_slot(row, index);
                }
            });
        });
}

fn spawn_skill_slot(parent: &mut ChildBuilder<'_>, index: usize) {
    parent
        .spawn((
            SkillSlot { index },
            Button,
            Node {
                width: Val::Px(SLOT_WIDTH),
                height: Val::Px(SLOT_HEIGHT),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(3.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.92)),
            BorderColor(Color::srgba(0.35, 0.38, 0.45, 0.9)),
        ))
        .with_children(|slot| {
            slot.spawn((
                SkillSlotKeyLabel,
                Text::new((index + 1).to_string()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.76, 0.84)),
            ));
            slot.spawn((
                SkillSlotImage { slot_index: index },
                ImageNode {
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Node {
                    width: Val::Px(ICON_SIZE),
                    height: Val::Px(ICON_SIZE),
                    ..default()
                },
                Visibility::Hidden,
            ));
            slot.spawn((
                SkillSlotNameLabel { slot_index: index },
                Text::new(""),
                TextFont {
                    font_size: NAME_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.82, 0.9)),
                Visibility::Hidden,
            ));
        });
}

pub fn sync_skill_bar(
    bindings: Res<SkillBindings>,
    drag: Res<SkillBarDrag>,
    icon_assets: Res<SkillIconAssets>,
    mut slots: Query<
        (
            &SkillSlot,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<SkillSlot>,
    >,
    mut icons: Query<
        (&SkillSlotImage, &mut ImageNode, &mut Visibility),
        (With<SkillSlotImage>, Without<SkillSlotNameLabel>),
    >,
    mut names: Query<
        (&SkillSlotNameLabel, &mut Text, &mut Visibility),
        (With<SkillSlotNameLabel>, Without<SkillSlotImage>),
    >,
) {
    for (slot, interaction, mut bg, mut border) in &mut slots {
        apply_slot_highlight(slot, interaction, drag.from_slot, &mut bg, &mut border);
    }

    for (image, mut node, mut visibility) in &mut icons {
        let dragging = drag.from_slot == Some(image.slot_index);
        if let Some(skill) = bindings.slots[image.slot_index] {
            node.image = icon_assets.handle_for(skill);
            node.rect = Some(skill.icon_rect());
            node.color = Color::WHITE;
            *visibility = if dragging {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    for (label, mut text, mut visibility) in &mut names {
        let dragging = drag.from_slot == Some(label.slot_index);
        let next = bindings
            .slots[label.slot_index]
            .map(|skill| skill.label())
            .unwrap_or("");
        if text.as_str() != next {
            text.0 = next.to_string();
        }
        *visibility = if !next.is_empty() && !dragging {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn update_skill_bar_drag_ghost(
    mut commands: Commands,
    mut drag: ResMut<SkillBarDrag>,
    bindings: Res<SkillBindings>,
    icon_assets: Res<SkillIconAssets>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut ghosts: Query<&mut Node, With<SkillBarDragGhost>>,
    mut ghost_names: Query<&mut Text, With<SkillBarDragGhostName>>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };

    let Some(from_slot) = drag.from_slot else {
        despawn_drag_ghost(&mut commands, &mut drag);
        return;
    };

    let Some(skill) = bindings.slots[from_slot] else {
        despawn_drag_ghost(&mut commands, &mut drag);
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let ghost_left = cursor.x - GHOST_WIDTH * 0.5;
    let ghost_top = cursor.y - GHOST_HEIGHT * 0.5;

    if drag.ghost.is_none() {
        let ghost = commands
            .spawn((
                SkillBarDragGhost,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(GHOST_WIDTH),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(2.0),
                    left: Val::Px(ghost_left),
                    top: Val::Px(ghost_top),
                    ..default()
                },
                GlobalZIndex(200),
            ))
            .with_children(|ghost| {
                ghost.spawn((
                    ImageNode {
                        image: icon_assets.handle_for(skill),
                        rect: Some(skill.icon_rect()),
                        image_mode: NodeImageMode::Stretch,
                        color: Color::srgba(1.0, 1.0, 1.0, 0.92),
                        ..default()
                    },
                    Node {
                        width: Val::Px(ICON_SIZE),
                        height: Val::Px(ICON_SIZE),
                        ..default()
                    },
                ));
                ghost.spawn((
                    SkillBarDragGhostName,
                    Text::new(skill.label()),
                    TextFont {
                        font_size: NAME_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::srgb(0.78, 0.82, 0.9)),
                ));
            })
            .id();
        drag.ghost = Some(ghost);
        return;
    }

    let Ok(mut node) = ghosts.get_single_mut() else {
        return;
    };

    node.left = Val::Px(ghost_left);
    node.top = Val::Px(ghost_top);

    if let Ok(mut name) = ghost_names.get_single_mut() {
        let label = skill.label();
        if name.as_str() != label {
            name.0 = label.to_string();
        }
    }
}

fn despawn_drag_ghost(commands: &mut Commands, drag: &mut SkillBarDrag) {
    if let Some(entity) = drag.ghost.take() {
        commands.entity(entity).despawn_recursive();
    }
}

fn apply_slot_highlight(
    slot: &SkillSlot,
    interaction: &Interaction,
    drag_from: Option<usize>,
    bg: &mut BackgroundColor,
    border: &mut BorderColor,
) {
    let dragging = drag_from == Some(slot.index);
    let drop_target = drag_from.is_some()
        && matches!(*interaction, Interaction::Hovered | Interaction::Pressed);

    if dragging {
        bg.0 = Color::srgba(0.18, 0.2, 0.28, 0.96);
        border.0 = Color::srgb(0.95, 0.82, 0.35);
    } else if drop_target {
        bg.0 = Color::srgba(0.14, 0.18, 0.24, 0.96);
        border.0 = Color::srgb(0.55, 0.78, 0.95);
    } else {
        bg.0 = Color::srgba(0.08, 0.08, 0.12, 0.92);
        border.0 = Color::srgba(0.35, 0.38, 0.45, 0.9);
    }
}

pub fn handle_skill_bar_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    mut bindings: ResMut<SkillBindings>,
    mut drag: ResMut<SkillBarDrag>,
    slots: Query<(&SkillSlot, &Interaction), With<SkillSlot>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        for (slot, interaction) in &slots {
            if *interaction != Interaction::Pressed {
                continue;
            }
            if let Some(from) = drag.from_slot {
                bindings.swap_slots(from, slot.index);
                drag.from_slot = None;
            } else if bindings.slots[slot.index].is_some() {
                drag.from_slot = Some(slot.index);
            }
            return;
        }
    }

    if mouse.just_released(MouseButton::Left) {
        if let Some(from) = drag.from_slot {
            let mut target = None;
            for (slot, interaction) in &slots {
                if matches!(*interaction, Interaction::Hovered | Interaction::Pressed) {
                    target = Some(slot.index);
                    break;
                }
            }
            if let Some(to) = target {
                bindings.swap_slots(from, to);
            }
            drag.from_slot = None;
        }
    }
}

pub fn cleanup_skill_bar(
    mut commands: Commands,
    mut drag: ResMut<SkillBarDrag>,
    hud: Query<Entity, With<SkillBarHud>>,
) {
    despawn_drag_ghost(&mut commands, &mut drag);
    drag.from_slot = None;
    for entity in &hud {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn setup_skill_icon_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SkillIconAssets::load(&asset_server));
}