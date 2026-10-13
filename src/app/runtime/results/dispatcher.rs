use crate::contracts::daemon::ActivityCompletion;
use log::warn;
use serde_json::Value;

const REMOVE_ERROR: &str = "card-back-remove-error";
const OUTDOOR_ACTION_ERROR: &str = "settings-outdoors-action-error";
const LOCKSCREEN_MISMATCH: &str = "Mismatched lockscreen result";

#[allow(clippy::wildcard_imports)]
use crate::app::*;

impl App {
    fn error_toast(&mut self, key: &str, error: &str) {
        self.show_toast(crate::i18n::tr_args!(key, error => error));
        self.scene.touch();
        self.retick();
    }

    fn removal_error(&mut self, unsubscribe: bool, error: &str) {
        self.error_toast(REMOVE_ERROR, error);
        if !unsubscribe {
            self.call_tracked(wall_proto::rpc::WALL_LIST, serde_json::json!({}), Pending::List);
        }
    }

    fn weather_rpc_error(&mut self) {
        let previous = self.daemon.weather.is_some();
        if !previous && self.outdoor_mode() {
            self.daemon.weather = Some(crate::contracts::daemon::WeatherStatus {
                enabled: true,
                source: crate::contracts::daemon::WeatherSource::Outdoors,
                status: crate::contracts::daemon::WeatherState::Unavailable,
                location_key: Some(self.outdoor_location_key()),
                outdoors: Some(crate::contracts::daemon::OutdoorStatus::default()),
                ..Default::default()
            });
        }
        if let Some(weather) = self.daemon.weather.as_mut() {
            weather.status = crate::contracts::daemon::WeatherState::Unavailable;
            if let Some(outdoors) = weather.outdoors.as_mut() {
                outdoors.stale = previous;
            }
        }
        self.retick();
    }

    fn weather_action_error(&mut self, error: &str) {
        self.show_toast(crate::i18n::tr_args!(OUTDOOR_ACTION_ERROR, error => error));
        self.call_tracked(wall_proto::rpc::WALL_WEATHER, serde_json::json!({}), Pending::Weather);
        self.retick();
    }

    fn app_theme_error(&mut self, kind: &Pending, error: &str) {
        if matches!(kind, Pending::AppThemes) {
            self.daemon.app_themes = None;
        } else {
            self.call_tracked("theme.apps", serde_json::json!({}), Pending::AppThemes);
        }
        self.show_toast(crate::i18n::tr_args!("settings-app-themes-error", error => error));
        self.invalidate_settings();
        self.retick();
    }

