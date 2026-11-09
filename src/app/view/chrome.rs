use iced::widget::{canvas, container};
use iced::{Element, Length};

use crate::frontend::scene::layout::Mode;

#[allow(clippy::wildcard_imports)]
use super::super::*;

const VERTICAL_MARGIN: f32 = 32.0;

pub(crate) fn bar_intent_message(intent: crate::frontend::ui::BarIntent) -> Message {
    use crate::frontend::ui::{BarAction, BarIntent};
    match intent {
        BarIntent::Hover(hit, menu_hit) => Message::BarHover(hit, menu_hit),
        BarIntent::Activate(action) => match action {
            BarAction::SetType { next, .. } => Message::SetTypeFilter(next),
            BarAction::Sort(mode) => Message::SetSort(mode),
            BarAction::Orient(orientation) => Message::SetOrient(orientation),
            BarAction::Resolution(resolution) => Message::SetResolution(resolution),
            BarAction::Favs => Message::ToggleFavourites,
            BarAction::Color(value) => Message::SetColorFilter(value),
            BarAction::FolderToggle => Message::FolderMenuToggle,
            BarAction::Settings => Message::ToggleSettings,
            BarAction::Stash => Message::Stash,
            BarAction::ShowHidden => Message::ToggleShowHidden,
            BarAction::Download => Message::OpenSourceBrowser,
            BarAction::Playlists => Message::Pl(crate::frontend::playlists::PlMsg::OpenPicker),
            BarAction::ColourStudio => Message::KeyColourStudio,
            BarAction::Widgets => Message::OpenWidgets,
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
            BarAction::ThemePinSettings => {
                Message::Theme(crate::frontend::theme_designer::ThemeMsg::ToggleWallpaperSettings)
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

pub(crate) fn bar_show(
    app: &App,
    config: &crate::infrastructure::config::Config,
) -> crate::frontend::ui::BarShow {
    crate::frontend::ui::BarShow {
        order: config.filter_bar_order(),
        hidden_buttons: config.filter_bar_hidden_buttons(),
        types: crate::frontend::ui::TYPES
            .iter()
            .copied()
            .filter(|ty| {
                config.filter_show(&format!("type.{}", crate::frontend::ui::BarShow::type_key(ty)))
            })
            .collect(),
        sorts: crate::frontend::ui::SORTS
            .iter()
            .map(|(mode, _)| *mode)
            .filter(|mode| config.filter_show(&format!("sort.{mode}")))
            .collect(),
        orient: config.filter_show("orient"),
        resolution: config.filter_show("resolution"),
        resolution_presets: config.resolution_presets(),
        folder: config.filter_show("folder"),
        favourites: config.filter_show("favourites"),
        random: config.filter_show("random"),
        colors: config.filter_show("colors"),
        theme: config.filter_show("theme"),
        tagcloud: config.filter_show("tagcloud"),
        download: config.filter_show("download"),
        playlists: config.filter_show("playlists"),
        colour_studio: config
            .flag_default_config(skwd_config::keys::filter_bar::SHOW_COLOUR_STUDIO),
        widgets: config.flag_default_config(skwd_config::keys::filter_bar::SHOW_WIDGETS),
        hidden: (app.library_session.library.catalog().has_hidden()
            || app.library_session.filters.reveal)
            .then_some(app.library_session.filters.reveal),
    }
}

fn theme_bar_model(
    app: &App,
    backend_now: &str,
    backend_menu_open: bool,
    backend_count: usize,
) -> crate::frontend::ui::ThemeBar {
    let settings = crate::app::update::theme::selected_wallpaper_settings(app);
    let setting = |key: &str| settings.as_ref().and_then(|settings| settings.get(key));
    let or_default = |key: &str, def: &str| {
        let val = setting(key)
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_else(|| app.config.str_path(key));
        if val.is_empty() { String::from(def) } else { val }
    };
    let mode = app.config.str_path(skwd_config::keys::theme::MODE);
    crate::frontend::ui::ThemeBar {
        backend: backend_now.to_owned(),
        menu_open: backend_menu_open,
        mode: or_default(skwd_config::keys::theme::MODE, &mode),
        static_theme: or_default(skwd_config::keys::theme::STATIC_THEME, "nord"),
        scheme: or_default(skwd_config::keys::matugen::SCHEME_TYPE, "scheme-fidelity"),
        style: or_default(skwd_config::keys::theme::STYLE, "natural"),
        iris_scheme: or_default(skwd_config::keys::theme::SCHEME, "tonal-spot"),
        color_index: setting(skwd_config::keys::matugen::COLOR_INDEX)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| app.config.num_path(skwd_config::keys::matugen::COLOR_INDEX) as u64)
            .min(3) as u32,
        wallust_palette: or_default(skwd_config::keys::theme::WALLUST_PALETTE, "dark"),
        wallust_colorspace: or_default(skwd_config::keys::theme::WALLUST_COLORSPACE, "lab"),
        pywal_saturate: app.config.str_path(skwd_config::keys::theme::PYWAL_SATURATE),
        noctalia_scheme: or_default(skwd_config::keys::theme::NOCTALIA_SCHEME, "m3-tonal-spot"),
        noctalia_pure_black: setting(skwd_config::keys::theme::NOCTALIA_PURE_BLACK)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_else(|| {
                app.config.flag_default_config(skwd_config::keys::theme::NOCTALIA_PURE_BLACK)
            }),
        backend_count,
        wallpaper_settings_pinned: crate::app::update::theme::wallpaper_settings_pinned(app),
    }
}

#[derive(Clone, Copy)]
pub(crate) enum BarExtent {
    Content,
    Expanded,
}

fn menu_side(y: f32, vh: f32, content_h: f32, menu_h: f32, preferred_up: bool) -> (bool, f32) {
    let above = (y * vh).min((vh - content_h).max(0.0));
    let below = (vh - above - content_h).max(0.0);
    let up = if above >= menu_h && below >= menu_h {
        preferred_up
    } else if above >= menu_h {
        true
    } else if below >= menu_h {
        false
    } else {
        above > below
    };
    (up, if up { above } else { below })
}

fn fit_menu_rows(
    model: &mut crate::frontend::ui::BarModel,
    available: f32,
    content_height: f32,
    scale: f32,
) {
    let menu_height = (model.height - content_height).max(0.0);
    if menu_height <= available || model.menu_visible_rows == 0 {
        return;
    }
    let row_height = crate::frontend::ui::MENU_ROW_H * scale;
    let chrome = menu_height - model.menu_visible_rows as f32 * row_height;
    let rows = ((available - chrome) / row_height).floor().max(1.0) as usize;
    model.fit_menu_rows(rows, scale);
}

fn anchored_bar_origin(
    position: (f32, f32),
    viewport: (f32, f32),
    model: &crate::frontend::ui::BarModel,
    content: Option<&crate::frontend::ui::BarModel>,
) -> (f32, f32) {
    let (vw, vh) = viewport;
    let (content_w, content_h, offset_x, offset_y) = if let Some(content) = content {
        let (content_x, content_y) =
            content.items.first().map_or((0.0, 0.0), |item| (item.x, item.y));
        let (expanded_x, expanded_y) =
            model.items.first().map_or((0.0, 0.0), |item| (item.x, item.y));
        (content.width.min(vw), content.height, content_x - expanded_x, content_y - expanded_y)
    } else {
        (model.width.min(vw), model.height, 0.0, 0.0)
    };
    (
        (position.0 * vw).min((vw - content_w).max(0.0)) + offset_x,
        (position.1 * vh).min((vh - content_h).max(0.0)) + offset_y,
    )
}

pub(crate) fn bar_bounds(app: &App, context: &super::layout::LayoutContext<'_>) -> iced::Rectangle {
    bar_geometry(app, context, BarExtent::Content).1
}

#[expect(clippy::too_many_lines, reason = "bar geometry declarative composition")]
fn bar_geometry(
    app: &App,
    context: &super::layout::LayoutContext<'_>,
    extent: BarExtent,
) -> (crate::frontend::ui::BarModel, iced::Rectangle) {
    let (vw, vh) = context.viewport;
    let mut menu_up = matches!(context.params.mode, Mode::Sandy);
    let (ox, oy) = context.config.filter_bar_offset();
    let scale = context.config.ui_scale();
    let vertical = context.vertical && !menu_up;
    let position = context.config.filter_bar_position();
    let downloads_enabled = context.config.wallhaven_enabled() || context.config.steam_enabled();
    let mut show = bar_show(app, context.config);
    if let Some(reorder) = app.chrome.bar.reorder.as_ref() {
        show.order = reorder.order();
    }
    let backend_now = crate::app::update::theme::wallpaper_theme_backend(app);
    let backend_options =
        crate::frontend::ui::backend_menu_options(app.theme.backends.as_deref(), &backend_now);
    let folder_menu_open = matches!(extent, BarExtent::Expanded)
        && app.chrome.bar.menu == Some(crate::frontend::ui::MenuKind::Folders);
    let backend_menu_open = matches!(extent, BarExtent::Expanded)
        && app.chrome.bar.menu == Some(crate::frontend::ui::MenuKind::Backends);
    let theme_bar = app
        .theme
        .bar_open
        .then(|| theme_bar_model(app, &backend_now, backend_menu_open, backend_options.len()));
    let tasks = app.daemon.tasks.bar_tasks();
    let build_model = |menu_up| {
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
            context.config.flag_default_config(skwd_config::keys::general::RANDOM_ROTATE),
            theme_bar.as_ref(),
            &tasks,
        );
        if vertical {
            let reserve = if position.is_some() {
                crate::frontend::ui::MENU_CHROME_H + crate::frontend::ui::MENU_ROW_H * scale
            } else {
                0.0
            };
            crate::frontend::ui::verticalize_bar(
                &mut model,
                (vh - VERTICAL_MARGIN - reserve).max(1.0),
                scale,
            );
        }
        model
    };
    let mut model = build_model(menu_up);
    let content = if position.is_some()
        && matches!(extent, BarExtent::Expanded)
        && (folder_menu_open || backend_menu_open)
    {
        Some(bar_geometry(app, context, BarExtent::Content).0)
    } else {
        None
    };
    if let (Some((_, y)), Some(content)) = (position, content.as_ref()) {
        let menu_height = (model.height - content.height).max(0.0);
        let (up, available) = menu_side(y, vh, content.height, menu_height, menu_up);
        if up != menu_up {
            menu_up = up;
            model = build_model(menu_up);
        }
        fit_menu_rows(&mut model, available, content.height, scale);
    }
    let (bar_w, bar_h) = (model.width.min(vw), model.height);
    let gap = 8.0 * scale;
    let (left, top) = if let Some(position) = position {
        anchored_bar_origin(position, (vw, vh), &model, content.as_ref())
    } else if menu_up {
        (
            (((vw - bar_w) * 0.5) + ox).max(0.0),
            (vh - crate::frontend::scene::sandy::BAR_GAP - bar_h + oy).max(0.0),
        )
    } else if vertical && context.params.mode == Mode::Grid {
        let group_w = bar_w + gap + context.params.grid.total_w();
        (
            (((vw - group_w) * 0.5) + ox).max(0.0),
            (((vh - context.params.grid.total_h()) * 0.5) + oy).max(0.0),
        )
    } else if vertical {
        ((16.0 + ox).max(0.0), (((vh - bar_h) * 0.5) + oy).max(0.0))
    } else if context.params.mode == Mode::Grid {
        let group_h = bar_h + gap + context.params.grid.total_h();
        ((((vw - bar_w) * 0.5) + ox).max(0.0), (((vh - group_h) * 0.5) + oy).max(0.0))
    } else {
        (
            (((vw - bar_w) * 0.5) + ox).max(0.0),
            (((vh - context.card_height()) * 0.5) + 13.0 + oy).max(0.0),
        )
    };
    let rect = iced::Rectangle { x: left, y: top, width: bar_w, height: bar_h };
    (model, rect)
}

pub(crate) fn filter_bar_target(
    app: &App,
    vw: f32,
    vh: f32,
) -> (crate::frontend::ui::BarModel, iced::Rectangle) {
    let context = super::layout::LayoutContext::current(app, (vw, vh));
    let (model, rect) = bar_geometry(app, &context, BarExtent::Expanded);
    let rect = crate::app::update::arrange::placement::resolve(
        app,
        crate::frontend::arrange::Target::Bar,
        rect,
    );
    (model, rect)
}

pub(crate) fn filter_bar_geometry(
    app: &App,
    vw: f32,
    vh: f32,
    extent: BarExtent,
) -> (crate::frontend::ui::BarModel, iced::Rectangle) {
    if !matches!(extent, BarExtent::Expanded) {
        let context = super::layout::LayoutContext::current(app, (vw, vh));
        return bar_geometry(app, &context, extent);
    }
    let (model, rect) = filter_bar_target(app, vw, vh);
    let shown = app
        .panels
        .arrange
        .as_ref()
        .and_then(|session| session.editor.shown(crate::frontend::arrange::Target::Bar));
    let rect = shown.map_or(rect, |shown| iced::Rectangle {
        x: shown.x,
        y: shown.y,
        width: shown.w,
        height: shown.h,
    });
    (model, rect)
}

pub(super) fn filter_bar_layer(app: &App, vw: f32, vh: f32) -> Element<'_, Message> {
    let (model, bounds) = filter_bar_geometry(app, vw, vh, crate::app::view::BarExtent::Expanded);
    let backend_now = crate::app::update::theme::wallpaper_theme_backend(app);
    let backend_options =
        crate::frontend::ui::backend_menu_options(app.theme.backends.as_deref(), &backend_now);
    let folder_menu_open = app.chrome.bar.menu == Some(crate::frontend::ui::MenuKind::Folders);
    let backend_menu_open = app.chrome.bar.menu == Some(crate::frontend::ui::MenuKind::Backends);
    let scale = app.config.ui_scale();
    let menu_count = if folder_menu_open {
        app.library_session.folder_options.len()
    } else if backend_menu_open {
        backend_options.len()
    } else {
        0
    };
    let menu_max_scroll = menu_count.saturating_sub(model.menu_visible_rows) as f32
        * crate::frontend::ui::MENU_ROW_H
        * scale;
    let bar: Element<'_, crate::frontend::ui::BarIntent> = canvas(crate::frontend::ui::FilterBar {
        model,
        hover: app.chrome.bar.hover,
        menu_open: folder_menu_open,
        backend_menu_open,
        backend: backend_now,
        backend_options,
        menu_hover: app.chrome.bar.menu_hover,
        folder_options: &app.library_session.folder_options,
        folder_hidden: &app.library_session.folder_hidden,
        folder_roots: &app.library_session.folder_roots,
        selected_folder: &app.library_session.filters.folder,
        menu_scroll: app.chrome.bar.menu_scroll.x.clamp(0.0, menu_max_scroll),
        theme_swatch: &app.theme.swatch,
        pal: &app.theme.palette,
        cache: app.chrome.bar.cache.canvas(),
        scale,
        fade: app.scene.open_fade()
            * app.chrome.filter_bar_fade()
            * if app.detail_open() { 0.0 } else { 1.0 },
        visual_style: crate::frontend::ui::BarVisualStyle::from_key(
            &app.config.filter_bar_visual_style(),
        ),
    })
    .width(Length::Fixed(bounds.width))
    .height(Length::Fixed(bounds.height))
    .into();
    let bar = bar.map(bar_intent_message);
    container(bar)
        .width(Length::Fill)
        .align_x(crate::frontend::ui::start())
        .padding(crate::frontend::ui::logical_padding(bounds.y, 0.0, 0.0, bounds.x))
        .into()
}

pub(crate) fn filter_bar_footprint(app: &App, vw: f32, vh: f32) -> (bool, f32, f32) {
    let menu_up = matches!(app.scene.mode, Mode::Sandy);
    let vertical = app.chrome.filter_bar_vertical && !menu_up;
    let (_, bounds) = filter_bar_geometry(app, vw, vh, crate::app::view::BarExtent::Content);
    let reveal = app.chrome.filter_bar_fade().clamp(0.0, 1.0);
    if vertical {
        (true, bounds.width * reveal, bounds.height.min(vh))
    } else {
        (false, bounds.width, bounds.height.min(vh) * reveal)
    }
}

mod tests;
