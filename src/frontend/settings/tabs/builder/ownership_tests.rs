use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::contracts::settings::schema;
use crate::frontend::settings::Control;
use crate::frontend::settings::tables::TABS;
use crate::frontend::settings::test_source::FakeSettingsSource;

fn control_ids(control: &Control) -> Vec<String> {
    match control {
        Control::Toggle { path, .. }
        | Control::Dropdown { path, .. }
        | Control::Chips { path, .. } => vec![format!("path:{path}")],
        Control::Number { path, .. }
        | Control::TextField { path, .. }
        | Control::KeyBinding { path, .. } => {
            vec![format!("path:{path}")]
        }
        Control::MotionWeights { weights } => {
            weights.iter().map(|(_, path, _)| format!("path:{path}")).collect()
        }
        Control::ActionBtn { id, .. } => vec![format!("action:{id:?}")],
        Control::Presets { mode, .. } => vec![format!("presets:{mode}")],
        Control::Details { rows, .. } | Control::StackBar { rows, .. } => {
            rows.iter().flat_map(|row| control_ids(&row.control)).collect()
        }
        Control::Static | Control::Code { .. } | Control::Preview => Vec::new(),
    }
}

fn flattened_rows(rows: Vec<Row>) -> Vec<Row> {
    let mut flattened = Vec::new();
    for row in rows {
        match row {
            Row {
                control: Control::Details { rows, .. } | Control::StackBar { rows, .. }, ..
            } => {
                flattened.extend(flattened_rows(rows));
            }
            row => flattened.push(row),
        }
    }
    flattened
}

fn ids(cards: Vec<(Card, Vec<Row>)>) -> Vec<String> {
    cards.into_iter().flat_map(|(_, rows)| rows).flat_map(|row| control_ids(&row.control)).collect()
}

fn source_cards(
    cfg: &dyn SettingsSource,
    themes: &[String],
    folders: &[String],
    _analysis: &str,
    backends: &[String],
    outputs: &[String],
) -> Vec<(Card, Vec<Row>)> {
    let mut cards = Vec::new();
    cards.extend(collect(cfg, |builder| tab_general(builder, themes, outputs)));
    cards.extend(collect(cfg, tab_launch));
    cards.extend(collect(cfg, tab_motion));
    cards.extend(collect(cfg, tab_selector));
    cards.extend(collect(cfg, |builder| tab_filter(builder, folders)));
    cards.extend(collect(cfg, tab_position));
    cards.extend(collect(cfg, tab_paper));
    cards.extend(collect(cfg, tab_transitions));
    cards.extend(collect(cfg, tab_schedule));
    cards.extend(collect(cfg, tab_paths));
    cards.extend(collect(cfg, tab_performance));
    cards.extend(collect(cfg, tab_postprocessing));
    cards.extend(collect(cfg, tab_keybinds));
    cards.extend(collect(cfg, tab_language));
    cards.extend(collect(cfg, |builder| tab_theme(builder, backends)));
    cards.extend(collect(cfg, tab_integrations));
    cards.extend(collect(cfg, tab_wallhaven));
    cards.extend(collect(cfg, tab_steam));
    cards.extend(collect(cfg, tab_wallpaper_engine));
    cards.extend(collect(cfg, tab_sources));
    cards.extend(collect(cfg, tab_search));
    cards.extend(collect(cfg, tab_matugen));
    section(
        &mut cards,
        tr("settings-section-library-watching"),
        tr("settings-library-watch-section-desc"),
        library_watch_rows(cfg, None),
    );
    cards
}

