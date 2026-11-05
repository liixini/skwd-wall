use serde_json::{Value, json};

use super::*;
use crate::frontend::settings::test_source::FakeSettingsSource;

fn control_value(control: &Control) -> Value {
    match control {
        Control::Toggle { path, value } => json!({"kind":"toggle","path":path,"value":value}),
        Control::Colour { path, .. } => json!({"kind":"colour","path":path}),
        Control::ToggleAction { id, value } => {
            json!({"kind":"toggle_action","action":format!("{id:?}"),"value":value})
        }
        Control::Number { path, unit, .. } => json!({"kind":"number","path":path,"unit":unit}),
        Control::TextField { path, placeholder, .. } => {
            json!({"kind":"text","path":path,"placeholder":placeholder})
        }
        Control::Resolution { path, width_placeholder, height_placeholder, .. } => {
            json!({"kind":"resolution","path":path,"width":width_placeholder,"height":height_placeholder})
        }
        Control::KeyBinding { path, default, .. } => {
            json!({"kind":"keybinding","path":path,"default":default})
        }
        Control::Dropdown { path, options, current, .. } => {
            json!({"kind":"dropdown","path":path,"options":options,"current":current})
        }
        Control::Chips { path, options, current, disabled, background } => {
            json!({"kind":"chips","path":path,"options":options,"current":current,"disabled":disabled,"background":background.is_some()})
        }
        Control::ActionBtn { id, label } => {
            json!({"kind":"action","action":format!("{id:?}"),"label":label})
        }
        Control::ActionChips { items } => {
            json!({"kind":"action_chips","items":items.iter().map(|(id, label)| json!({"action":format!("{id:?}"),"label":label})).collect::<Vec<_>>()})
        }
        Control::Details { id, summary, rows } => {
            json!({"kind":"details","id":id,"summary":summary,"rows":rows.iter().map(row_value).collect::<Vec<_>>()})
        }
        Control::StackBar { id, summary, preview, rows } => {
            json!({"kind":"stack_bar","id":id,"summary":summary,"preview":preview,"rows":rows.iter().map(row_value).collect::<Vec<_>>()})
        }
        Control::Display(value) => json!({"kind":"display","value":format!("{value:?}")}),
        Control::AppTheme { app, .. } => {
            json!({"kind":"app_theme","id":app.id,"state":app.state,"enabled":app.enabled})
        }
        Control::Segment => json!({"kind":"segment"}),
        Control::Static => json!({"kind":"static"}),
        Control::Code { snippet } => json!({"kind":"code","snippet":snippet}),
        Control::Preview => json!({"kind":"preview"}),
    }
}

fn row_value(row: &Row) -> Value {
    json!({"title":row.title,"description":row.desc,"control":control_value(&row.control)})
}

fn fixture(
    name: &str,
    cfg: &dyn SettingsSource,
    statuses: &[crate::contracts::daemon::OutputStatus],
) -> Value {
    let outputs = [String::from("DP-1")];
    let themes = [String::from("Nord"), String::from("Dracula")];
    let folders = [String::from("/walls"), String::from("/walls/film")];
    let backends = [String::from("skwd"), String::from("matugen"), String::from("wallust")];
    let apps = crate::contracts::daemon::AppThemesResult {
        apps: vec![crate::contracts::daemon::AppThemeStatus {
            id: String::from("example"),
            name: String::from("Example app"),
            installed: true,
            config_found: true,
            config_path: String::new(),
            output_path: String::new(),
            enabled: true,
            state: String::from("applied"),
            detail: String::new(),
            can_enable: false,
            can_disable: true,
            can_adopt: true,
            template_path: String::new(),
            customized: false,
            can_disconnect: true,
            can_reconnect: true,
        }],
    };
    let playback = crate::contracts::daemon::PlaybackStatus {
        fullscreen_supported: true,
        maximized_supported: true,
        full_width_supported: true,
        other_audio_supported: true,
        ..Default::default()
    };
    let watch = crate::contracts::daemon::LibraryWatchStatus {
        ok: true,
        mode: String::from("native"),
        ..Default::default()
    };
    let weather = crate::contracts::daemon::WeatherStatus {
        enabled: true,
        status: crate::contracts::daemon::WeatherState::Active,
        ..Default::default()
    };
    let sddm_themes = [String::from("breeze")];
    let source = crate::frontend::settings::PlaceSource {
        cfg,
        themes: &themes,
        folders: &folders,
        analysis: "Semantic model ready",
        backends: &backends,
        runtime: RuntimeStatus {
            outputs: &outputs,
            statuses,
            playback: Some(&playback),
            app_themes: Some(&apps),
            library_watch: Some(&watch),
            weather: Some(&weather),
            sddm_themes: &sddm_themes,
            ..Default::default()
        },
    };
    let places: Vec<_> = source.places().into_iter().map(|place| {
        json!({"key":place.key,"sections":place.sections.iter().map(|section| json!({"title":section.title,"description":section.desc,"rows":section.rows().map(row_value).collect::<Vec<_>>()})).collect::<Vec<_>>()})
    }).collect();
    json!({"scenario":name,"places":places})
}

