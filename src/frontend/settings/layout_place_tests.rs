use super::*;
use crate::contracts::settings::layout_paths;
use crate::frontend::settings::place;
use crate::frontend::settings::places::drawn;
use crate::frontend::settings::test_source::{FakeSettingsSource, place_source as source, plain};

const ARRANGED: [&str; 6] = [
    bar::ARRANGE_POSITION_ENABLED,
    bar::ARRANGE_POSITION_X,
    bar::ARRANGE_POSITION_Y,
    bar::SWATCH_POSITION_ENABLED,
    bar::SWATCH_POSITION_X,
    bar::SWATCH_POSITION_Y,
];
const METER_GAP: &str = "  ·  ";

fn cfg(mode: Mode) -> FakeSettingsSource {
    FakeSettingsSource::default().with_text(sel::DISPLAY_MODE, mode.as_key())
}

fn styled(mode: Mode, style: CornerStyle, same: bool) -> FakeSettingsSource {
    let keys = mode.corners().expect("corner keys");
    cfg(mode).with_text(keys.style, style.key()).with_flag(keys.same, same)
}

const CORNERED: [(Mode, &str); 5] = [
    (Mode::Slices, CORNERS),
    (Mode::Hand, RIBBONS),
    (Mode::Grid, SHAPE),
    (Mode::Depth, SHAPE),
    (Mode::Collection, SHAPE),
];
const LEGACY: [&str; 6] = [
    sel::ROUND_CORNERS,
    sel::GRID_ROUND_CORNERS,
    sel::GRID_CORNER_RADIUS,
    sel::HAND_CORNER_RADIUS,
    sel::DEPTH_CORNERS,
    sel::COLLECTION_CORNERS,
];

fn sections(cfg: &dyn SettingsSource) -> Vec<Section> {
    crate::i18n::with_language("en-US", || source(cfg).place(place::LAYOUT).sections)
}

fn keys_of(mode: Mode) -> Vec<&'static str> {
    sections(&cfg(mode)).iter().map(|section| section.id.key).collect()
}

fn said(cfg: &dyn SettingsSource, key: &str) -> String {
    let found = sections(cfg).into_iter().find(|section| section.id.key == key).expect("section");
    plain(&found.say.text)
}

#[test]
fn board_sections_each_mode() {
    let tail = [PLACEMENT, CARDS, OPENING];
    let with = |middle: &[&'static str]| {
        let mut keys = vec![STYLE];
        keys.extend_from_slice(middle);
        keys.extend(tail);
        keys
    };
    assert_eq!(keys_of(Mode::Slices), with(&[SHAPE, CORNERS]));
    assert_eq!(keys_of(Mode::Hand), with(&[SHAPE, RIBBONS, ATMOSPHERE]));
    assert_eq!(keys_of(Mode::Sandy), with(&[SHAPE, SAND, FLOW, RING]));
    for mode in [Mode::Hex, Mode::Grid, Mode::Depth, Mode::Collection] {
        assert_eq!(keys_of(mode), with(&[SHAPE]));
    }
}

#[test]
fn head_rows_drawn() {
    for mode in Mode::ALL {
        for section in sections(&cfg(mode)) {
            assert!(drawn(&section), "{}", section.title);
        }
    }
    for (mode, _) in CORNERED {
        for style in CornerStyle::ALL {
            for same in [true, false] {
                for section in sections(&styled(mode, style, same)) {
                    assert!(drawn(&section), "{} {style:?} {same}", section.title);
                }
            }
        }
    }
}

#[test]
fn studio_keeps_section() {
    let keyed = sections(&cfg(Mode::Slices));
    assert!(super::super::is_picker_layout_section(Some(keyed[0].id)));
}

