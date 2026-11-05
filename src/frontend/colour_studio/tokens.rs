use iced::{Point, Rectangle, Size};

pub const SCREEN: Size = Size::new(1366.0, 768.0);
pub const WINDOW: Rectangle = Rectangle { x: 14.0, y: 14.0, width: 1338.0, height: 740.0 };
pub const INDEX_WIDTH: f32 = 220.0;
pub const CONTENT_LEFT: f32 = 240.0;
pub const CONTENT_RIGHT: f32 = 1314.0;

pub const HEADER_TOP: f32 = 16.0;
pub const HEADER_HEIGHT: f32 = 28.0;
pub const CLOSE_RIGHT: f32 = 18.0;

pub const PREVIEW_TOP: f32 = 68.0;
pub const PREVIEW_BOTTOM: f32 = 724.0;
pub const PREVIEW_HEIGHT: f32 = PREVIEW_BOTTOM - PREVIEW_TOP;
pub const PREVIEW_WIDTH: f32 = PREVIEW_HEIGHT * crate::domain::studio::PREVIEW_ASPECT;
pub const PREVIEW: Rectangle = Rectangle {
    x: CONTENT_LEFT + (CONTENT_RIGHT - CONTENT_LEFT - PREVIEW_WIDTH) / 2.0,
    y: PREVIEW_TOP,
    width: PREVIEW_WIDTH,
    height: PREVIEW_HEIGHT,
};
pub const PREVIEW_EDGE_GAP: f32 = 4.0;
pub const PREVIEW_EDGE: Rectangle = Rectangle {
    x: PREVIEW.x - PREVIEW_EDGE_GAP,
    y: PREVIEW.y - PREVIEW_EDGE_GAP,
    width: PREVIEW.width + 2.0 * PREVIEW_EDGE_GAP,
    height: PREVIEW.height + 2.0 * PREVIEW_EDGE_GAP,
};
pub const PREVIEW_EDGE_RADIUS: f32 = 3.0;
pub const LABEL_LINE: f32 = 1.17;

pub const STRIP_SIZE: f32 = 10.0;
pub const STRIP_PITCH: f32 = 18.0;
pub const STRIP_HEIGHT: f32 = 20.0;
pub const STRIP_INSET: f32 = 14.0;
pub const STRIP_EDGE: f32 = 1.5;

pub const TABS: Point = Point::new(232.0, 8.0);
pub const TAB_WIDTHS: [f32; 7] = [118.0, 110.0, 110.0, 110.0, 110.0, 100.0, 126.0];
pub const TAB_HEIGHT: f32 = 48.0;
pub const TAB_PAD: f32 = 18.0;
pub const TAB_LABEL_TOP: f32 = 6.0;
pub const TAB_LABEL_LINE: f32 = 18.0;
pub const TAB_LABEL_SIZE: f32 = 14.0;
pub const TAB_VALUE_TOP: f32 = 25.0;
pub const TAB_VALUE_LINE: f32 = 16.0;
pub const TAB_VALUE_SIZE: f32 = 12.5;
pub const TAB_STEP_SIZE: f32 = 13.5;
pub const MARKER_LEFT: f32 = 6.0;
pub const MARKER_TOP: f32 = 11.0;
pub const MARKER_SIZE: f32 = 8.0;

pub const POP_GAP: f32 = 6.0;
pub const POP_PAD_TOP: f32 = 12.0;
pub const POP_PAD_SIDE: f32 = 18.0;
pub const POP_PAD_BOTTOM: f32 = 16.0;
pub const POP_ROW_GAP: f32 = 12.0;
pub const POP_CUT: f32 = 12.0;
pub const POP_DETAIL_SIZE: f32 = 12.5;
pub const POP_DETAIL_LINE: f32 = 16.0;
pub const POP_SLIDER: f32 = 280.0;
pub const POP_NODES: f32 = 200.0;
pub const POP_CONTROL: f32 = 20.0;
pub const POP_PLATE_GAP: f32 = 4.0;
pub const POP_SHIFT: f32 = 6.0;
pub const POP_LINGER: f32 = 1.6;
pub const POP_LEAVE: f32 = 0.45;

