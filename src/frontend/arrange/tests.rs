use paper_control::Palette;

use super::model::{
    Anchor, ArrangeMsg, Bead, Board, Button, CornerStep, Effect, Family, Grid, Pointer, Shelf,
    Step, Style, Target,
};
use super::state::{Arrange, Hit};
use crate::app::{
    ENTRY_HEIGHT, ENTRY_PAD, SWATCH_CELL, SWATCH_STACK_GAP, rest_bottom, swatch_height,
};
use crate::domain::strand::{Command, Item, Rect, zone_top};
use crate::frontend::strand::raster::ink;
use crate::frontend::strand::tokens::{FRAME, TIMING};

const OUTPUT: [f32; 2] = [1600.0, 900.0];
const TICK: f32 = 1.0 / 60.0;
const SETTLE: usize = 180;
const PICKER: Rect = Rect::new(160.0, 180.0, 1280.0, 540.0);
const BAR: Rect = Rect::new(500.0, 60.0, 600.0, 40.0);
const SEARCH: Rect = Rect::new(1300.0, 60.0, 200.0, 40.0);
const ENTRY_WIDTH: f32 = 115.0;
const STRIP_CELLS: f32 = 6.0;
const LOWEST: [f32; 2] = [1366.0, 768.0];
const ENTRY: Rect = floats(OUTPUT)[0];
const STRIP: Rect = floats(OUTPUT)[1];

const fn floats(output: [f32; 2]) -> [Rect; 2] {
    let entry =
        Rect::new(ENTRY_PAD, rest_bottom(output[1]) - ENTRY_HEIGHT, ENTRY_WIDTH, ENTRY_HEIGHT);
    let strip = Rect::new(
        ENTRY_PAD,
        entry.y - SWATCH_STACK_GAP - swatch_height(1.0),
        STRIP_CELLS * SWATCH_CELL,
        swatch_height(1.0),
    );
    [entry, strip]
}

fn tones() -> paper_widgets::Tones {
    ink(&Palette { text: [230, 230, 230], accent: [170, 190, 255], shade: [20, 22, 30] }, 0.05)
}

fn bead(id: &str, shown: bool) -> Bead {
    Bead { id: id.to_owned(), title: id.to_owned(), shown }
}

fn board() -> Board {
    Board {
        rects: [PICKER, BAR, SEARCH, ENTRY, STRIP],
        attached: [false; Target::ALL.len()],
        present: [true; Target::ALL.len()],
        buttons: vec![
            Button { id: "type:static".to_owned(), rect: Rect::new(510.0, 64.0, 60.0, 32.0) },
            Button { id: "random".to_owned(), rect: Rect::new(580.0, 64.0, 60.0, 32.0) },
        ],
        shelves: vec![
            Shelf {
                family: Family::Types,
                beads: vec![bead("type:static", true), bead("type:video", false)],
            },
            Shelf {
                family: Family::Tools,
                beads: vec![
                    bead("random", true),
                    bead("theme", false),
                    bead("arrange", true),
                    bead("swatch", true),
                ],
            },
        ],
        style: Style { layout: "Slices", spacing: Some(1), corners: Some(CornerStep::Step(1)) },
        grid: None,
        resizable: true,
        steps: 0,
        displays: false,
        copy: None,
        prompt: None,
    }
}

fn settle(arrange: &mut Arrange) {
    for _ in 0..SETTLE {
        arrange.tick(TICK);
    }
}

fn editor() -> Arrange {
    let mut arrange = Arrange::new(OUTPUT, board(), tones());
    settle(&mut arrange);
    arrange
}

fn click(arrange: &mut Arrange, at: [f32; 2]) -> Vec<Effect> {
    let mut effects = arrange.update(Pointer::Press(at));
    effects.extend(arrange.update(Pointer::Release(at)));
    effects
}

fn drag(arrange: &mut Arrange, from: [f32; 2], to: [f32; 2]) -> Vec<Effect> {
    let mut effects = arrange.update(Pointer::Press(from));
    let steps = 8;
    for step in 1..=steps {
        let share = step as f32 / steps as f32;
        let at = [from[0] + (to[0] - from[0]) * share, from[1] + (to[1] - from[1]) * share];
        effects.extend(arrange.update(Pointer::Move(at)));
        arrange.tick(TICK);
    }
    effects.extend(arrange.update(Pointer::Release(to)));
    effects
}

