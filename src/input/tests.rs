use std::collections::BTreeSet;

use bevy::prelude::*;

use super::{action_state_after_pressing, Action, ActionState, Binding, InputMap, InputPlugin};

#[test]
fn default_bindings_match_the_controls_gameplay_already_used() {
    assert_eq!(
        InputMap::default().bindings(),
        expected_default_bindings().as_slice()
    );
}

#[test]
fn each_default_binding_just_presses_its_action() {
    for (action, binding) in expected_default_bindings() {
        let state = press_binding(binding);
        assert!(
            state.just_pressed(action),
            "{action:?} should fire from {binding:?}"
        );
        assert!(
            state.pressed(action),
            "{action:?} should be held when {binding:?} is pressed"
        );
    }
}

#[test]
fn shared_keys_still_fire_every_action_they_used_to() {
    assert_eq!(
        just_pressed_set(&action_state_after_pressing(&[KeyCode::Digit1], &[])),
        BTreeSet::from([
            Action::EquipHoe,
            Action::SelectProfile(0),
            Action::SkillSlot(0),
        ])
    );
    assert_eq!(
        just_pressed_set(&action_state_after_pressing(&[KeyCode::Numpad1], &[])),
        BTreeSet::from([Action::EquipHoe])
    );
    assert_eq!(
        just_pressed_set(&action_state_after_pressing(&[KeyCode::ArrowUp], &[])),
        BTreeSet::from([Action::MoveUp, Action::PreviousRecipe])
    );
    assert_eq!(
        just_pressed_set(&action_state_after_pressing(&[KeyCode::Space], &[])),
        BTreeSet::from([Action::Jump, Action::UseTool])
    );
    assert_eq!(
        just_pressed_set(&action_state_after_pressing(&[], &[MouseButton::Left])),
        BTreeSet::from([Action::UseTool, Action::PrimaryButton])
    );
}

#[test]
fn tool_keys_keep_digit_row_priority_over_the_numpad() {
    let state = action_state_after_pressing(&[KeyCode::Digit2, KeyCode::Numpad1], &[]);
    let tools: Vec<_> = state
        .just_pressed_actions()
        .iter()
        .copied()
        .filter(|action| {
            matches!(
                action,
                Action::EquipHoe
                    | Action::EquipWateringCan
                    | Action::EquipSeeds
                    | Action::EquipHand
            )
        })
        .collect();
    assert_eq!(
        tools,
        vec![Action::EquipWateringCan, Action::EquipHoe]
    );
}

#[test]
fn profile_shortcuts_keep_digit_order() {
    let state = action_state_after_pressing(&[KeyCode::Digit3, KeyCode::Digit1], &[]);
    let profiles: Vec<_> = state
        .just_pressed_actions()
        .iter()
        .copied()
        .filter_map(|action| match action {
            Action::SelectProfile(index) => Some(index),
            _ => None,
        })
        .collect();
    assert_eq!(profiles, vec![0, 2]);
}

#[test]
fn held_binding_stays_pressed_without_firing_again() {
    let state = hold_key(KeyCode::KeyE);
    assert!(state.pressed(Action::Interact));
    assert!(!state.just_pressed(Action::Interact));
    assert!(state.typed_characters().is_empty());
}

#[test]
fn release_reports_just_released_and_clears_pressed() {
    let state = release_key(KeyCode::KeyE);
    assert!(state.just_released(Action::Interact));
    assert!(!state.pressed(Action::Interact));
    assert!(!state.just_pressed(Action::Interact));
}

#[test]
fn releasing_one_move_key_while_the_other_is_held_keeps_moving() {
    let mut keyboard = ButtonInput::default();
    keyboard.press(KeyCode::KeyA);
    keyboard.press(KeyCode::ArrowLeft);
    keyboard.clear();
    keyboard.release(KeyCode::KeyA);
    let state = ActionState::from_devices(
        &InputMap::default(),
        &keyboard,
        &ButtonInput::<MouseButton>::default(),
    );
    assert!(state.pressed(Action::MoveLeft));
    assert!(state.just_released(Action::MoveLeft));
    assert!(!state.just_pressed(Action::MoveLeft));
}

#[test]
fn click_while_space_is_held_still_just_presses_use_tool() {
    let mut keyboard = ButtonInput::default();
    keyboard.press(KeyCode::Space);
    keyboard.clear();
    let mut mouse = ButtonInput::default();
    mouse.press(MouseButton::Left);
    let state = ActionState::from_devices(&InputMap::default(), &keyboard, &mouse);

    assert!(state.pressed(Action::Jump));
    assert!(!state.just_pressed(Action::Jump));
    assert!(state.pressed(Action::UseTool));
    assert!(state.just_pressed(Action::UseTool));
    assert!(state.just_pressed(Action::PrimaryButton));
    assert!(!state.just_pressed(Action::Confirm));
}

