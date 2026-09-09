use std::collections::HashMap;

use serde_json::Value;

use crate::contracts::browser::Source;
use crate::infrastructure::ipc::DaemonClient;

use super::TaskUiState;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Pending {
    PlaybackProcesses,
    CurrentTheme { load: bool },
    List,
    ThemeBackends,
    EffectThemes,
    Diag,
    Doctor,
    BugReport,
    PlList,
    PlMembers,
    PlOutputs,
    CardPickerList,
    CardPickerMembers,
    CardPickerCreate,
    BrowserSearch { source: Source, append: bool, generation: u64 },
    BrowserCollections { source: Source },
    BrowserDownload { source: Source, id: String },
    BrowserPreview { source: Source, id: String },
    EffectsPreview { source: String, cache_key: String },
    EffectsCommit { apply: bool },
    Outputs,
    DemoOutputs,
    AudioOutputs,
    AudioPause,
    ThemePreview { card: usize, backend: String },
    ThemePreviews { backend: String },
    TaskList,
    Status,
    Weather,
    SceneProperties { we_id: String },
}

pub(crate) struct DaemonState {
    pub(crate) playback: crate::contracts::daemon::PlaybackStatus,
    pub(crate) client: DaemonClient,
    pub(crate) pending: HashMap<u64, Pending>,
    pub(crate) connected: bool,
    pub(crate) ever_connected: bool,
    pub(crate) diagnostic: String,
    pub(crate) effect_definitions: Vec<crate::domain::effects::EffectDefinition>,
    pub(crate) last_wallpaper: Option<Value>,
    pub(crate) current_wallpaper_art: Option<String>,
    pub(crate) output_wallpaper_art: HashMap<String, String>,
    pub(crate) effect_themes: Vec<String>,
    pub(crate) output_names: Vec<String>,
    pub(crate) output_statuses: Vec<crate::contracts::daemon::OutputStatus>,
    pub(crate) steam_helper_available: Option<bool>,
    pub(crate) library_watch: Option<crate::contracts::daemon::LibraryWatchStatus>,
    pub(crate) tasks: TaskUiState,
}

impl DaemonState {
    pub(crate) fn new(client: DaemonClient) -> Self {
        Self {
            playback: crate::contracts::daemon::PlaybackStatus::default(),
            client,
            pending: HashMap::new(),
            connected: false,
            ever_connected: false,
            diagnostic: String::from("walld: connecting..."),
            effect_definitions: Vec::new(),
            last_wallpaper: None,
            current_wallpaper_art: None,
            output_wallpaper_art: HashMap::new(),
            effect_themes: Vec::new(),
            output_names: Vec::new(),
            output_statuses: Vec::new(),
            steam_helper_available: None,
            library_watch: None,
            tasks: TaskUiState::default(),
        }
    }
}
