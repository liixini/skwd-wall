mod action;
mod automation;
pub mod background;
mod bar;
mod builder;
pub(crate) mod category;
mod choreography;
pub mod colour;
mod colours;
mod corners;
mod folio;
mod home;
mod input_ids;
mod keybinds;
mod layout;
pub(crate) mod layout_place;
mod library;
mod model;
pub mod place;
mod places;
mod search;
mod sources;
mod system;
mod tables;
mod tabs;
#[cfg(test)]
pub(crate) mod test_source;
mod wallpapers;

pub use action::{
    ActionId, LayoutMsg, LayoutPending, LayoutsUi, NudgeSpec, SettingsFocus, SettingsKey,
    SettingsMsg, WEATHER_REVIEW_TAB_COUNT, WeatherReviewTab,
};
pub(crate) use choreography::SETTINGS_MOTION;
pub(crate) use folio::number_editor::{number_step_text, scrub_bounds, step_number};
pub use folio::{
    CalmCtx, ChromeCtx, FocusCtx, FoldersCtx, ProcessPickerView, ProfilesCtx, STRIP_STOPS,
    SourceCtx, StripEntry, TokenSide, WorkbenchInput, app_key, busy, live_key, panel_actions,
    picker_layout_workbench, reveal_line, reveal_result, reveal_row, reveal_section,
    settings_workbench, strip_entries, strip_key, token_key, token_side,
};
pub use folio::{WeatherReviewItem, WeatherReviewView};
#[cfg(test)]
pub use home::setting_changes;
pub use home::{LEAD as HOME_LEAD, fresh_changes};
pub use input_ids::{
    settings_reading_id, settings_search_input_id, weather_review_input_id, workbench_input_id,
};
pub use keybinds::{
    KEYS_SHEET, KeysCtx, conflict_links, keys_find_id, links_control, second_keys_row,
    second_keys_token, undo_row,
};
pub use layout::picker_layout_studio_width;
pub use layout_place::{LAYOUT_NAME_KEY, PREMADE_KEY};
pub(crate) use library::profile_page;
pub use library::{
    FILTER_KEY, FOLDER_LIST, PageCtx, QUERY_FOLD, TREE_LIST, draft_key, folder_draft, page_rows,
    state_position,
};
pub use model::PageSnapshot;
pub use model::{
    Card, Cell, Control, DIMENSIONS, FolderLedger, FolderRow, Form, Group, GroupKind, Look, More,
    Note, Parked, Place, ProfileEntry, ProfilesPage, Row, Say, Section, SectionId, Slot, TagTone,
    TreeNode, Trim, mark_folds, masked,
};
pub use places::{
    PLACE_COUNT, PlaceSource, canonical_place, legacy_section, place_keys, place_title, tokenable,
};
#[cfg(test)]
pub use search::search_settings;
pub use search::{SettingsSearchResult, reread_settings, search_hits};
pub(crate) use tables::SHADERS;
pub(crate) use tables::is_transition_preview_section;
pub use tables::{
    DRAWING_LIBRARY_SCOPE, DRAWING_SCOPE_KEY, PageSection, SHADER_FAMILY_KEY, WEATHER_CONDITIONS,
    WEATHER_MODE_KEY, WEATHER_OFF, WEATHER_REVIEW_KEY, family_default, is_picker_layout_section,
    page_section,
};
#[cfg(test)]
pub use tabs::build_tab;
pub(crate) use tabs::build_tab_with_runtime_status;
pub(crate) use tabs::following_outdoors;
pub(crate) use tabs::{DRAWING_PACK_INSTALL, DRAWING_VIEW_EMPTY, DRAWING_VIEW_LIMIT};
pub(crate) use tabs::{DrawingJob, RuntimeStatus};
pub use tabs::{
    FOLDER_SEGMENT, SEGMENT_PREFIX, STATES, WORKSHOP_TAG, flash_key, folder_ledger,
    profile_commands, profiles_page, segment_key,
};
pub(crate) use wallpapers::DRAWING as DRAWING_SECTION;

use builder::Builder;

pub(crate) use tables::NIRI_SNIPPET;
