use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures_channel::mpsc::UnboundedSender;
use iced::widget::canvas;

use crate::contracts::preview::{UploadQueue, compressed_thumbnails};
use crate::domain::library::catalog::{Catalog, Wallpaper, WallpaperKind};
use crate::frontend::animation::{MotionProfile, MotionTier, Tween};
use crate::frontend::browser::{Browser, Source};
use crate::frontend::scene::layout::{ExtraParams, GridParams, HexParams, Mode, SliceParams};
use crate::infrastructure::preview::DecodePool;
use crate::infrastructure::runtime::Wake;
use crate::rendering::scene::atlas::{AtlasMap, browser_budget};

use crate::app::scene::SceneCore;

use super::BrowserMediaState;

pub(crate) struct BrowserWallState {
    pub(crate) scene: SceneCore,
    pub(crate) catalog: Catalog,
    pub(crate) filtered: Vec<u32>,
    pub(crate) atlas: Option<AtlasMap>,
    pub(crate) decoder: DecodePool,
    pub(crate) uploads: UploadQueue,
    pub(crate) chrome_cache: canvas::Cache,
    pub(crate) decoder_was_busy: bool,
    base_grid: GridParams,
    item_indices: HashMap<String, u32>,
}

impl BrowserWallState {
    pub(crate) fn new(
        sp: SliceParams,
        base_grid: GridParams,
        hp: HexParams,
        xp: &ExtraParams,
        motion: MotionProfile,
        wake: UnboundedSender<Wake>,
    ) -> Self {
        let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
        let decoder = DecodePool::start(uploads.clone(), wake, 2);
        let mut scene = SceneCore::new(Mode::Grid, sp, base_grid, hp, xp);
        scene.set_motion_profile(motion);
        scene.set_preview_config(false, 0, 1);
        scene.set_direct_thumbnails(true);
        Self {
            scene,
            catalog: Catalog::default(),
            filtered: Vec::new(),
            atlas: Some(AtlasMap::with_budget(768, browser_budget(compressed_thumbnails()))),
            decoder,
            uploads,
            chrome_cache: canvas::Cache::new(),
            decoder_was_busy: false,
            base_grid,
            item_indices: HashMap::new(),
        }
    }

    pub(crate) fn layout_grid(&self) -> crate::contracts::picker::BrowserGrid {
        crate::contracts::picker::BrowserGrid {
            cols: self.base_grid.cols,
            rows: self.base_grid.rows,
            thumb_w: self.base_grid.thumb_w,
            thumb_h: self.base_grid.thumb_h,
            gap_x: self.base_grid.gap_x,
            gap_y: self.base_grid.gap_y,
            corner_radius: self.base_grid.corners.sizes[0],
            border_width: self.base_grid.border_width,
        }
    }

    pub(crate) fn set_layout(&mut self, grid: GridParams) {
        self.base_grid = grid;
        self.fit_layout();
    }

    pub(crate) fn set_viewport(&mut self, width: f32, height: f32) {
        let width = width.max(1.0);
        let height = height.max(1.0);
        if self.scene.viewport == (width, height) {
            return;
        }
        self.scene.viewport = (width, height);
        self.fit_layout();
    }

    pub(crate) fn set_base_grid(&mut self, grid: GridParams) {
        self.base_grid = grid;
        self.fit_layout();
    }

    fn fit_layout(&mut self) {
        let (width, height) = self.scene.viewport;
        let (width, height) = (width.max(1.0), height.max(1.0));
        let mut grid = self.base_grid;
        let cols = grid.cols.max(1) as f32;
        let rows = grid.rows.max(1) as f32;
        let raw_w = cols * grid.thumb_w + (cols - 1.0) * grid.gap_x;
        let raw_h = rows * grid.thumb_h + (rows - 1.0) * grid.gap_y;
        let fit = (width / raw_w.max(1.0)).min(height / raw_h.max(1.0)).min(1.0);
        grid.gap_x *= fit;
        grid.gap_y *= fit;
        grid.thumb_w = (width - (cols - 1.0) * grid.gap_x) / cols;
        grid.thumb_h = (height - (rows - 1.0) * grid.gap_y) / rows;
        grid.corners = grid.corners.scaled(fit);
        grid.border_width *= fit;
        self.scene.set_grid_params(grid, false);
        self.scene.touch();
    }

    pub(crate) fn rebuild_catalogue(&mut self, browser: &Browser, reset_camera: bool) {
        let source = browser.source.key();
        let kind = if browser.source.apply_kind() == crate::contracts::browser::ApplyKind::Static {
            WallpaperKind::Static
        } else {
            WallpaperKind::Video
        };
        let mut filtered = Vec::with_capacity(browser.session.items.len());
        for item in &browser.session.items {
            let identity = format!("{source}:{}", item.id);
            let (width, height) = parse_resolution(&item.resolution);
            let wallpaper = Wallpaper {
                key: identity.clone(),
                name: item.title.clone(),
                kind,
                thumb: if item.thumb_ready { item.thumb_path.clone() } else { String::new() },
                path: item.full_url.clone(),
                preview: item.full_url.clone(),
                width,
                height,
                filesize: i64::try_from(item.file_size).unwrap_or(i64::MAX),
                ..Wallpaper::default()
            };
            let store = if let Some(&store) = self.item_indices.get(&identity) {
                self.catalog.items[store as usize] = wallpaper;
                store
            } else {
                let store = self.catalog.items.len() as u32;
                self.catalog.items.push(wallpaper);
                self.item_indices.insert(identity, store);
                store
            };
            filtered.push(store);
        }
        self.filtered = filtered;
        if reset_camera {
            self.scene.reset_to_start(self.filtered.len());
        } else {
            self.scene.relayout(self.filtered.len());
        }
        self.scene.touch();
        self.chrome_cache.clear();
    }

