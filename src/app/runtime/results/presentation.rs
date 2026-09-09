#[allow(clippy::wildcard_imports)]
use crate::app::*;
use crate::contracts::media::MediaKind;

impl App {
    pub(super) fn on_theme_previews(
        &mut self,
        result: crate::contracts::daemon::ThemePreviewsResult,
        requested_backend: &str,
    ) {
        if self.theme.audition_pending_backend.as_deref() == Some(requested_backend) {
            self.theme.audition_pending_backend = None;
        }
        if self.theme.audition_backend != requested_backend {
            return;
        }
        let Some(response_backend) = result.backend else { return };
        self.theme.audition_backend = response_backend;
        self.theme.audition_loading = false;
        self.theme.audition_backends = result.backends;
        self.theme.audition_previews = result
            .previews
            .into_iter()
            .map(|preview| crate::app::state::ThemeAuditionPreview {
                backend: preview.backend,
                key: preview.key,
                value: preview.value,
                label: preview.label,
                palette: crate::frontend::theme::Palette::from_spec(&preview.palette),
            })
            .collect();
        self.theme.audition_error = self
            .theme
            .audition_previews
            .is_empty()
            .then(|| crate::i18n::tr("theme-audition-empty").to_string());
        self.retick();
    }

    pub(super) fn on_theme_preview(
        &mut self,
        result: crate::contracts::daemon::ThemePreviewResult,
        card: usize,
        backend: &str,
    ) {
        let crate::contracts::daemon::ThemePreviewResult { colors, palette } = result;
        let cols: Vec<iced::Color> = colors
            .into_iter()
            .filter_map(|color| crate::frontend::theme::parse_hex(&color))
            .collect();
        if self.theme.job_pending == Some(card) {
            self.theme.job_pending = None;
        }
        if cols.is_empty() || self.config.theme_backend() != backend {
            return;
        }
        if self.theme.swatch_cache.len() > PALETTE_CACHE_CAP {
            self.theme.swatch_cache.clear();
        }
        self.theme.swatch_cache.insert(card, cols.clone());
        let palette = palette.as_ref().map(crate::frontend::theme::Palette::from_spec);
        if let Some(pal) = palette {
            if self.theme.cache.len() > PALETTE_CACHE_CAP {
                self.theme.cache.clear();
            }
            self.theme.cache.insert(card, pal);
        }
        if self.theme.swatch_target == Some(card) {
            self.theme.swatch = cols;
            self.retick();
        }
        if self.theme.preview_target == Some(card)
            && let Some(pal) = palette
        {
            self.set_swatch_for(card);
            self.start_fade(pal);
            self.retick();
        }
    }

