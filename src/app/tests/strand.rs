use paper_control::MAX_WIDGETS;
use serde_json::{Value, json};
use skwd_config::keys::widgets::ITEMS;

use super::{Message, drain_calls, test_app, update};
use crate::app::update::strand::{entries, save};
use crate::domain::strand::Command;
use crate::frontend::strand::scenery_tests::scenery;
use crate::frontend::strand::{Effect, Strand};

fn clock(x: f64, output: Option<&str>) -> Value {
    json!({
        "face": {"kind": "clock", "style": "aperture"},
        "depth": "behind",
        "pin": {"anchor": [0.0, 0.0], "offset": [x, 40.0]},
        "height": 0.2,
        "output": output,
    })
}

#[test]
fn done_passes_through() {
    let mut raw = vec![clock(10.0, None), json!({"face": "dial", "note": 1})];
    raw.push(clock(20.0, Some("HDMI-A-1")));
    raw.extend((1..MAX_WIDGETS).map(|step| clock(30.0 + step as f64, Some("DP-1"))));
    raw.push(clock(99.0, Some("DP-1")));
    let mut app = test_app();
    app.config.set_key(ITEMS, Value::Array(raw.clone()));
    let mut found = entries(raw.clone(), Some("DP-1"));
    assert_eq!(found.widgets.len(), MAX_WIDGETS - 1);
    let mut strand = Strand::new([1600.0, 900.0], 1.0, Some("DP-1".to_owned()));
    strand.adopt(std::mem::take(&mut found.widgets), scenery(), None);
    app.panels.strand_entries = found;
    for _ in 0..240 {
        strand.tick(1.0 / 60.0);
    }
    strand.select(strand.snapshot()[0].id);
    strand.command(Command::Nudge([10.0, 0.0]));
    let Effect::Save(items) = strand.done() else { panic!("saved") };
    save(&mut app, items);
    let saved = skwd_config::arr_ref(app.config.root(), ITEMS).to_vec();
    assert_eq!(saved.len(), raw.len());
    assert_eq!(saved[1], raw[1]);
    assert_eq!(saved[2], raw[2]);
    assert_eq!(saved.last(), raw.last());
    assert_ne!(saved[0], raw[0]);
    assert_eq!(saved[3..raw.len() - 1], raw[3..raw.len() - 1]);
}

#[test]
fn room_counts_passthrough() {
    let mut raw = vec![clock(10.0, Some("DP-1")), json!({"face": "dial"})];
    raw.extend((1..MAX_WIDGETS).map(|step| clock(step as f64, Some("HDMI-A-1"))));
    assert_eq!(entries(raw.clone(), Some("DP-1")).room, 1);
    raw.truncate(5);
    assert_eq!(entries(raw, Some("DP-1")).room, MAX_WIDGETS - 3);
}

fn edits(app: &crate::app::App) -> Vec<bool> {
    drain_calls(app)
        .into_iter()
        .filter(|(method, _)| method == wall_proto::rpc::WIDGETS_EDIT)
        .map(|(_, params)| params["active"].as_bool().unwrap())
        .collect()
}

fn loading(app: &mut crate::app::App) {
    app.panels.strand = Some(Strand::new([1600.0, 900.0], 1.0, None));
    drain_calls(app);
}

#[test]
fn exit_while_loading() {
    let mut app = test_app();
    loading(&mut app);
    let _ = update(&mut app, Message::Exit);
    assert!(app.panels.strand.is_none());
    assert_eq!(edits(&app), [false]);
}

#[test]
fn exit_during_entry() {
    let mut app = test_app();
    loading(&mut app);
    if let Some(strand) = app.panels.strand.as_mut() {
        strand.adopt(Vec::new(), scenery(), None);
    }
    let _ = update(&mut app, Message::Exit);
    assert!(app.panels.strand.is_none());
}

#[test]
fn hiding_closes_editor() {
    let mut app = test_app();
    loading(&mut app);
    app.on_hidden();
    assert!(app.panels.strand.is_none());
    assert_eq!(edits(&app), [false]);
}

#[test]
fn reconnect_resends_edit() {
    let mut app = test_app();
    loading(&mut app);
    if let Some(strand) = app.panels.strand.as_mut() {
        strand.adopt(Vec::new(), scenery(), None);
    }
    app.handle_ipc(crate::infrastructure::ipc::IpcMsg::Connected { list_id: 1 });
    assert_eq!(edits(&app), [true]);
}

#[test]
fn entry_hides_picker() {
    let mut app = test_app();
    loading(&mut app);
    assert_eq!(crate::app::view::scene_fade(&app), 1.0);
    let strand = app.panels.strand.as_mut().unwrap();
    strand.adopt(Vec::new(), scenery(), None);
    for _ in 0..240 {
        strand.tick(1.0 / 60.0);
    }
    assert_eq!(crate::app::view::scene_fade(&app), 0.0);
}
