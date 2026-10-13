use crate::domain::input::InputAction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyBindingDescriptor {
    pub action: InputAction,
    pub path: &'static str,
    pub title_key: &'static str,
    pub group: KeyBindingGroup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyBindingGroup {
    Wallpaper,
    Panels,
    Filters,
    Folders,
    Navigation,
    Downloads,
}

impl KeyBindingGroup {
    pub const ALL: [Self; 6] = [
        Self::Wallpaper,
        Self::Panels,
        Self::Filters,
        Self::Folders,
        Self::Navigation,
        Self::Downloads,
    ];

    pub const fn title_key(self) -> &'static str {
        match self {
            Self::Wallpaper => "settings-keybinds-wallpaper-card",
            Self::Panels => "settings-keybinds-panels-card",
            Self::Filters => "settings-keybinds-filters-card",
            Self::Folders => "settings-keybinds-folders-card",
            Self::Navigation => "settings-keybinds-navigation-card",
            Self::Downloads => "settings-keybinds-downloads-card",
        }
    }

    pub const fn description_key(self) -> &'static str {
        match self {
            Self::Wallpaper => "settings-keybinds-wallpaper-card-desc",
            Self::Panels => "settings-keybinds-panels-card-desc",
            Self::Filters => "settings-keybinds-filters-card-desc",
            Self::Folders => "settings-keybinds-folders-card-desc",
            Self::Navigation => "settings-keybinds-navigation-card-desc",
            Self::Downloads => "settings-keybinds-downloads-card-desc",
        }
    }
}