    pub(crate) fn on_outputs(&mut self, result: crate::contracts::daemon::OutputsResult) {
        let outs = result.outputs;
        self.daemon.output_statuses.clone_from(&outs);
        let mut output_names: Vec<String> = outs
            .iter()
            .filter(|output| output.is_connected())
            .map(|output| output.name.clone())
            .collect();
        output_names.sort();
        output_names.dedup();
        if self.daemon.output_names != output_names {
            self.daemon.output_names = output_names;
            if self.panels.settings.open {
                self.init_settings_inputs();
                self.invalidate_settings();
            }
        }
        if self.panels.settings.open {
            self.invalidate_settings();
            self.retick();
        }
        let active =
            outs.iter().filter(|out| out.is_connected()).any(|out| out.kind.has_audio_controls());
        let playing = outs
            .iter()
            .filter(|out| out.is_connected())
            .any(|out| out.kind.has_audio_controls() && !out.mute && out.volume > 0);
        if active != self.panels.audio_active || playing != self.panels.audio_playing {
            self.panels.audio_active = active;
            self.panels.audio_playing = playing;
            self.chrome.bar.cache.clear();
        }
        let thumb_for = |out: &crate::contracts::daemon::OutputStatus| -> Option<String> {
            let items = &self.library_session.library.catalog().items;
            let assigned = if out.current.is_empty() { &out.path } else { &out.current };
            let found = match &out.kind {
                MediaKind::WallpaperEngine if !out.we_id.is_empty() => {
                    items.iter().find(|it| it.we_id == out.we_id)
                }
                MediaKind::Video if !assigned.is_empty() => {
                    items.iter().find(|it| it.video_file == *assigned)
                }
                _ if !assigned.is_empty() => items.iter().find(|it| it.path == *assigned),
                _ => None,
            };
            found.and_then(|it| {
                (!it.thumb.is_empty())
                    .then(|| it.thumb.clone())
                    .or_else(|| (!it.preview.is_empty()).then(|| it.preview.clone()))
            })
        };
        self.daemon.output_wallpaper_art = outs
            .iter()
            .filter_map(|out| {
                let art = thumb_for(out).or_else(|| {
                    (out.kind == MediaKind::Static)
                        .then_some(if out.path.is_empty() { &out.current } else { &out.path })
                        .filter(|path| !path.is_empty())
                        .cloned()
                })?;
                Some((out.name.clone(), art))
            })
            .collect();
        let configured_monitor = self.config.main_monitor();
        let banner_output = outs
            .iter()
            .find(|out| {
                out.is_connected()
                    && !configured_monitor.is_empty()
                    && out.name == configured_monitor
            })
            .or_else(|| outs.iter().find(|out| out.is_connected()));
        self.daemon.current_wallpaper_art = banner_output.and_then(|out| {
            if out.kind == MediaKind::Static {
                return (!out.path.is_empty())
                    .then(|| out.path.clone())
                    .or_else(|| (!out.current.is_empty()).then(|| out.current.clone()));
            }
            thumb_for(out)
        });
        let mons: Vec<crate::frontend::effects::MonitorInfo> = outs
            .into_iter()
            .map(|out| {
                let (width, height) = out.logical_size();
                let target = out.target().to_string();
                let connected = out.is_connected();
                let lock_path =
                    format!("{}.{}", skwd_config::keys::display::OUTPUT_LOCKS, out.name);
                crate::frontend::effects::MonitorInfo {
                    current_thumb: thumb_for(&out),
                    name: out.name,
                    target,
                    connected,
                    width,
                    height,
                    kind: match &out.kind {
                        MediaKind::Video => crate::domain::library::catalog::WallpaperKind::Video,
                        MediaKind::WallpaperEngine => {
                            crate::domain::library::catalog::WallpaperKind::We
                        }
                        MediaKind::Static | MediaKind::Other(_) => {
                            crate::domain::library::catalog::WallpaperKind::Static
                        }
                    },
                    mute: out.mute,
                    volume: out.volume,
                    fill: out.fill,
                    locked: self.config.flag_default_config(&lock_path),
                    paused: out.paused,
                    manual_paused: out.manual_paused,
                    current: out.current,
                    we_id: out.we_id,
                }
            })
            .collect();
        if let Some(eff) = self.panels.effects.as_mut() {
            eff.set_monitors(mons);
        }
    }

    pub(super) fn on_demo_outputs(&mut self, result: crate::contracts::daemon::OutputsResult) {
        let outputs = result
            .outputs
            .into_iter()
            .filter(crate::contracts::daemon::OutputStatus::is_connected)
            .collect::<Vec<_>>();
        self.daemon.output_statuses.clone_from(&outputs);
        if let Some(session) = self.runtime_state.demo.as_mut()
            && !session.overrides_active
        {
            session.overridden_outputs = outputs;
        }
    }

    pub(super) fn on_theme_backends(
        &mut self,
        result: crate::contracts::daemon::ThemeBackendsResult,
    ) {
        if let Some(backends) = result.backends {
            self.theme.backends = Some(backends);
            self.chrome.bar.cache.clear();
            self.retick();
        }
    }

    pub(super) fn on_audio_outputs(
        &mut self,
        result: crate::contracts::daemon::AudioOutputsResult,
    ) {
        let mut mons: Vec<crate::frontend::audio_panel::AudioMon> = result
            .outputs
            .into_iter()
            .filter(crate::contracts::daemon::OutputStatus::is_connected)
            .map(|out| crate::frontend::audio_panel::AudioMon {
                label: crate::frontend::audio_panel::label_for(&out.kind, &out.path, &out.we_id),
                source: if out.kind == MediaKind::WallpaperEngine {
                    out.we_id.clone()
                } else {
                    out.path.clone()
                },
                name: out.name,
                wtype: out.kind,
                mute: out.mute,
                volume: out.volume,
                shared: out.audio_shared,
                paused: out.paused,
                manual_paused: out.manual_paused,
            })
            .collect();
        crate::frontend::audio_panel::align_shared_audio(&mut mons);
        if let Some(volume) =
            self.runtime_state.demo.as_ref().and_then(|session| session.audio_demo_volume)
        {
            for monitor in mons.iter_mut().filter(|monitor| monitor.has_audio_controls()) {
                monitor.volume = volume;
                monitor.mute = true;
            }
        }
        if let Some(panel) = self.panels.audio.as_mut() {
            panel.mons = mons;
        }
    }
}