    #[expect(clippy::too_many_lines, reason = "RPC error dispatch boundary")]
    pub(in crate::app) fn rpc_error(&mut self, kind: Pending, err: String) {
        match kind {
            Pending::Weather => self.weather_rpc_error(),
            Pending::WeatherCandidates { category, query, generation } => {
                let review = &mut self.panels.settings;
                if review.weather_review_category == Some(category.as_str())
                    && review.weather_review_generation == generation
                    && crate::app::update::weather_review::query_key(&category)
                        .is_some_and(|path| self.config.str_path(path).trim() == query)
                {
                    review.weather_review_loading = false;
                    review.weather_review_error = Some(err);
                    self.retick();
                }
            }
            Pending::WeatherAction => self.weather_action_error(&err),
            Pending::Status => {
                self.daemon.steam_client_available = None;
                if self.daemon.steam_refresh_needed {
                    self.refresh_unsubscribe_status();
                }
                self.scene.touch();
                self.retick();
            }
            Pending::RemoveWallpaper { unsubscribe, .. } => self.removal_error(unsubscribe, &err),
            Pending::AppThemeSet | Pending::AppThemes => self.app_theme_error(&kind, &err),
            Pending::LockScreenSet { target, enabled, sddm_theme } => {
                if target == wall_proto::LockScreenTarget::Sddm {
                    self.error_toast(crate::app::constants::LOCKSCREEN_ERROR, &err);
                    self.panels.settings.sddm_rollback = Some(crate::app::state::SddmRollback {
                        enabled: !enabled,
                        theme: sddm_theme,
                        error: err,
                    });
                    self.panels.settings.sddm_checking = true;
                    return;
                }
                self.error_toast(crate::app::constants::LOCKSCREEN_ERROR, &err);
            }
            Pending::ResetThumbnail { .. } => {
                self.error_toast("card-back-reset-thumbnail-error", &err);
            }
            Pending::AudioPause => {
                self.error_toast("audio-playback-error", &err);
            }
            Pending::CurrentTheme { .. } => {
                if let Some(designer) = self.panels.theme_designer.as_mut() {
                    designer.error = Some(err);
                }
                self.retick();
            }
            Pending::EffectsPreview { source, .. } => {
                if let Some(eff) =
                    self.panels.effects.as_mut().filter(|effects| effects.source_path() == source)
                {
                    eff.fail_request(&err);
                }
            }
            Pending::EffectsCommit { .. } => {
                if let Some(eff) = self.panels.effects.as_mut() {
                    eff.fail_request(&err);
                }
            }
            Pending::BrowserSearch { source, generation, .. } => {
                if let Some(br) = self.source_browser.browser_for_source_mut(source) {
                    if generation != br.session.search_generation {
                        return;
                    }
                    br.session.loading = false;
                    br.session.page_failed = true;
                    br.session.error = Some(err);
                }
                self.retick();
            }
            Pending::BrowserDownload { source, id: did } => {
                if let Some(br) = self.source_browser.browser_for_source_mut(source) {
                    if let Some(it) = br.item_mut(&did) {
                        it.downloading = false;
                    }
                    br.session.pending_apply = None;
                    br.session.error = Some(err);
                }
                self.retick();
            }
            Pending::BrowserPreview { source, .. } => {
                if let Some(br) = self.source_browser.browser_for_source_mut(source) {
                    br.session.error = Some(err);
                }
                self.retick();
            }
            Pending::ThemePreview { card, .. } => {
                if self.theme.job_pending == Some(card) {
                    self.theme.job_pending = None;
                }
                self.retick();
            }
            Pending::ThemePreviews { backend } => {
                if self.theme.audition_pending_backend.as_deref() == Some(backend.as_str()) {
                    self.theme.audition_pending_backend = None;
                }
                if self.theme.audition_backend == backend {
                    self.theme.audition_loading = false;
                    self.theme.audition_error = Some(err);
                    self.retick();
                }
            }
            Pending::SceneProperties { we_id, revision, .. } => {
                if self
                    .panels
                    .scene_properties
                    .as_ref()
                    .is_some_and(|panel| panel.we_id == we_id && panel.revision <= revision)
                {
                    self.on_scene_properties_error(&err);
                }
            }
            request @ (Pending::Activities | Pending::ActivityOutputs { .. }) => {
                self.fail_activity_outputs(&request, err);
            }
            Pending::ActivityApply { token } => {
                self.complete_activity_request(&token, ActivityCompletion::Failed(err));
            }
            _ => {}
        }
    }

    fn fail_activity_outputs(&mut self, request: &Pending, error: String) {
        let Some(effects) = self.panels.effects.as_mut() else { return };
        if let Pending::ActivityOutputs { activity, generation } = request
            && (effects.activity.selected != *activity
                || effects.activity.generation != *generation)
        {
            return;
        }
        effects.activity.ready = false;
        effects.activity.error = Some(error);
        effects.clear_monitors();
        self.retick();
    }

