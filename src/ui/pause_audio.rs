use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::audio::AudioSettings;
use crate::core::ProfileDirty;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioControl {
    Music,
    Sfx,
}

#[derive(Component, Clone, Copy)]
pub struct AudioToggleButton(pub AudioControl);

#[derive(Component, Clone, Copy)]
pub struct AudioToggleLabel(pub AudioControl);

#[derive(Component, Clone, Copy)]
pub struct AudioSliderTrack(pub AudioControl);

#[derive(Component, Clone, Copy)]
pub struct AudioSliderFill(pub AudioControl);

#[derive(Component, Clone, Copy)]
pub struct AudioSliderLabel(pub AudioControl);

const ROW_WIDTH: f32 = 420.0;
const SLIDER_WIDTH: f32 = 180.0;
const SLIDER_HEIGHT: f32 = 14.0;

pub fn spawn_pause_audio_controls(parent: &mut ChildBuilder<'_>, settings: &AudioSettings) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(14.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|panel| {
            spawn_audio_row(panel, "Music", AudioControl::Music, settings.music_enabled, settings.music_volume);
            spawn_audio_row(panel, "Sound Effects", AudioControl::Sfx, settings.sfx_enabled, settings.sfx_volume);
        });
}

fn spawn_audio_row(
    parent: &mut ChildBuilder<'_>,
    title: &str,
    control: AudioControl,
    enabled: bool,
    volume: f32,
) {
    parent
        .spawn(Node {
            width: Val::Px(ROW_WIDTH),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(title),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.86, 0.9, 0.95)),
            ));

            row.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|controls| {
                controls
                    .spawn((
                        Button,
                        AudioToggleButton(control),
                        Node {
                            min_width: Val::Px(72.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.16, 0.18, 0.24, 0.95)),
                    ))
                    .with_children(|toggle| {
                        toggle.spawn((
                            Text::new(toggle_label(enabled)),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.92, 0.94, 0.98)),
                            AudioToggleLabel(control),
                        ));
                    });

                controls
                    .spawn((
                        Button,
                        AudioSliderTrack(control),
                        RelativeCursorPosition::default(),
                        Node {
                            width: Val::Px(SLIDER_WIDTH),
                            height: Val::Px(SLIDER_HEIGHT),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.12, 0.16, 0.95)),
                    ))
                    .with_children(|track| {
                        track.spawn((
                            AudioSliderFill(control),
                            Node {
                                width: Val::Percent(volume.clamp(0.0, 1.0) * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.35, 0.72, 0.42)),
                        ));
                    });

                controls.spawn((
                    Text::new(format!("{}%", (volume.clamp(0.0, 1.0) * 100.0).round() as i32)),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.72, 0.76, 0.82)),
                    AudioSliderLabel(control),
                ));
            });
        });
}

fn toggle_label(enabled: bool) -> &'static str {
    if enabled {
        "ON"
    } else {
        "OFF"
    }
}

pub fn handle_pause_audio_input(
    mouse: Res<ButtonInput<MouseButton>>,
    mut settings: ResMut<AudioSettings>,
    mut profile_dirty: ResMut<ProfileDirty>,
    mut toggles: Query<
        (&Interaction, &AudioToggleButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut sliders: Query<
        (&Interaction, &RelativeCursorPosition, &AudioSliderTrack),
        With<Button>,
    >,
) {
    for (interaction, toggle) in &mut toggles {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match toggle.0 {
            AudioControl::Music => settings.music_enabled = !settings.music_enabled,
            AudioControl::Sfx => settings.sfx_enabled = !settings.sfx_enabled,
        }
        profile_dirty.mark();
    }

    let dragging = mouse.pressed(MouseButton::Left);
    for (interaction, cursor, track) in &mut sliders {
        let active = matches!(*interaction, Interaction::Pressed)
            || (dragging && matches!(*interaction, Interaction::Hovered | Interaction::Pressed));

        if !active {
            continue;
        }

        let Some(normalized) = cursor.normalized else {
            continue;
        };

        let volume = normalized.x.clamp(0.0, 1.0);
        match track.0 {
            AudioControl::Music => settings.music_volume = volume,
            AudioControl::Sfx => settings.sfx_volume = volume,
        }
        profile_dirty.mark();
    }
}

pub fn sync_pause_audio_display(
    settings: Res<AudioSettings>,
    mut toggle_labels: Query<(&AudioToggleLabel, &mut Text)>,
    mut slider_fills: Query<(&AudioSliderFill, &mut Node)>,
    mut slider_labels: Query<(&AudioSliderLabel, &mut Text), Without<AudioToggleLabel>>,
) {
    if !settings.is_changed() {
        return;
    }

    for (label, mut text) in &mut toggle_labels {
        text.0 = match label.0 {
            AudioControl::Music => toggle_label(settings.music_enabled),
            AudioControl::Sfx => toggle_label(settings.sfx_enabled),
        }
        .to_string();
    }

    for (fill, mut node) in &mut slider_fills {
        let volume = match fill.0 {
            AudioControl::Music => settings.music_volume,
            AudioControl::Sfx => settings.sfx_volume,
        };
        node.width = Val::Percent(volume.clamp(0.0, 1.0) * 100.0);
    }

    for (label, mut text) in &mut slider_labels {
        let volume = match label.0 {
            AudioControl::Music => settings.music_volume,
            AudioControl::Sfx => settings.sfx_volume,
        };
        text.0 = format!("{}%", (volume.clamp(0.0, 1.0) * 100.0).round() as i32);
    }
}