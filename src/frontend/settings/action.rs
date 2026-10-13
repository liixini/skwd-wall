#[derive(Debug, Clone, PartialEq)]
pub enum ActionId {
    SetThemeTarget(&'static str, bool),
    SetLockScreen(crate::contracts::settings::LockScreenTarget, bool),
    ChooseSddmTheme(Option<String>),
    RefreshAppThemes,
    SetAppTheme(u8, bool),
    AdoptAppTheme(u8),
    RefreshAppTheme(u8),
    CustomizeAppTheme(u8, &'static str),
    CopyAppThemeTemplate(u8),
    ClearCache,
    CaptureWeThumbnails,
    RecomputeColors,
    OptimizeImages,
    RefreshBackdrop,
    CopyLayerRule,
    AddPostCommand,
    RemovePostCommand(u16),
    AddFolder(crate::contracts::settings::FolderKind),
    ChooseFolder(crate::contracts::settings::FolderKind, Option<usize>),
    RemoveFolder(crate::contracts::settings::FolderKind, usize),
    UndoFolderRemove,
    NewProfile,
    DeleteProfile(u16),
    CopyProfileCommand(u16, bool),
    AddIntegration,
    RemoveIntegration(u16),
    ImportSemanticModel,
    AddSemanticModel,
    RemoveSemanticModel(u16),
    DeleteSemanticModel(u16),
    DeleteActiveSemanticModel,
    AddResolutionPreset,
    CreateResolutionPresetBand(&'static str),
    RemoveResolutionPreset(u16),
    OpenScheduleEditor,
    RefreshWeather,
    ResumeWeather,
    ReviewWeather(&'static str),
    FollowOutdoors(bool),
    OutdoorNext(String),
    OutdoorExclude { output: String, key: String },
    OutdoorPin(String),
    OutdoorResume(String),
    OutdoorCredits(String),
    OpenThemeDesigner,
    ChooseRunningProcess,
    GenerateBugReport,
    ResetMotionFast,
    ResetMotionStandard,
    ResetMotionSlow,
    ResetKeybinds,
    SetNavKeys(crate::domain::input::NavKeys),
}

impl ActionId {
    pub const fn is_destructive(&self) -> bool {
        matches!(
            self,
            Self::ClearCache
                | Self::OptimizeImages
                | Self::ResetKeybinds
                | Self::DeleteSemanticModel(_)
                | Self::DeleteActiveSemanticModel
                | Self::DeleteProfile(_)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsFocus {
    Index,
    Sections,
    Controls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsKey {
    FocusNext { backwards: bool },
    CategoryNext { backwards: bool },
    Search,
    Previous,
    Next,
    Up,
    Down,
    Activate,
    Cancel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeatherReviewTab {
    #[default]
    Suggested,
    Approved,
    Library,
}

pub const WEATHER_REVIEW_TAB_COUNT: usize = 3;

#[derive(Debug, Clone)]
pub enum SettingsMsg {
    Input(String, String),
    ResolutionInput(String, bool, String),
    ResolutionLimit(String, bool),
    Toggle(String, bool),
    Commit,
    Pick(String, String),
    BackgroundPicker(String),
    BackgroundMode(String, String),
    BackgroundColor(String, String),
    BackgroundCommit(String),
    SelectSection(usize),
    LeaveLayoutStudio,
    FocusControl(usize),
    ToggleDetails(String, usize),
    ToggleBar(String, usize),
    ToggleFold(String, bool),
    SelectProfile(usize),
    ProfileFilter(String),
    EditFolder(String),
    FolderDraft(crate::contracts::settings::FolderKind, String),
    SearchOpen,
    SearchClose,
    SearchInput(String),
    OpenSearchResult(String, usize, usize),
    SelectTab(String),
    Key(SettingsKey),
    Run(ActionId),
    KeybindCapture(String, usize),
    KeybindCaptureCancel,
    KeybindCaptureApply,
    KeybindCaptureDefault,
    KeybindCaptureUnbind,
    KeybindCaptureClick(crate::domain::input::MouseButton),
    ProcessPickerClose,
    ProcessPickerSearch(String),
    ProcessPickerManualInput(String),
    ProcessPickerManualAdd,
    ProcessPickerScroll(f32),
    ProcessPickerAdd(String),
    ProcessPickerRemove(String),
    WeatherReviewClose,
    WeatherReviewTab(WeatherReviewTab),
    WeatherReviewSearch(String),
    WeatherReviewToggle(String),
}
