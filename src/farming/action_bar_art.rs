use bevy::prelude::*;

#[cfg(test)]
use bevy::asset::AssetApp;

use super::action_bar::ACTION_BAR_SLOT_COUNT;

const FRAME_IMAGE_PATH: &str = "ui/action_bar/frame.png";
const HIGHLIGHT_IMAGE_PATH: &str = "ui/action_bar/highlight.png";

pub(super) fn load_action_bar_art(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(action_bar_art(&asset_server));
}

fn action_bar_art(asset_server: &AssetServer) -> ActionBarArt {
    ActionBarArt {
        frame: asset_server.load(FRAME_IMAGE_PATH),
        highlight: asset_server.load(HIGHLIGHT_IMAGE_PATH),
        slots: slot_images(asset_server),
    }
}

fn slot_images(asset_server: &AssetServer) -> [Handle<Image>; ACTION_BAR_SLOT_COUNT] {
    std::array::from_fn(|slot_index| asset_server.load(slot_image_path(slot_index)))
}

fn slot_image_path(slot_index: usize) -> String {
    let slot_number = slot_index + 1;
    format!("ui/action_bar/slot_{slot_number}.png")
}

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
pub(super) fn app_ready_for_the_overworld() -> App {
    let mut app = app_with_an_asset_server();
    app.add_plugins((
        bevy::state::app::StatesPlugin,
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

#[cfg(test)]
pub(super) fn app_with_an_asset_server() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>();
    app
}

#[cfg(test)]
pub(super) fn enter_overworld(app: &mut App) {
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

#[cfg(test)]
pub(super) fn leave_overworld(app: &mut App) {
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bevy::prelude::*;

    use super::{
        app_ready_for_the_overworld, app_with_an_asset_server, enter_overworld, leave_overworld,
        ActionBarArt,
    };
    use crate::farming::action_bar::ACTION_BAR_SLOT_COUNT;

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
        let loaded = loaded_image_paths(&art);
        assert_eq!(loaded, referenced_action_bar_paths());
        assert_eq!(
            unique_handle_count(&art),
            referenced_action_bar_paths().len()
        );
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
    }

    #[test]
    fn a_png_outside_the_loader_paths_is_not_referenced() {
        let referenced = referenced_action_bar_paths();
        let stray = vec!["ui/action_bar/slot_9.png".to_string()];
        assert_eq!(
            pngs_missing_a_loader_path(&stray, &referenced),
            vec!["ui/action_bar/slot_9.png"]
        );
    }

    #[test]
    fn the_frame_png_alone_is_still_referenced_by_the_loader() {
        let referenced = referenced_action_bar_paths();
        let only_frame = vec!["ui/action_bar/frame.png".to_string()];
        assert_eq!(
            pngs_missing_a_loader_path(&only_frame, &referenced),
            Vec::<&str>::new()
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

    fn referenced_action_bar_paths() -> Vec<String> {
        let mut image_paths = vec![
            super::FRAME_IMAGE_PATH.to_string(),
            super::HIGHLIGHT_IMAGE_PATH.to_string(),
        ];
        for slot_index in 0..ACTION_BAR_SLOT_COUNT {
            image_paths.push(super::slot_image_path(slot_index));
        }
        image_paths
    }

    fn unique_handle_count(art: &ActionBarArt) -> usize {
        let mut handles = HashSet::new();
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

    fn pngs_missing_a_loader_path<'a>(
        png_paths: &'a [String],
        referenced_paths: &[String],
    ) -> Vec<&'a str> {
        png_paths
            .iter()
            .map(String::as_str)
            .filter(|png_path| {
                !referenced_paths
                    .iter()
                    .any(|referenced| referenced == png_path)
            })
            .collect()
    }
}
