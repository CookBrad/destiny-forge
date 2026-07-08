//! Floating tooltip when the player can interact with something nearby.

use bevy::prelude::*;

/// What the player can do with E (or hold-E). Higher priority wins if several are near.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptKind {
    Sleep,
    Carve,
    OpenForge,
    ClimbLadder,
    EnterDungeon,
}

impl PromptKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sleep => "E — Sleep",
            Self::Carve => "Hold E — Carve",
            Self::OpenForge => "E — Open forge",
            Self::ClimbLadder => "E — Climb ladder",
            Self::EnterDungeon => "E — Enter dungeon (costs time)",
        }
    }

    pub fn priority(self) -> u8 {
        match self {
            Self::Sleep => 50,
            Self::Carve => 40,
            Self::OpenForge => 30,
            Self::ClimbLadder => 20,
            Self::EnterDungeon => 10,
        }
    }
}

/// Pick the highest-priority prompt from candidates (pure, unit-testable).
pub fn best_prompt(candidates: &[PromptKind]) -> Option<PromptKind> {
    candidates
        .iter()
        .copied()
        .max_by_key(|kind| kind.priority())
}

#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub struct InteractionPrompt {
    pub kind: Option<PromptKind>,
}

impl InteractionPrompt {
    pub fn set(&mut self, kind: Option<PromptKind>) {
        self.kind = kind;
    }

    pub fn clear(&mut self) {
        self.kind = None;
    }

    pub fn label(&self) -> Option<&'static str> {
        self.kind.map(PromptKind::label)
    }
}

#[derive(Component)]
pub struct InteractionPromptRoot;

#[derive(Component)]
pub struct InteractionPromptLabel;

pub fn setup_interaction_prompt(mut commands: Commands) {
    commands
        .spawn((
            InteractionPromptRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(48.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    min_width: Val::Px(160.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.04, 0.03, 0.82)),
                BorderColor(Color::srgba(0.72, 0.62, 0.38, 0.9)),
            ))
            .with_children(|pill| {
                pill.spawn((
                    InteractionPromptLabel,
                    Text::new(""),
                    TextFont {
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.96, 0.92, 0.82)),
                ));
            });
        });
}

pub fn cleanup_interaction_prompt(
    mut commands: Commands,
    roots: Query<Entity, With<InteractionPromptRoot>>,
    mut prompt: ResMut<InteractionPrompt>,
) {
    prompt.clear();
    for entity in &roots {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub fn sync_interaction_prompt_ui(
    prompt: Res<InteractionPrompt>,
    mut roots: Query<&mut Node, With<InteractionPromptRoot>>,
    mut labels: Query<&mut Text, With<InteractionPromptLabel>>,
) {
    if !prompt.is_changed() {
        return;
    }

    let visible = prompt.kind.is_some();
    let label = prompt.label().unwrap_or("");

    for mut node in &mut roots {
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut text in &mut labels {
        text.0 = label.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highest_priority_wins() {
        let picked = best_prompt(&[PromptKind::EnterDungeon, PromptKind::Sleep, PromptKind::OpenForge]);
        assert_eq!(picked, Some(PromptKind::Sleep));
    }

    #[test]
    fn empty_candidates_yield_none() {
        assert_eq!(best_prompt(&[]), None);
    }

    #[test]
    fn carve_beats_ladder() {
        let picked = best_prompt(&[PromptKind::ClimbLadder, PromptKind::Carve]);
        assert_eq!(picked, Some(PromptKind::Carve));
    }
}
