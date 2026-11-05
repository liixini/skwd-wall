use paper_control::{ClockStyle, Face};

use super::state::Tool;
use super::tokens::{GROUP, PLATE};
use crate::domain::strand::Kind;
use crate::frontend::components::Words;
use crate::frontend::ui::cached_line_width;
use crate::i18n::{tr, tr_args};

const CANCEL: &str = "strand-cancel";
const CANCEL_COUNT: &str = "strand-cancel-count";
pub const DONE: &str = "strand-done";
pub const UNDO: &str = "strand-undo";
const REMOVED: &str = "strand-removed";
pub const RELEASE: &str = "strand-release";
pub const ARRANGING: &str = "strand-arranging";
const BEAD: &str = "strand-bead";
const BEAD_PLACED: &str = "strand-bead-placed";
const PERCENT: &str = "strand-percent";
const ANCHOR: &str = "strand-anchor";
const ROWS: [&str; 3] = ["strand-row-top", "strand-row-middle", "strand-row-bottom"];
const COLUMNS: [&str; 3] = ["strand-column-left", "strand-column-centre", "strand-column-right"];
const GROUP_COUNT: &str = "strand-group-count";
const PERCENT_SCALE: f32 = 100.0;

pub fn kind(kind: Kind) -> &'static str {
    tr(match kind {
        Kind::Clock => "strand-kind-clock",
    })
}

pub fn style(face: Face) -> &'static str {
    tr(match face {
        Face::Clock(ClockStyle::Aperture) => "strand-style-aperture",
        Face::Clock(ClockStyle::ApertureRing) => "strand-style-aperture-ring",
        Face::Clock(ClockStyle::Greeter) => "strand-style-greeter",
        Face::Clock(ClockStyle::ControlCenter) => "strand-style-control-center",
        Face::Clock(ClockStyle::DayDial) => "strand-style-day-dial",
    })
}

pub fn tool(tool: Tool, face: Face) -> &'static str {
    match tool {
        Tool::Behind => tr("strand-behind"),
        Tool::Front => tr("strand-front"),
        Tool::Raise => tr("strand-raise"),
        Tool::Lower => tr("strand-lower"),
        Tool::Prev => tr("strand-previous"),
        Tool::Style => style(face),
        Tool::Next => tr("strand-next"),
        Tool::Remove => tr("strand-remove"),
    }
}

pub fn cancel(count: usize) -> String {
    if count == 0 {
        tr(CANCEL).to_owned()
    } else {
        tr_args!(CANCEL_COUNT, count => count.to_string())
    }
}

pub fn removed(face: Face) -> String {
    tr_args!(REMOVED, kind => kind(Kind::of(face)))
}

pub fn bead(face: Face, placed: bool) -> String {
    let key = if placed { BEAD_PLACED } else { BEAD };
    tr_args!(key, kind => kind(Kind::of(face)), style => style(face))
}

pub fn percent(share: f32) -> String {
    tr_args!(PERCENT, value => ((share * PERCENT_SCALE).round() as i32).to_string())
}

pub fn group(count: usize, styles: usize) -> String {
    tr_args!(GROUP_COUNT, count => count.to_string(), styles => styles.to_string())
}

pub fn anchor(row: usize, column: usize, offset: [f32; 2]) -> String {
    let signed = |value: f32| format!("{:+}", value.round() as i32);
    tr_args!(
        ANCHOR,
        row => tr(ROWS[row]),
        column => tr(COLUMNS[column]),
        x => signed(offset[0]),
        y => signed(offset[1])
    )
}

pub fn shown(label: &str) -> String {
    label.to_uppercase()
}

pub fn plate_width(label: &str) -> f32 {
    let shown = shown(label);
    let words = Words::new(&shown, iced::Point::ORIGIN, PLATE.text, iced::Color::BLACK)
        .tracked(PLATE.tracking);
    words.advance().ceil() + PLATE.pad
}

pub fn group_width(styles: usize) -> f32 {
    let widest = group(styles, styles);
    cached_line_width(&widest, PLATE.text, crate::frontend::ui::UI_FONT).ceil()
        + GROUP.height
        + GROUP.pad
}
