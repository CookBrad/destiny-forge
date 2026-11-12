use bevy::prelude::*;
use bevy::ui::widget::{ImageNode, NodeImageMode};

const FRAME_IMAGE_PATH: &str = "ui/action_bar/frame.png";
const FRAME_WIDTH: f32 = 498.0;
const FRAME_HEIGHT: f32 = 78.0;
const BAR_BOTTOM: f32 = 14.0;

pub(super) fn load_action_bar_art(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ActionBarArt {
        frame: asset_server.load(FRAME_IMAGE_PATH),
    });
}

pub(super) fn setup_action_bar_hud(mut commands: Commands, art: Res<ActionBarArt>) {
    commands.spawn(action_bar_root()).with_children(|root| {
        root.spawn(action_bar_frame(&art));
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
        ImageNode::new(art.frame.clone()).with_mode(NodeImageMode::Stretch),
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
}

#[derive(Component)]
pub(super) struct ActionBarHud;

#[derive(Component)]
struct ActionBarFrame;

#[cfg(test)]
mod tests {
    use bevy::asset::AssetApp;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;
    use bevy::ui::widget::NodeImageMode;

    use super::{ActionBarArt, ActionBarFrame, ActionBarHud};

    const FRAME_IMAGE_PATH: &str = "ui/action_bar/frame.png";

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
            .expect("frame image keeps its asset path")
            .path()
            .to_string_lossy()
            .into_owned()
    }
}
