use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::contracts::settings::schema;
use crate::frontend::settings::tables::TABS;
use crate::frontend::settings::test_source::FakeSettingsSource;
use crate::frontend::settings::{ActionId, Control};

fn control_ids(control: &Control) -> Vec<String> {
    match control {
        Control::Toggle { path, .. }
        | Control::Colour { path, .. }
        | Control::Dropdown { path, .. }
        | Control::Chips { path, .. } => vec![format!("path:{path}")],
        Control::Number { path, .. }
        | Control::TextField { path, .. }
        | Control::Resolution { path, .. }
        | Control::KeyBinding { path, .. } => {
            vec![format!("path:{path}")]
        }
        Control::ActionBtn { id, .. } | Control::ToggleAction { id, .. } => {
            if matches!(id, ActionId::UsePremade(_)) {
                return Vec::new();
            }
            vec![format!("action:{id:?}")]
        }
        Control::ActionChips { items } => items
            .iter()
            .filter(|(id, _)| !matches!(id, ActionId::Nudge(..) | ActionId::CentreNudge(_)))
            .map(|(id, _)| format!("action:{id:?}"))
            .collect(),
        Control::Details { rows, .. } | Control::StackBar { rows, .. } => {
            rows.iter().flat_map(|row| control_ids(&row.control)).collect()
        }
        Control::Display(_)
        | Control::Segment
        | Control::Static
        | Control::Code { .. }
        | Control::AppTheme { .. }
        | Control::Preview => Vec::new(),
    }
}

pub(super) enum Needs {
    Value(&'static str),
    Options,
    Argument,
}

pub(super) struct Part {
    pub id: &'static str,
    pub needs: Needs,
}

impl Part {
    pub(super) fn matches(&self, id: &str) -> bool {
        if self.id.ends_with('(') { id.starts_with(self.id) } else { id == self.id }
    }

