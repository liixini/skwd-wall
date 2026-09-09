use crate::domain::effects::EffectDefinition;
use crate::domain::library::catalog::WallpaperKind;

use super::displays::DisplaySelectionState;
use super::editor::EffectEditorState;
use super::panel::EffectsPanelState;
use super::preview::PreviewSourceState;

#[derive(Debug, Clone)]
pub enum EffectsMsg {
    Select(String),
    SetNum(String, f64),
    SetStr(String, String),
    SetChoice(String, String),
    Preview,
    ToggleSaved,
    Apply,
    MonitorToggle(String),
    MonitorToggleAll,
    MonitorLock(String, bool),
    MonitorHover(Option<String>),
    MonitorApplySelected,
    SrcMute(bool),
    SrcVolume(u32),
}

pub const NAV_CATEGORY: usize = 0;
pub const NAV_EFFECT: usize = 1;
pub const NAV_CONFIG_BASE: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectsMode {
    Studio,
    Displays,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileAudio {
    None,
    Live,
    PreApply,
}

#[derive(Clone)]
pub struct MonitorInfo {
    pub name: String,
    pub target: String,
    pub connected: bool,
    pub width: i32,
    pub height: i32,
    pub current_thumb: Option<String>,
    pub kind: WallpaperKind,
    pub mute: bool,
    pub volume: u32,
    pub fill: String,
    pub locked: bool,
    pub paused: bool,
    pub manual_paused: bool,
    pub current: String,
    pub we_id: String,
}

impl MonitorInfo {
    pub(in crate::frontend::effects) fn preview_dimensions(
        &self,
        max_width: f32,
        max_height: f32,
    ) -> (f32, f32) {
        let max_width = max_width.max(1.0);
        let max_height = max_height.max(1.0);
        if self.width <= 0 || self.height <= 0 {
            return (max_width, (max_width * 9.0 / 16.0).min(max_height));
        }
        let fit = (max_width / self.width as f32).min(max_height / self.height as f32);
        (self.width as f32 * fit, self.height as f32 * fit)
    }

    pub(in crate::frontend::effects) fn orientation_label_key(&self) -> &'static str {
        match self.width.cmp(&self.height) {
            std::cmp::Ordering::Less => "effects-orientation-portrait",
            std::cmp::Ordering::Greater => "effects-orientation-landscape",
            std::cmp::Ordering::Equal => "effects-orientation-square",
        }
    }
}

pub const FILL_MODES: [(&str, &str); 6] = [
    ("fill", "effects-fill-fill"),
    ("fit", "effects-fill-fit"),
    ("stretch", "effects-fill-stretch"),
    ("center", "effects-fill-center"),
    ("tile", "effects-fill-tile"),
    ("span", "effects-fill-span"),
];

pub struct Effects {
    pub(in crate::frontend::effects) editor: EffectEditorState,
    pub(in crate::frontend::effects) displays: DisplaySelectionState,
    pub(in crate::frontend::effects) preview: PreviewSourceState,
    pub(in crate::frontend::effects) panel: EffectsPanelState,
    card: usize,
}

impl Effects {
    pub fn new(
        definitions: Vec<EffectDefinition>,
        source: String,
        source_thumb: Option<String>,
        card: usize,
        source_kind: WallpaperKind,
        source_mute: bool,
        source_volume: u32,
        _source_key: String,
    ) -> Self {
        let mut effects = Self {
            editor: EffectEditorState::new(definitions),
            displays: DisplaySelectionState::default(),
            preview: PreviewSourceState::new(
                source,
                source_thumb,
                source_kind,
                source_mute,
                source_volume,
            ),
            panel: EffectsPanelState::new(),
            card,
        };
        effects.reset_params();
        effects
    }

    pub fn card(&self) -> usize {
        self.card
    }

    pub fn replace_source(
        &mut self,
        source: String,
        source_thumb: Option<String>,
        card: usize,
        source_kind: WallpaperKind,
        source_mute: bool,
        source_volume: u32,
    ) {
        self.preview =
            PreviewSourceState::new(source, source_thumb, source_kind, source_mute, source_volume);
        self.card = card;
    }

    pub fn mode(&self) -> EffectsMode {
        self.panel.mode
    }

    pub fn set_mode(&mut self, mode: EffectsMode) {
        self.panel.mode = mode;
    }

    pub fn set_chrome(&mut self, palette: crate::frontend::theme::Palette) {
        self.panel.chrome = palette;
    }

    pub fn set_motion_profile(&mut self, motion: crate::frontend::animation::MotionProfile) {
        self.panel.set_motion_profile(motion);
    }

    pub fn is_busy(&self) -> bool {
        self.panel.busy
    }

    #[cfg(test)]
    pub fn preview_queued(&self) -> bool {
        self.panel.preview_dirty
    }

    #[cfg(test)]
    pub fn status(&self) -> &str {
        &self.panel.status
    }

    pub fn set_source_mute(&mut self, mute: bool) {
        self.preview.mute = mute;
    }

    pub fn set_source_volume(&mut self, volume: u32) {
        self.preview.volume = volume.min(100);
    }

    pub fn source_audio(&self) -> (bool, u32) {
        (self.preview.mute, self.preview.volume)
    }

    pub fn source_path(&self) -> &str {
        &self.preview.path
    }

    pub fn preview_path(&self) -> Option<&str> {
        self.preview.rendered.as_deref()
    }

    #[cfg(test)]
    pub fn effect_id(&self) -> &str {
        &self.editor.selected_id
    }

    pub fn monitors(&self) -> &[MonitorInfo] {
        &self.displays.monitors
    }

    pub fn set_monitors(&mut self, mut monitors: Vec<MonitorInfo>) {
        super::displays::align_shared_audio(&mut monitors);
        self.displays.monitors = monitors;
    }

    pub fn matches_source(&self, card: usize, source: &str) -> bool {
        self.card == card && self.preview.path == source
    }
}
