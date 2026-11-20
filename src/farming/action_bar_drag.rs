use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::input::{Action, ActionState};
use crate::items::Inventory;
use crate::ui::inventory_window::{material_visual, InventorySlot};

use super::action_bar::{ActionBar, ActionBarHeldSlotOutOfRange, ACTION_BAR_SLOT_COUNT};

const GHOST_SIZE: f32 = 40.0;
const GHOST_LAYER: i32 = 400;
const FRAME_WIDTH: f32 = 498.0;
const FRAME_HEIGHT: f32 = 78.0;
const SLOT_SIZE: f32 = 54.0;
const SLOT_ORIGIN_LEFT: f32 = 12.0;
const SLOT_ORIGIN_TOP: f32 = 12.0;
const SLOT_STRIDE: f32 = 60.0;
const BAR_BOTTOM: f32 = 14.0;

#[derive(Resource, Default)]
pub(super) struct ActionBarDrag {
    pub from_inventory_slot: Option<usize>,
    pub ghost: Option<Entity>,
}

#[derive(Component)]
pub(super) struct ActionBarDragGhost;

pub(super) fn start_action_bar_drag(
    actions: Res<ActionState>,
    inventory: Res<Inventory>,
    mut drag: ResMut<ActionBarDrag>,
    slots: Query<(&InventorySlot, &Interaction), With<Button>>,
) {
    if drag.from_inventory_slot.is_some() || !actions.just_pressed(Action::PrimaryButton) {
        return;
    }
    let Some(inventory_index) = pressed_drag_source(&inventory, &slots) else {
        return;
    };
    drag.from_inventory_slot = Some(inventory_index);
}

fn pressed_drag_source(
    inventory: &Inventory,
    slots: &Query<(&InventorySlot, &Interaction), With<Button>>,
) -> Option<usize> {
    slots
        .iter()
        .find_map(|(slot, interaction)| drag_source_from_slot(inventory, slot.index, interaction))
}

fn drag_source_from_slot(
    inventory: &Inventory,
    inventory_index: usize,
    interaction: &Interaction,
) -> Option<usize> {
    if *interaction != Interaction::Pressed {
        return None;
    }
    let stack = inventory.slots.get(inventory_index)?;
    if stack.material.is_some() && stack.count > 0 {
        Some(inventory_index)
    } else {
        None
    }
}

pub(super) fn update_action_bar_drag_ghost(
    mut commands: Commands,
    mut drag: ResMut<ActionBarDrag>,
    inventory: Res<Inventory>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut ghosts: Query<&mut Node, With<ActionBarDragGhost>>,
) {
    let Some(inventory_index) = drag.from_inventory_slot else {
        return;
    };
    let Some(cursor) = primary_cursor(&windows) else {
        return;
    };
    let Some((color, label)) = dragged_material_visual(&inventory, inventory_index) else {
        return;
    };
    place_action_bar_drag_ghost(&mut commands, &mut drag, &mut ghosts, color, label, cursor);
}

fn primary_cursor(windows: &Query<&Window, With<PrimaryWindow>>) -> Option<Vec2> {
    windows.iter().next()?.cursor_position()
}

fn dragged_material_visual(
    inventory: &Inventory,
    inventory_index: usize,
) -> Option<(Color, &'static str)> {
    let stack = inventory.slots.get(inventory_index)?;
    let material = stack.material.filter(|_| stack.count > 0)?;
    Some(material_visual(material))
}

fn place_action_bar_drag_ghost(
    commands: &mut Commands,
    drag: &mut ActionBarDrag,
    ghosts: &mut Query<&mut Node, With<ActionBarDragGhost>>,
    color: Color,
    label: &'static str,
    cursor: Vec2,
) {
    let origin = cursor - Vec2::splat(GHOST_SIZE * 0.5);
    if let Some(ghost) = drag.ghost {
        if let Ok(mut node) = ghosts.get_mut(ghost) {
            node.left = Val::Px(origin.x);
            node.top = Val::Px(origin.y);
            return;
        }
    }
    drag.ghost = Some(spawn_action_bar_drag_ghost(commands, color, label, origin));
}

