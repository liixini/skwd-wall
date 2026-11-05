use super::*;
use crate::contracts::settings::SettingsSource;
use crate::frontend::settings::place;
use crate::frontend::settings::places::drawn;
use crate::frontend::settings::search_settings;
use crate::frontend::settings::test_source::{FakeSettingsSource, place_source as source};

fn sheet(cfg: &dyn SettingsSource) -> Section {
    crate::i18n::with_language("en-US", || source(cfg).place(place::KEYBINDS).sections).remove(0)
}

fn map(bindings: &[(InputAction, &str)]) -> InputMap {
    InputMap::from_bindings(bindings.iter().map(|(action, raw)| (*action, (*raw).to_string())))
}

#[test]
fn groups_cover_all() {
    for group in KeyBindingGroup::ALL {
        assert!(SHEET_GROUPS.contains(&group));
    }
}

#[test]
fn board_group_order() {
    let cfg = FakeSettingsSource::default();
    let titles: Vec<_> = sheet(&cfg)
        .rows()
        .filter(|row| matches!(row.control, Control::Segment))
        .map(|row| row.title.clone())
        .collect();
    assert_eq!(
        titles,
        ["Wallpaper actions", "Folders", "Filters & tags", "Panels", "Navigation", "Downloads"]
    );
}

#[test]
fn every_action_once() {
    let cfg = FakeSettingsSource::default();
    let section = sheet(&cfg);
    for entry in KEY_BINDINGS {
        let found = section.rows().filter(|row| row.control.path() == Some(entry.path)).count();
        assert_eq!(found, 1, "{}", entry.path);
    }
}

#[test]
fn nothing_dropped() {
    let cfg = FakeSettingsSource::default();
    let section = sheet(&cfg);
    let mut cards = source(&cfg).cards(APP);
    let legacy = crate::i18n::with_language("en-US", || take_card(&mut cards, text(CONTROLS)));
    for row in legacy.iter().filter(|row| !matches!(row.control, Control::Segment)) {
        assert!(section.rows().any(|kept| kept.title == row.title), "{}", row.title);
    }
}

#[test]
fn stays_composed() {
    let cfg = FakeSettingsSource::default();
    let section = sheet(&cfg);
    assert!(drawn(&section));
    assert!(!section.has_body());
}

#[test]
fn reset_reads_defaults() {
    let cfg = FakeSettingsSource::default();
    let section = sheet(&cfg);
    let reset = section
        .rows()
        .find(|row| matches!(&row.control, Control::ActionBtn { id: ActionId::ResetKeybinds, .. }))
        .expect("reset");
    let Control::ActionBtn { label, .. } = &reset.control else {
        panic!("button");
    };
    assert_eq!(label, "Reset to defaults");
}

#[test]
fn fixed_rows_park() {
    let cfg = FakeSettingsSource::default();
    let section = sheet(&cfg);
    for (at, row) in section.rows().enumerate() {
        assert_eq!(
            section.say.parks(at).is_some(),
            matches!(row.control, Control::Static),
            "{}",
            row.title
        );
    }
}

#[test]
fn default_has_no_links() {
    let cfg = FakeSettingsSource::default();
    let section = sheet(&cfg);
    assert!((0..section.rows().count()).all(|at| !conflict_links(&section, at)));
}

#[test]
fn clash_makes_links() {
    let clash = map(&[(InputAction::Playlists, "f")]);
    let cfg = FakeSettingsSource::default().with_bindings(clash.clone());
    let section = sheet(&cfg);
    let at = (0..section.rows().count()).find(|at| conflict_links(&section, *at)).expect("links");
    let Some(Control::ActionChips { items }) = section.row(at).map(|row| row.control.clone())
    else {
        panic!("links");
    };
    assert_eq!(items[0].1, "Favourite card");
    assert_eq!(items[1].1, "Toggle playlists");
    assert_eq!(shared_key(&clash).map(|key| key.label()), Some(String::from("F")));
}

#[test]
fn nav_keys_derive() {
    assert_eq!(second_keys(&InputMap::default()), Some(NavKeys::Wasd));
    let vim = map(&[
        (InputAction::NavLeft, "left, h"),
        (InputAction::NavDown, "down, j"),
        (InputAction::NavUp, "up, k"),
        (InputAction::NavRight, "right, l"),
    ]);
    assert_eq!(second_keys(&vim), Some(NavKeys::Vim));
    let custom = map(&[(InputAction::NavLeft, "left, q"), (InputAction::NavUp, "up")]);
    assert_eq!(second_keys(&custom), None);
    let summary = crate::i18n::with_language("en-US", || second_summary(&custom));
    assert_eq!(summary, "Q D - S");
}

#[test]
fn find_matches_keys() {
    let labels = [String::from("Ctrl+Left")];
    assert!(matches_query("", "Previous folder", &labels));
    assert!(matches_query(" FOLDER ", "Previous folder", &labels));
    assert!(matches_query("ctrl+l", "Previous folder", &labels));
    assert!(!matches_query("tag", "Previous folder", &labels));
}

#[test]
fn keys_stay_findable() {
    let cfg = FakeSettingsSource::default();
    let found =
        crate::i18n::with_language("en-US", || search_settings("Favourite card", source(&cfg)));
    let hit = found.first().expect("result");
    assert_eq!(hit.tab, place::KEYBINDS);
    assert_eq!(hit.section, 0);
    let section = sheet(&cfg);
    assert_eq!(section.row(hit.row).map(|row| row.title.as_str()), Some("Favourite card"));
}

#[test]
fn undo_row_runs() {
    assert!(matches!(undo_row().control, Control::ActionBtn { id: ActionId::UndoKeyMove, .. }));
}

#[test]
fn bindings_cover_every_action() {
    use crate::contracts::picker::KEY_BINDINGS;
    let listed: Vec<_> = KEY_BINDINGS.into_iter().map(|entry| entry.action).collect();
    assert!(InputAction::ALL.iter().all(|action| listed.contains(action)));
    assert!(listed.iter().all(|action| InputAction::ALL.contains(action)));
    assert_eq!(listed.len(), InputAction::ALL.len());
}