fn bead_at(arrange: &Arrange, bead: (usize, usize)) -> [f32; 2] {
    arrange.spool.slot(arrange.now, bead)
}

#[test]
fn entry_gates_input() {
    let mut arrange = Arrange::new(OUTPUT, board(), tones());
    assert!(arrange.entering());
    assert!(arrange.hit(PICKER.centre()).is_none());
    settle(&mut arrange);
    assert!(arrange.hits_ready());
    assert_eq!(arrange.hit(PICKER.centre()), Some(Hit::Object(Target::Picker)));
}

#[test]
fn click_selects_and_unfurls() {
    let mut arrange = editor();
    assert!(click(&mut arrange, PICKER.centre()).is_empty());
    assert_eq!(arrange.selection.id, Some(Target::Picker));
    settle(&mut arrange);
    let frame = arrange.selection.frame_rect().expect("frame");
    assert_eq!(frame, PICKER.padded(FRAME.pad));
    let bar = arrange.bar(Target::Picker, frame);
    assert!(bar.plates.iter().all(|plate| arrange.selection.opening(arrange.now, plate) > 0.99));
    assert!(bar.plates.last().is_some_and(|plate| plate.rect.bottom() <= frame.y));
}

#[test]
fn drag_snaps_to_centre() {
    let mut arrange = editor();
    let from = PICKER.centre();
    let effects = drag(&mut arrange, from, [from[0] + 7.0, from[1] + 120.0]);
    let placed = effects.iter().rev().find_map(|effect| match effect {
        Effect::Place(Target::Picker, rect) => Some(*rect),
        _ => None,
    });
    assert_eq!(placed.map(|rect| rect.centre()[0]), Some(800.0));
    let committed = effects.last();
    assert!(matches!(
        committed,
        Some(Effect::Commit(Target::Picker, Anchor::Screen([1, _]), Step::Move))
    ));
}

#[test]
fn bar_snapped_to_picker_attaches() {
    let mut arrange = editor();
    let from = [BAR.x + 2.0, BAR.centre()[1]];
    let to = [from[0] + 3.0, PICKER.y - BAR.h / 2.0 - 4.0];
    let effects = drag(&mut arrange, from, to);
    assert!(matches!(
        effects.last(),
        Some(Effect::Commit(Target::Bar, Anchor::Picker(_), Step::Move))
    ));
}

#[test]
fn gem_resize_holds_anchor() {
    let mut arrange = editor();
    click(&mut arrange, PICKER.centre());
    settle(&mut arrange);
    let frame = arrange.selection.frame_rect().expect("frame");
    let gem = [frame.right(), frame.bottom()];
    assert_eq!(arrange.hit(gem), Some(Hit::Gem(2)));
    let effects = drag(&mut arrange, gem, [gem[0] - 237.0, gem[1] - 141.0]);
    let placed = effects.iter().rev().find_map(|effect| match effect {
        Effect::Place(Target::Picker, rect) => Some(*rect),
        _ => None,
    });
    let rect = placed.expect("resized");
    assert_eq!([rect.x, rect.y], [PICKER.x, PICKER.y]);
    assert!(rect.w < PICKER.w && rect.h < PICKER.h);
    assert!(matches!(effects.last(), Some(Effect::Commit(Target::Picker, _, Step::Size))));
}

#[test]
fn fixed_picker_keeps_size() {
    let mut start = board();
    start.resizable = false;
    let mut arrange = Arrange::new(OUTPUT, start, tones());
    settle(&mut arrange);
    click(&mut arrange, PICKER.centre());
    settle(&mut arrange);
    let frame = arrange.selection.frame_rect().expect("frame");
    assert_ne!(arrange.hit([frame.right(), frame.bottom()]), Some(Hit::Gem(2)));
    assert!(arrange.command(Command::Detent(true)).is_empty());
    assert!(arrange.shake.is_some());
}

