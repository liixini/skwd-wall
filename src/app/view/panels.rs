use iced::Element;

#[allow(clippy::wildcard_imports)]
use super::super::*;
use super::settings::settings_layer;

pub(crate) fn browser_intent_message(intent: crate::frontend::browser::BrowserIntent) -> Message {
    match intent {
        crate::frontend::browser::BrowserIntent::Update(message) => Message::Browser(message),
        crate::frontend::browser::BrowserIntent::Close => Message::CloseBrowser,
        crate::frontend::browser::BrowserIntent::Capture => Message::Noop,
    }
}

pub(super) fn panel_layers(app: &App) -> Vec<Element<'_, Message>> {
    let mut layers = Vec::new();

    if app.panels.settings.open && app.panels.theme_designer.is_none() {
        layers.push(settings_layer(app));
    }
    if let Some(br) = &app.source_browser.browser {
        let availability = crate::infrastructure::browser::source_availability(
            &app.config,
            app.daemon.steam_helper_available,
        );
        layers.push(
            crate::frontend::browser::view(
                br,
                &availability,
                app.config.browser_apply_button(br.source),
                app.scene.viewport,
                app.source_browser.wall.layout_grid(),
                app.source_browser.wall.scene.render.clone(),
                app.source_browser.wall.uploads.clone(),
                app.source_browser.wall.scene.frame_pool().clone(),
                &app.source_browser.wall.chrome_cache,
                app.source_browser.entrance.x,
                app.source_browser.spinner_phase,
                app.source_browser.shimmer_phase,
                app.source_browser.preview_animation,
                app.config.ui_scale(),
                &app.theme.palette,
                app.daemon.current_wallpaper_art.as_deref(),
            )
            .map(browser_intent_message),
        );
    }
    if let Some(pl) = &app.panels.playlists {
        layers.push(crate::frontend::playlists::view(
            pl,
            app.scene.viewport,
            app.config.ui_scale(),
            &app.theme.palette,
        ));
    }
    if let Some(cp) = &app.panels.card_picker {
        layers.push(crate::frontend::playlists::card_picker_view(
            cp,
            app.scene.viewport,
            app.config.ui_scale(),
            &app.theme.palette,
        ));
    }
    if let Some(td) = &app.panels.theme_designer {
        let saved = crate::infrastructure::theme::saved_palettes(
            &app.config.array_values(skwd_config::keys::theme::SAVED_THEMES),
        );
        layers.push(td.view(saved, app.scene.viewport, app.config.ui_scale(), &app.theme.palette));
    }
    if let Some(panel) = &app.panels.scene_properties {
        layers.push(crate::frontend::scene_properties::view(
            panel,
            app.scene.viewport,
            app.config.ui_scale(),
            &app.theme.palette,
        ));
    }
    if let Some(ed) = &app.panels.schedule {
        layers.push(ed.view(app.scene.viewport, app.config.ui_scale(), &app.theme.palette));
    }
    if let Some(eff) = &app.panels.effects {
        layers.push(eff.view(
            app.scene.viewport,
            app.config.ui_scale(),
            &app.theme.palette,
            &crate::rendering::effects::GpuPreviewRenderer,
        ));
    }
    if let Some(panel) = &app.panels.audio {
        layers.push(panel.view(app.scene.viewport, app.config.ui_scale(), &app.theme.palette));
    }
    layers
}

mod tests;
