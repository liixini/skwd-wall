mod apps;
mod bar;
mod compose;
mod hatch;
mod mock;
use crate::frontend::components::pen;
mod nav;
mod pick;
mod preview;
mod shapes;
mod syntax;
mod wheel;
mod window;

pub use bar::{fraction, node_x, options, pop, pop_offset, tab_needs, track, value_rect};
pub use nav::{Pick, kids, mode_plates, pane_rect, saved_kids};
pub use pen::{Ink, Pen, Sheet};
pub use window::{Chrome, actions, backing, header_limit};

pub use compose::{PARTS, Part, entering, faint, hover, studio, touch};
