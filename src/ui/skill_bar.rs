use bevy::prelude::*;

use crate::combat::{SkillBindings, SKILL_SLOT_COUNT};

#[derive(Component)]
pub struct SkillBarHud;

#[derive(Component, Clone, Copy)]
pub struct SkillSlot {
    pub index: usize,
}

#[derive(Component, Clone, Copy)]
pub struct SkillSlotIcon {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct SkillSlotKeyLabel;

#[derive(Component, Clone, Copy)]
pub struct SkillSlotIconLabel {
    pub slot_index: usize,
}

#[derive(Resource, Default)]
pub struct SkillBarDrag {
    pub from_slot: Option<usize>,
}

const SLOT_SIZE: f32 = 54.0;
const SLOT_GAP: f32 = 6.0;
const BAR_BOTTOM: f32 = 14.0;

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
                width: Val::Px(SLOT_SIZE),
                height: Val::Px(SLOT_SIZE),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
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
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.76, 0.84)),
            ));
            slot.spawn((
                SkillSlotIcon { slot_index: index },
                Node {
                    width: Val::Px(34.0),
                    height: Val::Px(28.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.14, 0.14, 0.18, 0.85)),
                BorderColor(Color::srgba(0.25, 0.27, 0.32, 0.9)),
            ))
            .with_children(|icon| {
                icon.spawn((
                    SkillSlotIconLabel { slot_index: index },
                    Text::new(""),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.94, 0.98)),
                ));
            });
        });
}

pub fn sync_skill_bar(
    bindings: Res<SkillBindings>,
    drag: Res<SkillBarDrag>,
    mut slots: Query<
        (
            &SkillSlot,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (With<SkillSlot>, Without<SkillSlotIcon>),
    >,
    mut icons: Query<
        (&SkillSlotIcon, &mut BackgroundColor),
        (With<SkillSlotIcon>, Without<SkillSlot>),
    >,
    mut labels: Query<(&SkillSlotIconLabel, &mut Text)>,
) {
    for (slot, interaction, mut bg, mut border) in &mut slots {
        apply_slot_highlight(slot, interaction, drag.from_slot, &mut bg, &mut border);
    }

    for (icon, mut icon_bg) in &mut icons {
        let skill = bindings.slots[icon.slot_index];
        if let Some(skill) = skill {
            icon_bg.0 = skill.color();
        } else {
            icon_bg.0 = Color::srgba(0.14, 0.14, 0.18, 0.85);
        }
    }

    for (label, mut text) in &mut labels {
        let next = bindings
            .slots[label.slot_index]
            .map(|skill| skill.abbrev())
            .unwrap_or("");
        if text.as_str() != next {
            text.0 = next.to_string();
        }
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
    drag.from_slot = None;
    for entity in &hud {
        commands.entity(entity).despawn_recursive();
    }
}

