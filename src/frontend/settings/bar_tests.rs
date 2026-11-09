use super::*;
use crate::frontend::settings::places::drawn;
use crate::frontend::settings::test_source::{
    FakeSettingsSource, place_source as source, plain as shown, render,
};
use crate::frontend::settings::{Parked, place};
use crate::frontend::ui::calm::sentinel;

const PRESETS: &str = bar::RESOLUTION_PRESETS;
const HOVERED: Choice = Choice { arg: VISIBILITY_ARG, values: ["always", "hover"] };
const HIDDEN: Choice = Choice { arg: VISIBILITY_ARG, values: ["always", "hidden"] };
const ARGS: [&str; 7] = ["val", "end", "buttons", "sorts", "style", "folder", "model"];

fn filled<'a>(
    slot: &str,
    mark: &'a str,
    choices: &[(&'a str, &'a str)],
) -> Vec<(&'a str, &'a str)> {
    let mut pairs: Vec<(&str, &str)> =
        ARGS.iter().map(|arg| (*arg, if *arg == slot { mark } else { "" })).collect();
    pairs.extend(choices);
    pairs
}

fn sections(cfg: &dyn SettingsSource) -> Vec<Section> {
    crate::i18n::with_language("en-US", || source(cfg).place(place::BAR).sections)
}

fn section(cfg: &dyn SettingsSource, key: &str) -> Section {
    sections(cfg).into_iter().find(|section| section.id.key == key).expect("section")
}

fn paths(rows: &[Row]) -> Vec<&str> {
    rows.iter().filter_map(|row| row.control.path()).collect()
}

fn actions(rows: &[Row]) -> Vec<ActionId> {
    rows.iter()
        .filter_map(|row| match &row.control {
            Control::ActionBtn { id, .. } => Some(id.clone()),
            _ => None,
        })
        .collect()
}

fn bands(labels: &[&str]) -> FakeSettingsSource {
    let cfg = FakeSettingsSource::default().with_array_len(PRESETS, labels.len());
    labels.iter().enumerate().fold(cfg, |cfg, (at, label)| {
        cfg.with_text(&format!("{PRESETS}.{at}.label"), label)
            .with_text(&format!("{PRESETS}.{at}.from"), "1920x1080")
    })
}

fn models(active: usize) -> FakeSettingsSource {
    let manifest = |at: usize| format!("/models/{at}/semantic-pack.json");
    FakeSettingsSource::default()
        .with_array_len(semantic::MODELS, 2)
        .with_text("semantic.models.0.manifest", &manifest(0))
        .with_text("semantic.models.1.manifest", &manifest(1))
        .with_text(semantic::MANIFEST, &manifest(active))
}

#[test]
fn board_sections_only() {
    let keys: Vec<_> =
        sections(&FakeSettingsSource::default()).iter().map(|section| section.id.key).collect();
    assert_eq!(keys, SECTIONS);
}

#[test]
fn filter_sentence_states() {
    let hidden = FakeSettingsSource::default()
        .with_flag(bar::SHOW_ORIENT, false)
        .with_flag(bar::SHOW_DOWNLOAD, false)
        .with_flag("filterBar.show.sort.pop", false)
        .with_flag("filterBar.show.sort.richness", false)
        .with_text(keys::general::FILTER_BAR_VISIBILITY, "hover")
        .with_text(bar::ORIENTATION, "vertical");
    for (cfg, expected) in [
        (
            FakeSettingsSource::default(),
            "Shows 14 of 16 buttons and 7 of 7 sorts, laid out horizontally. It stays pinned. Visual style: [0].",
        ),
        (
            hidden,
            "Shows 12 of 16 buttons and 5 of 7 sorts, laid out vertically. It shows while the pointer is over it. Visual style: [0].",
        ),
    ] {
        let filter = section(&cfg, FILTER);
        assert_eq!(shown(&filter.say.text), expected);
        assert_eq!(paths(&filter.head), [bar::VISUAL_STYLE]);
    }
}

