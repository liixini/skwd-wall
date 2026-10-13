use std::time::{Duration, Instant};

use serde_json::json;

use crate::app::scene::{FrameDemand, RebuildCtx};
use crate::domain::library::catalog::WallpaperKind;
use crate::frontend::scene::layout::Mode;

#[allow(clippy::wildcard_imports)]
use super::super::*;

const PREHEAT_DEBOUNCE: Duration = Duration::from_millis(120);

impl App {
    pub(in crate::app) fn schedule_frame(&mut self) {
        if self.runtime_state.stashed
            || self.scene.viewport.0 <= 0.0
            || self.scene.viewport.1 <= 0.0
        {
            self.clear_animation_phase();
            self.runtime_state.frame_clock.schedule(None);
            return;
        }
        let now = Instant::now();
        let deadline = if self.animating() {
            let interval = self.tick_interval();
            let deadline = crate::app::scene::rephase_animation_deadline(
                self.runtime_state.next_animation_tick,
                self.runtime_state.animation_interval,
                interval,
                self.runtime_state.last_tick,
                now,
            );
            self.runtime_state.next_animation_tick = Some(deadline);
            self.runtime_state.animation_interval = Some(interval);
            Some(deadline)
        } else {
            self.clear_animation_phase();
            let toast = self.runtime_state.toast.as_ref().map(|(_, since)| {
                let fade_ms =
                    self.motion_profile().duration_ms(crate::frontend::animation::MotionTier::Slow)
                        as u128;
                let wait_ms = TOAST_MS.saturating_sub(fade_ms) as u64;
                (*since + Duration::from_millis(wait_ms)).max(now)
            });
            [self.scene.preview_deadline(), toast].into_iter().flatten().min()
        };
        self.runtime_state.frame_clock.schedule(deadline);
    }

    fn clear_animation_phase(&mut self) {
        self.runtime_state.next_animation_tick = None;
        self.runtime_state.animation_interval = None;
        self.preview_resources.render_loop_active = false;
    }

    pub(in crate::app) fn retick(&mut self) {
        if self.runtime_state.stashed
            || self.scene.viewport.0 <= 0.0
            || self.scene.viewport.1 <= 0.0
        {
            return;
        }
        let now = Instant::now();
        if !crate::app::scene::retick_due(
            self.preview_resources.render_loop_active,
            self.runtime_state.last_tick,
            now,
            self.tick_interval(),
        ) {
            return;
        }
        self.run_tick(now);
    }

    pub(in crate::app) fn detail_open(&self) -> bool {
        !matches!(self.scene.mode, Mode::Slices | Mode::Depth) && self.scene.flip_open()
    }

    pub(in crate::app) fn wants_transition_preview(&self) -> bool {
        let transition_section = crate::frontend::settings::is_transition_preview_section(
            &self.panels.settings.tab,
            self.panels.settings.section,
        );
        self.panels.settings.open
            && self.panels.settings.tab == "motion"
            && transition_section
            && self.config.flag_default_true(skwd_config::keys::transition::ENABLED)
            && self.config.flag_default_true(skwd_config::keys::transition::PREVIEW)
    }

    pub(in crate::app) fn animating(&self) -> bool {
        self.frame_demand() != FrameDemand::Idle || self.wants_transition_preview()
    }

    pub(in crate::app) fn tick_interval(&self) -> std::time::Duration {
        let base = self.config.frame_interval();
        let demand = self.frame_demand();
        let demand_interval = match demand {
            FrameDemand::Idle | FrameDemand::Motion | FrameDemand::Direct => base,
            FrameDemand::Preview => base.max(Duration::from_secs_f32(1.0 / 30.0)),
            FrameDemand::Passive => base.max(Duration::from_secs_f32(1.0 / 20.0)),
        };
        if self.wants_transition_preview() {
            let preview_interval = self.config.transition_preview_interval().max(base);
            if demand == FrameDemand::Idle {
                return preview_interval;
            }
            return demand_interval.min(preview_interval);
        }
        demand_interval
    }

