use bevy::prelude::*;

pub(super) const ACTION_BAR_SLOT_COUNT: usize = 8;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActionBar {
    selected_slot: usize,
}

impl ActionBar {
    pub fn select_slot(&mut self, slot_index: usize) -> Result<(), ActionBarSlotOutOfRange> {
        if slot_index_is_out_of_range(slot_index) {
            return Err(ActionBarSlotOutOfRange);
        }
        self.selected_slot = slot_index;
        Ok(())
    }

    pub fn selected_slot(self) -> usize {
        self.selected_slot
    }
}

fn slot_index_is_out_of_range(slot_index: usize) -> bool {
    slot_index >= ACTION_BAR_SLOT_COUNT
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionBarSlotOutOfRange;

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::{ActionBar, ActionBarSlotOutOfRange, ACTION_BAR_SLOT_COUNT};
    use crate::farming::FarmingPlugin;

    #[test]
    fn the_action_bar_starts_on_slot_zero() {
        let action_bar = ActionBar::default();
        assert_eq!(action_bar.selected_slot(), 0);
    }

    #[test]
    fn selecting_every_slot_from_zero_through_seven_stores_that_slot() {
        let mut action_bar = ActionBar::default();
        for slot_index in 0..ACTION_BAR_SLOT_COUNT {
            assert_eq!(action_bar.select_slot(slot_index), Ok(()));
            assert_eq!(action_bar.selected_slot(), slot_index);
        }
    }

    #[test]
    fn selecting_slot_eight_or_a_huge_index_returns_an_error_and_leaves_the_previous_slot() {
        let mut action_bar = ActionBar::default();
        assert_eq!(action_bar.select_slot(8), Err(ActionBarSlotOutOfRange));
        assert_eq!(action_bar.selected_slot(), 0);

        action_bar.select_slot(3).unwrap();
        assert_eq!(action_bar.select_slot(8), Err(ActionBarSlotOutOfRange));
        assert_eq!(action_bar.selected_slot(), 3);
        assert_eq!(
            action_bar.select_slot(usize::MAX),
            Err(ActionBarSlotOutOfRange)
        );
        assert_eq!(action_bar.selected_slot(), 3);
    }

    #[test]
    fn an_app_with_the_farming_plugin_starts_on_slot_zero_and_keeps_a_rejected_slot() {
        let mut app = app_with_farming_plugin();
        assert_eq!(app.world().resource::<ActionBar>().selected_slot(), 0);

        for slot_index in 0..ACTION_BAR_SLOT_COUNT {
            let selected = app
                .world_mut()
                .resource_mut::<ActionBar>()
                .select_slot(slot_index);
            assert_eq!(selected, Ok(()));
            assert_eq!(
                app.world().resource::<ActionBar>().selected_slot(),
                slot_index
            );
        }

        let rejected = app.world_mut().resource_mut::<ActionBar>().select_slot(8);
        assert_eq!(rejected, Err(ActionBarSlotOutOfRange));
        assert_eq!(app.world().resource::<ActionBar>().selected_slot(), 7);

        let rejected = app
            .world_mut()
            .resource_mut::<ActionBar>()
            .select_slot(usize::MAX);
        assert_eq!(rejected, Err(ActionBarSlotOutOfRange));
        assert_eq!(app.world().resource::<ActionBar>().selected_slot(), 7);
    }

    fn app_with_farming_plugin() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, FarmingPlugin));
        app
    }
}
