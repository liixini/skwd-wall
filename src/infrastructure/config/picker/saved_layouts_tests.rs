#![cfg(test)]

use serde_json::{Value, json};
use skwd_config::keys::selector as sel;

use super::*;
use crate::contracts::picker::Mode;
use crate::contracts::settings::{ACTIVE_LAYOUT, PREMADES, Premade, SAVED_LAYOUTS, Stored};
use skwd_config::keys::{filter_bar, general, system};
use skwd_config::picker_layout;

const VIEWPORT: picker_layout::Size = picker_layout::Size { width: 1280.0, height: 720.0 };
const OLD: &str = "components.wallpaperSelector.presets";
const REJECTING: Premade = Premade {
    name: "rejecting",
    desc: "rejecting",
    mode: Mode::Slices,
    values: &[
        (sel::SLICE_HEIGHT, Stored::Number(1.0)),
        (sel::VISIBLE_COUNT, Stored::Number(1.0)),
        (sel::EXPANDED_WIDTH, Stored::Number(1.0)),
        (sel::SLICE_WIDTH, Stored::Number(1.0)),
        (sel::SLICE_SPACING, Stored::Number(1.0)),
        (sel::SKEW_OFFSET, Stored::Number(1.0)),
        (sel::SLICE_EDGE_TILT, Stored::Number(1.0)),
        (sel::FRAME_X, Stored::Number(25.0)),
    ],
};
const LEADING: [&str; 7] = [
    sel::SLICE_HEIGHT,
    sel::VISIBLE_COUNT,
    sel::EXPANDED_WIDTH,
    sel::SLICE_WIDTH,
    sel::SLICE_SPACING,
    sel::SKEW_OFFSET,
    sel::SLICE_EDGE_TILT,
];

fn cfg(data: Value) -> Config {
    let name = std::process::id();
    let dir = std::env::temp_dir().join(format!("skwd_layouts_test_{name}"));
    Config {
        data,
        layout: None,
        effective: std::sync::OnceLock::new(),
        layout_cache: std::sync::OnceLock::new(),
        palette_previews: std::sync::OnceLock::new(),
        overview: crate::contracts::settings::OverviewMemo::default(),
        config_path: dir.join("config.json"),
        small: false,
        transient: true,
        writes: super::document::Writes::default(),
        on_battery: false,
    }
}

fn seeded() -> Config {
    let mut conf = cfg(json!({}));
    conf.seed_saved_layouts("Desk");
    conf
}

#[test]
fn seed_starts_clean() {
    let conf = seeded();
    let view = conf.saved_layouts_view();
    assert_eq!(view.names, ["Desk"]);
    assert_eq!(view.changes, 0);
    assert_eq!(conf.get(ACTIVE_LAYOUT), Some(&json!("Desk")));
}

#[test]
fn seed_keeps_existing() {
    let mut conf = seeded();
    conf.seed_saved_layouts("Other");
    assert_eq!(conf.saved_layouts_view().names, ["Desk"]);
}

#[test]
fn seed_imports_presets() {
    let mut conf = cfg(json!({"components":{"wallpaperSelector":{
        "presets":{"hex":[{"name":"Tight","params":{"hexRows":2}}]}
    }}}));
    conf.seed_saved_layouts("Desk");
    assert_eq!(conf.saved_layouts_view().names, ["Desk", "Tight"]);
    conf.switch_layout(1).unwrap();
    assert_eq!(conf.str_path(sel::DISPLAY_MODE), "hex");
    assert_eq!(conf.num_path(sel::HEX_ROWS), 2.0);
}

#[test]
fn edits_count_changes() {
    let mut conf = seeded();
    conf.set_key(sel::VISIBLE_COUNT, json!(10));
    conf.set_key(sel::SKEW_OFFSET, json!(28));
    assert_eq!(conf.saved_layouts_view().changes, 2);
    conf.save_layout();
    assert_eq!(conf.saved_layouts_view().changes, 0);
}

