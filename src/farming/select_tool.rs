use bevy::prelude::*;

use crate::input::ActionState;

use super::tools::{EquippedTool, HomesteadTool};

pub fn select_homestead_tool(actions: Res<ActionState>, mut equipped: ResMut<EquippedTool>) {
    let Some(tool) = equipped_tool_just_selected(&actions) else {
        return;
    };
    if equipped.0 == tool {
        return;
    }
    equipped.0 = tool;
    info!("Equipped {}.", tool.label());
}

fn equipped_tool_just_selected(actions: &ActionState) -> Option<HomesteadTool> {
    actions
        .just_pressed_actions()
        .iter()
        .copied()
        .find_map(HomesteadTool::from_action)
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;

    use crate::input::action_state_after_pressing;

    use super::{select_homestead_tool, EquippedTool};
    use crate::farming::tools::HomesteadTool;

    fn equipped_after(keys: &[KeyCode]) -> HomesteadTool {
        let mut app = App::new();
        app.insert_resource(action_state_after_pressing(keys, &[]))
            .init_resource::<EquippedTool>();
        app.world_mut()
            .run_system_once(select_homestead_tool)
            .unwrap();
        app.world().resource::<EquippedTool>().0
    }

    #[test]
    fn digit_and_numpad_still_equip_the_same_tools() {
        assert_eq!(equipped_after(&[KeyCode::Digit1]), HomesteadTool::Hoe);
        assert_eq!(
            equipped_after(&[KeyCode::Numpad2]),
            HomesteadTool::WateringCan
        );
        assert_eq!(equipped_after(&[KeyCode::Digit3]), HomesteadTool::Seeds);
        assert_eq!(equipped_after(&[KeyCode::Numpad4]), HomesteadTool::Hand);
    }

    #[test]
    fn an_earlier_digit_key_wins_when_a_numpad_key_arrives_with_it() {
        assert_eq!(
            equipped_after(&[KeyCode::Numpad1, KeyCode::Digit2]),
            HomesteadTool::WateringCan
        );
    }
}
