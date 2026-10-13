use super::super::Config;

impl Config {
    pub fn wallpaper_mute(&self) -> bool {
        skwd_config::wallpaper_mute(&self.data)
    }

    pub fn wallpaper_volume(&self) -> u32 {
        skwd_config::wallpaper_volume(&self.data)
    }

    pub fn cache_dir(&self) -> String {
        skwd_config::cache_dir_of(&self.data)
    }

    pub fn wallpaper_dir(&self) -> String {
        skwd_config::wallpaper_dir(&self.data)
    }

    pub fn video_dir(&self) -> String {
        skwd_config::video_dir(&self.data)
    }

    pub fn ui_scale(&self) -> f32 {
        skwd_config::schema::setting::general::UI_SCALE.read(self.root()) as f32
    }

    pub fn open_fade_ms(&self) -> f32 {
        self.launch_motion_ms()
    }

    pub fn open_fade_from(&self) -> f32 {
        (skwd_config::schema::setting::general::OPEN_FADE_FROM.read(self.root()) / 100.0) as f32
    }

    pub fn max_fps(&self) -> f32 {
        let configured =
            (skwd_config::schema::setting::general::MAX_FPS.read(self.root()) as f32).max(1.0);
        skwd_config::effective_picker_fps(&self.data, self.on_battery, configured).max(1.0)
    }

    pub fn battery_saver_active(&self) -> bool {
        self.on_battery && skwd_config::battery_saver_enabled(&self.data)
    }

    pub fn frame_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f32(1.0 / self.max_fps())
    }

    pub fn transition_preview_fps(&self) -> f32 {
        (self.num_path(skwd_config::keys::transition::PREVIEW_FPS) as f32)
            .max(1.0)
            .min(self.max_fps())
    }

    pub fn transition_preview_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f32(1.0 / self.transition_preview_fps())
    }

    pub fn theme_backend(&self) -> String {
        skwd_config::theme_backend(&self.data)
    }
}
