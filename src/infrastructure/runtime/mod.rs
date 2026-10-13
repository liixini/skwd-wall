mod debug;
mod desktop;
mod frame_clock;
mod wake;

pub use debug::{debug, set_debug};
pub use desktop::{PROFILE_COMMAND, is_niri, startup_panel, startup_profile};
pub use frame_clock::FrameClock;
pub use wake::{Reply, Wake};