#[test]
fn explicit_default_unchanged() {
    let mut conf = seeded();
    conf.set_key(sel::SLICE_WOBBLE, json!(false));
    assert_eq!(conf.saved_layouts_view().changes, 0);
}

#[test]
fn revert_restores_absent() {
    let mut conf = seeded();
    conf.set_key(sel::VISIBLE_COUNT, json!(10));
    conf.revert_layout().unwrap();
    assert!(conf.get(sel::VISIBLE_COUNT).is_none());
    assert_eq!(conf.saved_layouts_view().changes, 0);
}

#[test]
fn switch_applies_snapshot() {
    let mut conf = seeded();
    conf.set_key(sel::VISIBLE_COUNT, json!(10));
    conf.add_layout("Laptop");
    conf.switch_layout(0).unwrap();
    assert!(conf.get(sel::VISIBLE_COUNT).is_none());
    assert_eq!(conf.saved_layouts_view().active, Some(0));
    conf.switch_layout(1).unwrap();
    assert_eq!(conf.num_path(sel::VISIBLE_COUNT), 10.0);
    assert_eq!(conf.saved_layouts_view().current(), Some("Laptop"));
}

#[test]
fn rename_rejects_taken() {
    let mut conf = seeded();
    conf.add_layout("Laptop");
    assert!(!conf.rename_layout("Desk"));
    assert!(!conf.rename_layout("  "));
    assert!(conf.rename_layout(" Streaming "));
    assert_eq!(conf.saved_layouts_view().names, ["Desk", "Streaming"]);
    assert_eq!(conf.saved_layouts_view().current(), Some("Streaming"));
}

#[test]
fn delete_keeps_one() {
    let mut conf = seeded();
    conf.delete_layout().unwrap();
    assert_eq!(conf.array_len(SAVED_LAYOUTS), 1);
    conf.add_layout("Laptop");
    conf.delete_layout().unwrap();
    assert_eq!(conf.saved_layouts_view().names, ["Desk"]);
    assert_eq!(conf.saved_layouts_view().current(), Some("Desk"));
}

#[test]
fn float_positions_round_trip() {
    let mut conf = seeded();
    conf.set_key(filter_bar::ARRANGE_POSITION_ENABLED, json!(true));
    conf.set_key(filter_bar::ARRANGE_POSITION_X, json!(40));
    conf.set_key(filter_bar::SWATCH_POSITION_Y, json!(75));
    assert_eq!(conf.saved_layouts_view().changes, 3);
    conf.add_layout("Laptop");
    conf.switch_layout(0).unwrap();
    assert!(conf.get(filter_bar::ARRANGE_POSITION_ENABLED).is_none());
    conf.switch_layout(1).unwrap();
    assert_eq!(conf.get(filter_bar::ARRANGE_POSITION_ENABLED), Some(&json!(true)));
    assert_eq!(conf.num_path(filter_bar::ARRANGE_POSITION_X), 40.0);
    assert_eq!(conf.num_path(filter_bar::SWATCH_POSITION_Y), 75.0);
}

#[test]
fn corners_round_trip() {
    use skwd_config::corners::{ALL, CornerStyle};
    let mut conf = seeded();
    for keys in ALL {
        conf.set_key(keys.style, json!("chamfered"));
        conf.set_key(keys.same, json!(false));
        conf.set_key(keys.values[1], json!(9));
    }
    conf.add_layout("Laptop");
    conf.switch_layout(0).unwrap();
    for keys in ALL {
        assert!(keys.values.iter().chain([&keys.same]).all(|path| conf.get(path).is_none()));
        assert_ne!(conf.corners(&keys).style, CornerStyle::Chamfered);
    }
    conf.switch_layout(1).unwrap();
    for keys in ALL {
        let corners = conf.corners(&keys);
        assert_eq!(
            (corners.style, corners.same, corners.values[1]),
            (CornerStyle::Chamfered, false, 9.0)
        );
    }
}

