#[cfg(test)]
mod tests;

use iced::widget::{column, container, text};
use iced::{Alignment, Element, Length, Padding};

use crate::app::Message;
use crate::contracts::daemon::AppThemeStatus;
use crate::frontend::components::with_alpha;
use crate::frontend::settings::{ActionId, SettingsMsg};
use crate::frontend::theme::Palette;
use crate::frontend::ui::{TYPE_SMALL, folio_action, label, row};
use crate::i18n::tr;

fn state_label(state: &str, detail: &str) -> &'static str {
    if state == "conflict" && detail == "custom-output" {
        return tr("settings-app-themes-custom-output-status");
    }
    tr(match state {
        "disconnected" => "settings-app-themes-disconnected",
        "applied" => "settings-app-themes-applied",
        "ready" => "settings-app-themes-ready",
        "not-installed" => "settings-app-themes-not-installed",
        "conflict" => "settings-app-themes-conflict",
        "needs-review" => "settings-app-themes-review",
        "changed" => "settings-app-themes-changed",
        "interrupted" => "settings-app-themes-interrupted",
        "reload-needed" => "settings-app-themes-reload-needed",
        "reload-sent" => "settings-app-themes-reload-sent",
        "watching" => "settings-app-themes-watching",
        "on-next-open" => "settings-app-themes-next-open",
        "busy" => "settings-app-themes-busy",
        _ => "settings-app-themes-configured",
    })
}

#[derive(Clone, Copy)]
pub(super) struct Position {
    pub row: usize,
    pub app: usize,
}

