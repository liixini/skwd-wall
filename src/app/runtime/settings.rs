use crate::contracts::settings::FolderKind;
use crate::contracts::settings::keys::library::{PROFILES, profile::NAME};
use crate::frontend::settings;

#[allow(clippy::wildcard_imports)]
use super::super::*;

impl App {
    pub(in crate::app) fn flipped_drawing_key(&self) -> Option<&str> {
        let index = self.scene.flip_open().then(|| self.scene.flipped()).flatten()?;
        let source = *self.library_session.filtered.get(index)? as usize;
        let item = self.library_session.library.catalog().items.get(source)?;
        crate::app::scene::SceneCore::drawing_key(item)
    }

    pub(in crate::app) fn adopt_drawing_status(&mut self, status: wall_proto::drawing::Status) {
        let card_open = self.flipped_drawing_key() == Some(status.key.as_str());
        let previous = self.daemon.drawing.as_ref();
        let completed = card_open
            && status.phase == wall_proto::drawing::Phase::Ready
            && status.enabled
            && status.prepared
            && previous.is_some_and(|previous| {
                previous.key == status.key
                    && previous.active_key == status.key
                    && previous.phase == wall_proto::drawing::Phase::Preparing
            });
        let error = card_open
            .then_some(status.error.as_deref())
            .flatten()
            .filter(|error| !error.is_empty())
            .filter(|_| {
                matches!(
                    status.phase,
                    wall_proto::drawing::Phase::Failed | wall_proto::drawing::Phase::Paused
                )
            })
            .filter(|error| {
                !previous.is_some_and(|previous| {
                    previous.key == status.key
                        && previous.phase == status.phase
                        && previous.error.as_deref() == Some(*error)
                })
            })
            .map(str::to_owned);
        if completed {
            self.adopt_external_config();
        }
        let phase = status.phase;
        let changed = if card_open {
            let state = if status.enabled && status.prepared {
                crate::frontend::scene::DrawingCardState::Drawn
            } else if status.phase == wall_proto::drawing::Phase::Checking {
                crate::frontend::scene::DrawingCardState::Checking
            } else if status.active_key == status.key {
                match status.phase {
                    wall_proto::drawing::Phase::Preparing => {
                        crate::frontend::scene::DrawingCardState::Preparing
                    }
                    wall_proto::drawing::Phase::Paused => {
                        crate::frontend::scene::DrawingCardState::Paused
                    }
                    _ => crate::frontend::scene::DrawingCardState::Draw,
                }
            } else {
                crate::frontend::scene::DrawingCardState::Draw
            };
            self.scene.set_card_drawing(&status.key, state)
        } else {
            false
        };
        self.daemon.drawing = Some(status);
        let has_error = error.is_some();
        if let Some(error) = error {
            if phase == wall_proto::drawing::Phase::Paused {
                self.show_toast(crate::i18n::tr_args!(crate::app::constants::DRAWING_CARD_STOPPED, error => error));
            } else {
                self.show_toast(crate::i18n::tr_args!(crate::app::constants::DRAWING_CARD_FAILED, error => error));
            }
        }
        if self.panels.settings.open {
            self.invalidate_settings();
        }
        if changed || has_error || self.panels.settings.open {
            self.retick();
        }
    }

    pub(in crate::app) fn drawing_view_keys(&self) -> impl Iterator<Item = &str> {
        self.library_session
            .filtered
            .iter()
            .filter_map(|&index| self.library_session.library.catalog().items.get(index as usize))
            .filter_map(crate::app::scene::SceneCore::drawing_key)
    }

    pub(in crate::app) fn drawing_view_count(&self) -> usize {
        let open = self
            .panels
            .settings
            .open_section
            .is_some_and(|section| section.key == crate::frontend::settings::DRAWING_SECTION);
        let viewing = self.config.str_path(skwd_config::keys::drawing::SCOPE)
            == skwd_config::keys::drawing::SCOPE_VIEW;
        if open || viewing { self.drawing_view_keys().count() } else { 0 }
    }

    fn drawing_job(&self) -> settings::DrawingJob {
        let running = self
            .daemon
            .tasks
            .values()
            .any(|task| task.id == wall_proto::drawing::TASK && task.state.is_active());
        let library =
            self.panels.settings.drawing_library || (self.panels.settings.library_job && running);
        settings::DrawingJob { library, running }
    }

    pub(in crate::app) fn drawing_selection(&self) -> Option<&str> {
        let index = self.scene.flipped().unwrap_or(self.scene.current);
        let catalog = *self.library_session.filtered.get(index)? as usize;
        let wallpaper = self.library_session.library.catalog().items.get(catalog)?;
        crate::app::scene::SceneCore::drawing_key(wallpaper)
    }