#[test]
fn filter_body_rows() {
    let filter = section(&FakeSettingsSource::default(), FILTER);
    let headings: Vec<_> = filter.groups.iter().filter_map(|group| group.heading.clone()).collect();
    assert_eq!(headings, ["What it offers", "How it behaves"]);
    let [offers, behaves] = &filter.groups[..] else { panic!("two columns") };
    let cells: Vec<_> = offers.rows.iter().map(|row| row.look.cell).collect();
    let mut expected = vec![Cell::Checks];
    expected.extend([Cell::Joined; 16]);
    expected.push(Cell::Checks);
    expected.extend([Cell::Joined; 7]);
    expected.extend([Cell::Value; 4]);
    assert_eq!(cells, expected);
    assert_eq!(offers.rows[0].title, "Buttons on the bar");
    assert_eq!(offers.rows[17].title, "Sort options");
    assert_eq!(
        paths(&offers.rows[25..]),
        [bar::BUTTON_ORDER, bar::HIDDEN_BUTTONS, bar::SHOW_ARRANGE, bar::SHOW_SWATCH]
    );
    assert_eq!(offers.rows[27].title, "Arrange button");
    assert_eq!(offers.rows[28].title, "Colour strip");
    assert_eq!(offers.rows[27].hint(), "Place it in Arrange");
    assert_eq!(behaves.rows[4].title, "Tag panel");
    assert_eq!(behaves.rows[4].control.path(), Some(keys::selector::TAG_CLOUD_WIDTH));
    assert_eq!(behaves.rows[5].look.cell, Cell::Beside);
    assert_eq!(behaves.rows[0].look.cell, Cell::Buttons(&ORIENTATIONS));
    assert_eq!(behaves.rows[1].look.cell, Cell::Buttons(&VISIBILITIES));
    assert_eq!(behaves.rows[1].control.path(), Some(keys::general::FILTER_BAR_VISIBILITY));
    assert_eq!(behaves.rows[1].hint(), "Show the filter bar every time the picker opens.");
    assert_eq!(behaves.rows[2].control.path(), Some(keys::general::SEARCH_ALWAYS_VISIBLE));
    assert_eq!(behaves.rows[2].hint(), "Opens with the picker");
    assert_eq!(actions(&behaves.rows), [ActionId::ArrangePicker]);
    assert_eq!(filter.foot.as_deref(), Some("Where the bar sits stays in Layout."));
}

#[test]
fn checks_own_members() {
    let filter = section(&FakeSettingsSource::default(), FILTER);
    let lead = filter.header_rows();
    let members = filter.checks(lead).expect("buttons menu");
    assert_eq!(members, (lead + 1..=lead + 16).collect::<Vec<_>>());
    assert!(members.iter().all(|at| filter.checked_by(*at) == Some(lead) && filter.follows(*at)));
    assert_eq!(filter.checks(lead + 17).map(|rows| rows.len()), Some(7));
    assert_eq!(filter.checked_by(lead), None);
    assert!(filter.checks(lead + 1).is_none());
}

#[test]
fn members_found_themselves() {
    let cfg = FakeSettingsSource::default();
    let results = crate::i18n::with_language("en-US", || {
        crate::frontend::settings::search_settings("shape filter", source(&cfg))
    });
    let filter = section(&cfg, FILTER);
    let shape =
        filter.rows().position(|row| row.control.path() == Some(bar::SHOW_ORIENT)).expect("row");
    assert!(results.iter().any(|result| result.tab == place::BAR && result.row == shape));
}

#[test]
fn sizes_sentence_states() {
    for (cfg, expected) in [
        (bands(&[]), "No size bands yet, so the Size button has nothing to offer."),
        (bands(&["1080p", "", "4K"]), "The Size button offers 1080p, unnamed and 4K."),
    ] {
        assert_eq!(section(&cfg, SIZES).say.text, expected);
    }
}

#[test]
fn band_lines_hold_ranges() {
    let sizes = section(&bands(&["1080p"]), SIZES);
    let [lines, add] = &sizes.groups[..] else { panic!("bands and add") };
    assert_eq!(lines.kind, GroupKind::Lines);
    assert_eq!(lines.blank.as_deref(), Some("unnamed"));
    let base = format!("{PRESETS}.0");
    assert_eq!(
        paths(&lines.rows),
        ["label", "from", "to", "orientation"].map(|leaf| format!("{base}.{leaf}"))
    );
    let titles: Vec<_> = lines.rows[1..3].iter().map(|row| row.title.as_str()).collect();
    assert_eq!(titles, ["from", "up to"]);
    assert_eq!(actions(&lines.rows), [ActionId::RemoveResolutionPreset(0)]);
    assert_eq!(add.rows[0].look.cell, Cell::Menu);
    assert_eq!(add.rows[0].hint(), "A ready band, or a custom range");
}

