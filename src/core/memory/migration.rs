use super::super::day_cycle::TOOL_ENERGY_MAX;
use super::profile::{PlayerProfile, PROFILE_VERSION};
use super::ron_value::{parse_ron_value, print_ron_value, RonValue};

const PROFILE_V7_SLOT_COUNT: usize = 32;

struct ProfileMigration {
    to_version: u32,
    apply: fn(&mut RonValue),
}

const PROFILE_MIGRATIONS: &[ProfileMigration] = &[
    ProfileMigration {
        to_version: 3,
        apply: migrate_profile_to_v3,
    },
    ProfileMigration {
        to_version: 4,
        apply: migrate_profile_to_v4,
    },
    ProfileMigration {
        to_version: 5,
        apply: migrate_profile_to_v5,
    },
    ProfileMigration {
        to_version: 6,
        apply: migrate_profile_to_v6,
    },
    ProfileMigration {
        to_version: 7,
        apply: migrate_profile_to_v7,
    },
    ProfileMigration {
        to_version: 8,
        apply: migrate_profile_to_v8,
    },
];

pub(crate) fn profile_from_ron(text: &str) -> Result<PlayerProfile, String> {
    let mut value = parse_ron_value(text)?;
    apply_profile_migrations(&mut value)?;
    let rendered = print_ron_value(&value);
    ron::from_str(&rendered).map_err(|error| error.to_string())
}

fn apply_profile_migrations(value: &mut RonValue) -> Result<(), String> {
    let mut version = profile_version(value)?;
    if version > PROFILE_VERSION {
        return Ok(());
    }
    for step in PROFILE_MIGRATIONS {
        if version >= step.to_version {
            continue;
        }
        if step.to_version != version + 1 {
            return Err(migration_gap(version));
        }
        (step.apply)(value);
        version = step.to_version;
        stamp_profile_version(value, version);
    }
    if version < PROFILE_VERSION {
        return Err(migration_gap(version));
    }
    Ok(())
}

fn migration_gap(version: u32) -> String {
    format!("save version {version} cannot migrate to {PROFILE_VERSION}")
}

fn profile_version(value: &RonValue) -> Result<u32, String> {
    let RonValue::Map(fields) = value else {
        return Err("profile save is not a struct".to_string());
    };
    let Some((_, version)) = fields.iter().find(|(name, _)| name == "version") else {
        return Err("profile save has no version".to_string());
    };
    let RonValue::Integer(version) = version else {
        return Err("profile version is not an integer".to_string());
    };
    u32::try_from(*version).map_err(|_| "profile version is out of range".to_string())
}

fn stamp_profile_version(value: &mut RonValue, version: u32) {
    let Some(field) = field_mut(value, "version") else {
        return;
    };
    *field = RonValue::Integer(i64::from(version));
}

fn migrate_profile_to_v3(profile: &mut RonValue) {
    insert_field_if_missing(profile, "name", RonValue::String(String::new()));
}

fn migrate_profile_to_v4(profile: &mut RonValue) {
    insert_field_if_missing(profile, "calendar_day", RonValue::Integer(1));
    insert_field_if_missing(profile, "day_phase", RonValue::Unit("Morning".to_string()));
}

fn migrate_profile_to_v5(profile: &mut RonValue) {
    insert_field_if_missing(
        profile,
        "tool_energy",
        RonValue::Float(f64::from(TOOL_ENERGY_MAX)),
    );
}

fn migrate_profile_to_v6(profile: &mut RonValue) {
    insert_field_if_missing(profile, "crop_plots", RonValue::List(Vec::new()));
}

fn migrate_profile_to_v7(profile: &mut RonValue) {
    widen_inventory_to_v7(profile);
    if let Some(loadout) = field_mut(profile, "loadout") {
        insert_field_if_missing(loadout, "stash", empty_stash());
        insert_field_if_missing(loadout, "food_buff", RonValue::Option(None));
    }
    insert_field_if_missing(profile, "depleted_ore_node_ids", RonValue::List(Vec::new()));
}

