#[cfg(test)]
mod tests;

use crate::contracts::daemon::{
    APP_THEME_BUSY as BUSY, APP_THEME_CONFLICT as CONFLICT, APP_THEME_INTERRUPTED as INTERRUPTED,
    AppThemeStatus,
};
use crate::frontend::settings::ActionId;
use crate::i18n::tr;

pub(in crate::frontend::settings) fn state_label(state: &str, detail: &str) -> &'static str {
    if state == CONFLICT && detail == "custom-output" {
        return tr("settings-app-themes-custom-output-status");
    }
    tr(match state {
        "disconnected" => "settings-app-themes-disconnected",
        "applied" => "settings-app-themes-applied",
        "ready" => "settings-app-themes-ready",
        "not-installed" => "settings-app-themes-not-installed",
        CONFLICT => "settings-app-themes-conflict",
        "needs-review" => "settings-app-themes-review",
        "changed" => "settings-app-themes-changed",
        INTERRUPTED => "settings-app-themes-interrupted",
        "reload-needed" => "settings-app-themes-reload-needed",
        "reload-sent" => "settings-app-themes-reload-sent",
        "watching" => "settings-app-themes-watching",
        "on-next-open" => "settings-app-themes-next-open",
        BUSY => "settings-app-themes-busy",
        _ => "settings-app-themes-configured",
    })
}

pub fn busy(app: &AppThemeStatus) -> bool {
    app.state == BUSY
}

pub fn panel_actions(app: &AppThemeStatus, index: u8) -> Vec<ActionId> {
    panel_parts(app, index).actions.into_iter().map(|(id, _)| id).collect()
}

pub(super) struct PanelParts {
    pub lines: Vec<String>,
    pub actions: Vec<(ActionId, String)>,
}

fn help(app: &AppThemeStatus) -> &'static str {
    match app.state.as_str() {
        CONFLICT if app.detail == "custom-output" => tr(if app.can_adopt {
            "settings-app-themes-custom-switch"
        } else {
            "settings-app-themes-custom-conflict"
        }),
        CONFLICT => tr("settings-app-themes-other-owner"),
        "reload-needed" => tr("settings-app-themes-retry-reload"),
        _ => tr("settings-app-themes-restore-desc"),
    }
}

pub(super) fn panel_parts(app: &AppThemeStatus, index: u8) -> PanelParts {
    let idle = app.state != BUSY;
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
    let mut lines = vec![format!("{installed} · {config}"), help(app).to_string()];
    let mut actions = Vec::new();
    for (title, value) in [
        (tr("settings-app-themes-config"), &app.config_path),
        (tr("settings-app-themes-output"), &app.output_path),
    ] {
        if !value.is_empty() {
            lines.push(format!("{title}: {value}"));
        }
    }
    if app.enabled && app.can_disable && !matches!(app.state.as_str(), "changed" | INTERRUPTED) {
        let refresh = tr("settings-app-themes-refresh-colours").to_string();
        actions.push((ActionId::RefreshAppTheme(index), refresh));
    }
    if !app.template_path.is_empty() {
        lines.push(tr("settings-app-themes-template-desc").to_string());
        lines.push(app.template_path.clone());
        if idle && app.customized {
            let copy = tr("settings-app-themes-copy-template").to_string();
            actions.push((ActionId::CopyAppThemeTemplate(index), copy));
            let reset = tr("settings-app-themes-reset-template").to_string();
            actions.push((ActionId::CustomizeAppTheme(index, "reset-template"), reset));
        } else if idle {
            let create = tr("settings-app-themes-create-template").to_string();
            actions.push((ActionId::CustomizeAppTheme(index, "create-template"), create));
        }
    }
    if app.can_disconnect && idle {
        lines.push(tr("settings-app-themes-disconnect-desc").to_string());
        let disconnect = tr("settings-app-themes-disconnect").to_string();
        actions.push((ActionId::CustomizeAppTheme(index, "disconnect"), disconnect));
    }
    if app.can_reconnect
        && matches!(app.state.as_str(), "changed" | INTERRUPTED | "needs-review" | "disconnected")
    {
        lines.push(tr("settings-app-themes-reconnect-desc").to_string());
        let reconnect = tr("settings-app-themes-reconnect").to_string();
        actions.push((ActionId::CustomizeAppTheme(index, "reconnect"), reconnect));
    }
    if app.can_adopt {
        lines.push(tr("settings-app-themes-adopt-desc").to_string());
        actions.push((ActionId::AdoptAppTheme(index), tr("settings-app-themes-adopt").to_string()));
    }
    if !app.detail.is_empty() && app.state != CONFLICT {
        lines.push(app.detail.clone());
    }
    PanelParts { lines, actions }
}
