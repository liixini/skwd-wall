use std::time::Instant;

use iced::widget::container;
use iced::{Alignment, Element, Length, Padding};
use serde_json::json;

use crate::frontend::theme::Palette;

#[allow(clippy::wildcard_imports)]
use super::*;

const THEME_DEBOUNCE_MS: u128 = 50;

pub(super) fn tint_follows_wallpaper(backend: &str) -> bool {
    !matches!(backend, "static" | "off")
}

pub(super) fn dwell_gate(
    hover: &mut Option<(usize, Instant)>,
    idx: usize,
    now: Instant,
    debounce_ms: u128,
) -> bool {
    match hover {
        Some((prev, _)) if *prev == idx => {}
        _ => *hover = Some((idx, now)),
    }
    hover.is_some_and(|(_, since)| now.duration_since(since).as_millis() >= debounce_ms)
}

impl App {
    pub(super) fn request_theme_previews(&mut self) {
        if !self.daemon.connected {
            return;
        }
        let backend = self.theme.audition_backend.clone();
        if self.theme.audition_pending_backend.as_deref() == Some(backend.as_str()) {
            return;
        }
        self.theme.audition_error = None;
        let id = self.call_tracked(
            "theme.previews",
            json!({"backend": backend.clone()}),
            Pending::ThemePreviews { backend: backend.clone() },
        );
        self.theme.audition_loading = id > 0;
        if id > 0 {
            self.theme.audition_pending_backend = Some(backend);
        }
    }

    pub(super) fn update_theme_preview(&mut self, now: Instant, dt: f32) -> bool {
        let off = self.theme.suspended
            || !tint_follows_wallpaper(&self.config.theme_backend())
            || !self.config.flag_default_true(skwd_config::keys::selector::LIVE_PREVIEW)
            || self.source_browser.browser.is_some()
            || self.panels.playlists.is_some()
            || self.menu_capturing();
        let target = (!off)
            .then(|| self.scene.theme_target())
            .flatten()
            .and_then(|fi| self.library_session.filtered.get(fi).map(|&si| si as usize));
        let Some(idx) = target else {
            return self.reset_theme_preview(dt);
        };
        if self.theme.preview_target != Some(idx) {
            if !dwell_gate(&mut self.theme.hover_since, idx, now, THEME_DEBOUNCE_MS) {
                return true;
            }
            self.theme.hover_since = None;
            self.theme.preview_target = Some(idx);
            if let Some(pal) = self.theme.cache.get(&idx).copied() {
                self.set_swatch_for(idx);
                self.start_fade(pal);
            } else {
                self.request_preview_palette(idx);
            }
            self.step_shell_preview(idx);
        }
        self.advance_fade(dt) || self.theme.job_pending.is_some()
    }

    fn reset_theme_preview(&mut self, dt: f32) -> bool {
        if self.theme.preview_target.take().is_some() {
            self.theme.hover_since = None;
            self.start_fade(self.theme.base_palette);
            if self.theme.bar_open {
                self.set_base_swatch();
            }
        }
        if self.theme.shell_preview_sent.take().is_some() {
            self.daemon.client.call("wall.shell_preview_end", serde_json::json!({}));
        }
        if !self.theme.bar_open {
            self.clear_swatch();
        }
        self.advance_fade(dt)
    }

    pub(super) fn shell_hover_enabled(&self) -> bool {
        match self.config.theme_backend().as_str() {
            "noctalia" => self.config.flag_default_true(skwd_config::keys::noctalia::HOVER_PREVIEW),
            "dms" => self.config.flag_default_true(skwd_config::keys::dms::HOVER_PREVIEW),
            backend => tint_follows_wallpaper(backend),
        }
    }

    pub(super) fn step_shell_preview(&mut self, idx: usize) {
        if !self.shell_hover_enabled() || self.theme.shell_preview_sent == Some(idx) {
            return;
        }
        let Some(item) = self.library_session.library.catalog().items.get(idx) else {
            return;
        };
        let image = &item.thumb;
        if image.is_empty() {
            return;
        }
        self.theme.shell_preview_sent = Some(idx);
        crate::zone!("shell_preview_ipc");
        self.daemon.client.call("wall.shell_preview", serde_json::json!({ "path": image }));
    }