    pub(crate) fn update_thumb(&mut self, source: &str, id: &str, path: &str) {
        let identity = format!("{source}:{id}");
        let Some(&store) = self.item_indices.get(&identity) else {
            return;
        };
        self.catalog.items[store as usize].thumb = path.to_string();
        if let Some(atlas) = self.atlas.as_mut() {
            atlas.near.release(store as usize);
            atlas.far.release(store as usize);
            atlas.near_failed.remove(&(store as usize));
            atlas.failed.remove(&(store as usize));
        }
        self.scene.touch();
    }

    pub(crate) fn begin_session(&mut self) {
        self.scene.reset_to_start(self.filtered.len());
        self.scene.touch();
        self.chrome_cache.clear();
    }
}

fn parse_resolution(value: &str) -> (i64, i64) {
    let normalized = value.replace('×', "x").replace(' ', "");
    let mut parts = normalized.split('x');
    let width = parts.next().and_then(|part| part.parse().ok()).unwrap_or(0);
    let height = parts.next().and_then(|part| part.parse().ok()).unwrap_or(0);
    (width, height)
}

pub(crate) struct SourceBrowserState {
    pub(crate) media: BrowserMediaState,
    pub(crate) browser: Option<Browser>,
    pub(crate) tabs: HashMap<Source, Browser>,
    pub(crate) search_generation: u64,
    pub(crate) last_source: Source,
    pub(crate) entrance: Tween,
    pub(crate) spinner_phase: f32,
    pub(crate) shimmer_phase: f32,
    pub(crate) preview_animation: f32,
    pub(crate) preview_closing: bool,
    pub(crate) motion: MotionProfile,
    pub(crate) wall: BrowserWallState,
}

impl SourceBrowserState {
    #[allow(clippy::large_types_passed_by_value)]
    pub(crate) fn new(
        motion: MotionProfile,
        layout: (SliceParams, GridParams, HexParams, ExtraParams),
        wake: UnboundedSender<Wake>,
    ) -> Self {
        Self {
            media: BrowserMediaState::default(),
            browser: None,
            tabs: HashMap::new(),
            search_generation: 0,
            last_source: Source::Wallhaven,
            entrance: motion.tween(0.0, MotionTier::Slow),
            spinner_phase: 0.0,
            shimmer_phase: 0.0,
            preview_animation: 0.0,
            preview_closing: false,
            motion,
            wall: BrowserWallState::new(layout.0, layout.1, layout.2, &layout.3, motion, wake),
        }
    }

    pub(crate) fn set_motion_profile(&mut self, motion: MotionProfile) {
        self.motion = motion;
        for browser in self.browser.iter_mut().chain(self.tabs.values_mut()) {
            browser.view.motion = motion;
        }
        motion.retime_tween(&mut self.entrance, MotionTier::Slow);
        self.wall.scene.set_motion_profile(motion);
    }

    pub(crate) fn activate(&mut self, source: Source) -> bool {
        if self.browser.as_ref().is_some_and(|browser| browser.source == source) {
            return false;
        }
        if let Some(mut browser) = self.browser.take() {
            browser.session.preview = None;
            self.tabs.insert(browser.source, browser);
        }
        let mut browser = self.tabs.remove(&source).unwrap_or_else(|| Browser::new(source));
        browser.session.preview = None;
        browser.view.motion = self.motion;
        self.browser = Some(browser);
        self.last_source = source;
        true
    }

    pub(crate) fn browser_for_source_mut(&mut self, source: Source) -> Option<&mut Browser> {
        if self.browser.as_ref().is_some_and(|browser| browser.source == source) {
            return self.browser.as_mut();
        }
        self.tabs.get_mut(&source)
    }

    pub(crate) fn browser_for_download_mut(&mut self, id: &str) -> Option<&mut Browser> {
        let active_match = self.source_browser_matches_download(id);
        if active_match {
            return self.browser.as_mut();
        }
        self.tabs.values_mut().find(|browser| browser_matches_download(browser, id))
    }

    fn source_browser_matches_download(&self, id: &str) -> bool {
        self.browser.as_ref().is_some_and(|browser| browser_matches_download(browser, id))
    }

    pub(crate) fn close(&mut self) {
        self.browser = None;
        self.tabs.clear();
    }
}

fn browser_matches_download(browser: &Browser, id: &str) -> bool {
    browser.session.pending_apply.as_deref() == Some(id)
        || browser.session.items.iter().any(|item| item.id == id && item.downloading)
}
