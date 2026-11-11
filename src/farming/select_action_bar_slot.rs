use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use super::action_bar::ACTION_BAR_SLOT_COUNT;
use super::tools::{EquippedTool, HomesteadTool};
use super::{ActionBar, ActionBarSlotOutOfRange};

pub(super) fn select_action_bar_slot_from_number_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut action_bar: ResMut<ActionBar>,
    mut equipped_tool: ResMut<EquippedTool>,
) {
    let Some(slot_index) = just_pressed_number_key_slot(&keyboard) else {
        return;
    };
    select_action_bar_slot(&mut action_bar, &mut equipped_tool, slot_index);
}

fn just_pressed_number_key_slot(keyboard: &ButtonInput<KeyCode>) -> Option<usize> {
    number_key_slots()
        .into_iter()
        .find(|(key, _slot_index)| keyboard.just_pressed(*key))
        .map(|(_key, slot_index)| slot_index)
}

fn number_key_slots() -> [(KeyCode, usize); 16] {
    [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Numpad1, 0),
        (KeyCode::Numpad2, 1),
        (KeyCode::Numpad3, 2),
        (KeyCode::Numpad4, 3),
        (KeyCode::Numpad5, 4),
        (KeyCode::Numpad6, 5),
        (KeyCode::Numpad7, 6),
        (KeyCode::Numpad8, 7),
    ]
}

fn select_action_bar_slot(
    action_bar: &mut ActionBar,
    equipped_tool: &mut EquippedTool,
    slot_index: usize,
) {
    if try_store_selected_slot(action_bar, slot_index).is_err() {
        return;
    }
    equip_tool_for_slot(equipped_tool, slot_index);
}

fn try_store_selected_slot(
    action_bar: &mut ActionBar,
    slot_index: usize,
) -> Result<(), ActionBarSlotOutOfRange> {
    action_bar.select_slot(slot_index)
}

fn equip_tool_for_slot(equipped_tool: &mut EquippedTool, slot_index: usize) {
    let Some(tool) = homestead_tool_for_slot(slot_index) else {
        return;
    };
    if equipped_tool.0 == tool {
        return;
    }
    equipped_tool.0 = tool;
    info!("Equipped {}.", tool.label());
}

fn homestead_tool_for_slot(slot_index: usize) -> Option<HomesteadTool> {
    HomesteadTool::ALL.get(slot_index).copied()
}

pub(super) fn cycle_action_bar_slot_from_mouse_wheel(
    mut mouse_wheels: EventReader<MouseWheel>,
    mut action_bar: ResMut<ActionBar>,
    mut equipped_tool: ResMut<EquippedTool>,
) {
    let slots_moved = slots_moved_by_mouse_wheel(mouse_wheels.read());
    if slots_moved == 0 {
        return;
    }
    let slot_index = slot_index_after_steps(action_bar.selected_slot(), slots_moved);
    select_action_bar_slot(&mut action_bar, &mut equipped_tool, slot_index);
}

fn slots_moved_by_mouse_wheel<'a>(mouse_wheels: impl Iterator<Item = &'a MouseWheel>) -> i32 {
    mouse_wheels.map(slots_moved_by_one_wheel_event).sum()
}

fn slots_moved_by_one_wheel_event(mouse_wheel: &MouseWheel) -> i32 {
    if mouse_wheel.y < 0.0 {
        1
    } else if mouse_wheel.y > 0.0 {
        -1
    } else {
        0
    }
}

fn slot_index_after_steps(selected_slot: usize, slots_moved: i32) -> usize {
    let slot_count = ACTION_BAR_SLOT_COUNT as i32;
    let moved_slot = selected_slot as i32 + slots_moved;
    moved_slot.rem_euclid(slot_count) as usize
}

#[cfg(test)]
mod tests {
    use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use super::{cycle_action_bar_slot_from_mouse_wheel, select_action_bar_slot_from_number_keys};
    use crate::core::GameState;
    use crate::farming::action_bar::ActionBar;
    use crate::farming::tools::{EquippedTool, HomesteadTool};
    use crate::input::InputPlugin;
    use crate::ui::forge_window::{forge_closed, ForgeWindowOpen};
    use crate::ui::inventory_window::{inventory_closed, InventoryWindowOpen};

    #[test]
    fn number_keys_and_the_numpad_select_slots_zero_through_seven() {
        let mut app = overworld_selection_app();
        for (slot_index, key) in digit_keys().into_iter().enumerate() {
            press_key(&mut app, key);
            assert_eq!(selected_slot(&app), slot_index);
        }
        for (slot_index, key) in numpad_keys().into_iter().enumerate() {
            press_key(&mut app, key);
            assert_eq!(selected_slot(&app), slot_index);
        }
    }

