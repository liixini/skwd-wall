#[cfg(test)]
#[path = "layout_place_tests.rs"]
mod tests;

use std::collections::HashSet;

use crate::contracts::picker::{Mode, format_config_number};
use crate::contracts::settings::corners::{self as corner_keys, CornerKeys, CornerStyle};
use crate::contracts::settings::{PREMADES, SettingsSource, keys, layout_paths};
use crate::i18n::{text, tr, tr_args};

use super::action::NudgeSpec;
use super::category::PICKER;
use super::corners as corner;
use super::places::{Asleep, Bind, PlaceSource, bind, checks, column, looked, noted, park, rows};
use super::places::{spoken, unfold};
use super::tabs::{layout_asleep_rows, take_card, take_control_paths};
use super::{ActionId, Cell, Control, Group, GroupKind, Note, Row, Section, SectionId, Trim};

use keys::filter_bar as bar;
use keys::general;
use keys::launch;
use keys::motion;
use keys::selector as sel;
use keys::system;
use keys::video_preview as video;

pub const PREMADE_KEY: &str = "layout.premade";
pub const LAYOUT_NAME_KEY: &str = "layout.name";

pub const STYLE: &str = "settings-section-layout-presets";
pub const SHAPE: &str = "settings-layout-shape";
pub const CORNERS: &str = "settings-selector-corners-card";
pub const RIBBONS: &str = "settings-selector-ribbons-card";
pub const ATMOSPHERE: &str = "settings-selector-hand-effects-card";
pub const SAND: &str = "settings-selector-sand-card";
pub const FLOW: &str = "settings-layout-flow";
pub const RING: &str = "settings-selector-ring-card";
pub const PLACEMENT: &str = "settings-layout-placement";
pub const CARDS: &str = "settings-layout-cards";
pub const OPENING: &str = "settings-layout-opening";

pub const GENERAL_CARD: &str = "settings-general-general-card";

const STYLE_TITLE: &str = "settings-layout-style";
const STYLE_DESC: &str = "settings-layout-style-desc";
const STYLE_SAY: &str = "settings-layout-style-say";
const STYLE_UNSAVED: &str = "settings-layout-style-unsaved";
const PREMADE_START: &str = "settings-layout-premade-start";
const PREMADE_HEAD: &str = "settings-layout-premade-head";
const USE: &str = "settings-layout-use";
const MODE_SLOT: Bind<'_> = Bind::new("mode", sel::DISPLAY_MODE);
const PREMADE_SLOT: Bind<'_> =
    Bind { blank: Some(PREMADE_START), ..Bind::new("premade", PREMADE_KEY) };
const NAME_ARG: &str = "name";
const ON_WORD: &str = "settings-layout-state-on";
const OFF_WORD: &str = "settings-layout-state-off";
const MILLIS: &str = "ms";

#[derive(Clone, Copy)]
enum Fact {
    Label(&'static str, &'static str),
    Number(&'static str, &'static str),
    Flag(&'static str, &'static str),
    Text(&'static str, &'static str),
}

