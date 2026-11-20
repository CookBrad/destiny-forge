use bevy::prelude::*;

use crate::core::GameState;
use crate::overworld::setup::{cleanup_overworld, setup_overworld};
use crate::ui::forge_window::forge_closed;
use crate::ui::inventory_window::{inventory_closed, inventory_window_open};

use super::action_bar::clear_emptied_action_bar_slots;
use super::action_bar_art::{
    cleanup_action_bar_hud, load_action_bar_art, setup_action_bar_hud, sync_action_bar_highlight,
    sync_action_bar_slot_contents,
};
use super::action_bar_drag::{
    clear_action_bar_drag, release_action_bar_drag, start_action_bar_drag,
    update_action_bar_drag_ghost, ActionBarDrag,
};
use super::plots::sync_plot_visuals;
use super::select_action_bar_slot::{
    cycle_action_bar_slot_from_mouse_wheel, select_action_bar_slot_from_number_keys,
};
use super::tools::EquippedTool;
use super::use_tool::{
    capture_crop_plots_on_exit, ensure_starter_kit, update_player_facing, use_homestead_tool,
};
use super::ActionBar;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ActionBarSlotContentsSync;

pub struct FarmingPlugin;

impl Plugin for FarmingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EquippedTool>()
            .init_resource::<ActionBar>()
            .init_resource::<ActionBarDrag>()
            .add_systems(
                OnEnter(GameState::Overworld),
                (
                    ensure_starter_kit,
                    load_action_bar_art,
                    setup_action_bar_hud,
                )
                    .chain()
                    .after(setup_overworld),
            )
            .add_systems(
                OnExit(GameState::Overworld),
                (
                    clear_action_bar_drag,
                    capture_crop_plots_on_exit.before(cleanup_overworld),
                    cleanup_action_bar_hud,
                ),
            )
            .add_systems(
                Update,
                (
                    select_action_bar_slot_from_number_keys,
                    cycle_action_bar_slot_from_mouse_wheel,
                    sync_action_bar_highlight,
                    update_player_facing,
                    use_homestead_tool,
                    sync_action_bar_slot_contents.in_set(ActionBarSlotContentsSync),
                    sync_plot_visuals,
                )
                    .chain()
                    .run_if(in_state(GameState::Overworld))
                    .run_if(inventory_closed)
                    .run_if(forge_closed),
            )
            .add_systems(
                Update,
                (
                    start_action_bar_drag,
                    update_action_bar_drag_ghost.after(start_action_bar_drag),
                    release_action_bar_drag.after(update_action_bar_drag_ghost),
                    sync_action_bar_slot_contents
                        .after(release_action_bar_drag)
                        .in_set(ActionBarSlotContentsSync),
                )
                    .run_if(in_state(GameState::Overworld))
                    .run_if(inventory_window_open),
            )
            .add_systems(
                Update,
                clear_emptied_action_bar_slots
                    .after(use_homestead_tool)
                    .before(ActionBarSlotContentsSync)
                    .run_if(in_state(GameState::Overworld)),
            )
            .add_systems(
                Update,
                clear_action_bar_drag
                    .run_if(in_state(GameState::Overworld))
                    .run_if(inventory_closed),
            );
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::mouse::MouseButton;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;
    use bevy::window::{PrimaryWindow, WindowResolution};

    use crate::core::{GameState, PlayerProfile, ProfileDirty, ToolEnergy};
    use crate::farming::ActionBar;
    use crate::input::InputPlugin;
    use crate::items::{Inventory, MaterialId};
    use crate::overworld::setup::{cleanup_overworld, setup_overworld};
    use crate::ui::forge_window::ForgeWindowOpen;
    use crate::ui::inventory_window::{InventorySlot, InventoryWindowOpen};

    use super::super::action_bar_art::{ActionBarSlotCount, ActionBarSlotIcon, ActionBarSlotIndex};
    use super::FarmingPlugin;

    const FRAME_WIDTH: f32 = 498.0;
    const FRAME_HEIGHT: f32 = 78.0;
    const SLOT_SIZE: f32 = 54.0;
    const SLOT_ORIGIN_LEFT: f32 = 12.0;
    const SLOT_ORIGIN_TOP: f32 = 12.0;
    const SLOT_STRIDE: f32 = 60.0;
    const BAR_BOTTOM: f32 = 14.0;
    const WINDOW_WIDTH: f32 = 1280.0;
    const WINDOW_HEIGHT: f32 = 720.0;

    #[test]
    fn releasing_a_hoe_over_a_slot_shows_the_hoe_icon_while_the_inventory_is_open() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);
        app.world_mut().resource_mut::<InventoryWindowOpen>().0 = true;
        put_stack(&mut app, 10, MaterialId::Hoe, 1);

        assert!(app.world().resource::<InventoryWindowOpen>().0);
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(6),
            Ok(None)
        );
        assert_eq!(icon_visibility(&mut app, 6), Visibility::Hidden);
        assert_eq!(count_label(&mut app, 6), "");

        let window = start_drag(&mut app, 10, slot_center(6));
        set_cursor(&mut app, window, Some(slot_center(6)));
        just_release_primary(&mut app);
        app.update();

        assert!(app.world().resource::<InventoryWindowOpen>().0);
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(6),
            Ok(Some(10))
        );
        assert_eq!(icon_visibility(&mut app, 6), Visibility::Visible);
        assert_eq!(icon_image_path(&mut app, 6), "ui/tools/hoe_t1.png");
        assert_eq!(count_label(&mut app, 6), "");
    }

    fn app_ready_for_the_overworld() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            StatesPlugin,
            InputPlugin,
            FarmingPlugin,
        ))
        .init_asset::<Image>()
        .init_asset::<TextureAtlasLayout>()
        .init_state::<GameState>()
        .init_resource::<Inventory>()
        .init_resource::<PlayerProfile>()
        .init_resource::<ProfileDirty>()
        .init_resource::<ToolEnergy>()
        .init_resource::<InventoryWindowOpen>()
        .init_resource::<ForgeWindowOpen>()
        .add_event::<bevy::input::mouse::MouseWheel>()
        .add_systems(OnEnter(GameState::Overworld), setup_overworld)
        .add_systems(OnExit(GameState::Overworld), cleanup_overworld);
        app
    }

    fn enter_overworld(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Overworld);
        app.update();
        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Overworld
        );
    }

    fn put_stack(app: &mut App, inventory_index: usize, material: MaterialId, count: u32) {
        let mut inventory = app.world_mut().resource_mut::<Inventory>();
        inventory.slots[inventory_index].material = Some(material);
        inventory.slots[inventory_index].count = count;
    }

    fn icon_visibility(app: &mut App, slot_index: usize) -> Visibility {
        icon(app, slot_index).0
    }

    fn icon(app: &mut App, slot_index: usize) -> (Visibility, Handle<Image>) {
        let slot = slot_entity(app, slot_index);
        let mut icons = app
            .world_mut()
            .query_filtered::<(&Visibility, &ImageNode, &Parent), With<ActionBarSlotIcon>>();
        let (visibility, image, _) = icons
            .iter(app.world())
            .find(|(_, _, parent)| parent.get() == slot)
            .expect("slot icon");
        (*visibility, image.image.clone())
    }

    fn slot_entity(app: &mut App, slot_index: usize) -> Entity {
        let expected_left = Val::Px(SLOT_ORIGIN_LEFT + slot_index as f32 * SLOT_STRIDE);
        let expected_top = Val::Px(SLOT_ORIGIN_TOP);
        let mut slots = app
            .world_mut()
            .query_filtered::<(Entity, &Node), With<ActionBarSlotIndex>>();
        slots
            .iter(app.world())
            .find_map(|(entity, node)| {
                (node.left == expected_left && node.top == expected_top).then_some(entity)
            })
            .expect("action bar slot")
    }

    fn count_label(app: &mut App, slot_index: usize) -> String {
        let slot = slot_entity(app, slot_index);
        let mut labels = app
            .world_mut()
            .query_filtered::<(&Text, &Parent), With<ActionBarSlotCount>>();
        labels
            .iter(app.world())
            .find_map(|(text, parent)| (parent.get() == slot).then(|| text.0.clone()))
            .expect("slot count")
    }

    fn slot_center(slot_index: usize) -> Vec2 {
        let frame_left = (WINDOW_WIDTH - FRAME_WIDTH) * 0.5;
        let slot_top = WINDOW_HEIGHT - BAR_BOTTOM - FRAME_HEIGHT + SLOT_ORIGIN_TOP;
        let slot_left = frame_left + SLOT_ORIGIN_LEFT + slot_index as f32 * SLOT_STRIDE;
        Vec2::new(slot_left + SLOT_SIZE * 0.5, slot_top + SLOT_SIZE * 0.5)
    }

    fn start_drag(app: &mut App, inventory_index: usize, cursor: Vec2) -> Entity {
        app.world_mut().spawn((
            Button,
            InventorySlot {
                index: inventory_index,
            },
            Interaction::Pressed,
        ));
        let window = spawn_primary_window(app, cursor);
        press_primary(app);
        app.update();
        release_primary(app);
        window
    }

    fn spawn_primary_window(app: &mut App, cursor: Vec2) -> Entity {
        let mut window = Window {
            resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            ..default()
        };
        window.set_cursor_position(Some(cursor));
        app.world_mut().spawn((window, PrimaryWindow)).id()
    }

    fn press_primary(app: &mut App) {
        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Left);
        app.world_mut().insert_resource(mouse);
    }

    fn release_primary(app: &mut App) {
        app.world_mut()
            .insert_resource(ButtonInput::<MouseButton>::default());
    }

    fn set_cursor(app: &mut App, window: Entity, cursor: Option<Vec2>) {
        app.world_mut()
            .get_mut::<Window>(window)
            .expect("primary window")
            .set_cursor_position(cursor);
    }

    fn just_release_primary(app: &mut App) {
        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Left);
        mouse.clear();
        mouse.release(MouseButton::Left);
        app.world_mut().insert_resource(mouse);
    }

    fn icon_image_path(app: &mut App, slot_index: usize) -> String {
        icon(app, slot_index)
            .1
            .path()
            .expect("action bar image keeps its asset path")
            .path()
            .to_string_lossy()
            .into_owned()
    }
}
