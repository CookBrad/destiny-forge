use bevy::hierarchy::ChildBuilder;
use bevy::prelude::*;
use bevy::ui::widget::{ImageNode, NodeImageMode};

use crate::items::{Inventory, MaterialId};

use super::action_bar::ACTION_BAR_SLOT_COUNT;
use super::ActionBar;

const FRAME_IMAGE_PATH: &str = "ui/action_bar/frame.png";
const HIGHLIGHT_IMAGE_PATH: &str = "ui/action_bar/highlight.png";
const SLOT_IMAGE_PATHS: [&str; ACTION_BAR_SLOT_COUNT] = [
    "ui/action_bar/slot_1.png",
    "ui/action_bar/slot_2.png",
    "ui/action_bar/slot_3.png",
    "ui/action_bar/slot_4.png",
    "ui/action_bar/slot_5.png",
    "ui/action_bar/slot_6.png",
    "ui/action_bar/slot_7.png",
    "ui/action_bar/slot_8.png",
];
const FRAME_WIDTH: f32 = 498.0;
const FRAME_HEIGHT: f32 = 78.0;
const SLOT_SIZE: f32 = 54.0;
const SLOT_ORIGIN_LEFT: f32 = 12.0;
const SLOT_ORIGIN_TOP: f32 = 12.0;
const SLOT_STRIDE: f32 = 60.0;
const BAR_BOTTOM: f32 = 14.0;

pub(super) fn load_action_bar_art(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ActionBarArt {
        frame: asset_server.load(FRAME_IMAGE_PATH),
        highlight: asset_server.load(HIGHLIGHT_IMAGE_PATH),
        slots: load_slot_images(&asset_server),
        hoe_icon: asset_server.load("ui/tools/hoe_t1.png"),
    });
}

fn load_slot_images(asset_server: &AssetServer) -> [Handle<Image>; ACTION_BAR_SLOT_COUNT] {
    SLOT_IMAGE_PATHS.map(|path| asset_server.load(path))
}

pub(super) fn setup_action_bar_hud(
    mut commands: Commands,
    art: Res<ActionBarArt>,
    action_bar: Res<ActionBar>,
    inventory: Res<Inventory>,
) {
    let selected_slot = action_bar.selected_slot();
    commands.spawn(action_bar_root()).with_children(|root| {
        root.spawn(action_bar_frame(&art)).with_children(|frame| {
            fill_action_bar_frame(frame, &art, &action_bar, &inventory, selected_slot);
        });
    });
}

fn action_bar_root() -> (ActionBarHud, Node) {
    (
        ActionBarHud,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(BAR_BOTTOM),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    )
}

fn action_bar_frame(art: &ActionBarArt) -> (ActionBarFrame, ImageNode, Node, BackgroundColor) {
    (
        ActionBarFrame,
        stretched_image(art.frame.clone()),
        Node {
            position_type: PositionType::Relative,
            width: Val::Px(FRAME_WIDTH),
            height: Val::Px(FRAME_HEIGHT),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(Color::NONE),
    )
}

fn stretched_image(image: Handle<Image>) -> ImageNode {
    ImageNode::new(image).with_mode(NodeImageMode::Stretch)
}

fn fill_action_bar_frame(
    frame: &mut ChildBuilder<'_>,
    art: &ActionBarArt,
    action_bar: &ActionBar,
    inventory: &Inventory,
    selected_slot: usize,
) {
    spawn_action_bar_slots(frame, art, action_bar, inventory);
    frame.spawn(action_bar_highlight(art, selected_slot));
}

fn spawn_action_bar_slots(
    frame: &mut ChildBuilder<'_>,
    art: &ActionBarArt,
    action_bar: &ActionBar,
    inventory: &Inventory,
) {
    for slot_index in 0..ACTION_BAR_SLOT_COUNT {
        let (material, count) = held_material(action_bar, inventory, slot_index);
        frame
            .spawn(action_bar_slot(art, slot_index))
            .with_children(|slot| {
                slot.spawn(hoe_icon(art, hoe_visibility(material, count)));
                slot.spawn(stack_count_label(stack_count_text(material, count)));
            });
    }
}

fn action_bar_slot(
    art: &ActionBarArt,
    slot_index: usize,
) -> (
    ActionBarSlotImage,
    ActionBarSlotIndex,
    ImageNode,
    Node,
    BackgroundColor,
) {
    let (image, node, background) = action_bar_piece(art.slots[slot_index].clone(), slot_index);
    (
        ActionBarSlotImage,
        ActionBarSlotIndex(slot_index),
        image,
        node,
        background,
    )
}

fn action_bar_highlight(
    art: &ActionBarArt,
    selected_slot: usize,
) -> (ActionBarHighlight, ImageNode, Node, BackgroundColor) {
    let (image, node, background) = action_bar_piece(art.highlight.clone(), selected_slot);
    (ActionBarHighlight, image, node, background)
}

fn action_bar_piece(image: Handle<Image>, slot_index: usize) -> (ImageNode, Node, BackgroundColor) {
    (
        stretched_image(image),
        slot_node(slot_index),
        BackgroundColor(Color::NONE),
    )
}

fn slot_node(slot_index: usize) -> Node {
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(SLOT_SIZE),
        height: Val::Px(SLOT_SIZE),
        flex_shrink: 0.0,
        ..default()
    };
    place_on_slot(&mut node, slot_index);
    node
}

