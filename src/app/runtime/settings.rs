use crate::contracts::settings::FolderKind;
use crate::contracts::settings::keys::library::{PROFILES, profile::NAME};
use crate::frontend::settings;

#[allow(clippy::wildcard_imports)]
use super::super::*;

impl App {
    pub(in crate::app) fn outdoor_mode(&self) -> bool {
        self.config.flag_default_config(skwd_config::keys::weather::ENABLED)
            && self.config.str_path(skwd_config::keys::weather::SOURCE)
                == skwd_config::keys::weather::SOURCE_OUTDOORS
    }

    pub(in crate::app) fn outdoor_location_key(&self) -> String {
        wall_proto::weather_location_key(
            &skwd_config::locale(self.config.root()),
            skwd_config::num_at(self.config.root(), skwd_config::keys::schedule::LATITUDE, 0.0),
            skwd_config::num_at(self.config.root(), skwd_config::keys::schedule::LONGITUDE, 0.0),
        )
    }

    pub(in crate::app) fn weather_pending(&self) -> bool {
        self.daemon.pending.values().any(|pending| *pending == Pending::WeatherAction)
    }

    pub(in crate::app) fn settings_runtime_status(&self) -> settings::RuntimeStatus<'_> {
        settings::RuntimeStatus {
            outputs: &self.daemon.output_names,
            statuses: &self.daemon.output_statuses,
            previews: Some(&self.daemon.output_wallpaper_art),
            library_watch: self.daemon.library_watch.as_ref(),
            playback: Some(&self.daemon.playback),
            app_themes: self.daemon.app_themes.as_ref(),
            weather: self.daemon.weather.as_ref(),
            weather_pending: self.weather_pending(),
            playlists: &self.daemon.weather_playlists,
            sddm_themes: &self.panels.settings.sddm_themes,
        }
    }

    pub(in crate::app) fn settings_source_ctx(&self) -> settings::SourceCtx<'_> {
        settings::SourceCtx {
            config: &self.config,
            values: &self.panels.settings.inputs,
            themes: &self.daemon.effect_themes,
            folders: &self.library_session.all_folders,
            backends: self.theme.backends.as_deref().unwrap_or(&[]),
            outputs: &self.daemon.output_names,
            output_statuses: &self.daemon.output_statuses,
            output_previews: &self.daemon.output_wallpaper_art,
            library_watch: self.daemon.library_watch.as_ref(),
            playback: Some(&self.daemon.playback),
            app_themes: self.daemon.app_themes.as_ref(),
            weather: self.daemon.weather.as_ref(),
            weather_pending: self.weather_pending(),
            playlists: &self.daemon.weather_playlists,
            sddm_themes: &self.panels.settings.sddm_themes,
            thumbnail_task: self.daemon.tasks.values().find(|task| task.id == WE_THUMBNAILS_TASK),
            analysis: self.panels.settings.semantic_import_status.clone(),
        }
    }

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
            &self.library_session.all_folders,
            &self.panels.settings.semantic_import_status,
            &backends,
            self.settings_runtime_status(),
        );
        for (_, rows) in &cards {
            for row in rows {
                self.init_settings_row_inputs(row);
            }
        }
        let mode = self.config.selector_mode();
        let selected = self.config.selected_preset(&mode).unwrap_or_default();
        self.panels.settings.inputs.insert(settings::PRESET_NAME_KEY.to_string(), selected);
        for kind in [FolderKind::Source, FolderKind::Excluded] {
            for row in settings::folder_ledger(&self.config, kind, None).rows {
                self.panels.settings.inputs.insert(row.key, row.path);
            }
        }
        for (index, entry) in self.config.array_values(PROFILES).iter().enumerate() {
            let name = entry[NAME].as_str().unwrap_or_default().to_owned();
            self.panels.settings.inputs.insert(format!("{PROFILES}.{index}.{NAME}"), name);
        }
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
            settings::Control::Resolution { key, path, .. } => {
                let value = self.config.str_path(path);
                let base = path.rsplit_once('.').map_or("", |(base, _)| base);
                let tall = self.config.str_path(&format!("{base}.orientation")) == "tall";
                let value = crate::domain::library::filter::parse_resolution(&value).map_or(
                    value,
                    |(width, height)| {
                        let (width, height) = if tall {
                            (width.min(height), width.max(height))
                        } else {
                            (width.max(height), width.min(height))
                        };
                        format!("{width}x{height}")
                    },
                );
                self.panels.settings.inputs.insert(key.clone(), value);
            }
            settings::Control::TextField { key, path, .. }
            | settings::Control::KeyBinding { key, path, .. } => {
                self.panels.settings.inputs.insert(
                    key.clone(),
                    crate::contracts::settings::SettingsSource::text(&self.config, path),
                );
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
        let effect = self.config.str_path(skwd_config::keys::selector::FLIP_EFFECT);
        if let Some(id) = EFFECT_NAMES.iter().position(|&name| name == effect)
            && self.scene.card_flip_effect() != id as u32
        {
            self.scene.set_effect(id as u32);
        }
        self.apply_motion_speeds();
        self.scene.touch();
        self.chrome.bar.cache.clear();
        self.retick();
    }

    pub(in crate::app) fn snap_layout(&mut self) {
        let layout = layout_params(&self.config);
        self.scene.set_params(layout.slices, layout.grid, layout.hex, layout.extra, false);
        self.source_browser.wall.set_base_grid(browser_wall_params(&self.config));
        if self.panels.settings.open {
            self.init_settings_inputs();
        }
    }

    pub(in crate::app) fn reload_bindings(&mut self) {
        self.input.bindings = crate::infrastructure::config::load_bindings(&self.config);
    }

    pub(in crate::app) fn adopt_external_config(&mut self) -> bool {
        let outdoor_mode_before = self.outdoor_mode();
        let source_roots_before = skwd_config::library_folders(self.config.root());
        let profiles_before = profile_config(&self.config);
        let saved_themes_before = self.config.array_values(skwd_config::keys::theme::SAVED_THEMES);
        let semantic_before = (
            self.config.flag_default_true(skwd_config::keys::semantic::ENABLED),
            self.config.str_path(skwd_config::keys::semantic::MANIFEST),
            self.config.str_path(skwd_config::keys::semantic::INDEX_PROFILE),
        );
        if !self.config.reload() {
            return false;
        }
        if outdoor_mode_before != self.outdoor_mode() {
            self.daemon.pending.retain(|_, pending| *pending != Pending::List);
            self.request_library_list();
        }
        if source_roots_before != skwd_config::library_folders(self.config.root())
            || profiles_before != profile_config(&self.config)
        {
            self.reload_library_profile();
        }
        if saved_themes_before != self.config.array_values(skwd_config::keys::theme::SAVED_THEMES) {
            self.call_tracked("effects.list", serde_json::json!({}), Pending::EffectThemes);
        }
        let previous = crate::i18n::active_script();
        crate::i18n::set_language(&self.config.str_path(skwd_config::keys::general::LANGUAGE));
        crate::shell::reload_ui_font(previous);
        let semantic_after = (
            self.config.flag_default_true(skwd_config::keys::semantic::ENABLED),
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
        self.sync_semantic_enabled();
        if semantic_before != semantic_after {
            self.clear_semantic_search();
            self.request_semantic_search();
        }
        true
    }
}

fn profile_config(
    config: &crate::infrastructure::config::Config,
) -> (String, Vec<serde_json::Value>) {
    (
        config.str_path(skwd_config::keys::library::PROFILE),
        config.array_values(skwd_config::keys::library::PROFILES),
    )
}
