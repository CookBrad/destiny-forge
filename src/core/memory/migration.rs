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
    ProfileMigration {
        to_version: 9,
        apply: migrate_profile_to_v9,
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
    let Some(inventory) = field_mut(profile, "inventory") else {
        return;
    };
    let Some(slots) = field_mut(inventory, "slots") else {
        return;
    };
    let items = match slots {
        RonValue::Tuple(items) | RonValue::List(items) => items,
        _ => return,
    };
    for slot in items {
        force_slot_quality_normal(slot);
    }
}

fn force_slot_quality_normal(slot: &mut RonValue) {
    let RonValue::Map(fields) = slot else {
        return;
    };
    let normal = RonValue::Unit("Normal".to_string());
    if let Some((_, quality)) = fields.iter_mut().find(|(name, _)| name == "quality") {
        *quality = normal;
        return;
    }
    fields.push(("quality".to_string(), normal));
}

fn migrate_profile_to_v9(profile: &mut RonValue) {
    let RonValue::Map(fields) = profile else {
        return;
    };
    if fields.iter().any(|(name, _)| name == "header") {
        return;
    }
    let copy = |key: &str, fallback: RonValue| -> RonValue {
        let Some((_, value)) = fields.iter().find(|(name, _)| name == key) else {
            return fallback;
        };
        match value {
            RonValue::String(text) => RonValue::String(text.clone()),
            RonValue::Integer(number) => RonValue::Integer(*number),
            RonValue::Unit(name) => RonValue::Unit(name.clone()),
            _ => fallback,
        }
    };
    let name = copy("name", RonValue::String(String::new()));
    let calendar_day = copy("calendar_day", RonValue::Integer(1));
    let day_phase = copy("day_phase", RonValue::Unit("Morning".to_string()));
    fields.push((
        "header".to_string(),
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
        ]),
    ));
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
    use crate::items::{ItemQuality, MaterialId, INVENTORY_SLOT_COUNT};

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
        assert_eq!(profile.name, "");
        assert_eq!(profile.calendar_day, 1);
        assert_eq!(profile.day_phase, DayPhase::Morning);
        assert_eq!(profile.tool_energy, TOOL_ENERGY_MAX);
        assert!(profile.crop_plots.is_empty());
        assert!(profile.depleted_ore_node_ids.is_empty());
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
    }

    #[test]
    fn version_3_save_keeps_its_name() {
        let profile = profile_from_ron(V3_PROFILE).unwrap();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.name, "Ada");
        assert_eq!(profile.calendar_day, 1);
        assert_eq!(profile.day_phase, DayPhase::Morning);
        assert_eq!(profile.tool_energy, TOOL_ENERGY_MAX);
        assert!(profile.crop_plots.is_empty());
        assert_eq!(profile.inventory.count(MaterialId::IronScrap), 4);
        assert_eq!(profile.loadout.weapon, WeaponKind::RustySword);
    }

    #[test]
    fn version_4_save_keeps_the_calendar() {
        let profile = profile_from_ron(V4_PROFILE).unwrap();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.name, "Bram");
        assert_eq!(profile.calendar_day, 12);
        assert_eq!(profile.day_phase, DayPhase::Afternoon);
        assert_eq!(profile.tool_energy, TOOL_ENERGY_MAX);
        assert!(profile.crop_plots.is_empty());
        assert_eq!(profile.inventory.count(MaterialId::Turnip), 6);
        assert_eq!(profile.loadout.weapon, WeaponKind::IronSword);
    }

    #[test]
    fn version_5_save_keeps_tool_energy() {
        let profile = profile_from_ron(V5_PROFILE).unwrap();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.name, "Cora");
        assert_eq!(profile.calendar_day, 3);
        assert_eq!(profile.day_phase, DayPhase::Evening);
        assert_eq!(profile.tool_energy, 40.0);
        assert!(profile.crop_plots.is_empty());
        assert!(profile.depleted_ore_node_ids.is_empty());
        assert_eq!(profile.inventory.count(MaterialId::Potato), 9);
        assert_eq!(profile.loadout.weapon, WeaponKind::SlimeBlade);
        assert!(profile.progress.boss_defeated_floor_1);
    }

    #[test]
    fn version_6_profile_with_24_slots_loads_without_losing_items() {
        let profile = profile_from_ron(V6_PROFILE).unwrap();

        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.name, "Old Save");
        assert_eq!(profile.calendar_day, 4);
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
        assert!(profile.depleted_ore_node_ids.is_empty());
        assert!(profile
            .inventory
            .slots
            .iter()
            .all(|slot| slot.quality == ItemQuality::Normal));
    }

    #[test]
    fn version_6_save_keeps_depleted_ore_recorded_before_the_version_bump() {
        let text = V6_PROFILE.replace(
            "crop_plots: [],",
            "crop_plots: [],\n    depleted_ore_node_ids: [4, 8],",
        );
        let profile = profile_from_ron(&text).unwrap();
        assert_eq!(profile.depleted_ore_node_ids, vec![4, 8]);
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.inventory.count(MaterialId::SlimeGel), 120);
    }

    #[test]
    fn version_7_save_keeps_mining_and_cooking_schema() {
        let profile = profile_from_ron(V7_PROFILE).unwrap();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.name, "Hearth");
        assert_eq!(profile.calendar_day, 18);
        assert_eq!(profile.day_phase, DayPhase::Evening);
        assert_eq!(profile.tool_energy, 55.0);
        assert!(profile.progress.boss_defeated_floor_1);
        assert_eq!(profile.depleted_ore_node_ids, vec![3, 9]);
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
        assert_eq!(profile.crop_plots.len(), 1);
        assert_eq!(profile.crop_plots[0].tile_x, 4);
        assert_eq!(profile.crop_plots[0].tile_y, 2);
        assert_eq!(
            profile.crop_plots[0].stage,
            SavedPlotStage::Growing {
                crop: SavedCropKind::Turnip,
                days: 1,
                watered: true,
            }
        );
    }

    #[test]
    fn current_profile_round_trips_through_the_save_text() {
        let mut profile = PlayerProfile::default();
        profile.name = "Hélène \"the\" smith".to_string();
        profile.calendar_day = 18;
        profile.day_phase = DayPhase::Evening;
        profile.tool_energy = 55.0;
        profile.progress.boss_defeated_floor_1 = true;
        profile.depleted_ore_node_ids = vec![3, 9];
        profile.loadout.food_buff = Some(MaterialId::PotatoStew);
        profile.loadout.weapon = WeaponKind::IronSword;
        profile.loadout.stash.weapons.push(WeaponKind::RustySword);
        profile.inventory.slots[31].material = Some(MaterialId::IronScrap);
        profile.inventory.slots[31].count = 2;
        profile.inventory.try_add(MaterialId::CopperOre, 12);
        profile
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
    fn version_7_save_loads_every_stack_as_normal() {
        let mut profile = PlayerProfile {
            version: 7,
            ..PlayerProfile::default()
        };
        assert_eq!(
            profile
                .inventory
                .try_add_quality(MaterialId::Turnip, ItemQuality::Silver, 4),
            0
        );
        assert_eq!(
            profile
                .inventory
                .try_add_quality(MaterialId::Potato, ItemQuality::Destiny, 1),
            0
        );

        let loaded = profile_from_ron(&ron::to_string(&profile).unwrap()).unwrap();

        assert_eq!(loaded.version, PROFILE_VERSION);
        assert_eq!(
            loaded
                .inventory
                .count_quality(MaterialId::Turnip, ItemQuality::Normal),
            4
        );
        assert_eq!(
            loaded
                .inventory
                .count_quality(MaterialId::Potato, ItemQuality::Normal),
            1
        );
        assert_eq!(
            loaded
                .inventory
                .count_quality(MaterialId::Turnip, ItemQuality::Silver),
            0
        );
        assert_eq!(
            loaded
                .inventory
                .count_quality(MaterialId::Potato, ItemQuality::Destiny),
            0
        );
        assert!(loaded
            .inventory
            .slots
            .iter()
            .all(|slot| slot.quality == ItemQuality::Normal));
    }

    #[test]
    fn current_profile_keeps_split_quality_stacks() {
        let mut profile = PlayerProfile::default();
        assert_eq!(
            profile
                .inventory
                .try_add_quality(MaterialId::Potato, ItemQuality::Gold, 2),
            0
        );
        assert_eq!(profile.inventory.try_add(MaterialId::Potato, 5), 0);

        let loaded = profile_from_ron(&ron::to_string(&profile).unwrap()).unwrap();

        assert_eq!(loaded.version, PROFILE_VERSION);
        assert_eq!(
            loaded
                .inventory
                .count_quality(MaterialId::Potato, ItemQuality::Gold),
            2
        );
        assert_eq!(
            loaded
                .inventory
                .count_quality(MaterialId::Potato, ItemQuality::Normal),
            5
        );
    }

    #[test]
    fn version_8_save_fills_a_blank_header_and_keeps_the_silver_stack() {
        let profile = profile_from_ron(V8_PROFILE).unwrap();
        assert_eq!(profile.version, PROFILE_VERSION);
        assert_eq!(profile.header.name, "Hearth");
        assert_eq!(profile.header.playtime_seconds, 0);
        assert_eq!(profile.header.calendar.calendar_day, 18);
        assert_eq!(profile.header.calendar.day_phase, DayPhase::Evening);
        assert_eq!(profile.header.location, SaveLocation::Homestead);
        assert_eq!(profile.header.gold, 0);
        assert_eq!(profile.name, "Hearth");
        assert_eq!(profile.calendar_day, 18);
        assert_eq!(profile.tool_energy, 55.0);
        assert_eq!(
            profile
                .inventory
                .count_quality(MaterialId::CopperOre, ItemQuality::Silver),
            12
        );
    }

    #[test]
    fn version_8_save_keeps_an_existing_header() {
        let text = V8_PROFILE.replace(
            "version:8,",
            "version:8,header:(name:\"Kept\",playtime_seconds:9,calendar:(calendar_day:2,day_phase:Morning),location:Mine,gold:4),",
        );
        let profile = profile_from_ron(&text).unwrap();
        assert_eq!(profile.header.name, "Kept");
        assert_eq!(profile.header.playtime_seconds, 9);
        assert_eq!(profile.header.location, SaveLocation::Mine);
        assert_eq!(profile.header.gold, 4);
        assert_eq!(profile.header.calendar.calendar_day, 2);
        assert_eq!(profile.name, "Hearth");
        assert_eq!(profile.calendar_day, 18);
    }

    #[test]
    fn version_9_migration_leaves_a_list_unchanged() {
        let mut value = RonValue::List(vec![RonValue::Integer(1)]);
        migrate_profile_to_v9(&mut value);
        assert!(matches!(value, RonValue::List(items) if items.len() == 1));
    }

    #[test]
    fn save_older_than_the_first_schema_does_not_load() {
        let text = V2_PROFILE.replace("version:2", "version:1");
        let error = profile_from_ron(&text).unwrap_err();
        assert!(error.contains("cannot migrate"));
    }

    fn assert_profiles_match(loaded: &PlayerProfile, expected: &PlayerProfile) {
        assert_eq!(loaded.version, expected.version);
        assert_eq!(loaded.header, expected.header);
        assert_eq!(loaded.name, expected.name);
        assert_eq!(loaded.inventory, expected.inventory);
        assert_eq!(loaded.loadout, expected.loadout);
        assert_eq!(loaded.progress, expected.progress);
        assert_eq!(loaded.calendar_day, expected.calendar_day);
        assert_eq!(loaded.day_phase, expected.day_phase);
        assert_eq!(loaded.tool_energy, expected.tool_energy);
        assert_eq!(loaded.crop_plots, expected.crop_plots);
        assert_eq!(loaded.depleted_ore_node_ids, expected.depleted_ore_node_ids);
        assert_eq!(loaded.settings, expected.settings);
    }
}
