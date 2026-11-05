use super::super::super::misc::{glyph_width, text_width};
use super::super::action::BarAction;
use super::super::catalog::{DROP_ARROW, DROP_ARROW_UP, MENU_ROW_H, SORTS, TYPES, sort_label_key};
use super::super::edit_id;
use crate::contracts::settings::wallpaper_kind::{STATIC, VIDEO, WE};
use crate::domain::library::filter::ResolutionPreset;
use crate::i18n::tr;

const EDIT_ALL_KEY: &str = "settings-filter-type-all";
const EDIT_IMAGES_KEY: &str = "settings-filter-type-images";
const EDIT_VIDEOS_KEY: &str = "settings-filter-type-videos";
const EDIT_WE_KEY: &str = "settings-filter-type-we";
const EDIT_SHAPE_KEY: &str = "settings-filter-show-shape-label";
const EDIT_RESOLUTION_KEY: &str = "settings-filter-show-resolution-label";
const EDIT_FAVOURITES_KEY: &str = "settings-filter-show-favourites-label";
const EDIT_FOLDER_KEY: &str = "settings-filter-show-folder-label";
const EDIT_HIDDEN_KEY: &str = "settings-profiles-hidden-note-label";
const EDIT_DOWNLOAD_KEY: &str = "settings-filter-show-download-label";
const EDIT_PLAYLISTS_KEY: &str = "settings-filter-show-playlists-label";
const EDIT_RANDOM_KEY: &str = "settings-filter-show-random-label";
const EDIT_SEARCH_KEY: &str = "settings-filter-show-tag-cloud-label";
const EDIT_THEME_KEY: &str = "settings-filter-show-theme-label";
const EDIT_AUDIO_KEY: &str = "settings-group-playback-audio";
const EDIT_MIXER_KEY: &str = "audio-mixer-heading";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarVisualStyle {
    Slices,
    Hex,
    Wall,
}

impl BarVisualStyle {
    pub fn from_key(key: &str) -> Self {
        match key {
            "hex" => Self::Hex,
            "wall" => Self::Wall,
            _ => Self::Slices,
        }
    }
}

#[derive(Clone)]
pub struct BarShow {
    pub order: Vec<String>,
    pub hidden_buttons: Vec<String>,
    pub types: Vec<&'static str>,
    pub sorts: Vec<&'static str>,
    pub orient: bool,
    pub resolution: bool,
    pub resolution_presets: Vec<ResolutionPreset>,
    pub folder: bool,
    pub favourites: bool,
    pub random: bool,
    pub colors: bool,
    pub theme: bool,
    pub tagcloud: bool,
    pub download: bool,
    pub playlists: bool,
    pub hidden: Option<bool>,
}

impl BarShow {
    pub fn all() -> Self {
        Self {
            order: Vec::new(),
            hidden_buttons: Vec::new(),
            types: TYPES.to_vec(),
            sorts: SORTS.iter().map(|(mode, _)| *mode).collect(),
            orient: true,
            resolution: true,
            resolution_presets: Vec::new(),
            folder: true,
            favourites: true,
            random: true,
            colors: true,
            theme: true,
            tagcloud: true,
            download: true,
            playlists: true,
            hidden: Some(false),
        }
    }

    pub fn type_key(kind: &str) -> &'static str {
        match kind {
            STATIC => STATIC,
            VIDEO => VIDEO,
            WE => WE,
            _ => "all",
        }
    }
}

#[derive(Clone)]
pub struct BarItem {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
    pub(crate) skew: f32,
    pub(crate) label: String,
    pub(crate) nerd: bool,
    pub(crate) text_size: f32,
    pub(crate) swatch: Option<usize>,
    pub(crate) notice: Option<BarNotice>,
    pub(crate) active: bool,
    pub(crate) action: Option<BarAction>,
    pub(crate) z: i32,
}

impl BarItem {
    pub fn edit_title(&self) -> String {
        if let Some(index) = self.swatch {
            return crate::frontend::ui::color_bucket_name(crate::frontend::ui::strip_bucket(
                index,
            ))
            .to_owned();
        }
        let key = match self.action.as_ref() {
            Some(BarAction::SetType { key, .. }) => match *key {
                STATIC => EDIT_IMAGES_KEY,
                VIDEO => EDIT_VIDEOS_KEY,
                WE => EDIT_WE_KEY,
                _ => EDIT_ALL_KEY,
            },
            Some(BarAction::Sort(mode)) => sort_label_key(mode),
            Some(BarAction::Orient(_)) => EDIT_SHAPE_KEY,
            Some(BarAction::Resolution(_)) => EDIT_RESOLUTION_KEY,
            Some(BarAction::Favs) => EDIT_FAVOURITES_KEY,
            Some(BarAction::FolderToggle) => EDIT_FOLDER_KEY,
            Some(BarAction::ShowHidden) => EDIT_HIDDEN_KEY,
            Some(BarAction::Download) => EDIT_DOWNLOAD_KEY,
            Some(BarAction::Playlists) => EDIT_PLAYLISTS_KEY,
            Some(BarAction::Random) => EDIT_RANDOM_KEY,
            Some(BarAction::TagCloud) => EDIT_SEARCH_KEY,
            Some(BarAction::ThemePanel) => EDIT_THEME_KEY,
            Some(BarAction::Audio) => EDIT_AUDIO_KEY,
            Some(BarAction::AudioPanel) => EDIT_MIXER_KEY,
            _ => return self.label.clone(),
        };
        tr(key).to_owned()
    }