#[test]
fn mouse_release_just_releases_the_primary_button() {
    let mut mouse = ButtonInput::default();
    mouse.press(MouseButton::Left);
    mouse.clear();
    mouse.release(MouseButton::Left);
    let state = ActionState::from_devices(
        &InputMap::default(),
        &ButtonInput::<KeyCode>::default(),
        &mouse,
    );
    assert!(state.just_released(Action::PrimaryButton));
    assert!(state.just_released(Action::UseTool));
    assert!(!state.pressed(Action::PrimaryButton));
}

#[test]
fn space_types_a_space_and_does_not_confirm() {
    let state = action_state_after_pressing(&[KeyCode::Space], &[]);
    assert_eq!(state.typed_characters(), " ");
    assert!(!state.just_pressed(Action::Confirm));
    assert!(state.just_pressed(Action::Jump));
}

#[test]
fn enter_confirms_without_typing() {
    let state = action_state_after_pressing(&[KeyCode::Enter], &[]);
    assert!(state.just_pressed(Action::Confirm));
    assert!(state.typed_characters().is_empty());
    assert!(!state.just_pressed(Action::Jump));
}

#[test]
fn typed_characters_match_the_profile_rename_alphabet() {
    let letters = [
        (KeyCode::KeyA, "a"),
        (KeyCode::KeyB, "b"),
        (KeyCode::KeyC, "c"),
        (KeyCode::KeyD, "d"),
        (KeyCode::KeyE, "e"),
        (KeyCode::KeyF, "f"),
        (KeyCode::KeyG, "g"),
        (KeyCode::KeyH, "h"),
        (KeyCode::KeyI, "i"),
        (KeyCode::KeyJ, "j"),
        (KeyCode::KeyK, "k"),
        (KeyCode::KeyL, "l"),
        (KeyCode::KeyM, "m"),
        (KeyCode::KeyN, "n"),
        (KeyCode::KeyO, "o"),
        (KeyCode::KeyP, "p"),
        (KeyCode::KeyQ, "q"),
        (KeyCode::KeyR, "r"),
        (KeyCode::KeyS, "s"),
        (KeyCode::KeyT, "t"),
        (KeyCode::KeyU, "u"),
        (KeyCode::KeyV, "v"),
        (KeyCode::KeyW, "w"),
        (KeyCode::KeyX, "x"),
        (KeyCode::KeyY, "y"),
        (KeyCode::KeyZ, "z"),
        (KeyCode::Space, " "),
        (KeyCode::Minus, "-"),
        (KeyCode::Period, "."),
        (KeyCode::Digit0, "0"),
        (KeyCode::Digit1, "1"),
        (KeyCode::Digit2, "2"),
        (KeyCode::Digit3, "3"),
        (KeyCode::Digit4, "4"),
        (KeyCode::Digit5, "5"),
        (KeyCode::Digit6, "6"),
        (KeyCode::Digit7, "7"),
        (KeyCode::Digit8, "8"),
        (KeyCode::Digit9, "9"),
    ];
    for (key, expected) in letters {
        assert_eq!(
            action_state_after_pressing(&[key], &[]).typed_characters(),
            expected,
            "{key:?}"
        );
    }
}

#[test]
fn non_character_keys_do_not_type() {
    for key in [
        KeyCode::Escape,
        KeyCode::Enter,
        KeyCode::Backspace,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::Numpad1,
        KeyCode::BracketLeft,
        KeyCode::BracketRight,
    ] {
        assert!(
            action_state_after_pressing(&[key], &[]).typed_characters().is_empty(),
            "{key:?}"
        );
    }
}

#[test]
fn sync_system_copies_a_fresh_key_press_into_action_state() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyE);
    app.update();
    let actions = app.world().resource::<ActionState>();
    assert!(actions.just_pressed(Action::Interact));
    assert_eq!(actions.typed_characters(), "e");
}

fn press_binding(binding: Binding) -> ActionState {
    match binding {
        Binding::Key(key) => action_state_after_pressing(&[key], &[]),
        Binding::Mouse(button) => action_state_after_pressing(&[], &[button]),
    }
}

fn hold_key(key: KeyCode) -> ActionState {
    let mut keyboard = ButtonInput::default();
    keyboard.press(key);
    keyboard.clear();
    ActionState::from_devices(
        &InputMap::default(),
        &keyboard,
        &ButtonInput::<MouseButton>::default(),
    )
}

fn release_key(key: KeyCode) -> ActionState {
    let mut keyboard = ButtonInput::default();
    keyboard.press(key);
    keyboard.clear();
    keyboard.release(key);
    ActionState::from_devices(
        &InputMap::default(),
        &keyboard,
        &ButtonInput::<MouseButton>::default(),
    )
}

