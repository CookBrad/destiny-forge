use bevy::ecs::system::SystemParam;
use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;

use crate::audio::AudioSettings;
use crate::combat::SkillBindings;
use crate::core::{
    rename_profile_on_disk, sanitize_profile_name, ActiveProfile, CalendarRules, DayClock,
    GameSettings, GameState, PlayerProfile, ProfileAutosaveBlock, ProfileDirty, ProfileSwitch,
    RuntimeProfile, Season, ToolEnergy, PROFILE_COUNT,
};
use crate::input::{Action, ActionState};
use crate::items::Inventory;
use crate::player::{Loadout, WorldProgress};

use super::profile_picker::{begin_profile_run, ProfilePicker};

const CARD_WIDTH: f32 = 420.0;
const CARD_HEIGHT: f32 = 72.0;

#[derive(SystemParam)]
pub(crate) struct ProfileRunGate<'w> {
    inventory: ResMut<'w, Inventory>,
    loadout: ResMut<'w, Loadout>,
    progress: ResMut<'w, WorldProgress>,
    day_clock: ResMut<'w, DayClock>,
    tool_energy: ResMut<'w, ToolEnergy>,
    season: ResMut<'w, Season>,
    rules: Res<'w, CalendarRules>,
    active: ResMut<'w, ActiveProfile>,
    profile: ResMut<'w, PlayerProfile>,
    audio: ResMut<'w, AudioSettings>,
    bindings: ResMut<'w, SkillBindings>,
    global: ResMut<'w, GameSettings>,
    profile_dirty: ResMut<'w, ProfileDirty>,
    autosave_block: ResMut<'w, ProfileAutosaveBlock>,
}

impl ProfileRunGate<'_> {
    fn begin(&mut self, index: u8) -> bool {
        begin_profile_run(
            index,
            &mut RuntimeProfile {
                inventory: &mut self.inventory,
                loadout: &mut self.loadout,
                progress: &mut self.progress,
                day_clock: &mut self.day_clock,
                tool_energy: &mut self.tool_energy,
                season: &mut self.season,
                audio: &mut self.audio,
                bindings: &mut self.bindings,
            },
            &self.rules,
            &mut ProfileSwitch {
                active: &mut self.active,
                profile: &mut self.profile,
                global: &mut self.global,
                dirty: &mut self.profile_dirty,
                autosave_block: &mut self.autosave_block,
            },
        )
    }
}

pub(crate) type TitleProfileCardClicks<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static TitleProfileCard),
    (Changed<Interaction>, With<Button>),
>;

pub(crate) type TitleProfileRenameClicks<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static TitleProfileRenameButton),
    (Changed<Interaction>, With<Button>),
>;

#[derive(Component)]
pub struct TitleProfileContainer;

#[derive(Component, Clone, Copy)]
pub struct TitleProfileCard {
    pub index: u8,
}

#[derive(Component, Clone, Copy)]
pub struct TitleProfileCardName {
    pub index: u8,
}

#[derive(Component, Clone, Copy)]
pub struct TitleProfileCardDetails {
    pub index: u8,
}

#[derive(Component, Clone, Copy)]
pub struct TitleProfileRenameButton {
    pub index: u8,
}

#[derive(Resource, Default)]
pub struct ProfileRenameState {
    pub active: Option<ProfileRenameSession>,
}

#[derive(Clone, Debug)]
pub struct ProfileRenameSession {
    pub index: u8,
    pub buffer: String,
}

pub fn spawn_title_profile_cards(parent: &mut ChildBuilder<'_>, picker: &ProfilePicker) {
    parent
        .spawn((
            TitleProfileContainer,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|profiles| {
            for index in 0..PROFILE_COUNT {
                let summary = &picker.cards[index as usize];
                spawn_profile_card(profiles, index, summary);
            }
        });
}

fn spawn_profile_card(
    parent: &mut ChildBuilder<'_>,
    index: u8,
    summary: &super::profile_picker::ProfileCardSummary,
) {
    parent
        .spawn(Node {
            width: Val::Px(CARD_WIDTH),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Button,
                TitleProfileCard { index },
                Node {
                    flex_grow: 1.0,
                    height: Val::Px(CARD_HEIGHT),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.11, 0.16, 0.95)),
                BorderColor(Color::srgba(0.3, 0.34, 0.42, 0.9)),
            ))
            .with_children(|card| {
                card.spawn((
                    TitleProfileCardName { index },
                    Text::new(&summary.name),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.92, 0.55)),
                ));
                card.spawn((
                    TitleProfileCardDetails { index },
                    Text::new(card_details(summary)),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.68, 0.72, 0.78)),
                ));
            });

            row.spawn((
                Button,
                TitleProfileRenameButton { index },
                Node {
                    min_width: Val::Px(72.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.14, 0.16, 0.22, 0.95)),
                BorderColor(Color::srgba(0.35, 0.38, 0.46, 0.9)),
            ))
            .with_children(|rename| {
                rename.spawn((
                    Text::new("Rename"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.86, 0.92)),
                ));
            });
        });
}

