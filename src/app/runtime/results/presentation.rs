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
        let current_backend =
            self.library_session.library.catalog().items.get(card).map(|item| {
                crate::app::update::theme::wallpaper_theme_backend_for(self, &item.key)
            });
        if cols.is_empty() || current_backend.as_deref() != Some(backend) {
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
        let mut outs = result.outputs;
        self.panels.audio_volumes.retain(|target, _| {
            outs.iter().any(|out| {
                out.is_connected()
                    && out.target() == target
                    && self.daemon.output_statuses.iter().any(|old| {
                        old.target() == target
                            && old.kind == out.kind
                            && old.current == out.current
                            && old.path == out.path
                            && old.we_id == out.we_id
                    })
            })
        });
        let mut audio: Vec<_> = outs
            .iter()
            .filter(|out| out.is_connected())
            .map(crate::frontend::audio_panel::AudioMon::from)
            .collect();
        crate::frontend::audio_panel::align_shared_audio(&mut audio);
        for out in &mut outs {
            if out.is_connected()
                && let Some(mon) = audio.iter().find(|mon| mon.name == out.name)
            {
                out.volume = mon.volume;
                out.mute = mon.mute;
                out.audio_shared = mon.shared;
            }
            if let Some(volume) = self.panels.audio_volumes.get(out.target()) {
                out.volume = *volume;
                out.mute = *volume == 0;
            }
        }
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
        let items = &self.library_session.library.catalog().items;
        let thumb_for = |out| output_thumb(out, items);
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
        let live_selection = self.panels.effects.as_ref().is_some_and(|effects| {
            effects.mode() == crate::frontend::effects::EffectsMode::Studio
                || effects.activity.can_apply() && !effects.activity.supported
        });
        if live_selection {
            self.set_effect_monitors(&outs);
        }
        crate::app::update::settings_background::sync_monitors(self);
    }

    pub(super) fn on_activities(&mut self, result: crate::contracts::daemon::ActivitiesResult) {
        let Some(effects) =
            self.panels.effects.as_mut().filter(|effects| {
                effects.mode() == crate::frontend::effects::EffectsMode::Displays
            })
        else {
            return;
        };
        if !result.supported {
            effects.activity.supported = false;
            effects.activity.ready = true;
            effects.activity.error = None;
            self.set_effect_monitors(&self.daemon.output_statuses.clone());
            self.retick();
            return;
        }
        let valid = !result.current.is_empty()
            && result
                .activities
                .iter()
                .any(|activity| activity.id == result.current && activity.current);
        if !valid {
            effects.activity.error =
                Some(crate::i18n::tr("effects-activity-unavailable").to_string());
            effects.activity.ready = false;
            effects.clear_monitors();
            self.retick();
            return;
        }
        effects.activity.supported = true;
        effects.activity.current = result.current.clone();
        if effects.activity.selected.is_empty() {
            effects.activity.selected = result.current;
        }
        effects.activity.activities = result.activities;
        effects.activity.error = None;
        effects.activity.ready = false;
        if !effects
            .activity
            .activities
            .iter()
            .any(|activity| activity.id == effects.activity.selected)
        {
            effects.activity.error =
                Some(crate::i18n::tr("effects-activity-unavailable").to_string());
            effects.clear_monitors();
            self.retick();
            return;
        }
        effects.activity.generation += 1;
        let generation = effects.activity.generation;
        let selected = effects.activity.selected.clone();
        let id = self.call_tracked(
            wall_proto::rpc::WALL_OUTPUTS,
            serde_json::json!({(wall_proto::ACTIVITY_PARAM): selected.clone()}),
            Pending::ActivityOutputs { activity: selected, generation },
        );
        if let Some(effects) = self.panels.effects.as_mut() {
            effects.activity.output_request = id;
        }
        self.retick();
    }

    pub(super) fn on_activity_outputs(
        &mut self,
        result: &crate::contracts::daemon::OutputsResult,
        activity: &str,
        generation: u64,
    ) {
        let Some(effects) = self.panels.effects.as_mut() else { return };
        if effects.activity.selected != activity || effects.activity.generation != generation {
            return;
        }
        effects.activity.ready = true;
        effects.activity.error = None;
        self.set_effect_monitors(&result.outputs);
        self.retick();
    }

    fn set_effect_monitors(&mut self, outs: &[crate::contracts::daemon::OutputStatus]) {
        let items = &self.library_session.library.catalog().items;
        let theme_output = self.config.str_path(skwd_config::keys::display::THEME_OUTPUT);
        let mons: Vec<crate::frontend::effects::MonitorInfo> = outs
            .iter()
            .map(|out| {
                let (width, height) = out.logical_size();
                let target = out.target().to_string();
                let connected = out.is_connected();
                let lock_path =
                    format!("{}.{}", skwd_config::keys::display::OUTPUT_LOCKS, out.name);
                let theme_source = out.name == theme_output;
                crate::frontend::effects::MonitorInfo {
                    current_thumb: output_thumb(out, items),
                    name: out.name.clone(),
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
                    fill: out.fill.clone(),
                    background: crate::frontend::settings::background::BackgroundControl::default(),
                    background_reveal: 0.0,
                    background_picker: 0.0,
                    locked: self.config.flag_default_config(&lock_path),
                    theme_source,
                    paused: out.paused,
                    manual_paused: out.manual_paused,
                    current: out.current.clone(),
                    we_id: out.we_id.clone(),
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
        self.on_outputs(crate::contracts::daemon::OutputsResult { outputs: result.outputs });
        let mut mons: Vec<_> = self
            .daemon
            .output_statuses
            .iter()
            .filter(|out| out.is_connected())
            .map(crate::frontend::audio_panel::AudioMon::from)
            .collect();
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

fn output_thumb(
    out: &crate::contracts::daemon::OutputStatus,
    items: &[crate::domain::library::catalog::Wallpaper],
) -> Option<String> {
    let assigned = if out.current.is_empty() { &out.path } else { &out.current };
    let found = match &out.kind {
        MediaKind::WallpaperEngine if !out.we_id.is_empty() => {
            items.iter().find(|item| item.we_id == out.we_id)
        }
        MediaKind::Video if !assigned.is_empty() => {
            items.iter().find(|item| item.video_file == *assigned)
        }
        _ if !assigned.is_empty() => items.iter().find(|item| item.path == *assigned),
        _ => None,
    };
    found.and_then(|item| {
        (!item.thumb.is_empty())
            .then(|| item.thumb.clone())
            .or_else(|| (!item.preview.is_empty()).then(|| item.preview.clone()))
    })
}