pub const KEY_BINDINGS: [KeyBindingDescriptor; 46] = [
    KeyBindingDescriptor {
        action: InputAction::Select,
        path: skwd_config::keys::keybind::SELECT,
        title_key: "keybind-select",
        group: KeyBindingGroup::Wallpaper,
    },
    KeyBindingDescriptor {
        action: InputAction::Apply,
        path: skwd_config::keys::keybind::APPLY,
        title_key: "keybind-apply",
        group: KeyBindingGroup::Wallpaper,
    },
    KeyBindingDescriptor {
        action: InputAction::RandomApply,
        path: skwd_config::keys::keybind::RANDOM_APPLY,
        title_key: "keybind-random-apply",
        group: KeyBindingGroup::Wallpaper,
    },
    KeyBindingDescriptor {
        action: InputAction::Flip,
        path: skwd_config::keys::keybind::FLIP,
        title_key: "keybind-flip",
        group: KeyBindingGroup::Wallpaper,
    },
    KeyBindingDescriptor {
        action: InputAction::Reveal,
        path: skwd_config::keys::keybind::REVEAL,
        title_key: "keybind-reveal",
        group: KeyBindingGroup::Wallpaper,
    },
    KeyBindingDescriptor {
        action: InputAction::Favourite,
        path: skwd_config::keys::keybind::FAVOURITE,
        title_key: "keybind-favourite",
        group: KeyBindingGroup::Wallpaper,
    },
    KeyBindingDescriptor {
        action: InputAction::Effects,
        path: skwd_config::keys::keybind::EFFECTS,
        title_key: "keybind-effects",
        group: KeyBindingGroup::Wallpaper,
    },
    KeyBindingDescriptor {
        action: InputAction::Studio,
        path: skwd_config::keys::keybind::STUDIO,
        title_key: "keybind-studio",
        group: KeyBindingGroup::Wallpaper,
    },
    KeyBindingDescriptor {
        action: InputAction::SceneProperties,
        path: skwd_config::keys::keybind::SCENE_PROPERTIES,
        title_key: "keybind-scene-properties",
        group: KeyBindingGroup::Wallpaper,
    },
    KeyBindingDescriptor {
        action: InputAction::Playlists,
        path: skwd_config::keys::keybind::PLAYLISTS,
        title_key: "keybind-playlists",
        group: KeyBindingGroup::Panels,
    },
    KeyBindingDescriptor {
        action: InputAction::Settings,
        path: skwd_config::keys::keybind::SETTINGS,
        title_key: "keybind-settings",
        group: KeyBindingGroup::Panels,
    },
    KeyBindingDescriptor {
        action: InputAction::Help,
        path: skwd_config::keys::keybind::HELP,
        title_key: "keybind-help",
        group: KeyBindingGroup::Panels,
    },
    KeyBindingDescriptor {
        action: InputAction::ThemePanel,
        path: skwd_config::keys::keybind::THEME_PANEL,
        title_key: "keybind-theme-panel",
        group: KeyBindingGroup::Panels,
    },
    KeyBindingDescriptor {
        action: InputAction::TagCloud,
        path: skwd_config::keys::keybind::TAG_CLOUD,
        title_key: "keybind-tag-cloud",
        group: KeyBindingGroup::Filters,
    },
    KeyBindingDescriptor {
        action: InputAction::TagMode,
        path: skwd_config::keys::keybind::TAG_MODE,
        title_key: "keybind-tag-mode",
        group: KeyBindingGroup::Filters,
    },
    KeyBindingDescriptor {
        action: InputAction::FilterBar,
        path: skwd_config::keys::keybind::FILTER_BAR,
        title_key: "keybind-filter-bar",
        group: KeyBindingGroup::Filters,
    },
    KeyBindingDescriptor {
        action: InputAction::FolderPrev,
        path: skwd_config::keys::keybind::FOLDER_PREV,
        title_key: "keybind-folder-prev",
        group: KeyBindingGroup::Folders,
    },
    KeyBindingDescriptor {
        action: InputAction::FolderNext,
        path: skwd_config::keys::keybind::FOLDER_NEXT,
        title_key: "keybind-folder-next",
        group: KeyBindingGroup::Folders,
    },
    KeyBindingDescriptor {
        action: InputAction::FolderToggle,
        path: skwd_config::keys::keybind::FOLDER_TOGGLE,
        title_key: "keybind-folder-toggle",
        group: KeyBindingGroup::Folders,
    },
    KeyBindingDescriptor {
        action: InputAction::ShowHidden,
        path: skwd_config::keys::keybind::HIDDEN_FOLDERS,
        title_key: "keybind-hidden-folders",
        group: KeyBindingGroup::Folders,
    },
    KeyBindingDescriptor {
        action: InputAction::ColorPrev,
        path: skwd_config::keys::keybind::COLOR_PREV,
        title_key: "keybind-color-prev",
        group: KeyBindingGroup::Filters,
    },
    KeyBindingDescriptor {
        action: InputAction::ColorNext,
        path: skwd_config::keys::keybind::COLOR_NEXT,
        title_key: "keybind-color-next",
        group: KeyBindingGroup::Filters,
    },
    KeyBindingDescriptor {
        action: InputAction::NavLeft,
        path: skwd_config::keys::keybind::NAV_LEFT,
        title_key: "keybind-nav-left",
        group: KeyBindingGroup::Navigation,
    },
    KeyBindingDescriptor {
        action: InputAction::NavRight,
        path: skwd_config::keys::keybind::NAV_RIGHT,
        title_key: "keybind-nav-right",
        group: KeyBindingGroup::Navigation,
    },
    KeyBindingDescriptor {
        action: InputAction::NavUp,
        path: skwd_config::keys::keybind::NAV_UP,
        title_key: "keybind-nav-up",
        group: KeyBindingGroup::Navigation,
    },
    KeyBindingDescriptor {
        action: InputAction::NavDown,
        path: skwd_config::keys::keybind::NAV_DOWN,
        title_key: "keybind-nav-down",
        group: KeyBindingGroup::Navigation,
    },
    KeyBindingDescriptor {
        action: InputAction::PageBack,
        path: skwd_config::keys::keybind::PAGE_BACK,
        title_key: "keybind-page-back",
        group: KeyBindingGroup::Navigation,
    },
    KeyBindingDescriptor {
        action: InputAction::PageForward,
        path: skwd_config::keys::keybind::PAGE_FORWARD,
        title_key: "keybind-page-forward",
        group: KeyBindingGroup::Navigation,
    },
    KeyBindingDescriptor {
        action: InputAction::JumpFirst,
        path: skwd_config::keys::keybind::JUMP_FIRST,
        title_key: "keybind-jump-first",
        group: KeyBindingGroup::Navigation,
    },
    KeyBindingDescriptor {
        action: InputAction::JumpLast,
        path: skwd_config::keys::keybind::JUMP_LAST,
        title_key: "keybind-jump-last",
        group: KeyBindingGroup::Navigation,
    },
    KeyBindingDescriptor {
        action: InputAction::ModePrev,
        path: skwd_config::keys::keybind::MODE_PREV,
        title_key: "keybind-mode-prev",
        group: KeyBindingGroup::Navigation,
    },
    KeyBindingDescriptor {
        action: InputAction::ModeNext,
        path: skwd_config::keys::keybind::MODE_NEXT,
        title_key: "keybind-mode-next",
        group: KeyBindingGroup::Navigation,
    },
    KeyBindingDescriptor {
        action: InputAction::Autocomplete,
        path: skwd_config::keys::keybind::AUTOCOMPLETE,
        title_key: "keybind-autocomplete",
        group: KeyBindingGroup::Navigation,
    },
    KeyBindingDescriptor {
        action: InputAction::TypePrev,
        path: skwd_config::keys::keybind::TYPE_PREV,
        title_key: "keybind-type-prev",
        group: KeyBindingGroup::Filters,
    },
    KeyBindingDescriptor {
        action: InputAction::TypeNext,
        path: skwd_config::keys::keybind::TYPE_NEXT,
        title_key: "keybind-type-next",
        group: KeyBindingGroup::Filters,
    },
    KeyBindingDescriptor {
        action: InputAction::SortPrev,
        path: skwd_config::keys::keybind::SORT_PREV,
        title_key: "keybind-sort-prev",
        group: KeyBindingGroup::Filters,
    },
    KeyBindingDescriptor {
        action: InputAction::SortNext,
        path: skwd_config::keys::keybind::SORT_NEXT,
        title_key: "keybind-sort-next",
        group: KeyBindingGroup::Filters,
    },
    KeyBindingDescriptor {
        action: InputAction::RandomRotate,
        path: skwd_config::keys::keybind::RANDOM_ROTATE,
        title_key: "keybind-random-rotate",
        group: KeyBindingGroup::Filters,
    },
    KeyBindingDescriptor {
        action: InputAction::SearchMode,
        path: skwd_config::keys::keybind::SEARCH_MODE,
        title_key: "keybind-search-mode",
        group: KeyBindingGroup::Filters,
    },
    KeyBindingDescriptor {
        action: InputAction::Downloads,
        path: skwd_config::keys::keybind::DOWNLOADS,
        title_key: "keybind-downloads",
        group: KeyBindingGroup::Panels,
    },
    KeyBindingDescriptor {
        action: InputAction::SourceWallhaven,
        path: skwd_config::keys::keybind::SOURCE_WALLHAVEN,
        title_key: "keybind-source-wallhaven",
        group: KeyBindingGroup::Downloads,
    },
    KeyBindingDescriptor {
        action: InputAction::SourceSteam,
        path: skwd_config::keys::keybind::SOURCE_STEAM,
        title_key: "keybind-source-steam",
        group: KeyBindingGroup::Downloads,
    },
    KeyBindingDescriptor {
        action: InputAction::SourceUnsplash,
        path: skwd_config::keys::keybind::SOURCE_UNSPLASH,
        title_key: "keybind-source-unsplash",
        group: KeyBindingGroup::Downloads,
    },
    KeyBindingDescriptor {
        action: InputAction::SourcePexels,
        path: skwd_config::keys::keybind::SOURCE_PEXELS,
        title_key: "keybind-source-pexels",
        group: KeyBindingGroup::Downloads,
    },
    KeyBindingDescriptor {
        action: InputAction::SourceYoutube,
        path: skwd_config::keys::keybind::SOURCE_YOUTUBE,
        title_key: "keybind-source-youtube",
        group: KeyBindingGroup::Downloads,
    },
    KeyBindingDescriptor {
        action: InputAction::SourceBing,
        path: skwd_config::keys::keybind::SOURCE_BING,
        title_key: "keybind-source-bing",
        group: KeyBindingGroup::Downloads,
    },
];
