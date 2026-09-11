use crate::domain::input::InputAction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyBindingDescriptor {
    pub action: InputAction,
    pub path: &'static str,
    pub title_key: &'static str,
}

pub const KEY_BINDINGS: [KeyBindingDescriptor; 24] = [
    KeyBindingDescriptor {
        action: InputAction::Select,
        path: skwd_config::keys::keybind::SELECT,
        title_key: "keybind-select",
    },
    KeyBindingDescriptor {
        action: InputAction::Apply,
        path: skwd_config::keys::keybind::APPLY,
        title_key: "keybind-apply",
    },
    KeyBindingDescriptor {
        action: InputAction::Flip,
        path: skwd_config::keys::keybind::FLIP,
        title_key: "keybind-flip",
    },
    KeyBindingDescriptor {
        action: InputAction::Favourite,
        path: skwd_config::keys::keybind::FAVOURITE,
        title_key: "keybind-favourite",
    },
    KeyBindingDescriptor {
        action: InputAction::Effects,
        path: skwd_config::keys::keybind::EFFECTS,
        title_key: "keybind-effects",
    },
    KeyBindingDescriptor {
        action: InputAction::Studio,
        path: skwd_config::keys::keybind::STUDIO,
        title_key: "keybind-studio",
    },
    KeyBindingDescriptor {
        action: InputAction::SceneProperties,
        path: skwd_config::keys::keybind::SCENE_PROPERTIES,
        title_key: "keybind-scene-properties",
    },
    KeyBindingDescriptor {
        action: InputAction::Playlists,
        path: skwd_config::keys::keybind::PLAYLISTS,
        title_key: "keybind-playlists",
    },
    KeyBindingDescriptor {
        action: InputAction::Settings,
        path: skwd_config::keys::keybind::SETTINGS,
        title_key: "keybind-settings",
    },
    KeyBindingDescriptor {
        action: InputAction::Help,
        path: skwd_config::keys::keybind::HELP,
        title_key: "keybind-help",
    },
    KeyBindingDescriptor {
        action: InputAction::TagCloud,
        path: skwd_config::keys::keybind::TAG_CLOUD,
        title_key: "keybind-tag-cloud",
    },
    KeyBindingDescriptor {
        action: InputAction::TagMode,
        path: skwd_config::keys::keybind::TAG_MODE,
        title_key: "keybind-tag-mode",
    },
    KeyBindingDescriptor {
        action: InputAction::FilterBar,
        path: skwd_config::keys::keybind::FILTER_BAR,
        title_key: "keybind-filter-bar",
    },
    KeyBindingDescriptor {
        action: InputAction::FolderPrev,
        path: skwd_config::keys::keybind::FOLDER_PREV,
        title_key: "keybind-folder-prev",
    },
    KeyBindingDescriptor {
        action: InputAction::FolderNext,
        path: skwd_config::keys::keybind::FOLDER_NEXT,
        title_key: "keybind-folder-next",
    },
    KeyBindingDescriptor {
        action: InputAction::FolderToggle,
        path: skwd_config::keys::keybind::FOLDER_TOGGLE,
        title_key: "keybind-folder-toggle",
    },
    KeyBindingDescriptor {
        action: InputAction::HiddenFolders,
        path: skwd_config::keys::keybind::HIDDEN_FOLDERS,
        title_key: "keybind-hidden-folders",
    },
    KeyBindingDescriptor {
        action: InputAction::ColorPrev,
        path: skwd_config::keys::keybind::COLOR_PREV,
        title_key: "keybind-color-prev",
    },
    KeyBindingDescriptor {
        action: InputAction::ColorNext,
        path: skwd_config::keys::keybind::COLOR_NEXT,
        title_key: "keybind-color-next",
    },
    KeyBindingDescriptor {
        action: InputAction::NavLeft,
        path: skwd_config::keys::keybind::NAV_LEFT,
        title_key: "keybind-nav-left",
    },
    KeyBindingDescriptor {
        action: InputAction::NavRight,
        path: skwd_config::keys::keybind::NAV_RIGHT,
        title_key: "keybind-nav-right",
    },
    KeyBindingDescriptor {
        action: InputAction::NavUp,
        path: skwd_config::keys::keybind::NAV_UP,
        title_key: "keybind-nav-up",
    },
    KeyBindingDescriptor {
        action: InputAction::NavDown,
        path: skwd_config::keys::keybind::NAV_DOWN,
        title_key: "keybind-nav-down",
    },
    KeyBindingDescriptor {
        action: InputAction::Autocomplete,
        path: skwd_config::keys::keybind::AUTOCOMPLETE,
        title_key: "keybind-autocomplete",
    },
];
