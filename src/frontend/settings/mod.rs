mod action;
mod builder;
mod folio;
mod input_ids;
mod layout;
mod model;
mod search;
mod tables;
mod tabs;
#[cfg(test)]
pub(crate) mod test_source;

pub use action::{ActionId, SettingsFocus, SettingsKey, SettingsMsg};
pub use folio::{
    ChromeCtx, FocusCtx, KeybindCaptureView, SourceCtx, WorkbenchInput, picker_layout_workbench,
    settings_workbench,
};
pub use input_ids::{PRESET_NAME_KEY, settings_search_input_id, workbench_input_id};
pub use layout::picker_layout_studio_width;
pub use model::{Card, Control, Row};
pub use search::{SettingsSearchResult, search_settings};
pub(crate) use tables::SHADERS;
pub(crate) use tables::is_transition_preview_section;
pub use tables::{
    SHADER_FAMILY_KEY, canonical_category, family_default, is_picker_layout_section, visible_tabs,
};
pub use tabs::build_tab;
pub(crate) use tabs::build_tab_with_runtime_status;

use builder::Builder;
