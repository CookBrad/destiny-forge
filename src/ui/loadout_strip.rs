use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;

use crate::combat::WeaponKind;
use crate::core::{GameState, ProfileDirty};
use crate::input::{Action, ActionState};
use crate::player::{weapon_kind_label, ArmorSet, Loadout};

const HINT: Color = Color::srgb(0.52, 0.5, 0.46);
const LABEL: Color = Color::srgb(0.9, 0.88, 0.84);
const BUTTON_BG: Color = Color::srgb(0.18, 0.14, 0.1);
const BUTTON_BORDER: Color = Color::srgb(0.42, 0.32, 0.18);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadoutSwapAccess {
    Hub,
    Locked,
}

impl LoadoutSwapAccess {
    pub fn from_game_state(state: &GameState) -> Self {
        match state {
            GameState::Overworld => Self::Hub,
            _ => Self::Locked,
        }
    }
}

#[derive(Component)]
struct EquippedWeaponLabel;

#[derive(Component)]
struct StashHintLabel;

#[derive(Component, Clone, Copy)]
pub struct StashWeaponButton {
    pub weapon: WeaponKind,
}

#[derive(Component, Clone, Copy)]
pub struct StashArmorButton {
    pub kind: ArmorSet,
}

pub fn spawn_loadout_strip(
    parent: &mut ChildBuilder<'_>,
    loadout: &Loadout,
    access: LoadoutSwapAccess,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            width: Val::Percent(100.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
            border: UiRect::top(Val::Px(2.0)),
            ..default()
        })
        .with_children(|strip| {
            spawn_equipped_row(strip, loadout);
            spawn_stash_row(strip, loadout, access);
        });
}

fn spawn_equipped_row(parent: &mut ChildBuilder<'_>, loadout: &Loadout) {
    parent.spawn((
        EquippedWeaponLabel,
        Text::new(format!("Equipped: {}", loadout.weapon_label())),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(LABEL),
    ));
}

fn spawn_stash_row(parent: &mut ChildBuilder<'_>, loadout: &Loadout, access: LoadoutSwapAccess) {
    parent.spawn((
        StashHintLabel,
        Text::new(stash_hint(loadout, access)),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(HINT),
    ));

    if access != LoadoutSwapAccess::Hub {
        return;
    }

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(6.0),
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|row| {
            for weapon in &loadout.stash.weapons {
                spawn_weapon_button(row, *weapon);
            }
            for kind in &loadout.stash.armor {
                spawn_armor_button(row, *kind);
            }
        });
}

fn spawn_weapon_button(parent: &mut ChildBuilder<'_>, weapon: WeaponKind) {
    spawn_stash_button(
        parent,
        StashWeaponButton { weapon },
        weapon_kind_label(weapon),
    );
}

fn spawn_armor_button(parent: &mut ChildBuilder<'_>, kind: ArmorSet) {
    spawn_stash_button(parent, StashArmorButton { kind }, kind.label());
}

fn spawn_stash_button(parent: &mut ChildBuilder<'_>, marker: impl Bundle, label: &'static str) {
    parent
        .spawn((
            Button,
            marker,
            Node {
                height: Val::Px(24.0),
                padding: UiRect::horizontal(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(BUTTON_BG),
            BorderColor(BUTTON_BORDER),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(LABEL),
            ));
        });
}

fn stash_hint(loadout: &Loadout, access: LoadoutSwapAccess) -> String {
    match access {
        LoadoutSwapAccess::Locked => "Swap loadout at the homestead.".to_string(),
        LoadoutSwapAccess::Hub
            if loadout.stash.weapons.is_empty() && loadout.stash.armor.is_empty() =>
        {
            "Stash is empty. Forging an alternate stores the old piece.".to_string()
        }
        LoadoutSwapAccess::Hub => {
            "Click a stashed piece to equip it  ·  [ ] cycle weapons".to_string()
        }
    }
}

pub(crate) type StashWeaponClicks<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static StashWeaponButton),
    (Changed<Interaction>, With<Button>),