#[test]
fn unset_orientation_wide() {
    let cfg = bands(&["1080p"]).with_text(&format!("{PRESETS}.0.orientation"), "tall");
    let current = |cfg: &FakeSettingsSource| {
        let sizes = section(cfg, SIZES);
        match &sizes.groups[0].rows[3].control {
            Control::Chips { current, .. } => current.clone(),
            _ => panic!("orientation chips"),
        }
    };
    assert_eq!(current(&bands(&["1080p"])), "wide");
    assert_eq!(current(&cfg), "tall");
}

#[test]
fn add_menu_skips_present() {
    let offered = |cfg: &FakeSettingsSource| {
        let sizes = section(cfg, SIZES);
        let Control::ActionChips { items } = &sizes.groups[1].rows[0].control else {
            panic!("add menu")
        };
        items.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>()
    };
    let band = ActionId::CreateResolutionPresetBand;
    assert_eq!(offered(&bands(&[])), ["fhd", "qhd", "4k", "5k", "8k", "custom"].map(band));
    assert_eq!(
        offered(&bands(&["1080p", "Custom range", "4K"])),
        ["qhd", "5k", "8k", "custom"].map(band)
    );
}

#[test]
fn start_sentence_states() {
    for (sticky, expected) in [
        (true, "The picker opens on [0] and keeps your filters after it closes."),
        (false, "The picker opens on [0] and resets your filters each time it opens."),
    ] {
        let cfg = FakeSettingsSource::default().with_flag(bar::STICKY, sticky);
        let start = section(&cfg, START);
        assert_eq!(shown(&start.say.text), expected);
        assert_eq!(paths(&start.head), [bar::DEFAULT_FOLDER]);
        assert_eq!(start.groups[0].rows[0].control.path(), Some(bar::STICKY));
    }
}

#[test]
fn search_sentence_states() {
    let describe = FakeSettingsSource::default()
        .with_text(keys::tagging::DEFAULT_SEARCH_MODE, "describe")
        .with_text(semantic::INDEX_PROFILE, "multiview");
    let off = FakeSettingsSource::default().with_flag(semantic::ENABLED, false);
    for (cfg, expected) in [
        (
            FakeSettingsSource::default(),
            "Search opens with Tags. Describe search uses [0] on the full image.",
        ),
        (
            describe,
            "Search opens with Describe. Describe search uses [0] on the composition and crops.",
        ),
        (
            off,
            "Search opens with Tags. Describe search is off: no model loads and nothing is indexed. Tags keep working.",
        ),
    ] {
        assert_eq!(shown(&section(&cfg, SEARCH).say.text), expected);
    }
}

#[test]
fn model_status_spoken() {
    let cfg = FakeSettingsSource::default();
    let status = PlaceSource { analysis: "Validating pack", ..source(&cfg) };
    let search = crate::i18n::with_language("en-US", || compose(&status)).pop().expect("search");
    assert!(shown(&search.say.text).ends_with(" Model status: Validating pack."));
}

#[test]
fn search_parks_when_off() {
    let off = FakeSettingsSource::default().with_flag(semantic::ENABLED, false);
    let search = section(&off, SEARCH);
    assert_eq!(search.say.parked.len(), 1);
    assert!(drawn(&search));
    assert!(section(&FakeSettingsSource::default(), SEARCH).say.parked.is_empty());
}

#[test]
fn parked_found_as_self() {
    let off = FakeSettingsSource::default().with_flag(semantic::ENABLED, false);
    let at = sections(&off).iter().position(|section| section.id.key == SEARCH).expect("search");
    let search = section(&off, SEARCH);
    let rows: Vec<Row> = search.rows().cloned().collect();
    let row = rows.iter().position(|row| row.control.path() == Some(semantic::MANIFEST)).unwrap();
    let cause = rows.iter().position(|row| row.control.path() == Some(semantic::ENABLED)).unwrap();
    assert_eq!(search.say.parked, [Parked { row, cause }]);
    let found = crate::i18n::with_language("en-US", || {
        crate::frontend::settings::search_settings("Active model", source(&off))
    });
    assert!(
        found.iter().any(|hit| hit.tab == place::BAR
            && hit.section == at
            && hit.row == row
            && hit.title == "Active model"),
        "{found:?}"
    );
}