    fn frame_demand(&self) -> FrameDemand {
        let mut demand = self.scene.frame_demand();
        if self
            .runtime_state
            .demo
            .as_ref()
            .is_some_and(|session| session.scroll_rate.abs() > f32::EPSILON)
        {
            demand = demand.max(FrameDemand::Direct);
        }
        if self.theme.hover_since.is_some()
            || self.theme.job_pending.is_some()
            || self
                .source_browser
                .browser
                .as_ref()
                .is_some_and(crate::frontend::browser::Browser::busy)
        {
            demand = demand.max(FrameDemand::Passive);
        }
        if !self.theme.fade_t.settled()
            || self.runtime_state.toast.as_ref().is_some_and(|(_, since)| {
                let fade_ms =
                    self.motion_profile().duration_ms(crate::frontend::animation::MotionTier::Slow)
                        as u128;
                since.elapsed().as_millis() + fade_ms >= TOAST_MS
            })
            || self.source_browser.browser.as_ref().is_some_and(|br| {
                (br.session.preview.is_some() && self.source_browser.preview_animation < 1.0)
                    || self.source_browser.preview_closing
                    || br.view.thumb_fades.values().any(|spring| !spring.settled())
                    || br.view.hover_fades.values().any(|spring| !spring.settled())
            })
        {
            demand = demand.max(FrameDemand::Preview);
        }
        if (self.tags.cloud_open && !self.tags.cloud_entrance.settled())
            || (self.panels.settings.open && !self.panels.settings.entrance.settled())
            || (self.panels.settings.open && !self.panels.settings.tab_anim.settled())
            || (self.panels.settings.open && !self.panels.settings.section_anim.settled())
            || (self.panels.settings.open && !self.panels.settings.control_anim.settled())
            || (self.panels.settings.open && self.panels.settings.profile_animating())
            || ((self.panels.settings.open || self.panels.effects.is_some())
                && self.panels.settings.bars_animating())
            || self
                .panels
                .schedule
                .as_ref()
                .is_some_and(crate::frontend::schedule_editor::ScheduleEditor::animating)
            || self
                .panels
                .effects
                .as_ref()
                .is_some_and(crate::frontend::effects::Effects::animating)
            || self
                .panels
                .audio
                .as_ref()
                .is_some_and(crate::frontend::audio_panel::AudioPanel::animating)
            || self
                .panels
                .theme_designer
                .as_ref()
                .is_some_and(crate::frontend::theme_designer::ThemeDesigner::animating)
            || (self.source_browser.browser.is_some() && !self.source_browser.entrance.settled())
            || self.chrome.filter_bar_animating()
        {
            demand = demand.max(FrameDemand::Motion);
        }
        if !self.tags.cloud_scroll.settled()
            || !self.chrome.bar.menu_scroll.settled()
            || self.chrome.pane_scrolls.values().any(|pane| !pane.spring.settled() || pane.dirty)
            || (self.source_browser.browser.is_some()
                && !self.source_browser.wall.filtered.is_empty()
                && self.source_browser.wall.scene.is_animating())
            || (demand != FrameDemand::Idle
                && self.input.last_activity.elapsed() < Duration::from_millis(120))
        {
            demand = FrameDemand::Direct;
        }
        demand
    }

    pub(in crate::app) fn tick_browser_anim(&mut self, now: Instant) {
        let dt = self
            .runtime_state
            .last_animation_tick
            .map_or(1.0 / 60.0, |prev| now.duration_since(prev).as_secs_f32().min(0.05));
        self.runtime_state.last_animation_tick = Some(now);
        if self.source_browser.browser.is_none() {
            return;
        }
        let (
            count,
            loading,
            has_unready,
            preview,
            preview_loading,
            page,
            last_page,
            busy,
            page_failed,
        ) = {
            let br = self.source_browser.browser.as_ref().unwrap();
            (
                br.session.items.len(),
                br.session.loading,
                br.thumbs_pending(),
                br.session.preview.is_some(),
                br.preview_loading(),
                br.session.page,
                br.session.last_page,
                br.transfer_busy(),
                br.session.page_failed,
            )
        };

        self.source_browser.entrance.tick(dt);
        if loading || preview_loading || busy {
            self.source_browser.spinner_phase =
                (self.source_browser.spinner_phase + dt * 6.0) % std::f32::consts::TAU;
        }
        if has_unready || loading {
            self.source_browser.shimmer_phase =
                (self.source_browser.shimmer_phase + dt * 0.8) % 1.0;
        }
        self.tick_preview_anim(dt, preview);

        let hover_fading = {
            let br = self.source_browser.browser.as_mut().unwrap();
            br.view.thumb_fades.retain(|_, spring| {
                spring.tick(dt);
                !spring.settled()
            });
            br.view.hover_fades.retain(|_, spring| {
                spring.tick(dt);
                !(spring.settled() && spring.target <= 0.0)
            });
            br.view.hover_fades.values().any(|spring| !spring.settled())
        };
        if hover_fading {
            self.source_browser.wall.scene.touch();
        }

        let visible_end = self.source_browser.wall.scene.visible_range().1;
        if !loading && !page_failed && page < last_page && count > 0 && visible_end + 8 >= count {
            self.run_browser_search(true);
        }
    }

