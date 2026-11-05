use crate::app::{App, startup::LayoutParams};
use crate::frontend::scene::layout::Mode;
use crate::infrastructure::config::Config;

const SLICE_CHROME: f32 = 110.0;
const GRID_CHROME: f32 = 85.0;
const HEX_CHROME: f32 = 140.0;
const OVERLAP_CHROME: f32 = 90.0;
const OVERLAP_MIN: f32 = 200.0;
pub(super) const VERTICAL: &str = "vertical";

pub(in crate::app) struct LayoutContext<'a> {
    pub config: &'a Config,
    pub viewport: (f32, f32),
    pub params: LayoutParams,
    pub vertical: bool,
    pub entrance: f32,
}

impl<'a> LayoutContext<'a> {
    pub(in crate::app) fn current(app: &'a App, viewport: (f32, f32)) -> Self {
        Self {
            config: &app.config,
            viewport,
            params: LayoutParams {
                mode: app.scene.mode,
                slices: app.scene.sp,
                grid: app.scene.gp,
                hex: app.scene.hp,
                extra: app.scene.xp,
            },
            vertical: app.chrome.filter_bar_vertical,
            entrance: if app.panels.arrange.is_some() { 1.0 } else { app.tags.cloud_entrance.x },
        }
    }

    pub(in crate::app) fn projected(config: &'a Config, viewport: (f32, f32)) -> Self {
        Self {
            config,
            viewport,
            params: crate::app::startup::layout_params(config),
            vertical: config.filter_bar_orientation() == VERTICAL,
            entrance: 1.0,
        }
    }

    pub(in crate::app) fn card_height(&self) -> f32 {
        match self.params.mode {
            Mode::Slices | Mode::Depth => self.params.slices.slice_h + SLICE_CHROME,
            Mode::Grid => self.params.grid.total_h() + GRID_CHROME,
            Mode::Hex => self.params.hex.content_h() + HEX_CHROME,
            Mode::Sandy | Mode::Hand | Mode::Collection => {
                (self.viewport.1 - OVERLAP_CHROME).max(OVERLAP_MIN)
            }
        }
    }
}