    pub fn edit_id(&self) -> Option<String> {
        if let Some(index) = self.swatch {
            return Some(if index == edit_id::COLOR_CLEAR_INDEX {
                edit_id::COLOR_CLEAR.to_string()
            } else {
                format!("color:{index}")
            });
        }
        let id = match self.action.as_ref()? {
            BarAction::SetType { key, .. } => match *key {
                "" => edit_id::TYPE_ALL,
                STATIC => edit_id::TYPE_STATIC,
                VIDEO => edit_id::TYPE_VIDEO,
                _ => edit_id::TYPE_WE,
            },
            BarAction::Sort(mode) => return Some(format!("sort:{mode}")),
            BarAction::Orient(_) => edit_id::ORIENT,
            BarAction::Resolution(_) => edit_id::RESOLUTION,
            BarAction::Favs => edit_id::FAVOURITES,
            BarAction::FolderToggle => edit_id::FOLDER,
            BarAction::ShowHidden => edit_id::HIDDEN,
            BarAction::Settings => edit_id::SETTINGS,
            BarAction::Stash => edit_id::STASH,
            BarAction::Download => edit_id::DOWNLOAD,
            BarAction::Playlists => edit_id::PLAYLISTS,
            BarAction::Random => edit_id::RANDOM,
            BarAction::TagCloud => edit_id::TAGCLOUD,
            BarAction::ThemePanel => edit_id::THEME,
            BarAction::Audio => edit_id::AUDIO,
            BarAction::AudioPanel => edit_id::AUDIO_PANEL,
            BarAction::Color(_)
            | BarAction::TaskControl { .. }
            | BarAction::ThemeBackendToggle
            | BarAction::ThemeOpt(_, _)
            | BarAction::ThemePinSettings => return None,
        };
        Some(id.to_string())
    }

    pub fn hideable(&self) -> bool {
        self.edit_id().is_some_and(|id| id != edit_id::SETTINGS && id != edit_id::STASH)
    }
}

#[derive(Clone)]
pub(crate) struct BarNotice {
    pub(crate) state: crate::contracts::daemon::TaskState,
    pub(crate) progress: Option<f32>,
}

#[derive(Clone)]
pub struct BarModel {
    pub items: Vec<BarItem>,
    pub width: f32,
    pub height: f32,
    pub menu_up: bool,
    pub menu_visible_rows: usize,
    pub swatch_at: Option<(f32, f32)>,
    pub rtl: bool,
}

impl BarModel {
    pub fn fit_menu_rows(&mut self, rows: usize, scale: f32) {
        let visible = rows.clamp(1, self.menu_visible_rows.max(1));
        let removed = self.menu_visible_rows.saturating_sub(visible) as f32 * MENU_ROW_H * scale;
        self.height -= removed;
        if self.menu_up {
            for item in &mut self.items {
                item.y -= removed;
            }
            if let Some((_, y)) = &mut self.swatch_at {
                *y -= removed;
            }
        }
        self.menu_visible_rows = visible;
    }
}

pub(super) fn dropdown_width(current: &str, scale: f32) -> f32 {
    let size = 10.0 * scale;
    text_width(current, size, false) + 2.0 * glyph_width(size) + 44.0 * scale
}

pub(super) fn dropdown_item(
    items: &mut Vec<BarItem>,
    x: &mut f32,
    scale: f32,
    icon: &str,
    current: &str,
    menu_up: bool,
    active: bool,
    action: BarAction,
) {
    let skew = 10.0 * scale;
    let arrow = if menu_up { DROP_ARROW_UP } else { DROP_ARROW };
    let width = dropdown_width(current, scale);
    items.push(BarItem {
        x: *x,
        y: 0.0,
        w: width,
        h: 24.0 * scale,
        skew,
        label: format!("{icon} {current} {arrow}"),
        nerd: true,
        text_size: 10.0 * scale,
        swatch: None,
        notice: None,
        active,
        action: Some(action),
        z: 1,
    });
    *x += width - skew;
}