pub const TRACK_ALPHA: f32 = 0.6;
pub const TRACK_REACH_X: f32 = 6.0;
pub const TRACK_REACH_Y: f32 = 10.0;
pub const TRACK_ON: f32 = 2.0;
pub const TRACK_ON_RISE: f32 = 0.5;
pub const TICK_SIZE: f32 = 6.0;
pub const KNOB_SIZE: f32 = 8.5;
pub const KNOB_GRAB: f32 = 1.25;
pub const KNOB_GAP: f32 = 8.5;
pub const TICK_GAP: f32 = 6.0;
pub const NODE_BOX: f32 = 18.0;
pub const NODE_SIZE: f32 = 9.0;
pub const NODE_PICKED: f32 = 11.0;
pub const NODE_GAP: f32 = 7.0;
pub const SWITCH_WIDTH: f32 = 58.0;
pub const SWITCH_KNOB: f32 = 10.0;
pub const SWITCH_KNOB_EDGE: f32 = 1.5;
pub const SWITCH_END: f32 = 6.0;
pub const SWITCH_END_AT: f32 = 50.0;
pub const SWITCH_ON_AT: f32 = 50.0;
pub const SWITCH_OFF_AT: f32 = 8.0;
pub const SWITCH_WIRE_ON: (f32, f32) = (8.0, 16.0);
pub const SWITCH_WIRE_OFF: (f32, f32) = (16.0, 13.2);

pub const BUTTON_GAP: f32 = 8.0;
pub const ACTIONS_PAD: f32 = 14.0;
pub const NAME_INPUT: f32 = 124.0;

pub const NAV_TOP: f32 = 10.0;
pub const SECTION_HEIGHT: f32 = 40.0;
pub const SECTION_LEFT: f32 = 40.0;
pub const SECTION_RIGHT: f32 = 14.0;
pub const SECTION_GAP: f32 = 10.0;
pub const SECTION_SIZE: f32 = 14.0;
pub const KID_HEIGHT: f32 = 30.0;
pub const KID_STEP: f32 = 34.0;
pub const KID_LEFT: f32 = 56.0;
pub const KID_RIGHT: f32 = 14.0;
pub const KID_PANE_RIGHT: f32 = 22.0;
pub const KID_GAP: f32 = 8.0;
pub const KID_SIZE: f32 = 13.5;
pub const KID_LINE: f32 = 1.1;
pub const KID_DOT: f32 = 6.0;
pub const KID_DOT_GAP: f32 = 6.0;
pub const KID_DOTS: usize = 3;
pub const RAIL_LEFT: f32 = 36.0;
pub const RAIL_ALPHA: f32 = 0.4;
pub const NODE_LEFT: f32 = 33.0;
pub const NAV_NODE: f32 = 6.0;
pub const NAV_NODE_EDGE: f32 = 1.0;
pub const SECTION_MARK_LEFT: f32 = 14.0;
pub const KID_MARK_LEFT: f32 = 32.0;
pub const NAV_MARK: f32 = 8.0;
pub const GROUP_GAP: f32 = 10.0;
pub const SAVED_GAP: f32 = 8.0;
pub const PANE_HEIGHT: f32 = 272.0;
pub const PANE_FADE: f32 = 26.0;
pub const PANE_BAR_RIGHT: f32 = 7.0;
pub const PANE_BAR_INSET: f32 = 10.0;
pub const PANE_BAR_ALPHA: f32 = 0.45;
pub const PANE_THUMB: f32 = 7.0;
pub const REVEAL_MARGIN: f32 = 20.0;

pub const MODES_FLOOR: f32 = 466.0;
pub const MODES_GAP: f32 = 14.0;
pub const MODES_LEFT: f32 = 24.0;
pub const MODES_LABEL_SIZE: f32 = 12.5;
pub const MODES_ROW_GAP: f32 = 8.0;
pub const MODES_PLATE_GAP: f32 = 4.0;
pub const WHEEL_LEFT: f32 = 20.0;
pub const WHEEL_BOX: f32 = 180.0;
pub const WHEEL_BELOW_MODES: f32 = 64.0;
pub const WHEEL_MAX: f32 = 170.0;
pub const WHEEL_MIN: f32 = 110.0;
pub const WHEEL_CAPTION: f32 = 56.0;
pub const WHEEL_CAPTION_GAP: f32 = 6.0;
pub const WHEEL_CAPTION_SIZE: f32 = 11.5;
pub const WHEEL_CAPTION_LINE: f32 = 1.35;
pub const INDEX_BOTTOM: f32 = 736.0;

