use log::{info, warn};
use serde::Deserialize;
use serde_json::{Value, json};
use wall_proto::ev;

use crate::contracts::browser::{ApplyTarget, DownloadStatus, DownloadUpdate};
use crate::contracts::daemon::{TaskState, TaskStatus};
use crate::domain::library::filter::{filter_sort, insert_index};
use crate::frontend::theme::Palette;
use crate::infrastructure::library::{LibraryPaths, decode_cached};
use crate::rendering::scene::atlas::AtlasMap;

#[allow(clippy::wildcard_imports)]
use super::super::*;
use super::apply_error_message;

impl App {
    pub(in crate::app) fn on_event(&mut self, name: &str, data: &Value) {
        match name {
            ev::PLAYBACK => {
                if let Ok(mut status) =
                    serde_json::from_value::<crate::contracts::daemon::PlaybackStatus>(data.clone())
                {
                    status.available_processes =
                        std::mem::take(&mut self.daemon.playback.available_processes);
                    self.daemon.playback = status;
                    if self.panels.audio.is_some() {
                        self.call_tracked("wall.outputs", json!({}), Pending::AudioOutputs);
                    }
                    if self.panels.effects.is_some() {
                        self.call_tracked("wall.outputs", json!({}), Pending::Outputs);
                    }
                    self.retick();
                }
            }
            ev::REMOTE_THUMB => self.on_remote_thumb(data),
            ev::PREVIEW_READY => {
                let Ok(payload) = ev::PreviewReady::deserialize(data) else { return };
                let adopted = self.source_browser.browser.as_mut().is_some_and(|br| {
                    let current_id = br
                        .session
                        .preview
                        .and_then(|index| br.session.items.get(index))
                        .map(|item| item.id.as_str());
                    if current_id != Some(payload.id.as_str()) {
                        return false;
                    }
                    let Some(item) = br.item_mut(&payload.id) else { return false };
                    item.preview_path = Some(payload.path.clone());
                    true
                });
                if !adopted {
                    let _ = std::fs::remove_file(&payload.path);
                }
            }
            ev::DOWNLOAD => self.on_download(data),
            ev::TASK_STATUS => self.on_task_status(data),
            ev::WATCH_STATUS => {
                let Ok(status) = crate::infrastructure::rpc_results::decode_library_watch(data)
                else {
                    return;
                };
                self.daemon.library_watch = Some(status);
                self.scene.touch();
                self.retick();
            }
            ev::CACHED => self.on_cached(data),
            ev::REMOVED => {
                let Ok(payload) = ev::Removed::deserialize(data) else { return };
                if self.library_session.library.remove_by_key(&payload.key) {
                    info!(
                        "removed {} incrementally ({} items left)",
                        payload.key,
                        self.library_session.library.catalog().items.len()
                    );
                    self.refilter_after_removal();
                    self.preview_resources.atlas = Some(AtlasMap::new(
                        self.library_session.library.catalog().items.len().max(1),
                    ));
                    self.scene.touch();
                }
            }
            ev::FILE_REMOVED => {
                let Ok(payload) = ev::FileRemoved::deserialize(data) else { return };
                self.library_session.library.remove_file(&payload.name);
                self.refilter_after_removal();
            }
            ev::UNSUBSCRIBED => {
                if ev::Unsubscribed::deserialize(data).unwrap_or_default().warn {
                    self.show_toast(crate::i18n::tr("status-unsubscribe-failed"));
                    self.scene.touch();
                    self.retick();
                }
            }
            ev::FILE_RENAMED => {
                let payload = ev::FileRenamed::deserialize(data).unwrap_or_default();
                if !payload.old_name.is_empty() && !payload.new_name.is_empty() {
                    let paths = LibraryPaths::new(
                        &self.library_session.wallpaper_dir,
                        &self.library_session.video_dir,
                    );
                    let new_path = paths.renamed_static_path(&payload.new_name);
                    self.library_session.library.rename_file(
                        &payload.old_name,
                        &payload.new_name,
                        &new_path,
                        crate::backend::picker::library::RenameMetadata {
                            mtime: payload.mtime,
                            filesize: payload.filesize as i64,
                            width: i64::from(payload.width),
                            height: i64::from(payload.height),
                        },
                    );
                }
            }
            ev::FOLDER_REMOVED => self.on_folder_removed(data),
            ev::APPLIED => self.on_applied(data),
            ev::THEME_DONE => self.on_theme_done(data),
            ev::APPLY_RESULT => self.on_apply_result(data),
            ev::HIDE | ev::TOGGLE => {
                info!("daemon requested hide");
                crate::app::warm::request_hide(self);
            }
            ev::SCAN_DONE => self.on_scan_done(data),
            ev::SEMANTIC_INDEX_READY => {
                info!("semantic index updated by daemon");
                self.runtime_state.semantic = None;
                if self.tags.search_mode == SearchMode::Describe
                    && !self.tags.semantic.search.trim().is_empty()
                {
                    self.request_semantic_search();
                }
            }
            ev::CONFIG_CHANGED => {
                self.adopt_external_config();
            }
            ev::OUTPUTS_CHANGED => {
                self.call_tracked("wall.outputs", json!({}), Pending::Outputs);
                if self.panels.audio.is_some() {
                    self.call_tracked("wall.outputs", json!({}), Pending::AudioOutputs);
                }
            }
            ev::POWER_CHANGED => {
                let Ok(payload) = ev::PowerChanged::deserialize(data) else { return };
                if self.config.set_on_battery_power(payload.on_battery) {
                    info!(
                        "picker power policy changed to {}",
                        if payload.on_battery { "battery" } else { "external power" }
                    );
                    self.scene.set_live_preview_fps(self.config.video_preview_fps());
                    self.chrome.bar.cache.clear();
                    self.scene.touch();
                    self.retick();
                }
            }
            _ => {}
        }
    }

