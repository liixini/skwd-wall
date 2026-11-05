use super::{SettingsState, TransitionPreviewState};

pub(crate) type StudioRestore = (
    Vec<(&'static str, Option<serde_json::Value>)>,
    crate::frontend::theme::Palette,
    Option<String>,
);

#[derive(Default)]
pub(crate) struct StrandEntries {
    pub(crate) raw: Vec<serde_json::Value>,
    pub(crate) slots: Vec<usize>,
    pub(crate) widgets: Vec<paper_control::Widget>,
    pub(crate) room: usize,
}

#[derive(Default)]
pub(crate) struct PanelsState {
    pub(crate) arrange: Option<super::arrange::ArrangeSession>,
    pub(crate) audio_volumes: std::collections::HashMap<String, u32>,
    pub(crate) audio_active: bool,
    pub(crate) audio_playing: bool,
    pub(crate) audio: Option<crate::frontend::audio_panel::AudioPanel>,
    pub(crate) settings: SettingsState,
    pub(crate) transition_preview: TransitionPreviewState,
    pub(crate) playlists: Option<crate::frontend::playlists::Playlists>,
    pub(crate) card_picker: Option<crate::frontend::playlists::CardPicker>,
    pub(crate) effects: Option<crate::frontend::effects::Effects>,
    pub(crate) schedule: Option<crate::frontend::schedule_editor::ScheduleEditor>,
    pub(crate) theme_designer: Option<crate::frontend::theme_designer::ThemeDesigner>,
    pub(crate) colour_studio: Option<crate::frontend::colour_studio::ColourStudio>,
    pub(crate) strand: Option<crate::frontend::strand::Strand>,
    pub(crate) strand_entries: StrandEntries,
    pub(crate) studio_restore: Option<StudioRestore>,
    pub(crate) scene_properties: Option<crate::frontend::scene_properties::SceneProperties>,
}

impl PanelsState {
    pub(crate) fn audio_view(&self) -> (bool, bool, bool, bool) {
        (
            self.settings.open,
            self.settings.tab == crate::frontend::settings::place::WALLPAPERS,
            self.effects.is_some(),
            self.audio.is_some(),
        )
    }
}
