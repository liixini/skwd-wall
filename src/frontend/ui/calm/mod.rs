mod options;
mod page;
mod popover;
#[cfg(test)]
pub(crate) mod render_tests;
mod reveal;
mod sentence;
pub mod style;
mod switch;
mod tint;
mod toggle;

pub use options::{OptionsSpec, options_button};
pub use page::page;
pub use popover::{Popover, Side};
pub use reveal::{GROW, PlaneSpec, RevealSpec, StaggerSpec, plane, reveal, stagger};
pub use sentence::{
    ASIDE_GAP, KeyLook, PLAIN_LEADING, Piece, SAY_LEADING, SAY_SIZE, Scrub, SentenceSpec, Token,
    VALUE_CLOSE, VALUE_OPEN, is_sentinel, key_width, prose, sentence, sentinel, split, worded,
};
pub use tint::{ROW_TINT, SELECTED_TINT, tint};
pub use toggle::{ToggleSpec, wipe_toggle};

#[cfg(test)]
pub(crate) use sentence::laid;
