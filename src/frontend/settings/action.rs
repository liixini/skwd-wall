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
    ArrangePicker,
    UsePremade(usize),
    Nudge(&'static NudgeSpec, i8, i8),
    CentreNudge(&'static NudgeSpec),
    ClearCache,
    ChooseDrawingPack,
    DetachDrawingPack,
    PrepareDrawing,
    PrepareAllDrawings,
    CancelDrawing,
    PauseDrawing,
    ResumeDrawing,
    ClearDrawingCache,
    PreviewDrawing,
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
    SelectProfile(usize),
    ActivateProfile(String),
    FoldTree(String, bool),
    OpenSection(usize),
    AddIntegration,
    RemoveIntegration(u16),
    ExportConfiguration,
    ImportConfiguration,
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
    ResetKeybinds,
    SetNavKeys(crate::domain::input::NavKeys),
    LightKey(&'static str),
    UndoKeyMove,
    OpenSearch,
    OpenPlace { place: &'static str, setting: Option<&'static str> },
    ResetSetting(Box<crate::contracts::settings::SettingChange>),
    ResetEverything,
}

impl ActionId {
    pub fn steady(&self) -> Self {
        match self {
            Self::SetThemeTarget(target, _) => Self::SetThemeTarget(target, false),
            Self::SetLockScreen(target, _) => Self::SetLockScreen(*target, false),
            Self::SetAppTheme(index, _) => Self::SetAppTheme(*index, false),
            Self::CopyProfileCommand(index, _) => Self::CopyProfileCommand(*index, false),
            Self::FollowOutdoors(_) => Self::FollowOutdoors(false),
            other => other.clone(),
        }
    }

    pub const fn is_destructive(&self) -> bool {
        matches!(
            self,
            Self::ClearCache
                | Self::DetachDrawingPack
                | Self::ClearDrawingCache
                | Self::OptimizeImages
                | Self::ResetKeybinds
                | Self::DeleteSemanticModel(_)
                | Self::DeleteActiveSemanticModel
                | Self::DeleteProfile(_)
                | Self::ResetEverything
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct NudgeSpec {
    pub x: &'static str,
    pub y: &'static str,
    pub coarse: f64,
    pub fine: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutPending {
    Switch(usize),
    Premade(usize),
    Delete,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LayoutsUi {
    pub confirm: Option<LayoutPending>,
    pub armed: bool,
    pub premade: Option<usize>,
    pub renaming: bool,
    pub rejected: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutMsg {
    Menu,
    Pick(usize),
    Premade(usize),
    Save,
    Revert,
    SaveNew,
    Rename,
    RenameInput(String),
    RenameCommit,
    Duplicate,
    Delete,
    ConfirmSave,
    ConfirmDiscard,
    ConfirmCancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsFocus {
    Index,
    Strip,
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
    Space,
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
    NumberStep(String, f64),
    ResolutionInput(String, bool, String),
    ResolutionLimit(String, bool),
    Toggle(String, bool),
    Commit,
    Pick(String, String),
    BackgroundPicker(String),
    BackgroundMode(String, String),
    BackgroundColor(String, String),
    BackgroundCommit(String),
    ColourPicker(String),
    Colour(String, String),
    ColourCommit(String),
    ToggleSection(usize),
    OpenPop(u64),
    ClosePop,
    EditToken(u64, String),
    EnterLayoutStudio,
    LeaveLayoutStudio,
    Layout(LayoutMsg),
    FocusControl(usize),
    ToggleDetails(String, usize),
    ToggleBar(String, usize),
    ToggleFold(String, bool),
    SelectProfile(usize),
    ProfileFilter(String),
    SearchOpen,
    SearchClose,
    SearchInput(String),
    OpenSearchResult(String, usize, usize),
    SelectTab(String),
    Key(SettingsKey),
    Run(ActionId),
    KeybindCapture(String, usize),
    KeybindCaptureCancel,
    KeybindCaptureDefault,
    KeybindCaptureUnbind,
    KeybindCaptureClick(crate::domain::input::MouseButton),
    KeysFind(String),
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

impl SettingsMsg {
    pub fn changes_config(&self) -> bool {
        match self {
            Self::Key(key) => matches!(
                key,
                SettingsKey::Previous
                    | SettingsKey::Next
                    | SettingsKey::Activate
                    | SettingsKey::Space
            ),
            Self::SearchInput(_)
            | Self::SearchOpen
            | Self::SearchClose
            | Self::OpenSearchResult(..)
            | Self::SelectTab(_)
            | Self::ToggleSection(_)
            | Self::OpenPop(_)
            | Self::ClosePop
            | Self::FocusControl(_)
            | Self::ToggleDetails(..)
            | Self::ToggleBar(..)
            | Self::ToggleFold(..)
            | Self::SelectProfile(_)
            | Self::ProfileFilter(_)
            | Self::KeysFind(_)
            | Self::EnterLayoutStudio
            | Self::LeaveLayoutStudio
            | Self::ProcessPickerClose
            | Self::ProcessPickerSearch(_)
            | Self::ProcessPickerManualInput(_)
            | Self::ProcessPickerScroll(_)
            | Self::WeatherReviewClose
            | Self::WeatherReviewTab(_)
            | Self::WeatherReviewSearch(_) => false,
            _ => true,
        }
    }
}