fn card_details(summary: &super::profile_picker::ProfileCardSummary) -> String {
    if summary.needs_attention {
        return super::profile_picker::SAVE_NEEDS_ATTENTION.to_string();
    }
    let boss = if summary.boss_cleared {
        " · ✓ boss"
    } else {
        ""
    };
    format!("{} · {} mats{boss}", summary.weapon, summary.materials)
}

pub fn handle_title_profile_card_clicks(
    rename: Res<ProfileRenameState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut interactions: TitleProfileCardClicks,
    mut run: ProfileRunGate,
) {
    if rename.active.is_some() {
        return;
    }

    for (interaction, card) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if run.begin(card.index) {
            next_state.set(GameState::Overworld);
        }
    }
}

pub fn handle_title_profile_rename_clicks(
    mut rename: ResMut<ProfileRenameState>,
    picker: Res<ProfilePicker>,
    mut interactions: TitleProfileRenameClicks,
) {
    for (interaction, button) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let summary = &picker.cards[button.index as usize];
        rename.active = Some(ProfileRenameSession {
            index: button.index,
            buffer: summary.name.clone(),
        });
    }
}

pub fn handle_profile_rename_input(
    actions: Res<ActionState>,
    mut rename: ResMut<ProfileRenameState>,
    mut picker: ResMut<ProfilePicker>,
    active: Res<ActiveProfile>,
    mut profile: ResMut<PlayerProfile>,
    mut autosave_block: ResMut<ProfileAutosaveBlock>,
) {
    let Some(session) = rename.active.as_mut() else {
        return;
    };

    if actions.just_pressed(Action::Cancel) {
        rename.active = None;
        return;
    }

    if actions.just_pressed(Action::Confirm) {
        let index = session.index;
        let buffer = session.buffer.clone();
        if let Some(saved) = rename_profile_on_disk(index, buffer) {
            if active.index() == index {
                profile.name = saved.name;
            }
        }
        *picker = ProfilePicker::refresh(&mut autosave_block);
        rename.active = None;
        return;
    }

    if actions.just_pressed(Action::EraseCharacter) {
        session.buffer.pop();
        return;
    }

    for ch in actions.typed_characters().chars() {
        let mut next = session.buffer.clone();
        next.push(ch);
        session.buffer = sanitize_profile_name(next);
    }
}

pub fn handle_title_profile_keyboard_shortcuts(
    actions: Res<ActionState>,
    rename: Res<ProfileRenameState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut run: ProfileRunGate,
) {
    if rename.active.is_some() {
        return;
    }

    let Some(index) = profile_shortcut(&actions) else {
        return;
    };

    if run.begin(index) {
        next_state.set(GameState::Overworld);
    }
}