>;

pub(crate) type StashArmorClicks<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static StashArmorButton),
    (Changed<Interaction>, With<Button>),
>;

pub fn handle_stash_weapon_click(
    interactions: StashWeaponClicks,
    mut loadout: ResMut<Loadout>,
    mut dirty: ResMut<ProfileDirty>,
) {
    for (interaction, button) in &interactions {
        if *interaction == Interaction::Pressed && loadout.swap_to_stashed_weapon(button.weapon) {
            dirty.mark();
        }
    }
}

pub fn handle_stash_armor_click(
    interactions: StashArmorClicks,
    mut loadout: ResMut<Loadout>,
    mut dirty: ResMut<ProfileDirty>,
) {
    for (interaction, button) in &interactions {
        if *interaction == Interaction::Pressed && loadout.swap_to_stashed_armor(button.kind) {
            dirty.mark();
        }
    }
}

enum StashCycle {
    First,
    Last,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn strip_app() -> App {
        let mut app = App::new();
        app.insert_resource(Loadout::default());
        app.insert_resource(ProfileDirty::default());
        app
    }

    fn armor_button_label(kind: ArmorSet) -> String {
        let mut app = App::new();
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                commands.spawn(Node::default()).with_children(|parent| {
                    spawn_armor_button(parent, kind);
                });
            })
            .unwrap();
        let mut labels = app.world_mut().query::<&Text>();
        labels.single(app.world()).0.clone()
    }

    #[test]
    fn an_armor_button_shows_the_piece_label() {
        assert_eq!(armor_button_label(ArmorSet::Helm), "Slime Helm");
        let mut app = App::new();
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                commands.spawn(Node::default()).with_children(|parent| {
                    spawn_armor_button(parent, ArmorSet::Helm);
                });
            })
            .unwrap();
        let kind = app
            .world_mut()
            .query::<&StashArmorButton>()
            .single(app.world())
            .kind;
        assert_eq!(kind, ArmorSet::Helm);
    }

    #[test]
    fn an_armor_button_for_greaves_does_not_use_the_helm_label() {
        assert_eq!(armor_button_label(ArmorSet::Greaves), "Slime Greaves");
    }

    #[test]
    fn an_armor_button_is_not_spawned_for_a_piece_that_was_not_requested() {
        let mut app = App::new();
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                commands.spawn(Node::default()).with_children(|parent| {
                    spawn_armor_button(parent, ArmorSet::Mail);
                });
            })
            .unwrap();
        let kinds: Vec<_> = app
            .world_mut()
            .query::<&StashArmorButton>()
            .iter(app.world())
            .map(|button| button.kind)
            .collect();
        assert_eq!(kinds, vec![ArmorSet::Mail]);
    }

    #[test]
    fn pressing_a_stashed_weapon_button_equips_that_weapon() {
        let mut app = strip_app();
        {
            let mut loadout = app.world_mut().resource_mut::<Loadout>();
            loadout.stash.weapons.push(WeaponKind::RustySpear);
        }
        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            StashWeaponButton {
                weapon: WeaponKind::RustySpear,
            },
        ));
        app.world_mut()
            .run_system_once(handle_stash_weapon_click)
            .unwrap();
        let loadout = app.world().resource::<Loadout>();
        assert_eq!(loadout.weapon, WeaponKind::RustySpear);
        assert_eq!(loadout.stash.weapons, vec![WeaponKind::RustySword]);
        assert!(app.world().resource::<ProfileDirty>().0);
    }

    #[test]
    fn hovering_a_stashed_weapon_button_does_not_equip_it() {
        let mut app = strip_app();
        {
            let mut loadout = app.world_mut().resource_mut::<Loadout>();
            loadout.stash.weapons.push(WeaponKind::RustySpear);
        }
        app.world_mut().spawn((
            Button,
            Interaction::Hovered,
            StashWeaponButton {
                weapon: WeaponKind::RustySpear,
            },
        ));
        app.world_mut()
            .run_system_once(handle_stash_weapon_click)
            .unwrap();
        let loadout = app.world().resource::<Loadout>();
        assert_eq!(loadout.weapon, WeaponKind::RustySword);
        assert_eq!(loadout.stash.weapons, vec![WeaponKind::RustySpear]);
        assert!(!app.world().resource::<ProfileDirty>().0);
    }

    #[test]
    fn pressing_a_weapon_button_that_is_not_stashed_leaves_the_loadout() {
        let mut app = strip_app();
        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            StashWeaponButton {
                weapon: WeaponKind::IronSword,
            },
        ));
        app.world_mut()
            .run_system_once(handle_stash_weapon_click)
            .unwrap();
        let loadout = app.world().resource::<Loadout>();
        assert_eq!(loadout.weapon, WeaponKind::RustySword);
        assert!(loadout.stash.weapons.is_empty());
        assert!(!app.world().resource::<ProfileDirty>().0);
    }

    #[test]
    fn pressing_a_stashed_armor_button_equips_that_piece() {
        let mut app = strip_app();
        {
            let mut loadout = app.world_mut().resource_mut::<Loadout>();
            loadout.stash.armor.push(ArmorSet::Helm);
        }
        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            StashArmorButton {
                kind: ArmorSet::Helm,
            },
        ));
        app.world_mut()
            .run_system_once(handle_stash_armor_click)
            .unwrap();
        let loadout = app.world().resource::<Loadout>();
        assert_eq!(loadout.armor.head, Some(ArmorSet::Helm));
        assert!(loadout.stash.armor.is_empty());
        assert!(app.world().resource::<ProfileDirty>().0);
    }

    #[test]
    fn hovering_a_stashed_armor_button_does_not_equip_it() {
        let mut app = strip_app();
        {
            let mut loadout = app.world_mut().resource_mut::<Loadout>();
            loadout.stash.armor.push(ArmorSet::Mail);
        }
        app.world_mut().spawn((
            Button,
            Interaction::Hovered,
            StashArmorButton {
                kind: ArmorSet::Mail,
            },
        ));
        app.world_mut()
            .run_system_once(handle_stash_armor_click)
            .unwrap();
        let loadout = app.world().resource::<Loadout>();
        assert_eq!(loadout.armor.chest, None);
        assert_eq!(loadout.stash.armor, vec![ArmorSet::Mail]);
        assert!(!app.world().resource::<ProfileDirty>().0);
    }

    #[test]
    fn pressing_an_armor_button_that_is_not_stashed_leaves_the_loadout() {
        let mut app = strip_app();
        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            StashArmorButton {
                kind: ArmorSet::Greaves,
            },
        ));
        app.world_mut()
            .run_system_once(handle_stash_armor_click)
            .unwrap();
        let loadout = app.world().resource::<Loadout>();
        assert_eq!(loadout.armor.legs, None);
        assert!(loadout.stash.armor.is_empty());
        assert!(!app.world().resource::<ProfileDirty>().0);
    }
}

pub fn handle_loadout_swap_keys(
    actions: Res<ActionState>,
    game: Res<State<GameState>>,
    mut loadout: ResMut<Loadout>,
    mut dirty: ResMut<ProfileDirty>,
) {
    if LoadoutSwapAccess::from_game_state(game.get()) != LoadoutSwapAccess::Hub {
        return;
    }

    let cycle = if actions.just_pressed(Action::EquipFirstStashedWeapon) {
        StashCycle::First
    } else if actions.just_pressed(Action::EquipLastStashedWeapon) {
        StashCycle::Last
    } else {
        return;
    };

    let Some(weapon) = cycled_stash_weapon(&loadout, cycle) else {
        return;
    };
    if loadout.swap_to_stashed_weapon(weapon) {
        dirty.mark();
    }
}

fn cycled_stash_weapon(loadout: &Loadout, cycle: StashCycle) -> Option<WeaponKind> {
    match cycle {
        StashCycle::First => loadout.stash.weapons.first().copied(),
        StashCycle::Last => loadout.stash.weapons.last().copied(),
    }
}