    #[test]
    fn number_keys_one_through_four_equip_the_same_tools() {
        assert_eq!(
            tool_after_pressing(HomesteadTool::Hand, KeyCode::Digit1),
            HomesteadTool::Hoe
        );
        assert_eq!(
            tool_after_pressing(HomesteadTool::Hand, KeyCode::Numpad1),
            HomesteadTool::Hoe
        );
        assert_eq!(
            tool_after_pressing(HomesteadTool::Hoe, KeyCode::Digit2),
            HomesteadTool::WateringCan
        );
        assert_eq!(
            tool_after_pressing(HomesteadTool::Hoe, KeyCode::Numpad2),
            HomesteadTool::WateringCan
        );
        assert_eq!(
            tool_after_pressing(HomesteadTool::Hoe, KeyCode::Digit3),
            HomesteadTool::Seeds
        );
        assert_eq!(
            tool_after_pressing(HomesteadTool::Hoe, KeyCode::Numpad3),
            HomesteadTool::Seeds
        );
        assert_eq!(
            tool_after_pressing(HomesteadTool::Seeds, KeyCode::Digit4),
            HomesteadTool::Hand
        );
        assert_eq!(
            tool_after_pressing(HomesteadTool::Seeds, KeyCode::Numpad4),
            HomesteadTool::Hand
        );
    }

    #[test]
    fn an_earlier_digit_key_wins_over_the_numpad() {
        let mut app = overworld_selection_app();
        set_equipped_tool(&mut app, HomesteadTool::Hand);
        press_keys(&mut app, &[KeyCode::Numpad1, KeyCode::Digit2]);
        assert_eq!(selected_slot(&app), 1);
        assert_eq!(equipped_tool(&app), HomesteadTool::WateringCan);
    }

    #[test]
    fn selecting_slots_four_through_seven_leaves_the_equipped_tool() {
        for (slot_index, key) in empty_digit_keys().into_iter().enumerate() {
            let mut app = overworld_selection_app();
            set_equipped_tool(&mut app, HomesteadTool::Seeds);
            press_key(&mut app, key);
            assert_eq!(selected_slot(&app), slot_index + 4);
            assert_eq!(equipped_tool(&app), HomesteadTool::Seeds);
        }
        for (slot_index, key) in empty_numpad_keys().into_iter().enumerate() {
            let mut app = overworld_selection_app();
            set_equipped_tool(&mut app, HomesteadTool::Seeds);
            press_key(&mut app, key);
            assert_eq!(selected_slot(&app), slot_index + 4);
            assert_eq!(equipped_tool(&app), HomesteadTool::Seeds);
        }
    }

    #[test]
    fn the_mouse_wheel_wraps_from_the_last_slot_to_the_first_and_back() {
        let mut app = overworld_selection_app();
        set_equipped_tool(&mut app, HomesteadTool::Hand);
        select_slot_directly(&mut app, 7);
        scroll_mouse_wheel_down(&mut app);
        assert_eq!(selected_slot(&app), 0);
        assert_eq!(equipped_tool(&app), HomesteadTool::Hoe);

        scroll_mouse_wheel_up(&mut app);
        assert_eq!(selected_slot(&app), 7);
        assert_eq!(equipped_tool(&app), HomesteadTool::Hoe);
    }

    #[test]
    fn the_mouse_wheel_equips_the_tool_when_it_lands_on_slots_zero_through_three() {
        let mut app = overworld_selection_app();
        set_equipped_tool(&mut app, HomesteadTool::Seeds);
        select_slot_directly(&mut app, 4);

        scroll_mouse_wheel_up(&mut app);
        assert_eq!(selected_slot(&app), 3);
        assert_eq!(equipped_tool(&app), HomesteadTool::Hand);

        scroll_mouse_wheel_up(&mut app);
        assert_eq!(selected_slot(&app), 2);
        assert_eq!(equipped_tool(&app), HomesteadTool::Seeds);

        scroll_mouse_wheel_up(&mut app);
        assert_eq!(selected_slot(&app), 1);
        assert_eq!(equipped_tool(&app), HomesteadTool::WateringCan);

        scroll_mouse_wheel_up(&mut app);
        assert_eq!(selected_slot(&app), 0);
        assert_eq!(equipped_tool(&app), HomesteadTool::Hoe);
    }

    #[test]
    fn the_mouse_wheel_leaves_the_equipped_tool_on_an_empty_slot() {
        let mut app = overworld_selection_app();
        set_equipped_tool(&mut app, HomesteadTool::Seeds);
        select_slot_directly(&mut app, 3);
        scroll_mouse_wheel_down(&mut app);
        assert_eq!(selected_slot(&app), 4);
        assert_eq!(equipped_tool(&app), HomesteadTool::Seeds);
    }

