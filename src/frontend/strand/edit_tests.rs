use paper_control::{ClockStyle, Depth, Face, Pin, Widget};

use super::Effect;
use crate::domain::strand::{Command, Item, Saved, Spot};
use crate::frontend::strand::gesture::StrandMsg;
use crate::frontend::strand::scenery_tests::scenery;
use crate::frontend::strand::state::Strand;

const SCREEN: [f32; 2] = [1600.0, 900.0];
const TICK: f32 = 1.0 / 60.0;
const SETTLE: usize = 240;
const APERTURE: Face = Face::Clock(ClockStyle::Aperture);
const GREETER: Face = Face::Clock(ClockStyle::Greeter);

fn widget(face: Face, offset: [f32; 2], output: Option<&str>) -> Widget {
    Widget {
        face,
        depth: Depth::Behind,
        pin: Pin { anchor: [0.0, 0.0], offset },
        height: Some(0.2),
        output: output.map(str::to_owned),
    }
}

fn settle(strand: &mut Strand) {
    for _ in 0..SETTLE {
        strand.tick(TICK);
    }
}

fn editor(widgets: Vec<Widget>) -> Strand {
    named(widgets, Some("DP-1"))
}

fn named(widgets: Vec<Widget>, name: Option<&str>) -> Strand {
    let mut strand = Strand::new(SCREEN, 1.0, name.map(str::to_owned));
    strand.adopt(widgets, scenery(), None);
    settle(&mut strand);
    strand
}

fn first(strand: &Strand) -> u32 {
    strand.snapshot()[0].id
}

#[test]
fn undo_redo_move() {
    let mut strand = editor(vec![widget(APERTURE, [100.0, 100.0], None)]);
    let id = first(&strand);
    let before = strand.snapshot();
    strand.select(id);
    strand.command(Command::Nudge([10.0, 0.0]));
    assert_eq!(strand.cancel_count(), 1);
    settle(&mut strand);
    strand.command(Command::Undo);
    settle(&mut strand);
    assert_eq!(strand.snapshot(), before);
    assert_eq!(strand.cancel_count(), 0);
    strand.command(Command::Redo);
    settle(&mut strand);
    assert_eq!(strand.snapshot()[0].centre[0], before[0].centre[0] + 10.0);
}

#[test]
fn nudges_coalesce() {
    let mut strand = editor(vec![widget(APERTURE, [100.0, 100.0], None)]);
    strand.select(first(&strand));
    strand.command(Command::Nudge([1.0, 0.0]));
    strand.command(Command::Nudge([1.0, 0.0]));
    assert_eq!(strand.cancel_count(), 1);
}

#[test]
fn cancel_rewinds_session() {
    let mut strand = editor(vec![widget(APERTURE, [100.0, 100.0], None)]);
    let before = strand.snapshot();
    let id = first(&strand);
    strand.select(id);
    strand.command(Command::Depth);
    strand.command(Command::Remove);
    settle(&mut strand);
    assert!(strand.snapshot().is_empty());
    strand.command(Command::Cancel);
    assert_eq!(strand.cancel_count(), 0);
    settle(&mut strand);
    let after = strand.snapshot();
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].depth, before[0].depth);
    assert!(strand.finished());
}

#[test]
fn escape_deselects_first() {
    let mut strand = editor(vec![widget(APERTURE, [100.0, 100.0], None)]);
    strand.select(first(&strand));
    strand.command(Command::Deselect);
    assert!(strand.selection.id.is_none());
    assert!(!strand.spool.leaving);
}

#[test]
fn done_keeps_untouched() {
    let mut strand =
        editor(vec![widget(APERTURE, [100.0, 100.0], None), widget(GREETER, [900.0, 100.0], None)]);
    let moved = strand.snapshot()[1];
    strand.select(moved.id);
    strand.command(Command::Nudge([10.0, 0.0]));
    let Effect::Save(items) = strand.done() else { panic!("saved") };
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], Saved::Kept(0));
    let Saved::Placed(placed) = &items[1] else { panic!("placed") };
    let spot = Spot::placed(0, placed, SCREEN, 1.0).unwrap();
    assert!((spot.centre[0] - moved.centre[0] - 10.0).abs() < 1e-3);
    assert!(placed.output.is_none());
}

