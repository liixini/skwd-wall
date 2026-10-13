use iced::Task;
use serde_json::json;

#[allow(clippy::wildcard_imports)]
use super::super::*;

pub(super) fn folder_menu_scroll(app: &mut App, dy: f32) -> Task<Message> {
    let count = app.library_session.folder_options.len();
    let vis = count.min(crate::frontend::ui::MENU_MAX_ROWS);
    let max =
        count.saturating_sub(vis) as f32 * crate::frontend::ui::MENU_ROW_H * app.config.ui_scale();
    app.chrome.bar.menu_scroll.retarget((app.chrome.bar.menu_scroll.target + dy).clamp(0.0, max));
    app.chrome.bar.cache.clear();
    app.retick();
    Task::none()
}

pub(super) fn toggle_audio(app: &mut App) -> Task<Message> {
    let muted = !app.config.wallpaper_mute();
    super::settings_policy::save_value(app, skwd_config::keys::wallpaper::MUTE, &json!(muted));
    app.panels.audio_playing =
        app.panels.audio_active && !muted && app.config.wallpaper_volume() > 0;
    app.chrome.bar.cache.clear();
    Task::none()
}

pub(super) fn pane_wheel(app: &mut App, key: &'static str, dy: f32) -> Task<Message> {
    let motion = app.chrome.motion;
    let pane =
        app.chrome.pane_scrolls.entry(key).or_insert_with(|| PaneScroll::at(0.0, f32::MAX, motion));
    let target = (pane.spring.target - dy * crate::frontend::ui::PANE_WHEEL_STEP)
        .clamp(0.0, pane.max.max(0.0));
    pane.spring.retarget(target);
    app.retick();
    Task::none()
}

pub(super) fn toggle_settings(app: &mut App) -> Task<Message> {
    if app.panels.settings.open {
        app.close_settings();
    } else {
        app.panels.settings.open = true;
        app.panels.settings.input_edit = None;
        app.panels.settings.keybind_capture = None;
        app.panels.settings.process_picker_open = false;
        app.panels.settings.weather_review_category = None;
        app.panels.settings.process_picker_query.clear();
        app.panels.settings.process_picker_manual.clear();
        app.panels.settings.process_picker_scroll = 0.0;
        app.panels.settings.search_open = false;
        app.panels.settings.search_query.clear();
        app.panels.settings.search_results.clear();
        app.apply_motion_speeds();
        app.panels.settings.entrance.run(0.0, 1.0);
        app.init_settings_inputs();
        app.call_tracked("wall.outputs", json!({}), Pending::Outputs);
        app.call_tracked("theme.backends", json!({}), Pending::ThemeBackends);
        app.call_tracked("theme.apps", json!({}), Pending::AppThemes);
        app.chrome.pane_scrolls.remove("settings");
    }
    app.retick();
    app.invalidate_settings();
    Task::none()
}

pub(super) fn toggle_theme_panel(app: &mut App) -> Task<Message> {
    if app.theme.bar_open {
        app.theme.bar_open = false;
        app.chrome.bar.menu = None;
    } else if !app.menu_capturing() {
        app.theme.bar_open = true;
        app.chrome.filter_bar_visible = true;
        app.set_base_swatch();
        app.call_tracked("theme.backends", json!({}), Pending::ThemeBackends);
        app.call_tracked("theme.apps", json!({}), Pending::AppThemes);
    }
    app.chrome.bar.cache.clear();
    app.retick();
    Task::none()
}

pub(super) fn pane_scrolled(app: &mut App, key: &'static str, y: f32, max: f32) -> Task<Message> {
    let motion = app.chrome.motion;
    let pane = app.chrome.pane_scrolls.entry(key).or_insert_with(|| PaneScroll::at(y, max, motion));
    pane.max = max;
    if pane.spring.settled() && (pane.spring.x - y).abs() > 1.0 {
        pane.spring.snap(y);
    }
    if pane.spring.target > max {
        pane.spring.retarget(max.max(0.0));
    }
    Task::none()
}

pub(super) fn toggle_help(app: &mut App) -> Task<Message> {
    if !app.input.help_open && app.menu_capturing() {
        return Task::none();
    }
    app.input.help_open = !app.input.help_open;
    app.retick();
    Task::none()
}

pub(super) fn toggle_bar_menu(app: &mut App, kind: crate::frontend::ui::MenuKind) -> Task<Message> {
    app.chrome.bar.menu = app.chrome.bar.menu.ne(&Some(kind)).then_some(kind);
    app.chrome.bar.menu_scroll.snap(0.0);
    app.chrome.bar.cache.clear();
    app.retick();
    Task::none()
}
