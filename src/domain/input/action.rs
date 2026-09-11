#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    Select,
    Apply,
    Flip,
    Favourite,
    Effects,
    Studio,
    SceneProperties,
    Playlists,
    Settings,
    Help,
    TagCloud,
    TagMode,
    FilterBar,
    FolderPrev,
    FolderNext,
    FolderToggle,
    HiddenFolders,
    ColorPrev,
    ColorNext,
    NavLeft,
    NavRight,
    NavUp,
    NavDown,
    Autocomplete,
}

impl InputAction {
    pub const ALL: [Self; 24] = [
        Self::Select,
        Self::Apply,
        Self::Flip,
        Self::Favourite,
        Self::Effects,
        Self::Studio,
        Self::SceneProperties,
        Self::Playlists,
        Self::Settings,
        Self::Help,
        Self::TagCloud,
        Self::TagMode,
        Self::FilterBar,
        Self::FolderPrev,
        Self::FolderNext,
        Self::FolderToggle,
        Self::HiddenFolders,
        Self::ColorPrev,
        Self::ColorNext,
        Self::NavLeft,
        Self::NavRight,
        Self::NavUp,
        Self::NavDown,
        Self::Autocomplete,
    ];

    pub const fn default_binding(self) -> &'static str {
        match self {
            Self::Select => "click",
            Self::Apply => "enter",
            Self::Flip => "right-click",
            Self::Favourite => "f",
            Self::Effects => "ctrl+click",
            Self::Studio => "shift+right-click",
            Self::SceneProperties => "shift+w",
            Self::Playlists => "p",
            Self::Settings => "shift+s",
            Self::Help => "?",
            Self::TagCloud => "shift+down",
            Self::TagMode => "t",
            Self::FilterBar => "shift+up",
            Self::FolderPrev => "ctrl+left",
            Self::FolderNext => "ctrl+right",
            Self::FolderToggle => "ctrl+m",
            Self::HiddenFolders => "ctrl+h",
            Self::ColorPrev => "shift+left",
            Self::ColorNext => "shift+right",
            Self::NavLeft => "left",
            Self::NavRight => "right",
            Self::NavUp => "up",
            Self::NavDown => "down",
            Self::Autocomplete => "tab",
        }
    }

    pub const fn targets_card(self) -> bool {
        matches!(
            self,
            Self::Select
                | Self::Apply
                | Self::Flip
                | Self::Favourite
                | Self::Effects
                | Self::Studio
                | Self::SceneProperties
        )
    }
}