fn migrate_profile_to_v8(profile: &mut RonValue) {
    if !has_field(profile, "header") {
        let name = take_field(profile, "name").unwrap_or_else(|| RonValue::String(String::new()));
        let calendar_day = take_field(profile, "calendar_day").unwrap_or(RonValue::Integer(1));
        let day_phase = take_field(profile, "day_phase")
            .unwrap_or_else(|| RonValue::Unit("Morning".to_string()));
        insert_field_if_missing(
            profile,
            "header",
            save_header(name, calendar_day, day_phase),
        );
    } else {
        remove_field(profile, "name");
        remove_field(profile, "calendar_day");
        remove_field(profile, "day_phase");
    }
    if !has_field(profile, "homestead") {
        let tool_energy = take_field(profile, "tool_energy")
            .unwrap_or(RonValue::Float(f64::from(TOOL_ENERGY_MAX)));
        let crop_plots = take_field(profile, "crop_plots").unwrap_or(RonValue::List(Vec::new()));
        insert_field_if_missing(
            profile,
            "homestead",
            homestead_save(tool_energy, crop_plots),
        );
    } else {
        remove_field(profile, "tool_energy");
        remove_field(profile, "crop_plots");
    }
    if !has_field(profile, "mine") {
        let depleted =
            take_field(profile, "depleted_ore_node_ids").unwrap_or(RonValue::List(Vec::new()));
        insert_field_if_missing(profile, "mine", mine_save(depleted));
    } else {
        remove_field(profile, "depleted_ore_node_ids");
    }
}

fn save_header(name: RonValue, calendar_day: RonValue, day_phase: RonValue) -> RonValue {
    RonValue::Map(vec![
        ("name".to_string(), name),
        ("playtime_seconds".to_string(), RonValue::Integer(0)),
        (
            "calendar".to_string(),
            RonValue::Map(vec![
                ("calendar_day".to_string(), calendar_day),
                ("day_phase".to_string(), day_phase),
            ]),
        ),
        (
            "location".to_string(),
            RonValue::Unit("Homestead".to_string()),
        ),
        ("gold".to_string(), RonValue::Integer(0)),
    ])
}

fn homestead_save(tool_energy: RonValue, crop_plots: RonValue) -> RonValue {
    RonValue::Map(vec![
        ("tool_energy".to_string(), tool_energy),
        ("crop_plots".to_string(), crop_plots),
    ])
}

fn mine_save(depleted_ore_node_ids: RonValue) -> RonValue {
    RonValue::Map(vec![(
        "depleted_ore_node_ids".to_string(),
        depleted_ore_node_ids,
    )])
}

fn widen_inventory_to_v7(profile: &mut RonValue) {
    let Some(inventory) = field_mut(profile, "inventory") else {
        return;
    };
    let Some(slots) = field_mut(inventory, "slots") else {
        return;
    };
    promote_list_to_tuple(slots);
    let RonValue::Tuple(items) = slots else {
        return;
    };
    while items.len() < PROFILE_V7_SLOT_COUNT {
        items.push(empty_inventory_slot());
    }
}

fn promote_list_to_tuple(value: &mut RonValue) {
    if let RonValue::List(items) = value {
        let items = std::mem::take(items);
        *value = RonValue::Tuple(items);
    }
}

fn empty_inventory_slot() -> RonValue {
    RonValue::Map(vec![
        ("material".to_string(), RonValue::Option(None)),
        ("count".to_string(), RonValue::Integer(0)),
    ])
}

fn empty_stash() -> RonValue {
    RonValue::Map(vec![
        ("weapons".to_string(), RonValue::List(Vec::new())),
        ("armor".to_string(), RonValue::List(Vec::new())),
    ])
}

fn has_field(value: &RonValue, key: &str) -> bool {
    let RonValue::Map(fields) = value else {
        return false;
    };
    fields.iter().any(|(name, _)| name == key)
}

fn take_field(value: &mut RonValue, key: &str) -> Option<RonValue> {
    let RonValue::Map(fields) = value else {
        return None;
    };
    let index = fields.iter().position(|(name, _)| name == key)?;
    Some(fields.remove(index).1)
}

fn remove_field(value: &mut RonValue, key: &str) {
    let _ = take_field(value, key);
}

