mod controls;
mod decode;
mod draw;
mod layout;
mod motion;
mod program;
mod state;
mod text;
mod tokens;
mod update;
mod view;

#[cfg(test)]
pub use controls::Phase;
pub use controls::{Edit, Row, StudioMsg};
pub use decode::decode;
pub use state::{ColourStudio, Decoded, Load, Schemes, Wallpaper};
pub use update::Effect;

#[cfg(test)]
mod render_tests;