#[test]
fn grid_resize_counts_cells() {
    let mut start = board();
    start.grid = Some(Grid { cell: [180.0, 120.0], gap: [20.0, 20.0], cells: [6, 4] });
    let mut arrange = Arrange::new(OUTPUT, start, tones());
    settle(&mut arrange);
    click(&mut arrange, PICKER.centre());
    settle(&mut arrange);
    let frame = arrange.selection.frame_rect().expect("frame");
    let gem = [frame.right(), frame.bottom()];
    let effects = drag(&mut arrange, gem, [PICKER.x + 595.0, PICKER.y + 265.0]);
    let grid = effects.iter().rev().find_map(|effect| match effect {
        Effect::Grid(rect, cells) => Some((*rect, *cells)),
        _ => None,
    });
    let (rect, cells) = grid.expect("grid resize");
    assert_eq!(cells, [3, 2]);
    assert_eq!([rect.w, rect.h], [580.0, 260.0]);
    assert!(matches!(effects.last(), Some(Effect::Commit(Target::Picker, _, Step::Grid))));
}

#[test]
fn button_on_thread_hides() {
    let mut arrange = editor();
    click(&mut arrange, BAR.centre());
    settle(&mut arrange);
    let button = arrange.board.buttons[1].rect.centre();
    assert_eq!(arrange.hit(button), Some(Hit::Button(1)));
    let below = [button[0], zone_top(OUTPUT[1]) + 40.0];
    let effects = drag(&mut arrange, button, below);
    assert!(matches!(effects.first(), Some(Effect::Reorder(id, _)) if id == "random"));
    assert_eq!(effects.last(), Some(&Effect::Hide("random".to_owned())));
    assert!(arrange.spool.toast.is_some());
    assert_eq!(arrange.spool.travels.len(), 1);
}

#[test]
fn hidden_bead_click_shows() {
    let mut arrange = editor();
    let hidden = bead_at(&arrange, (1, 1));
    let hovered = arrange.hit(hidden);
    if hovered == Some(Hit::Item(Item::Group(1))) {
        arrange.update(Pointer::Move(hidden));
        settle(&mut arrange);
    }
    let hidden = bead_at(&arrange, (1, 1));
    let effects = click(&mut arrange, hidden);
    assert_eq!(effects, vec![Effect::Show("theme".to_owned(), None)]);
    assert_eq!(arrange.selection.id, Some(Target::Bar));
    let mut shown = board();
    shown.shelves[1].beads[1].shown = true;
    shown.buttons.push(Button { id: "theme".to_owned(), rect: Rect::new(650.0, 64.0, 60.0, 32.0) });
    arrange.sync(shown, tones());
    assert_eq!(arrange.spool.threads.len(), 1);
}

#[test]
fn pulled_bead_shows_where_dropped() {
    let mut arrange = editor();
    let hidden = bead_at(&arrange, (0, 1));
    let effects = drag(&mut arrange, hidden, [700.0, 80.0]);
    assert!(matches!(effects.first(), Some(Effect::Show(id, Some(_))) if id == "type:video"));
    assert!(
        effects.iter().any(|effect| matches!(effect, Effect::Reorder(id, _) if id == "type:video"))
    );
    assert_eq!(effects.last(), Some(&Effect::Drop));
}

#[test]
fn escape_deselects_then_cancels() {
    let mut arrange = editor();
    click(&mut arrange, SEARCH.centre());
    assert!(arrange.command(Command::Deselect).is_empty());
    assert!(arrange.selection.id.is_none());
    assert_eq!(arrange.command(Command::Cancel), vec![Effect::Message(ArrangeMsg::Cancel)]);
    assert!(arrange.reverting);
    for _ in 0..((TIMING.revert_hold + 1.0) / TICK) as usize {
        arrange.tick(TICK);
    }
    assert!(arrange.finished());
}

#[test]
fn undo_selects_restored_object() {
    let mut arrange = editor();
    assert_eq!(arrange.command(Command::Undo), vec![Effect::Message(ArrangeMsg::Undo)]);
    let mut restored = board();
    restored.rects[1] = Rect::new(400.0, 60.0, 600.0, 40.0);
    arrange.sync(restored, tones());
    assert_eq!(arrange.selection.id, Some(Target::Bar));
    settle(&mut arrange);
    assert_eq!(arrange.rect(Target::Bar).x, 400.0);
}

#[test]
fn nudges_coalesce() {
    let mut arrange = editor();
    click(&mut arrange, SEARCH.centre());
    let first = arrange.command(Command::Nudge([1.0, 0.0]));
    assert!(matches!(first.as_slice(), [Effect::Nudge(Target::Search, _, _, false)]));
    let mut moved = board();
    moved.steps = 1;
    moved.rects[2].x += 1.0;
    arrange.sync(moved, tones());
    let second = arrange.command(Command::Nudge([1.0, 0.0]));
    assert!(
        matches!(second.as_slice(), [Effect::Nudge(Target::Search, rect, _, true)] if rect.x == SEARCH.x + 2.0)
    );
}