#[test]
fn old_layout_drops_corners() {
    let mut conf = cfg(json!({}));
    let mut values = serde_json::Map::new();
    values.insert(sel::GRID_CORNER_RADIUS.into(), json!(18));
    conf.set_key(SAVED_LAYOUTS, json!([{"name": "Old", "values": values}]));
    conf.set_key(ACTIVE_LAYOUT, json!("Old"));
    conf.set_key(sel::GRID_CORNER_STYLE, json!("chamfered"));
    conf.set_key(sel::GRID_CORNER_TL, json!(3));
    conf.revert_layout().unwrap();
    let grid = conf.corners(&skwd_config::corners::GRID);
    assert_eq!((grid.style.key(), grid.sizes()), ("round", [18.0; 4]));
}

#[test]
fn show_flags_stay_global() {
    let mut conf = seeded();
    conf.set_key(filter_bar::SHOW_ARRANGE, json!(false));
    conf.set_key(filter_bar::SHOW_SWATCH, json!(false));
    assert_eq!(conf.saved_layouts_view().changes, 0);
}

#[test]
fn premade_replaces_everything() {
    let mut conf = seeded();
    conf.set_key(sel::SLICE_WIDTH, json!(77));
    conf.set_key(filter_bar::SWATCH_POSITION_X, json!(10));
    conf.apply_premade(&PREMADES[1]).unwrap();
    assert!(conf.get(filter_bar::SWATCH_POSITION_X).is_none());
    assert_eq!(conf.str_path(sel::DISPLAY_MODE), "hex");
    assert_eq!(conf.num_path(sel::HEX_RADIUS), 160.0);
    assert!(conf.get(sel::SLICE_WIDTH).is_none());
    assert!(conf.saved_layouts_view().dirty());
}

#[test]
fn unique_names_count() {
    let mut conf = seeded();
    assert_eq!(conf.unique_layout("Laptop"), "Laptop");
    assert_eq!(conf.unique_layout("Desk"), "Desk 2");
    conf.add_layout("Desk 2");
    assert_eq!(conf.unique_layout("Desk"), "Desk 3");
}

fn bytes(conf: &Config) -> Vec<u8> {
    serde_json::to_vec(&conf.data).unwrap()
}

fn rejecting() -> Config {
    let mut conf = cfg(json!({}));
    conf.set_key(sel::DISPLAY_MODE, json!("hex"));
    for path in LEADING {
        conf.set_key(path, json!(1));
    }
    conf.seed_saved_layouts("Desk");
    conf.add_layout("Laptop");
    let mut entries = conf.array_values(SAVED_LAYOUTS);
    for entry in &mut entries {
        entry["values"][sel::FRAME_X] = json!(25);
    }
    conf.set_key(SAVED_LAYOUTS, Value::Array(entries));
    conf.set_key(picker_layout::ROOT, json!({"version": 2, "outputs": {}}));
    conf.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    conf.set_key(ACTIVE_LAYOUT, json!("Laptop"));
    conf.set_key(sel::SLICE_WOBBLE, json!(true));
    conf
}

#[test]
fn rejected_switch_keeps() {
    let mut conf = rejecting();
    let before = bytes(&conf);
    assert!(conf.switch_layout(0).is_err());
    assert_eq!(bytes(&conf), before);
    assert_eq!(conf.saved_layouts_view().current(), Some("Laptop"));
}

#[test]
fn rejected_revert_keeps() {
    let mut conf = rejecting();
    let before = bytes(&conf);
    assert!(conf.revert_layout().is_err());
    assert_eq!(bytes(&conf), before);
}

#[test]
fn rejected_premade_keeps() {
    let mut conf = rejecting();
    let before = bytes(&conf);
    assert!(conf.apply_premade(&REJECTING).is_err());
    assert_eq!(bytes(&conf), before);
}

#[test]
fn rejected_delete_keeps() {
    let mut conf = rejecting();
    let before = bytes(&conf);
    assert!(conf.delete_layout().is_err());
    assert_eq!(bytes(&conf), before);
    assert_eq!(conf.saved_layouts_view().names, ["Desk", "Laptop"]);
}