pub const PLATE_HEIGHT: f32 = 28.0;
pub const PLATE_SMALL: f32 = 22.0;
pub const PLATE_PAD: f32 = 18.0;
pub const PLATE_SMALL_PAD: f32 = 13.0;
pub const PLATE_TEXT: f32 = 11.0;
pub const PLATE_RIM: f32 = 1.5;
pub const PLATE_HOVER: f32 = 0.8;
pub const PLATE_PICKED_HOVER: f32 = 0.86;
pub const PLATE_FOCUS: f32 = 0.72;
pub const PLATE_PRESS: f32 = 0.97;
pub const PLATE_SHAPE_AT: f32 = 0.3;
pub const PLATE_TEXT_DELAY: f32 = 0.7;

pub const TAG_INSET: f32 = 8.0;
pub const NOW_TAG_FROM: f32 = 0.13;
pub const NEW_TAG_UNTIL: f32 = 0.82;
pub const CUT_WIDTH: f32 = 2.0;
pub const CUT_SHADOW: f32 = 1.0;
pub const GRIP_SIZE: f32 = 12.0;
pub const GRIP_GRAB: f32 = 1.25;
pub const SPLIT_RANGE: (f32, f32) = (0.02, 0.98);
pub const SPLIT_GRAB: f32 = 12.0;
pub const DRAG_SLOP: f32 = 3.0;
pub const PICK_MIN: f32 = 12.0;
pub const PICK_EDGE: f32 = 1.5;
pub const PICK_GHOST_ALPHA: f32 = 0.6;
pub const PICK_DASH: f32 = 4.5;

pub const INDEX_TINT: f32 = 0.4;
pub const OUTLINE_MIX: f32 = 0.25;
pub const WINDOW_EDGE_ALPHA: f32 = 0.45;
pub const FRAME_ALPHA: f32 = 0.55;
pub const SHADOW_DROP: f32 = 20.0;
pub const SHADOW_BLUR: f32 = 50.0;
pub const SHADOW_ALPHA: f32 = 0.53;

pub const LIFT: f32 = 6.0;
pub const SLIDE_IN: f32 = 18.0;
pub const POP_FROM: f32 = 0.6;
pub const TURN_FROM: f32 = 14.0;
pub const SWAP_Y: f32 = 4.0;
pub const SWAP_X: f32 = 10.0;
pub const SWAP_DELAY: f32 = 0.06;

pub const ACTIONS_DELAY: f32 = 0.1;
pub const SECTION_DELAY: f32 = 0.18;
pub const KID_DELAY: (f32, f32) = (0.04, 0.025);
pub const NODE_DELAY: (f32, f32) = (0.06, 0.025);
pub const SAVED_DELAY: f32 = 0.03;
pub const MARK_DELAY: f32 = 0.3;
pub const STRIP_DELAY: f32 = 0.6;
pub const WHEEL_DELAY: f32 = 0.12;
pub const WHEEL_NOTE_DELAY: f32 = 0.45;
pub const WHEEL_DOT_DELAY: f32 = 0.35;
pub const MODES_DELAY: f32 = 0.4;
pub const BAND_DELAY: f32 = 0.25;
pub const BAND_MARK_DELAY: f32 = 0.3;
pub const TAB_DELAY: (f32, f32) = (0.1, 0.03);
pub const SEGMENT_DELAY: (f32, f32) = (0.25, 0.02);
pub const NODE_SEGMENT_DELAY: f32 = 0.2;

pub const INTRO_ZOOM: f32 = 0.75;
pub const INTRO_END: f32 = 2.4;
pub const DESK_DRAW: f32 = 0.62;
pub const DESK_LINE: f32 = 0.4;
pub const DESK_RIM: f32 = 1.0;
pub const DESK_RIM_HOLD: f32 = 0.55;
pub const DESK_AT: f32 = 0.8;
pub const DESK_STEP: f32 = 0.15;
pub const DESK_RIM_WIDTH: f32 = 1.5;
pub const CUT_INTRO: f32 = 1.35;
pub const TAG_INTRO: f32 = 1.4;

pub const WHEEL_QUIET: f32 = 0.24;
pub const WHEEL_NOTCH: f32 = 20.0;
pub const WHEEL_STEP_GATE: f32 = 40.0;
pub const WHEEL_STEP_GAP: f32 = 0.14;
