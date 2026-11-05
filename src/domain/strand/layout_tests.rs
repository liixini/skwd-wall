use super::{Item, KindSpec, MARGIN, StrandSpec, obstacles, plan, shut};
use crate::domain::strand::Rect;

const CLOCK: KindSpec = KindSpec { plate: 68.0, group: 42.0, styles: 5 };
const OTHERS: [KindSpec; 4] = [
    CLOCK,
    KindSpec { plate: 84.0, group: 42.0, styles: 2 },
    KindSpec { plate: 109.0, group: 42.0, styles: 2 },
    KindSpec { plate: 91.0, group: 42.0, styles: 2 },
];

fn spec<'a>(width: f32, kinds: &'a [KindSpec], obstacles: &'a [[f32; 2]]) -> StrandSpec<'a> {
    StrandSpec {
        width,
        kinds,
        cancel: 76.0,
        done: 61.0,
        extras: &[],
        open: None,
        obstacles,
        margin: MARGIN,
        dock: &[],
    }
}

fn x_of(items: &[super::Placed], item: Item) -> f32 {
    items.iter().find(|placed| placed.item == item).expect("item").x
}

#[test]
fn design_positions() {
    let laid = plan(&spec(1600.0, &OTHERS, &[]));
    assert!(laid.fits);
    assert!(laid.folded.iter().all(|folded| !folded));
    assert_eq!(x_of(&laid.items, Item::Kind(0)), 310.5);
    assert_eq!(x_of(&laid.items, Item::Bead(0, 4)), 530.5);
    assert_eq!(x_of(&laid.items, Item::Cancel), 1146.5);
    assert_eq!(x_of(&laid.items, Item::Done), 1228.5);
}

#[test]
fn folds_on_narrow() {
    let laid = plan(&spec(897.0, &OTHERS, &[]));
    assert_eq!(laid.folded, vec![true, true, false, false]);
    let wide = plan(&spec(957.0, &OTHERS, &[]));
    assert_eq!(wide.folded, vec![true, false, false, false]);
}

#[test]
fn open_kind_stays() {
    let narrow = StrandSpec { open: Some(0), ..spec(897.0, &OTHERS, &[]) };
    let laid = plan(&narrow);
    assert!(!laid.folded[0]);
}

#[test]
fn dodges_obstacles() {
    let spans = [[300.0, 420.0]];
    let laid = plan(&spec(1600.0, &[CLOCK], &spans));
    for placed in laid.items.iter().filter(|placed| !placed.off) {
        let (left, right) = (placed.x + placed.reach[0], placed.x + placed.reach[1]);
        assert!(right <= 300.0 - 14.0 || left >= 420.0 + 14.0, "{:?}", placed.item);
    }
}

#[test]
fn merges_band_widgets() {
    let boxes = [
        Rect::new(100.0, 800.0, 100.0, 80.0),
        Rect::new(150.0, 820.0, 200.0, 50.0),
        Rect::new(600.0, 100.0, 100.0, 100.0),
    ];
    assert_eq!(obstacles(boxes.into_iter(), 900.0), vec![[100.0, 350.0]]);
}

#[test]
fn extras_precede_cancel() {
    let extras = [90.0, 70.0];
    let laid = plan(&StrandSpec { extras: &extras, ..spec(1600.0, &[CLOCK], &[]) });
    let first = x_of(&laid.items, Item::Extra(0));
    assert_eq!(x_of(&laid.items, Item::Extra(1)), first + 96.0);
    assert_eq!(x_of(&laid.items, Item::Cancel), first + 172.0);
}

#[test]
fn shut_keeps_only_open() {
    let laid = shut(&StrandSpec { open: Some(1), ..spec(1600.0, &OTHERS, &[]) });
    assert_eq!(laid.folded, vec![true, false, true, true]);
    assert!(laid.items.iter().any(|placed| placed.item == Item::Bead(1, 1) && !placed.off));
    assert!(laid.items.iter().all(|placed| !matches!(placed.item, Item::Bead(0, _)) || placed.off));
}

#[test]
fn dock_lifts_commands() {
    let extras = [90.0];
    let dock = [[10.0, 20.0], [110.0, 20.0], [200.0, 50.0]];
    let laid = plan(&StrandSpec { extras: &extras, dock: &dock, ..spec(1600.0, &[CLOCK], &[]) });
    let done = laid.items.iter().find(|placed| placed.item == Item::Done).expect("done");
    assert_eq!((done.x, done.top), (200.0, Some(50.0)));
    assert_eq!(laid.items.iter().filter(|placed| placed.top.is_some()).count(), 3);
    assert_eq!(x_of(&laid.items, Item::Kind(0)), (1600.0 - 232.0) / 2.0);
}

#[test]
fn group_stays_with_kind() {
    let spans = [[150.0, 900.0]];
    let laid = plan(&spec(1600.0, &[KindSpec { plate: 60.0, group: 60.0, styles: 2 }], &spans));
    let plate = x_of(&laid.items, Item::Kind(0));
    let group = x_of(&laid.items, Item::Group(0));
    assert!(group - plate < 100.0, "{plate} {group}");
}