#[test]
fn held_keeps_order() {
    let mut strand = editor(vec![
        widget(APERTURE, [100.0, 100.0], None),
        widget(GREETER, [900.0, 100.0], None),
        widget(Face::Clock(ClockStyle::DayDial), [500.0, 500.0], None),
    ]);
    let middle = strand.snapshot()[1].id;
    strand.widgets.retain(|live| live.id != middle);
    strand.sources.remove(&middle);
    strand.held.push(1);
    let Effect::Save(items) = strand.done() else { panic!("saved") };
    assert_eq!(items, [Saved::Kept(0), Saved::Kept(1), Saved::Kept(2)]);
}

fn added(name: Option<&str>) -> Widget {
    let mut strand = named(Vec::new(), name);
    strand.update(StrandMsg::Move([0.0, 0.0]));
    let x = strand.spool.item_x(strand.now, Item::Bead(0, 0));
    let y = strand.spool.thread_at(x);
    strand.update(StrandMsg::Press([x, y]));
    strand.update(StrandMsg::Release([x, y]));
    assert_eq!(strand.cancel_count(), 1);
    let Effect::Save(mut items) = strand.done() else { panic!("saved") };
    assert_eq!(items.len(), 1);
    let Some(Saved::Placed(widget)) = items.pop() else { panic!("placed") };
    widget
}

#[test]
fn full_refuses_add() {
    let mut strand = editor(vec![widget(GREETER, [900.0, 100.0], None)]);
    strand.room = 1;
    strand.update(StrandMsg::Move([0.0, 0.0]));
    let x = strand.spool.item_x(strand.now, Item::Bead(0, 0));
    let y = strand.spool.thread_at(x);
    strand.update(StrandMsg::Press([x, y]));
    strand.update(StrandMsg::Move([x, y - 200.0]));
    strand.update(StrandMsg::Release([x, y - 200.0]));
    assert_eq!(strand.snapshot().len(), 1);
    assert_eq!(strand.cancel_count(), 0);
}

#[test]
fn adds_with_output() {
    let widget = added(Some("DP-1"));
    assert_eq!(widget.face, APERTURE);
    assert_eq!(widget.output.as_deref(), Some("DP-1"));
}

#[test]
fn unknown_output_unset() {
    assert_eq!(added(None).output, None);
}

#[test]
fn fractional_round_trip() {
    let starts = [([100.0, 100.0], 0.0), ([640.0, 360.0], 0.5), ([1300.0, 700.0], 1.0)];
    for (offset, anchor) in starts {
        let mut strand = Strand::new(SCREEN, 1.25, Some("DP-1".to_owned()));
        strand.adopt(vec![widget(APERTURE, offset, None)], scenery(), None);
        settle(&mut strand);
        let moved = strand.snapshot()[0];
        strand.select(moved.id);
        strand.command(Command::Nudge([7.0, 3.0]));
        settle(&mut strand);
        let after = strand.snapshot()[0];
        let Effect::Save(items) = strand.done() else { panic!("saved") };
        let Saved::Placed(placed) = &items[0] else { panic!("placed") };
        assert_eq!(placed.pin.anchor, [anchor; 2]);
        let back = Spot::placed(0, placed, SCREEN, 1.25).unwrap();
        let again = Spot::placed(0, &back.widget(SCREEN, 1.25, None), SCREEN, 1.25).unwrap();
        assert_eq!((again.centre, again.height), (back.centre, back.height));
        for axis in 0..2 {
            assert!((back.centre[axis] - after.centre[axis]).abs() <= 2.0, "anchor {anchor}");
        }
    }
}

#[test]
fn idle_after_settle() {
    let mut strand = editor(vec![widget(APERTURE, [100.0, 100.0], None)]);
    settle(&mut strand);
    assert!(!strand.animating());
}
