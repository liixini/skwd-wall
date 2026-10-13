use std::collections::HashMap;

use serde_json::Value;

use crate::contracts::browser::Source;
use crate::infrastructure::ipc::DaemonClient;

use super::TaskUiState;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Pending {
    PlaybackProcesses,
    CurrentTheme {
        load: bool,
    },
    List,
    ThemeBackends,
    AppThemes,
    AppThemeSet,
    LockScreenSet {
        target: wall_proto::LockScreenTarget,
        enabled: bool,
        sddm_theme: Option<String>,
    },
    EffectThemes,
    Diag,
    BugReport,
    PlList,
    PlMembers,
    PlOutputs,
    CardPickerList,
    CardPickerMembers,
    CardPickerCreate,
    BrowserSearch {
        source: Source,
        append: bool,
        generation: u64,
    },
    BrowserCollections {
        source: Source,
    },
    BrowserDownload {
        source: Source,
        id: String,
    },
    BrowserPreview {
        source: Source,
        id: String,
    },
    EffectsPreview {
        source: String,
        cache_key: String,
    },
    EffectsCommit {
        apply: bool,
    },
    Outputs,
    Activities,
    ActivityOutputs {
        activity: String,
        generation: u64,
    },
    ActivityApply {
        token: String,
    },
    DemoOutputs,
    AudioOutputs,
    AudioPause,
    ThemePreview {
        card: usize,
        backend: String,
    },
    ThemePreviews {
        backend: String,
    },
    TaskList,
    Status,
    Weather,
    WeatherCandidates {
        category: String,
        query: String,
        generation: u64,
    },
    WeatherAction,
    WeatherPlaylists,
    SceneProperties {
        we_id: String,
        revision: u64,
        writes: Vec<String>,
    },
    ResetThumbnail {
        key: String,
    },
    RemoveWallpaper {
        key: String,
        unsubscribe: bool,
    },
}

pub(crate) struct DaemonState {
    pub(crate) app_themes: Option<crate::contracts::daemon::AppThemesResult>,
    pub(crate) weather: Option<crate::contracts::daemon::WeatherStatus>,
    pub(crate) weather_playlists: Vec<crate::contracts::playlists::Playlist>,
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
    pub(crate) steam_client_available: Option<bool>,
    pub(crate) steam_refresh_needed: bool,
    pub(crate) library_watch: Option<crate::contracts::daemon::LibraryWatchStatus>,
    pub(crate) tasks: TaskUiState,
    pub(crate) activity_applies: HashMap<u64, ActivityApplyBatch>,
    pub(crate) activity_requests: HashMap<String, ActivityApplyRequest>,
    pub(crate) next_activity_apply: u64,
}

pub(crate) struct ActivityApplyBatch {
    pub(crate) activity: String,
    pub(crate) name: String,
    pub(crate) pending: usize,
    pub(crate) failed: bool,
    pub(crate) visible: bool,
}

pub(crate) struct ActivityApplyRequest {
    pub(crate) batch: u64,
    pub(crate) id: u64,
    pub(crate) output: String,
}

impl DaemonState {
    pub(crate) fn new(client: DaemonClient) -> Self {
        Self {
            app_themes: None,
            weather: None,
            weather_playlists: Vec::new(),
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
            steam_client_available: None,
            steam_refresh_needed: false,
            library_watch: None,
            tasks: TaskUiState::default(),
            activity_applies: HashMap::new(),
            activity_requests: HashMap::new(),
            next_activity_apply: 0,
        }
    }
}