#[test]
fn preview_hides_chrome() {
    let mut arrange = editor();
    let extra = arrange
        .spool
        .laid
        .as_ref()
        .and_then(|laid| laid.plan.items.iter().find(|placed| placed.item == Item::Extra(0)))
        .map(|placed| [placed.x + 4.0, arrange.spool.thread_at(placed.x)])
        .expect("preview plate");
    assert_eq!(click(&mut arrange, extra), vec![Effect::Message(ArrangeMsg::Preview)]);
    settle(&mut arrange);
    assert!(arrange.previewing());
    assert!(arrange.hit(PICKER.centre()).is_none());
    let back = arrange.back_rect().centre();
    assert_eq!(click(&mut arrange, back), vec![Effect::Message(ArrangeMsg::Preview)]);
    assert!(!arrange.previewing());
}

#[test]
fn idle_after_settle() {
    let mut arrange = editor();
    click(&mut arrange, PICKER.centre());
    settle(&mut arrange);
    assert!(!arrange.animating());
    assert!(arrange.shown(Target::Picker).is_none());
}

#[test]
fn object_drag_keeps_thread_cached() {
    let mut arrange = editor();
    assert!(arrange.thread_still());
    let from = PICKER.centre();
    arrange.update(Pointer::Press(from));
    for step in 1..=10 {
        arrange.update(Pointer::Move([from[0] - 12.0 * step as f32, from[1]]));
        arrange.tick(TICK);
        assert!(arrange.thread_still());
    }
    arrange.update(Pointer::Release([from[0] - 120.0, from[1]]));
    click(&mut arrange, BAR.centre());
    settle(&mut arrange);
    let button = arrange.board.buttons[1].rect.centre();
    arrange.update(Pointer::Press(button));
    arrange.update(Pointer::Move([button[0], zone_top(OUTPUT[1]) + 40.0]));
    arrange.tick(TICK);
    assert!(!arrange.thread_still());
}

#[test]
fn plate_row_clears_bar() {
    let mut start = board();
    start.rects[0] = Rect::new(160.0, 130.0, 1280.0, 540.0);
    let mut arrange = Arrange::new(OUTPUT, start, tones());
    settle(&mut arrange);
    click(&mut arrange, [800.0, 400.0]);
    settle(&mut arrange);
    let frame = arrange.selection.frame_rect().expect("frame");
    let bar = arrange.bar(Target::Picker, frame);
    assert!(!bar.overlaps(&[BAR, SEARCH]));
    assert!(bar.below);
}

#[test]
fn hide_toast_survives_tick_before_record() {
    let mut arrange = editor();
    click(&mut arrange, BAR.centre());
    settle(&mut arrange);
    let button = arrange.board.buttons[1].rect.centre();
    let below = [button[0], zone_top(OUTPUT[1]) + 40.0];
    arrange.update(Pointer::Press(button));
    arrange.update(Pointer::Move(below));
    let effects = arrange.update(Pointer::Release(below));
    assert_eq!(effects, vec![Effect::Hide("random".to_owned())]);
    arrange.tick(TICK);
    arrange.recorded();
    for _ in 0..20 {
        arrange.tick(TICK);
    }
    let toast = arrange.spool.toast.as_ref().expect("toast");
    assert!(toast.leave > arrange.now);
    arrange.recorded();
    assert!(arrange.spool.toast.as_ref().is_some_and(|toast| toast.leave <= arrange.now));
}

fn hidden_board() -> Board {
    let mut hidden = board();
    hidden.present[Target::Arrange.index()] = false;
    hidden.shelves[1].beads[2].shown = false;
    hidden
}

fn tab(arrange: &mut Arrange) -> Option<Target> {
    arrange.command(Command::Tab(true));
    arrange.selection.id
}

#[test]
fn floating_plate_drags() {
    let mut arrange = editor();
    let from = [ENTRY.x + 4.0, ENTRY.centre()[1]];
    assert_eq!(arrange.hit(from), Some(Hit::Object(Target::Arrange)));
    let effects = drag(&mut arrange, from, [700.0, 300.0]);
    assert!(effects.iter().any(|effect| matches!(effect, Effect::Place(Target::Arrange, _))));
    assert!(matches!(
        effects.last(),
        Some(Effect::Commit(Target::Arrange, Anchor::Screen(_), Step::Move))
    ));
}

