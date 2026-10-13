use super::super::super::misc::{glyph_width, text_width};
use super::super::action::BarAction;
use super::super::catalog::{DROP_ARROW, DROP_ARROW_UP};
#[cfg(test)]
use super::super::catalog::{SORTS, TYPES};
use crate::contracts::settings::wallpaper_kind::{STATIC, VIDEO, WE};
use crate::domain::library::filter::ResolutionPreset;

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
    #[cfg(test)]
    pub fn all() -> Self {
        Self {
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

#[derive(Clone)]
pub(crate) struct BarNotice {
    pub(crate) state: crate::contracts::daemon::TaskState,
    pub(crate) progress: Option<f32>,
}

pub struct BarModel {
    pub items: Vec<BarItem>,
    pub width: f32,
    pub height: f32,
    pub menu_up: bool,
    pub swatch_at: Option<(f32, f32)>,
    pub rtl: bool,
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
