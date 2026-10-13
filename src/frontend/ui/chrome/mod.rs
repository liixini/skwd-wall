mod back;
mod badges;
mod canvas;
mod panel;
mod selection;
mod signature;

#[cfg(test)]
pub(crate) use back::back_layout_for;
#[cfg(test)]
use back::back_rise;
pub use back::{
    BACK_TOGGLE_GAP, BACK_TOGGLE_INSET, BackLayout, back_bounds, back_contains, back_layout,
};
pub use canvas::ChromeCanvas;
pub use panel::{ChamferPanel, PANEL_SKEW};
pub use selection::SelectionMarks;
pub use signature::chrome_signature;

mod tests;