    fn tick_preview_anim(&mut self, dt: f32, preview: bool) {
        let rate = 1_000.0
            / self.source_browser.motion.duration_ms(crate::frontend::animation::MotionTier::Fast);
        if self.source_browser.preview_closing {
            self.source_browser.preview_animation =
                (self.source_browser.preview_animation - dt * rate).max(0.0);
            if self.source_browser.preview_animation <= 0.0 {
                self.source_browser.preview_closing = false;
                if let Some(br) = self.source_browser.browser.as_mut() {
                    br.session.preview = None;
                }
            }
        } else if preview && self.source_browser.preview_animation < 1.0 {
            self.source_browser.preview_animation =
                (self.source_browser.preview_animation + dt * rate).min(1.0);
        }
    }

    pub(in crate::app) fn preheat_focused(&mut self, now: Instant) {
        if !self.config.video_preview_enabled() || self.config.battery_saver_active() {
            let _ = self.library_session.preheat.poll(None, now, PREHEAT_DEBOUNCE);
            return;
        }
        let fidx = self.scene.hover.unwrap_or(self.scene.current);
        let target = self
            .library_session
            .filtered
            .get(fidx)
            .and_then(|&sidx| self.library_session.library.catalog().items.get(sidx as usize))
            .filter(|item| item.kind == WallpaperKind::Video && !item.path.is_empty())
            .map(|item| item.path.as_str());
        if let Some(path) = self.library_session.preheat.poll(target, now, PREHEAT_DEBOUNCE) {
            self.daemon.client.call("wall.preheat", json!({ "path": path }));
        }
    }

    fn request_tall_thumb(&mut self) {
        if self.scene.mode != Mode::Hand || !self.scene.hand_reveal_open() {
            return;
        }
        let Some(&store) = self.library_session.filtered.get(self.scene.current) else {
            return;
        };
        let Some(item) = self.library_session.library.catalog().items.get(store as usize) else {
            return;
        };
        if !item.is_tall() || !self.scene.tall_needs(store as usize) {
            return;
        }
        let video = item.effective_kind() == WallpaperKind::Video;
        let source = if video && !item.video_file.is_empty() {
            item.video_file.clone()
        } else {
            item.path.clone()
        };
        crate::infrastructure::preview::ensure_tall(
            store as usize,
            source,
            item.thumb.clone(),
            video,
            self.preview_resources.decoder.clone(),
        );
    }