#[test]
fn corrupt_name_acts_nowhere() {
    let mut conf = seeded();
    conf.add_layout("Laptop");
    conf.set_key(ACTIVE_LAYOUT, json!("Gone"));
    conf.set_key(sel::VISIBLE_COUNT, json!(10));
    let before = bytes(&conf);
    let view = conf.saved_layouts_view();
    assert_eq!((view.active, view.current(), view.changes), (None, None, 0));
    conf.save_layout();
    conf.revert_layout().unwrap();
    conf.delete_layout().unwrap();
    assert!(!conf.rename_layout("Fresh"));
    assert_eq!(bytes(&conf), before);
    conf.save_named_layout("Fresh");
    assert_eq!(conf.saved_layouts_view().names, ["Desk", "Laptop", "Fresh"]);
}

#[test]
fn machine_keys_survive() {
    let mut conf = seeded();
    conf.add_layout("Laptop");
    conf.set_key(system::MONITOR, json!("DP-9"));
    conf.set_key(general::UI_SCALE, json!(1.5));
    conf.set_key(sel::VISIBLE_COUNT, json!(10));
    let kept =
        |conf: &Config| (conf.get(system::MONITOR).cloned(), conf.get(general::UI_SCALE).cloned());
    let machine = kept(&conf);
    assert!(machine.0.is_some() && machine.1.is_some());
    conf.revert_layout().unwrap();
    assert_eq!(kept(&conf), machine);
    conf.switch_layout(0).unwrap();
    assert_eq!(kept(&conf), machine);
    conf.apply_premade(&PREMADES[1]).unwrap();
    assert_eq!(kept(&conf), machine);
}

#[test]
fn later_presets_import() {
    let mut conf = seeded();
    conf.set_key(
        OLD,
        json!({"hex": [{"name": "Tight", "params": {"hexRows": 2}}, {"params": {}}]}),
    );
    conf.seed_saved_layouts("Desk");
    assert_eq!(conf.saved_layouts_view().names, ["Desk", "Tight"]);
    assert_eq!(conf.get(OLD), Some(&json!({"hex": [{"params": {}}]})));
    conf.seed_saved_layouts("Desk");
    assert_eq!(conf.saved_layouts_view().names, ["Desk", "Tight"]);
}

#[test]
fn import_skips_present() {
    let mut conf = cfg(json!({}));
    let preset = json!({"name": "Tight", "params": {"hexRows": 2}});
    conf.set_key(OLD, json!({"hex": [preset]}));
    conf.seed_saved_layouts("Desk");
    conf.set_key(OLD, json!({"hex": [preset]}));
    conf.seed_saved_layouts("Desk");
    assert_eq!(conf.saved_layouts_view().names, ["Desk", "Tight"]);
    assert!(conf.get(OLD).is_none());
}

#[test]
fn import_renames_clash() {
    let mut conf = seeded();
    conf.set_key(OLD, json!({"grid": [{"name": "Desk", "params": {"gridColumns": 4}}]}));
    conf.seed_saved_layouts("Desk");
    assert_eq!(conf.saved_layouts_view().names, ["Desk", "Desk 2"]);
    assert_eq!(conf.saved_layouts_view().current(), Some("Desk"));
}

#[test]
fn failed_write_rejects() {
    let mut conf = seeded();
    conf.add_layout("Laptop");
    conf.set_key(sel::VISIBLE_COUNT, json!(10));
    let blocker = std::env::temp_dir().join(format!("skwd_layouts_block_{}", std::process::id()));
    std::fs::write(&blocker, "").unwrap();
    conf.config_path = blocker.join("config.json");
    conf.transient = false;
    let before = bytes(&conf);
    assert!(conf.switch_layout(0).is_err());
    assert!(conf.apply_premade(&PREMADES[1]).is_err());
    assert_eq!(bytes(&conf), before);
    std::fs::remove_file(&blocker).unwrap();
}
