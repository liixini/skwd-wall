mod catalog;
mod folders;
mod layouts;
mod memory;
mod overview;
mod source;
pub mod wallpaper_kind;

pub use catalog::EFFECT_NAMES;
pub use folders::{ExclusionScope, FolderKind, folder_tag};
pub use layouts::{
    ACTIVE_LAYOUT, DEFAULT_CORNER, PREMADES, Premade, SAVED_LAYOUTS, SavedLayouts, Stored,
    layout_paths,
};
pub use memory::saved_memory_mib;
pub use overview::{Overview, OverviewMemo, Scan, SettingChange, Undo};
pub use skwd_config::schema;
pub use skwd_config::{EVERYTHING_PROFILE, FolderState as ProfileState, parse_rgb};
pub use skwd_config::{corners, keys};

pub const SHOW_TYPE_PREFIX: &str = "filterBar.show.type.";
pub const SHOW_SORT_PREFIX: &str = "filterBar.show.sort.";

pub type ProfileStates = (Vec<ProfileState>, ProfileState);
pub use source::{GraphicsCard, GraphicsTier, SettingsSource};
pub use wall_proto::LockScreenTarget;
pub use wall_proto::drawing;
pub use wall_proto::transitions;