fn all_fixtures() -> Vec<Value> {
    let outputs = [
        crate::contracts::daemon::OutputStatus {
            name: String::from("DP-1"),
            kind: crate::contracts::media::MediaKind::Static,
            path: String::from("/walls/forest.png"),
            ..Default::default()
        },
        crate::contracts::daemon::OutputStatus {
            name: String::from("HDMI-A-1"),
            kind: crate::contracts::media::MediaKind::Video,
            path: String::from("/walls/film.mp4"),
            ..Default::default()
        },
        crate::contracts::daemon::OutputStatus {
            name: String::from("DP-2"),
            kind: crate::contracts::media::MediaKind::WallpaperEngine,
            we_id: String::from("1234"),
            ..Default::default()
        },
        crate::contracts::daemon::OutputStatus {
            name: String::from("DP-3"),
            target: String::from("@monitor:offline"),
            connected: false,
            ..Default::default()
        },
    ];
    let cases = [
        ("default", FakeSettingsSource::default()),
        ("niri", FakeSettingsSource::default().with_niri(true)),
        ("steam_off", FakeSettingsSource::default().with_flag(keys::features::STEAM, false)),
        ("awww", FakeSettingsSource::default().with_text(keys::paper::ENGINE, "awww")),
        ("static_theme", FakeSettingsSource::default().with_text(keys::theme::BACKEND, "static")),
        ("matugen", FakeSettingsSource::default().with_text(keys::theme::BACKEND, "matugen")),
        ("wallust", FakeSettingsSource::default().with_text(keys::theme::BACKEND, "wallust")),
        (
            "presets",
            FakeSettingsSource::default()
                .with_array_len(keys::filter_bar::RESOLUTION_PRESETS, 2)
                .with_array_len(keys::library::PROFILES, 2)
                .with_saved_theme("Custom"),
        ),
        ("hex", FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hex")),
        ("wall", FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "wall")),
        ("grid", FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "grid")),
        ("sandy", FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "sandy")),
        ("hand", FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hand")),
        ("depth", FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "depth")),
        (
            "collection",
            FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "collection"),
        ),
    ];
    cases.iter().map(|(name, cfg)| fixture(name, cfg, &outputs)).collect()
}

#[test]
fn dump_inventory() {
    println!(
        "SETTINGS_INVENTORY_JSON={}",
        json!({"baseline_revision":"c9f315794a8593e127c0293b2865c20b89477c6f","fixtures":all_fixtures()})
    );
}

fn control_contract(row: &Value, out: &mut Vec<Value>) {
    let control = &row["control"];
    let kind = control["kind"].as_str().unwrap_or_default();
    if !matches!(kind, "static" | "segment" | "preview") {
        let mut record = serde_json::Map::new();
        for field in [
            "kind",
            "path",
            "action",
            "id",
            "mode",
            "options",
            "disabled",
            "background",
            "weights",
            "unit",
            "placeholder",
            "width",
            "height",
            "items",
            "default",
            "snippet",
            "value",
        ] {
            if let Some(value) = control.get(field) {
                record.insert(field.to_string(), value.clone());
            }
        }
        out.push(Value::Object(record));
    }
    if let Some(rows) = control["rows"].as_array() {
        for child in rows {
            control_contract(child, out);
        }
    }
}

fn fixture_contract(fixture: &Value) -> Value {
    let mut controls = Vec::new();
    for place in fixture["places"].as_array().into_iter().flatten() {
        for section in place["sections"].as_array().into_iter().flatten() {
            for row in section["rows"].as_array().into_iter().flatten() {
                control_contract(row, &mut controls);
            }
        }
    }
    controls.sort_by_key(ToString::to_string);
    json!({"scenario":fixture["scenario"],"controls":controls})
}