#[test]
fn snapshot_covers_rows() {
    let excused = [
        general::APPLY_ON_PICKER_MONITOR,
        sel::HEX_ARC_INTENSITY_X10,
        PREMADE_KEY,
        system::MONITOR,
        general::UI_SCALE,
    ];
    for mode in Mode::ALL {
        for section in sections(&cfg(mode)) {
            for row in section.rows() {
                let Some(path) = row.control.path() else { continue };
                assert!(
                    excused.contains(&path) || layout_paths().any(|owned| owned == path),
                    "{path}"
                );
            }
        }
    }
}

#[test]
fn snapshot_paths_reached() {
    let cfg = cfg(Mode::Slices);
    let src = source(&cfg);
    let shown = crate::i18n::with_language("en-US", || src.place(place::LAYOUT).sections);
    let mut reached: HashSet<String> = shown
        .iter()
        .flat_map(Section::rows)
        .filter_map(|row| row.control.path().map(str::to_string))
        .collect();
    reached.extend(
        asleep(&shown, &src)
            .into_iter()
            .filter_map(|sleeper| sleeper.row.control.path().map(str::to_string)),
    );
    let lost: Vec<_> = layout_paths()
        .filter(|path| {
            !reached.contains(*path)
                && *path != sel::HEX_ARC_INTENSITY
                && !ARRANGED.contains(path)
                && !LEGACY.contains(path)
        })
        .collect();
    assert!(lost.is_empty(), "{lost:?}");
}

#[test]
fn every_sleeper_lands() {
    for mode in Mode::ALL {
        let cfg = cfg(mode);
        let src = source(&cfg);
        let shown = crate::i18n::with_language("en-US", || src.place(place::LAYOUT).sections);
        let sleepers = crate::i18n::with_language("en-US", || asleep(&shown, &src));
        assert!(!sleepers.is_empty());
        for sleeper in sleepers {
            assert!(seen_row(&shown, sleeper.wakes), "{}", sleeper.wakes);
        }
    }
}

#[test]
fn gated_rows_wake_gate() {
    let cfg = cfg(Mode::Hex);
    let src = source(&cfg);
    let shown = crate::i18n::with_language("en-US", || src.place(place::LAYOUT).sections);
    let sleepers = asleep(&shown, &src);
    let wave = sleepers
        .iter()
        .find(|sleeper| sleeper.row.control.path() == Some(sel::HEX_CURVE_FREQUENCY))
        .expect("waves");
    assert_eq!(wave.wakes, sel::HEX_CURVE);
}

#[test]
fn bar_rows_stay_out() {
    for mode in Mode::ALL {
        let paths: Vec<_> = sections(&cfg(mode))
            .iter()
            .flat_map(Section::rows)
            .filter_map(|row| row.control.path().map(str::to_string))
            .collect();
        assert!(
            !paths.iter().any(|path| path == sel::TAG_CLOUD_WIDTH || path == sel::TAG_CLOUD_ROWS)
        );
    }
}

#[test]
fn sentences_read_right() {
    let slices = cfg(Mode::Slices);
    assert_eq!(said(&slices, STYLE), "The picker uses [0]. [1].");
    assert_eq!(said(&slices, CORNERS), "Corner style [0]: square corners.");
    let same = styled(Mode::Slices, CornerStyle::Round, true);
    assert_eq!(said(&same, CORNERS), "Corner style [0], radius [1] on every corner.");
    let each = styled(Mode::Slices, CornerStyle::Round, false);
    assert_eq!(
        said(&each, CORNERS),
        "Corner style [0], radius [1], [2], [3], [4], top row then bottom."
    );
    let cut = styled(Mode::Slices, CornerStyle::Chamfered, true);
    assert_eq!(said(&cut, CORNERS), "Corner style [0], cut [1] on every corner.");
    let hand = styled(Mode::Hand, CornerStyle::Chamfered, true);
    assert!(said(&hand, RIBBONS).ends_with("; chamfered corners."));
    assert_eq!(said(&cfg(Mode::Depth), PLACEMENT), "Depth always sits in the centre.");
    let still = cfg(Mode::Slices).with_text(launch::ANIMATION, NO_ANIMATION);
    assert!(said(&still, OPENING).contains("Appears with [3]."));
    let off = cfg(Mode::Slices).with_flag(video::ENABLED, false);
    assert!(said(&off, CARDS).ends_with("Video preview is off."));
}

