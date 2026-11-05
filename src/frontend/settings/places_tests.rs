use super::*;
use crate::contracts::settings::keys;
use crate::frontend::settings::test_source::{FakeSettingsSource, place_source as source};

fn fixtures() -> Vec<FakeSettingsSource> {
    vec![
        FakeSettingsSource::default(),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hand"),
        FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "sandy"),
        FakeSettingsSource::default().with_niri(true),
    ]
}

fn row(title: &str, control: Control) -> Row {
    Row::of(title.to_string(), String::new(), control)
}

fn toggle(path: &str) -> Row {
    row(path, Control::Toggle { path: path.to_string(), value: false })
}

#[test]
fn no_legacy_places() {
    let legacy: Vec<_> =
        PLACES.iter().filter(|spec| composer(spec.key).is_none()).map(|spec| spec.key).collect();
    assert!(legacy.is_empty(), "{legacy:?}");
}

#[test]
fn routes_reach_places() {
    for (tab, place) in [
        ("picker", place::LAYOUT),
        ("search", place::BAR),
        ("filter", place::BAR),
        ("position", place::LAYOUT),
        ("motion", place::LAYOUT),
        ("displays", place::WALLPAPERS),
        ("playback", place::WALLPAPERS),
        ("transitions", place::WALLPAPERS),
        ("theme", place::COLOURS),
        ("integrations", place::COLOURS),
        ("performance", place::SYSTEM),
        ("language", place::SYSTEM),
        ("app", place::SYSTEM),
        ("library", place::LIBRARY),
        ("keybinds", place::KEYBINDS),
    ] {
        assert_eq!(canonical_place(tab), Some(place), "{tab}");
    }
    assert_eq!(canonical_place("editorial"), None);
}

#[test]
fn layout_leaves_bar() {
    let cfg = FakeSettingsSource::default();
    let layout = source(&cfg).place(place::LAYOUT);
    let keys: Vec<_> = layout.sections.iter().map(|section| section.id.key).collect();
    assert!(keys.iter().all(|key| !super::super::bar::CARDS.contains(key)), "{keys:?}");
}

#[test]
fn segments_become_headings() {
    let segment = |title: &str| row(title, Control::Segment);
    let groups = groups(vec![
        segment("Picker"),
        toggle("a"),
        segment("Cards"),
        toggle("b"),
        segment("Motion"),
        toggle("c"),
    ]);
    let headings: Vec<_> = groups.iter().map(|group| group.heading.as_deref()).collect();
    assert_eq!(headings, [Some("Picker"), Some("Cards")]);
    let second: Vec<_> = groups[1].rows.iter().map(|row| row.title.as_str()).collect();
    assert_eq!(second, ["b", "Motion", "c"]);
}

#[test]
fn lead_marks_token() {
    let number = row(
        "Max FPS",
        Control::Number { key: String::from("fps"), path: String::from("fps"), unit: "fps" },
    );
    let section = legacy(
        SectionId::new(category::APP, "power"),
        &Card { key: "power", title: "Power", subtitle: "" },
        vec![toggle("Saver"), number],
    );
    assert_eq!(section.say.rows().collect::<Vec<_>>(), [1]);
    let mark = crate::frontend::ui::calm::sentinel(0).expect("slot");
    assert!(section.say.text.contains(mark));
    assert!(section.say.text.contains("Saver"));
}

#[test]
fn plain_summary_without_tokens() {
    let section = legacy(
        SectionId::new(category::APP, "power"),
        &Card { key: "power", title: "Power", subtitle: "" },
        (0..5).map(|at| toggle(&format!("t{at}"))).collect(),
    );
    assert!(section.say.tokens.is_empty());
    assert_eq!(section.say.text, "t0 · t1 · t2 · t3 · t4");
}

fn number(path: &str) -> Row {
    row(path, Control::Number { key: path.to_string(), path: path.to_string(), unit: "" })
}

#[test]
fn drawn_needs_tokens() {
    let mut section = Section {
        head: vec![number("fps")],
        ..Section::new(SectionId::new(category::APP, "power"), "Power", "")
    };
    assert!(!drawn(&section));
    section.say.tokens = vec![Slot::Value { row: 0, blank: None, form: Form::Plain }];
    assert!(drawn(&section));
    section.master = Some(number("saver"));
    assert!(!drawn(&section));
}

#[test]
fn bind_needs_rows() {
    let section = Section {
        head: vec![number("fps")],
        ..Section::new(SectionId::new(category::APP, "power"), "Power", "")
    };
    let rate = [(LEAD_SETTING, "Rate")];
    let say = bind(&section, LEAD_KEY, &[Bind::new(LEAD_VALUE, "fps")], &rate).expect("bound");
    assert_eq!(say.rows().collect::<Vec<_>>(), [0]);
    assert!(bind(&section, LEAD_KEY, &[Bind::new(LEAD_VALUE, "idle")], &rate).is_none());
}

#[test]
fn unbound_say_falls_back() {
    let section = Section {
        head: vec![number("fps")],
        ..Section::new(SectionId::new(category::APP, "power"), "Power", "")
    };
    let kept = spoken(section, |_| None);
    assert!(kept.head.is_empty());
    assert_eq!(kept.rows().filter_map(|row| row.control.path()).collect::<Vec<_>>(), ["fps"]);
    assert!(drawn(&kept));
}

