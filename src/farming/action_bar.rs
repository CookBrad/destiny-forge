use bevy::prelude::*;

use crate::items::{Inventory, INVENTORY_SLOT_COUNT};

use super::tools::{EquippedTool, HomesteadTool};

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

pub(super) fn clear_emptied_action_bar_slots(
    inventory: Res<Inventory>,
    mut action_bar: ResMut<ActionBar>,
    mut equipped_tool: ResMut<EquippedTool>,
) {
    let selected_slot = action_bar.selected_slot();
    let mut selected_slot_cleared = false;
    for slot_index in 0..ACTION_BAR_SLOT_COUNT {
        if !held_stack_was_emptied(&action_bar, &inventory, slot_index) {
            continue;
        }
        match action_bar.clear_held_inventory_slot(slot_index) {
            Ok(()) => {}
            Err(ActionBarHeldSlotOutOfRange) => {}
        }
        if slot_index == selected_slot {
            selected_slot_cleared = true;
        }
    }
    if selected_slot_cleared {
        equip_hand_if_needed(&mut equipped_tool);
    }
}

fn held_stack_was_emptied(
    action_bar: &ActionBar,
    inventory: &Inventory,
    slot_index: usize,
) -> bool {
    let Ok(Some(inventory_index)) = action_bar.held_inventory_slot(slot_index) else {
        return false;
    };
    let Some(stack) = inventory.slots.get(inventory_index) else {
        return true;
    };
    stack.material.is_none() || stack.count == 0
}

fn equip_hand_if_needed(equipped_tool: &mut EquippedTool) {
    if equipped_tool.0 == HomesteadTool::Hand {
        return;
    }
    equipped_tool.0 = HomesteadTool::Hand;
    info!("Equipped {}.", HomesteadTool::Hand.label());
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::{
        clear_emptied_action_bar_slots, ActionBar, ActionBarHeldSlotOutOfRange,
        ActionBarSlotOutOfRange, ACTION_BAR_SLOT_COUNT,
    };
    use crate::farming::actions::apply_tool;
    use crate::farming::crops::{CropKind, PlotStage};
    use crate::farming::tools::{EquippedTool, HomesteadTool};
    use crate::farming::FarmingPlugin;
    use crate::items::{Inventory, MaterialId, INVENTORY_SLOT_COUNT};

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

    #[test]
    fn emptying_a_held_stack_clears_that_action_bar_slot() {
        let mut app = clear_emptied_slots_app();
        put_stack(&mut app, 3, MaterialId::TurnipSeed, 1);
        put_stack(&mut app, 5, MaterialId::Hoe, 1);
        assign_held_slot(&mut app, 0, 3);
        assign_held_slot(&mut app, 7, 5);
        app.update();
        assert_eq!(held_slot(&app, 0), Ok(Some(3)));
        assert_eq!(held_slot(&app, 7), Ok(Some(5)));

        assert!(app
            .world_mut()
            .resource_mut::<Inventory>()
            .try_remove(MaterialId::TurnipSeed, 1));
        app.update();
        assert_eq!(held_slot(&app, 0), Ok(None));
        assert_eq!(held_slot(&app, 7), Ok(Some(5)));
    }

    #[test]
    fn emptying_the_selected_slot_equips_the_hand() {
        let mut app = clear_emptied_slots_app();
        set_equipped_tool(&mut app, HomesteadTool::Seeds);
        put_stack(&mut app, 3, MaterialId::TurnipSeed, 1);
        assign_held_slot(&mut app, 0, 3);
        assert_eq!(app.world().resource::<ActionBar>().selected_slot(), 0);
        assert!(app
            .world_mut()
            .resource_mut::<Inventory>()
            .try_remove(MaterialId::TurnipSeed, 1));
        app.update();
        assert_eq!(held_slot(&app, 0), Ok(None));
        assert_eq!(equipped_tool(&app), HomesteadTool::Hand);
    }

    #[test]
    fn emptying_a_non_selected_slot_leaves_the_equipped_tool() {
        let mut app = clear_emptied_slots_app();
        set_equipped_tool(&mut app, HomesteadTool::Hoe);
        put_stack(&mut app, 3, MaterialId::TurnipSeed, 1);
        put_stack(&mut app, 5, MaterialId::Hoe, 1);
        assign_held_slot(&mut app, 0, 5);
        assign_held_slot(&mut app, 2, 3);
        assert!(app
            .world_mut()
            .resource_mut::<Inventory>()
            .try_remove(MaterialId::TurnipSeed, 1));
        app.update();
        assert_eq!(held_slot(&app, 0), Ok(Some(5)));
        assert_eq!(held_slot(&app, 2), Ok(None));
        assert_eq!(equipped_tool(&app), HomesteadTool::Hoe);
    }

    #[test]
    fn a_zero_count_or_material_less_held_stack_clears_the_bar_slot() {
        let mut app = clear_emptied_slots_app();
        put_stack(&mut app, 0, MaterialId::Hoe, 0);
        app.world_mut().resource_mut::<Inventory>().slots[1].material = None;
        app.world_mut().resource_mut::<Inventory>().slots[1].count = 1;
        assign_held_slot(&mut app, 1, 0);
        assign_held_slot(&mut app, 4, 1);
        app.update();
        assert_eq!(held_slot(&app, 1), Ok(None));
        assert_eq!(held_slot(&app, 4), Ok(None));
    }

    #[test]
    fn planting_the_last_seed_clears_that_action_bar_slot() {
        let mut app = clear_emptied_slots_app();
        set_equipped_tool(&mut app, HomesteadTool::Seeds);
        put_stack(&mut app, 3, MaterialId::TurnipSeed, 1);
        assign_held_slot(&mut app, 0, 3);
        let planted = apply_tool(
            HomesteadTool::Seeds,
            PlotStage::Tilled,
            &mut app.world_mut().resource_mut::<Inventory>(),
        );
        let (stage, _result) = planted.expect("planted the last seed");
        assert!(matches!(
            stage,
            PlotStage::Growing {
                crop: CropKind::Turnip,
                ..
            }
        ));
        app.update();
        assert_eq!(held_slot(&app, 0), Ok(None));
        assert_eq!(equipped_tool(&app), HomesteadTool::Hand);
        assert_eq!(
            app.world()
                .resource::<Inventory>()
                .count(MaterialId::TurnipSeed),
            0
        );
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

    fn clear_emptied_slots_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Inventory>()
            .init_resource::<ActionBar>()
            .init_resource::<EquippedTool>()
            .add_systems(Update, clear_emptied_action_bar_slots);
        app
    }

    fn put_stack(app: &mut App, inventory_index: usize, material: MaterialId, count: u32) {
        let mut inventory = app.world_mut().resource_mut::<Inventory>();
        inventory.slots[inventory_index].material = Some(material);
        inventory.slots[inventory_index].count = count;
    }

    fn assign_held_slot(app: &mut App, slot_index: usize, inventory_index: usize) {
        let assigned = app
            .world_mut()
            .resource_mut::<ActionBar>()
            .assign_held_inventory_slot(slot_index, inventory_index);
        assert_eq!(assigned, Ok(()));
    }

    fn set_equipped_tool(app: &mut App, tool: HomesteadTool) {
        app.world_mut().resource_mut::<EquippedTool>().0 = tool;
    }

    fn held_slot(app: &App, slot_index: usize) -> Result<Option<usize>, ActionBarSlotOutOfRange> {
        app.world()
            .resource::<ActionBar>()
            .held_inventory_slot(slot_index)
    }

    fn equipped_tool(app: &App) -> HomesteadTool {
        app.world().resource::<EquippedTool>().0
    }
}
