use iced::widget::{canvas, container};
use iced::{Alignment, Element, Length, Padding};

use crate::frontend::scene::layout::Mode;

#[allow(clippy::wildcard_imports)]
use super::super::*;

pub(crate) fn bar_intent_message(intent: crate::frontend::ui::BarIntent) -> Message {
    use crate::frontend::ui::{BarAction, BarIntent};
    match intent {
        BarIntent::Hover(hit, menu_hit) => Message::BarHover(hit, menu_hit),
        BarIntent::Activate(action) => match action {
            BarAction::SetType(kind) => Message::SetTypeFilter(kind),
            BarAction::Sort(mode) => Message::SetSort(mode),
            BarAction::Orient(orientation) => Message::SetOrient(orientation),
            BarAction::Resolution(resolution) => Message::SetResolution(resolution),
            BarAction::Favs => Message::ToggleFavourites,
            BarAction::Color(value) => Message::SetColorFilter(value),
            BarAction::FolderToggle => Message::FolderMenuToggle,
            BarAction::Settings => Message::ToggleSettings,
            BarAction::Download => Message::OpenSourceBrowser,
            BarAction::Playlists => Message::OpenPlaylists,
            BarAction::Random => Message::ToggleRandomRotate,
            BarAction::TagCloud => Message::OpenTagCloud,
            BarAction::ThemePanel => Message::ToggleThemePanel,
            BarAction::Audio => Message::ToggleAudio,
            BarAction::AudioPanel => Message::ToggleAudioPanel,
            BarAction::TaskControl { id, action } => Message::TaskControl { id, action },
            BarAction::ThemeBackendToggle => {
                Message::Theme(crate::frontend::theme_designer::ThemeMsg::BackendMenu)
            }
            BarAction::ThemeOpt(key, value) => {
                Message::Theme(crate::frontend::theme_designer::ThemeMsg::Option(key, value))
            }
        },
        BarIntent::SelectFolder(folder) => Message::SetFolder(folder),
        BarIntent::ToggleFolderMenu => Message::FolderMenuToggle,
        BarIntent::Theme(message) => Message::Theme(message),
        BarIntent::ScrollFolderMenu(delta) => Message::FolderMenuScroll(delta),
        BarIntent::StepVolume(step) => {
            Message::Audio(crate::frontend::audio_panel::AudioMsg::VolumeStep(step))
        }
    }
}

pub(in crate::app) fn overview_set(app: &App) -> bool {
    if !app.config.flag_default_config(skwd_config::keys::niri::OVERVIEW_BACKDROP)
        || app.config.flag_default_config(skwd_config::keys::niri::BACKDROP_FOLLOW_WALLPAPER)
    {
        return false;
    }
    app.scene.flipped().and_then(|fi| app.library_session.filtered.get(fi)).is_some_and(|&si| {
        let it = &app.library_session.library.catalog().items[si as usize];
        let img = it.backdrop_source();
        !img.is_empty() && app.config.str_path(skwd_config::keys::niri::BACKDROP) == img
    })
}

pub(super) fn card_h(app: &App, vh: f32) -> f32 {
    match app.scene.mode {
        Mode::Slices => app.scene.sp.slice_h + 110.0,
        Mode::Grid => app.scene.gp.total_h() + 85.0,
        Mode::Hex => app.scene.hp.content_h() + 140.0,
        Mode::Sandy => (vh - 90.0).max(200.0),
    }
}

fn bar_show(app: &App) -> crate::frontend::ui::BarShow {
    crate::frontend::ui::BarShow {
        types: crate::frontend::ui::TYPES
            .iter()
            .copied()
            .filter(|ty| {
                app.config
                    .filter_show(&format!("type.{}", crate::frontend::ui::BarShow::type_key(ty)))
            })
            .collect(),
        sorts: crate::frontend::ui::SORTS
            .iter()
            .map(|(mode, _)| *mode)
            .filter(|mode| app.config.filter_show(&format!("sort.{mode}")))
            .collect(),
        orient: app.config.filter_show("orient"),
        resolution: app.config.filter_show("resolution"),
        resolution_presets: app.config.resolution_presets(),
        folder: app.config.filter_show("folder"),
        favourites: app.config.filter_show("favourites"),
        random: app.config.filter_show("random"),
        colors: app.config.filter_show("colors"),
        theme: app.config.filter_show("theme"),
        tagcloud: app.config.filter_show("tagcloud"),
    }
}

