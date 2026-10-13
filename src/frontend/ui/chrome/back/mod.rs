mod actions;
mod background;
mod content;
mod geometry;
mod layout;
mod tests;
mod view;

pub use geometry::{back_bounds, back_contains};
#[cfg(test)]
pub use layout::back_layout_for;
#[cfg(test)]
pub(super) use layout::back_rise;
pub use layout::{
    BackLayout, TOGGLE_GAP as BACK_TOGGLE_GAP, TOGGLE_INSET as BACK_TOGGLE_INSET, back_layout,
};
pub(super) use view::draw_back;