fn place_on_slot(node: &mut Node, slot_index: usize) {
    node.left = Val::Px(slot_left(slot_index));
    node.top = Val::Px(SLOT_ORIGIN_TOP);
}

fn slot_left(slot_index: usize) -> f32 {
    SLOT_ORIGIN_LEFT + slot_index as f32 * SLOT_STRIDE
}

pub(super) fn sync_action_bar_highlight(
    action_bar: Res<ActionBar>,
    mut highlights: Query<&mut Node, With<ActionBarHighlight>>,
) {
    if !action_bar.is_changed() {
        return;
    }
    let selected_slot = action_bar.selected_slot();
    for mut highlight in &mut highlights {
        place_on_slot(&mut highlight, selected_slot);
    }
}

pub(super) fn sync_action_bar_slot_contents(
    mut action_bar: ResMut<ActionBar>,
    inventory: Res<Inventory>,
    slots: Query<(&ActionBarSlotIndex, &Children)>,
    mut icons: Query<&mut Visibility, With<ActionBarSlotIcon>>,
    mut counts: Query<&mut Text, With<ActionBarSlotCount>>,
) {
    if !action_bar.is_changed() && !inventory.is_changed() {
        return;
    }
    for (slot_index, children) in &slots {
        let (material, count) = held_material(&action_bar, &inventory, slot_index.0);
        if count == 0 && matches!(action_bar.held_inventory_slot(slot_index.0), Ok(Some(_))) {
            let _ = action_bar.clear_held_inventory_slot(slot_index.0);
        }
        for child in children.iter() {
            if let Ok(mut visibility) = icons.get_mut(*child) {
                *visibility = hoe_visibility(material, count);
            }
            if let Ok(mut text) = counts.get_mut(*child) {
                text.0 = stack_count_text(material, count);
            }
        }
    }
}

fn hoe_icon(
    art: &ActionBarArt,
    visibility: Visibility,
) -> (ActionBarSlotIcon, ImageNode, Node, Visibility) {
    (
        ActionBarSlotIcon,
        ImageNode::new(art.hoe_icon.clone()),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(19.0),
            top: Val::Px(19.0),
            width: Val::Px(16.0),
            height: Val::Px(16.0),
            ..default()
        },
        visibility,
    )
}