fn theme_bar_model(
    app: &App,
    backend_now: &str,
    backend_menu_open: bool,
    backend_count: usize,
) -> crate::frontend::ui::ThemeBar {
    let or_default = |key: &str, def: &str| {
        let val = app.config.str_path(key);
        if val.is_empty() { String::from(def) } else { val }
    };
    crate::frontend::ui::ThemeBar {
        backend: backend_now.to_owned(),
        menu_open: backend_menu_open,
        mode: or_default(skwd_config::keys::matugen::MODE, "auto"),
        static_theme: or_default(skwd_config::keys::theme::STATIC_THEME, "nord"),
        scheme: or_default(skwd_config::keys::matugen::SCHEME_TYPE, "scheme-fidelity"),
        style: or_default(skwd_config::keys::theme::STYLE, "natural"),
        iris_scheme: or_default(skwd_config::keys::theme::SCHEME, "tonal-spot"),
        color_index: (app.config.num_path(skwd_config::keys::matugen::COLOR_INDEX) as u32).min(3),
        wallust_palette: or_default(skwd_config::keys::theme::WALLUST_PALETTE, "dark"),
        wallust_colorspace: or_default(skwd_config::keys::theme::WALLUST_COLORSPACE, "lab"),
        pywal_saturate: app.config.str_path(skwd_config::keys::theme::PYWAL_SATURATE),
        noctalia_scheme: or_default(skwd_config::keys::theme::NOCTALIA_SCHEME, "m3-tonal-spot"),
        noctalia_pure_black: app
            .config
            .flag_default_config(skwd_config::keys::theme::NOCTALIA_PURE_BLACK),
        backend_count,
    }
}

