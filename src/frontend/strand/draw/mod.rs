mod compose;
pub(crate) mod selection;
pub(crate) mod shapes;
mod stage;
pub(crate) mod thread;

pub use compose::{PARTS, Part, draw};
pub use stage::stage_alpha;