#[test]
fn premade_marks_unsaved() {
    let cfg = cfg(Mode::Slices);
    let runtime =
        crate::frontend::settings::RuntimeStatus { layout_premade: Some(0), ..Default::default() };
    let src = PlaceSource { runtime, ..source(&cfg) };
    let text = crate::i18n::with_language("en-US", || {
        src.place(place::LAYOUT).sections[0].say.text.clone()
    });
    assert!(plain(&text).ends_with("[1] (Classic slices, unsaved)."));
}

#[test]
fn corner_cells_follow_same() {
    for (mode, key) in CORNERED {
        for (style, same, numbers, cut) in [
            (CornerStyle::Straight, true, 0, None),
            (CornerStyle::Round, true, 1, Some(false)),
            (CornerStyle::Round, false, 4, Some(false)),
            (CornerStyle::Chamfered, true, 1, Some(true)),
            (CornerStyle::Chamfered, false, 4, Some(true)),
        ] {
            let found = sections(&styled(mode, style, same))
                .into_iter()
                .find(|section| section.id.key == key)
                .unwrap();
            let keys = mode.corners().unwrap();
            let cells: Vec<_> = found
                .body()
                .filter(|row| {
                    let path = row.control.path().unwrap_or_default();
                    if same { path == keys.all } else { keys.values.contains(&path) }
                })
                .collect();
            assert_eq!(cells.len(), numbers, "{mode:?} {style:?} {same}");
            let lead = cells.first().map(|row| row.look.cell);
            assert_eq!(lead, cut.map(|cut| Cell::Corners { cut }), "{mode:?} {style:?}");
            assert!(found.rows().any(|row| row.control.path() == Some(keys.style)));
        }
    }
}

#[test]
fn pads_step_board() {
    assert_eq!((SLICES_PAD.coarse, SLICES_PAD.fine), (1.0, 1.0));
    assert_eq!((FILTER_NUDGE.coarse, FILTER_NUDGE.fine), (10.0, 1.0));
    let placement = sections(&cfg(Mode::Slices))
        .into_iter()
        .find(|section| section.id.key == PLACEMENT)
        .unwrap();
    let pads = placement.body().filter(|row| row.look.cell == Cell::Nudge).count();
    assert_eq!(pads, 3);
    let depth = sections(&cfg(Mode::Depth))
        .into_iter()
        .find(|section| section.id.key == PLACEMENT)
        .unwrap();
    assert_eq!(depth.body().filter(|row| row.look.cell == Cell::Nudge).count(), 2);
}

#[test]
fn no_blank_values() {
    let mut fixtures: Vec<_> = Mode::ALL.iter().map(|mode| cfg(*mode)).collect();
    fixtures.extend(CORNERED.map(|(mode, _)| styled(mode, CornerStyle::Chamfered, false)));
    fixtures.push(cfg(Mode::Hex).with_text(sel::HEX_CURVE, "cylinder"));
    for fixture in &fixtures {
        for section in sections(fixture) {
            for row in section.rows() {
                let named = row.control.current_label().is_some_and(|label| !label.is_empty());
                let menu = matches!(row.control, Control::Dropdown { .. } | Control::Chips { .. });
                let premade = row.control.path() == Some(PREMADE_KEY);
                assert!(!menu || named || premade, "{}", row.title);
            }
            let said = plain(&section.say.text).replace(METER_GAP, " · ");
            for gap in ["  ", " .", " ,", " ;", "()"] {
                assert!(!said.contains(gap), "{said}");
            }
            assert!(!said.trim().is_empty(), "{}", section.title);
        }
    }
}
