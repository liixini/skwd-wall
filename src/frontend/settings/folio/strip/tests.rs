use super::*;
use crate::frontend::settings::LayoutPending;

fn view(changes: usize, names: &[&str]) -> SavedLayouts {
    SavedLayouts {
        names: names.iter().map(ToString::to_string).collect(),
        active: Some(0),
        changes,
    }
}

fn messages(entries: &[StripEntry]) -> Vec<Option<LayoutMsg>> {
    entries
        .iter()
        .filter_map(|entry| match entry {
            StripEntry::Item { message, .. } => Some(message.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn lists_board_order() {
    crate::i18n::with_language("en-US", || {
        let found = messages(&strip_entries(&view(0, &["Desk", "Laptop"]), LayoutsUi::default()));
        assert_eq!(found[0], Some(LayoutMsg::Pick(0)));
        assert_eq!(found[2], Some(LayoutMsg::SaveNew));
        assert_eq!(found[3], Some(LayoutMsg::Premade(0)));
        assert_eq!(found[9], Some(LayoutMsg::Rename));
        assert_eq!(found[11], Some(LayoutMsg::Delete));
    });
}

#[test]
fn confirm_swaps_items() {
    let ui = LayoutsUi { confirm: Some(LayoutPending::Delete), ..LayoutsUi::default() };
    let found = messages(&strip_entries(&view(2, &["Desk"]), ui));
    assert_eq!(
        found,
        [
            Some(LayoutMsg::ConfirmSave),
            Some(LayoutMsg::ConfirmDiscard),
            Some(LayoutMsg::ConfirmCancel)
        ]
    );
}

#[test]
fn last_layout_undeletable() {
    let found = messages(&strip_entries(&view(0, &["Desk"]), LayoutsUi::default()));
    assert_eq!(found.last(), Some(&None));
}

#[test]
fn armed_names_delete() {
    crate::i18n::with_language("en-US", || {
        let ui = LayoutsUi { armed: true, ..LayoutsUi::default() };
        let entries = strip_entries(&view(0, &["Desk", "Laptop"]), ui);
        let Some(StripEntry::Item { caption, .. }) = entries.last() else {
            unreachable!();
        };
        assert!(caption.contains("Desk"));
    });
}

#[test]
fn unmatched_offers_save_new() {
    let lost = SavedLayouts { active: None, ..view(0, &["Desk", "Laptop"]) };
    crate::i18n::with_language("en-US", || {
        let found = messages(&strip_entries(&lost, LayoutsUi::default()));
        assert!(found.contains(&Some(LayoutMsg::SaveNew)));
        for gone in [LayoutMsg::Rename, LayoutMsg::Duplicate, LayoutMsg::Delete] {
            assert!(!found.contains(&Some(gone)));
        }
    });
}