fn insert_field_if_missing(value: &mut RonValue, key: &str, field: RonValue) {
    let RonValue::Map(fields) = value else {
        return;
    };
    if fields.iter().any(|(name, _)| name == key) {
        return;
    }
    fields.push((key.to_string(), field));
}

fn field_mut<'a>(value: &'a mut RonValue, key: &str) -> Option<&'a mut RonValue> {
    let RonValue::Map(fields) = value else {
        return None;
    };
    fields
        .iter_mut()
        .find(|(name, _)| name == key)
        .map(|(_, field)| field)
}

#[cfg(test)]
mod tests {
    use super::super::super::day_cycle::DayPhase;
    use super::super::crop_save::{SavedCropKind, SavedPlotStage};
    use super::super::header::SaveLocation;
    use super::*;
    use crate::combat::WeaponKind;
    use crate::items::{MaterialId, INVENTORY_SLOT_COUNT};

    const V2_PROFILE: &str = include_str!("fixtures/profile_v2.ron");
    const V3_PROFILE: &str = include_str!("fixtures/profile_v3.ron");
    const V4_PROFILE: &str = include_str!("fixtures/profile_v4.ron");
    const V5_PROFILE: &str = include_str!("fixtures/profile_v5.ron");
    const V6_PROFILE: &str = include_str!("fixtures/profile_v6_24_slots.ron");
    const V7_PROFILE: &str = include_str!("fixtures/profile_v7.ron");
    const V8_PROFILE: &str = include_str!("fixtures/profile_v8.ron");

