mod catalog;
mod folders;
mod source;
pub mod wallpaper_kind;

pub use catalog::EFFECT_NAMES;
pub use folders::{ExclusionScope, FolderKind, folder_tag};
pub use skwd_config::keys;
pub use skwd_config::schema;
pub use skwd_config::{EVERYTHING_PROFILE, FolderState as ProfileState};

pub type ProfileStates = (Vec<ProfileState>, ProfileState);
pub use source::{GraphicsCard, GraphicsTier, SettingsSource};
pub use wall_proto::LockScreenTarget;
pub use wall_proto::transitions;