type Table = &'static [(&'static str, Option<&'static str>)];
type Block = (Option<&'static str>, Table);

struct Part {
    key: &'static str,
    title: &'static str,
    desc: &'static str,
    columns: [&'static [Block]; 2],
    corners: Option<&'static CornerKeys>,
    trim: Trim,
    say: &'static str,
    slots: &'static [Bind<'static>],
    facts: &'static [Fact],
}

const SLICES: [Part; 1] = [Part {
    key: SHAPE,
    title: SHAPE,
    desc: "settings-layout-slices-desc",
    columns: [
        &[(
            Some("settings-layout-size"),
            &[
                (sel::SLICE_HEIGHT, None),
                (sel::VISIBLE_COUNT, None),
                (sel::EXPANDED_WIDTH, None),
                (sel::SLICE_WIDTH, None),
                (sel::SLICE_SPACING, None),
            ],
        )],
        &[(
            Some("settings-layout-tilt"),
            &[
                (sel::SKEW_OFFSET, None),
                (sel::SLICE_EDGE_TILT, None),
                (sel::SLICE_WOBBLE, None),
                (sel::SLICE_WOBBLE_STRENGTH, None),
                (sel::SLICE_PARALLAX, None),
                (sel::SLICE_SHADOWS, None),
            ],
        )],
    ],
    corners: None,
    trim: Trim::Bottom,
    say: "settings-layout-slices-say",
    slots: &[
        Bind::new("visible", sel::VISIBLE_COUNT),
        Bind::new("width", sel::EXPANDED_WIDTH),
        Bind::new("skew", sel::SKEW_OFFSET),
    ],
    facts: &[],
}];

const HAND: [Part; 3] = [
    Part {
        key: SHAPE,
        title: SHAPE,
        desc: "settings-layout-hand-desc",
        columns: [
            &[(
                Some("settings-selector-cards-card"),
                &[
                    (sel::HAND_MOVE, None),
                    (sel::HAND_COUNT, Some("settings-layout-hand-count")),
                    (sel::HAND_CARD_WIDTH, None),
                    (sel::HAND_CARD_HEIGHT, None),
                    (sel::HAND_SPEED, None),
                ],
            )],
            &[(
                Some("settings-layout-fan"),
                &[
                    (sel::HAND_SPREAD, None),
                    (sel::HAND_FAN_ANGLE, None),
                    (sel::HAND_FAN_ROLL, None),
                    (sel::HAND_ARCH, None),
                    (sel::HAND_PERSPECTIVE, None),
                    (sel::HAND_TILT, None),
                ],
            )],
        ],
        corners: None,
        trim: Trim::Bottom,
        say: "settings-layout-hand-say",
        slots: &[Bind::new("count", sel::HAND_COUNT), Bind::new("spread", sel::HAND_SPREAD)],
        facts: &[Fact::Label("move", sel::HAND_MOVE)],
    },
    Part {
        key: RIBBONS,
        title: RIBBONS,
        desc: "settings-layout-ribbons-desc",
        columns: [
            &[(
                Some("settings-layout-cuts"),
                &[
                    (sel::HAND_RIBBON_AXIS, None),
                    (sel::HAND_RIBBONS, None),
                    (sel::HAND_CUT, None),
                    (sel::HAND_CUT_VARIANCE, None),
                    (sel::HAND_SKEW, None),
                ],
            )],
            &[(Some(CORNERS), &[])],
        ],
        corners: Some(&corner_keys::HAND),
        trim: Trim::None,
        say: "settings-layout-ribbons-say",
        slots: &[],
        facts: &[
            Fact::Number("count", sel::HAND_RIBBONS),
            Fact::Label("axis", sel::HAND_RIBBON_AXIS),
            Fact::Text("style", corner_keys::HAND.style),
        ],
    },
    Part {
        key: ATMOSPHERE,
        title: ATMOSPHERE,
        desc: "settings-layout-atmosphere-desc",
        columns: [
            &[(
                None,
                &[
                    (sel::HAND_GHOSTS, None),
                    (sel::HAND_BACKDROP, None),
                    (sel::HAND_BACKDROP_BLUR, None),
                ],
            )],
            &[(None, &[(sel::HAND_BOB, None), (sel::HAND_REVEAL_FILL, None)])],
        ],
        corners: None,
        trim: Trim::None,
        say: "settings-layout-atmosphere-say",
        slots: &[],
        facts: &[
            Fact::Flag("ghosts", sel::HAND_GHOSTS),
            Fact::Flag("backdrop", sel::HAND_BACKDROP),
            Fact::Flag("bob", sel::HAND_BOB),
        ],
    },
];

const SANDY: [Part; 4] = [
    Part {
        key: SHAPE,
        title: SHAPE,
        desc: "settings-layout-sandy-desc",
        columns: [
            &[(
                Some("settings-selector-performance-card"),
                &[
                    (sel::SANDY_GRAIN, None),
                    (sel::SANDY_RES_SCALE, None),
                    (sel::SANDY_LOD_AUTO, None),
                    (sel::SANDY_LOD, None),
                ],
            )],
            &[(
                Some("settings-layout-sandy-slices"),
                &[
                    (sel::SANDY_SLICE_HEIGHT, None),
                    (sel::SANDY_SLICE_WIDTH, None),
                    (sel::SANDY_SPACING, None),
                    (sel::SANDY_SKEW, None),
                    (sel::SANDY_EDGE_SPEED, None),
                ],
            )],
        ],
        corners: None,
        trim: Trim::Bottom,
        say: "settings-layout-sandy-say",
        slots: &[Bind::new("grain", sel::SANDY_GRAIN), Bind::new("scale", sel::SANDY_RES_SCALE)],
        facts: &[],
    },
    Part {
        key: SAND,
        title: SAND,
        desc: "settings-layout-sand-desc",
        columns: [
            &[(
                Some("settings-layout-swap"),
                &[
                    (sel::SANDY_DURATION, None),
                    (sel::SANDY_BLEND, None),
                    (sel::SANDY_SWAP_STYLE, None),
                    (sel::SANDY_ARC, None),
                ],
            )],
            &[(
                Some("settings-layout-hand-off"),
                &[(sel::SANDY_SWAP_LOOP, None), (sel::SANDY_OUTGOING_LIVE, None)],
            )],
        ],
        corners: None,
        trim: Trim::Bottom,
        say: "settings-layout-sand-say",
        slots: &[],
        facts: &[
            Fact::Label("style", sel::SANDY_SWAP_STYLE),
            Fact::Number("duration", sel::SANDY_DURATION),
            Fact::Flag("loop", sel::SANDY_SWAP_LOOP),
        ],
    },
    Part {
        key: FLOW,
        title: FLOW,
        desc: "settings-layout-flow-desc",
        columns: [
            &[(
                None,
                &[
                    (sel::SANDY_STRANDS, None),
                    (sel::SANDY_TWIST, None),
                    (sel::SANDY_ORBIT, None),
                    (sel::SANDY_TURBULENCE, None),
                ],
            )],
            &[(
                None,
                &[(sel::SANDY_WAIST, None), (sel::SANDY_FRONT, None), (sel::SANDY_FAN, None)],
            )],
        ],
        corners: None,
        trim: Trim::None,
        say: "settings-layout-flow-say",
        slots: &[],
        facts: &[
            Fact::Number("ribbons", sel::SANDY_STRANDS),
            Fact::Number("twist", sel::SANDY_TWIST),
            Fact::Number("turbulence", sel::SANDY_TURBULENCE),
        ],
    },
    Part {
        key: RING,
        title: RING,
        desc: "settings-layout-ring-desc",
        columns: [
            &[(
                None,
                &[
                    (sel::SANDY_RING_SIZE, None),
                    (sel::SANDY_RING_SPIN, None),
                    (sel::SANDY_RING_WAVE, None),
                ],
            )],
            &[(
                None,
                &[
                    (sel::SANDY_RING_SOFT, None),
                    (sel::SANDY_RING_BLEND, None),
                    (sel::SANDY_RING_HOLD, None),
                ],
            )],
        ],
        corners: None,
        trim: Trim::None,
        say: "settings-layout-ring-say",
        slots: &[],
        facts: &[
            Fact::Number("size", sel::SANDY_RING_SIZE),
            Fact::Number("hold", sel::SANDY_RING_HOLD),
        ],
    },
];

const HEX: [Part; 1] = [Part {
    key: SHAPE,
    title: SHAPE,
    desc: "settings-layout-hex-desc",
    columns: [
        &[(
            Some("settings-selector-composition-card"),
            &[
                (sel::HEX_CURVE, None),
                (sel::HEX_SHAPE, None),
                (sel::HEX_ARC_INTENSITY_X10, None),
                (sel::HEX_CURVE_FREQUENCY, None),
                (sel::HEX_ORBIT_RADIUS, None),
                (sel::HEX_PARALLAX, None),
            ],
        )],
        &[(
            Some("settings-layout-grid-cells"),
            &[
                (sel::HEX_ROWS, None),
                (sel::HEX_COLS, None),
                (sel::HEX_RADIUS, None),
                (sel::HEX_GAP_X, None),
                (sel::HEX_GAP_Y, None),
            ],
        )],
    ],
    corners: None,
    trim: Trim::Bottom,
    say: "settings-layout-hex-say",
    slots: &[Bind::new("rows", sel::HEX_ROWS), Bind::new("columns", sel::HEX_COLS)],
    facts: &[Fact::Label("tiles", sel::HEX_SHAPE), Fact::Label("curve", sel::HEX_CURVE)],
}];

const WALL: [Part; 1] = [Part {
    key: SHAPE,
    title: SHAPE,
    desc: "settings-layout-wall-desc",
    columns: [
        &[(
            Some("settings-selector-composition-card"),
            &[
                (sel::GRID_LAYOUT, None),
                (sel::GRID_COLUMNS, None),
                (sel::GRID_ROWS, None),
                (sel::GRID_STAGGER, None),
                (sel::GRID_CYLINDER_BEND, None),
                (sel::GRID_CYLINDER_RADIUS, None),
                (sel::GRID_GAP_X, None),
                (sel::GRID_GAP_Y, None),
            ],
        )],
        &[(
            Some("settings-selector-cards-card"),
            &[(sel::GRID_THUMB_WIDTH, None), (sel::GRID_THUMB_HEIGHT, None)],
        )],
    ],
    corners: Some(&corner_keys::GRID),
    trim: Trim::Bottom,
    say: "settings-layout-wall-say",
    slots: &[Bind::new("columns", sel::GRID_COLUMNS), Bind::new("width", sel::GRID_THUMB_WIDTH)],
    facts: &[Fact::Label("mode", sel::GRID_LAYOUT)],
}];

const DEPTH: [Part; 1] = [Part {
    key: SHAPE,
    title: SHAPE,
    desc: "settings-layout-depth-desc",
    columns: [
        &[(
            Some("settings-selector-depth-card"),
            &[
                (sel::DEPTH_HEIGHT, None),
                (sel::DEPTH_WIDTH_PX, None),
                (sel::DEPTH_SPACING_PX, None),
                (sel::DEPTH_COUNT, None),
                (sel::DEPTH_FALLOFF_FACTOR, None),
                (sel::DEPTH_SKEW, None),
                (sel::DEPTH_NAVIGATION_MS, None),
            ],
        )],
        &[(
            Some("settings-layout-look"),
            &[(sel::DEPTH_SELECTION_FRAME, None), (sel::DEPTH_SHADOWS, None)],
        )],
    ],
    corners: Some(&corner_keys::DEPTH),
    trim: Trim::Rows,
    say: "settings-layout-depth-say",
    slots: &[
        Bind::new("count", sel::DEPTH_COUNT),
        Bind::new("spacing", sel::DEPTH_SPACING_PX),
        Bind::new("speed", sel::DEPTH_NAVIGATION_MS),
    ],
    facts: &[],
}];

const COLLECTION: [Part; 1] = [Part {
    key: SHAPE,
    title: SHAPE,
    desc: "settings-layout-collection-desc",
    columns: [
        &[(
            Some("settings-layout-stack"),
            &[
                (sel::COLLECTION_SIZE, None),
                (sel::COLLECTION_SPACING, None),
                (sel::COLLECTION_COUNT, None),
                (sel::COLLECTION_TILT, None),
                (sel::COLLECTION_SPEED, None),
            ],
        )],
        &[(Some(CORNERS), &[])],
    ],
    corners: Some(&corner_keys::COLLECTION),
    trim: Trim::Bottom,
    say: "settings-layout-collection-say",
    slots: &[
        Bind::new("count", sel::COLLECTION_COUNT),
        Bind::new("size", sel::COLLECTION_SIZE),
        Bind::new("tilt", sel::COLLECTION_TILT),
    ],
    facts: &[],
}];

const HAND_MOVES: [&str; 5] = [
    sel::HAND_MOVE_CORKSCREW,
    sel::HAND_MOVE_CASCADE,
    sel::HAND_MOVE_SHUFFLE,
    sel::HAND_MOVE_RIBBON,
    sel::HAND_MOVE_SPIRAL,
];
const PERFORMANCE: &str = "settings-selector-performance-card";
const METER_ARG: &str = "meter";
const PAIR_JOINER: &str = " · ";
const PAIRS: [(&str, &str); 3] = [
    (sel::SANDY_SPACING, sel::SANDY_SKEW),
    (sel::HAND_CARD_WIDTH, sel::HAND_CARD_HEIGHT),
    (sel::HAND_FAN_ANGLE, sel::HAND_FAN_ROLL),
];
const HAND_MOVES_TITLE: &str = "settings-layout-hand-moves";
const CORNERS_DESC: &str = "settings-layout-corners-desc";
const CORNERS_STRAIGHT_SAY: &str = "settings-layout-corners-straight";
const CORNERS_ALL_SAY: [&str; 2] =
    ["settings-layout-corners-all", "settings-layout-corners-all-cut"];
const CORNERS_EACH_SAY: [&str; 2] =
    ["settings-layout-corners-each", "settings-layout-corners-each-cut"];
const STYLE_SLOT: Bind<'_> = Bind::new("style", corner_keys::SLICES.style);
const CORNER_ALL_SLOTS: [Bind<'_>; 2] =
    [Bind::new("radius", corner_keys::SLICES.all), Bind::new("cut", corner_keys::SLICES.all)];
const CORNER_SLOTS: [Bind<'_>; 5] = [
    STYLE_SLOT,
    Bind::new("tl", sel::CORNER_TL),
    Bind::new("tr", sel::CORNER_TR),
    Bind::new("bl", sel::CORNER_BL),
    Bind::new("br", sel::CORNER_BR),
];

const PLACEMENT_DESC: &str = "settings-layout-placement-desc";
const PLACEMENT_SAY: &str = "settings-layout-placement-say";
const PLACEMENT_FIXED: &str = "settings-layout-placement-fixed";
const THIS_STYLE: &str = "settings-layout-this-style";
const SHARED: &str = "settings-layout-shared";
const PICKER_PAD: &str = "settings-layout-pad-picker";
const FILTER_PAD: &str = "settings-layout-pad-filter";
const SEARCH_PAD: &str = "settings-layout-pad-search";
const STEP_PERCENT: &str = "settings-layout-step-percent";
const STEP_PIXELS: &str = "settings-layout-step-pixels";
const CENTRE: &str = "settings-layout-centre";
const CENTRE_HINT: &str = "settings-layout-centre-hint";
const NO_OFFSETS: &str = "settings-layout-no-offsets";
const FRAME_SIZE: &str = "settings-layout-frame-size";
const MODE_ARG: &str = "mode";
const STAGES: [&str; 10] = [
    sel::SLICE_STAGE_X,
    sel::SLICE_STAGE_Y,
    sel::HEX_STAGE_X,
    sel::HEX_STAGE_Y,
    sel::GRID_STAGE_X,
    sel::GRID_STAGE_Y,
    sel::SANDY_STAGE_X,
    sel::SANDY_STAGE_Y,
    sel::HAND_STAGE_X,
    sel::HAND_STAGE_Y,
];
const ACROSS_SLOT: &str = "across";
const DOWN_SLOT: &str = "down";
const ARROWS: [(&str, i8, i8); 4] = [("←", -1, 0), ("↑", 0, -1), ("↓", 0, 1), ("→", 1, 0)];

const fn picker_pad(x: &'static str, y: &'static str) -> NudgeSpec {
    NudgeSpec { x, y, coarse: 1.0, fine: 1.0 }
}

static SLICES_PAD: NudgeSpec = picker_pad(sel::SLICE_STAGE_X, sel::SLICE_STAGE_Y);
static HEX_PAD: NudgeSpec = picker_pad(sel::HEX_STAGE_X, sel::HEX_STAGE_Y);
static WALL_PAD: NudgeSpec = picker_pad(sel::GRID_STAGE_X, sel::GRID_STAGE_Y);
static SANDY_PAD: NudgeSpec = picker_pad(sel::SANDY_STAGE_X, sel::SANDY_STAGE_Y);
static HAND_PAD: NudgeSpec = picker_pad(sel::HAND_STAGE_X, sel::HAND_STAGE_Y);
static FILTER_NUDGE: NudgeSpec =
    NudgeSpec { x: bar::OFFSET_X, y: bar::OFFSET_Y, coarse: 10.0, fine: 1.0 };
static SEARCH_NUDGE: NudgeSpec =
    NudgeSpec { x: sel::TAG_CLOUD_OFFSET_X, y: sel::TAG_CLOUD_OFFSET_Y, coarse: 10.0, fine: 1.0 };

const CARDS_DESC: &str = "settings-layout-cards-desc";
const CARDS_SAY: &str = "settings-layout-cards-say";
const PREVIEWS: &str = "settings-layout-previews";
const FLIP: &str = "settings-selector-card-flip-card";
const FLIP_HINT: &str = "settings-layout-flip-hint";
const VIDEO_ARG: &str = "video";
const DELAY_ARG: &str = "delay";
const VIDEO_STATES: [&str; 2] = ["off", "on"];
const EFFECT_SLOT: Bind<'_> = Bind::new("effect", sel::FLIP_EFFECT);
const FILTER_SLOT: Bind<'_> = Bind::new("filter", motion::FILTER_SWAP_SPEED);

const OPENING_DESC: &str = "settings-layout-opening-desc";
const OPENING_SAY: &str = "settings-layout-opening-say";
const OPENING_STILL: &str = "settings-layout-opening-still";
const OPENING_GROUP: &str = "settings-layout-opening-group";
const MOTION_GROUP: &str = "settings-layout-motion-group";
const SPEEDS: &str = "settings-layout-speeds";
const SPEED_NAMES: [&str; 3] =
    ["settings-motion-fast", "settings-motion-standard", "settings-motion-slow"];
const ANIM_SLOT: &str = "anim";
const CLOSES_ARG: &str = "closes";
const CLOSES: [&str; 2] = ["no", "yes"];
const ANIMATIONS: [(&str, &str); 4] = [
    ("none", "settings-layout-animation-none"),
    ("fade", "settings-layout-animation-fade"),
    ("rise", "settings-layout-animation-rise"),
    ("zoom", "settings-layout-animation-zoom"),
];
const NO_ANIMATION: &str = "none";
const MONITOR_SLOT: Bind<'_> = Bind::new("monitor", system::MONITOR);
const SCALE_SLOT: Bind<'_> = Bind::new("scale", general::UI_SCALE);
const START_SLOT: Bind<'_> = Bind::new("start", sel::START_POSITION);
const PACE_SLOT: Bind<'_> = Bind::new("pace", motion::LAUNCH_SPEED);

const AFTER_APPLYING: &str = "settings-layout-after-applying";
const NOT_SAVED: &str = "settings-layout-not-saved";
const FADE_HINT: &str = "settings-layout-fade-hint";
const FOLLOWS: &str = "settings-layout-follows";
const WRAPS: &str = "settings-layout-wraps";

const GATED: [(&str, &str); 6] = [
    (sel::HEX_ARC_INTENSITY_X10, sel::HEX_CURVE),
    (sel::HEX_CURVE_FREQUENCY, sel::HEX_CURVE),
    (sel::HEX_ORBIT_RADIUS, sel::HEX_CURVE),
    (sel::GRID_STAGGER, sel::GRID_LAYOUT),
    (sel::GRID_CYLINDER_BEND, sel::GRID_LAYOUT),
    (sel::GRID_CYLINDER_RADIUS, sel::GRID_LAYOUT),
];

fn parts(mode: Mode) -> &'static [Part] {
    match mode {
        Mode::Slices => &SLICES,
        Mode::Hand => &HAND,
        Mode::Sandy => &SANDY,
        Mode::Hex => &HEX,
        Mode::Grid => &WALL,
        Mode::Depth => &DEPTH,
        Mode::Collection => &COLLECTION,
    }
}

pub(super) fn compose(source: &PlaceSource<'_>) -> Vec<Section> {
    let cfg = source.cfg;
    let mode = Mode::from_key(&cfg.display_mode());
    let picker = source.cards(PICKER);
    let meter = picker
        .iter()
        .find(|(card, _)| card.key == PERFORMANCE)
        .and_then(|(_, rows)| rows.iter().find(|row| matches!(row.control, Control::Static)))
        .map(|row| row.title.clone());
    let mut pool: Vec<Row> = picker.into_iter().flat_map(|(_, rows)| rows).collect();
    pool.extend(take_card(&mut source.cards(super::category::APP), text(GENERAL_CARD)));
    let mut pool = flattened(pool);
    pool.iter_mut().for_each(settled);
    let mut sections = vec![style(source, &mut pool)];
    for part in parts(mode) {
        sections.push(shaped(cfg, part, meter.as_deref(), &mut pool));
    }
    if mode == Mode::Slices {
        sections.insert(2, corners(cfg, &mut pool));
    }
    sections.push(placement(mode, &mut pool));
    sections.push(cards(cfg, &mut pool));
    sections.push(opening(cfg, &mut pool));
    sections
}

pub(super) fn asleep(sections: &[Section], source: &PlaceSource<'_>) -> Vec<Asleep> {
    let mut seen: HashSet<String> = sections
        .iter()
        .flat_map(Section::rows)
        .filter_map(|row| row.control.path().map(str::to_string))
        .collect();
    let gate = |path: &str| {
        GATED
            .iter()
            .find(|(gated, gate)| *gated == path && seen_row(sections, gate))
            .map_or(sel::DISPLAY_MODE, |(_, gate)| gate)
    };
    layout_asleep_rows(source.cfg)
        .into_iter()
        .filter(|row| row.control.path().is_some_and(known))
        .filter(|row| row.control.path().is_some_and(|path| seen.insert(path.to_string())))
        .filter_map(|row| {
            let wakes = gate(row.control.path()?);
            Some(Asleep { row, wakes })
        })
        .collect()
}

fn settled(row: &mut Row) {
    if let Control::Dropdown { options, current, .. } | Control::Chips { options, current, .. } =
        &mut row.control
        && current.is_empty()
        && !options.iter().any(|(value, _)| value.is_empty())
        && let Some((first, _)) = options.first()
    {
        current.clone_from(first);
    }
}

fn flattened(mut pool: Vec<Row>) -> Vec<Row> {
    while pool.iter().any(|row| matches!(row.control, Control::Details { .. })) {
        pool = unfold(pool, |_| true);
    }
    pool
}

fn known(path: &str) -> bool {
    layout_paths().any(|owned| owned == path) || path == sel::HEX_ARC_INTENSITY_X10
}

fn seen_row(sections: &[Section], path: &str) -> bool {
    sections.iter().flat_map(Section::rows).any(|row| row.control.path() == Some(path))
}

fn id(key: &'static str) -> SectionId {
    SectionId::new(PICKER, key)
}

fn value_arg(cfg: &dyn SettingsSource, path: &str) -> String {
    format_config_number(cfg.number(path))
}

fn word(on: bool) -> String {
    tr(if on { ON_WORD } else { OFF_WORD }).to_string()
}

fn heading(key: Option<&'static str>) -> Option<String> {
    key.map(|key| tr(key).to_string())
}

fn style(source: &PlaceSource<'_>, pool: &mut Vec<Row>) -> Section {
    let premades: Vec<(String, String)> = PREMADES
        .iter()
        .enumerate()
        .map(|(at, premade)| (at.to_string(), tr(premade.name).to_string()))
        .collect();
    let mut head = rows(pool, &[(sel::DISPLAY_MODE, None)]);
    head.push(Row::of(
        tr(PREMADE_HEAD).to_string(),
        String::new(),
        Control::Dropdown {
            path: PREMADE_KEY.to_string(),
            options: premades,
            current: String::new(),
            palettes: Vec::new(),
        },
    ));
    let mut uses: Vec<Row> = PREMADES
        .iter()
        .enumerate()
        .map(|(at, premade)| {
            Row::of(
                tr(premade.name).to_string(),
                tr(premade.desc).to_string(),
                Control::ActionBtn { id: ActionId::UsePremade(at), label: tr(USE).to_string() },
            )
        })
        .collect();
    let rest = uses.split_off(uses.len().div_ceil(2));
    let section = Section {
        head,
        groups: vec![column(Some(PREMADE_HEAD), uses), Group::new(GroupKind::Column, None, rest)],
        ..Section::new(id(STYLE), tr(STYLE_TITLE), tr(STYLE_DESC))
    };
    let unsaved = source
        .runtime
        .layout_premade
        .and_then(|at| PREMADES.get(at))
        .map(|premade| tr(premade.name));
    let key = if unsaved.is_some() { STYLE_UNSAVED } else { STYLE_SAY };
    spoken(section, |section| {
        bind(section, key, &[MODE_SLOT, PREMADE_SLOT], &[(NAME_ARG, unsaved.unwrap_or_default())])
    })
}

fn facts(
    section: &Section,
    cfg: &dyn SettingsSource,
    facts: &[Fact],
) -> Vec<(&'static str, String)> {
    facts
        .iter()
        .map(|fact| match fact {
            Fact::Number(arg, path) => (*arg, value_arg(cfg, path)),
            Fact::Flag(arg, path) => {
                let on = section
                    .rows()
                    .find(|row| row.control.path() == Some(*path))
                    .and_then(|row| row.control.switched_on());
                (*arg, word(on.unwrap_or_default()))
            }
            Fact::Label(arg, path) => {
                let row = section.rows().find(|row| row.control.path() == Some(*path));
                let label = row.and_then(|row| row.control.current_label()).unwrap_or_default();
                (*arg, label.to_string())
            }
            Fact::Text(arg, path) => (*arg, cfg.text(path)),
        })
        .collect()
}

fn paired(mut column: Vec<Row>) -> Vec<Row> {
    for (lead, partner) in PAIRS {
        let Some(at) = column.iter().position(|row| row.control.path() == Some(lead)) else {
            continue;
        };
        if column.get(at + 1).and_then(|row| row.control.path()) != Some(partner) {
            continue;
        }
        let title = format!("{}{PAIR_JOINER}{}", column[at].title, column[at + 1].title);
        column[at].title = title;
        column[at + 1].look.cell = Cell::Beside;
    }
    column
}

fn shaped(
    cfg: &dyn SettingsSource,
    part: &Part,
    meter: Option<&str>,
    pool: &mut Vec<Row>,
) -> Section {
    let mut groups: Vec<Group> = part
        .columns
        .iter()
        .map(|blocks| {
            let mut column = Vec::new();
            for (at, (key, table)) in blocks.iter().enumerate() {
                if at > 0 {
                    column.extend(key.map(segment));
                }
                let start = column.len();
                column.extend(paired(rows(pool, table)));
                if table.first().is_some_and(|(path, _)| *path == sel::HAND_MOVE) {
                    let after = start + (column.len() - start).min(1);
                    column.splice(after..after, hand_moves(pool));
                }
            }
            Group::new(GroupKind::Column, blocks.first().and_then(|(key, _)| heading(*key)), column)
        })
        .collect();
    let group = part.corners.map(|keys| corner_group(cfg, keys, pool));
    let mut head = Vec::new();
    if let (Some(group), Some(column)) = (group, groups.last_mut()) {
        column.rows.extend(group.lead.into_iter().chain(group.shown));
        head = group.hidden;
    }
    let section = Section {
        head,
        groups,
        wide: true,
        undivided: true,
        trim: part.trim,
        ..Section::new(id(part.key), tr(part.title), tr(part.desc))
    };
    let args = facts(&section, cfg, part.facts);
    let mut args: Vec<_> = args.iter().map(|(arg, value)| (*arg, value.as_str())).collect();
    args.push((METER_ARG, meter.unwrap_or_default()));
    let mut section = spoken(section, |section| bind(section, part.say, part.slots, &args));
    if let Some(keys) = part.corners {
        park(&mut section, keys.style, |_| true);
    }
    section
}

fn segment(key: &'static str) -> Row {
    Row::of(tr(key).to_string(), String::new(), Control::Segment)
}

fn hand_moves(pool: &mut Vec<Row>) -> Vec<Row> {
    let members: Vec<Row> =
        HAND_MOVES.iter().flat_map(|path| take_control_paths(pool, &[path])).collect();
    if members.is_empty() {
        return members;
    }
    checks(tr(HAND_MOVES_TITLE), String::new(), members)
}

struct CornerGroup {
    style: CornerStyle,
    same: bool,
    lead: Vec<Row>,
    shown: Vec<Row>,
    hidden: Vec<Row>,
}

fn corner_group(cfg: &dyn SettingsSource, keys: &CornerKeys, pool: &mut Vec<Row>) -> CornerGroup {
    let style = corner::style(cfg, keys);
    let same = cfg.flag(keys.same);
    let words = corner::words(style);
    let lead = rows(pool, &[(keys.style, None)]);
    let toggle = rows(pool, &[(keys.same, Some(words.hint))]);
    let every = rows(pool, &[(keys.all, None)]);
    let cells = rows(pool, &corner::CELLS.map(|at| (keys.values[at], None)));
    let cut = style == CornerStyle::Chamfered;
    let mut shown = Vec::new();
    let mut hidden = Vec::new();
    match (style, same) {
        (CornerStyle::Straight, _) => {
            hidden.extend(toggle.into_iter().chain(every).chain(cells));
        }
        (_, true) => {
            shown.extend(toggle);
            shown.extend(every.into_iter().map(|row| {
                let all = Row { title: tr(words.all).to_string(), ..row };
                corner_cell(all, Cell::Corners { cut })
            }));
            hidden.extend(cells);
        }
        (_, false) => {
            shown.extend(toggle);
            hidden.extend(every);
            let cell = |(at, row)| {
                corner_cell(row, if at == 0 { Cell::Corners { cut } } else { Cell::Beside })
            };
            shown.extend(cells.into_iter().enumerate().map(cell));
        }
    }
    CornerGroup { style, same, lead, shown, hidden }
}

fn corners(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Section {
    let keys = &corner_keys::SLICES;
    let CornerGroup { style, same, lead, shown, hidden } = corner_group(cfg, keys, pool);
    let mut group = Group::new(GroupKind::Column, None, shown);
    group.echoes = lead.first().cloned().map(|echo| (0, echo)).into_iter().collect();
    let section = Section {
        head: lead.into_iter().chain(hidden).collect(),
        groups: vec![group],
        wide: true,
        undivided: true,
        min_columns: 2,
        ..Section::new(id(CORNERS), tr(CORNERS), tr(CORNERS_DESC))
    };
    let cut = usize::from(style == CornerStyle::Chamfered);
    let mut section = spoken(section, |section| match (style, same) {
        (CornerStyle::Straight, _) => bind(section, CORNERS_STRAIGHT_SAY, &[STYLE_SLOT], &[]),
        (_, true) => bind(section, CORNERS_ALL_SAY[cut], &[STYLE_SLOT, CORNER_ALL_SLOTS[cut]], &[]),
        (_, false) => bind(section, CORNERS_EACH_SAY[cut], &CORNER_SLOTS, &[]),
    });
    park(&mut section, keys.style, |row| row.control.path() != Some(keys.style));
    section
}

fn corner_cell(row: Row, cell: Cell) -> Row {
    Row { look: super::Look { cell, note: Note::Quiet, ..super::Look::default() }, ..row }
}

fn pad(
    title: &'static str,
    hint: &'static str,
    spec: &'static NudgeSpec,
    pool: &mut Vec<Row>,
) -> Vec<Row> {
    let mut items: Vec<(ActionId, String)> = ARROWS
        .iter()
        .map(|(glyph, dx, dy)| (ActionId::Nudge(spec, *dx, *dy), (*glyph).to_string()))
        .collect();
    items.push((ActionId::CentreNudge(spec), tr(CENTRE).to_string()));
    let lead = Row {
        look: super::Look {
            cell: Cell::Nudge,
            note: Note::Own(tr(hint).to_string()),
            ..super::Look::default()
        },
        ..Row::of(tr(title).to_string(), String::new(), Control::ActionChips { items })
    };
    let axes: Vec<Row> =
        [spec.x, spec.y].iter().flat_map(|path| take_control_paths(pool, &[path])).collect();
    std::iter::once(lead).chain(looked(noted(axes, None), Cell::Beside)).collect()
}

fn beside(pool: &mut Vec<Row>, lead: &str, partners: &[&str]) -> Vec<Row> {
    let mut row = noted(take_control_paths(pool, &[lead]), None);
    let partners: Vec<Row> =
        partners.iter().flat_map(|path| take_control_paths(pool, &[path])).collect();
    row.extend(looked(noted(partners, None), Cell::Beside));
    row
}

fn placement(mode: Mode, pool: &mut Vec<Row>) -> Section {
    let spec = match mode {
        Mode::Slices => Some(&SLICES_PAD),
        Mode::Hex => Some(&HEX_PAD),
        Mode::Grid => Some(&WALL_PAD),
        Mode::Sandy => Some(&SANDY_PAD),
        Mode::Hand => Some(&HAND_PAD),
        Mode::Depth | Mode::Collection => None,
    };
    let name = mode_label(mode);
    let mut left = match spec {
        Some(spec) => pad(PICKER_PAD, STEP_PERCENT, spec, pool),
        None => vec![Row::of(String::new(), tr_args!(NO_OFFSETS, mode => name), Control::Static)],
    };
    if mode == Mode::Sandy {
        left.extend(rows(pool, &[(sel::SANDY_CENTER, Some(CENTRE_HINT))]));
    }
    left.extend(beside(pool, sel::FRAME_ENABLED, &[sel::FRAME_X, sel::FRAME_Y]));
    let mut size = beside(pool, sel::FRAME_WIDTH, &[sel::FRAME_HEIGHT]);
    if let Some(lead) = size.first_mut() {
        lead.title = tr(FRAME_SIZE).to_string();
    }
    left.extend(size);
    let mut right = pad(FILTER_PAD, STEP_PIXELS, &FILTER_NUDGE, pool);
    right.extend(beside(pool, bar::POSITION_ENABLED, &[bar::POSITION_X, bar::POSITION_Y]));
    right.extend(pad(SEARCH_PAD, STEP_PIXELS, &SEARCH_NUDGE, pool));
    right.extend(beside(
        pool,
        sel::TAG_CLOUD_POSITION_ENABLED,
        &[sel::TAG_CLOUD_POSITION_X, sel::TAG_CLOUD_POSITION_Y],
    ));
    let this = tr_args!(THIS_STYLE, mode => name);
    let head: Vec<Row> = STAGES
        .iter()
        .filter(|path| spec.is_none_or(|spec| **path != spec.x && **path != spec.y))
        .flat_map(|path| take_control_paths(pool, &[path]))
        .collect();
    let section = Section {
        head,
        groups: vec![
            Group::new(GroupKind::Column, Some(this), left),
            Group::new(GroupKind::Column, heading(Some(SHARED)), right),
        ],
        wide: true,
        ..Section::new(id(PLACEMENT), tr(PLACEMENT), tr(PLACEMENT_DESC))
    };
    let mut section = spoken(section, |section| match spec {
        Some(spec) => bind(
            section,
            PLACEMENT_SAY,
            &[Bind::new(ACROSS_SLOT, spec.x), Bind::new(DOWN_SLOT, spec.y)],
            &[],
        ),
        None => bind(section, PLACEMENT_FIXED, &super::places::NO_SLOTS, &[(MODE_ARG, name)]),
    });
    park(&mut section, sel::FRAME_ENABLED, |_| true);
    section
}

fn mode_label(mode: Mode) -> &'static str {
    tr(match mode {
        Mode::Slices => "settings-selector-mode-slices",
        Mode::Hex => "settings-selector-mode-hex",
        Mode::Grid => "settings-selector-mode-wall",
        Mode::Sandy => "settings-selector-mode-sandy",
        Mode::Hand => "settings-selector-mode-hand",
        Mode::Depth => "settings-selector-mode-depth",
        Mode::Collection => "settings-selector-mode-collection",
    })
}

fn cards(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Section {
    let head = rows(pool, &[(sel::FLIP_EFFECT, None), (motion::FILTER_SWAP_SPEED, None)]);
    let mut left = rows(pool, &[(sel::LIVE_PREVIEW, None), (sel::SHOW_TYPE_BADGES, None)]);
    left.extend(beside(pool, video::ENABLED, &[video::DELAY_MS]));
    let right = rows(
        pool,
        &[
            (sel::FLIP_SHADER, None),
            (sel::FLIP_BACK_REVEAL, None),
            (sel::FLIP_DURATION_MS, Some(FLIP_HINT)),
        ],
    );
    let section = Section {
        head,
        groups: vec![column(Some(PREVIEWS), left), column(Some(FLIP), right)],
        ..Section::new(id(CARDS), tr(CARDS), tr(CARDS_DESC))
    };
    let delay = value_arg(cfg, video::DELAY_MS);
    let state = VIDEO_STATES[usize::from(cfg.flag(video::ENABLED))];
    spoken(section, |section| {
        bind(
            section,
            CARDS_SAY,
            &[EFFECT_SLOT, FILTER_SLOT],
            &[(VIDEO_ARG, state), (DELAY_ARG, delay.as_str())],
        )
    })
}

fn speeds() -> Vec<Row> {
    let number = |key: &'static str, path: &str, cell: Cell, unit: &'static str| Row {
        look: super::Look { cell, note: Note::Quiet, ..super::Look::default() },
        ..Row::of(
            tr(key).to_string(),
            String::new(),
            Control::Number { key: path.to_string(), path: path.to_string(), unit },
        )
    };
    let mut fast = number(SPEED_NAMES[0], motion::FAST_MS, Cell::Value, "");
    fast.title = tr(SPEEDS).to_string();
    vec![
        fast,
        number(SPEED_NAMES[1], motion::STANDARD_MS, Cell::Beside, ""),
        number(SPEED_NAMES[2], motion::SLOW_MS, Cell::Beside, MILLIS),
    ]
}

fn opening(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Section {
    let mut head = rows(
        pool,
        &[
            (system::MONITOR, None),
            (general::UI_SCALE, None),
            (sel::START_POSITION, None),
            (launch::ANIMATION, None),
            (motion::LAUNCH_SPEED, None),
        ],
    );
    head.retain(|row| !matches!(row.control, Control::Static));
    let left = rows(
        pool,
        &[
            (general::CLOSE_ON_SELECTION, Some(AFTER_APPLYING)),
            (general::APPLY_ON_PICKER_MONITOR, Some(NOT_SAVED)),
            (general::OPEN_FADE_FROM, Some(FADE_HINT)),
        ],
    );
    let mut right = speeds();
    right.extend(rows(
        pool,
        &[(general::HOVER_SELECTS, Some(FOLLOWS)), (sel::LOOP_AROUND, Some(WRAPS))],
    ));
    let section = Section {
        head,
        groups: vec![column(Some(OPENING_GROUP), left), column(Some(MOTION_GROUP), right)],
        trim: Trim::Bottom,
        ..Section::new(id(OPENING), tr(OPENING), tr(OPENING_DESC))
    };
    let current = cfg.launch_animation();
    let phrase = ANIMATIONS.iter().find(|(value, _)| *value == current).map(|(_, key)| *key);
    let anim = Bind { blank: phrase, ..Bind::new(ANIM_SLOT, launch::ANIMATION) };
    let closes = (CLOSES_ARG, CLOSES[usize::from(cfg.flag(general::CLOSE_ON_SELECTION))]);
    let still = current == NO_ANIMATION;
    let mut section = spoken(section, |section| {
        if still {
            let slots = [MONITOR_SLOT, SCALE_SLOT, START_SLOT, anim];
            return bind(section, OPENING_STILL, &slots, &[closes]);
        }
        let slots = [MONITOR_SLOT, SCALE_SLOT, START_SLOT, anim, PACE_SLOT];
        bind(section, OPENING_SAY, &slots, &[closes])
    });
    if still {
        park(&mut section, launch::ANIMATION, |row| {
            row.control.path() == Some(motion::LAUNCH_SPEED)
        });
    }
    section
}