#[test]
fn controls_owned_once() {
    let cfg = FakeSettingsSource::default();
    let themes = Vec::new();
    let folders = Vec::new();
    let backends = Vec::new();
    let outputs = Vec::new();
    let mut expected: BTreeSet<_> =
        ids(source_cards(&cfg, &themes, &folders, "", &backends, &outputs)).into_iter().collect();
    // theme.backend stays a daemon compat key; the workbench owns policy/authority/engine.
    expected.remove("path:theme.backend");
    let mut actual = BTreeMap::<String, usize>::new();
    for tab in TABS {
        for id in ids(build_tab_with_outputs(tab, &cfg, &themes, &folders, "", &backends, &outputs))
        {
            *actual.entry(id).or_default() += 1;
        }
    }

    assert_eq!(actual.keys().cloned().collect::<BTreeSet<_>>(), expected);
    assert!(
        actual.iter().all(|(_, count)| *count == 1),
        "duplicates: {:?}",
        actual.iter().filter(|(_, count)| **count != 1).collect::<Vec<_>>()
    );
}

#[test]
fn controls_match_schema() {
    let configs = [
        FakeSettingsSource::default(),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hex"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "grid"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "sandy"),
        FakeSettingsSource::default().with_text(keys::paper::ENGINE, "awww"),
        FakeSettingsSource::default().with_text(keys::transition::SHADER, "sand-helix"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "static"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "matugen"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "wallust"),
    ];
    let mut missing = BTreeSet::new();
    let mut wrong = Vec::new();
    for cfg in &configs {
        for (_, rows) in source_cards(cfg, &[], &[], "", &[], &[]) {
            for row in flattened_rows(rows) {
                let (path, expected, numeric_text_field) = match row.control {
                    Control::Toggle { path, .. } => (path, schema::ValueKind::Boolean, false),
                    Control::Number { path, .. } => (path, schema::ValueKind::Number, false),
                    Control::TextField { path, .. } => (path, schema::ValueKind::Text, true),
                    Control::KeyBinding { path, .. } => (path, schema::ValueKind::Text, false),
                    Control::Dropdown { path, .. } | Control::Chips { path, .. } => {
                        let expected = schema::value_kind(&path).unwrap_or(schema::ValueKind::Text);
                        (path, expected, false)
                    }
                    Control::MotionWeights { weights } => {
                        for (_, path, _) in weights {
                            match schema::value_kind(&path) {
                                Some(schema::ValueKind::Number) => {}
                                Some(actual) => {
                                    wrong.push((path, actual, schema::ValueKind::Number));
                                }
                                None => {
                                    missing.insert(path);
                                }
                            }
                        }
                        continue;
                    }
                    Control::ActionBtn { .. }
                    | Control::Presets { .. }
                    | Control::Details { .. }
                    | Control::StackBar { .. }
                    | Control::Static
                    | Control::Code { .. }
                    | Control::Preview => continue,
                };
                if numeric_text_field
                    && schema::value_kind(&path) == Some(schema::ValueKind::Number)
                {
                    continue;
                }
                match schema::value_kind(&path) {
                    Some(actual) if actual == expected => {}
                    Some(actual) => wrong.push((path, actual, expected)),
                    None => {
                        missing.insert(path);
                    }
                }
            }
        }
    }
    assert!(missing.is_empty(), "{missing:#?}");
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn fixed_controls_in_schema() {
    let configs = [
        FakeSettingsSource::default(),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hex"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "wall"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "sandy"),
        FakeSettingsSource::default().with_text(keys::paper::ENGINE, "awww"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "matugen"),
    ];
    let mut missing = BTreeSet::new();
    for cfg in &configs {
        for tab in TABS {
            for id in ids(build_tab_with_outputs(tab, cfg, &[], &[], "", &[], &[])) {
                let Some(path) = id.strip_prefix("path:") else {
                    continue;
                };
                let dynamic = path.starts_with("filterBar.resolutionPresets.")
                    || path.starts_with("filterBar.show.type.")
                    || path.starts_with("filterBar.show.sort.")
                    || path.starts_with("transition.shaderScopes.")
                    || path.starts_with("postProcessing.")
                    || path.starts_with("integrations.");
                if !dynamic && schema::find(path).is_none() {
                    missing.insert(path.to_string());
                }
            }
        }
    }
    assert!(missing.is_empty(), "{missing:#?}");
}

#[test]
fn selectable_current_value() {
    let configs = [
        FakeSettingsSource::default(),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hex"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "grid"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "sandy"),
        FakeSettingsSource::default().with_text(keys::paper::ENGINE, "awww"),
        FakeSettingsSource::default().with_text(keys::transition::SHADER, "sand-helix"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "static"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "matugen"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "wallust"),
    ];
    for cfg in &configs {
        for tab in TABS {
            for (_, rows) in build_tab_with_outputs(tab, cfg, &[], &[], "", &[], &[]) {
                for row in flattened_rows(rows) {
                    let (Control::Dropdown { path, options, current }
                    | Control::Chips { path, options, current, .. }) = row.control
                    else {
                        continue;
                    };
                    assert!(
                        options.iter().any(|(value, _)| value == &current),
                        "{tab} / {path} / {current:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn controls_emit_valid_values() {
    let configs = [
        FakeSettingsSource::default(),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hex"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "wall"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "sandy"),
        FakeSettingsSource::default().with_text(keys::paper::ENGINE, "awww"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "matugen"),
    ];
    for cfg in &configs {
        for tab in TABS {
            for (_, rows) in build_tab_with_outputs(tab, cfg, &[], &[], "", &[], &[]) {
                for row in flattened_rows(rows) {
                    let values: Vec<(&str, serde_json::Value)> = match &row.control {
                        Control::Toggle { path, .. } => vec![(path, serde_json::json!(true))],
                        Control::Number { path, .. } => vec![(path, serde_json::json!(1))],
                        Control::TextField { path, .. } => {
                            let value =
                                if schema::value_kind(path) == Some(schema::ValueKind::Number) {
                                    serde_json::json!("1")
                                } else {
                                    serde_json::json!("value")
                                };
                            vec![(path, value)]
                        }
                        Control::KeyBinding { path, .. } => {
                            vec![(path, serde_json::json!("p"))]
                        }
                        Control::Dropdown { path, options, .. }
                        | Control::Chips { path, options, .. } => options
                            .iter()
                            .map(|(value, _)| (path.as_str(), serde_json::json!(value)))
                            .collect(),
                        Control::MotionWeights { weights } => weights
                            .iter()
                            .map(|(_, path, _)| (path.as_str(), serde_json::json!(1)))
                            .collect(),
                        Control::ActionBtn { .. }
                        | Control::Presets { .. }
                        | Control::Details { .. }
                        | Control::StackBar { .. }
                        | Control::Static
                        | Control::Code { .. }
                        | Control::Preview => Vec::new(),
                    };
                    for (path, value) in values {
                        assert!(
                            schema::normalize_value(path, &value).is_some(),
                            "{tab} / {path} / {value}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn workshop_native_settings() {
    let cfg = FakeSettingsSource::default();
    let exposed: BTreeSet<_> = ids(collect(&cfg, tab_wallpaper_engine)).into_iter().collect();
    assert_eq!(
        exposed,
        [
            format!("path:{}", keys::we_render::FPS),
            format!("path:{}", keys::we_render::SCALING),
            format!("path:{}", keys::we_render::DISABLE_PARTICLES),
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn card_subtitles_authored() {
    let cfg = FakeSettingsSource::default();
    let mut subtitled = 0usize;
    let mut total = 0usize;
    for tab in TABS {
        for (card, _) in build_tab_with_outputs(tab, &cfg, &[], &[], "", &[], &[]) {
            total += 1;
            if card.subtitle.is_empty() {
                continue;
            }
            subtitled += 1;
            assert!(!card.subtitle.ends_with("-card-desc"), "{}", card.subtitle);
            assert_ne!(card.subtitle, card.title);
        }
    }
    assert!(subtitled * 2 >= total, "{subtitled}/{total}");
}