    pub(super) fn start_fade(&mut self, to: Palette) {
        self.theme.fade_from = self.theme.palette;
        self.theme.fade_to = to;
        self.theme.fade_t.run(0.0, 1.0);
    }

    pub(super) fn advance_fade(&mut self, dt: f32) -> bool {
        if self.theme.fade_t.settled() {
            return false;
        }
        let moving = self.theme.fade_t.tick(dt);
        let eased = crate::frontend::animation::smoothstep(self.theme.fade_t.x);
        self.theme.palette = self.theme.fade_from.lerp(&self.theme.fade_to, eased);
        self.chrome.bar.cache.clear();
        self.invalidate_settings();
        moving
    }

    pub(super) fn request_preview_palette(&mut self, idx: usize) {
        if self.theme.job_pending == Some(idx) {
            return;
        }
        let Some(item) = self.library_session.library.catalog().items.get(idx) else { return };
        if item.thumb.is_empty() {
            return;
        }
        let backend = self.config.theme_backend();
        let id = self.call_tracked(
            "theme.preview",
            json!({ "image": item.thumb.clone() }),
            Pending::ThemePreview { card: idx, backend },
        );
        if id > 0 {
            self.theme.job_pending = Some(idx);
        }
    }

    pub(super) fn set_swatch_for(&mut self, idx: usize) {
        self.theme.swatch_target = Some(idx);
        if let Some(cached) = self.theme.swatch_cache.get(&idx) {
            self.theme.swatch = cached.clone();
        }
    }

    pub(super) fn clear_swatch(&mut self) {
        self.theme.swatch_target = None;
        if !self.theme.swatch.is_empty() {
            self.theme.swatch.clear();
        }
    }

    pub(super) fn set_base_swatch(&mut self) {
        self.theme.swatch_target = None;
        let pal = self.theme.base_palette;
        self.theme.swatch = vec![
            pal.primary,
            pal.tertiary,
            pal.surface_variant,
            pal.surface_container,
            pal.surface,
            pal.outline,
        ];
    }

    pub(super) fn invalidate_swatch(&mut self) {
        self.theme.swatch_cache.clear();
        self.theme.swatch_target = None;
        self.theme.preview_target = None;
    }
}

fn swatch_backend_label(backend: &str) -> &'static str {
    match backend {
        "matugen" => "matugen",
        "wallust" => "wallust",
        "pywal" => "pywal",
        "skwd-iris" => "skwd-iris",
        "skwd-pywal" => "skwd-pywal",
        "skwd-wallust" => "skwd-wallust",
        "iris" => "iris",
        _ => "native",
    }
}

pub(crate) fn swatch_overlay(app: &App) -> Option<Element<'_, Message>> {
    use iced::widget::{column, row, text};
    if app.config.theme_backend() == "off" {
        return None;
    }
    let scale = app.config.ui_scale();
    let sw = 22.0 * scale;
    let base = app.theme.base_palette;
    let fallback = [
        base.primary,
        base.tertiary,
        base.surface_variant,
        base.surface_container,
        base.surface,
        base.outline,
    ];
    let colors =
        if app.theme.swatch.is_empty() { fallback.as_slice() } else { app.theme.swatch.as_slice() };
    let cells: Vec<Element<'_, Message>> = colors
        .iter()
        .map(|cell| {
            let col = *cell;
            container(text(""))
                .width(Length::Fixed(sw))
                .height(Length::Fixed(sw))
                .style(move |_| {
                    crate::frontend::ui::box_style(
                        col,
                        crate::frontend::ui::with_alpha(iced::Color::BLACK, 0.22),
                    )
                })
                .into()
        })
        .collect();
    let strip = row(cells).spacing(0);
    let label = text(crate::i18n::tr_args!(
        "theme-bar-current",
        backend => swatch_backend_label(&app.config.theme_backend())
    ))
    .size(10.0 * scale)
    .color(crate::frontend::ui::with_alpha(app.theme.palette.surface_text, 0.6));
    let block = column(vec![label.into(), strip.into()]).spacing(4.0 * scale);
    Some(
        container(
            iced::widget::mouse_area(block)
                .on_press(Message::ToggleThemePanel)
                .interaction(iced::mouse::Interaction::Pointer),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Start)
        .align_y(Alignment::End)
        .padding(Padding { left: 20.0, bottom: 20.0, ..Padding::ZERO })
        .into(),
    )
}