#[test]
fn drop_near_thread_keeps() {
    let mut arrange = editor();
    let from = [STRIP.right() - 4.0, STRIP.centre()[1]];
    assert_eq!(arrange.hit(from), Some(Hit::Object(Target::Swatch)));
    let effects = drag(&mut arrange, from, [from[0] + 200.0, from[1] + 10.0]);
    assert!(matches!(effects.last(), Some(Effect::Commit(Target::Swatch, _, Step::Move))));
}

#[test]
fn hidden_plate_absent() {
    let mut arrange = Arrange::new(OUTPUT, hidden_board(), tones());
    settle(&mut arrange);
    assert_ne!(arrange.hit(ENTRY.centre()), Some(Hit::Object(Target::Arrange)));
    let expected = [Target::Picker, Target::Bar, Target::Search, Target::Swatch, Target::Picker];
    let order: Vec<_> = expected.iter().map(|_| tab(&mut arrange)).collect();
    assert_eq!(order, expected.map(Some));
}

#[test]
fn hide_plate_toolbar() {
    let mut arrange = editor();
    click(&mut arrange, [ENTRY.x + 4.0, ENTRY.centre()[1]]);
    settle(&mut arrange);
    assert_eq!(arrange.selection.id, Some(Target::Arrange));
    let plates = arrange.plates();
    let hide = plates.iter().find(|(id, _)| id == "tool:Hide").expect("hide plate").1;
    assert!(plates.iter().any(|(id, _)| id == "tool:Reset"));
    assert_eq!(click(&mut arrange, hide.centre()), vec![Effect::Hide("arrange".to_owned())]);
    assert!(arrange.selection.id.is_none());
    assert!(arrange.spool.toast.is_some());
}

#[test]
fn bead_shows_plate() {
    let mut arrange = Arrange::new(OUTPUT, hidden_board(), tones());
    settle(&mut arrange);
    arrange.update(Pointer::Move(bead_at(&arrange, (1, 2))));
    settle(&mut arrange);
    let spot = bead_at(&arrange, (1, 2));
    assert_eq!(click(&mut arrange, spot), vec![Effect::Show("arrange".to_owned(), None)]);
    arrange.sync(board(), tones());
    assert_eq!(arrange.selection.id, Some(Target::Arrange));
    assert_eq!(arrange.spool.threads.len(), 1);
    assert!(arrange.shoot.is_none());
}

#[test]
fn hidden_selection_drops() {
    let mut arrange = editor();
    click(&mut arrange, [ENTRY.x + 4.0, ENTRY.centre()[1]]);
    arrange.sync(hidden_board(), tones());
    assert!(arrange.selection.id.is_none());
}

#[test]
fn thread_fits_lowest() {
    let mut start = board();
    let shelf = |family, count: usize| Shelf {
        family,
        beads: (0..count).map(|at| bead(&format!("{family:?}{at}"), true)).collect(),
    };
    start.shelves = vec![
        shelf(Family::Types, 4),
        shelf(Family::Sort, 7),
        shelf(Family::Colours, 13),
        shelf(Family::Tools, 12),
    ];
    start.rects[Target::Picker.index()] = Rect::new(100.0, 100.0, 1166.0, 400.0);
    start.rects[Target::Search.index()] = Rect::new(1000.0, 20.0, 200.0, 40.0);
    let [entry, strip] = floats(LOWEST);
    start.rects[Target::Arrange.index()] = entry;
    start.rects[Target::Swatch.index()] = strip;
    let plan = |board: Board| {
        let mut arrange = Arrange::new(LOWEST, board, tones());
        settle(&mut arrange);
        arrange.spool.laid.as_ref().expect("laid").plan.clone()
    };
    let mut clear = start.clone();
    clear.present[Target::Arrange.index()] = false;
    clear.present[Target::Swatch.index()] = false;
    let laid = plan(start);
    assert!(laid.fits);
    assert_eq!(laid, plan(clear));
}

#[test]
fn corners_each_plate() {
    let style = Style { layout: "Slices", spacing: None, corners: Some(CornerStep::Each) };
    let label = crate::i18n::with_language("en-US", || {
        super::labels::tool(super::model::Tool::Corners, &style)
    });
    assert_eq!(label, "Corners · Each");
}