fn hoe_visibility(material: Option<MaterialId>, count: u32) -> Visibility {
    if material == Some(MaterialId::Hoe) && count > 0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

fn stack_count_label(text: String) -> (ActionBarSlotCount, Text, TextFont, TextColor, Node) {
    (
        ActionBarSlotCount,
        Text::new(text),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextColor(Color::srgb(0.98, 0.98, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(3.0),
            bottom: Val::Px(1.0),
            ..default()
        },
    )
}

fn held_material(
    action_bar: &ActionBar,
    inventory: &Inventory,
    slot_index: usize,
) -> (Option<MaterialId>, u32) {
    let Ok(Some(inventory_index)) = action_bar.held_inventory_slot(slot_index) else {
        return (None, 0);
    };
    let Some(stack) = inventory.slots.get(inventory_index) else {
        return (None, 0);
    };
    (stack.material, stack.count)
}

fn stack_count_text(material: Option<MaterialId>, count: u32) -> String {
    match material {
        Some(MaterialId::TurnipSeed | MaterialId::PotatoSeed) if count > 1 => count.to_string(),
        _ => String::new(),
    }
}

pub(super) fn cleanup_action_bar_hud(
    mut commands: Commands,
    roots: Query<Entity, With<ActionBarHud>>,
) {
    for root in &roots {
        commands.entity(root).try_despawn_recursive();
    }
}

#[derive(Resource)]
pub(super) struct ActionBarArt {
    frame: Handle<Image>,
    highlight: Handle<Image>,
    slots: [Handle<Image>; ACTION_BAR_SLOT_COUNT],
    hoe_icon: Handle<Image>,
}

#[derive(Component)]
pub(super) struct ActionBarHud;

#[derive(Component)]
struct ActionBarFrame;

#[derive(Component)]
struct ActionBarSlotImage;

#[derive(Component, Clone, Copy)]
pub(super) struct ActionBarSlotIndex(usize);

#[derive(Component)]
pub(super) struct ActionBarSlotIcon;

#[derive(Component)]
pub(super) struct ActionBarSlotCount;

#[derive(Component)]
pub(super) struct ActionBarHighlight;

#[cfg(test)]
mod tests {
    use bevy::asset::AssetApp;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;
    use bevy::ui::widget::NodeImageMode;

    use super::{
        ActionBarArt, ActionBarFrame, ActionBarHighlight, ActionBarHud, ActionBarSlotCount,
        ActionBarSlotIcon, ActionBarSlotImage, ActionBarSlotIndex, ACTION_BAR_SLOT_COUNT,
    };
    use crate::farming::{ActionBar, ActionBarSlotOutOfRange};
    use crate::items::{Inventory, MaterialId};

    const FRAME_IMAGE_PATH: &str = "ui/action_bar/frame.png";
    const HIGHLIGHT_IMAGE_PATH: &str = "ui/action_bar/highlight.png";

    #[test]
    fn the_title_screen_does_not_load_or_show_the_action_bar_frame() {
        let mut app = app_ready_for_the_overworld();
        app.update();
        assert!(app.world().get_resource::<ActionBarArt>().is_none());
        assert_eq!(action_bar_root_count(&mut app), 0);
        assert_eq!(action_bar_frame_count(&mut app), 0);
        assert_eq!(text_tool_root_count(&mut app), 0);
    }

    #[test]
    fn the_overworld_shows_the_loaded_frame_centered_at_the_bottom() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);

        let loaded_frame = app.world().resource::<ActionBarArt>().frame.clone();
        assert_eq!(image_path(&loaded_frame), FRAME_IMAGE_PATH);
        assert_eq!(action_bar_root_count(&mut app), 1);
        assert_eq!(action_bar_frame_count(&mut app), 1);
        assert_eq!(root_child_count(&mut app), 1);
        assert_eq!(text_tool_root_count(&mut app), 1);

        let root = action_bar_root_node(&mut app);
        assert_eq!(root.position_type, PositionType::Absolute);
        assert_eq!(root.bottom, Val::Px(14.0));
        assert_eq!(root.width, Val::Percent(100.0));
        assert_eq!(root.justify_content, JustifyContent::Center);

        let frame = action_bar_frame_node(&mut app);
        assert_eq!(frame.position_type, PositionType::Relative);
        assert_eq!(frame.width, Val::Px(498.0));
        assert_eq!(frame.height, Val::Px(78.0));
        assert_eq!(frame.flex_shrink, 0.0);

        let image = action_bar_frame_image(&mut app);
        assert_eq!(image.image, loaded_frame);
        assert_eq!(image.color, Color::WHITE);
        assert!(matches!(image.image_mode, NodeImageMode::Stretch));
        assert_eq!(action_bar_frame_background(&mut app), Color::NONE);
        assert!(frame_parent_is_the_root(&mut app));
    }

    #[test]
    fn leaving_the_overworld_despawns_the_frame_and_entering_again_spawns_it() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);
        assert_eq!(action_bar_frame_count(&mut app), 1);

        leave_overworld(&mut app);
        assert_eq!(action_bar_root_count(&mut app), 0);
        assert_eq!(action_bar_frame_count(&mut app), 0);
        assert_eq!(text_tool_root_count(&mut app), 0);

        enter_overworld(&mut app);
        assert_eq!(action_bar_root_count(&mut app), 1);
        assert_eq!(action_bar_frame_count(&mut app), 1);
        assert_eq!(root_child_count(&mut app), 1);
        assert_eq!(text_tool_root_count(&mut app), 1);
        let image = action_bar_frame_image(&mut app);
        assert_eq!(image.image, app.world().resource::<ActionBarArt>().frame);
        assert_eq!(image_path(&image.image), FRAME_IMAGE_PATH);
    }

    #[test]
    fn the_title_screen_does_not_show_slots_or_the_highlight() {
        let mut app = app_ready_for_the_overworld();
        app.update();
        assert_eq!(action_bar_slot_count(&mut app), 0);
        assert_eq!(action_bar_highlight_count(&mut app), 0);
    }

    #[test]
    fn the_overworld_shows_eight_slots_and_a_highlight_on_slot_zero() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);
        assert_eight_slots_at_native_offsets(&mut app);
        assert_highlight_on_slot(&mut app, 0);
        assert_eq!(frame_child_count(&mut app), ACTION_BAR_SLOT_COUNT + 1);
        assert!(highlight_is_the_last_frame_child(&mut app));
    }

    #[test]
    fn changing_the_selected_slot_moves_the_highlight_onto_each_slot() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);
        let highlight = highlight_entity(&mut app);
        for slot_index in 0..ACTION_BAR_SLOT_COUNT {
            select_slot(&mut app, slot_index);
            assert_eq!(highlight_entity(&mut app), highlight);
            assert_highlight_on_slot(&mut app, slot_index);
        }
    }

    #[test]
    fn a_rejected_slot_leaves_the_highlight_on_the_previous_slot() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);
        select_slot(&mut app, 4);

        let rejected = app
            .world_mut()
            .resource_mut::<ActionBar>()
            .select_slot(ACTION_BAR_SLOT_COUNT);
        assert_eq!(rejected, Err(ActionBarSlotOutOfRange));
        app.update();
        assert_eq!(app.world().resource::<ActionBar>().selected_slot(), 4);
        assert_highlight_on_slot(&mut app, 4);

        let rejected = app
            .world_mut()
            .resource_mut::<ActionBar>()
            .select_slot(usize::MAX);
        assert_eq!(rejected, Err(ActionBarSlotOutOfRange));
        app.update();
        assert_highlight_on_slot(&mut app, 4);
    }

    #[test]
    fn leaving_the_overworld_despawns_slots_and_the_highlight_and_entering_again_spawns_them() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);
        select_slot(&mut app, 5);

        leave_overworld(&mut app);
        assert_eq!(action_bar_slot_count(&mut app), 0);
        assert_eq!(action_bar_highlight_count(&mut app), 0);

        enter_overworld(&mut app);
        assert_eight_slots_at_native_offsets(&mut app);
        assert_highlight_on_slot(&mut app, 5);
    }

    #[test]
    fn the_title_screen_shows_no_slot_icons_stack_labels_or_the_old_tool_list() {
        let mut app = app_ready_for_the_overworld();
        app.update();
        assert_eq!(slot_icon_count(&mut app), 0);
        assert_eq!(slot_count_label_count(&mut app), 0);
        assert!(!any_text_contains(&mut app, "Watering Can"));
        assert!(!any_text_contains(&mut app, "[1 Hoe]"));
    }

    #[test]
    fn the_overworld_starts_with_every_slot_icon_hidden_and_every_stack_label_empty() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);
        assert_eq!(visible_slot_icon_count(&mut app), 0);
        for slot_index in 0..ACTION_BAR_SLOT_COUNT {
            assert_eq!(
                icon_on_slot(&mut app, slot_index).visibility,
                Visibility::Hidden
            );
            assert_eq!(count_label_on_slot(&mut app, slot_index).text, "");
        }
        assert_highlight_on_slot(&mut app, 0);
    }

    #[test]
    fn a_slot_that_holds_a_hoe_shows_the_hoe_icon_centered_at_its_native_size() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);
        put_stack(&mut app, 10, MaterialId::Hoe, 1);
        assign_held_slot(&mut app, 6, 10);

        let icon = icon_on_slot(&mut app, 6);
        let hoe = app.world().resource::<ActionBarArt>().hoe_icon.clone();
        assert_eq!(icon.visibility, Visibility::Visible);
        assert_eq!(icon.image.image, hoe);
        assert_eq!(image_path(&icon.image.image), "ui/tools/hoe_t1.png");
        assert!(matches!(icon.image.image_mode, NodeImageMode::Auto));
        assert_eq!(icon.node.position_type, PositionType::Absolute);
        assert_eq!(icon.node.left, Val::Px(19.0));
        assert_eq!(icon.node.top, Val::Px(19.0));
        assert_eq!(icon.node.width, Val::Px(16.0));
        assert_eq!(icon.node.height, Val::Px(16.0));
        assert_eq!(icon.parent, slot_entity(&mut app, 6));
        assert_ne!(icon.parent, frame_entity(&mut app));
        assert_eq!(count_label_on_slot(&mut app, 6).text, "");
        for slot_index in 0..ACTION_BAR_SLOT_COUNT {
            if slot_index == 6 {
                continue;
            }
            assert_eq!(
                icon_on_slot(&mut app, slot_index).visibility,
                Visibility::Hidden
            );
        }

        app.update();
        assert_eq!(icon_on_slot(&mut app, 6).visibility, Visibility::Visible);
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(6),
            Ok(Some(10))
        );

        clear_held_slot(&mut app, 6);
        assert_eq!(icon_on_slot(&mut app, 6).visibility, Visibility::Hidden);
    }

    #[test]
    fn a_watering_can_or_seed_stack_shows_no_icon() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);
        put_stack(&mut app, 11, MaterialId::WateringCan, 1);
        put_stack(&mut app, 12, MaterialId::TurnipSeed, 8);
        put_stack(&mut app, 13, MaterialId::PotatoSeed, 4);
        assign_held_slot(&mut app, 2, 11);
        assign_held_slot(&mut app, 3, 12);
        assign_held_slot(&mut app, 4, 13);

        assert_eq!(icon_on_slot(&mut app, 2).visibility, Visibility::Hidden);
        assert_eq!(icon_on_slot(&mut app, 3).visibility, Visibility::Hidden);
        assert_eq!(icon_on_slot(&mut app, 4).visibility, Visibility::Hidden);
    }

    #[test]
    fn a_seed_stack_shows_its_count_only_when_the_count_is_greater_than_one() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);
        put_stack(&mut app, 12, MaterialId::TurnipSeed, 8);
        assign_held_slot(&mut app, 3, 12);
        let label = count_label_on_slot(&mut app, 3);
        assert_eq!(label.text, "8");
        assert_eq!(label.font.font_size, 11.0);
        assert_eq!(label.color.0, Color::srgb(0.98, 0.98, 1.0));
        assert_eq!(label.node.position_type, PositionType::Absolute);
        assert_eq!(label.node.right, Val::Px(3.0));
        assert_eq!(label.node.bottom, Val::Px(1.0));

        put_stack(&mut app, 12, MaterialId::TurnipSeed, 1);
        app.update();
        assert_eq!(count_label_on_slot(&mut app, 3).text, "");

        put_stack(&mut app, 12, MaterialId::TurnipSeed, 0);
        app.update();
        assert_eq!(count_label_on_slot(&mut app, 3).text, "");

        clear_held_slot(&mut app, 3);
        assert_eq!(count_label_on_slot(&mut app, 3).text, "");

        put_stack(&mut app, 10, MaterialId::Hoe, 1);
        assign_held_slot(&mut app, 6, 10);
        assert_eq!(count_label_on_slot(&mut app, 6).text, "");

        put_stack(&mut app, 13, MaterialId::PotatoSeed, 4);
        assign_held_slot(&mut app, 4, 13);
        assert_eq!(count_label_on_slot(&mut app, 4).text, "4");
    }

    #[test]
    fn leaving_the_overworld_despawns_slot_icons_and_labels_and_entering_again_shows_them() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);
        put_stack(&mut app, 10, MaterialId::Hoe, 1);
        assign_held_slot(&mut app, 5, 10);
        assert_eq!(icon_on_slot(&mut app, 5).visibility, Visibility::Visible);

        leave_overworld(&mut app);
        assert_eq!(slot_icon_count(&mut app), 0);
        assert_eq!(slot_count_label_count(&mut app), 0);
        assert_eq!(
            app.world().resource::<ActionBar>().held_inventory_slot(5),
            Ok(Some(10))
        );

        enter_overworld(&mut app);
        let icon = icon_on_slot(&mut app, 5);
        assert_eq!(icon.visibility, Visibility::Visible);
        assert_eq!(image_path(&icon.image.image), "ui/tools/hoe_t1.png");
        assert_eq!(icon.node.width, Val::Px(16.0));
        assert_eq!(icon.node.height, Val::Px(16.0));
        assert_eq!(count_label_on_slot(&mut app, 5).text, "");
    }

    fn app_ready_for_the_overworld() -> App {
        let mut app = app_with_an_asset_server();
        app.add_plugins((
            StatesPlugin,
            crate::input::InputPlugin,
            crate::farming::FarmingPlugin,
        ))
        .init_asset::<TextureAtlasLayout>()
        .init_state::<crate::core::GameState>()
        .init_resource::<crate::items::Inventory>()
        .init_resource::<crate::core::PlayerProfile>()
        .init_resource::<crate::core::ProfileDirty>()
        .init_resource::<crate::core::ToolEnergy>()
        .init_resource::<crate::ui::inventory_window::InventoryWindowOpen>()
        .init_resource::<crate::ui::forge_window::ForgeWindowOpen>()
        .add_event::<bevy::input::mouse::MouseWheel>()
        .add_systems(
            OnEnter(crate::core::GameState::Overworld),
            crate::overworld::setup::setup_overworld,
        )
        .add_systems(
            OnExit(crate::core::GameState::Overworld),
            crate::overworld::setup::cleanup_overworld,
        );
        app
    }

    fn app_with_an_asset_server() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Image>();
        app
    }

    fn enter_overworld(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<crate::core::GameState>>()
            .set(crate::core::GameState::Overworld);
        app.update();
        assert_eq!(
            app.world()
                .resource::<State<crate::core::GameState>>()
                .get(),
            &crate::core::GameState::Overworld
        );
    }

    fn leave_overworld(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<crate::core::GameState>>()
            .set(crate::core::GameState::Title);
        app.update();
        assert_eq!(
            app.world()
                .resource::<State<crate::core::GameState>>()
                .get(),
            &crate::core::GameState::Title
        );
    }

    fn action_bar_root_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ActionBarHud>()
            .iter(app.world())
            .count()
    }

    fn action_bar_frame_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ActionBarFrame>()
            .iter(app.world())
            .count()
    }

    fn text_tool_root_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&crate::farming::hud::ToolHudRoot>()
            .iter(app.world())
            .count()
    }

    fn action_bar_root_node(app: &mut App) -> Node {
        app.world_mut()
            .query_filtered::<&Node, With<ActionBarHud>>()
            .single(app.world())
            .clone()
    }

    fn action_bar_frame_node(app: &mut App) -> Node {
        app.world_mut()
            .query_filtered::<&Node, With<ActionBarFrame>>()
            .single(app.world())
            .clone()
    }

    fn action_bar_frame_image(app: &mut App) -> ImageNode {
        app.world_mut()
            .query_filtered::<&ImageNode, With<ActionBarFrame>>()
            .single(app.world())
            .clone()
    }

    fn action_bar_frame_background(app: &mut App) -> Color {
        app.world_mut()
            .query_filtered::<&BackgroundColor, With<ActionBarFrame>>()
            .single(app.world())
            .0
    }

    fn frame_parent_is_the_root(app: &mut App) -> bool {
        let root = app
            .world_mut()
            .query_filtered::<Entity, With<ActionBarHud>>()
            .single(app.world());
        let parent = app
            .world_mut()
            .query_filtered::<&Parent, With<ActionBarFrame>>()
            .single(app.world())
            .get();
        parent == root
    }

    fn root_child_count(app: &mut App) -> usize {
        let root = app
            .world_mut()
            .query_filtered::<Entity, With<ActionBarHud>>()
            .single(app.world());
        app.world()
            .get::<Children>(root)
            .map(|children| children.len())
            .unwrap_or(0)
    }

    fn image_path(image: &Handle<Image>) -> String {
        image
            .path()
            .expect("action bar image keeps its asset path")
            .path()
            .to_string_lossy()
            .into_owned()
    }

    fn assert_eight_slots_at_native_offsets(app: &mut App) {
        assert_eq!(action_bar_slot_count(app), ACTION_BAR_SLOT_COUNT);
        for slot_index in 0..ACTION_BAR_SLOT_COUNT {
            assert_slot_at_native_offset(app, slot_index);
        }
    }

    fn assert_slot_at_native_offset(app: &mut App, slot_index: usize) {
        let loaded_slot = app.world().resource::<ActionBarArt>().slots[slot_index].clone();
        assert_eq!(image_path(&loaded_slot), slot_image_path(slot_index));
        let placed = slot_row(app, &loaded_slot);
        assert_eq!(placed.image.image, loaded_slot);
        assert_piece_on_slot(app, &placed, slot_index);
    }

    fn assert_highlight_on_slot(app: &mut App, slot_index: usize) {
        let loaded_highlight = app.world().resource::<ActionBarArt>().highlight.clone();
        assert_eq!(image_path(&loaded_highlight), HIGHLIGHT_IMAGE_PATH);
        let placed = highlight_row(app);
        assert_eq!(placed.image.image, loaded_highlight);
        assert_piece_on_slot(app, &placed, slot_index);
        assert_eq!(action_bar_highlight_count(app), 1);
    }

    fn assert_piece_on_slot(app: &mut App, placed: &PlacedPiece, slot_index: usize) {
        assert_eq!(placed.image.color, Color::WHITE);
        assert!(matches!(placed.image.image_mode, NodeImageMode::Stretch));
        assert_eq!(placed.node.position_type, PositionType::Absolute);
        assert_eq!(placed.node.left, native_slot_left(slot_index));
        assert_eq!(placed.node.top, native_slot_top());
        assert_eq!(placed.node.width, Val::Px(54.0));
        assert_eq!(placed.node.height, Val::Px(54.0));
        assert_eq!(placed.node.flex_shrink, 0.0);
        assert_eq!(placed.background, Color::NONE);
        assert_eq!(placed.parent, frame_entity(app));
    }

    fn slot_row(app: &mut App, loaded_slot: &Handle<Image>) -> PlacedPiece {
        let mut slots = app.world_mut().query_filtered::<(
            &ImageNode,
            &Node,
            &BackgroundColor,
            &Parent,
        ), With<ActionBarSlotImage>>();
        slots
            .iter(app.world())
            .find(|(image, _, _, _)| image.image == *loaded_slot)
            .map(|(image, node, background, parent)| PlacedPiece {
                image: image.clone(),
                node: node.clone(),
                background: background.0,
                parent: parent.get(),
            })
            .expect("action bar slot")
    }

    fn highlight_row(app: &mut App) -> PlacedPiece {
        let mut highlights = app.world_mut().query_filtered::<(
            &ImageNode,
            &Node,
            &BackgroundColor,
            &Parent,
        ), With<ActionBarHighlight>>();
        let (image, node, background, parent) = highlights.single(app.world());
        PlacedPiece {
            image: image.clone(),
            node: node.clone(),
            background: background.0,
            parent: parent.get(),
        }
    }

    fn action_bar_slot_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ActionBarSlotImage>()
            .iter(app.world())
            .count()
    }

    fn action_bar_highlight_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ActionBarHighlight>()
            .iter(app.world())
            .count()
    }

    fn highlight_entity(app: &mut App) -> Entity {
        app.world_mut()
            .query_filtered::<Entity, With<ActionBarHighlight>>()
            .single(app.world())
    }

    fn frame_entity(app: &mut App) -> Entity {
        app.world_mut()
            .query_filtered::<Entity, With<ActionBarFrame>>()
            .single(app.world())
    }

    fn frame_child_count(app: &mut App) -> usize {
        let frame = frame_entity(app);
        app.world()
            .get::<Children>(frame)
            .map(|children| children.len())
            .unwrap_or(0)
    }

    fn highlight_is_the_last_frame_child(app: &mut App) -> bool {
        let frame = frame_entity(app);
        let children = app.world().get::<Children>(frame).expect("frame children");
        let last = *children.last().expect("highlight child");
        app.world().get::<ActionBarHighlight>(last).is_some()
    }

    fn select_slot(app: &mut App, slot_index: usize) {
        let selected = app
            .world_mut()
            .resource_mut::<ActionBar>()
            .select_slot(slot_index);
        assert_eq!(selected, Ok(()));
        app.update();
    }

    fn slot_image_path(slot_index: usize) -> String {
        format!("ui/action_bar/slot_{}.png", slot_index + 1)
    }

    fn native_slot_left(slot_index: usize) -> Val {
        Val::Px(12.0 + slot_index as f32 * 60.0)
    }

    fn native_slot_top() -> Val {
        Val::Px(12.0)
    }

    struct PlacedPiece {
        image: ImageNode,
        node: Node,
        background: Color,
        parent: Entity,
    }

    struct ShownIcon {
        visibility: Visibility,
        image: ImageNode,
        node: Node,
        parent: Entity,
    }

    struct ShownCount {
        text: String,
        font: TextFont,
        color: TextColor,
        node: Node,
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
        app.update();
    }

    fn clear_held_slot(app: &mut App, slot_index: usize) {
        let cleared = app
            .world_mut()
            .resource_mut::<ActionBar>()
            .clear_held_inventory_slot(slot_index);
        assert_eq!(cleared, Ok(()));
        app.update();
    }

    fn icon_on_slot(app: &mut App, slot_index: usize) -> ShownIcon {
        let slot = slot_entity(app, slot_index);
        let mut icons = app
            .world_mut()
            .query_filtered::<(&Visibility, &ImageNode, &Node, &Parent), With<ActionBarSlotIcon>>();
        let (visibility, image, node, parent) = icons
            .iter(app.world())
            .find(|(_, _, _, parent)| parent.get() == slot)
            .expect("slot icon");
        ShownIcon {
            visibility: *visibility,
            image: image.clone(),
            node: node.clone(),
            parent: parent.get(),
        }
    }

    fn count_label_on_slot(app: &mut App, slot_index: usize) -> ShownCount {
        let slot = slot_entity(app, slot_index);
        let mut labels = app.world_mut().query_filtered::<(
            &Text,
            &TextFont,
            &TextColor,
            &Node,
            &Parent,
        ), With<ActionBarSlotCount>>();
        let (text, font, color, node, _) = labels
            .iter(app.world())
            .find(|(_, _, _, _, parent)| parent.get() == slot)
            .expect("stack count");
        ShownCount {
            text: text.0.clone(),
            font: font.clone(),
            color: *color,
            node: node.clone(),
        }
    }

    fn slot_entity(app: &mut App, slot_index: usize) -> Entity {
        let mut slots = app.world_mut().query::<(Entity, &ActionBarSlotIndex)>();
        slots
            .iter(app.world())
            .find(|(_, index)| index.0 == slot_index)
            .map(|(entity, _)| entity)
            .expect("action bar slot")
    }

    fn slot_icon_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ActionBarSlotIcon>()
            .iter(app.world())
            .count()
    }

    fn slot_count_label_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&ActionBarSlotCount>()
            .iter(app.world())
            .count()
    }

    fn visible_slot_icon_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<&Visibility, With<ActionBarSlotIcon>>()
            .iter(app.world())
            .filter(|visibility| **visibility == Visibility::Visible)
            .count()
    }

    fn any_text_contains(app: &mut App, needle: &str) -> bool {
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0.contains(needle))
    }
}
