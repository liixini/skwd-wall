#[derive(Debug, Clone)]
pub enum BarAction {
    SetType(String),
    Sort(String),
    Orient(String),
    Resolution(String),
    Favs,
    Color(i64),
    FolderToggle,
    ShowHidden,
    Settings,
    Stash,
    Download,
    Playlists,
    Random,
    TagCloud,
    ThemePanel,
    Audio,
    AudioPanel,
    TaskControl { id: String, action: crate::contracts::daemon::TaskControl },
    ThemeBackendToggle,
    ThemeOpt(&'static str, &'static str),
    ThemePinSettings,
}

#[derive(Debug, Clone)]
pub enum BarIntent {
    Hover(Option<usize>, Option<usize>),
    Activate(BarAction),
    SelectFolder(String),
    ToggleFolderMenu,
    Theme(crate::frontend::theme_designer::ThemeMsg),
    ScrollFolderMenu(f32),
    StepVolume(i32),
}