    #[test]
    fn version_2_save_migrates_to_the_current_profile() {
        let profile = profile_from_ron(V2_PROFILE).unwrap();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.header.name, "");
        assert_eq!(profile.header.calendar.calendar_day, 1);
        assert_eq!(profile.header.calendar.day_phase, DayPhase::Morning);
        assert_eq!(profile.homestead.tool_energy, TOOL_ENERGY_MAX);
        assert!(profile.homestead.crop_plots.is_empty());
        assert!(profile.mine.depleted_ore_node_ids.is_empty());
        assert_eq!(profile.loadout.food_buff, None);
        assert!(profile.loadout.stash.weapons.is_empty());
        assert_eq!(profile.inventory.slots.len(), INVENTORY_SLOT_COUNT);
        assert_eq!(profile.inventory.count(MaterialId::SlimeGel), 12);
        assert_eq!(profile.inventory.slots[5].material, Some(MaterialId::Fang));
        assert_eq!(profile.inventory.slots[5].count, 3);
        assert!(profile.inventory.slots[24..]
            .iter()
            .all(|slot| slot.material.is_none()));
        assert!(profile.progress.boss_defeated_floor_1);
        assert_eq!(profile.loadout.weapon, WeaponKind::RustySpear);
        assert_blank_header(&profile);
    }

    #[test]
    fn version_3_save_keeps_its_name() {
        let profile = profile_from_ron(V3_PROFILE).unwrap();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.header.name, "Ada");
        assert_eq!(profile.header.calendar.calendar_day, 1);
        assert_eq!(profile.header.calendar.day_phase, DayPhase::Morning);
        assert_eq!(profile.homestead.tool_energy, TOOL_ENERGY_MAX);
        assert!(profile.homestead.crop_plots.is_empty());
        assert_eq!(profile.inventory.count(MaterialId::IronScrap), 4);
        assert_eq!(profile.loadout.weapon, WeaponKind::RustySword);
        assert_blank_header(&profile);
    }

    #[test]
    fn version_4_save_keeps_the_calendar() {
        let profile = profile_from_ron(V4_PROFILE).unwrap();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.header.name, "Bram");
        assert_eq!(profile.header.calendar.calendar_day, 12);
        assert_eq!(profile.header.calendar.day_phase, DayPhase::Afternoon);
        assert_eq!(profile.homestead.tool_energy, TOOL_ENERGY_MAX);
        assert!(profile.homestead.crop_plots.is_empty());
        assert_eq!(profile.inventory.count(MaterialId::Turnip), 6);
        assert_eq!(profile.loadout.weapon, WeaponKind::IronSword);
        assert_blank_header(&profile);
    }

    #[test]
    fn version_5_save_keeps_tool_energy() {
        let profile = profile_from_ron(V5_PROFILE).unwrap();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.header.name, "Cora");
        assert_eq!(profile.header.calendar.calendar_day, 3);
        assert_eq!(profile.header.calendar.day_phase, DayPhase::Evening);
        assert_eq!(profile.homestead.tool_energy, 40.0);
        assert!(profile.homestead.crop_plots.is_empty());
        assert!(profile.mine.depleted_ore_node_ids.is_empty());
        assert_eq!(profile.inventory.count(MaterialId::Potato), 9);
        assert_eq!(profile.loadout.weapon, WeaponKind::SlimeBlade);
        assert!(profile.progress.boss_defeated_floor_1);
        assert_blank_header(&profile);
    }

    #[test]
    fn version_6_profile_with_24_slots_loads_without_losing_items() {
        let profile = profile_from_ron(V6_PROFILE).unwrap();

        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.header.name, "Old Save");
        assert_eq!(profile.header.calendar.calendar_day, 4);
        assert_eq!(profile.inventory.slots.len(), INVENTORY_SLOT_COUNT);
        assert_eq!(profile.inventory.count(MaterialId::Hoe), 1);
        assert_eq!(profile.inventory.count(MaterialId::TurnipSeed), 8);
        assert_eq!(profile.inventory.count(MaterialId::Turnip), 5);
        assert_eq!(profile.inventory.count(MaterialId::Potato), 3);
        assert_eq!(profile.inventory.count(MaterialId::SlimeGel), 120);
        assert_eq!(profile.inventory.slots[6].count, 99);
        assert_eq!(profile.inventory.slots[7].count, 21);
        assert_eq!(
            profile.inventory.slots[23].material,
            Some(MaterialId::IronScrap)
        );
        assert_eq!(profile.inventory.slots[23].count, 2);
        assert!(profile.inventory.slots[24..]
            .iter()
            .all(|slot| slot.material.is_none()));
        assert_eq!(profile.loadout.food_buff, None);
        assert!(profile.mine.depleted_ore_node_ids.is_empty());
        assert_blank_header(&profile);
    }

    #[test]
    fn version_6_save_keeps_depleted_ore_recorded_before_the_version_bump() {
        let text = V6_PROFILE.replace(
            "crop_plots: [],",
            "crop_plots: [],\n    depleted_ore_node_ids: [4, 8],",
        );
        let profile = profile_from_ron(&text).unwrap();
        assert_eq!(profile.mine.depleted_ore_node_ids, vec![4, 8]);
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.inventory.count(MaterialId::SlimeGel), 120);
    }

    #[test]
    fn version_7_save_keeps_mining_and_cooking_schema() {
        let profile = profile_from_ron(V7_PROFILE).unwrap();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.header.name, "Hearth");
        assert_eq!(profile.header.calendar.calendar_day, 18);
        assert_eq!(profile.header.calendar.day_phase, DayPhase::Evening);
        assert_eq!(profile.homestead.tool_energy, 55.0);
        assert!(profile.progress.boss_defeated_floor_1);
        assert_eq!(profile.mine.depleted_ore_node_ids, vec![3, 9]);
        assert_eq!(profile.loadout.food_buff, Some(MaterialId::PotatoStew));
        assert_eq!(profile.loadout.weapon, WeaponKind::IronSword);
        assert_eq!(profile.loadout.stash.weapons, vec![WeaponKind::RustySword]);
        assert_eq!(profile.inventory.slots.len(), INVENTORY_SLOT_COUNT);
        assert_eq!(profile.inventory.count(MaterialId::PickaxeTier2), 1);
        assert_eq!(profile.inventory.count(MaterialId::PickaxeTier1), 1);
        assert_eq!(profile.inventory.count(MaterialId::CopperOre), 12);
        assert_eq!(profile.inventory.count(MaterialId::IronOre), 7);
        assert_eq!(profile.inventory.count(MaterialId::RoastTurnip), 2);
        assert_eq!(profile.inventory.count(MaterialId::PotatoStew), 4);
        assert_eq!(
            profile.inventory.slots[31].material,
            Some(MaterialId::IronScrap)
        );
        assert_eq!(profile.inventory.slots[31].count, 2);
        assert_eq!(profile.homestead.crop_plots.len(), 1);
        assert_eq!(profile.homestead.crop_plots[0].tile_x, 4);
        assert_eq!(profile.homestead.crop_plots[0].tile_y, 2);
        assert_eq!(
            profile.homestead.crop_plots[0].stage,
            SavedPlotStage::Growing {
                crop: SavedCropKind::Turnip,
                days: 1,
                watered: true,
            }
        );
        assert_blank_header(&profile);
    }

    #[test]
    fn version_8_save_keeps_the_header_and_the_same_gameplay() {
        let profile = profile_from_ron(V8_PROFILE).unwrap();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.header.name, "Hearth");
        assert_eq!(profile.header.playtime_seconds, 5400);
        assert_eq!(profile.header.calendar.calendar_day, 18);
        assert_eq!(profile.header.calendar.day_phase, DayPhase::Evening);
        assert_eq!(profile.header.location, SaveLocation::Dungeon);
        assert_eq!(profile.header.gold, 80);
        assert_eq!(profile.homestead.tool_energy, 55.0);
        assert_eq!(profile.mine.depleted_ore_node_ids, vec![3, 9]);
        assert_eq!(profile.loadout.weapon, WeaponKind::IronSword);
        assert_eq!(profile.inventory.count(MaterialId::CopperOre), 12);
        assert_eq!(profile.homestead.crop_plots.len(), 1);
    }

    #[test]
    fn current_profile_round_trips_through_the_save_text() {
        let mut profile = PlayerProfile::default();
        profile.header.name = "Hélène \"the\" smith".to_string();
        profile.header.calendar.calendar_day = 18;
        profile.header.calendar.day_phase = DayPhase::Evening;
        profile.homestead.tool_energy = 55.0;
        profile.progress.boss_defeated_floor_1 = true;
        profile.mine.depleted_ore_node_ids = vec![3, 9];
        profile.loadout.food_buff = Some(MaterialId::PotatoStew);
        profile.loadout.weapon = WeaponKind::IronSword;
        profile.loadout.stash.weapons.push(WeaponKind::RustySword);
        profile.inventory.slots[31].material = Some(MaterialId::IronScrap);
        profile.inventory.slots[31].count = 2;
        profile.inventory.try_add(MaterialId::CopperOre, 12);
        profile.header.playtime_seconds = 5400;
        profile.header.location = SaveLocation::Dungeon;
        profile.header.gold = 80;
        profile
            .homestead
            .crop_plots
            .push(super::super::crop_save::SavedCropPlot {
                tile_x: 4,
                tile_y: 2,
                stage: SavedPlotStage::Ready {
                    crop: SavedCropKind::Potato,
                },
            });

        let pretty = ron::ser::PrettyConfig::new().depth_limit(4);
        let pretty_text = ron::ser::to_string_pretty(&profile, pretty).unwrap();
        let compact_text = ron::to_string(&profile).unwrap();
        assert_profiles_match(&profile_from_ron(&pretty_text).unwrap(), &profile);
        assert_profiles_match(&profile_from_ron(&compact_text).unwrap(), &profile);
    }

    #[test]
    fn save_older_than_the_first_schema_does_not_load() {
        let text = V2_PROFILE.replace("version:2", "version:1");
        let error = profile_from_ron(&text).unwrap_err();
        assert!(error.contains("cannot migrate"));
    }

    fn assert_blank_header(profile: &PlayerProfile) {
        assert_eq!(profile.header.playtime_seconds, 0);
        assert_eq!(profile.header.location, SaveLocation::Homestead);
        assert_eq!(profile.header.gold, 0);
    }

    fn assert_profiles_match(loaded: &PlayerProfile, expected: &PlayerProfile) {
        assert_eq!(loaded.version, expected.version);
        assert_eq!(loaded.header, expected.header);
        assert_eq!(loaded.inventory, expected.inventory);
        assert_eq!(loaded.loadout, expected.loadout);
        assert_eq!(loaded.progress, expected.progress);
        assert_eq!(loaded.homestead, expected.homestead);
        assert_eq!(loaded.mine, expected.mine);
        assert_eq!(loaded.settings, expected.settings);
    }
}
