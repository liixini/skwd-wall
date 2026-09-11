use crate::contracts::settings::keys;

use super::tables::{NIRI_SNIPPET, POST_TYPES, motion_speed_options};
use super::{ActionId, Builder, Card, Control, PRESET_NAME_KEY, Row};

mod builder;
mod launch_tab;
mod media_tabs;
mod motion_tab;
mod position_tab;
mod selector_tab;
mod source_defaults;
mod system_tabs;
mod theme_tabs;
mod transition_tab;

pub use builder::build_tab;
#[cfg(test)]
pub(crate) use builder::build_tab_with_output_statuses;
#[cfg(test)]
pub(crate) use builder::build_tab_with_outputs;
pub(crate) use builder::build_tab_with_runtime_status;

mod tests;
