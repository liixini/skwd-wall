#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionId {
    ClearCache,
    CaptureWeThumbnails,
    RecomputeColors,
    OptimizeImages,
    RefreshBackdrop,
    CopyLayerRule,
    AddPostCommand,
    RemovePostCommand(u16),
    AddIntegration,
    RemoveIntegration(u16),
    ImportSemanticModel,
    AddSemanticModel,
    RemoveSemanticModel(u16),
    AddResolutionPreset,
    RemoveResolutionPreset(u16),
    OpenScheduleEditor,
    OpenThemeDesigner,
    ChooseRunningProcess,
    GenerateBugReport,
    ResetMotionFast,
    ResetMotionStandard,
    ResetMotionSlow,
    ResetKeybinds,
}

impl ActionId {
    pub const fn is_destructive(self) -> bool {
        matches!(self, Self::ClearCache | Self::OptimizeImages | Self::ResetKeybinds)
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

#[derive(Debug, Clone)]
pub enum SettingsMsg {
    Input(String, String),
    Toggle(String, bool),
    Commit,
    Pick(String, String),
    SelectSection(usize),
    LeaveLayoutStudio,
    FocusControl(usize),
    ToggleDetails(String, usize),
    ToggleBar(String, usize),
    SearchOpen,
    SearchClose,
    SearchInput(String),
    OpenSearchResult(String, usize, usize),
    SelectTab(String),
    Key(SettingsKey),
    Run(ActionId),
    KeybindCapture(String),
    KeybindCaptureCancel,
    KeybindCaptureApply,
    KeybindCaptureDefault,
    KeybindCaptureUnbind,
    KeybindCaptureClick(crate::domain::input::MouseButton),
}
