use crate::frontend::settings;

#[allow(clippy::wildcard_imports)]
use super::super::*;

impl App {
    pub(in crate::app) fn sync_filter_bar_chrome(&mut self) {
        let vertical = self.config.filter_bar_orientation() == "vertical"
            && self.scene.mode != crate::frontend::scene::layout::Mode::Sandy;
        self.chrome.sync_filter_bar(vertical);
    }

    pub(in crate::app) fn motion_profile(&self) -> crate::frontend::animation::MotionProfile {
        crate::frontend::animation::MotionProfile::new(
            self.config.motion_fast_ms(),
            self.config.motion_standard_ms(),
            self.config.motion_slow_ms(),
        )
    }

    pub(in crate::app) fn apply_motion_speeds(&mut self) {
        let motion = self.motion_profile();
        self.panels.settings.set_motion_profile(motion);
        self.scene.set_motion_profile(motion);
        self.chrome.set_motion_profile(motion);
        self.source_browser.set_motion_profile(motion);
        self.tags.set_motion_profile(motion);
        self.theme.set_motion_profile(motion);
        if let Some(audio) = self.panels.audio.as_mut() {
            audio.set_motion_profile(motion);
        }
        if let Some(effects) = self.panels.effects.as_mut() {
            effects.set_motion_profile(motion);
        }
        if let Some(schedule) = self.panels.schedule.as_mut() {
            schedule.set_motion_profile(motion);
        }
    }

    pub(in crate::app) fn init_settings_inputs(&mut self) {
        self.panels.settings.inputs.clear();
        let backends = self.theme.backends.clone().unwrap_or_default();
        let cards = settings::build_tab_with_runtime_status(
            &self.panels.settings.tab,
            &self.config,
            &self.daemon.effect_themes,
            &self.library_session.folder_options,
            &self.panels.settings.semantic_import_status,
            &backends,
            &self.daemon.output_names,
            &self.daemon.output_statuses,
            &self.daemon.output_wallpaper_art,
            self.daemon.library_watch.as_ref(),
            Some(&self.daemon.playback),
        );
        for (_, rows) in &cards {
            for row in rows {
                self.init_settings_row_inputs(row);
            }
        }
        let mode = self.config.selector_mode();
        let selected = self.config.selected_preset(&mode).unwrap_or_default();
        self.panels.settings.inputs.insert(settings::PRESET_NAME_KEY.to_string(), selected);
    }

    fn init_settings_row_inputs(&mut self, row: &settings::Row) {
        match &row.control {
            settings::Control::Number { key, path, .. } => {
                let value = self.config.num_path(path.as_str());
                self.panels
                    .settings
                    .inputs
                    .insert(key.clone(), crate::contracts::picker::format_config_number(value));
            }
            settings::Control::MotionWeights { weights } => {
                for (_, key, _) in weights {
                    let value = self.config.num_path(key.as_str());
                    self.panels
                        .settings
                        .inputs
                        .insert(key.clone(), crate::contracts::picker::format_config_number(value));
                }
            }
            settings::Control::TextField { key, path, .. }
            | settings::Control::KeyBinding { key, path, .. } => {
                self.panels
                    .settings
                    .inputs
                    .insert(key.clone(), self.config.str_path(path.as_str()));
            }
            settings::Control::Details { rows, .. } | settings::Control::StackBar { rows, .. } => {
                for row in rows {
                    self.init_settings_row_inputs(row);
                }
            }
            _ => {}
        }
    }

    pub(in crate::app) fn apply_layout(&mut self) {
        let layout = layout_params(&self.config);
        let animate = self.scene.mode == layout.mode;
        if !animate {
            self.scene.begin_transition(0, [0.5, 0.5]);
        }
        self.scene.set_mode(layout.mode);
        self.scene.set_params(layout.slices, layout.grid, layout.hex, layout.extra, animate);
        self.scene.set_filter_swap_ms(self.config.filter_swap_ms());
        self.scene.set_open_fade_ms(self.config.open_fade_ms());
        self.scene.set_open_fade_from(self.config.open_fade_from());
        self.scene.set_launch_animation(&self.config.launch_animation());
        self.scene.set_card_flip_options(
            self.config.card_flip_duration_ms(),
            self.config.card_flip_shader(),
            self.config.card_flip_back_reveal(),
        );
        self.apply_motion_speeds();
        self.scene.touch();
        self.chrome.bar.cache.clear();
        self.retick();
    }

    pub(in crate::app) fn reload_bindings(&mut self) {
        self.input.bindings = crate::infrastructure::config::load_bindings(&self.config);
    }

    pub(in crate::app) fn adopt_external_config(&mut self) -> bool {
        let semantic_before = (
            self.config.str_path(skwd_config::keys::semantic::MANIFEST),
            self.config.str_path(skwd_config::keys::semantic::INDEX_PROFILE),
        );
        if !self.config.reload() {
            return false;
        }
        crate::i18n::set_language(&self.config.str_path(skwd_config::keys::general::LANGUAGE));
        let semantic_after = (
            self.config.str_path(skwd_config::keys::semantic::MANIFEST),
            self.config.str_path(skwd_config::keys::semantic::INDEX_PROFILE),
        );
        self.reload_bindings();
        self.invalidate_settings();
        self.init_settings_inputs();
        self.scene.set_preview_config(
            self.config.video_preview_enabled(),
            self.config.video_preview_delay_ms(),
            self.config.video_preview_fps(),
        );
        self.scene.set_sandy_res_scale(self.config.sandy_res_scale());
        let graphics = crate::infrastructure::capabilities::graphics_card(
            &crate::rendering::capabilities::WgpuGraphicsProbe,
        );
        self.scene.set_sandy_lod(crate::rendering::capabilities::effective_lod(
            self.config.sandy_lod(),
            self.config.sandy_lod_auto(),
            graphics.tier,
        ));
        self.apply_layout();
        if semantic_before != semantic_after {
            self.clear_semantic_search();
            self.request_semantic_search();
        }
        true
    }
}
