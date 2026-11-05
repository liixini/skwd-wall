use std::borrow::Cow;

use super::model::{CornerStep, DisplayRecipient, Family, Style, Tool};
use super::state::Extra;
use crate::domain::strand::Rect;
use crate::i18n::{tr, tr_args};

pub(crate) const RELEASE: &str = "arrange-release";
pub(crate) const BACK: &str = "arrange-back-editing";
const CORNERS_EACH: &str = "arrange-corners-each";
const SPACING: [&str; 3] =
    ["arrange-spacing-none", "arrange-spacing-default", "arrange-spacing-roomy"];
const CORNERS: [&str; 5] = [
    "arrange-corners-square",
    "arrange-corners-soft",
    "arrange-corners-round",
    "arrange-corners-soft-cut",
    "arrange-corners-large-cut",
];
const ROWS: [&str; 3] = ["strand-row-top", "strand-row-middle", "strand-row-bottom"];
const COLUMNS: [&str; 3] = ["strand-column-left", "strand-column-centre", "strand-column-right"];

pub(crate) fn family(family: Family) -> &'static str {
    tr(match family {
        Family::Types => "arrange-family-types",
        Family::Sort => "arrange-family-sort",
        Family::Colours => "arrange-family-colours",
        Family::Tools => "arrange-family-tools",
    })
}

pub(crate) fn extra(extra: Extra) -> &'static str {
    tr(match extra {
        Extra::Preview => "arrange-preview",
        Extra::Displays => "arrange-displays",
    })
}

pub(crate) fn tool(tool: Tool, style: &Style) -> Cow<'static, str> {
    match tool {
        Tool::Prev => Cow::Borrowed(tr("strand-previous")),
        Tool::Layout => Cow::Borrowed(style.layout),
        Tool::Next => Cow::Borrowed(tr("strand-next")),
        Tool::Spacing => Cow::Owned(tr_args!(
            "arrange-spacing-value",
            value => tr(SPACING[style.spacing.unwrap_or_default()])
        )),
        Tool::Corners => {
            let value = match style.corners {
                Some(CornerStep::Step(at)) => CORNERS[at],
                _ => CORNERS_EACH,
            };
            Cow::Owned(tr_args!("arrange-corners-value", value => tr(value)))
        }
        Tool::Hide => Cow::Borrowed(tr("arrange-hide")),
        Tool::Reset => Cow::Borrowed(tr("arrange-reset")),
    }
}

pub(crate) fn bead(family_name: Family, title: &str, shown: bool) -> String {
    let key = if shown { "arrange-bead-shown" } else { "arrange-bead" };
    tr_args!(key, family => family(family_name), button => title)
}

pub(crate) fn hidden(title: &str) -> String {
    tr_args!("arrange-hidden-toast", button => title)
}

pub(crate) fn size(rect: Rect) -> String {
    tr_args!(
        "arrange-size",
        width => (rect.w.round() as i32).to_string(),
        height => (rect.h.round() as i32).to_string()
    )
}

pub(crate) fn cells(cells: [usize; 2]) -> String {
    tr_args!("arrange-size", width => cells[0].to_string(), height => cells[1].to_string())
}

pub(crate) fn anchor(picker: bool, cell: [usize; 2], offset: [f32; 2]) -> String {
    let signed = |value: f32| format!("{:+}", value.round() as i32);
    let key = if picker { "arrange-anchor-picker" } else { "strand-anchor" };
    tr_args!(
        key,
        row => tr(ROWS[cell[1]]),
        column => tr(COLUMNS[cell[0]]),
        x => signed(offset[0]),
        y => signed(offset[1])
    )
}

pub(crate) fn recipient(recipient: &DisplayRecipient) -> String {
    let state = if recipient.staged {
        "arrange-display-staged"
    } else if recipient.connected {
        "arrange-display-connected"
    } else {
        "arrange-display-disconnected"
    };
    tr_args!("arrange-display-entry", output => recipient.name.as_str(), state => tr(state))
}

pub(crate) fn disconnected(outputs: &[String]) -> String {
    tr_args!("arrange-disconnected-label", outputs => outputs.join(", "))
}

pub(crate) fn layout_name(mode: &str) -> &'static str {
    tr(match mode {
        "depth" => "arrange-mode-depth",
        "collection" => "arrange-mode-collection",
        "wall" | "grid" => "arrange-mode-wall",
        "hex" => "arrange-mode-hex",
        "sandy" => "arrange-mode-sandy",
        "hand" => "arrange-mode-hand",
        _ => "arrange-mode-slices",
    })
}