fn just_pressed_set(state: &ActionState) -> BTreeSet<Action> {
    state.just_pressed_actions().iter().copied().collect()
}

fn expected_default_bindings() -> Vec<(Action, Binding)> {
    let mut bindings = Vec::new();
    bindings.extend(movement_bindings());
    bindings.extend(gameplay_bindings());
    bindings.extend(menu_bindings());
    bindings.extend(tool_bindings());
    bindings.extend(profile_bindings());
    bindings.extend(skill_slot_bindings());
    bindings
}

fn movement_bindings() -> [(Action, Binding); 8] {
    [
        (Action::MoveUp, Binding::Key(KeyCode::KeyW)),
        (Action::MoveUp, Binding::Key(KeyCode::ArrowUp)),
        (Action::MoveDown, Binding::Key(KeyCode::KeyS)),
        (Action::MoveDown, Binding::Key(KeyCode::ArrowDown)),
        (Action::MoveLeft, Binding::Key(KeyCode::KeyA)),
        (Action::MoveLeft, Binding::Key(KeyCode::ArrowLeft)),
        (Action::MoveRight, Binding::Key(KeyCode::KeyD)),
        (Action::MoveRight, Binding::Key(KeyCode::ArrowRight)),
    ]
}

fn gameplay_bindings() -> [(Action, Binding); 6] {
    [
        (Action::Jump, Binding::Key(KeyCode::Space)),
        (Action::Interact, Binding::Key(KeyCode::KeyE)),
        (Action::UseTool, Binding::Key(KeyCode::Space)),
        (Action::UseTool, Binding::Mouse(MouseButton::Left)),
        (Action::PrimaryButton, Binding::Mouse(MouseButton::Left)),
        (Action::Craft, Binding::Key(KeyCode::KeyF)),
    ]
}

fn menu_bindings() -> [(Action, Binding); 9] {
    [
        (Action::Cancel, Binding::Key(KeyCode::Escape)),
        (Action::Confirm, Binding::Key(KeyCode::Enter)),
        (Action::ToggleInventory, Binding::Key(KeyCode::KeyI)),
        (Action::ReturnToHomestead, Binding::Key(KeyCode::KeyQ)),
        (Action::EraseCharacter, Binding::Key(KeyCode::Backspace)),
        (
            Action::EquipFirstStashedWeapon,
            Binding::Key(KeyCode::BracketRight),
        ),
        (
            Action::EquipLastStashedWeapon,
            Binding::Key(KeyCode::BracketLeft),
        ),
        (Action::PreviousRecipe, Binding::Key(KeyCode::ArrowUp)),
        (Action::NextRecipe, Binding::Key(KeyCode::ArrowDown)),
    ]
}

fn tool_bindings() -> [(Action, Binding); 8] {
    [
        (Action::EquipHoe, Binding::Key(KeyCode::Digit1)),
        (Action::EquipWateringCan, Binding::Key(KeyCode::Digit2)),
        (Action::EquipSeeds, Binding::Key(KeyCode::Digit3)),
        (Action::EquipHand, Binding::Key(KeyCode::Digit4)),
        (Action::EquipHoe, Binding::Key(KeyCode::Numpad1)),
        (Action::EquipWateringCan, Binding::Key(KeyCode::Numpad2)),
        (Action::EquipSeeds, Binding::Key(KeyCode::Numpad3)),
        (Action::EquipHand, Binding::Key(KeyCode::Numpad4)),
    ]
}

fn profile_bindings() -> [(Action, Binding); 3] {
    [
        (Action::SelectProfile(0), Binding::Key(KeyCode::Digit1)),
        (Action::SelectProfile(1), Binding::Key(KeyCode::Digit2)),
        (Action::SelectProfile(2), Binding::Key(KeyCode::Digit3)),
    ]
}

fn skill_slot_bindings() -> [(Action, Binding); 9] {
    [
        (Action::SkillSlot(0), Binding::Key(KeyCode::Digit1)),
        (Action::SkillSlot(1), Binding::Key(KeyCode::Digit2)),
        (Action::SkillSlot(2), Binding::Key(KeyCode::Digit3)),
        (Action::SkillSlot(3), Binding::Key(KeyCode::Digit4)),
        (Action::SkillSlot(4), Binding::Key(KeyCode::Digit5)),
        (Action::SkillSlot(5), Binding::Key(KeyCode::Digit6)),
        (Action::SkillSlot(6), Binding::Key(KeyCode::Digit7)),
        (Action::SkillSlot(7), Binding::Key(KeyCode::Digit8)),
        (Action::SkillSlot(8), Binding::Key(KeyCode::Digit9)),
    ]
}