    pub(in crate::app) fn request_drawing_status(&mut self) {
        let key = self.drawing_selection().unwrap_or_default().to_string();
        self.call_tracked(
            wall_proto::rpc::DRAWING_STATUS,
            serde_json::json!(wall_proto::drawing::KeyRequest { key, output: None }),
            Pending::DrawingStatus,
        );
    }

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
            layout_premade: self.panels.settings.layouts.premade,
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
            drawing: self.daemon.drawing.as_ref(),
            drawing_selection: self.drawing_selection(),
            drawing_view_count: self.drawing_view_count(),
            drawing_job: self.drawing_job(),
            connected: self.daemon.connected,
        }
    }

    pub(in crate::app) fn place_source(&self) -> settings::PlaceSource<'_> {
        settings::PlaceSource {
            cfg: &self.config,
            themes: &self.daemon.effect_themes,
            folders: &self.library_session.all_folders,
            analysis: &self.panels.settings.semantic_import_status,
            backends: self.theme.backends.as_deref().unwrap_or(&[]),
            runtime: self.settings_runtime_status(),
        }
    }

    pub(in crate::app) fn page_ctx(&self) -> settings::PageCtx<'_> {
        let settings = &self.panels.settings;
        settings::PageCtx {
            cfg: &self.config,
            folders: &self.library_session.all_folders,
            watch: self.daemon.library_watch.as_ref(),
            profiles: settings::ProfilesCtx {
                selected: settings.profile,
                filter: &settings.profile_filter,
            },
            lists: settings::FoldersCtx {
                drafts: &settings.folder_drafts,
                undo: settings.folder_undo.as_ref().map(|undo| (undo.kind, undo.path.as_str())),
            },
            folds: &settings.expanded_details,
        }
    }

    pub(in crate::app) fn settings_source_ctx(&self) -> settings::SourceCtx<'_> {
        settings::SourceCtx {
            layout_premade: self.panels.settings.layouts.premade,
            config: &self.config,
            values: &self.panels.settings.inputs,
            themes: &self.daemon.effect_themes,
            folders: &self.library_session.all_folders,
            backends: self.theme.backends.as_deref().unwrap_or(&[]),
            outputs: &self.daemon.output_names,
            output_statuses: &self.daemon.output_statuses,
            output_previews: &self.daemon.output_wallpaper_art,
            drawing: self.daemon.drawing.as_ref(),
            drawing_selection: self.drawing_selection(),
            drawing_view_count: self.drawing_view_count(),
            drawing_library: self.drawing_job().library,
            connected: self.daemon.connected,
            library_watch: self.daemon.library_watch.as_ref(),
            playback: Some(&self.daemon.playback),
            app_themes: self.daemon.app_themes.as_ref(),
            weather: self.daemon.weather.as_ref(),
            weather_pending: self.weather_pending(),
            playlists: &self.daemon.weather_playlists,
            sddm_themes: &self.panels.settings.sddm_themes,
            thumbnail_task: self.daemon.tasks.values().find(|task| task.id == WE_THUMBNAILS_TASK),
            drawing_task: self
                .daemon
                .tasks
                .values()
                .find(|task| task.id == wall_proto::drawing::TASK),
            analysis: self.panels.settings.semantic_import_status.clone(),
        }
    }

    pub(in crate::app) fn sync_filter_bar_chrome(&mut self) {
        let vertical = self.config.filter_bar_orientation() == "vertical"
            && self.scene.mode != crate::frontend::scene::layout::Mode::Sandy;
        self.chrome.sync_filter_bar(vertical);
    }

    pub(in crate::app) fn apply_bar_default(&mut self) {
        let mode = self.config.filter_bar_visibility();
        self.chrome.filter_bar_mode = mode;
        self.chrome.filter_bar_visible =
            self.chrome.filter_bar_forced || mode == skwd_config::FilterBarVisibility::Always;
        self.chrome.clear_filter_hover();
        self.chrome.bar.cache.clear();
        self.retick();
    }

    pub(in crate::app) fn follow_bar_mode(&mut self) {
        if self.panels.arrange.is_none()
            && self.chrome.filter_bar_mode != self.config.filter_bar_visibility()
        {
            self.apply_bar_default();
        }
    }

    pub(in crate::app) fn place_hover_zone(&mut self) {
        let context =
            crate::app::view::layout::LayoutContext::projected(&self.config, self.scene.viewport);
        let rect = crate::app::view::bar_bounds(self, &context);
        self.chrome.filter_bar_hover_zone = crate::app::update::arrange::placement::resolve(
            self,
            crate::frontend::arrange::Target::Bar,
            rect,
        );
        self.chrome.bar.cache.zone_placed();
    }

    pub(in crate::app) fn filter_hover_allowed(&self) -> bool {
        self.chrome.filter_bar_mode == skwd_config::FilterBarVisibility::Hover
            && self.panels.arrange.is_none()
            && self.runtime_state.demo.is_none()
            && !self.menu_capturing()
            && !self.detail_open()
    }

    pub(in crate::app) fn hover_filter_bar(&mut self, x: f32, y: f32) {
        if self.chrome.bar.menu.is_some() || !self.filter_hover_allowed() {
            return;
        }
        if self.chrome.bar.cache.zone_stale() {
            self.place_hover_zone();
        }
        let inside = self.chrome.filter_bar_hover_zone.contains(iced::Point::new(x, y));
        if self.chrome.track_filter_hover(inside) {
            self.retick();
        }
    }

    pub(in crate::app) fn leave_filter_bar(&mut self) {
        if self.chrome.clear_filter_hover() {
            self.retick();
        }
    }

    pub(in crate::app) fn apply_search_default(&mut self) {
        if self.config.search_always_visible() && !self.tags.cloud_open {
            self.reveal_tag_cloud();
            self.retick();
        }
    }

    pub(in crate::app) fn follow_search_setting(&mut self) {
        if self.config.search_always_visible() {
            self.apply_search_default();
        } else if self.tags.cloud_open {
            self.close_overlay(crate::app::overlay::Overlay::TagCloud);
        }
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
        let place = self.place_source().place(&self.panels.settings.tab);
        for row in place.sections.iter().flat_map(settings::Section::rows) {
            self.init_settings_row_inputs(row);
        }
        for kind in [FolderKind::Source, FolderKind::Excluded] {
            for row in settings::folder_ledger(&self.config, kind, None).rows {
                self.panels.settings.inputs.insert(row.key, row.path);
            }
            let draft = self.panels.settings.folder_drafts.get(kind.key()).cloned();
            self.panels
                .settings
                .inputs
                .insert(settings::draft_key(kind), draft.unwrap_or_default());
        }
        let filter = self.panels.settings.profile_filter.clone();
        self.panels.settings.inputs.insert(settings::FILTER_KEY.to_string(), filter);
        for (index, entry) in self.config.array_values(PROFILES).iter().enumerate() {
            let name = entry[NAME].as_str().unwrap_or_default().to_owned();
            self.panels.settings.inputs.insert(format!("{PROFILES}.{index}.{NAME}"), name);
        }
    }

    pub(in crate::app) fn seed_search_inputs(&mut self) {
        let rows: Vec<settings::Row> = self
            .panels
            .settings
            .search_results
            .iter()
            .flat_map(|result| {
                let home = result.home.iter().flat_map(settings::Section::rows);
                result.setting.iter().chain(home).cloned().collect::<Vec<_>>()
            })
            .collect();
        for row in rows {
            let known = row
                .control
                .token()
                .is_some_and(|key| self.panels.settings.inputs.contains_key(&key));
            if !known {
                self.init_settings_row_inputs(&row);
            }
        }
    }

    fn init_settings_row_inputs(&mut self, row: &settings::Row) {
        match &row.control {
            settings::Control::ActionBtn {
                id: settings::ActionId::ChooseRunningProcess, ..
            } => {
                let key = skwd_config::keys::playback::PROCESSES;
                self.panels.settings.inputs.insert(key.to_string(), self.config.str_path(key));
            }
            settings::Control::Number { key, path, .. } => {
                let value = self.config.num_path(path.as_str());
                self.panels
                    .settings
                    .inputs
                    .insert(key.clone(), crate::contracts::picker::format_config_number(value));
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
        let reduced = self.config.reduced_motion();
        let animate = self.scene.mode == layout.mode && !reduced;
        if self.scene.mode != layout.mode && !reduced {
            self.scene.begin_transition(0, [0.5, 0.5]);
        }
        self.scene.set_mode(layout.mode);
        self.scene.set_picker_frame(crate::app::update::arrange::placement::frame(self));
        self.scene.set_params(layout.slices, layout.grid, layout.hex, &layout.extra, animate);
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
        self.follow_bar_mode();
        self.scene.touch();
        self.chrome.bar.cache.clear();
        self.retick();
    }

    pub(in crate::app) fn snap_layout(&mut self) {
        self.scene.set_picker_frame(crate::app::update::arrange::placement::frame(self));
        let layout = layout_params(&self.config);
        self.scene.set_params(layout.slices, layout.grid, layout.hex, &layout.extra, false);
        self.source_browser.wall.set_base_grid(browser_wall_params(&self.config));
        if self.panels.settings.open {
            self.init_settings_inputs();
        }
    }

    pub(in crate::app) fn reload_bindings(&mut self) {
        self.input.bindings = crate::infrastructure::config::load_bindings(&self.config);
    }

    pub(in crate::app) fn sync_picker_preview(&mut self) {
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
        let search_default = self.config.search_always_visible();
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
        self.sync_picker_preview();
        self.apply_layout();
        self.sync_semantic_enabled();
        if search_default != self.config.search_always_visible() {
            self.follow_search_setting();
        }
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