    #[test]
    fn number_keys_leave_the_slot_while_the_inventory_is_open() {
        let mut app = overworld_selection_app();
        open_inventory(&mut app);
        press_key(&mut app, KeyCode::Digit2);
        assert_eq!(selected_slot(&app), 0);
        assert_eq!(equipped_tool(&app), HomesteadTool::Hoe);
    }

    #[test]
    fn the_mouse_wheel_leaves_the_slot_while_the_forge_is_open() {
        let mut app = overworld_selection_app();
        open_forge(&mut app);
        scroll_mouse_wheel_down(&mut app);
        assert_eq!(selected_slot(&app), 0);
        assert_eq!(equipped_tool(&app), HomesteadTool::Hoe);
    }

    #[test]
    fn number_keys_leave_the_slot_outside_the_overworld() {
        let mut app = title_selection_app();
        press_key(&mut app, KeyCode::Digit3);
        assert_eq!(selected_slot(&app), 0);
        assert_eq!(equipped_tool(&app), HomesteadTool::Hoe);
    }

    fn title_selection_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin, InputPlugin))
            .init_state::<GameState>()
            .init_resource::<InventoryWindowOpen>()
            .init_resource::<ForgeWindowOpen>()
            .init_resource::<ActionBar>()
            .init_resource::<EquippedTool>()
            .add_event::<MouseWheel>()
            .add_systems(
                Update,
                (
                    select_action_bar_slot_from_number_keys,
                    cycle_action_bar_slot_from_mouse_wheel,
                )
                    .chain()
                    .run_if(in_state(GameState::Overworld))
                    .run_if(inventory_closed)
                    .run_if(forge_closed),
            );
        app
    }

    fn overworld_selection_app() -> App {
        let mut app = title_selection_app();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Overworld);
        app.update();
        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Overworld
        );
        app
    }

    fn digit_keys() -> [KeyCode; 8] {
        [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
        ]
    }

    fn numpad_keys() -> [KeyCode; 8] {
        [
            KeyCode::Numpad1,
            KeyCode::Numpad2,
            KeyCode::Numpad3,
            KeyCode::Numpad4,
            KeyCode::Numpad5,
            KeyCode::Numpad6,
            KeyCode::Numpad7,
            KeyCode::Numpad8,
        ]
    }

    fn empty_digit_keys() -> [KeyCode; 4] {
        [
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
        ]
    }

    fn empty_numpad_keys() -> [KeyCode; 4] {
        [
            KeyCode::Numpad5,
            KeyCode::Numpad6,
            KeyCode::Numpad7,
            KeyCode::Numpad8,
        ]
    }

    fn tool_after_pressing(start: HomesteadTool, key: KeyCode) -> HomesteadTool {
        let mut app = overworld_selection_app();
        set_equipped_tool(&mut app, start);
        press_key(&mut app, key);
        equipped_tool(&app)
    }

    fn press_key(app: &mut App, key: KeyCode) {
        press_keys(app, &[key]);
    }

    fn press_keys(app: &mut App, keys: &[KeyCode]) {
        {
            let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            for key in keys {
                keyboard.press(*key);
            }
        }
        app.update();
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        for key in keys {
            keyboard.release(*key);
        }
        keyboard.clear();
    }

    fn scroll_mouse_wheel_down(app: &mut App) {
        send_vertical_mouse_wheel(app, -1.0);
    }

    fn scroll_mouse_wheel_up(app: &mut App) {
        send_vertical_mouse_wheel(app, 1.0);
    }

    fn send_vertical_mouse_wheel(app: &mut App, vertical: f32) {
        let sent = app.world_mut().send_event(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: vertical,
            window: Entity::PLACEHOLDER,
        });
        assert!(sent.is_some());
        app.update();
    }

    fn select_slot_directly(app: &mut App, slot_index: usize) {
        app.world_mut()
            .resource_mut::<ActionBar>()
            .select_slot(slot_index)
            .unwrap();
    }

    fn set_equipped_tool(app: &mut App, tool: HomesteadTool) {
        app.world_mut().resource_mut::<EquippedTool>().0 = tool;
    }

    fn open_inventory(app: &mut App) {
        app.world_mut().resource_mut::<InventoryWindowOpen>().0 = true;
    }

    fn open_forge(app: &mut App) {
        app.world_mut().resource_mut::<ForgeWindowOpen>().0 = true;
    }

    fn selected_slot(app: &App) -> usize {
        app.world().resource::<ActionBar>().selected_slot()
    }

    fn equipped_tool(app: &App) -> HomesteadTool {
        app.world().resource::<EquippedTool>().0
    }
}
