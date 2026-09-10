mod browser;
mod common;
mod diagnostics;
mod effects;
mod library;
mod playlists;
mod presentation;
mod status;

pub use browser::{decode_browser_collections, decode_browser_download, decode_browser_search};
#[allow(unused_imports)]
pub use common::{DecodeError, DecodeResult};
pub use diagnostics::{decode_bug_report, decode_diagnostic, decode_weather};
pub use effects::{decode_effect_operation, decode_effects_list};
pub use library::decode_library_list;
pub use playlists::{
    decode_card_picker_create, decode_card_picker_memberships, decode_playlist_list,
    decode_playlist_members, decode_playlist_outputs,
};
pub use presentation::{
    decode_audio_outputs, decode_current_theme, decode_outputs, decode_running_processes,
    decode_theme_backends, decode_theme_preview, decode_theme_previews,
};
pub use status::{
    decode_library_watch, decode_scene_properties, decode_status, decode_task_list,
    decode_task_status, decode_thumbnail_reset,
};

#[cfg(test)]
mod tests;