#[test]
fn undrawn_sections_wrapped() {
    let section = Section {
        master: Some(number("saver")),
        head: vec![number("fps")],
        ..Section::new(SectionId::new(category::APP, "power"), "Power", "")
    };
    let kept = kept(section);
    assert!(kept.master.is_none() && kept.head.is_empty());
    assert_eq!(
        kept.rows().filter_map(|row| row.control.path()).collect::<Vec<_>>(),
        ["saver", "fps"]
    );
}

#[test]
fn composed_every_locale() {
    let cfg = FakeSettingsSource::default();
    for Composed { place, compose, .. } in COMPOSED {
        let shape = |tag: &str| {
            crate::i18n::with_language(tag, || {
                compose(&source(&cfg))
                    .into_iter()
                    .map(|section| {
                        (
                            section.id,
                            section.master.is_some(),
                            section.head.len(),
                            section.say.rows().collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
        };
        let english = shape("en-US");
        for language in crate::i18n::LANGUAGES {
            let catalog = crate::i18n::Catalog::for_locale(language.tag);
            assert_eq!(shape(language.tag), english, "{place} {}", language.tag);
            for (id, ..) in &english {
                assert!(!catalog.format(id.key, None).is_empty(), "{} {}", language.tag, id.key);
            }
        }
    }
}

#[test]
fn value_spans_guarded() {
    let token = crate::frontend::ui::calm::sentinel(0).expect("slot");
    for (text, ok) in [
        (format!("On {VALUE_OPEN}startup{VALUE_CLOSE} {token}."), true),
        (format!("{VALUE_CLOSE}off{VALUE_OPEN}"), false),
        (format!("{VALUE_OPEN}a{VALUE_OPEN}b{VALUE_CLOSE}{VALUE_CLOSE}"), false),
        (format!("{VALUE_OPEN}every {token}{VALUE_CLOSE}"), false),
        (format!("{VALUE_OPEN}open"), false),
    ] {
        assert_eq!(spans_close(&text), ok, "{text:?}");
    }
}

#[test]
fn bind_spans_rows() {
    let section = Section {
        head: vec![toggle("general"), toggle("anime"), toggle("people")],
        ..Section::new(SectionId::new(category::APP, "filters"), "Filters", "")
    };
    let label = "settings-toggle-on";
    let checks =
        |paths| Spec::Checks(Checks { arg: LEAD_VALUE, paths, heading: label, blank: label });
    let rate = [(LEAD_SETTING, "Show")];
    let say = bind(&section, LEAD_KEY, &[checks(&["general", "anime", "people"])], &rate);
    assert_eq!(say.expect("bound").rows().collect::<Vec<_>>(), [0, 1, 2]);
    assert!(bind(&section, LEAD_KEY, &[checks(&["general", "sketchy"])], &rate).is_none());
}

#[test]
fn parking_needs_cause() {
    let mut section = Section {
        master: Some(toggle("enabled")),
        head: vec![number("sort"), number("range")],
        ..Section::new(SectionId::new(category::APP, "browser"), "Browser", "")
    };
    section.say =
        bind(&section, LEAD_KEY, &[Bind::new(LEAD_VALUE, "sort")], &[(LEAD_SETTING, "x")])
            .expect("bound");
    assert!(!drawn(&section));
    park(&mut section, "sort", |row| row.control.path() == Some("range"));
    assert_eq!(section.say.parked, [Parked { row: 2, cause: 1 }]);
    assert!(drawn(&section));
    park(&mut section, "elsewhere", |_| true);
    assert_eq!(section.say.parked, [Parked { row: 2, cause: 1 }], "unknown cause ignored");
    park(&mut section, "range", |row| row.control.path() == Some("sort"));
    assert!(!drawn(&section), "cause must be shown");
    section.say = Say::default();
    park(&mut section, "enabled", |_| true);
    assert!(drawn(&section));
}

#[test]
fn lists_join_items() {
    crate::i18n::with_language("en-US", || {
        assert_eq!(listed(LIST_KEY, &["a"]), "a");
        assert_eq!(listed(LIST_KEY, &["a", "b"]), "a and b");
        assert_eq!(listed(LIST_KEY, &["a", "b", "c", "d"]), "a, b, c and d");
        assert_eq!(listed(LIST_KEY, &["a", "b", "c", "d", "e"]), "a, b, c and others");
    });
}

fn owned(rows: &[Row], paths: &mut Vec<String>) {
    for row in rows {
        match &row.control {
            Control::Details { rows, .. } | Control::StackBar { rows, .. } => owned(rows, paths),
            control => paths.extend(control.path().map(str::to_string)),
        }
    }
}

#[test]
fn one_owner_each() {
    let mut owners: Vec<(String, &str)> = Vec::new();
    for cfg in fixtures() {
        let places = crate::i18n::with_language("en-US", || source(&cfg).places());
        let mut states = Vec::new();
        for place in &places {
            let mut paths = Vec::new();
            for section in &place.sections {
                let rows: Vec<Row> = section.rows().cloned().collect();
                owned(&rows, &mut paths);
            }
            let mut unique = paths.clone();
            unique.sort_unstable();
            unique.dedup();
            let twice: Vec<_> = unique
                .iter()
                .filter(|path| paths.iter().filter(|seen| seen == path).count() > 1)
                .collect();
            assert!(twice.is_empty(), "{} twice: {twice:?}", place.key);
            states.extend(unique.into_iter().map(|path| (path, place.key)));
        }
        owners.extend(states);
    }
    owners.sort();
    owners.dedup();
    let split: Vec<_> = owners.windows(2).filter(|pair| pair[0].0 == pair[1].0).collect();
    assert!(split.is_empty(), "{split:?}");
}