const DISSOLVED_BARS: [&str; 1] = ["display:"];
const DISSOLVED_FOLDS: [&str; 16] = [
    "app-profile:example",
    "app-themes.custom",
    keys::lock_screens::MODE,
    "power.battery-limits",
    "search.discovery",
    "filterBar.resolutionPresets.0",
    "filterBar.resolutionPresets.0.custom",
    "filterBar.resolutionPresets.1",
    "filterBar.resolutionPresets.1.custom",
    "weather.advanced",
    "source.wallhaven",
    "source.unsplash",
    "source.pexels",
    "source.bing",
    "source.workshop",
    "source.youtube",
];

fn dissolved(control: &Value) -> bool {
    let id = control["id"].as_str().unwrap_or_default();
    match control["kind"].as_str() {
        Some("details") => DISSOLVED_FOLDS.contains(&id),
        Some("stack_bar") => DISSOLVED_BARS.iter().any(|bar| id.starts_with(bar)),
        _ => false,
    }
}

const RETIRED: [&str; 6] = [
    keys::selector::ROUND_CORNERS,
    keys::selector::GRID_ROUND_CORNERS,
    keys::selector::GRID_CORNER_RADIUS,
    keys::selector::HAND_CORNER_RADIUS,
    keys::selector::DEPTH_CORNERS,
    keys::selector::COLLECTION_CORNERS,
];

fn corner_group(control: &Value, baseline: &[Value]) -> bool {
    let path = control["path"].as_str().unwrap_or_default();
    skwd_config::corners::owner(path).is_some()
        && !baseline.iter().any(|old| old["path"] == control["path"])
}

fn layout_extra(control: &Value) -> bool {
    let action = control["action"].as_str().unwrap_or_default();
    action.starts_with("UsePremade")
        || control["items"].as_array().is_some_and(|items| {
            items
                .iter()
                .any(|item| item["action"].as_str().is_some_and(|id| id.starts_with("Nudge")))
        })
}

fn values(control: &Value) -> Vec<String> {
    control["options"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|option| option[0].as_str().map(str::to_string))
        .collect()
}

fn contract_id(control: &Value) -> String {
    control["action"].as_str().map_or_else(
        || format!("path:{}", control["path"].as_str().unwrap_or_default()),
        |action| format!("action:{action}"),
    )
}

#[test]
fn baseline_controls_preserved() {
    let expected: Value = serde_json::from_str(include_str!("settings_contract.json")).unwrap();
    let actual: Vec<_> = all_fixtures().iter().map(fixture_contract).collect();
    let expected = expected.as_array().unwrap();
    assert_eq!(actual.len(), expected.len());
    for (fixture, expected) in actual.iter().zip(expected) {
        assert_eq!(fixture["scenario"], expected["scenario"]);
        let mut remaining = fixture["controls"].as_array().unwrap().clone();
        remaining
            .retain(|item| !super::ownership_tests::ADDED.iter().any(|path| item["path"] == *path));
        let composites: Vec<_> = super::ownership_tests::COMPOSITES
            .iter()
            .filter_map(|(path, parts)| {
                let owners: Vec<_> =
                    remaining.iter().filter(|item| item["path"] == *path).cloned().collect();
                if owners.is_empty() {
                    return None;
                }
                assert_eq!(owners.len(), 1, "{path}");
                Some((*parts, values(&owners[0])))
            })
            .collect();
        let baseline = expected["controls"].as_array().unwrap();
        remaining.retain(|item| {
            !layout_extra(item)
                && !corner_group(item, baseline)
                && !super::ownership_tests::COMPOSITES.iter().any(|(path, _)| item["path"] == *path)
        });
        let retired = |control: &&Value| RETIRED.iter().any(|path| control["path"] == *path);
        for control in baseline.iter().filter(|control| !dissolved(control) && !retired(control)) {
            if let Some(index) = remaining.iter().position(|item| item == control) {
                remaining.swap_remove(index);
                continue;
            }
            let id = contract_id(control);
            let owner = composites.iter().find_map(|(parts, offered)| {
                parts.iter().find(|part| part.matches(&id)).map(|part| (part, offered))
            });
            let Some((part, offered)) = owner else { panic!("missing control {control}") };
            for value in part.needed(&id, &values(control)) {
                assert!(offered.contains(&value), "{id} needs {value}");
            }
        }
        assert!(remaining.is_empty(), "{remaining:?}");
    }
}

#[test]
fn unset_convergence_reads_never() {
    let never = tr("settings-library-watch-convergence-never");
    assert_eq!(convergence_label(None), never);
    assert_eq!(convergence_label(Some(0)), never);
    assert_ne!(convergence_label(Some(1)), never);
}
