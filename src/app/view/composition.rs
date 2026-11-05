use iced::widget::{canvas, shader, stack};
use iced::{Element, Length};

use crate::app::scene::SceneProgram;
use crate::frontend::strand::draw::stage_alpha;

#[allow(clippy::wildcard_imports)]
use super::super::*;
use super::chrome::{filter_bar_layer, overview_set};
use super::panels::panel_layers;
use super::tags::{
    card_tag_drawer, card_tag_remove_overlay, selection_marks, tag_add_overlay, tag_cloud_layer,
    tag_mode_panel,
};
use super::transient::{help_layer, hud_layer, library_hint, toast_layer};

const OVERLAY_VISIBILITY: f32 = 0.02;

static VIEWS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub(crate) fn view_count() -> u64 {
    VIEWS.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn view(app: &App, window: iced::window::Id) -> Element<'_, Message> {
    if app.runtime_state.overlay != Some(window) || app.runtime_state.stashed {
        return iced::widget::Space::new().into();
    }
    crate::app::warm::note_overlay_drawn();
    overlay_view(app)
}

pub fn view_single(app: &App) -> Element<'_, Message> {
    if app.runtime_state.stashed {
        return iced::widget::Space::new().into();
    }
    overlay_view(app)
}

pub(crate) fn entry_shown(app: &App, overlays_clear: bool, settled: bool) -> bool {
    overlays_clear
        && settled
        && crate::app::update::arrange::placement::present(
            &app.config,
            crate::frontend::arrange::Target::Arrange,
        )
}

pub(crate) fn scene_fade(app: &App) -> f32 {
    app.panels.strand.as_ref().map_or(1.0, |strand| 1.0 - stage_alpha(strand))
}

fn overlay_view(app: &App) -> Element<'_, Message> {
    crate::zone!("overlay_view");
    VIEWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if app.theme.audition_open {
        let current_value = app
            .theme
            .audition_previews
            .first()
            .map(|preview| match preview.key.as_str() {
                skwd_config::keys::theme::SCHEME => {
                    let value = app.config.str_path(skwd_config::keys::theme::SCHEME);
                    if value.is_empty() { String::from("tonal-spot") } else { value }
                }
                skwd_config::keys::theme::STYLE => {
                    let value = app.config.str_path(skwd_config::keys::theme::STYLE);
                    if value.is_empty() { String::from("natural") } else { value }
                }
                skwd_config::keys::theme::STATIC_THEME => {
                    let value = app.config.str_path(skwd_config::keys::theme::STATIC_THEME);
                    if value.is_empty() { String::from("nord") } else { value }
                }
                skwd_config::keys::theme::WALLUST_PALETTE => {
                    let value = app.config.str_path(skwd_config::keys::theme::WALLUST_PALETTE);
                    if value.is_empty() { String::from("dark") } else { value }
                }
                skwd_config::keys::matugen::SCHEME_TYPE => {
                    let value = app.config.str_path(skwd_config::keys::matugen::SCHEME_TYPE);
                    if value.is_empty() { String::from("scheme-fidelity") } else { value }
                }
                key => app.config.str_path(key),
            })
            .unwrap_or_default();
        return crate::frontend::theme_audition::view(
            &app.theme.audition_previews,
            &app.theme.audition_backends,
            &app.theme.audition_backend,
            &app.config.theme_backend(),
            &current_value,
            app.theme.audition_loading,
            app.theme.audition_error.as_deref(),
            app.scene.viewport,
            app.config.ui_scale(),
            &app.theme.palette,
        );
    }
    let (vw, vh) = app.scene.viewport;
    let vis = app.scene.render.vis;
    let settled = vis > 0.985;
    let scene: Element<'_, Message> = shader(SceneProgram {
        render: app.scene.render.clone(),
        uploads: app.preview_resources.uploads.clone(),
        pool: app.scene.frame_pool().clone(),
        frame_clock: app.runtime_state.frame_clock.clone(),
        pointer_enabled: !app.detail_open() && !app.menu_capturing(),
        fade: scene_fade(app),
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into();
    if let Some(strand) = app.panels.strand.as_ref() {
        return stack![scene, strand.view()].into();
    }
    let fade = app.scene.open_fade();
    let folio_sheet_open = app.panels.settings.open || app.panels.theme_designer.is_some();
    let mut layers = stack![scene];

    if fade > OVERLAY_VISIBILITY {
        layers = layers.push(
            canvas(crate::frontend::ui::ChromeCanvas {
                render: app.scene.render.clone(),
                pal: &app.theme.palette,
                cache: &app.chrome.cache,
                overview_set: overview_set(app),
                show_type_badges: app
                    .config
                    .flag_default_config(skwd_config::keys::selector::SHOW_TYPE_BADGES)
                    && !app
                        .runtime_state
                        .demo
                        .as_ref()
                        .is_some_and(|session| session.type_badges_suppressed),
                show_video_indicators: !app
                    .runtime_state
                    .demo
                    .as_ref()
                    .is_some_and(|session| session.type_badges_suppressed),
                fade,
            })
            .width(Length::Fill)
            .height(Length::Fill),
        );
    }
    if settled
        && app.tags.mode
        && let Some(marks) = selection_marks(app)
    {
        layers = layers.push(marks);
    }
    if settled && let Some(el) = card_tag_remove_overlay(app) {
        layers = layers.push(el);
    }
    if settled && let Some(el) = tag_add_overlay(app) {
        layers = layers.push(el);
    }
    if settled && let Some(el) = card_tag_drawer(app) {
        layers = layers.push(el);
    }
    if fade * app.chrome.filter_bar_fade() > OVERLAY_VISIBILITY
        && vh > 0.0
        && !folio_sheet_open
        && app.chrome.bar.reorder.is_none()
    {
        layers = layers.push(filter_bar_layer(app, vw, vh));
    }

    let overlays_clear = !folio_sheet_open
        && !app.detail_open()
        && app.source_browser.browser.is_none()
        && app.panels.playlists.is_none()
        && app.panels.effects.is_none()
        && app.panels.card_picker.is_none()
        && app.panels.colour_studio.is_none();
    if settled
        && overlays_clear
        && let Some(swatch) = swatch_overlay(app)
    {
        layers = layers.push(swatch);
    }
    if settled
        && overlays_clear
        && let Some(hint) = library_hint(app)
    {
        layers = layers.push(hint);
    }

    for panel in panel_layers(app) {
        layers = layers.push(panel);
    }
    if tag_cloud_visible(app.tags.cloud_open || app.panels.arrange.is_some(), folio_sheet_open, vh)
    {
        layers = layers.push(tag_cloud_layer(app, vh));
    }
    if let Some(help) = help_layer(app) {
        layers = layers.push(help);
    }
    if let Some(hud) = hud_layer(app) {
        layers = layers.push(hud);
    }
    if let Some(toast) = toast_layer(app) {
        layers = layers.push(toast);
    }
    if settled && app.tags.mode {
        layers = layers.push(tag_mode_panel(app));
    }

    if entry_shown(app, overlays_clear, settled) {
        layers = layers.push(super::arrange::entry(app));
    }
    if app.panels.arrange.is_some() {
        layers = layers.push(super::arrange::editor(app));
    }
    layers.into()
}

pub(crate) fn tag_cloud_visible(
    cloud_open: bool,
    settings_open: bool,
    viewport_height: f32,
) -> bool {
    cloud_open && !settings_open && viewport_height > 0.0
}
