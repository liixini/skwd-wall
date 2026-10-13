use super::{ARRAY_ACTIONS, choice_enabled, initial_choice, next_choice, remove_index};
use crate::frontend::settings::{ActionId, Control};

fn variant_name(id: &ActionId) -> String {
    format!("{id:?}").split('(').next().unwrap_or_default().to_string()
}

#[test]
fn array_actions_cover_ids() {
    let mut covered: Vec<String> = ARRAY_ACTIONS
        .iter()
        .flat_map(|row| [variant_name(&row.add), variant_name(&(row.remove)(0))])
        .collect();
    covered.sort_unstable();
    let mut declared: Vec<String> = include_str!("../../../frontend/settings/action.rs")
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("Add") || line.starts_with("Remove"))
        .map(|line| line.split(['(', ',']).next().unwrap_or_default().to_string())
        .filter(|name| !matches!(name.as_str(), "AddFolder" | "RemoveFolder"))
        .collect();
    declared.sort_unstable();
    assert_eq!(declared, covered);
}

#[test]
fn array_rows_distinct_keys() {
    let mut keys: Vec<&str> = ARRAY_ACTIONS.iter().map(|row| row.key).collect();
    keys.sort_unstable();
    let unique = keys.len();
    keys.dedup();
    assert_eq!(keys.len(), unique);
    for row in ARRAY_ACTIONS {
        assert_eq!(remove_index(&(row.remove)(9)), Some(9));
        assert_eq!(remove_index(&row.add), None);
        assert!(!row.add.is_destructive(), "{:?}", row.add);
    }
}

#[test]
fn destructive_actions_arm() {
    for id in [ActionId::ClearCache, ActionId::OptimizeImages] {
        assert!(id.is_destructive(), "{id:?}");
    }
    for id in [ActionId::RecomputeColors, ActionId::RefreshBackdrop, ActionId::ResetMotionSlow] {
        assert!(!id.is_destructive(), "{id:?}");
    }
}

#[test]
fn disabled_choices_are_skipped_by_keyboard_navigation() {
    let control = Control::Chips {
        path: String::from("paper.videoEngine"),
        options: vec![
            (String::from("vulkan"), String::from("Vulkan")),
            (String::from("tinier"), String::from("Tinier")),
        ],
        current: String::from("tinier"),
        disabled: vec![String::from("tinier")],
        background: None,
    };

    assert_eq!(initial_choice(&control), 0);
    assert_eq!(next_choice(&control, 0, true), Some(0));
    assert_eq!(next_choice(&control, 0, false), Some(0));
    assert!(choice_enabled(&control, 0));
    assert!(!choice_enabled(&control, 1));
}