fn spawn_action_bar_drag_ghost(
    commands: &mut Commands,
    color: Color,
    label: &'static str,
    origin: Vec2,
) -> Entity {
    commands
        .spawn((
            ActionBarDragGhost,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(origin.x),
                top: Val::Px(origin.y),
                width: Val::Px(GHOST_SIZE),
                height: Val::Px(GHOST_SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            GlobalZIndex(GHOST_LAYER),
        ))
        .with_children(|ghost| {
            ghost.spawn((
                Text::new(label),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 0.98)),
            ));
        })
        .id()
}

pub(super) fn release_action_bar_drag(
    actions: Res<ActionState>,
    mut action_bar: ResMut<ActionBar>,
    mut commands: Commands,
    mut drag: ResMut<ActionBarDrag>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if !actions.just_released(Action::PrimaryButton) {
        return;
    }
    let Some(inventory_index) = drag.from_inventory_slot else {
        return;
    };
    if let Some(slot_index) = drop_slot_from_cursor(&windows) {
        match action_bar.assign_held_inventory_slot(slot_index, inventory_index) {
            Ok(()) => {}
            Err(ActionBarHeldSlotOutOfRange) => {}
        }
    }
    drag.from_inventory_slot = None;
    despawn_action_bar_drag_ghost(&mut commands, &mut drag);
}

fn drop_slot_from_cursor(windows: &Query<&Window, With<PrimaryWindow>>) -> Option<usize> {
    let window = windows.iter().next()?;
    let cursor = window.cursor_position()?;
    action_bar_slot_at_cursor(cursor, window.width(), window.height())
}

fn action_bar_slot_at_cursor(cursor: Vec2, window_width: f32, window_height: f32) -> Option<usize> {
    if !cursor.x.is_finite()
        || !cursor.y.is_finite()
        || !window_width.is_finite()
        || !window_height.is_finite()
    {
        return None;
    }
    let frame_left = (window_width - FRAME_WIDTH) * 0.5;
    let slot_top = window_height - BAR_BOTTOM - FRAME_HEIGHT + SLOT_ORIGIN_TOP;
    if cursor.y < slot_top || cursor.y >= slot_top + SLOT_SIZE {
        return None;
    }
    for index in 0..ACTION_BAR_SLOT_COUNT {
        let slot_left = frame_left + SLOT_ORIGIN_LEFT + index as f32 * SLOT_STRIDE;
        if cursor.x >= slot_left && cursor.x < slot_left + SLOT_SIZE {
            return Some(index);
        }
    }
    None
}

fn despawn_action_bar_drag_ghost(commands: &mut Commands, drag: &mut ActionBarDrag) {
    if let Some(entity) = drag.ghost.take() {
        commands.entity(entity).try_despawn_recursive();
    }
}

pub(super) fn clear_action_bar_drag(mut commands: Commands, mut drag: ResMut<ActionBarDrag>) {
    drag.from_inventory_slot = None;
    despawn_action_bar_drag_ghost(&mut commands, &mut drag);
}

#[cfg(test)]
mod tests {
    use bevy::input::mouse::MouseButton;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;
    use bevy::window::{PrimaryWindow, WindowResolution};

    use crate::core::GameState;
    use crate::farming::ActionBar;
    use crate::input::{action_state_after_pressing, ActionState, InputMap};
    use crate::items::{Inventory, MaterialId};
    use crate::ui::inventory_window::{inventory_closed, InventorySlot, InventoryWindowOpen};

    use super::{
        action_bar_slot_at_cursor, clear_action_bar_drag, drag_source_from_slot,
        release_action_bar_drag, start_action_bar_drag, update_action_bar_drag_ghost,
        ActionBarDrag, ActionBarDragGhost, BAR_BOTTOM, FRAME_HEIGHT, FRAME_WIDTH, GHOST_SIZE,
        SLOT_ORIGIN_LEFT, SLOT_ORIGIN_TOP, SLOT_SIZE, SLOT_STRIDE,
    };