    pub(in crate::app) fn run_tick(&mut self, now: Instant) {
        crate::app::update::drive_transition_preview(self);
        self.scene.input_idle =
            now.duration_since(self.input.last_activity) > std::time::Duration::from_secs(60);
        crate::zone!("run_tick");
        let dt = self
            .runtime_state
            .last_tick
            .map_or(1.0 / 60.0, |prev| now.duration_since(prev).as_secs_f32().min(0.05));
        self.runtime_state.last_tick = Some(now);
        self.tick_demo_scroll(dt);
        let apply_done = if let Some(eff) = self.panels.effects.as_mut() {
            eff.tick(dt);
            eff.apply_finished()
        } else {
            false
        };
        if apply_done && let Some(eff) = self.panels.effects.take() {
            for path in eff.discardable_previews() {
                self.discard_effect_preview(&path);
            }
            crate::app::warm::exit_picker(self);
        }
        if let Some(panel) = self.panels.audio.as_mut() {
            panel.tick(dt);
        }
        if let Some(designer) = self.panels.theme_designer.as_mut() {
            designer.tick(dt);
        }
        let t0 = Instant::now();
        {
            crate::zone!("preheat");
            self.preheat_focused(now);
        }
        self.request_tall_thumb();
        {
            crate::zone!("browser_anim");
            self.tick_browser_anim(now);
        }
        {
            crate::zone!("tick_overlays");
            self.tick_overlays(dt);
        }
        {
            crate::zone!("theme_preview");
            self.update_theme_preview(now, dt);
        }
        if let Some(item) = self
            .scene
            .flipped()
            .and_then(|index| self.library_session.filtered.get(index))
            .and_then(|&index| self.library_session.library.catalog().items.get(index as usize))
        {
            use crate::frontend::scene::RemovalAction;
            let pending = self.daemon.pending.values().find_map(|request| {
                if let Pending::RemoveWallpaper { key, unsubscribe } = request
                    && key == &item.key
                {
                    Some(*unsubscribe)
                } else {
                    None
                }
            });
            let action = match pending {
                Some(true) => RemovalAction::Unsubscribing,
                Some(false) => RemovalAction::Deleting,
                None if item.steam_subscribed && self.unsubscribe_available() => {
                    RemovalAction::Unsubscribe
                }
                None if item.steam_subscribed => RemovalAction::UnsubscribeUnavailable,
                None if item.steam_confirmation_required => RemovalAction::SteamStatusUnknown,
                None => RemovalAction::Delete,
            };
            self.scene.set_removal_action(action);
        }
        {
            crate::zone!("scene_tick");
            self.scene.tick(
                now,
                RebuildCtx {
                    catalog: self.library_session.library.catalog(),
                    filtered: &self.library_session.filtered,
                    atlas: &mut self.preview_resources.atlas,
                    pool: &self.preview_resources.decoder,
                    palette: &self.theme.palette,
                    uploads: &self.preview_resources.uploads,
                    hover_fades: None,
                },
            );
        }
        if self.source_browser.browser.is_some() {
            crate::zone!("browser_wall_tick");
            let hover_fades = self.source_browser.browser.as_ref().map(|br| &br.view.hover_fades);
            let gp = self.source_browser.wall.layout_grid();
            let (_, _, width, height) = crate::frontend::browser::wall_aperture_dims(
                self.scene.viewport,
                gp,
                self.config.ui_scale(),
            );
            self.source_browser.wall.set_viewport(width, height);
            self.source_browser.wall.scene.tick(
                now,
                RebuildCtx {
                    catalog: &self.source_browser.wall.catalog,
                    filtered: &self.source_browser.wall.filtered,
                    atlas: &mut self.source_browser.wall.atlas,
                    pool: &self.source_browser.wall.decoder,
                    palette: &self.theme.palette,
                    uploads: &self.source_browser.wall.uploads,
                    hover_fades,
                },
            );
            self.source_browser.wall.chrome_cache.clear();
        }
        {
            crate::zone!("flush_filter");
            self.flush_pending_filter();
        }
        if !self.scene.open_fade_settled() {
            self.chrome.bar.cache.clear();
            self.chrome.cache.clear();
        }
        {
            crate::zone!("metrics_frame");
            self.runtime_state.metrics.on_frame(
                now,
                t0.elapsed(),
                self.scene.render.instances.len(),
            );
        }
        let sig = {
            crate::zone!("chrome_sig");
            crate::frontend::ui::chrome_signature(
                &self.scene.render.chrome,
                self.scene.render.back.as_ref(),
                &self.theme.palette,
            )
        };
        if sig != self.chrome.signature {
            self.chrome.signature = sig;
            self.chrome.cache.clear();
        }
        if let Some((_, since)) = &self.runtime_state.toast
            && now.duration_since(*since).as_millis() >= TOAST_MS
        {
            self.runtime_state.toast = None;
        }
        let animating = self.animating();
        let interval = self.tick_interval();
        crate::app::scene::record_animation_tick(
            animating,
            &mut self.runtime_state.next_animation_tick,
            &mut self.runtime_state.animation_interval,
            now,
            interval,
        );
        self.preview_resources.render_loop_active = animating;
        crate::frame_mark!();
    }

    pub(in crate::app) fn tick_demo_scroll(&mut self, dt: f32) {
        let rate = self.runtime_state.demo.as_ref().map_or(0.0, |session| session.scroll_rate);
        if rate.abs() <= f32::EPSILON || !matches!(self.scene.mode, Mode::Slices | Mode::Sandy) {
            return;
        }
        self.scene.slice_scroll(rate * dt, self.library_session.filtered.len());
    }

    fn tick_overlays(&mut self, dt: f32) {
        if self.chrome.tick_filter_bar(dt) {
            self.chrome.bar.cache.clear();
        }
        if self.panels.settings.open {
            self.panels.settings.entrance.tick(dt);
        }
        if let Some(schedule) = self.panels.schedule.as_mut() {
            schedule.tick(dt);
        }
        if self.tags.cloud_open {
            self.tags.cloud_entrance.tick(dt);
        }
        if !self.tags.cloud_scroll.settled() {
            self.tags.cloud_scroll.tick(dt);
        }
        if !self.chrome.bar.menu_scroll.settled() {
            self.chrome.bar.menu_scroll.tick(dt);
            self.chrome.bar.cache.clear();
        }
        for pane in self.chrome.pane_scrolls.values_mut() {
            if !pane.spring.settled() {
                pane.spring.tick(dt);
                pane.dirty = true;
            }
        }
        if self.panels.settings.open && !self.panels.settings.tab_anim.settled() {
            self.panels.settings.tab_anim.tick(dt);
        }
        if self.panels.settings.open {
            self.panels.settings.section_anim.tick(dt);
            self.panels.settings.control_anim.tick(dt);
            self.panels.settings.tick_profile(dt);
        }
        if self.panels.settings.open || self.panels.effects.is_some() {
            self.panels.settings.tick_bars(dt);
            crate::app::update::settings_background::sync_monitors(self);
        }
    }
}
