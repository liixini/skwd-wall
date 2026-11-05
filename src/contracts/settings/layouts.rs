use crate::contracts::picker::Mode;

use super::corners;
use super::keys::{filter_bar, general, launch, motion, selector, video_preview};

pub const DEFAULT_CORNER: f64 = 12.0;
pub const SAVED_LAYOUTS: &str = "components.wallpaperSelector.savedLayouts";
pub const ACTIVE_LAYOUT: &str = "components.wallpaperSelector.activeLayout";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SavedLayouts {
    pub names: Vec<String>,
    pub active: Option<usize>,
    pub changes: usize,
}

impl SavedLayouts {
    pub fn dirty(&self) -> bool {
        self.changes > 0
    }

    pub fn current(&self) -> Option<&str> {
        self.active.and_then(|at| self.names.get(at)).map(String::as_str)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stored {
    Number(f64),
    Text(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Premade {
    pub name: &'static str,
    pub desc: &'static str,
    pub mode: Mode,
    pub values: &'static [(&'static str, Stored)],
}

pub const PREMADES: [Premade; 6] = [
    Premade {
        name: "settings-layout-premade-classic",
        desc: "settings-layout-premade-classic-desc",
        mode: Mode::Slices,
        values: &[
            (selector::VISIBLE_COUNT, Stored::Number(12.0)),
            (selector::EXPANDED_WIDTH, Stored::Number(924.0)),
            (selector::SKEW_OFFSET, Stored::Number(35.0)),
        ],
    },
    Premade {
        name: "settings-layout-premade-compact",
        desc: "settings-layout-premade-compact-desc",
        mode: Mode::Hex,
        values: &[
            (selector::HEX_ROWS, Stored::Number(2.0)),
            (selector::HEX_COLS, Stored::Number(9.0)),
            (selector::HEX_RADIUS, Stored::Number(160.0)),
            (selector::HEX_CURVE, Stored::Text("flat")),
        ],
    },
    Premade {
        name: "settings-layout-premade-gallery",
        desc: "settings-layout-premade-gallery-desc",
        mode: Mode::Grid,
        values: &[
            (selector::GRID_LAYOUT, Stored::Text("justified")),
            (selector::GRID_COLUMNS, Stored::Number(7.0)),
            (selector::GRID_THUMB_HEIGHT, Stored::Number(180.0)),
        ],
    },
    Premade {
        name: "settings-layout-premade-sandy",
        desc: "settings-layout-premade-sandy-desc",
        mode: Mode::Sandy,
        values: &[
            (selector::SANDY_STRANDS, Stored::Number(10.0)),
            (selector::SANDY_SWAP_STYLE, Stored::Text("weave")),
        ],
    },
    Premade {
        name: "settings-layout-premade-hand",
        desc: "settings-layout-premade-hand-desc",
        mode: Mode::Hand,
        values: &[
            (selector::HAND_COUNT, Stored::Number(7.0)),
            (selector::HAND_SPREAD, Stored::Number(200.0)),
            (selector::HAND_FAN_ANGLE, Stored::Number(9.0)),
        ],
    },
    Premade {
        name: "settings-layout-premade-depth",
        desc: "settings-layout-premade-depth-desc",
        mode: Mode::Depth,
        values: &[
            (selector::DEPTH_COUNT, Stored::Number(9.0)),
            (selector::DEPTH_SPACING_PX, Stored::Number(90.0)),
        ],
    },
];

const PLAIN_PATHS: &[&str] = &[
    selector::DISPLAY_MODE,
    selector::SLICE_HEIGHT,
    selector::VISIBLE_COUNT,
    selector::EXPANDED_WIDTH,
    selector::SLICE_WIDTH,
    selector::SLICE_SPACING,
    selector::SKEW_OFFSET,
    selector::SLICE_EDGE_TILT,
    selector::SLICE_WOBBLE,
    selector::SLICE_WOBBLE_STRENGTH,
    selector::SLICE_PARALLAX,
    selector::SLICE_SHADOWS,
    selector::ROUND_CORNERS,
    selector::SLICE_STAGE_X,
    selector::SLICE_STAGE_Y,
    selector::HEX_PARALLAX,
    selector::HEX_CURVE,
    selector::HEX_SHAPE,
    selector::HEX_ARC_INTENSITY,
    selector::HEX_CURVE_FREQUENCY,
    selector::HEX_ORBIT_RADIUS,
    selector::HEX_ROWS,
    selector::HEX_COLS,
    selector::HEX_RADIUS,
    selector::HEX_GAP_X,
    selector::HEX_GAP_Y,
    selector::HEX_STAGE_X,
    selector::HEX_STAGE_Y,
    selector::GRID_LAYOUT,
    selector::GRID_COLUMNS,
    selector::GRID_ROWS,
    selector::GRID_STAGGER,
    selector::GRID_CYLINDER_BEND,
    selector::GRID_CYLINDER_RADIUS,
    selector::GRID_THUMB_WIDTH,
    selector::GRID_THUMB_HEIGHT,
    selector::GRID_GAP_X,
    selector::GRID_GAP_Y,
    selector::GRID_ROUND_CORNERS,
    selector::GRID_CORNER_RADIUS,
    selector::GRID_STAGE_X,
    selector::GRID_STAGE_Y,
    selector::HAND_MOVE,
    selector::HAND_MOVE_CORKSCREW,
    selector::HAND_MOVE_CASCADE,
    selector::HAND_MOVE_SHUFFLE,
    selector::HAND_MOVE_RIBBON,
    selector::HAND_MOVE_SPIRAL,
    selector::HAND_COUNT,
    selector::HAND_CARD_WIDTH,
    selector::HAND_CARD_HEIGHT,
    selector::HAND_SPREAD,
    selector::HAND_FAN_ANGLE,
    selector::HAND_FAN_ROLL,
    selector::HAND_ARCH,
    selector::HAND_CORNER_RADIUS,
    selector::HAND_SKEW,
    selector::HAND_PERSPECTIVE,
    selector::HAND_TILT,
    selector::HAND_SPEED,
    selector::HAND_RIBBON_AXIS,
    selector::HAND_RIBBONS,
    selector::HAND_CUT,
    selector::HAND_CUT_VARIANCE,
    selector::HAND_GHOSTS,
    selector::HAND_BACKDROP,
    selector::HAND_BACKDROP_BLUR,
    selector::HAND_BOB,
    selector::HAND_REVEAL_FILL,
    selector::HAND_STAGE_X,
    selector::HAND_STAGE_Y,
    selector::SANDY_GRAIN,
    selector::SANDY_RES_SCALE,
    selector::SANDY_LOD_AUTO,
    selector::SANDY_LOD,
    selector::SANDY_CENTER,
    selector::SANDY_SLICE_HEIGHT,
    selector::SANDY_SLICE_WIDTH,
    selector::SANDY_SPACING,
    selector::SANDY_SKEW,
    selector::SANDY_EDGE_SPEED,
    selector::SANDY_DURATION,
    selector::SANDY_BLEND,
    selector::SANDY_STRANDS,
    selector::SANDY_TWIST,
    selector::SANDY_ORBIT,
    selector::SANDY_TURBULENCE,
    selector::SANDY_WAIST,
    selector::SANDY_FRONT,
    selector::SANDY_FAN,
    selector::SANDY_SWAP_STYLE,
    selector::SANDY_ARC,
    selector::SANDY_SWAP_LOOP,
    selector::SANDY_OUTGOING_LIVE,
    selector::SANDY_RING_SIZE,
    selector::SANDY_RING_SPIN,
    selector::SANDY_RING_WAVE,
    selector::SANDY_RING_SOFT,
    selector::SANDY_RING_BLEND,
    selector::SANDY_RING_HOLD,
    selector::SANDY_STAGE_X,
    selector::SANDY_STAGE_Y,
    selector::DEPTH_HEIGHT,
    selector::DEPTH_WIDTH_PX,
    selector::DEPTH_SPACING_PX,
    selector::DEPTH_COUNT,
    selector::DEPTH_FALLOFF_FACTOR,
    selector::DEPTH_CORNERS,
    selector::DEPTH_SKEW,
    selector::DEPTH_NAVIGATION_MS,
    selector::DEPTH_SELECTION_FRAME,
    selector::DEPTH_SHADOWS,
    selector::COLLECTION_SIZE,
    selector::COLLECTION_SPACING,
    selector::COLLECTION_COUNT,
    selector::COLLECTION_TILT,
    selector::COLLECTION_CORNERS,
    selector::COLLECTION_SPEED,
    selector::FRAME_ENABLED,
    selector::FRAME_X,
    selector::FRAME_Y,
    selector::FRAME_WIDTH,
    selector::FRAME_HEIGHT,
    filter_bar::POSITION_ENABLED,
    filter_bar::POSITION_X,
    filter_bar::POSITION_Y,
    filter_bar::OFFSET_X,
    filter_bar::OFFSET_Y,
    filter_bar::ARRANGE_POSITION_ENABLED,
    filter_bar::ARRANGE_POSITION_X,
    filter_bar::ARRANGE_POSITION_Y,
    filter_bar::SWATCH_POSITION_ENABLED,
    filter_bar::SWATCH_POSITION_X,
    filter_bar::SWATCH_POSITION_Y,
    selector::TAG_CLOUD_POSITION_ENABLED,
    selector::TAG_CLOUD_POSITION_X,
    selector::TAG_CLOUD_POSITION_Y,
    selector::TAG_CLOUD_OFFSET_X,
    selector::TAG_CLOUD_OFFSET_Y,
    selector::LIVE_PREVIEW,
    selector::SHOW_TYPE_BADGES,
    video_preview::ENABLED,
    video_preview::DELAY_MS,
    selector::FLIP_EFFECT,
    selector::FLIP_SHADER,
    selector::FLIP_BACK_REVEAL,
    selector::FLIP_DURATION_MS,
    motion::FILTER_SWAP_SPEED,
    selector::START_POSITION,
    launch::ANIMATION,
    motion::LAUNCH_SPEED,
    general::OPEN_FADE_FROM,
    general::CLOSE_ON_SELECTION,
    general::HOVER_SELECTS,
    selector::LOOP_AROUND,
    motion::FAST_MS,
    motion::STANDARD_MS,
    motion::SLOW_MS,
];

pub fn layout_paths() -> impl Iterator<Item = &'static str> {
    let corner_paths = corners::ALL
        .iter()
        .flat_map(|keys| [keys.style, keys.same, keys.all].into_iter().chain(keys.values));
    PLAIN_PATHS.iter().copied().chain(corner_paths)
}
