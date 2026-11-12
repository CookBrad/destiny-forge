use bevy::prelude::*;

use super::action_bar::ACTION_BAR_SLOT_COUNT;

const FRAME_IMAGE_PATH: &str = "ui/action_bar/frame.png";
const HIGHLIGHT_IMAGE_PATH: &str = "ui/action_bar/highlight.png";

pub(super) fn load_action_bar_art(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(action_bar_art(&asset_server));
}

fn action_bar_art(asset_server: &AssetServer) -> ActionBarArt {
    let art = ActionBarArt {
        frame: asset_server.load(FRAME_IMAGE_PATH),
        highlight: asset_server.load(HIGHLIGHT_IMAGE_PATH),
        slots: slot_images(asset_server),
    };
    require_loaded_action_bar_images(&art);
    art
}

fn slot_images(asset_server: &AssetServer) -> [Handle<Image>; ACTION_BAR_SLOT_COUNT] {
    std::array::from_fn(|slot_index| asset_server.load(slot_image_path(slot_index)))
}

fn slot_image_path(slot_index: usize) -> String {
    format!("ui/action_bar/slot_{}.png", slot_index + 1)
}

fn require_loaded_action_bar_images(art: &ActionBarArt) {
    require_image_path(&art.frame_image()).expect("action bar frame keeps its asset path");
    require_image_path(&art.highlight_image()).expect("action bar highlight keeps its asset path");
    for slot_index in 0..ACTION_BAR_SLOT_COUNT {
        let slot_image = art
            .slot_image(slot_index)
            .expect("action bar slot keeps an image");
        require_image_path(&slot_image).expect("action bar slot keeps its asset path");
    }
}

