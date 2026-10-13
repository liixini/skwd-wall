#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct AppThemeStatus {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub config_found: bool,
    pub config_path: String,
    pub output_path: String,
    pub enabled: bool,
    pub state: String,
    pub detail: String,
    pub can_enable: bool,
    pub can_disable: bool,
    pub can_adopt: bool,
    pub template_path: String,
    pub customized: bool,
    pub can_disconnect: bool,
    pub can_reconnect: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppThemesResult {
    pub apps: Vec<AppThemeStatus>,
}

#[derive(Debug, Clone)]
pub struct CurrentTheme {
    pub key: String,
    pub name: String,
    pub thumb: String,
    pub palette: crate::domain::theme::Candidate,
    pub dark: bool,
}

use crate::contracts::media::MediaKind;
use crate::contracts::picker::PaletteSpec;
use crate::contracts::playlists::{Playlist, PlaylistAssignment, PlaylistMember};
use crate::domain::effects::EffectDefinition;
use crate::domain::scene_properties::SceneProperty;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiagnosticResult {
    pub banner: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherState {
    #[default]
    Disabled,
    Loading,
    NoLocation,
    Unavailable,
    Unmapped,
    Missing,
    Empty,
    SearchUnavailable,
    Active,
    Paused,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherSource {
    #[default]
    Playlists,
    Semantic,
    Outdoors,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherSelection {
    #[default]
    Automatic,
    Approved,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize)]
pub struct WeatherCandidates {
    pub category: String,
    pub query: Option<String>,
    pub member_keys: Vec<String>,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutdoorSeason {
    #[default]
    Neutral,
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LightPhase {
    Dawn,
    #[default]
    Day,
    Dusk,
    Night,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutdoorMatch {
    Library,
    Starter,
    #[default]
    Ambient,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OutdoorStatus {
    pub season: OutdoorSeason,
    pub light: LightPhase,
    pub match_kind: OutdoorMatch,
    pub stale: bool,
    pub generation: u64,
    pub location: String,
    pub credits: Option<String>,
    pub outputs: Vec<OutdoorOutput>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct OutdoorOutput {
    pub output: String,
    pub key: String,
    pub name: String,
    pub preview: Option<String>,
    pub held: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct WeatherStatus {
    pub weather: Vec<String>,
    pub enabled: bool,
    pub status: WeatherState,
    pub source: WeatherSource,
    pub selection: WeatherSelection,
    pub location_key: Option<String>,
    pub category: Option<String>,
    pub query: Option<String>,
    pub playlist_id: Option<i64>,
    pub playlist_name: Option<String>,
    pub member_keys: Vec<String>,
    pub resolved_count: usize,
    pub fetched_at: Option<u64>,
    pub paused_outputs: Vec<String>,
    pub resume_available: bool,
    pub outdoors: Option<OutdoorStatus>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BugReportResult {
    pub path: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EffectOperationResult {
    pub output: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EffectsListResult {
    pub definitions: Option<Vec<EffectDefinition>>,
    pub theme_options: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaylistListResult {
    pub playlists: Option<Vec<Playlist>>,
    pub assignments: Option<Vec<PlaylistAssignment>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaylistMembersResult {
    pub id: Option<i64>,
    pub members: Option<Vec<PlaylistMember>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaylistOutputsResult {
    pub outputs: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CardPickerMembershipsResult {
    pub ids: Option<Vec<i64>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CardPickerCreateResult {
    pub id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputStatus {
    pub name: String,
    pub target: String,
    pub connected: bool,
    pub width: i32,
    pub height: i32,
    pub logical_width: i32,
    pub logical_height: i32,
    pub current: String,
    pub kind: MediaKind,
    pub path: String,
    pub we_id: String,
    pub mute: bool,
    pub volume: u32,
    pub fill: String,
    pub audio_shared: bool,
    pub paused: bool,
    pub manual_paused: bool,
}

impl Default for OutputStatus {
    fn default() -> Self {
        Self {
            name: String::new(),
            target: String::new(),
            connected: true,
            width: 0,
            height: 0,
            logical_width: 0,
            logical_height: 0,
            current: String::new(),
            kind: MediaKind::default(),
            path: String::new(),
            we_id: String::new(),
            mute: false,
            volume: 100,
            fill: String::new(),
            audio_shared: false,
            paused: false,
            manual_paused: false,
        }
    }
}

impl OutputStatus {
    pub fn target(&self) -> &str {
        if self.target.is_empty() { &self.name } else { &self.target }
    }

    pub fn is_connected(&self) -> bool {
        self.connected || self.target.is_empty()
    }

    pub fn logical_size(&self) -> (i32, i32) {
        if self.logical_width > 0 && self.logical_height > 0 {
            (self.logical_width, self.logical_height)
        } else {
            (self.width, self.height)
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OutputsResult {
    pub outputs: Vec<OutputStatus>,
}

#[derive(Clone)]
pub struct Activity {
    pub id: String,
    pub name: String,
    pub current: bool,
}

pub struct ActivitiesResult {
    pub supported: bool,
    pub current: String,
    pub activities: Vec<Activity>,
}

#[derive(serde::Deserialize)]
pub struct ActivityApplyResult {
    pub activity: String,
    pub visible: bool,
    pub queued: bool,
}

pub enum ActivityCompletion {
    Applied { visible: bool },
    Failed(String),
    Cancelled,
}

pub struct ActivityApplyEvent {
    pub token: String,
    pub request_id: u64,
    pub activity: String,
    pub output: String,
    pub completion: Option<ActivityCompletion>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AudioOutputsResult {
    pub outputs: Vec<OutputStatus>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThemeBackendsResult {
    pub backends: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThemePreviewResult {
    pub colors: Vec<String>,
    pub palette: Option<PaletteSpec>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThemeAuditionPreview {
    pub backend: String,
    pub key: String,
    pub value: String,
    pub label: String,
    pub palette: PaletteSpec,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThemePreviewsResult {
    pub backend: Option<String>,
    pub backends: Vec<String>,
    pub previews: Vec<ThemeAuditionPreview>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProtocolStatus {
    pub name: String,
    pub version: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LibraryWatchRootStatus {
    pub path: String,
    pub mode: String,
    pub native_error: Option<String>,
    pub last_completed_sweep_unix_ms: Option<u64>,
    pub last_scan_requested_unix_ms: Option<u64>,
    pub last_successful_convergence_unix_ms: Option<u64>,
    pub pending_scans: usize,
    pub last_poll_error: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LibraryWatchStatus {
    pub ok: bool,
    pub degraded: bool,
    pub mode: String,
    pub detail: String,
    pub interval_seconds: Option<u64>,
    pub entry_budget_per_root: Option<usize>,
    pub last_successful_convergence_unix_ms: Option<u64>,
    pub roots: Vec<LibraryWatchRootStatus>,
}

pub struct LibraryWatchMode;

impl LibraryWatchMode {
    pub const NATIVE: &str = "native";
    pub const POLLING: &str = "polling";
    pub const RECOVERING: &str = "recovering";
    pub const UNAVAILABLE: &str = "unavailable";
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatusResult {
    pub playback: Option<PlaybackStatus>,
    pub version: String,
    pub protocol: Option<ProtocolStatus>,
    pub capabilities: Vec<String>,
    pub steam_helper_available: Option<bool>,
    pub steam_client_available: Option<bool>,
    pub library_watch_present: bool,
    pub library_watch: Option<LibraryWatchStatus>,
}

impl StatusResult {
    pub fn advertises(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|entry| entry == capability)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskCapabilities {
    pub pause: bool,
    pub resume: bool,
    pub stop: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum TaskState {
    #[default]
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
    Other(String),
}

impl TaskState {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Paused | Self::Other(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskStatus {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub state: TaskState,
    pub progress: u64,
    pub total: u64,
    pub detail: String,
    pub eta: String,
    pub capabilities: TaskCapabilities,
}

impl TaskStatus {
    pub fn running(
        id: impl Into<String>,
        kind: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: kind.into(),
            label: label.into(),
            state: TaskState::Running,
            progress: 0,
            total: 0,
            detail: String::new(),
            eta: String::new(),
            capabilities: TaskCapabilities::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskControl {
    Pause,
    Resume,
    Stop,
}

impl TaskControl {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Stop => "stop",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskListResult {
    pub tasks: Option<Vec<TaskStatus>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScenePropertiesResult {
    pub we_id: String,
    pub fps: Option<u32>,
    pub global_fps: Option<u32>,
    pub rows: Vec<SceneProperty>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(default)]
pub struct PlaybackStatus {
    pub fullscreen_supported: bool,
    pub maximized_supported: bool,
    pub maximized_paused: bool,
    pub full_width_supported: bool,
    pub full_width_paused: bool,
    pub overview_paused: bool,
    pub processes: Vec<String>,
    pub outputs: Vec<String>,
    pub all_displays: bool,
    pub automatic_paused: bool,
    pub resume_pending: bool,
    pub other_audio_supported: bool,
    pub audio_ducked: bool,
    pub available_processes: Vec<String>,
}
pub const APP_THEME_BUSY: &str = "busy";
