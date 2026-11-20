use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::input::{Action, ActionState};
use crate::items::Inventory;
use crate::ui::inventory_window::{material_visual, InventorySlot};

const GHOST_SIZE: f32 = 40.0;
const GHOST_LAYER: i32 = 400;

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

pub(super) fn clear_action_bar_drag(mut commands: Commands, mut drag: ResMut<ActionBarDrag>) {
    drag.from_inventory_slot = None;
    despawn_action_bar_drag_ghost(&mut commands, &mut drag);
}

fn despawn_action_bar_drag_ghost(commands: &mut Commands, drag: &mut ActionBarDrag) {
    if let Some(entity) = drag.ghost.take() {
        commands.entity(entity).try_despawn_recursive();
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::mouse::MouseButton;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;
    use bevy::window::{PrimaryWindow, WindowResolution};

    use crate::core::GameState;
    use crate::input::action_state_after_pressing;
    use crate::items::{Inventory, MaterialId};
    use crate::ui::inventory_window::{inventory_closed, InventorySlot, InventoryWindowOpen};

    use super::{
        clear_action_bar_drag, drag_source_from_slot, start_action_bar_drag,
        update_action_bar_drag_ghost, ActionBarDrag, ActionBarDragGhost, GHOST_SIZE,
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

    fn app_with_open_inventory() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ActionBarDrag>()
            .init_resource::<Inventory>()
            .insert_resource(InventoryWindowOpen(true))
            .insert_resource(action_state_after_pressing(&[], &[]))
            .add_systems(
                Update,
                (
                    start_action_bar_drag,
                    update_action_bar_drag_ghost.after(start_action_bar_drag),
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