fn require_image_path(image: &Handle<Image>) -> Result<(), MissingActionBarImagePath> {
    if image.path().is_none() {
        Err(MissingActionBarImagePath)
    } else {
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MissingActionBarImagePath;

#[derive(Resource, Clone)]
pub(super) struct ActionBarArt {
    frame: Handle<Image>,
    highlight: Handle<Image>,
    slots: [Handle<Image>; ACTION_BAR_SLOT_COUNT],
}

impl ActionBarArt {
    pub(super) fn frame_image(&self) -> Handle<Image> {
        self.frame.clone()
    }

    pub(super) fn highlight_image(&self) -> Handle<Image> {
        self.highlight.clone()
    }

    pub(super) fn slot_image(&self, slot_index: usize) -> Option<Handle<Image>> {
        self.slots.get(slot_index).cloned()
    }
}

#[cfg(test)]
mod tests {
    use bevy::asset::AssetApp;
    use bevy::input::mouse::MouseWheel;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use super::ActionBarArt;
    use crate::core::{GameState, PlayerProfile, ProfileDirty, ToolEnergy};
    use crate::farming::action_bar::ACTION_BAR_SLOT_COUNT;
    use crate::farming::FarmingPlugin;
    use crate::input::InputPlugin;
    use crate::items::Inventory;
    use crate::overworld::setup::{cleanup_overworld, setup_overworld};
    use crate::ui::forge_window::ForgeWindowOpen;
    use crate::ui::inventory_window::InventoryWindowOpen;

    const FRAME_IMAGE_PATH: &str = "ui/action_bar/frame.png";
    const HIGHLIGHT_IMAGE_PATH: &str = "ui/action_bar/highlight.png";

    #[test]
    fn the_title_screen_does_not_load_the_action_bar_images() {
        let mut app = app_ready_for_the_overworld();
        app.update();
        assert!(app.world().get_resource::<ActionBarArt>().is_none());
    }

    #[test]
    fn the_overworld_loads_the_frame_the_highlight_and_all_eight_slots() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);

        let art = app.world().resource::<ActionBarArt>().clone();
        assert_eq!(loaded_image_paths(&art), expected_image_paths());
        assert_eq!(unique_handle_count(&art), expected_image_paths().len());
        for image_path in expected_image_paths() {
            assert_png_file(&image_path);
        }
    }

    #[test]
    fn entering_the_overworld_again_loads_the_same_ten_image_paths() {
        let mut app = app_ready_for_the_overworld();
        enter_overworld(&mut app);
        let first_paths = loaded_image_paths(app.world().resource::<ActionBarArt>());

        leave_overworld(&mut app);
        enter_overworld(&mut app);

        assert_eq!(
            loaded_image_paths(app.world().resource::<ActionBarArt>()),
            first_paths
        );
    }

    #[test]
    fn slot_image_returns_the_first_and_last_slots_and_rejects_an_index_past_the_end() {
        let app = app_with_an_asset_server();
        let art = super::action_bar_art(app.world().resource::<AssetServer>());

        let first = art.slot_image(0).expect("first slot");
        let last = art
            .slot_image(ACTION_BAR_SLOT_COUNT - 1)
            .expect("last slot");
        assert_eq!(asset_path_of(&first), "ui/action_bar/slot_1.png");
        assert_eq!(asset_path_of(&last), "ui/action_bar/slot_8.png");
        assert_ne!(first, last);
        assert!(art.slot_image(ACTION_BAR_SLOT_COUNT).is_none());
        assert!(art.slot_image(usize::MAX).is_none());
        assert_eq!(
            super::require_image_path(&Handle::<Image>::default()),
            Err(super::MissingActionBarImagePath)
        );
        assert_eq!(super::require_image_path(&first), Ok(()));
    }

    fn app_ready_for_the_overworld() -> App {
        let mut app = app_with_an_asset_server();
        app.add_plugins((StatesPlugin, InputPlugin, FarmingPlugin))
            .init_asset::<TextureAtlasLayout>()
            .init_state::<GameState>()
            .init_resource::<Inventory>()
            .init_resource::<PlayerProfile>()
            .init_resource::<ProfileDirty>()
            .init_resource::<ToolEnergy>()
            .init_resource::<InventoryWindowOpen>()
            .init_resource::<ForgeWindowOpen>()
            .add_event::<MouseWheel>()
            .add_systems(OnEnter(GameState::Overworld), setup_overworld)
            .add_systems(OnExit(GameState::Overworld), cleanup_overworld);
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
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Overworld);
        app.update();
        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Overworld
        );
    }

    fn leave_overworld(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Title);
        app.update();
        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Title
        );
    }

    fn loaded_image_paths(art: &ActionBarArt) -> Vec<String> {
        let mut image_paths = vec![
            asset_path_of(&art.frame_image()),
            asset_path_of(&art.highlight_image()),
        ];
        for slot_index in 0..ACTION_BAR_SLOT_COUNT {
            let slot_image = art.slot_image(slot_index).expect("loaded slot");
            image_paths.push(asset_path_of(&slot_image));
        }
        image_paths
    }

    fn expected_image_paths() -> Vec<String> {
        let mut image_paths = vec![
            FRAME_IMAGE_PATH.to_string(),
            HIGHLIGHT_IMAGE_PATH.to_string(),
        ];
        for slot_number in 1..=ACTION_BAR_SLOT_COUNT {
            image_paths.push(format!("ui/action_bar/slot_{slot_number}.png"));
        }
        image_paths
    }

    fn unique_handle_count(art: &ActionBarArt) -> usize {
        let mut handles = std::collections::HashSet::new();
        handles.insert(art.frame_image());
        handles.insert(art.highlight_image());
        for slot_index in 0..ACTION_BAR_SLOT_COUNT {
            handles.insert(art.slot_image(slot_index).expect("loaded slot"));
        }
        handles.len()
    }

    fn asset_path_of(image: &Handle<Image>) -> String {
        image
            .path()
            .expect("action bar image keeps its asset path")
            .path()
            .to_string_lossy()
            .into_owned()
    }

    fn assert_png_file(asset_path: &str) {
        let file_path = format!("{}/assets/{asset_path}", env!("CARGO_MANIFEST_DIR"));
        let bytes =
            std::fs::read(&file_path).unwrap_or_else(|error| panic!("read {file_path}: {error}"));
        assert!(
            bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]),
            "{file_path} is not a PNG"
        );
    }
}
