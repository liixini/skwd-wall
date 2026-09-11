mod constants;
mod helpers;
pub(crate) mod input;
mod lifecycle;
mod message;
mod model;
mod overlay;
mod policy;
mod runtime;
pub(crate) mod scene;
mod settings_source;
mod startup;
mod state;
mod tests;
mod theme_bar;
mod ui_ids;
mod update;
mod view;
pub(crate) mod warm;

pub use crate::contracts::settings::EFFECT_NAMES;
pub use constants::PLAYLIST_DOCK_W;
pub use input::{style, subscription};
pub use message::Message;
pub use model::App;
pub(crate) use state::SearchMode;
pub use update::update;
pub use view::{view, view_single};

pub(crate) use constants::PALETTE_CACHE_CAP;
pub(crate) use helpers::{
    apply_browser_defaults, apply_task, begin_card_tag_edit, browser_key_nav, browser_wall_params,
    commit_mass_tags, commit_pending_tag, ghost_completion, mass_tag_suggestions, open_effects,
    remove_tag_at, sync_card_tag_input, toggle_card_tag_drawer, toggle_favourite,
};
pub(crate) use policy::{
    empty_library_hint, needs_list_refresh, scene_visibility, version_mismatch,
};
pub(crate) use runtime::TOAST_MS;
#[cfg(test)]
pub(crate) use startup::startup_filters;
use startup::take_wake_receiver;
pub(crate) use startup::{layout_params, wake_sender};
pub(crate) use state::PaneScroll;
pub(crate) use state::Pending;
pub(crate) use state::ThemeAuditionPreview;
use state::{
    AppRuntimeState, ChromeState, DaemonState, DemoSession, InputState, LibrarySession,
    PanelsState, PreviewResources, SourceBrowserState, TagState, ThemeState,
};
pub(crate) use theme_bar::swatch_overlay;
pub(crate) use ui_ids::{
    mass_tag_input_id, sync_library_search, tag_input_id, tag_query_id, tag_search_partial,
};
pub(crate) use update::effects_do_preview;