    #[test]
    fn pressing_primary_button_on_a_hoe_starts_a_drag_from_that_slot() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        spawn_pressed_slot(&mut app, 3);
        press_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            Some(3)
        );
    }

    #[test]
    fn pressing_primary_button_on_turnip_seed_starts_a_drag_from_that_slot() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 0, MaterialId::TurnipSeed, 8);
        spawn_pressed_slot(&mut app, 0);
        press_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            Some(0)
        );
    }

    #[test]
    fn the_drag_ghost_is_centered_on_the_cursor() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        spawn_pressed_slot(&mut app, 3);
        spawn_primary_window(&mut app, Vec2::new(100.0, 80.0));
        press_primary(&mut app);
        app.update();
        assert_eq!(
            ghost_origin(&mut app),
            Some(ghost_origin_for(Vec2::new(100.0, 80.0)))
        );
    }

    #[test]
    fn moving_the_cursor_moves_the_drag_ghost() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        spawn_pressed_slot(&mut app, 3);
        let window = spawn_primary_window(&mut app, Vec2::new(100.0, 80.0));
        press_primary(&mut app);
        app.update();
        release_primary(&mut app);
        set_cursor(&mut app, window, Some(Vec2::new(300.0, 200.0)));
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            Some(3)
        );
        assert_eq!(
            ghost_origin(&mut app),
            Some(ghost_origin_for(Vec2::new(300.0, 200.0)))
        );
    }

    #[test]
    fn pressing_an_empty_inventory_slot_does_not_start_a_drag() {
        let mut app = app_with_open_inventory();
        spawn_pressed_slot(&mut app, 5);
        press_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            None
        );
    }

    #[test]
    fn pressing_a_slot_with_count_zero_does_not_start_a_drag() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 2, MaterialId::Potato, 0);
        spawn_pressed_slot(&mut app, 2);
        press_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            None
        );
    }

    #[test]
    fn pressing_an_occupied_hovered_slot_does_not_start_a_drag() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        spawn_slot(&mut app, 3, Interaction::Hovered);
        press_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            None
        );
    }

    #[test]
    fn pressing_an_occupied_slot_with_no_interaction_does_not_start_a_drag() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        spawn_slot(&mut app, 3, Interaction::None);
        press_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            None
        );
    }

    #[test]
    fn a_slot_index_past_the_inventory_returns_none_without_panicking() {
        let inventory = Inventory::default();
        assert_eq!(
            drag_source_from_slot(&inventory, inventory.slots.len(), &Interaction::Pressed),
            None
        );
        assert_eq!(
            drag_source_from_slot(&inventory, usize::MAX, &Interaction::Pressed),
            None
        );
    }

    #[test]
    fn a_second_press_on_another_slot_keeps_the_original_drag() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        put_stack(&mut app, 0, MaterialId::TurnipSeed, 8);
        spawn_pressed_slot(&mut app, 3);
        press_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            Some(3)
        );
        release_primary(&mut app);
        spawn_pressed_slot(&mut app, 0);
        press_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            Some(3)
        );
    }

    #[test]
    fn emptying_a_stack_mid_drag_leaves_the_ghost_in_place() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        spawn_pressed_slot(&mut app, 3);
        spawn_primary_window(&mut app, Vec2::new(100.0, 80.0));
        press_primary(&mut app);
        app.update();
        assert_eq!(
            ghost_origin(&mut app),
            Some(ghost_origin_for(Vec2::new(100.0, 80.0)))
        );
        release_primary(&mut app);
        put_stack(&mut app, 3, MaterialId::Hoe, 0);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            Some(3)
        );
        assert_eq!(
            ghost_origin(&mut app),
            Some(ghost_origin_for(Vec2::new(100.0, 80.0)))
        );
        assert_eq!(ghost_count(&mut app), 1);
    }

    #[test]
    fn a_missing_cursor_leaves_the_ghost_at_its_last_position() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        spawn_pressed_slot(&mut app, 3);
        let window = spawn_primary_window(&mut app, Vec2::new(100.0, 80.0));
        press_primary(&mut app);
        app.update();
        release_primary(&mut app);
        set_cursor(&mut app, window, None);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            Some(3)
        );
        assert_eq!(
            ghost_origin(&mut app),
            Some(ghost_origin_for(Vec2::new(100.0, 80.0)))
        );
    }

    #[test]
    fn closing_the_inventory_clears_an_in_progress_drag_and_despawns_the_ghost() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        spawn_pressed_slot(&mut app, 3);
        spawn_primary_window(&mut app, Vec2::new(100.0, 80.0));
        press_primary(&mut app);
        app.update();
        assert!(app.world().resource::<ActionBarDrag>().ghost.is_some());
        app.world_mut().resource_mut::<InventoryWindowOpen>().0 = false;
        release_primary(&mut app);
        app.update();
        let drag = app.world().resource::<ActionBarDrag>();
        assert_eq!(drag.from_inventory_slot, None);
        assert_eq!(drag.ghost, None);
        assert_eq!(ghost_count(&mut app), 0);
    }

    #[test]
    fn leaving_the_overworld_clears_an_in_progress_drag_and_despawns_the_ghost() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<GameState>()
            .init_resource::<ActionBarDrag>()
            .init_resource::<Inventory>()
            .insert_resource(InventoryWindowOpen(true))
            .insert_resource(action_state_after_pressing(&[], &[]))
            .add_systems(
                Update,
                (
                    start_action_bar_drag,
                    update_action_bar_drag_ghost.after(start_action_bar_drag),
                )
                    .run_if(in_state(GameState::Overworld)),
            )
            .add_systems(OnExit(GameState::Overworld), clear_action_bar_drag);
        enter_state(&mut app, GameState::Overworld);
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        spawn_pressed_slot(&mut app, 3);
        spawn_primary_window(&mut app, Vec2::new(100.0, 80.0));
        press_primary(&mut app);
        app.update();
        assert!(app.world().resource::<ActionBarDrag>().ghost.is_some());
        release_primary(&mut app);
        enter_state(&mut app, GameState::Title);
        let drag = app.world().resource::<ActionBarDrag>();
        assert_eq!(drag.from_inventory_slot, None);
        assert_eq!(drag.ghost, None);
        assert_eq!(ghost_count(&mut app), 0);
    }

    #[test]
    fn a_missing_primary_window_does_not_panic_during_a_drag() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        spawn_pressed_slot(&mut app, 3);
        press_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            Some(3)
        );
        assert_eq!(app.world().resource::<ActionBarDrag>().ghost, None);
        assert_eq!(ghost_count(&mut app), 0);
    }

    #[test]
    fn drag_systems_do_not_panic_when_no_inventory_slot_entities_exist() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        press_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            None
        );
    }

    #[test]
    fn releasing_over_slot_zero_assigns_the_dragged_inventory_index() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::TurnipSeed, 4);
        let window = start_drag(&mut app, 3, slot_center(0));
        just_release_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(0),
            Ok(Some(3))
        );
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            None
        );
        assert_eq!(ghost_count(&mut app), 0);
        let _ = window;
    }

    #[test]
    fn a_seed_can_drop_in_slot_zero_and_a_hoe_can_drop_in_slot_seven() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 1, MaterialId::TurnipSeed, 8);
        let window = start_drag(&mut app, 1, slot_center(0));
        just_release_primary(&mut app);
        app.update();
        clear_slot_interactions(&mut app);
        put_stack(&mut app, 10, MaterialId::Hoe, 1);
        set_cursor(&mut app, window, Some(slot_center(7)));
        spawn_pressed_slot(&mut app, 10);
        press_primary(&mut app);
        app.update();
        set_cursor(&mut app, window, Some(slot_center(7)));
        just_release_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(0),
            Ok(Some(1))
        );
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(7),
            Ok(Some(10))
        );
    }

    #[test]
    fn releasing_over_a_filled_slot_replaces_the_held_inventory_index() {
        let mut app = app_with_open_inventory();
        app.world_mut()
            .resource_mut::<ActionBar>()
            .assign_held_inventory_slot(0, 5)
            .expect("valid assignment");
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        start_drag(&mut app, 3, slot_center(0));
        just_release_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(0),
            Ok(Some(3))
        );
    }

    #[test]
    fn releasing_off_the_bar_cancels_the_drag_and_leaves_the_bar_unchanged() {
        let mut app = app_with_open_inventory();
        app.world_mut()
            .resource_mut::<ActionBar>()
            .assign_held_inventory_slot(2, 4)
            .expect("valid assignment");
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        start_drag(&mut app, 3, Vec2::new(100.0, 80.0));
        just_release_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(2),
            Ok(Some(4))
        );
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(0),
            Ok(None)
        );
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            None
        );
        assert_eq!(ghost_count(&mut app), 0);
    }

    #[test]
    fn a_cursor_in_the_gap_between_slots_does_not_assign_a_slot() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        start_drag(&mut app, 3, gap_between(0, 1));
        just_release_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(0),
            Ok(None)
        );
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(1),
            Ok(None)
        );
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            None
        );
    }

    #[test]
    fn slot_hit_tests_are_inclusive_on_the_left_and_top_and_exclusive_on_the_right_and_bottom() {
        let (left, top, right, bottom) = slot_edges(0);
        assert_eq!(
            action_bar_slot_at_cursor(Vec2::new(left, top + 1.0), 1280.0, 720.0),
            Some(0)
        );
        assert_eq!(
            action_bar_slot_at_cursor(Vec2::new(left + 1.0, top), 1280.0, 720.0),
            Some(0)
        );
        assert_eq!(
            action_bar_slot_at_cursor(Vec2::new(right, top + 1.0), 1280.0, 720.0),
            None
        );
        assert_eq!(
            action_bar_slot_at_cursor(Vec2::new(left + 1.0, bottom), 1280.0, 720.0),
            None
        );
        assert_eq!(
            action_bar_slot_at_cursor(gap_between(0, 1), 1280.0, 720.0),
            None
        );
        assert_eq!(
            action_bar_slot_at_cursor(slot_center(7), 1280.0, 720.0),
            Some(7)
        );
    }

    #[test]
    fn a_non_finite_cursor_or_window_size_does_not_hit_a_slot() {
        let cursor = slot_center(0);
        assert_eq!(
            action_bar_slot_at_cursor(Vec2::new(f32::NAN, cursor.y), 1280.0, 720.0),
            None
        );
        assert_eq!(
            action_bar_slot_at_cursor(Vec2::new(cursor.x, f32::INFINITY), 1280.0, 720.0),
            None
        );
        assert_eq!(action_bar_slot_at_cursor(cursor, f32::NAN, 720.0), None);
        assert_eq!(
            action_bar_slot_at_cursor(cursor, 1280.0, f32::NEG_INFINITY),
            None
        );
    }

    #[test]
    fn releasing_without_a_primary_window_cancels_the_drag_and_does_not_panic() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        spawn_pressed_slot(&mut app, 3);
        press_primary(&mut app);
        app.update();
        just_release_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            None
        );
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(0),
            Ok(None)
        );
    }

    #[test]
    fn releasing_without_a_cursor_cancels_the_drag_and_leaves_the_bar_unchanged() {
        let mut app = app_with_open_inventory();
        put_stack(&mut app, 3, MaterialId::Hoe, 1);
        let window = start_drag(&mut app, 3, slot_center(0));
        set_cursor(&mut app, window, None);
        just_release_primary(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<ActionBarDrag>().from_inventory_slot,
            None
        );
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(0),
            Ok(None)
        );
        assert_eq!(ghost_count(&mut app), 0);
    }

    fn app_with_open_inventory() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ActionBarDrag>()
            .init_resource::<ActionBar>()
            .init_resource::<Inventory>()
            .insert_resource(InventoryWindowOpen(true))
            .insert_resource(action_state_after_pressing(&[], &[]))
            .add_systems(
                Update,
                (
                    start_action_bar_drag,
                    update_action_bar_drag_ghost.after(start_action_bar_drag),
                    release_action_bar_drag.after(update_action_bar_drag_ghost),
                    clear_action_bar_drag.run_if(inventory_closed),
                ),
            );
        app
    }

    fn enter_state(app: &mut App, state: GameState) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(state);
        app.update();
    }

    fn put_stack(app: &mut App, inventory_index: usize, material: MaterialId, count: u32) {
        let mut inventory = app.world_mut().resource_mut::<Inventory>();
        inventory.slots[inventory_index].material = Some(material);
        inventory.slots[inventory_index].count = count;
    }

    fn spawn_pressed_slot(app: &mut App, inventory_index: usize) {
        spawn_slot(app, inventory_index, Interaction::Pressed);
    }

    fn spawn_slot(app: &mut App, inventory_index: usize, interaction: Interaction) {
        app.world_mut().spawn((
            Button,
            InventorySlot {
                index: inventory_index,
            },
            interaction,
        ));
    }

    fn spawn_primary_window(app: &mut App, cursor: Vec2) -> Entity {
        let mut window = Window {
            resolution: WindowResolution::new(1280.0, 720.0),
            ..default()
        };
        window.set_cursor_position(Some(cursor));
        app.world_mut().spawn((window, PrimaryWindow)).id()
    }

    fn set_cursor(app: &mut App, window: Entity, cursor: Option<Vec2>) {
        app.world_mut()
            .get_mut::<Window>(window)
            .expect("primary window")
            .set_cursor_position(cursor);
    }

    fn press_primary(app: &mut App) {
        app.world_mut()
            .insert_resource(action_state_after_pressing(&[], &[MouseButton::Left]));
    }

    fn release_primary(app: &mut App) {
        app.world_mut()
            .insert_resource(action_state_after_pressing(&[], &[]));
    }

    fn just_release_primary(app: &mut App) {
        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Left);
        mouse.clear();
        mouse.release(MouseButton::Left);
        app.world_mut().insert_resource(ActionState::from_devices(
            &InputMap::default(),
            &ButtonInput::<KeyCode>::default(),
            &mouse,
        ));
    }

    fn clear_slot_interactions(app: &mut App) {
        let mut interactions = app.world_mut().query::<&mut Interaction>();
        for mut interaction in interactions.iter_mut(app.world_mut()) {
            *interaction = Interaction::None;
        }
    }

    fn start_drag(app: &mut App, inventory_index: usize, cursor: Vec2) -> Entity {
        spawn_pressed_slot(app, inventory_index);
        let window = spawn_primary_window(app, cursor);
        press_primary(app);
        app.update();
        release_primary(app);
        window
    }

    fn slot_center(slot_index: usize) -> Vec2 {
        let (left, top, right, bottom) = slot_edges(slot_index);
        Vec2::new((left + right) * 0.5, (top + bottom) * 0.5)
    }

    fn gap_between(left_slot: usize, right_slot: usize) -> Vec2 {
        let (_, top, right, bottom) = slot_edges(left_slot);
        let (next_left, _, _, _) = slot_edges(right_slot);
        Vec2::new((right + next_left) * 0.5, (top + bottom) * 0.5)
    }

    fn slot_edges(slot_index: usize) -> (f32, f32, f32, f32) {
        let frame_left = (1280.0 - FRAME_WIDTH) * 0.5;
        let slot_top = 720.0 - BAR_BOTTOM - FRAME_HEIGHT + SLOT_ORIGIN_TOP;
        let slot_left = frame_left + SLOT_ORIGIN_LEFT + slot_index as f32 * SLOT_STRIDE;
        (
            slot_left,
            slot_top,
            slot_left + SLOT_SIZE,
            slot_top + SLOT_SIZE,
        )
    }

    fn ghost_origin_for(cursor: Vec2) -> (f32, f32) {
        let origin = cursor - Vec2::splat(GHOST_SIZE * 0.5);
        (origin.x, origin.y)
    }

    fn ghost_origin(app: &mut App) -> Option<(f32, f32)> {
        let mut query = app
            .world_mut()
            .query_filtered::<&Node, With<ActionBarDragGhost>>();
        let node = query.iter(app.world()).next()?;
        match (node.left, node.top) {
            (Val::Px(left), Val::Px(top)) => Some((left, top)),
            _ => None,
        }
    }

    fn ghost_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ActionBarDragGhost>()
            .iter(app.world())
            .count()
    }
}