pub fn sync_title_profile_cards(
    picker: Res<ProfilePicker>,
    rename: Res<ProfileRenameState>,
    mut cards: Query<
        (
            &TitleProfileCard,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<TitleProfileCard>,
    >,
    mut names: Query<
        (&TitleProfileCardName, &mut Text, &mut TextColor),
        Without<TitleProfileCardDetails>,
    >,
    mut details: Query<(&TitleProfileCardDetails, &mut Text), Without<TitleProfileCardName>>,
    mut rename_buttons: Query<
        (
            &TitleProfileRenameButton,
            &Interaction,
            &mut BackgroundColor,
        ),
        Without<TitleProfileCard>,
    >,
) {
    for (card, interaction, mut bg, mut border) in &mut cards {
        apply_card_highlight(
            card.index,
            rename.active.as_ref(),
            interaction,
            &mut bg,
            &mut border,
        );
    }

    for (name_label, mut text, mut color) in &mut names {
        let summary = &picker.cards[name_label.index as usize];
        let renaming = rename
            .active
            .as_ref()
            .is_some_and(|session| session.index == name_label.index);

        if renaming {
            let buffer = &rename.active.as_ref().unwrap().buffer;
            text.0 = if buffer.is_empty() {
                "▌".to_string()
            } else {
                format!("{buffer}▌")
            };
            color.0 = Color::srgb(0.98, 0.94, 0.62);
        } else {
            text.0 = summary.name.clone();
            color.0 = Color::srgb(0.95, 0.92, 0.55);
        }
    }

    for (details_label, mut text) in &mut details {
        let summary = &picker.cards[details_label.index as usize];
        text.0 = card_details(summary);
    }

    for (button, interaction, mut bg) in &mut rename_buttons {
        apply_rename_highlight(button.index, rename.active.as_ref(), interaction, &mut bg);
    }
}

fn apply_card_highlight(
    index: u8,
    rename: Option<&ProfileRenameSession>,
    interaction: &Interaction,
    bg: &mut BackgroundColor,
    border: &mut BorderColor,
) {
    let renaming = rename.is_some_and(|session| session.index == index);

    if renaming {
        bg.0 = Color::srgba(0.16, 0.18, 0.28, 0.98);
        border.0 = Color::srgb(0.95, 0.82, 0.35);
    } else if matches!(*interaction, Interaction::Hovered | Interaction::Pressed) {
        bg.0 = Color::srgba(0.14, 0.16, 0.24, 0.98);
        border.0 = Color::srgb(0.55, 0.78, 0.95);
    } else {
        bg.0 = Color::srgba(0.1, 0.11, 0.16, 0.95);
        border.0 = Color::srgba(0.3, 0.34, 0.42, 0.9);
    }
}

fn apply_rename_highlight(
    index: u8,
    rename: Option<&ProfileRenameSession>,
    interaction: &Interaction,
    bg: &mut BackgroundColor,
) {
    let renaming = rename.is_some_and(|session| session.index == index);

    if renaming {
        bg.0 = Color::srgba(0.22, 0.24, 0.32, 0.98);
    } else if matches!(*interaction, Interaction::Hovered | Interaction::Pressed) {
        bg.0 = Color::srgba(0.2, 0.22, 0.3, 0.98);
    } else {
        bg.0 = Color::srgba(0.14, 0.16, 0.22, 0.95);
    }
}

fn profile_shortcut(actions: &ActionState) -> Option<u8> {
    actions
        .just_pressed_actions()
        .iter()
        .copied()
        .find_map(|action| match action {
            Action::SelectProfile(index) => Some(index),
            _ => None,
        })
}

#[cfg(test)]
mod rename_input_tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;

    use crate::core::{ActiveProfile, PlayerProfile, ProfileAutosaveBlock};
    use crate::input::action_state_after_pressing;

    use super::{
        handle_profile_rename_input, profile_shortcut, ProfilePicker, ProfileRenameSession,
        ProfileRenameState,
    };

    #[test]
    fn digit_order_still_picks_the_first_profile_shortcut() {
        assert_eq!(
            profile_shortcut(&action_state_after_pressing(&[KeyCode::Digit1], &[])),
            Some(0)
        );
        assert_eq!(
            profile_shortcut(&action_state_after_pressing(
                &[KeyCode::Digit3, KeyCode::Digit1],
                &[]
            )),
            Some(0)
        );
        assert_eq!(
            profile_shortcut(&action_state_after_pressing(&[KeyCode::KeyE], &[])),
            None
        );
    }

    fn rename_buffer_after(keys: &[KeyCode], starting: &str) -> Option<String> {
        let mut app = App::new();
        app.insert_resource(action_state_after_pressing(keys, &[]))
            .insert_resource(ProfileRenameState {
                active: Some(ProfileRenameSession {
                    index: 0,
                    buffer: starting.to_string(),
                }),
            })
            .init_resource::<ProfilePicker>()
            .init_resource::<ActiveProfile>()
            .init_resource::<PlayerProfile>()
            .init_resource::<ProfileAutosaveBlock>();
        app.world_mut()
            .run_system_once(handle_profile_rename_input)
            .unwrap();
        app.world()
            .resource::<ProfileRenameState>()
            .active
            .as_ref()
            .map(|session| session.buffer.clone())
    }

    #[test]
    fn rename_still_types_spaces_and_letters_and_cancels_on_escape() {
        assert_eq!(
            rename_buffer_after(&[KeyCode::KeyA], "R"),
            Some("Ra".to_string())
        );
        assert_eq!(
            rename_buffer_after(&[KeyCode::Space], ""),
            Some(String::new())
        );
        assert_eq!(
            rename_buffer_after(&[KeyCode::Minus], "R"),
            Some("R-".to_string())
        );
        assert_eq!(
            rename_buffer_after(&[KeyCode::Backspace], "Ra"),
            Some("R".to_string())
        );
        assert_eq!(rename_buffer_after(&[KeyCode::Escape], "Ra"), None);
    }
}
