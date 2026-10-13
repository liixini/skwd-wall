use serde_json::Value;

use crate::contracts::picker::BrowserGrid;

use super::super::Config;

const STEAM_CLIENT_BACKEND: &str = "steam";

impl Config {
    pub fn browser_grid(&self, _steam: bool) -> BrowserGrid {
        let value = |new: &str, legacy: &str, large: f64, small: f64| {
            self.get(&format!("components.wallpaperSelector.{new}"))
                .and_then(Value::as_f64)
                .unwrap_or_else(|| {
                    self.sel_num(&format!("components.wallpaperSelector.{legacy}"), large, small)
                })
        };
        BrowserGrid {
            cols: value("downloaderWallColumns", "wallhavenColumns", 6.0, 4.0) as usize,
            rows: value("downloaderWallRows", "wallhavenRows", 3.0, 3.0) as usize,
            thumb_w: value("downloaderWallThumbWidth", "wallhavenThumbWidth", 300.0, 220.0) as f32,
            thumb_h: value("downloaderWallThumbHeight", "wallhavenThumbHeight", 169.0, 124.0)
                as f32,
            gap_x: value("downloaderWallGapX", "wallhavenGapX", 16.0, 12.0) as f32,
            gap_y: value("downloaderWallGapY", "wallhavenGapY", 16.0, 12.0) as f32,
            corner_radius: value("downloaderWallCornerRadius", "wallhavenCornerRadius", 12.0, 8.0)
                as f32,
            border_width: value("downloaderWallBorderWidth", "wallhavenBorderWidth", 1.0, 1.0)
                as f32,
        }
    }

    pub fn video_preview_enabled(&self) -> bool {
        skwd_config::video_preview_enabled(&self.data)
    }

    pub fn video_preview_delay_ms(&self) -> u64 {
        skwd_config::video_preview_delay_ms(&self.data)
    }

    pub fn video_preview_fps(&self) -> u32 {
        let battery_cap = self.battery_saver_active() && skwd_config::battery_fps(&self.data) != 0;
        let cap = if battery_cap { 20.0 } else { 30.0 };
        self.max_fps().min(cap).floor().max(1.0) as u32
    }

    pub fn wallhaven_enabled(&self) -> bool {
        skwd_config::wallhaven_enabled(&self.data)
    }

    pub fn steam_uses_client(&self) -> bool {
        self.get(skwd_config::keys::steam::BACKEND)
            .and_then(Value::as_str)
            .is_none_or(|backend| backend.is_empty() || backend == STEAM_CLIENT_BACKEND)
    }

    pub fn steam_enabled(&self) -> bool {
        skwd_config::steam_enabled(&self.data)
    }

    pub fn source_enabled(&self, source: &str) -> bool {
        match source {
            "wallhaven" => self.wallhaven_enabled(),
            "steam" => self.steam_enabled(),
            "bing" | "unsplash" | "pexels" | "youtube" => {
                self.get(&format!("sources.{source}.enabled")).and_then(Value::as_bool)
                    == Some(true)
            }
            _ => false,
        }
    }

    pub fn unsplash_access_key(&self) -> String {
        skwd_config::unsplash_access_key(&self.data)
    }

    pub fn pexels_api_key(&self) -> String {
        skwd_config::pexels_api_key(&self.data)
    }

    pub fn youtube_max_minutes(&self) -> u64 {
        self.get(skwd_config::keys::sources::YOUTUBE_MAX_MINUTES)
            .and_then(Value::as_f64)
            .map_or(3, |val| (val.max(0.0) as u64).min(600))
    }
}
