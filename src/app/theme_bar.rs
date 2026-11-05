use std::time::Instant;

use iced::widget::container;
use iced::{Element, Length, Rectangle, Size};
use serde_json::json;

use crate::app::update::arrange::placement;
use crate::frontend::arrange::Target;
use crate::frontend::theme::Palette;
use crate::infrastructure::config::Config;

#[allow(clippy::wildcard_imports)]
use super::*;

const THEME_DEBOUNCE_MS: u128 = 50;
const BACKEND_OFF: &str = "off";
pub(crate) const SWATCH_CELL: f32 = 22.0;
const SWATCH_LABEL: f32 = 10.0;
const SWATCH_LABEL_LINE: f32 = 1.3;
const SWATCH_GAP: f32 = 4.0;
pub(crate) const SWATCH_STACK_GAP: f32 = crate::app::view::FLOAT_CLEAR;
const SWATCH_LABEL_KEY: &str = "theme-bar-current";

pub(super) fn tint_follows_wallpaper(backend: &str) -> bool {
    !matches!(backend, "static" | BACKEND_OFF)
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
        let Some(item) = self.library_session.library.catalog().items.get(idx) else {
            return self.reset_theme_preview(dt);
        };
        let (backend, pinned) =
            crate::app::update::theme::wallpaper_theme_state_for(self, &item.key);
        if backend == BACKEND_OFF || (backend == "static" && !pinned) {
            return self.reset_theme_preview(dt);
        }
        if self.theme.preview_target != Some(idx) {
            if !dwell_gate(&mut self.theme.hover_since, idx, now, THEME_DEBOUNCE_MS) {
                return true;
            }
            self.theme.hover_since = None;
            self.theme.preview_target = Some(idx);
            if let Some(pal) = self.theme.cache.get(&idx).copied() {
                self.set_swatch_for(idx);
                self.start_fade(&pal);
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
            let palette = self.theme.base_palette;
            self.start_fade(&palette);
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
        self.shell_hover_enabled_for(&self.config.theme_backend(), false)
    }

    fn shell_hover_enabled_for(&self, backend: &str, pinned: bool) -> bool {
        match backend {
            "noctalia" => self.config.flag_default_true(skwd_config::keys::noctalia::HOVER_PREVIEW),
            "dms" => self.config.flag_default_true(skwd_config::keys::dms::HOVER_PREVIEW),
            "static" => pinned,
            backend => tint_follows_wallpaper(backend),
        }
    }

    pub(super) fn step_shell_preview(&mut self, idx: usize) {
        if self.theme.shell_preview_sent == Some(idx) {
            return;
        }
        let Some(item) = self.library_session.library.catalog().items.get(idx) else {
            return;
        };
        let (backend, pinned) =
            crate::app::update::theme::wallpaper_theme_state_for(self, &item.key);
        let enabled = if pinned {
            self.shell_hover_enabled_for(&backend, true)
        } else {
            self.shell_hover_enabled()
        };
        if !enabled {
            if self.theme.shell_preview_sent.take().is_some() {
                self.daemon.client.call("wall.shell_preview_end", json!({}));
            }
            return;
        }
        let image = &item.thumb;
        if image.is_empty() {
            return;
        }
        self.theme.shell_preview_sent = Some(idx);
        crate::zone!("shell_preview_ipc");
        self.daemon.client.call("wall.shell_preview", serde_json::json!({ "path": image }));
    }

    pub(super) fn start_fade(&mut self, to: &Palette) {
        self.theme.fade_from = self.theme.palette;
        self.theme.fade_to = *to;
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
        let backend = crate::app::update::theme::wallpaper_theme_backend_for(self, &item.key);
        let settings = crate::app::update::theme::wallpaper_settings(self, &item.key);
        let id = self.call_tracked(
            "theme.preview",
            json!({ "image": item.thumb.clone(), "settings": settings }),
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

fn swatch_caption(config: &Config) -> String {
    crate::i18n::tr_args!(
        SWATCH_LABEL_KEY,
        backend => swatch_backend_label(&config.theme_backend())
    )
}

pub(crate) fn swatch_available(config: &Config) -> bool {
    config.theme_backend() != BACKEND_OFF
}

pub(crate) fn swatch_on(config: &Config) -> bool {
    swatch_available(config) && config.flag_default_true(skwd_config::keys::filter_bar::SHOW_SWATCH)
}

pub(crate) const fn swatch_height(scale: f32) -> f32 {
    (SWATCH_LABEL * SWATCH_LABEL_LINE + SWATCH_GAP + SWATCH_CELL) * scale
}

fn swatch_colours(app: &App) -> Vec<iced::Color> {
    if !app.theme.swatch.is_empty() {
        return app.theme.swatch.clone();
    }
    let base = app.theme.base_palette;
    vec![
        base.primary,
        base.tertiary,
        base.surface_variant,
        base.surface_container,
        base.surface,
        base.outline,
    ]
}

pub(crate) fn swatch_bounds(app: &App, config: &Config, viewport: (f32, f32)) -> Rectangle {
    let scale = config.ui_scale();
    let label = crate::frontend::ui::cached_line_width(
        &swatch_caption(config),
        SWATCH_LABEL * scale,
        crate::frontend::ui::UI_FONT,
    );
    let strip = swatch_colours(app).len() as f32 * SWATCH_CELL * scale;
    let size = Size::new(label.max(strip), swatch_height(scale));
    let arrange = crate::app::view::arrange_bounds(app, config, viewport);
    let stacked =
        placement::present(config, Target::Arrange) && config.arrange_position().is_none();
    let left = crate::app::view::float_left(app, config, viewport);
    let rest = if stacked {
        arrange.y - SWATCH_STACK_GAP
    } else {
        crate::app::view::rest_bottom(viewport.1)
    };
    let bottom = crate::app::view::clear_bottom(app, config, viewport, (left, rest), size);
    placement::floated(config.swatch_position(), (left, bottom), size, viewport)
}

pub(crate) fn swatch_overlay(app: &App) -> Option<Element<'_, Message>> {
    use iced::widget::{column, row, text};
    if !swatch_on(&app.config) {
        return None;
    }
    let scale = app.config.ui_scale();
    let sw = SWATCH_CELL * scale;
    let cells: Vec<Element<'_, Message>> = swatch_colours(app)
        .into_iter()
        .map(|col| {
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
    let label = text(swatch_caption(&app.config))
        .font(crate::frontend::ui::UI_FONT)
        .size(SWATCH_LABEL * scale)
        .line_height(iced::widget::text::LineHeight::Relative(SWATCH_LABEL_LINE))
        .color(crate::frontend::ui::with_alpha(app.theme.palette.surface_text, 0.6));
    let block = column(vec![label.into(), strip.into()]).spacing(SWATCH_GAP * scale);
    let area = iced::widget::mouse_area(block)
        .on_press(Message::ToggleThemePanel)
        .interaction(iced::mouse::Interaction::Pointer);
    Some(crate::app::view::pinned(app, Target::Swatch, area.into()))
}