#[test]
fn active_model_first() {
    let search = section(&models(1), SEARCH);
    let [searching, chosen] = &search.groups[..] else { panic!("two columns") };
    assert_eq!(
        paths(&searching.rows),
        [keys::tagging::DEFAULT_SEARCH_MODE, semantic::INDEX_PROFILE]
    );
    assert_eq!(
        actions(&searching.rows),
        [ActionId::ImportSemanticModel, ActionId::AddSemanticModel]
    );
    assert_eq!(paths(&chosen.rows), ["semantic.models.1.name", "semantic.models.1.manifest"]);
    assert_eq!(
        actions(&chosen.rows),
        [
            ActionId::RemoveSemanticModel(1),
            ActionId::DeleteActiveSemanticModel,
            ActionId::DeleteSemanticModel(0),
            ActionId::RemoveSemanticModel(0)
        ]
    );
    assert_eq!(chosen.rows[0].hint(), "Shown in the active-model list");
    let other = &chosen.rows[4];
    assert_eq!((other.title.as_str(), other.hint()), ("Model 1", "Not in use"));
    assert_eq!(chosen.rows[5].look.cell, Cell::Joined);
    assert!(search.foot.as_deref().is_some_and(|foot| foot.starts_with("Lens builds")));
}

#[test]
fn head_rows_drawn() {
    for cfg in [
        FakeSettingsSource::default(),
        bands(&["1080p", "", "4K"]),
        models(0),
        FakeSettingsSource::default().with_flag(semantic::ENABLED, false),
    ] {
        let composed = crate::i18n::with_language("en-US", || compose(&source(&cfg)));
        assert!(composed.iter().all(drawn));
        assert_eq!(composed.len(), SECTIONS.len());
    }
}

#[test]
fn sentences_keep_slots() {
    let mark = sentinel(0).expect("slot").to_string();
    let says = [
        (FILTER_SAY, "style", vec![ORIENTED, HOVERED]),
        (FILTER_SAY, "style", vec![ORIENTED, HIDDEN]),
        (START_SAY, "folder", vec![STICKY]),
        (SEARCH_SAY, "model", vec![MODE, VIEWED]),
    ];
    for language in crate::i18n::LANGUAGES {
        for (key, slot, choices) in &says {
            for on in [false, true] {
                let picks: Vec<_> = choices.iter().map(|choice| choice.pick(on)).collect();
                let text = render(language.tag, key, &filled(slot, &mark, &picks));
                assert_eq!(text.matches(mark.as_str()).count(), 1, "{} {key} {text}", language.tag);
            }
        }
    }
}

#[test]
fn selectors_change_text() {
    let says = [
        (FILTER_SAY, vec![ORIENTED, HOVERED]),
        (FILTER_SAY, vec![ORIENTED, HIDDEN]),
        (START_SAY, vec![STICKY]),
        (SEARCH_SAY, vec![MODE, VIEWED]),
        (SEARCH_OFF, vec![MODE]),
    ];
    for language in crate::i18n::LANGUAGES {
        for (key, choices) in &says {
            for flipped in 0..choices.len() {
                let say = |on: bool| {
                    let picks: Vec<_> = choices
                        .iter()
                        .enumerate()
                        .map(|(at, choice)| choice.pick(on && at == flipped))
                        .collect();
                    render(language.tag, key, &filled("", "", &picks))
                };
                assert_ne!(say(true), say(false), "{} {key} {flipped}", language.tag);
            }
        }
    }
}

#[test]
fn counts_name_numbers() {
    for language in crate::i18n::LANGUAGES {
        crate::i18n::with_language(language.tag, || {
            for key in [BUTTON_COUNT, SORT_COUNT] {
                for all in [7, 14] {
                    let text = tr_args!(key, shown => 5, all => all);
                    assert!(text.contains('5') && text.contains(&all.to_string()), "{text}");
                }
            }
        });
    }
}

#[test]
fn start_position_stays_layout() {
    let cfg = FakeSettingsSource::default();
    let results = crate::i18n::with_language("en-US", || {
        crate::frontend::settings::search_settings("Start position", source(&cfg))
    });
    assert_eq!(results.first().map(|result| result.tab.as_str()), Some(place::LAYOUT));
}