    fn on_remote_thumb(&mut self, data: &Value) {
        let Ok(payload) = ev::RemoteThumb::deserialize(data) else {
            return;
        };
        let motion = self.source_browser.motion;
        let active =
            self.source_browser.browser.as_ref().is_some_and(|browser| {
                browser.session.items.iter().any(|item| item.id == payload.id)
            });
        let br = if active {
            self.source_browser.browser.as_mut()
        } else {
            self.source_browser
                .tabs
                .values_mut()
                .find(|browser| browser.session.items.iter().any(|item| item.id == payload.id))
        };
        let Some(br) = br else {
            return;
        };
        if payload.path.is_empty() {
            if let Some(it) = br.item_mut(&payload.id) {
                it.thumb_failed = true;
            }
            return;
        }
        let mut became_ready = false;
        if let Some(it) = br.item_mut(&payload.id) {
            it.thumb_path.clone_from(&payload.path);
            it.thumb_failed = false;
            if !it.thumb_ready {
                it.thumb_ready = true;
                became_ready = true;
            }
        }
        if !became_ready {
            return;
        }
        let source = br.source.key().to_string();
        let mut spring = motion.spring(0.0, crate::frontend::animation::MotionTier::Standard);
        spring.retarget(1.0);
        br.view.thumb_fades.insert(payload.id.clone(), spring);
        if active {
            self.source_browser.wall.update_thumb(&source, &payload.id, &payload.path);
        }
        self.retick();
    }

    fn on_download(&mut self, data: &Value) {
        let Some(update) = crate::infrastructure::browser::decode_download_event(data) else {
            return;
        };
        let state = match update.status {
            DownloadStatus::Done => TaskState::Completed,
            DownloadStatus::Failed => TaskState::Failed,
            _ => TaskState::Running,
        };
        let mut task = TaskStatus::running(
            format!("download:{}", update.id),
            "download",
            crate::i18n::tr("status-download-wallpaper"),
        );
        task.state = state;
        task.progress = update.progress.map_or(0, |value| (value * 100.0) as u64);
        task.total = u64::from(update.progress.is_some()) * 100;
        task.detail = update.message.clone().unwrap_or_else(|| update.id.clone());
        self.daemon.tasks.update(task);
        self.chrome.bar.cache.clear();
        let apply = self
            .source_browser
            .browser_for_download_mut(&update.id)
            .and_then(|br| download_update(br, &update));
        if let Some(target) = apply {
            let params = crate::infrastructure::browser::encode_apply(&target);
            self.daemon.client.call("wall.apply", params);
            if self.config.close_on_selection() {
                self.clear_browser_previews();
                crate::app::warm::request_hide(self);
            }
        }
    }

    fn on_task_status(&mut self, data: &Value) {
        match crate::infrastructure::rpc_results::decode_task_status(data) {
            Ok(task) => {
                self.daemon.tasks.update(task);
                self.chrome.bar.cache.clear();
                self.retick();
            }
            Err(error) => warn!("invalid task status event: {error}"),
        }
    }

    fn on_cached(&mut self, data: &Value) {
        let paths =
            LibraryPaths::new(&self.library_session.wallpaper_dir, &self.library_session.video_dir);
        let Some(wallpaper) = decode_cached(data, paths) else {
            return;
        };
        if !self.library_session.library.insert(wallpaper) {
            return;
        }
        let new_idx = (self.library_session.library.catalog().items.len() - 1) as u32;
        let visible =
            filter_sort(self.library_session.library.catalog(), &self.library_session.filters)
                .contains(&new_idx)
                && self.library_session.playlist_filter.as_ref().is_none_or(|(_, _, keys)| {
                    keys.contains(
                        &self.library_session.library.catalog().items[new_idx as usize].key,
                    )
                });
        if visible {
            let pos = insert_index(
                self.library_session.library.catalog(),
                &self.library_session.filters,
                &self.library_session.filtered,
                new_idx,
            );
            self.library_session.filtered.insert(pos, new_idx);
            self.scene.shift_after_insert(pos);
            self.scene.touch();
        }
    }

    fn on_folder_removed(&mut self, data: &Value) {
        let payload = ev::FolderRemoved::deserialize(data).unwrap_or_default();
        if !payload.names.is_empty() {
            self.library_session.library.remove_folder(&payload.names);
            self.refilter_after_removal();
        }
    }