    pub(in crate::app) fn on_result(&mut self, kind: Pending, result: &Value) {
        let request_for_error = kind.clone();
        macro_rules! decoded {
            (@take $family:expr, $value:expr) => {{
                match $value {
                    Ok(value) => value,
                    Err(error) => {
                        let message = format!("invalid {} result: {error}", $family);
                        warn!("{message}");
                        self.rpc_error(request_for_error.clone(), message);
                        return;
                    }
                }
            }};
            ($family:expr, $result:expr, $decode:path $(,)?) => {
                decoded!(@take $family, $decode($result))
            };
        }
        match kind {
            Pending::RemoveWallpaper { .. } => {
                decoded!(
                    wall_proto::rpc::WALL_REMOVE,
                    result,
                    crate::infrastructure::rpc_results::decode_wall_remove,
                );
                self.call_tracked(wall_proto::rpc::WALL_LIST, serde_json::json!({}), Pending::List);
                self.scene.touch();
                self.retick();
            }
            Pending::PlaybackProcesses => {
                self.daemon.playback.available_processes = decoded!(
                    "playback.processes",
                    result,
                    crate::infrastructure::rpc_results::decode_running_processes
                );
                self.retick();
            }
            Pending::CurrentTheme { load } => {
                match crate::infrastructure::rpc_results::decode_current_theme(result) {
                    Ok(current) => {
                        if let Some(designer) = self.panels.theme_designer.as_mut() {
                            if load {
                                let mut candidate = current.palette.clone();
                                if let Some(profile) = self
                                    .config
                                    .array_slice(skwd_config::keys::theme::WALLPAPER_PROFILES)
                                    .iter()
                                    .find(|profile| {
                                        profile["key"].as_str() == Some(&current.key)
                                            && profile["enabled"].as_bool() == Some(true)
                                    })
                                {
                                    let dark = !current.dark;
                                    if let Some(saved) =
                                        profile.get(if dark { "dark" } else { "light" })
                                    {
                                        let variant =
                                            crate::infrastructure::theme::decode_candidate_variant(
                                                saved, dark,
                                            );
                                        candidate.set_dark(dark);
                                        candidate.colors = variant.colors;
                                    }
                                }
                                candidate.set_dark(current.dark);
                                designer.start_from(candidate);
                            }
                            designer.profile_enabled = self
                                .config
                                .array_slice(skwd_config::keys::theme::WALLPAPER_PROFILES)
                                .iter()
                                .any(|profile| {
                                    profile["key"].as_str() == Some(&current.key)
                                        && profile["enabled"].as_bool() == Some(true)
                                });
                            designer.settings_pinned = self
                                .config
                                .array_slice(skwd_config::keys::theme::WALLPAPER_PROFILES)
                                .iter()
                                .any(|profile| {
                                    profile["key"].as_str() == Some(&current.key)
                                        && profile["settingsPinned"] == true
                                });
                            designer.wallpaper = Some(current);
                            designer.error = None;
                        }
                    }
                    Err(error) => self.rpc_error(Pending::CurrentTheme { load }, error.to_string()),
                }
                self.retick();
            }
            Pending::List => {
                let paths = crate::infrastructure::library::LibraryPaths::new(
                    &self.library_session.wallpaper_dir,
                    &self.library_session.video_dir,
                );
                let catalog = decoded!(@take
                    "wall.list",
                    crate::infrastructure::rpc_results::decode_library_list(result, paths)
                );
                self.on_list(catalog);
            }
            Pending::EffectsPreview { source, cache_key } => self.on_effects_preview(
                decoded!(
                    "effects.preview",
                    result,
                    crate::infrastructure::rpc_results::decode_effect_operation,
                ),
                &source,
                &cache_key,
            ),
            Pending::ThemePreview { card, backend } => {
                self.on_theme_preview(
                    decoded!(
                        "theme.preview",
                        result,
                        crate::infrastructure::rpc_results::decode_theme_preview,
                    ),
                    card,
                    &backend,
                );
            }
            Pending::ThemePreviews { backend } => self.on_theme_previews(
                decoded!(
                    "theme.previews",
                    result,
                    crate::infrastructure::rpc_results::decode_theme_previews,
                ),
                &backend,
            ),
            Pending::Outputs => self.on_outputs(decoded!(
                "wall.outputs",
                result,
                crate::infrastructure::rpc_results::decode_outputs,
            )),
            Pending::Activities => self.on_activities(decoded!(
                wall_proto::rpc::WALL_ACTIVITIES,
                result,
                crate::infrastructure::rpc_results::decode_activities,
            )),
            Pending::ActivityOutputs { activity, generation } => self.on_activity_outputs(
                &decoded!(
                    "wall.outputs",
                    result,
                    crate::infrastructure::rpc_results::decode_outputs,
                ),
                &activity,
                generation,
            ),
            Pending::ActivityApply { token } => {
                let result = decoded!(
                    wall_proto::rpc::WALL_APPLY,
                    result,
                    crate::infrastructure::rpc_results::decode_activity_apply,
                );
                self.on_activity_apply_response(&token, &result);
            }
            Pending::DemoOutputs => self.on_demo_outputs(decoded!(
                wall_proto::rpc::WALL_OUTPUTS,
                result,
                crate::infrastructure::rpc_results::decode_outputs,
            )),
            Pending::AppThemes | Pending::AppThemeSet => {
                self.adopt_external_config();
                self.daemon.app_themes = Some(decoded!(
                    "theme.apps",
                    result,
                    crate::infrastructure::rpc_results::decode_app_themes,
                ));
                self.invalidate_settings();
                self.retick();
            }
            Pending::LockScreenSet { target, enabled, .. } => {
                let (actual_target, actual_enabled) = decoded!(
                    wall_proto::rpc::LOCKSCREEN_SET,
                    result,
                    crate::infrastructure::rpc_results::decode_lockscreen_set,
                );
                if actual_target != target || actual_enabled != enabled {
                    self.rpc_error(request_for_error, LOCKSCREEN_MISMATCH.into());
                    return;
                }
                self.adopt_external_config();
                self.show_toast(crate::i18n::tr(if enabled {
                    "settings-lockscreens-enabled"
                } else {
                    "settings-lockscreens-disabled"
                }));
                self.invalidate_settings();
                self.retick();
            }
            Pending::ThemeBackends => self.on_theme_backends(decoded!(
                "theme.backends",
                result,
                crate::infrastructure::rpc_results::decode_theme_backends,
            )),
            Pending::AudioPause => {
                self.call_tracked("wall.outputs", serde_json::json!({}), Pending::AudioOutputs);
                if self.panels.effects.is_some() {
                    self.call_tracked("wall.outputs", serde_json::json!({}), Pending::Outputs);
                }
            }
            Pending::AudioOutputs => self.on_audio_outputs(decoded!(
                "audio.outputs",
                result,
                crate::infrastructure::rpc_results::decode_audio_outputs,
            )),
            Pending::EffectsCommit { apply } => self.on_effects_commit(
                decoded!(
                    "effects.commit",
                    result,
                    crate::infrastructure::rpc_results::decode_effect_operation,
                ),
                apply,
            ),
            Pending::BrowserSearch { source, append, generation } => {
                self.on_browser_search(
                    decoded!(
                        "browser.search",
                        result,
                        crate::infrastructure::rpc_results::decode_browser_search,
                    ),
                    source,
                    append,
                    generation,
                );
            }
            Pending::BrowserCollections { source } => self.on_browser_collections(
                decoded!(
                    "browser.collections",
                    result,
                    crate::infrastructure::rpc_results::decode_browser_collections,
                ),
                source,
            ),
            Pending::BrowserPreview { .. } => {}
            Pending::BrowserDownload { source, id } => self.on_browser_download(
                decoded!(
                    "browser.download",
                    result,
                    crate::infrastructure::rpc_results::decode_browser_download,
                ),
                source,
                &id,
            ),
            Pending::Diag => self.on_diagnostic(decoded!(
                "diag",
                result,
                crate::infrastructure::rpc_results::decode_diagnostic,
            )),
            Pending::BugReport => self.on_bug_report(decoded!(
                "status.bug_report",
                result,
                crate::infrastructure::rpc_results::decode_bug_report,
            )),
            Pending::PlList => self.on_pl_list(decoded!(
                wall_proto::rpc::PLAYLIST_LIST,
                result,
                crate::infrastructure::rpc_results::decode_playlist_list,
            )),
            Pending::PlMembers => self.on_pl_members(decoded!(
                "playlist.members",
                result,
                crate::infrastructure::rpc_results::decode_playlist_members,
            )),
            Pending::PlOutputs => self.on_pl_outputs(decoded!(
                "wall.outputs",
                result,
                crate::infrastructure::rpc_results::decode_playlist_outputs,
            )),
            Pending::CardPickerList => self.on_card_picker_list(decoded!(
                "playlist.list",
                result,
                crate::infrastructure::rpc_results::decode_playlist_list,
            )),
            Pending::CardPickerMembers => self.on_card_picker_members(decoded!(
                "playlist.memberships",
                result,
                crate::infrastructure::rpc_results::decode_card_picker_memberships,
            )),
            Pending::CardPickerCreate => self.on_card_picker_create(decoded!(
                "playlist.create",
                result,
                crate::infrastructure::rpc_results::decode_card_picker_create,
            )),
            Pending::TaskList => self.on_task_list(decoded!(
                "task.list",
                result,
                crate::infrastructure::rpc_results::decode_task_list,
            )),
            Pending::Status => self.on_status(decoded!(
                "status",
                result,
                crate::infrastructure::rpc_results::decode_status,
            )),
            Pending::EffectThemes => self.on_effect_themes(decoded!(
                "effects.list",
                result,
                crate::infrastructure::rpc_results::decode_effects_list,
            )),
            Pending::Weather => self.on_weather(decoded!(
                wall_proto::rpc::WALL_WEATHER,
                result,
                crate::infrastructure::rpc_results::decode_weather,
            )),
            Pending::WeatherCandidates { category, query, generation } => self
                .on_weather_candidates(
                    &category,
                    &query,
                    generation,
                    decoded!(
                        wall_proto::rpc::WALL_WEATHER_CANDIDATES,
                        result,
                        crate::infrastructure::rpc_results::decode_weather_candidates,
                    ),
                ),
            Pending::WeatherAction => {
                self.adopt_external_config();
                self.on_weather(decoded!(
                    wall_proto::rpc::WALL_WEATHER,
                    result,
                    crate::infrastructure::rpc_results::decode_weather,
                ));
            }
            Pending::WeatherPlaylists => self.on_weather_playlists(decoded!(
                "playlist.list",
                result,
                crate::infrastructure::rpc_results::decode_playlist_list,
            )),
            Pending::ResetThumbnail { .. } => {
                let scheduled = decoded!(
                    "wall.reset_thumbnail",
                    result,
                    crate::infrastructure::rpc_results::decode_thumbnail_reset,
                );
                let message = if scheduled {
                    "card-back-reset-thumbnail-done"
                } else {
                    "card-back-reset-thumbnail-deferred"
                };
                self.show_toast(crate::i18n::tr(message));
                self.retick();
            }
            Pending::SceneProperties { revision, writes, .. } => self.on_scene_properties(
                decoded!(
                    "wall.we_properties",
                    result,
                    crate::infrastructure::rpc_results::decode_scene_properties,
                ),
                revision,
                &writes,
            ),
        }
    }
}
