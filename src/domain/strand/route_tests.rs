use super::{MARGIN, Route, RouteSpec, route};
use crate::domain::strand::{KindSpec, Rect, StrandSpec, plan, thread_y};

const LOWEST: [f32; 2] = [1366.0, 768.0];
const KINDS: [KindSpec; 4] = [
    KindSpec { plate: 67.0, group: 42.0, styles: 4 },
    KindSpec { plate: 61.0, group: 42.0, styles: 7 },
    KindSpec { plate: 83.0, group: 53.0, styles: 13 },
    KindSpec { plate: 68.0, group: 53.0, styles: 10 },
];
const EXTRAS: [f32; 2] = [81.0, 86.0];
const PICKER: Rect = Rect::new(136.6, 153.6, 1092.8, 460.8);
const BAR: Rect = Rect::new(148.3, 82.0, 1069.4, 24.0);
const SEARCH: Rect = Rect::new(303.0, 626.0, 760.0, 126.0);
const ENTRY: Rect = Rect::new(18.0, 620.4, 108.0, 35.6);
const STRIP: Rect = Rect::new(18.0, 575.4, 132.0, 39.0);
const PLATE: f32 = 26.0;

fn spec(output: [f32; 2], boxes: &[Rect]) -> RouteSpec<'_> {
    RouteSpec {
        output,
        kinds: &KINDS,
        cancel: 76.0,
        done: 61.0,
        extras: &EXTRAS,
        boxes,
        crown: 38.0,
        pad: 12.0,
        gem: 9.0,
        edge: 8.0,
    }
}

fn plates(found: &Route) -> Vec<Rect> {
    let widths = [81.0, 86.0, 76.0, 61.0];
    found.dock.iter().zip(widths).map(|(spot, w)| Rect::new(spot[0], spot[1], w, PLATE)).collect()
}

#[test]
fn free_row_stays() {
    let found = route(&spec([1920.0, 1080.0], &[]));
    assert_eq!(found, Route::rest(1080.0, Vec::new()));
}

#[test]
fn clear_boxes_stay() {
    let boxes = [Rect::new(192.0, 216.0, 1536.0, 648.0), Rect::new(580.0, 869.0, 760.0, 126.0)];
    let found = route(&spec([1920.0, 1080.0], &boxes));
    assert_eq!(found, Route::rest(1080.0, Vec::new()));
}

#[test]
fn default_docks_beside() {
    let boxes = [PICKER, BAR, SEARCH, ENTRY, STRIP];
    let found = route(&spec(LOWEST, &boxes));
    assert_eq!(found.lane, thread_y(LOWEST[1]));
    let done = *plates(&found).last().expect("done");
    assert_eq!(done.right(), LOWEST[0] - 8.0);
    for plate in plates(&found) {
        assert!(boxes.iter().all(|bounds| !bounds.overlaps(plate)), "{plate:?}");
    }
}

#[test]
fn dock_clears_thread() {
    let boxes = [PICKER, BAR, SEARCH, ENTRY, STRIP];
    let found = route(&spec(LOWEST, &boxes));
    let top = found.lane - 38.0;
    assert!(plates(&found).iter().all(|plate| plate.bottom() < top));
}

#[test]
fn margin_tightens_when_needed() {
    assert_eq!(route(&spec(LOWEST, &[PICKER, SEARCH])).margin, MARGIN);
    let kinds = KINDS.map(|kind| KindSpec { plate: kind.plate + 26.0, ..kind });
    let wide = RouteSpec { kinds: &kinds, ..spec(LOWEST, &[PICKER, SEARCH]) };
    assert_eq!(route(&wide).margin, 8.0);
}

#[test]
fn picker_low_lifts() {
    let picker = Rect::new(136.6, 300.0, 1092.8, 460.8);
    let found = route(&spec(LOWEST, &[picker, BAR]));
    assert!(found.lane < 300.0, "{found:?}");
    assert!(found.lane - 38.0 >= BAR.bottom(), "{found:?}");
}

#[test]
fn full_picker_falls_back() {
    let found = route(&spec(LOWEST, &[Rect::new(0.0, 0.0, LOWEST[0], LOWEST[1])]));
    assert_eq!(found, Route::rest(LOWEST[1], Vec::new()));
}

#[test]
fn sweep_stays_free() {
    let spread = |size: [f32; 2]| {
        let (w, h) = (LOWEST[0] - size[0], LOWEST[1] - size[1]);
        [0.0, 0.5, 1.0].map(|at: f32| {
            [0.0, 0.5, 1.0].map(|up: f32| Rect::new(at * w, up * h, size[0], size[1]))
        })
    };
    let mut misses = Vec::new();
    let mut docked = 0;
    for picker in spread([PICKER.w, PICKER.h]).into_iter().flatten() {
        for search in spread([SEARCH.w, SEARCH.h]).into_iter().flatten() {
            for bar in spread([BAR.w, BAR.h]).into_iter().flatten() {
                for entry in spread([ENTRY.w, ENTRY.h]).into_iter().flatten() {
                    let boxes = [picker, search, bar, entry];
                    let route_spec = spec(LOWEST, &boxes);
                    let found = route(&route_spec);
                    let strand = route_spec.strand(&found.spans, found.margin, &found.dock);
                    docked += usize::from(!found.dock.is_empty());
                    let squeezed = !found.dock.is_empty()
                        && (0..KINDS.len())
                            .any(|kind| !plan(&StrandSpec { open: Some(kind), ..strand }).fits);
                    let row = Rect::new(0.0, found.lane - 17.0, LOWEST[0], 34.0);
                    let lost =
                        found.spans.is_empty() && boxes.iter().any(|bounds| bounds.overlaps(row));
                    let hidden = plates(&found)
                        .iter()
                        .any(|plate| boxes.iter().any(|bounds| bounds.overlaps(*plate)));
                    if lost || hidden || squeezed {
                        misses.push(boxes);
                    }
                }
            }
        }
    }
    assert!(docked > 0);
    assert!(misses.is_empty(), "{} {:?}", misses.len(), misses.first());
}
