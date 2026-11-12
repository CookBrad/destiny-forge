use bevy::prelude::*;

#[cfg(test)]
use crate::items::INVENTORY_SLOT_COUNT;

pub(super) const ACTION_BAR_SLOT_COUNT: usize = 8;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActionBar {
    selected_slot: usize,
    held_inventory_slots: [Option<usize>; ACTION_BAR_SLOT_COUNT],
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

    pub fn held_inventory_slot(
        self,
        slot_index: usize,
    ) -> Result<Option<usize>, ActionBarSlotOutOfRange> {
        self.held_inventory_slots
            .get(slot_index)
            .copied()
            .ok_or(ActionBarSlotOutOfRange)
    }

    #[cfg(test)]
    pub fn assign_held_inventory_slot(
        &mut self,
        slot_index: usize,
        inventory_index: usize,
    ) -> Result<(), ActionBarHeldSlotOutOfRange> {
        if slot_index_is_out_of_range(slot_index) || inventory_index >= INVENTORY_SLOT_COUNT {
            return Err(ActionBarHeldSlotOutOfRange);
        }
        self.held_inventory_slots[slot_index] = Some(inventory_index);
        Ok(())
    }

    pub fn clear_held_inventory_slot(
        &mut self,
        slot_index: usize,
    ) -> Result<(), ActionBarHeldSlotOutOfRange> {
        if slot_index_is_out_of_range(slot_index) {
            return Err(ActionBarHeldSlotOutOfRange);
        }
        self.held_inventory_slots[slot_index] = None;
        Ok(())
    }
}

fn slot_index_is_out_of_range(slot_index: usize) -> bool {
    slot_index >= ACTION_BAR_SLOT_COUNT
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionBarSlotOutOfRange;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionBarHeldSlotOutOfRange;

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::{
        ActionBar, ActionBarHeldSlotOutOfRange, ActionBarSlotOutOfRange, ACTION_BAR_SLOT_COUNT,
    };
    use crate::farming::FarmingPlugin;
    use crate::items::INVENTORY_SLOT_COUNT;

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
    fn the_action_bar_starts_with_every_slot_empty() {
        assert_every_held_slot_empty(&ActionBar::default());
    }

    #[test]
    fn assigning_every_slot_stores_that_inventory_index() {
        let mut action_bar = ActionBar::default();
        for slot_index in 0..ACTION_BAR_SLOT_COUNT {
            let inventory_index = INVENTORY_SLOT_COUNT - 1 - slot_index;
            assert_eq!(
                action_bar.assign_held_inventory_slot(slot_index, inventory_index),
                Ok(())
            );
            assert_eq!(
                action_bar.held_inventory_slot(slot_index),
                Ok(Some(inventory_index))
            );
        }
    }

    #[test]
    fn assigning_or_clearing_an_out_of_range_bar_slot_or_inventory_index_returns_an_error_and_leaves_prior_contents(
    ) {
        let mut action_bar = ActionBar::default();
        action_bar.assign_held_inventory_slot(1, 4).unwrap();
        action_bar
            .assign_held_inventory_slot(2, INVENTORY_SLOT_COUNT - 1)
            .unwrap();
        let prior_contents = action_bar;

        assert_eq!(
            action_bar.assign_held_inventory_slot(ACTION_BAR_SLOT_COUNT, 1),
            Err(ActionBarHeldSlotOutOfRange)
        );
        assert_eq!(
            action_bar.assign_held_inventory_slot(usize::MAX, 1),
            Err(ActionBarHeldSlotOutOfRange)
        );
        assert_eq!(
            action_bar.assign_held_inventory_slot(0, INVENTORY_SLOT_COUNT),
            Err(ActionBarHeldSlotOutOfRange)
        );
        assert_eq!(
            action_bar.assign_held_inventory_slot(3, usize::MAX),
            Err(ActionBarHeldSlotOutOfRange)
        );
        assert_eq!(
            action_bar.clear_held_inventory_slot(ACTION_BAR_SLOT_COUNT),
            Err(ActionBarHeldSlotOutOfRange)
        );
        assert_eq!(
            action_bar.clear_held_inventory_slot(usize::MAX),
            Err(ActionBarHeldSlotOutOfRange)
        );
        assert_eq!(
            action_bar.held_inventory_slot(ACTION_BAR_SLOT_COUNT),
            Err(ActionBarSlotOutOfRange)
        );
        assert_eq!(
            action_bar.held_inventory_slot(usize::MAX),
            Err(ActionBarSlotOutOfRange)
        );
        assert_eq!(action_bar, prior_contents);
    }

    #[test]
    fn clearing_a_held_slot_removes_only_that_inventory_index() {
        let mut action_bar = ActionBar::default();
        action_bar.assign_held_inventory_slot(1, 4).unwrap();
        action_bar.assign_held_inventory_slot(2, 31).unwrap();

        assert_eq!(action_bar.clear_held_inventory_slot(1), Ok(()));
        assert_eq!(action_bar.held_inventory_slot(1), Ok(None));
        assert_eq!(action_bar.held_inventory_slot(2), Ok(Some(31)));
    }

    #[test]
    fn an_app_with_the_farming_plugin_starts_with_every_slot_empty() {
        let app = app_with_farming_plugin();
        assert_every_held_slot_empty(app.world().resource::<ActionBar>());
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

    fn assert_every_held_slot_empty(action_bar: &ActionBar) {
        for slot_index in 0..ACTION_BAR_SLOT_COUNT {
            assert_eq!(action_bar.held_inventory_slot(slot_index), Ok(None));
        }
    }

    fn app_with_farming_plugin() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, FarmingPlugin));
        app
    }
}