    pub(super) fn needed(&self, id: &str, options: &[String]) -> Vec<String> {
        match self.needs {
            Needs::Value(value) => vec![value.to_string()],
            Needs::Options => options.to_vec(),
            Needs::Argument => id.split('"').nth(1).map(str::to_string).into_iter().collect(),
        }
    }
}

pub(super) const ADDED: [&str; 1] = [keys::library::PROFILE];

pub(super) const COMPOSITES: [(&str, &[Part]); 7] = [
    (crate::frontend::settings::PREMADE_KEY, &[]),
    (keys::motion::FAST_MS, &[]),
    (keys::motion::STANDARD_MS, &[]),
    (keys::motion::SLOW_MS, &[]),
    (
        crate::frontend::settings::WEATHER_MODE_KEY,
        &[
            Part {
                id: "action:FollowOutdoors(",
                needs: Needs::Value(keys::weather::SOURCE_OUTDOORS),
            },
            Part {
                id: "path:weather.enabled",
                needs: Needs::Value(crate::frontend::settings::WEATHER_OFF),
            },
            Part { id: "path:weather.source", needs: Needs::Options },
        ],
    ),
    (
        crate::frontend::settings::WEATHER_REVIEW_KEY,
        &[Part { id: "action:ReviewWeather(", needs: Needs::Argument }],
    ),
    (
        crate::frontend::settings::DRAWING_SCOPE_KEY,
        &[Part { id: "path:drawing.scope", needs: Needs::Options }],
    ),
];

pub(super) fn composed_part(id: &str, parts: &[Part]) -> bool {
    parts.iter().any(|part| part.matches(id))
}

fn option_values(control: &Control) -> Vec<String> {
    match control {
        Control::Dropdown { options, .. } | Control::Chips { options, .. } => {
            options.iter().map(|(value, _)| value.clone()).collect()
        }
        _ => Vec::new(),
    }
}

fn composite_gaps(builder: &[Row], place: &[Row]) -> Vec<String> {
    let mut gaps = Vec::new();
    for (path, parts) in COMPOSITES {
        let owners: Vec<_> = place.iter().filter(|row| row.control.path() == Some(path)).collect();
        let covered: Vec<_> = builder
            .iter()
            .flat_map(|row| control_ids(&row.control).into_iter().map(move |id| (id, row)))
            .filter_map(|(id, row)| {
                parts.iter().find(|part| part.matches(&id)).map(|part| (part, id, row))
            })
            .collect();
        if covered.is_empty() {
            continue;
        }
        if owners.len() != 1 {
            gaps.push(format!("{path}: {} rows", owners.len()));
            continue;
        }
        let offered = option_values(&owners[0].control);
        for (part, id, row) in covered {
            for value in part.needed(&id, &option_values(&row.control)) {
                if !offered.contains(&value) {
                    gaps.push(format!("{path}: {value}"));
                }
            }
        }
    }
    gaps
}

fn composite_rows(cfg: &FakeSettingsSource, edit: impl Fn(&mut Control)) -> (Vec<Row>, Vec<Row>) {
    let builder = flattened_rows(
        source_cards(cfg, &[], &[], "", &[], &[]).into_iter().flat_map(|(_, rows)| rows).collect(),
    );
    let source = crate::frontend::settings::test_source::place_source(cfg);
    let mut place: Vec<Row> = source
        .places()
        .into_iter()
        .flat_map(|place| place.sections)
        .flat_map(crate::frontend::settings::Section::into_rows)
        .collect();
    for row in &mut place {
        edit(&mut row.control);
    }
    (builder, place)
}

fn search_source() -> FakeSettingsSource {
    FakeSettingsSource::default().with_text(keys::weather::SOURCE, keys::weather::SOURCE_SEMANTIC)
}

#[test]
fn composites_cover_parts() {
    for cfg in [FakeSettingsSource::default(), search_source()] {
        let (builder, place) = composite_rows(&cfg, |_| {});
        assert!(
            composite_gaps(&builder, &place).is_empty(),
            "{:?}",
            composite_gaps(&builder, &place)
        );
    }
}

#[test]
fn composite_gaps_caught() {
    let drop = |path: &'static str, value: &'static str| {
        move |control: &mut Control| {
            if let Control::Dropdown { path: owner, options, .. } = control
                && owner == path
            {
                options.retain(|(option, _)| option != value);
            }
        }
    };
    let mode = drop(crate::frontend::settings::WEATHER_MODE_KEY, keys::weather::SOURCE_SEMANTIC);
    let (builder, place) = composite_rows(&FakeSettingsSource::default(), mode);
    assert_eq!(composite_gaps(&builder, &place), ["weather.mode: semantic"]);
    let review = drop(crate::frontend::settings::WEATHER_REVIEW_KEY, "rainy");
    let (builder, place) = composite_rows(&search_source(), review);
    assert_eq!(composite_gaps(&builder, &place), ["weather.review: rainy"]);
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
    let mut cards = vec![(
        Card {
            key: "settings-app-themes-title",
            title: tr("settings-app-themes-title"),
            subtitle: "",
        },
        super::super::app_themes::rows(cfg, None),
    )];
    cards.extend(collect(cfg, |builder| tab_general(builder, themes, outputs)));
    cards.extend(collect(cfg, tab_launch));
    cards.extend(collect(cfg, tab_selector));
    cards.extend(collect(cfg, |builder| tab_filter(builder, folders)));
    cards.extend(collect(cfg, tab_position));
    cards.extend(collect(cfg, tab_paper));
    cards.extend(collect(cfg, tab_transitions));
    cards.extend(collect(cfg, |builder| tab_drawing(builder, None, None, 0, false)));
    cards.extend(collect(cfg, tab_schedule));
    cards.extend(collect(cfg, |builder| tab_weather(builder, None, &[], false)));
    cards.extend(collect(cfg, super::super::folders_tab::tab_paths));
    cards.extend(collect(cfg, super::super::profiles_tab::tab_profiles));
    cards.extend(collect(cfg, tab_performance));
    cards.extend(collect(cfg, tab_postprocessing));
    cards.extend(collect(cfg, tab_keybinds));
    cards.extend(collect(cfg, tab_language));
    cards.extend(collect(cfg, |builder| tab_theme(builder, backends)));
    cards.extend(collect(cfg, tab_integrations));
    cards.extend(collect(cfg, tab_lockscreens));
    cards.extend(collect(cfg, tab_wallhaven));
    cards.extend(collect(cfg, tab_steam));
    cards.extend(collect(cfg, tab_wallpaper_engine));
    cards.extend(collect(cfg, tab_sources));
    cards.extend(collect(cfg, tab_search));
    cards.extend(collect(cfg, tab_matugen));
    section(
        &mut cards,
        crate::i18n::text("settings-section-library-watching"),
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
    let source = crate::frontend::settings::PlaceSource {
        cfg: &cfg,
        themes: &themes,
        folders: &folders,
        analysis: "",
        backends: &backends,
        runtime: RuntimeStatus { outputs: &outputs, ..Default::default() },
    };
    let rows = source
        .places()
        .into_iter()
        .flat_map(|place| place.sections)
        .flat_map(crate::frontend::settings::Section::into_rows);
    for id in rows.flat_map(|row| control_ids(&row.control)) {
        *actual.entry(id).or_default() += 1;
    }
    for (composite, parts) in COMPOSITES {
        if actual.remove(&format!("path:{composite}")).is_some() {
            for id in expected.iter().filter(|id| composed_part(id, parts)) {
                *actual.entry(id.clone()).or_default() += 1;
            }
        }
    }

    for added in ADDED {
        actual.remove(&format!("path:{added}"));
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
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "depth"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "collection"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hex"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "grid"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "sandy"),
        FakeSettingsSource::default().with_text(keys::paper::ENGINE, "awww"),
        FakeSettingsSource::default().with_text(keys::transition::SHADER, "sand-helix"),
        FakeSettingsSource::default().with_text(keys::transition::SHADER, "photo-cutout"),
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
                    Control::TextField { path, .. } | Control::Resolution { path, .. } => {
                        (path, schema::ValueKind::Text, true)
                    }
                    Control::KeyBinding { path, .. } | Control::Colour { path, .. } => {
                        (path, schema::ValueKind::Text, false)
                    }
                    Control::Dropdown { path, .. } | Control::Chips { path, .. } => {
                        let expected = schema::value_kind(&path).unwrap_or(schema::ValueKind::Text);
                        (path, expected, false)
                    }
                    Control::ToggleAction { .. }
                    | Control::ActionBtn { .. }
                    | Control::ActionChips { .. }
                    | Control::Details { .. }
                    | Control::StackBar { .. }
                    | Control::Display(_)
                    | Control::Segment
                    | Control::Static
                    | Control::Code { .. }
                    | Control::AppTheme { .. }
                    | Control::Preview => continue,
                };
                if numeric_text_field
                    && schema::value_kind(&path) == Some(schema::ValueKind::Number)
                {
                    continue;
                }
                if [keys::library::FOLDERS, keys::library::EXCLUDED_FOLDERS]
                    .iter()
                    .any(|key| path.starts_with(key))
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
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "depth"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "collection"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hex"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "wall"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "sandy"),
        FakeSettingsSource::default().with_text(keys::paper::ENGINE, "awww"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "matugen"),
    ];
    let mut missing = BTreeSet::new();
    for cfg in &configs {
        for tab in TABS {
            for id in ids(build_tab_with_runtime_status(
                tab,
                cfg,
                &[],
                &[],
                "",
                &[],
                RuntimeStatus { outputs: &[], ..Default::default() },
            )) {
                let Some(path) = id.strip_prefix("path:") else {
                    continue;
                };
                let dynamic = path.starts_with("filterBar.resolutionPresets.")
                    || [keys::library::FOLDERS, keys::library::EXCLUDED_FOLDERS]
                        .iter()
                        .any(|key| path.starts_with(key))
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
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "depth"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "collection"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hex"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "grid"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "sandy"),
        FakeSettingsSource::default().with_text(keys::paper::ENGINE, "awww"),
        FakeSettingsSource::default().with_text(keys::transition::SHADER, "sand-helix"),
        FakeSettingsSource::default().with_text(keys::transition::SHADER, "photo-cutout"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "static"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "matugen"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "wallust"),
    ];
    for cfg in &configs {
        for tab in TABS {
            for (_, rows) in build_tab_with_runtime_status(
                tab,
                cfg,
                &[],
                &[],
                "",
                &[],
                RuntimeStatus { outputs: &[], ..Default::default() },
            ) {
                for row in flattened_rows(rows) {
                    let (Control::Dropdown { path, options, current, .. }
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
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "depth"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "collection"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hex"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "wall"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "sandy"),
        FakeSettingsSource::default().with_text(keys::paper::ENGINE, "awww"),
        FakeSettingsSource::default().with_text(keys::theme::BACKEND, "matugen"),
    ];
    for cfg in &configs {
        for tab in TABS {
            for (_, rows) in build_tab_with_runtime_status(
                tab,
                cfg,
                &[],
                &[],
                "",
                &[],
                RuntimeStatus { outputs: &[], ..Default::default() },
            ) {
                for row in flattened_rows(rows) {
                    let values: Vec<(&str, serde_json::Value)> = match &row.control {
                        Control::Toggle { path, .. } => vec![(path, serde_json::json!(true))],
                        Control::Colour { path, .. } => vec![(path, serde_json::json!("#141816"))],
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
                        Control::Resolution { path, .. } => {
                            vec![(path, serde_json::json!("1920x1080"))]
                        }
                        Control::KeyBinding { path, .. } => {
                            vec![(path, serde_json::json!("p"))]
                        }
                        Control::Dropdown { path, options, .. }
                        | Control::Chips { path, options, .. } => options
                            .iter()
                            .map(|(value, _)| (path.as_str(), serde_json::json!(value)))
                            .collect(),
                        Control::ToggleAction { .. }
                        | Control::ActionBtn { .. }
                        | Control::ActionChips { .. }
                        | Control::Details { .. }
                        | Control::StackBar { .. }
                        | Control::Display(_)
                        | Control::Segment
                        | Control::Static
                        | Control::Code { .. }
                        | Control::AppTheme { .. }
                        | Control::Preview => Vec::new(),
                    };
                    for (path, value) in values {
                        if [keys::library::FOLDERS, keys::library::EXCLUDED_FOLDERS]
                            .iter()
                            .any(|key| path.starts_with(key))
                        {
                            continue;
                        }
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
        for (card, _) in build_tab_with_runtime_status(
            tab,
            &cfg,
            &[],
            &[],
            "",
            &[],
            RuntimeStatus { outputs: &[], ..Default::default() },
        ) {
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

#[test]
fn new_layout_controls_have_one_composed_owner() {
    for mode in ["depth", "collection"] {
        let cfg = FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, mode);
        let expected: BTreeSet<_> = ids(collect(&cfg, tab_selector))
            .into_iter()
            .filter(|id| id.starts_with(&format!("path:components.wallpaperSelector.{mode}")))
            .collect();
        assert_eq!(expected.len(), if mode == "depth" { 16 } else { 12 });
        let actual: Vec<_> = TABS
            .iter()
            .flat_map(|tab| {
                ids(build_tab_with_runtime_status(
                    tab,
                    &cfg,
                    &[],
                    &[],
                    "",
                    &[],
                    RuntimeStatus { outputs: &[], ..Default::default() },
                ))
            })
            .collect();
        for id in expected {
            assert_eq!(actual.iter().filter(|value| **value == id).count(), 1, "{id}");
        }
    }
}
