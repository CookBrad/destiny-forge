use bevy::prelude::*;
use bevy::ui::widget::{ImageNode, NodeImageMode};

const FRAME_IMAGE_PATH: &str = "ui/action_bar/frame.png";
const FRAME_NATIVE_WIDTH: f32 = 166.0;
const FRAME_NATIVE_HEIGHT: f32 = 26.0;
const FRAME_SCALE: f32 = 3.0;
const FRAME_WIDTH: f32 = FRAME_NATIVE_WIDTH * FRAME_SCALE;
const FRAME_HEIGHT: f32 = FRAME_NATIVE_HEIGHT * FRAME_SCALE;
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
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    use bevy::asset::AssetApp;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;
    use bevy::ui::widget::NodeImageMode;

    use super::{ActionBarArt, ActionBarFrame, ActionBarHud};

    const FRAME_IMAGE_PATH: &str = "ui/action_bar/frame.png";
    const CFG_TEST_ATTRIBUTE: &str = "#[cfg(test)]";

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
    fn the_unreferenced_ui_pngs_are_the_five_hoe_icons() {
        let ui_pngs = pngs_under_assets_ui();
        assert!(ui_pngs.iter().any(|path| path == FRAME_IMAGE_PATH));
        assert_eq!(unreferenced_pngs(&ui_pngs), five_hoe_icon_paths());
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

    fn unreferenced_pngs(ui_pngs: &[String]) -> Vec<String> {
        let referenced = production_reference_text();
        ui_pngs
            .iter()
            .filter(|png_path| !referenced.contains(png_path.as_str()))
            .cloned()
            .collect()
    }

    fn pngs_under_assets_ui() -> Vec<String> {
        let mut png_paths = Vec::new();
        collect_pngs(
            &workspace_root().join("assets").join("ui"),
            "ui",
            &mut png_paths,
        );
        png_paths.sort();
        png_paths
    }

    fn collect_pngs(directory: &Path, prefix: &str, png_paths: &mut Vec<String>) {
        let entries = std::fs::read_dir(directory).expect("assets/ui can be read");
        for entry in entries {
            let entry = entry.expect("assets/ui entry");
            let name = entry.file_name();
            let name = name.to_str().expect("utf-8 asset name");
            let path = entry.path();
            if path.is_dir() {
                collect_pngs(&path, &format!("{prefix}/{name}"), png_paths);
            } else if name.ends_with(".png") {
                png_paths.push(format!("{prefix}/{name}"));
            }
        }
    }

    fn five_hoe_icon_paths() -> Vec<String> {
        vec![
            "ui/tools/hoe_t1.png".to_string(),
            "ui/tools/hoe_t2.png".to_string(),
            "ui/tools/hoe_t3.png".to_string(),
            "ui/tools/hoe_t4.png".to_string(),
            "ui/tools/hoe_t5.png".to_string(),
        ]
    }

    fn production_reference_text() -> String {
        let test_only = test_only_rust_files();
        let mut text = String::new();
        for rust_file in rust_files_under_src() {
            if test_only.contains(&rust_file) {
                continue;
            }
            text.push_str(&without_cfg_test_items(&read_source(&rust_file)));
            text.push('\n');
        }
        for data_file in ron_files_under_assets() {
            text.push_str(&read_source(&data_file));
            text.push('\n');
        }
        text
    }

    fn test_only_rust_files() -> HashSet<PathBuf> {
        let mut test_only = HashSet::new();
        let mut pending = Vec::new();
        for rust_file in rust_files_under_src() {
            let source = read_source(&rust_file);
            for module_name in cfg_test_file_module_names(&source) {
                push_module_files(&rust_file, &module_name, &mut test_only, &mut pending);
            }
        }
        while let Some(rust_file) = pending.pop() {
            let source = read_source(&rust_file);
            for module_name in external_module_names(&source) {
                push_module_files(&rust_file, &module_name, &mut test_only, &mut pending);
            }
        }
        test_only
    }

    fn push_module_files(
        parent: &Path,
        module_name: &str,
        test_only: &mut HashSet<PathBuf>,
        pending: &mut Vec<PathBuf>,
    ) {
        for candidate in module_file_candidates(parent, module_name) {
            if candidate.is_file() && test_only.insert(candidate.clone()) {
                pending.push(candidate);
            }
        }
    }

    fn module_file_candidates(parent: &Path, module_name: &str) -> [PathBuf; 2] {
        let directory = if parent.ends_with("mod.rs") || parent.ends_with("main.rs") {
            parent
                .parent()
                .expect("module file has a parent")
                .to_path_buf()
        } else {
            parent.with_extension("")
        };
        [
            directory.join(format!("{module_name}.rs")),
            directory.join(module_name).join("mod.rs"),
        ]
    }

    fn rust_files_under_src() -> Vec<PathBuf> {
        files_with_extension(&workspace_root().join("src"), "rs")
    }

    fn ron_files_under_assets() -> Vec<PathBuf> {
        files_with_extension(&workspace_root().join("assets"), "ron")
    }

    fn files_with_extension(directory: &Path, extension: &str) -> Vec<PathBuf> {
        let mut files = Vec::new();
        collect_files_with_extension(directory, extension, &mut files);
        files.sort();
        files
    }

    fn collect_files_with_extension(directory: &Path, extension: &str, files: &mut Vec<PathBuf>) {
        let entries = std::fs::read_dir(directory).expect("directory can be read");
        for entry in entries {
            let path = entry.expect("directory entry").path();
            if path.is_dir() {
                collect_files_with_extension(&path, extension, files);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
                files.push(path);
            }
        }
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn read_source(path: &Path) -> String {
        std::fs::read_to_string(path).expect("source can be read")
    }

    fn without_cfg_test_items(source: &str) -> String {
        let mut kept = String::new();
        let mut index = 0;
        let mut region = SourceRegion::Code;
        while index < source.len() {
            if region == SourceRegion::Code {
                if let Some(after_attribute) = cfg_test_attribute_end(source, index) {
                    index = end_of_cfg_test_item(source, after_attribute);
                    continue;
                }
            }
            let (next_region, next_index) = advance_source_region(source, index, region);
            kept.push_str(&source[index..next_index]);
            region = next_region;
            index = next_index;
        }
        kept
    }

    fn cfg_test_file_module_names(source: &str) -> Vec<String> {
        let mut names = Vec::new();
        let mut index = 0;
        let mut region = SourceRegion::Code;
        while index < source.len() {
            if region == SourceRegion::Code {
                if let Some(after_attribute) = cfg_test_attribute_end(source, index) {
                    if let Some(name) = external_module_name(source, after_attribute) {
                        names.push(name);
                    }
                    index = end_of_cfg_test_item(source, after_attribute);
                    continue;
                }
            }
            let (next_region, next_index) = advance_source_region(source, index, region);
            region = next_region;
            index = next_index;
        }
        names
    }

    fn external_module_names(source: &str) -> Vec<String> {
        let mut names = Vec::new();
        let mut index = 0;
        let mut region = SourceRegion::Code;
        while index < source.len() {
            if region == SourceRegion::Code {
                if let Some(name) = module_name_at(source, index) {
                    names.push(name);
                }
            }
            let (next_region, next_index) = advance_source_region(source, index, region);
            region = next_region;
            index = next_index;
        }
        names
    }

    fn cfg_test_attribute_end(source: &str, index: usize) -> Option<usize> {
        let rest = source_from(source, index);
        if rest.starts_with(CFG_TEST_ATTRIBUTE) {
            Some(index + CFG_TEST_ATTRIBUTE.len())
        } else {
            None
        }
    }

    fn external_module_name(source: &str, index: usize) -> Option<String> {
        let index = skip_whitespace_and_comments(source, index);
        let index = skip_outer_attributes(source, index);
        let index = skip_whitespace_and_comments(source, index);
        let index = skip_visibility(source, index);
        let index = skip_whitespace_and_comments(source, index);
        module_name_at(source, index)
    }

    fn module_name_at(source: &str, index: usize) -> Option<String> {
        let rest = source_from(source, index);
        let rest = rest.strip_prefix("mod")?;
        if !rest.starts_with(|ch: char| ch.is_whitespace()) {
            return None;
        }
        if index > 0 {
            let before = source[..index].chars().next_back()?;
            if before == '_' || before.is_ascii_alphanumeric() {
                return None;
            }
        }
        let name_index = skip_whitespace_and_comments(source, index + "mod".len());
        let name = take_identifier(source_from(source, name_index))?;
        let after_name = skip_whitespace_and_comments(source, name_index + name.len());
        if source_from(source, after_name).starts_with(';') {
            Some(name.to_string())
        } else {
            None
        }
    }

    fn skip_outer_attributes(source: &str, mut index: usize) -> usize {
        loop {
            index = skip_whitespace_and_comments(source, index);
            if source_from(source, index).starts_with("#[") {
                index = skip_balanced_delimiters(source, index + 1, '[', ']');
                continue;
            }
            return index;
        }
    }

    fn skip_visibility(source: &str, index: usize) -> usize {
        let rest = source_from(source, index);
        if !rest.starts_with("pub") {
            return index;
        }
        let after_pub = index + "pub".len();
        let boundary = source_from(source, after_pub).chars().next();
        if matches!(boundary, Some(ch) if ch == '_' || ch.is_ascii_alphanumeric()) {
            return index;
        }
        let after_pub = skip_whitespace(source, after_pub);
        if source_from(source, after_pub).starts_with('(') {
            return skip_balanced_delimiters(source, after_pub, '(', ')');
        }
        after_pub
    }

    fn skip_balanced_delimiters(source: &str, index: usize, open: char, close: char) -> usize {
        let mut depth = 0i32;
        let mut index = index;
        let mut region = SourceRegion::Code;
        let mut started = false;
        while index < source.len() {
            if region == SourceRegion::Code {
                let ch = next_char(source, index);
                if ch == open {
                    depth += 1;
                    started = true;
                } else if ch == close && started {
                    depth -= 1;
                    if depth == 0 {
                        return index + ch.len_utf8();
                    }
                }
            }
            let (next_region, next_index) = advance_source_region(source, index, region);
            region = next_region;
            index = next_index;
        }
        index
    }

    fn end_of_cfg_test_item(source: &str, start: usize) -> usize {
        let mut index = skip_whitespace_and_comments(source, start);
        let mut brace_depth = 0i32;
        let mut paren_depth = 0i32;
        let mut bracket_depth = 0i32;
        let mut seen_brace = false;
        let mut region = SourceRegion::Code;
        while index < source.len() {
            if region == SourceRegion::Code {
                let ch = next_char(source, index);
                match ch {
                    '{' => {
                        brace_depth += 1;
                        seen_brace = true;
                    }
                    '}' if seen_brace
                        && brace_depth == 1
                        && paren_depth == 0
                        && bracket_depth == 0 =>
                    {
                        return index + ch.len_utf8();
                    }
                    '}' => brace_depth -= 1,
                    '(' => paren_depth += 1,
                    ')' => paren_depth -= 1,
                    '[' => bracket_depth += 1,
                    ']' => bracket_depth -= 1,
                    ';' if !seen_brace
                        && brace_depth == 0
                        && paren_depth == 0
                        && bracket_depth == 0 =>
                    {
                        return index + ch.len_utf8();
                    }
                    _ => {}
                }
            }
            let (next_region, next_index) = advance_source_region(source, index, region);
            region = next_region;
            index = next_index;
        }
        source.len()
    }

    fn skip_whitespace_and_comments(source: &str, mut index: usize) -> usize {
        loop {
            index = skip_whitespace(source, index);
            if source_from(source, index).starts_with("//") {
                index = skip_line_comment(source, index);
                continue;
            }
            if source_from(source, index).starts_with("/*") {
                index = skip_block_comment(source, index);
                continue;
            }
            return index;
        }
    }

    fn skip_whitespace(source: &str, mut index: usize) -> usize {
        while index < source.len() {
            let ch = next_char(source, index);
            if !ch.is_whitespace() {
                break;
            }
            index += ch.len_utf8();
        }
        index
    }

    fn skip_line_comment(source: &str, mut index: usize) -> usize {
        while index < source.len() {
            let ch = next_char(source, index);
            index += ch.len_utf8();
            if ch == '\n' {
                break;
            }
        }
        index
    }

    fn skip_block_comment(source: &str, mut index: usize) -> usize {
        let mut depth = 0i32;
        while index < source.len() {
            if source_from(source, index).starts_with("/*") {
                depth += 1;
                index += 2;
                continue;
            }
            if source_from(source, index).starts_with("*/") {
                depth -= 1;
                index += 2;
                if depth == 0 {
                    break;
                }
                continue;
            }
            index += next_char(source, index).len_utf8();
        }
        index
    }

    fn advance_source_region(
        source: &str,
        index: usize,
        region: SourceRegion,
    ) -> (SourceRegion, usize) {
        let ch = next_char(source, index);
        let next = index + ch.len_utf8();
        match region {
            SourceRegion::Code => advance_code_region(source, index, ch, next),
            SourceRegion::LineComment => {
                if ch == '\n' {
                    (SourceRegion::Code, next)
                } else {
                    (SourceRegion::LineComment, next)
                }
            }
            SourceRegion::BlockComment { depth } => {
                advance_block_comment(source, index, depth, next)
            }
            SourceRegion::StringLiteral { escaped } => advance_string_literal(ch, escaped, next),
        }
    }

    fn advance_code_region(
        source: &str,
        index: usize,
        ch: char,
        next: usize,
    ) -> (SourceRegion, usize) {
        if source_from(source, index).starts_with("//") {
            return (SourceRegion::LineComment, index + 2);
        }
        if source_from(source, index).starts_with("/*") {
            return (SourceRegion::BlockComment { depth: 1 }, index + 2);
        }
        if ch == '"' {
            return (SourceRegion::StringLiteral { escaped: false }, next);
        }
        if ch == '\'' {
            if let Some(end) = char_literal_end(source, index) {
                return (SourceRegion::Code, end);
            }
        }
        (SourceRegion::Code, next)
    }

    fn advance_block_comment(
        source: &str,
        index: usize,
        depth: usize,
        next: usize,
    ) -> (SourceRegion, usize) {
        if source_from(source, index).starts_with("/*") {
            return (SourceRegion::BlockComment { depth: depth + 1 }, index + 2);
        }
        if source_from(source, index).starts_with("*/") {
            if depth == 1 {
                return (SourceRegion::Code, index + 2);
            }
            return (SourceRegion::BlockComment { depth: depth - 1 }, index + 2);
        }
        (SourceRegion::BlockComment { depth }, next)
    }

    fn advance_string_literal(ch: char, escaped: bool, next: usize) -> (SourceRegion, usize) {
        if escaped {
            return (SourceRegion::StringLiteral { escaped: false }, next);
        }
        if ch == '\\' {
            return (SourceRegion::StringLiteral { escaped: true }, next);
        }
        if ch == '"' {
            return (SourceRegion::Code, next);
        }
        (SourceRegion::StringLiteral { escaped: false }, next)
    }

    fn char_literal_end(source: &str, quote_index: usize) -> Option<usize> {
        let mut index = quote_index + 1;
        if index >= source.len() {
            return None;
        }
        let ch = next_char(source, index);
        index += ch.len_utf8();
        if ch == '\\' {
            if index >= source.len() {
                return None;
            }
            let escape = next_char(source, index);
            index += escape.len_utf8();
            if escape == 'u' {
                let rest = source_from(source, index);
                if !rest.starts_with('{') {
                    return None;
                }
                let close = rest.find('}')?;
                index += close + 1;
            }
        }
        if source_from(source, index).starts_with('\'') {
            Some(index + 1)
        } else {
            None
        }
    }

    fn take_identifier(source: &str) -> Option<&str> {
        let mut length = 0;
        for ch in source.chars() {
            let is_start = length == 0 && (ch == '_' || ch.is_ascii_alphabetic());
            let is_continue = length > 0 && (ch == '_' || ch.is_ascii_alphanumeric());
            if is_start || is_continue {
                length += ch.len_utf8();
            } else {
                break;
            }
        }
        if length == 0 {
            None
        } else {
            Some(&source[..length])
        }
    }

    fn next_char(source: &str, index: usize) -> char {
        source[index..].chars().next().expect("index inside source")
    }

    fn source_from(source: &str, index: usize) -> &str {
        source.get(index..).unwrap_or("")
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum SourceRegion {
        Code,
        LineComment,
        BlockComment { depth: usize },
        StringLiteral { escaped: bool },
    }
}