pub(super) fn filter_bar_layer(app: &App, vw: f32, vh: f32) -> Element<'_, Message> {
    let menu_up = matches!(app.scene.mode, Mode::Sandy);
    let (ox, oy) = app.config.filter_bar_offset();
    let scale = app.config.ui_scale();
    let vertical = app.chrome.filter_bar_vertical && !menu_up;
    let downloads_enabled = app.config.wallhaven_enabled() || app.config.steam_enabled();
    let show = bar_show(app);
    let backend_now = app.config.theme_backend();
    let backend_options =
        crate::frontend::ui::backend_menu_options(app.theme.backends.as_deref(), &backend_now);
    let folder_menu_open = app.chrome.bar.menu == Some(crate::frontend::ui::MenuKind::Folders);
    let backend_menu_open = app.chrome.bar.menu == Some(crate::frontend::ui::MenuKind::Backends);
    let theme_bar = app
        .theme
        .bar_open
        .then(|| theme_bar_model(app, &backend_now, backend_menu_open, backend_options.len()));
    let tasks = app.daemon.tasks.bar_tasks();
    let mut model = crate::frontend::ui::build_bar_with_tasks(
        &app.library_session.filters,
        &app.library_session.folder_options,
        folder_menu_open,
        app.library_session.visible_count,
        app.library_session.library.catalog().items.len(),
        scale,
        downloads_enabled,
        app.panels.audio_active,
        !app.panels.audio_playing,
        &show,
        if vertical { f32::MAX } else { (vw - 24.0).max(1.0) },
        menu_up,
        app.config.flag_default_config(skwd_config::keys::general::RANDOM_ROTATE),
        theme_bar.as_ref(),
        &tasks,
    );
    if vertical {
        crate::frontend::ui::verticalize_bar(&mut model, (vh - 32.0).max(1.0), scale);
    }
    let (bar_w, bar_h) = (model.width.min(vw), model.height);
    let gap = 8.0 * scale;
    let (left, top) = if menu_up {
        (
            (((vw - bar_w) * 0.5) + ox).max(0.0),
            (vh - crate::frontend::scene::sandy::BAR_GAP - bar_h + oy).max(0.0),
        )
    } else if vertical && app.scene.mode == Mode::Grid {
        let group_w = bar_w + gap + app.scene.gp.total_w();
        (
            (((vw - group_w) * 0.5) + ox).max(0.0),
            (((vh - app.scene.gp.total_h()) * 0.5) + oy).max(0.0),
        )
    } else if vertical {
        ((16.0 + ox).max(0.0), (((vh - bar_h) * 0.5) + oy).max(0.0))
    } else if app.scene.mode == Mode::Grid {
        let group_h = bar_h + gap + app.scene.gp.total_h();
        ((((vw - bar_w) * 0.5) + ox).max(0.0), (((vh - group_h) * 0.5) + oy).max(0.0))
    } else {
        (
            (((vw - bar_w) * 0.5) + ox).max(0.0),
            (((vh - card_h(app, vh)) * 0.5) + 13.0 + oy).max(0.0),
        )
    };
    let bar: Element<'_, crate::frontend::ui::BarIntent> = canvas(crate::frontend::ui::FilterBar {
        model,
        hover: app.chrome.bar.hover,
        menu_open: folder_menu_open,
        backend_menu_open,
        backend: backend_now,
        backend_options,
        menu_hover: app.chrome.bar.menu_hover,
        folder_options: &app.library_session.folder_options,
        selected_folder: &app.library_session.filters.folder,
        menu_scroll: app.chrome.bar.menu_scroll.x,
        theme_swatch: &app.theme.swatch,
        pal: &app.theme.palette,
        cache: &app.chrome.bar.cache,
        scale,
        fade: app.scene.open_fade()
            * app.chrome.filter_bar_fade()
            * if app.detail_open() { 0.0 } else { 1.0 },
        visual_style: crate::frontend::ui::BarVisualStyle::from_key(
            &app.config.filter_bar_visual_style(),
        ),
    })
    .width(Length::Fixed(bar_w))
    .height(Length::Fixed(bar_h))
    .into();
    let bar = bar.map(bar_intent_message);
    container(bar)
        .width(Length::Fill)
        .align_x(Alignment::Start)
        .padding(Padding { top, left, ..Padding::ZERO })
        .into()
}

pub(crate) fn filter_bar_footprint(app: &App, vw: f32, vh: f32) -> (bool, f32, f32) {
    let menu_up = matches!(app.scene.mode, Mode::Sandy);
    let vertical = app.chrome.filter_bar_vertical && !menu_up;
    let scale = app.config.ui_scale();
    let show = bar_show(app);
    let backend_now = app.config.theme_backend();
    let backend_options =
        crate::frontend::ui::backend_menu_options(app.theme.backends.as_deref(), &backend_now);
    let theme_bar = app
        .theme
        .bar_open
        .then(|| theme_bar_model(app, &backend_now, false, backend_options.len()));
    let tasks = app.daemon.tasks.bar_tasks();
    let mut model = crate::frontend::ui::build_bar_with_tasks(
        &app.library_session.filters,
        &app.library_session.folder_options,
        false,
        app.library_session.visible_count,
        app.library_session.library.catalog().items.len(),
        scale,
        app.config.wallhaven_enabled() || app.config.steam_enabled(),
        app.panels.audio_active,
        !app.panels.audio_playing,
        &show,
        if vertical { f32::MAX } else { (vw - 24.0).max(1.0) },
        menu_up,
        app.config.flag_default_config(skwd_config::keys::general::RANDOM_ROTATE),
        theme_bar.as_ref(),
        &tasks,
    );
    if vertical {
        crate::frontend::ui::verticalize_bar(&mut model, (vh - 32.0).max(1.0), scale);
    }
    let reveal = app.chrome.filter_bar_fade().clamp(0.0, 1.0);
    if vertical {
        (true, model.width.min(vw) * reveal, model.height.min(vh))
    } else {
        (false, model.width.min(vw), model.height.min(vh) * reveal)
    }
}

mod tests;
