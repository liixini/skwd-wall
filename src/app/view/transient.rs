use iced::widget::container;
use iced::{Alignment, Element, Length, Padding};

#[allow(clippy::wildcard_imports)]
use super::super::*;

const EMPTY_FOLDER_KEY: &str = "status-library-folder-empty";

pub(crate) fn help_intent_message(intent: crate::frontend::ui::HelpIntent) -> Message {
    match intent {
        crate::frontend::ui::HelpIntent::Close => Message::ToggleHelp,
        crate::frontend::ui::HelpIntent::Capture => Message::Noop,
    }
}

pub(super) fn library_hint(app: &App) -> Option<Element<'_, Message>> {
    if let Some((id, name, _)) = &app.library_session.playlist_filter {
        let title =
            crate::i18n::tr_args!("playlists-picker-entry", name => name, id => id.to_string());
        let width =
            crate::frontend::ui::folio_action_width_in(&title, app.config.ui_scale(), 100.0, 400.0);
        let chip = crate::frontend::ui::folio_action(
            title,
            true,
            Some(Message::Pl(crate::frontend::playlists::PlMsg::OpenPicker)),
            Length::Fixed(width),
            app.config.ui_scale(),
            &app.theme.palette,
        );
        return Some(
            container(chip).width(Length::Fill).align_x(Alignment::Center).padding(16).into(),
        );
    }
    let folder = &app.library_session.filters.folder;
    let empty_source = app.library_session.filtered.is_empty()
        && skwd_config::library_folders(app.config.root()).contains(folder)
        && !app.library_session.library.catalog().items.iter().any(|item| {
            item.kind != crate::domain::library::catalog::WallpaperKind::We
                && std::path::Path::new(&item.path).starts_with(folder)
        });
    let hint = if empty_source && app.daemon.connected {
        crate::i18n::tr_args!(
            EMPTY_FOLDER_KEY,
            directory => folder.as_str()
        )
    } else {
        empty_library_hint(
            app.library_session.library.catalog().items.len(),
            skwd_config::library_destination(app.config.root(), false).as_deref(),
            app.daemon.connected,
            app.daemon.ever_connected,
        )?
    };
    let color = crate::frontend::ui::with_alpha(app.theme.palette.surface_text, 0.75);
    let hint_text = iced::widget::text(hint)
        .size(16)
        .font(crate::frontend::ui::UI_FONT)
        .color(color)
        .align_x(Alignment::Center)
        .width(Length::Fixed(520.0));
    Some(
        container(hint_text)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .padding(24)
            .into(),
    )
}

pub(super) fn help_layer(app: &App) -> Option<Element<'_, Message>> {
    app.input.help_open.then(|| {
        crate::frontend::ui::help_overlay(
            &app.input.bindings,
            &app.theme.palette,
            app.config.ui_scale(),
        )
        .map(help_intent_message)
    })
}

#[cfg(test)]
mod tests;

pub(super) fn hud_layer(app: &App) -> Option<Element<'_, Message>> {
    app.runtime_state.metrics.hud_enabled.then(|| {
        let hud = crate::contracts::presentation::HudSnapshot {
            metrics: app.runtime_state.metrics.hud_lines(),
            daemon: app.daemon.diagnostic.clone(),
        };
        container(crate::frontend::ui::hud(&hud, app.palette(), app.config.ui_scale()))
            .width(Length::Fill)
            .align_x(Alignment::End)
            .padding(10)
            .into()
    })
}

pub(super) fn toast_layer(app: &App) -> Option<Element<'_, Message>> {
    let (msg, since) = app.runtime_state.toast.as_ref()?;
    let elapsed = app.runtime_state.last_tick.unwrap_or(*since).duration_since(*since).as_millis();
    if elapsed >= TOAST_MS {
        return None;
    }
    let remaining = (TOAST_MS - elapsed) as f32;
    let fade_ms = app.motion_profile().duration_ms(crate::frontend::animation::MotionTier::Slow);
    let alpha = (remaining / fade_ms).min(1.0);
    let pal = app.theme.palette;
    let toast_max = (app.scene.viewport.0 * 0.8).clamp(160.0, 500.0);
    let toast = container(
        iced::widget::text(msg.clone())
            .size(13)
            .color(crate::frontend::ui::with_alpha(pal.primary_text, alpha)),
    )
    .padding(Padding::from([8.0, 14.0]))
    .max_width(toast_max)
    .style(move |_t| {
        crate::frontend::ui::bg_style(crate::frontend::ui::with_alpha(pal.primary, 0.92 * alpha))
    });
    Some(
        container(toast)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::End)
            .padding(Padding { bottom: 40.0, ..Padding::ZERO })
            .into(),
    )
}