fn labels(rows: &[Row]) -> Vec<String> {
    rows.iter()
        .filter_map(|row| match &row.control {
            Control::ActionBtn { label, .. } => Some(label.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn removals_name_model() {
    let cfg = models(1)
        .with_text("semantic.models.0.name", "SigLIP2 B/16 large")
        .with_text("semantic.models.1.name", "PE-Core L14 336");
    let chosen = section(&cfg, SEARCH).groups.swap_remove(1).rows;
    assert_eq!(
        labels(&chosen),
        ["Forget model", "Delete model", "Delete SigLIP2 B/16 large", "Forget SigLIP2 B/16 large"]
    );
    assert_eq!(chosen[4].title, "SigLIP2 B/16 large");
}

#[test]
fn unnamed_model_numbered() {
    let chosen = section(&models(1), SEARCH).groups.swap_remove(1).rows;
    let named = labels(&chosen);
    assert_eq!(named[2..], ["Delete Model 1", "Forget Model 1"]);
}

#[test]
fn long_name_whole() {
    let long = "A very long semantic model name that nobody should have to read in a button";
    let cfg = models(1).with_text("semantic.models.0.name", long);
    let chosen = section(&cfg, SEARCH).groups.swap_remove(1).rows;
    let named = labels(&chosen);
    assert_eq!(named[2], format!("Delete {long}"));
    assert_eq!(named[3], format!("Forget {long}"));
    assert_eq!(chosen[4].title, long);
}

#[test]
fn every_locale_names() {
    for language in crate::i18n::LANGUAGES {
        crate::i18n::with_language(language.tag, || {
            for key in [DELETE_NAMED, FORGET_NAMED] {
                assert!(tr_args!(key, name => "PE-Core").contains("PE-Core"), "{}", language.tag);
            }
        });
    }
}

#[test]
fn off_keeps_rows() {
    let cfg = FakeSettingsSource::default()
        .with_flag(bar::SHOW_ARRANGE, false)
        .with_flag(bar::SHOW_SWATCH, false);
    let filter = section(&cfg, FILTER);
    let [offers, behaves] = &filter.groups[..] else { panic!("two columns") };
    assert_eq!(offers.rows[27].control.switched_on(), Some(false));
    assert_eq!(offers.rows[28].control.switched_on(), Some(false));
    assert_eq!(actions(&behaves.rows), [ActionId::ArrangePicker]);
    assert_eq!(filter.say.text, section(&FakeSettingsSource::default(), FILTER).say.text);
}

#[test]
fn floating_found_themselves() {
    let cfg = FakeSettingsSource::default().with_flag(bar::SHOW_SWATCH, false);
    for (query, path) in [("Arrange button", bar::SHOW_ARRANGE), ("Colour strip", bar::SHOW_SWATCH)]
    {
        let found = crate::i18n::with_language("en-US", || {
            crate::frontend::settings::search_settings(query, source(&cfg))
        });
        let filter = section(&cfg, FILTER);
        let row = filter.rows().position(|row| row.control.path() == Some(path)).expect("row");
        assert!(found.iter().any(|hit| hit.tab == place::BAR && hit.row == row), "{query}");
    }
}

fn visibility_hint(cfg: &FakeSettingsSource) -> String {
    let filter = section(cfg, FILTER);
    let row = filter.groups[1]
        .rows
        .iter()
        .find(|row| row.control.path() == Some(keys::general::FILTER_BAR_VISIBILITY));
    shown(row.expect("visibility row").hint())
}

#[test]
fn hidden_hint_names_binding() {
    use crate::domain::input::{InputAction, InputMap};
    let hidden = |raw: &str| {
        let map = InputMap::from_bindings([(InputAction::FilterBar, raw.to_owned())]);
        let cfg = FakeSettingsSource::default()
            .with_text(keys::general::FILTER_BAR_VISIBILITY, "hidden")
            .with_bindings(map);
        visibility_hint(&cfg)
    };
    assert_eq!(hidden("x"), "Hide the filter bar. Use the keybind X to restore access.");
    assert!(hidden("ctrl+b").contains("Ctrl+B"));
    assert_eq!(
        hidden("none"),
        "Hide the filter bar. It has no keybind yet, so bind one in Keybinds to restore access."
    );
    let hover =
        FakeSettingsSource::default().with_text(keys::general::FILTER_BAR_VISIBILITY, "hover");
    assert_eq!(
        visibility_hint(&hover),
        "Show the filter bar when the pointer reaches it. It hides again when the pointer leaves."
    );
}
