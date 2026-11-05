use super::{EditHistory, EditSnapshot, MAX_STATES};
use serde_json::json;

fn snapshot(value: Option<serde_json::Value>) -> EditSnapshot {
    EditSnapshot { values: vec![("path".into(), value)] }
}

fn history() -> EditHistory {
    EditHistory::new(snapshot(None), "Original".into())
}

#[test]
fn navigates_edits() {
    let mut history = history();
    let first = snapshot(Some(json!(1)));
    let second = snapshot(Some(json!(2)));
    assert!(history.record(first.clone(), "First".into()));
    assert!(history.record(second.clone(), "Second".into()));
    assert_eq!(history.undo(), Some(&first));
    assert_eq!(history.undo(), Some(&snapshot(None)));
    assert_eq!(history.undo(), None);
    assert_eq!(history.redo(), Some(&first));
    assert_eq!(history.redo(), Some(&second));
    assert_eq!(history.redo(), None);
    assert_eq!(history.index(), 2);
}

#[test]
fn branches_on_commit() {
    let mut history = history();
    let first = snapshot(Some(json!(1)));
    let second = snapshot(Some(json!(2)));
    history.record(first.clone(), "First".into());
    history.record(second, "Second".into());
    history.undo();
    assert!(!history.record(first, "Same".into()));
    assert!(history.redo().is_some());
    history.undo();
    let branch = snapshot(Some(json!(3)));
    assert!(history.record(branch.clone(), "Branch".into()));
    assert_eq!(history.redo(), None);
    assert_eq!(history.entries().len(), 3);
    assert_eq!(history.current(), &branch);
    assert_eq!(history.entries()[2].label, "Branch");
}

#[test]
fn preserves_absence() {
    let mut history = history();
    let null = snapshot(Some(serde_json::Value::Null));
    assert!(history.record(null.clone(), "Null".into()));
    assert_eq!(history.undo(), Some(&snapshot(None)));
    assert_eq!(history.redo(), Some(&null));
    assert_eq!(history.original(), &snapshot(None));
}

#[test]
fn caps_later_edits() {
    let mut history = history();
    for value in 0..MAX_STATES {
        history.record(snapshot(Some(json!(value))), value.to_string());
    }
    assert_eq!(history.entries().len(), MAX_STATES);
    assert_eq!(history.entries()[0].snapshot, snapshot(None));
    assert_eq!(history.entries()[1].snapshot, snapshot(Some(json!(1))));
    assert_eq!(history.index(), MAX_STATES - 1);
}

#[test]
fn amends_latest_edit() {
    let mut history = history();
    history.record(snapshot(Some(json!(1))), "First".into());
    history.record(snapshot(Some(json!(2))), "Second".into());
    history.undo();
    history.amend(snapshot(Some(json!(3))));
    assert_eq!(history.entries().len(), 2);
    assert_eq!(history.current(), &snapshot(Some(json!(3))));
    assert_eq!(history.redo(), None);
}
