use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use super::layout::Hit;
use crate::contracts::rendering::RendererSnapshot;

#[derive(Debug, Clone, Copy)]
pub struct Chrome {
    pub view: u8,
    pub cx: f32,
    pub cy: f32,
    pub hw: f32,
    pub hh: f32,
    pub skew: f32,
    pub edge_tilt: f32,
    pub kind: u8,
    pub has_video: bool,
    pub favourite: bool,
    pub radius: f32,
    pub opacity: f32,
}

#[derive(Debug, Default, Clone)]
pub struct RenderSnapshot {
    pub(crate) renderer: Arc<RendererSnapshot>,
    pub hits: Vec<Hit>,
    pub chrome: Vec<Chrome>,
    pub placeholders: bool,
    pub back: Option<BackPanel>,
    pub overlay: bool,
}

impl Deref for RenderSnapshot {
    type Target = RendererSnapshot;

    fn deref(&self) -> &Self::Target {
        &self.renderer
    }
}

impl DerefMut for RenderSnapshot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.renderer)
    }
}

#[derive(Debug, Default, Clone)]
pub struct BackPanel {
    pub cx: f32,
    pub cy: f32,
    pub hw: f32,
    pub hh: f32,
    pub skew: f32,
    pub edge_tilt: f32,
    pub radii: [f32; 4],
    pub progress: f32,
    pub coordinated_flip: bool,
    pub embedded: bool,
    pub animate_flip_shader: bool,
    pub animate_flip_back: bool,
    pub title: String,
    pub kind_label: String,
    pub fields: Vec<(String, String)>,
    pub tags: Vec<String>,
    pub favourite: bool,
    pub fav_fill: f32,
    pub add_open: f32,
    pub chip_pop: f32,
    pub pop_idx: i32,
    pub static_img: bool,
    pub overview_available: bool,
    pub scene_properties: bool,
    pub reset_thumbnail: bool,
}