pub(super) fn view(
    app: AppThemeStatus,
    position: Position,
    expanded: bool,
    focused: bool,
    scale: f32,
    palette: &Palette,
) -> Element<'_, Message> {
    let index = position.app;
    let available = app.can_disable || app.can_enable;
    let action = available.then_some(Message::Settings(SettingsMsg::Run(ActionId::SetAppTheme(
        index as u8,
        !app.enabled && !app.can_disable,
    ))));
    let status = state_label(&app.state, &app.detail);
    let installed = tr(if app.installed {
        "settings-app-themes-installed"
    } else {
        "settings-app-themes-not-installed"
    });
    let config = tr(if app.config_found {
        "settings-app-themes-config-found"
    } else {
        "settings-app-themes-config-create"
    });
    let details_message = Message::Settings(SettingsMsg::ToggleDetails(
        format!("app-theme:{}", app.id),
        position.row,
    ));
    let copy = column![
        label(app.name, 22.0, scale, palette.surface_text),
        label(
            format!("{installed} · {config}"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.65)
        ),
    ]
    .spacing(5.0 * scale)
    .width(Length::Fill);
    let switch = folio_action(
        tr(if app.state == "busy" {
            "settings-app-themes-busy"
        } else if app.state == "interrupted" {
            "settings-app-themes-undo"
        } else if app.enabled {
            "settings-app-themes-on"
        } else {
            "settings-app-themes-off"
        }),
        app.enabled,
        action,
        Length::Fixed(100.0 * scale),
        scale,
        palette,
    );
    let header = row![copy, switch].spacing(24.0 * scale).align_y(Alignment::Center);
    let detail_action = folio_action(
        tr(if expanded { "settings-app-themes-hide" } else { "settings-app-themes-details" }),
        expanded,
        Some(details_message),
        Length::Fixed(100.0 * scale),
        scale * 0.85,
        palette,
    );
    let mut actions = row![].spacing(8.0 * scale);
    if app.enabled && app.can_disable && !matches!(app.state.as_str(), "changed" | "interrupted") {
        actions = actions.push(folio_action(
            tr("settings-app-themes-refresh-colours"),
            false,
            Some(Message::Settings(SettingsMsg::Run(ActionId::RefreshAppTheme(index as u8)))),
            Length::Fixed(140.0 * scale),
            scale * 0.85,
            palette,
        ));
    }
    actions = actions.push(detail_action);
    let status_row = row![
        container(label(
            status,
            TYPE_SMALL,
            scale,
            if app.enabled { palette.primary } else { palette.surface_text }
        ))
        .width(Length::Fill),
        actions,
    ]
    .spacing(12.0 * scale)
    .align_y(Alignment::Center);
    let mut body = column![header, status_row].spacing(16.0 * scale);
    if expanded {
        let help = match app.state.as_str() {
            "conflict" if app.detail == "custom-output" => tr(if app.can_adopt {
                "settings-app-themes-custom-switch"
            } else {
                "settings-app-themes-custom-conflict"
            }),
            "conflict" => tr("settings-app-themes-other-owner"),
            "reload-needed" => tr("settings-app-themes-retry-reload"),
            _ => tr("settings-app-themes-restore-desc"),
        };
        body = body.push(label(help, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.75)));
        for (title, value) in [
            (tr("settings-app-themes-config"), app.config_path),
            (tr("settings-app-themes-output"), app.output_path),
        ] {
            body = body.push(
                column![
                    label(title, TYPE_SMALL, scale, palette.primary),
                    label(value, TYPE_SMALL, scale, palette.surface_text)
                ]
                .spacing(3.0 * scale),
            );
        }
        if !app.template_path.is_empty() {
            body = body.push(label(
                tr("settings-app-themes-template-desc"),
                TYPE_SMALL,
                scale,
                palette.surface_text,
            ));
            body = body.push(label(app.template_path.clone(), TYPE_SMALL, scale, palette.primary));
            let template_action = if app.customized {
                ActionId::CopyAppThemeTemplate(index as u8)
            } else {
                ActionId::CustomizeAppTheme(index as u8, "create-template")
            };
            body = body.push(folio_action(
                tr(if app.customized {
                    "settings-app-themes-copy-template"
                } else {
                    "settings-app-themes-create-template"
                }),
                false,
                (app.state != "busy")
                    .then_some(Message::Settings(SettingsMsg::Run(template_action))),
                Length::Fill,
                scale * 0.85,
                palette,
            ));
            if app.customized {
                body = body.push(folio_action(
                    tr("settings-app-themes-reset-template"),
                    false,
                    (app.state != "busy").then_some(Message::Settings(SettingsMsg::Run(
                        ActionId::CustomizeAppTheme(index as u8, "reset-template"),
                    ))),
                    Length::Fill,
                    scale * 0.85,
                    palette,
                ));
            }
        }
        if app.can_disconnect {
            body = body.push(label(
                tr("settings-app-themes-disconnect-desc"),
                TYPE_SMALL,
                scale,
                palette.surface_text,
            ));
            body = body.push(folio_action(
                tr("settings-app-themes-disconnect"),
                false,
                (app.state != "busy").then_some(Message::Settings(SettingsMsg::Run(
                    ActionId::CustomizeAppTheme(index as u8, "disconnect"),
                ))),
                Length::Fill,
                scale * 0.85,
                palette,
            ));
        }
        if app.can_reconnect
            && matches!(
                app.state.as_str(),
                "changed" | "interrupted" | "needs-review" | "disconnected"
            )
        {
            body = body.push(label(
                tr("settings-app-themes-reconnect-desc"),
                TYPE_SMALL,
                scale,
                palette.surface_text,
            ));
            body = body.push(folio_action(
                tr("settings-app-themes-reconnect"),
                false,
                Some(Message::Settings(SettingsMsg::Run(ActionId::CustomizeAppTheme(
                    index as u8,
                    "reconnect",
                )))),
                Length::Fill,
                scale * 0.85,
                palette,
            ));
        }
        if app.can_adopt {
            body = body.push(label(
                tr("settings-app-themes-adopt-desc"),
                TYPE_SMALL,
                scale,
                palette.surface_text,
            ));
            body = body.push(folio_action(
                tr("settings-app-themes-adopt"),
                false,
                Some(Message::Settings(SettingsMsg::Run(ActionId::AdoptAppTheme(index as u8)))),
                Length::Fixed(240.0 * scale),
                scale,
                palette,
            ));
        }
        if !app.detail.is_empty() && app.state != "conflict" {
            body = body.push(label(app.detail, TYPE_SMALL, scale, palette.tertiary));
        }
    }
    let rule = container(text(""))
        .width(Length::Fill)
        .height(Length::Fixed(if focused { 2.0 } else { 1.0 }))
        .style(move |_| {
            crate::frontend::ui::bg_style(with_alpha(
                if focused { palette.primary } else { palette.outline },
                0.6,
            ))
        });
    column![
        rule,
        container(body).padding(Padding {
            top: 18.0 * scale,
            right: 18.0 * scale,
            bottom: 20.0 * scale,
            left: 18.0 * scale
        })
    ]
    .width(Length::Fill)
    .into()
}
