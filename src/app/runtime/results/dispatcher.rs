use log::warn;
use serde_json::Value;

#[allow(clippy::wildcard_imports)]
use crate::app::*;

impl App {
    pub(in crate::app) fn rpc_error(&mut self, kind: Pending, err: String) {
        match kind {
            Pending::ResetThumbnail { .. } => {
                self.show_toast(
                    crate::i18n::tr_args!("card-back-reset-thumbnail-error", error => err),
                );
                self.retick();
            }
            Pending::AudioPause => {
                self.show_toast(crate::i18n::tr_args!("audio-playback-error", error => err));
                self.retick();
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
            Pending::SceneProperties { we_id } => {
                if self.panels.scene_properties.as_ref().is_some_and(|panel| panel.we_id == we_id) {
                    self.on_scene_properties_error(&err);
                }
            }
            _ => {}
        }
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
                                    .array_values(skwd_config::keys::theme::WALLPAPER_PROFILES)
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
                                .array_values(skwd_config::keys::theme::WALLPAPER_PROFILES)
                                .iter()
                                .any(|profile| {
                                    profile["key"].as_str() == Some(&current.key)
                                        && profile["enabled"].as_bool() == Some(true)
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
            Pending::DemoOutputs => self.on_demo_outputs(decoded!(
                "wall.outputs",
                result,
                crate::infrastructure::rpc_results::decode_outputs,
            )),
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
                "playlist.list",
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
                "wall.weather",
                result,
                crate::infrastructure::rpc_results::decode_weather,
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
            Pending::SceneProperties { .. } => self.on_scene_properties(decoded!(
                "wall.we_properties",
                result,
                crate::infrastructure::rpc_results::decode_scene_properties,
            )),
        }
    }
}
