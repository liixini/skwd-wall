use crate::contracts::settings::keys;

use super::tables::{NIRI_SNIPPET, POST_TYPES, motion_speed_options};
use super::{ActionId, Builder, Card, Control, Row};

mod app_themes;
mod builder;
mod folders_tab;
pub use folders_tab::{flash_key, folder_ledger};
mod launch_tab;
mod media_tabs;
mod position_tab;
mod profiles_tab;
pub use profiles_tab::{
    FOLDER_SEGMENT, SEGMENT_PREFIX, STATES, WORKSHOP_TAG, profile_commands, profiles_page,
    segment_key,
};
mod selector_tab;
mod source_defaults;
mod system_tabs;
pub(in crate::frontend::settings) use system_tabs::{WEATHER_CATEGORIES, WEATHER_QUERIES};
pub(crate) use system_tabs::{following_outdoors, weather_category_label};
mod theme_tabs;
mod transition_tab;
pub(crate) use transition_tab::{
    DRAWING_PACK_INSTALL, DRAWING_SKIPPED, DRAWING_VIEW_EMPTY, DRAWING_VIEW_LIMIT,
};

#[cfg(test)]
pub use builder::build_tab;
pub(crate) use builder::build_tab_with_runtime_status;
pub(crate) use builder::{DrawingJob, RuntimeStatus};
pub(in crate::frontend::settings) use builder::{
    asleep_rows, colours_asleep_rows, layout_asleep_rows, niri_rows, plasma_lock_rows,
};
pub(super) use builder::{library_watch_label, take_card, take_control_paths};

mod tests;