    fn on_applied(&mut self, data: &Value) {
        let payload = ev::Applied::deserialize(data).unwrap_or_default();
        let active =
            payload.kind == wall_proto::kind::VIDEO || payload.kind == wall_proto::kind::WE;
        let playing = active
            && !payload.mute.unwrap_or_else(|| self.config.wallpaper_mute())
            && payload
                .volume
                .map_or_else(|| self.config.wallpaper_volume(), |volume| volume as u32)
                > 0;
        if active != self.panels.audio_active || playing != self.panels.audio_playing {
            self.panels.audio_active = active;
            self.panels.audio_playing = playing;
            self.chrome.bar.cache.clear();
        }
        let recorded = self.library_session.library.record_applied(
            &payload.key,
            payload.we_id.as_deref().unwrap_or_default(),
            payload.name.as_deref().unwrap_or_default(),
        );
        if recorded.is_some() && self.library_session.filters.sort == "recent" {
            self.refilter();
        }
        self.call_tracked("wall.outputs", json!({}), Pending::Outputs);
        self.theme.base_palette = Palette::from_spec(&crate::infrastructure::theme::load_palette(
            &self.config.cache_dir(),
        ));
        self.theme.preview_target = None;
        self.theme.hover_since = None;
        self.scene.touch();
    }

    fn on_theme_done(&mut self, data: &Value) {
        let payload = ev::ThemeDone::deserialize(data).unwrap_or_default();
        if !payload.ok {
            warn!("theme update failed for {:?}", payload.source);
            self.show_toast(crate::i18n::tr("status-theme-failed"));
            self.scene.touch();
            self.retick();
            return;
        }
        let requested = payload.requested.as_str();
        let effective = payload.backend.as_str();
        if !requested.is_empty() && !effective.is_empty() && requested != effective {
            warn!("theme backend {requested} unavailable, themed with {effective}");
            self.show_toast(crate::i18n::tr_args!(
                "status-theme-backend-missing",
                requested => requested,
                effective => effective,
            ));
        }
        self.theme.base_palette = Palette::from_spec(&crate::infrastructure::theme::load_palette(
            &self.config.cache_dir(),
        ));
        info!(
            "theme_done: base primary=({:.2},{:.2},{:.2}) preview_target={:?}",
            self.theme.base_palette.primary.r,
            self.theme.base_palette.primary.g,
            self.theme.base_palette.primary.b,
            self.theme.preview_target
        );
        if self.theme.preview_target.is_none() {
            self.start_fade(self.theme.base_palette);
        }
        if self.theme.bar_open {
            self.set_base_swatch();
            self.chrome.bar.cache.clear();
        }
        self.scene.touch();
        self.retick();
    }

    fn on_apply_result(&mut self, data: &Value) {
        let Ok(payload) = ev::ApplyResult::deserialize(data) else {
            return;
        };
        if !payload.ok {
            let kind = payload.error_kind.as_deref().unwrap_or("apply_failed");
            let detail = payload.detail.as_deref().unwrap_or("");
            self.scene.reveal_hero();
            self.show_toast(apply_error_message(kind, detail));
            self.scene.touch();
        }
    }

    fn on_scan_done(&mut self, data: &Value) {
        let payload = ev::ScanDone::deserialize(data).unwrap_or_default();
        if payload.disk_full {
            self.show_toast(crate::i18n::tr("status-disk-full"));
        }
        let count = payload.count as usize;
        let total = payload.total.filter(|val| *val >= 0).map(|val| val as usize);
        let have = self.library_session.library.catalog().items.len();
        if !needs_list_refresh(have, count, total) {
            info!("scan_done: {count} items, library already in sync");
            return;
        }
        info!("scan_done: scanner has {count} items, catalog has {have}; refreshing list");
        self.call_tracked("wall.list", json!({"favourites": false}), Pending::List);
    }
}

pub(crate) fn download_update(
    br: &mut crate::frontend::browser::Browser,
    update: &DownloadUpdate,
) -> Option<ApplyTarget> {
    let is_steam = br.is_steam();
    let kind = br.source.apply_kind();
    if let Some(item) = br.item_mut(&update.id) {
        item.apply_download_update(update);
    }
    if matches!(
        update.status,
        DownloadStatus::Downloading | DownloadStatus::Done | DownloadStatus::Queued
    ) {
        br.session.error = None;
    }
    if update.status == DownloadStatus::Failed {
        if br.session.pending_apply.as_deref() == Some(update.id.as_str()) {
            br.session.pending_apply = None;
        }
        if let Some(message) = update.message.as_deref() {
            br.session.error = Some(message.to_string());
        }
    }
    if update.status == DownloadStatus::Done
        && br.session.pending_apply.as_deref() == Some(update.id.as_str())
    {
        br.session.pending_apply = None;
        return if is_steam {
            Some(ApplyTarget::WallpaperEngine { id: update.id.clone() })
        } else {
            update.path.as_ref().map(|path| ApplyTarget::Path { kind, path: path.clone() })
        };
    }
    None
}
