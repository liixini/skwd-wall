#![cfg(test)]

use super::{
    BarAction, BarModel, BarShow, ThemeBar, build_bar_with_tasks, next_orient, next_resolution,
    orient_label, resolution_label,
};
use crate::domain::library::filter::{Filters, ResolutionPreset};

#[allow(clippy::fn_params_excessive_bools)]
fn build_bar(
    filters: &Filters,
    folder_options: &[String],
    folder_menu_open: bool,
    filtered: usize,
    total: usize,
    scale: f32,
    downloads_enabled: bool,
    audio_visible: bool,
    audio_muted: bool,
    _tagging_visible: bool,
    _analysis_running: bool,
    _analysis_progress: Option<(usize, usize)>,
    _analysis_eta: &str,
    show: &BarShow,
    max_width: f32,
    menu_up: bool,
    random_on: bool,
    theme: Option<&ThemeBar>,
) -> BarModel {
    build_bar_with_tasks(
        filters,
        folder_options,
        folder_menu_open,
        filtered,
        total,
        scale,
        downloads_enabled,
        audio_visible,
        audio_muted,
        show,
        max_width,
        menu_up,
        random_on,
        theme,
        &[],
    )
}

fn has_download(downloads_enabled: bool) -> bool {
    let filters = Filters::default();
    let model = build_bar(
        &filters,
        &[],
        false,
        0,
        0,
        1.0,
        downloads_enabled,
        true,
        false,
        false,
        false,
        None,
        "",
        &BarShow::all(),
        0.0,
        false,
        false,
        None,
    );
    model.items.iter().any(|item| matches!(item.action, Some(BarAction::Download)))
}

#[test]
fn download_hidden() {
    assert!(!has_download(false));
}

#[test]
fn analysis_progress_off_bar() {
    let filters = Filters::default();
    let model = build_bar(
        &filters,
        &[],
        false,
        0,
        42,
        1.0,
        false,
        false,
        false,
        true,
        true,
        Some((0, 42)),
        "",
        &BarShow::all(),
        0.0,
        false,
        false,
        None,
    );
    assert!(!model.items.iter().any(|item| item.label.contains("estimating ETA")));
}

#[test]
fn task_notice_progress() {
    let mut task =
        crate::contracts::daemon::TaskStatus::running("semantic-index", "index", "Building index");
    task.progress = 270;
    task.total = 43_576;
    task.capabilities.pause = true;
    task.capabilities.stop = true;
    let tasks = [&task];
    let model = build_bar_with_tasks(
        &Filters::default(),
        &[],
        false,
        0,
        0,
        1.0,
        false,
        false,
        false,
        &BarShow::all(),
        0.0,
        false,
        false,
        None,
        &tasks,
    );
    let notice = model.items.iter().find(|item| item.notice.is_some()).expect("task notice");
    assert_eq!(notice.label, "Index · Running · 270/43576");
    let progress =
        notice.notice.as_ref().expect("notice metadata").progress.expect("known progress");
    assert!((progress - 270.0 / 43_576.0).abs() < f32::EPSILON);
    assert!(model.items.iter().any(|item| matches!(
        item.action,
        Some(BarAction::TaskControl { action: crate::contracts::daemon::TaskControl::Pause, .. })
    )));
    assert!(model.items.iter().any(|item| matches!(
        item.action,
        Some(BarAction::TaskControl { action: crate::contracts::daemon::TaskControl::Stop, .. })
    )));
}

#[test]
fn finished_task_no_progress() {
    let mut task =
        crate::contracts::daemon::TaskStatus::running("scan", "scan", "Scanning wallpapers");
    task.state = crate::contracts::daemon::TaskState::Completed;
    task.progress = 42;
    task.total = 42;
    let tasks = [&task];
    let model = build_bar_with_tasks(
        &Filters::default(),
        &[],
        false,
        0,
        0,
        1.0,
        false,
        false,
        false,
        &BarShow::all(),
        0.0,
        false,
        false,
        None,
        &tasks,
    );
    let notice = model.items.iter().find_map(|item| item.notice.as_ref()).expect("task notice");
    assert!(notice.progress.is_none());
}

#[test]
fn bar_wraps() {
    let filters = Filters::default();
    let wide = build_bar(
        &filters,
        &[],
        false,
        0,
        0,
        1.0,
        true,
        true,
        false,
        false,
        false,
        None,
        "",
        &BarShow::all(),
        0.0,
        false,
        false,
        None,
    );
    let narrow = build_bar(
        &filters,
        &[],
        false,
        0,
        0,
        1.0,
        true,
        true,
        false,
        false,
        false,
        None,
        "",
        &BarShow::all(),
        300.0,
        false,
        false,
        None,
    );
    assert!(wide.width > 300.0);
    assert!(narrow.width <= 300.0 + 0.01);
    assert!(narrow.height > wide.height);
    assert!(narrow.items.iter().any(|item| item.y > 0.0));
}

#[test]
fn split_icon_label() {
    let (icon, name, arrow) = super::split_icon_label("\u{f024b} anime/seasonal \u{25be}");
    assert_eq!(icon, "\u{f024b}");
    assert_eq!(name, "anime/seasonal");
    assert_eq!(arrow, "\u{25be}");
    let (i2, n2, a2) = super::split_icon_label("\u{f024b} seasonal");
    assert_eq!((i2, n2, a2), ("\u{f024b}", "seasonal", ""));
    let (i3, n3, a3) = super::split_icon_label("solo");
    assert_eq!((i3, n3, a3), ("solo", "", ""));
    let (i4, n4, a4) = super::split_icon_label("\u{f024b} seasonal \u{25b4}");
    assert_eq!((i4, n4, a4), ("\u{f024b}", "seasonal", "\u{25b4}"));
}

#[test]
fn folder_menu_up() {
    let folders = ["", "*", "anime", "cars"].map(String::from);
    let make = |menu_up| {
        build_bar(
            &Filters::default(),
            &folders,
            true,
            0,
            0,
            1.0,
            false,
            false,
            false,
            false,
            false,
            None,
            "",
            &BarShow::all(),
            9999.0,
            menu_up,
            false,
            None,
        )
    };
    let down = make(false);
    let up = make(true);
    assert_eq!(down.height, up.height);
    let rows_bottom = down.items.iter().map(|item| item.y + item.h).fold(0.0f32, f32::max);
    let menu_h = down.height - rows_bottom;
    assert!(menu_h > 0.0);
    for (down_item, up_item) in down.items.iter().zip(&up.items) {
        assert!((up_item.y - down_item.y - menu_h).abs() < 0.01);
    }
    assert!(down.items.iter().any(|item| item.label.contains(super::DROP_ARROW)));
    assert!(up.items.iter().any(|item| item.label.contains(super::DROP_ARROW_UP)));
    assert!(up.menu_up && !down.menu_up);
}

#[test]
fn random_chip_state() {
    let on = build_bar(
        &Filters::default(),
        &[],
        false,
        0,
        0,
        1.0,
        false,
        false,
        false,
        false,
        false,
        None,
        "",
        &BarShow::all(),
        0.0,
        false,
        true,
        None,
    );
    let chip = on
        .items
        .iter()
        .find(|item| matches!(item.action, Some(BarAction::Random)))
        .expect("random chip on");
    assert!(chip.active);
    let mut show = BarShow::all();
    show.random = false;
    let hidden = bar_with(&show);
    assert!(!hidden.items.iter().any(|item| matches!(item.action, Some(BarAction::Random))));
    let off = bar_with(&BarShow::all());
    let chip = off
        .items
        .iter()
        .find(|item| matches!(item.action, Some(BarAction::Random)))
        .expect("random chip off");
    assert!(!chip.active);
}

#[test]
fn folder_depth_leaf() {
    assert_eq!(super::folder_depth("anime"), 0);
    assert_eq!(super::folder_depth("anime/seasonal"), 1);
    assert_eq!(super::folder_depth("anime/seasonal/2024"), 2);
    assert_eq!(super::folder_leaf("anime/seasonal/2024"), "2024");
    assert_eq!(super::folder_leaf("anime"), "anime");
}

#[test]
fn download_shown() {
    assert!(has_download(true));
}

fn bar(filtered: usize, total: usize) -> super::BarModel {
    build_bar(
        &Filters::default(),
        &[],
        false,
        filtered,
        total,
        1.0,
        false,
        true,
        false,
        false,
        false,
        None,
        "",
        &BarShow::all(),
        0.0,
        false,
        false,
        None,
    )
}

fn bar_with(show: &BarShow) -> super::BarModel {
    build_bar(
        &Filters::default(),
        &[],
        false,
        0,
        0,
        1.0,
        false,
        true,
        false,
        false,
        false,
        None,
        "",
        show,
        0.0,
        false,
        false,
        None,
    )
}

#[test]
fn vertical_bar_columns() {
    let mut model = bar(0, 0);
    super::verticalize_bar(&mut model, 140.0, 1.0);
    assert!(model.height <= 140.0);
    assert!(model.items.windows(2).any(|pair| pair[1].x > pair[0].x));
    assert!(model.items.iter().all(|item| item.y + item.h <= model.height + 0.01));
}

#[test]
fn type_chip_gating() {
    let all = bar_with(&BarShow::all());
    let count = |model: &super::BarModel, key: &str| {
        model
            .items
            .iter()
            .filter(|item| matches!(&item.action, Some(BarAction::SetType(kind)) if kind == key))
            .count()
    };
    assert_eq!(count(&all, "video"), 1);
    assert_eq!(count(&all, "we"), 1);
    assert_eq!(count(&all, "shader"), 0);

    let mut show = BarShow::all();
    show.types.retain(|kind| *kind != "video" && *kind != "we");
    let trimmed = bar_with(&show);
    assert_eq!(count(&trimmed, "video"), 0);
    assert_eq!(count(&trimmed, "we"), 0);
    assert_eq!(count(&trimmed, "static"), 1);
    assert_eq!(count(&trimmed, ""), 1);
}

#[test]
fn sort_gating() {
    let count = |model: &super::BarModel, mode: &str| {
        model
            .items
            .iter()
            .filter(|item| matches!(&item.action, Some(BarAction::Sort(sort)) if sort == mode))
            .count()
    };
    let all = bar_with(&BarShow::all());
    assert_eq!(count(&all, "date"), 1);
    assert_eq!(count(&all, "recent"), 1);
    assert_eq!(count(&all, "applied"), 0);

    let mut show = BarShow::all();
    show.sorts.retain(|mode| *mode != "date");
    let trimmed = bar_with(&show);
    assert_eq!(count(&trimmed, "date"), 0);
    assert_eq!(count(&trimmed, "color"), 1);
}

#[test]
fn no_wallpaper_count() {
    assert!(!bar(2978, 3033).items.iter().any(|item| item.label == "2978/3033"));
    assert!(!bar(3033, 3033).items.iter().any(|item| item.label == "3033"));
}

#[test]
fn theme_mode_buttons() {
    let theme = super::ThemeBar {
        backend: String::from("skwd-iris"),
        mode: String::from("dark"),
        ..Default::default()
    };
    let model = theme_bar_model(&theme);
    let modes: Vec<_> = model
        .items
        .iter()
        .filter(|item| matches!(item.action, Some(BarAction::ThemeOpt("matugen.mode", _))))
        .collect();
    assert_eq!(modes.len(), 3);
    assert_eq!(modes.iter().filter(|item| item.active).count(), 1);
    assert!(
        modes
            .iter()
            .find(|item| matches!(item.action, Some(BarAction::ThemeOpt(_, "dark"))))
            .is_some_and(|item| item.active)
    );
}

#[test]
fn item_contains_skew() {
    let item = super::BarItem {
        x: 100.0,
        y: 0.0,
        w: 50.0,
        h: 20.0,
        skew: 10.0,
        label: String::new(),
        nerd: false,
        text_size: 11.0,
        swatch: None,
        notice: None,
        active: false,
        action: None,
        z: 0,
    };
    assert!(!super::item_contains(&item, 109.0, 0.0));
    assert!(super::item_contains(&item, 111.0, 0.0));
    assert!(super::item_contains(&item, 149.0, 0.0));
    assert!(!super::item_contains(&item, 151.0, 0.0));
    assert!(super::item_contains(&item, 101.0, 20.0));
    assert!(!super::item_contains(&item, 141.0, 20.0));
    assert!(super::item_contains(&item, 139.0, 20.0));
    assert!(!super::item_contains(&item, 125.0, -0.1));
    assert!(!super::item_contains(&item, 125.0, 20.1));
}

#[test]
fn theme_sub_bar() {
    let theme = super::ThemeBar {
        backend: String::from("matugen"),
        scheme: String::from("scheme-content"),
        color_index: 2,
        ..Default::default()
    };
    let base = bar_with(&BarShow::all());
    let with_theme = build_bar(
        &Filters::default(),
        &[],
        false,
        0,
        0,
        1.0,
        false,
        true,
        false,
        false,
        false,
        None,
        "",
        &BarShow::all(),
        1600.0,
        false,
        false,
        Some(&theme),
    );
    assert!(with_theme.height > base.height);
    let toggle = with_theme
        .items
        .iter()
        .find(|item| matches!(item.action, Some(BarAction::ThemeBackendToggle)))
        .expect("backend chip");
    let first_row_bottom = with_theme
        .items
        .iter()
        .filter(|item| item.y == 0.0)
        .map(|item| item.h)
        .fold(0.0f32, f32::max);
    assert!(toggle.y >= first_row_bottom);
    let schemes: Vec<_> = with_theme
        .items
        .iter()
        .filter(|item| matches!(item.action, Some(BarAction::ThemeOpt("matugen.schemeType", _))))
        .collect();
    assert_eq!(schemes.len(), 8);
    assert_eq!(schemes.iter().filter(|item| item.active).count(), 1);
    let sources: Vec<_> = with_theme
        .items
        .iter()
        .filter(|item| matches!(item.action, Some(BarAction::ThemeOpt("matugen.colorIndex", _))))
        .collect();
    assert_eq!(sources.len(), 4);
    assert!(
        sources.iter().find(|item| item.active).is_some_and(|item| item.label == "3"),
        "index 2 = chip 3"
    );
    assert!(with_theme.swatch_at.is_some());
    for item in &with_theme.items {
        assert!(item.x + item.w <= 1600.0 + 0.01);
    }
    let off = super::ThemeBar { backend: String::from("off"), ..Default::default() };
    let plain = theme_bar_model(&off);
    assert!(!plain.items.iter().any(|item| matches!(item.action, Some(BarAction::ThemeOpt(_, _)))));

    let iris = super::ThemeBar {
        backend: String::from("skwd-iris"),
        style: String::from("pastel"),
        iris_scheme: String::from("tonal-spot"),
        ..Default::default()
    };
    let iris_bar = theme_bar_model(&iris);
    let opt_count = |bar: &super::BarModel, key: &str| {
        bar.items
            .iter()
            .filter(|item| matches!(item.action, Some(BarAction::ThemeOpt(k, _)) if k == key))
            .count()
    };
    assert_eq!(opt_count(&iris_bar, "theme.style"), 4);
    assert_eq!(opt_count(&iris_bar, "theme.scheme"), 9);
    assert_eq!(
        iris_bar
            .items
            .iter()
            .filter(|item| {
                matches!(item.action, Some(BarAction::ThemeOpt("theme.style", _)) if item.active)
            })
            .count(),
        1
    );

    let wallust = super::ThemeBar { backend: String::from("skwd-wallust"), ..Default::default() };
    let wallust_bar = theme_bar_model(&wallust);
    assert_eq!(opt_count(&wallust_bar, "theme.style"), 4);
    assert_eq!(opt_count(&wallust_bar, "theme.scheme"), 0);
}

fn theme_bar_model(theme: &super::ThemeBar) -> super::BarModel {
    build_bar(
        &Filters::default(),
        &[],
        false,
        0,
        0,
        1.0,
        false,
        true,
        false,
        false,
        false,
        None,
        "",
        &BarShow::all(),
        1600.0,
        false,
        false,
        Some(theme),
    )
}

#[test]
fn backend_option_chips() {
    let opts = |model: &super::BarModel, key: &str| {
        model
            .items
            .iter()
            .filter(|item| matches!(item.action, Some(BarAction::ThemeOpt(k, _)) if k == key))
            .count()
    };
    let actives = |model: &super::BarModel, key: &str| {
        model
            .items
            .iter()
            .filter(|item| {
                item.active && matches!(item.action, Some(BarAction::ThemeOpt(k, _)) if k == key)
            })
            .count()
    };
    let wallust = theme_bar_model(&super::ThemeBar {
        backend: String::from("wallust"),
        wallust_palette: String::from("softdark"),
        wallust_colorspace: String::from("lab"),
        ..Default::default()
    });
    assert_eq!(opts(&wallust, "theme.wallustPalette"), 6);
    assert_eq!(opts(&wallust, "theme.wallustColorspace"), 4);
    assert_eq!(actives(&wallust, "theme.wallustPalette"), 1);
    let pywal = theme_bar_model(&super::ThemeBar {
        backend: String::from("pywal"),
        pywal_saturate: String::new(),
        ..Default::default()
    });
    assert_eq!(opts(&pywal, "theme.pywalSaturate"), 5);
    assert_eq!(actives(&pywal, "theme.pywalSaturate"), 1);
    let noctalia = theme_bar_model(&super::ThemeBar {
        backend: String::from("noctalia"),
        noctalia_scheme: String::from("vibrant"),
        noctalia_pure_black: true,
        ..Default::default()
    });
    assert_eq!(opts(&noctalia, "theme.noctaliaScheme"), 10);
    assert_eq!(actives(&noctalia, "theme.noctaliaScheme"), 1);
    assert_eq!(actives(&noctalia, "theme.noctaliaPureBlack"), 1);
    let dms = theme_bar_model(&super::ThemeBar {
        backend: String::from("dms"),
        scheme: String::from("scheme-tonal-spot"),
        ..Default::default()
    });
    assert_eq!(opts(&dms, "matugen.schemeType"), 8);
    assert_eq!(opts(&dms, "matugen.colorIndex"), 4);
}

#[test]
fn backend_menu_detected() {
    let all = crate::frontend::ui::backend_menu_options(None, "matugen");
    assert_eq!(all.len(), super::THEME_BACKENDS.len());
    let detected = vec![String::from("matugen"), String::from("noctalia")];
    let opts = crate::frontend::ui::backend_menu_options(Some(&detected), "matugen");
    let keys: Vec<&str> = opts.iter().map(|(key, _)| *key).collect();
    assert_eq!(keys, vec!["native", "static", "matugen", "noctalia", "off"]);
    let stranded = crate::frontend::ui::backend_menu_options(Some(&detected), "wallust");
    assert!(stranded.iter().any(|(key, _)| *key == "wallust"), "stranded backend stays");
}

#[test]
fn touchpad_audio_accumulates() {
    use iced::widget::canvas::Program;
    use iced::{Event, Point, Rectangle, Size, mouse};
    let _wheel = crate::frontend::components::wheel_test_guard();
    let model = build_bar(
        &Filters::default(),
        &[],
        false,
        0,
        0,
        1.0,
        false,
        true,
        false,
        false,
        false,
        None,
        "",
        &BarShow::all(),
        0.0,
        false,
        false,
        None,
    );
    let audio = model
        .items
        .iter()
        .find(|item| matches!(item.action, Some(BarAction::Audio)))
        .expect("audio chip");
    let cursor =
        mouse::Cursor::Available(Point::new(audio.x + audio.w / 2.0, audio.y + audio.h / 2.0));
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(model.width, model.height));
    let pal = crate::frontend::theme::Palette::default();
    let cache = iced::widget::canvas::Cache::new();
    let bar = super::FilterBar {
        model,
        hover: None,
        menu_open: false,
        backend_menu_open: false,
        backend: String::new(),
        backend_options: Vec::new(),
        menu_hover: None,
        folder_options: &[],
        selected_folder: "",
        menu_scroll: 0.0,
        theme_swatch: &[],
        pal: &pal,
        cache: &cache,
        scale: 1.0,
        fade: 1.0,
        visual_style: super::BarVisualStyle::Slices,
    };
    let pixels = |y: f32| {
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y },
        })
    };
    let mut state = super::BarState::default();
    let mut published = 0;
    for _ in 0..39 {
        let act = bar.update(&mut state, &pixels(1.0), bounds, cursor).expect("captured");
        let (msg, _, status) = act.into_inner();
        assert!(matches!(status, iced::event::Status::Captured));
        if msg.is_some() {
            published += 1;
        }
    }
    assert_eq!(published, 0, "39px under one line");
    let act = bar.update(&mut state, &pixels(2.0), bounds, cursor).expect("captured");
    let (msg, _, _) = act.into_inner();
    assert!(matches!(msg, Some(super::BarIntent::StepVolume(5))));
    state = super::BarState::default();
    let act = bar.update(&mut state, &pixels(-120.0), bounds, cursor).expect("captured");
    let (msg, _, _) = act.into_inner();
    assert!(matches!(msg, Some(super::BarIntent::StepVolume(-15))));
    let wheel = Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
    });
    let mut fresh = super::BarState::default();
    let act = bar.update(&mut fresh, &wheel, bounds, cursor).expect("captured");
    let (msg, _, _) = act.into_inner();
    assert!(matches!(msg, Some(super::BarIntent::StepVolume(5))));
}

#[test]
fn menu_index_upward() {
    let folders: Vec<String> = (0..16).map(|i| format!("folder{i}")).collect();
    let model = build_bar(
        &Filters::default(),
        &folders,
        true,
        0,
        0,
        1.0,
        false,
        false,
        false,
        false,
        false,
        None,
        "",
        &BarShow::all(),
        9999.0,
        true,
        false,
        None,
    );
    let pal = crate::frontend::theme::Palette::default();
    let cache = iced::widget::canvas::Cache::new();
    let bar = super::FilterBar {
        model,
        hover: None,
        menu_open: true,
        backend_menu_open: false,
        backend: String::new(),
        backend_options: Vec::new(),
        menu_hover: None,
        folder_options: &folders,
        selected_folder: "*",
        menu_scroll: 0.0,
        theme_swatch: &[],
        pal: &pal,
        cache: &cache,
        scale: 1.0,
        fade: 1.0,
        visual_style: super::BarVisualStyle::Slices,
    };
    let rect = bar.menu_rect().expect("menu open");
    assert!(rect.y.abs() < 0.01);
    let below = rect.y + rect.height + 20.0;
    assert!(bar.menu_index_at(rect.x + 5.0, below).is_none());
    assert!(bar.menu_index_at(rect.x + 5.0, rect.y + rect.height * 0.5).is_some());
}

#[test]
fn audio_residue_resets() {
    use iced::widget::canvas::Program;
    use iced::{Event, Point, Rectangle, Size, mouse};
    let _wheel = crate::frontend::components::wheel_test_guard();
    let model = build_bar(
        &Filters::default(),
        &[],
        false,
        0,
        0,
        1.0,
        false,
        true,
        false,
        false,
        false,
        None,
        "",
        &BarShow::all(),
        0.0,
        false,
        false,
        None,
    );
    let center = |want: BarAction| {
        let chip = model
            .items
            .iter()
            .find(|item| {
                item.action
                    .as_ref()
                    .is_some_and(|act| std::mem::discriminant(act) == std::mem::discriminant(&want))
            })
            .expect("chip present");
        Point::new(chip.x + chip.w / 2.0, chip.y + chip.h / 2.0)
    };
    let audio = mouse::Cursor::Available(center(BarAction::Audio));
    let settings = mouse::Cursor::Available(center(BarAction::Settings));
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(model.width, model.height));
    let pal = crate::frontend::theme::Palette::default();
    let cache = iced::widget::canvas::Cache::new();
    let bar = super::FilterBar {
        model,
        hover: None,
        menu_open: false,
        backend_menu_open: false,
        backend: String::new(),
        backend_options: Vec::new(),
        menu_hover: None,
        folder_options: &[],
        selected_folder: "",
        menu_scroll: 0.0,
        theme_swatch: &[],
        pal: &pal,
        cache: &cache,
        scale: 1.0,
        fade: 1.0,
        visual_style: super::BarVisualStyle::Slices,
    };
    let px = |y: f32| {
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y },
        })
    };
    let mut state = super::BarState::default();
    for _ in 0..39 {
        let _ = bar.update(&mut state, &px(1.0), bounds, audio);
    }
    let _ = bar.update(&mut state, &px(1.0), bounds, settings);
    let (msg, _, _) =
        bar.update(&mut state, &px(1.0), bounds, audio).expect("captured").into_inner();
    assert!(msg.is_none());
}

#[test]
fn theme_backends_canonical() {
    let ui: Vec<&str> = super::THEME_BACKENDS.iter().map(|(id, _)| *id).collect();
    let mut canonical: Vec<&str> = wall_proto::THEME_BACKENDS.to_vec();
    let mut shown = ui.clone();
    canonical.sort_unstable();
    shown.sort_unstable();
    assert_eq!(shown, canonical);
    assert!(super::THEME_BACKENDS.iter().all(|(_, label)| !label.is_empty()));
}

#[test]
fn control_colour_pairs() {
    use iced::Color;

    let palette = crate::frontend::theme::Palette {
        primary: Color::from_rgb8(1, 2, 3),
        primary_text: Color::from_rgb8(4, 5, 6),
        surface_text: Color::from_rgb8(7, 8, 9),
        surface_variant: Color::from_rgb8(10, 11, 12),
        surface_container: Color::from_rgb8(13, 14, 15),
        outline: Color::from_rgb8(16, 17, 18),
        ..crate::frontend::theme::Palette::default()
    };

    assert_eq!(super::view::control_text(&palette, true), palette.primary_text);
    assert_eq!(super::view::control_text(&palette, false), palette.surface_text);
    assert_eq!(super::view::control_background(&palette, true, false), palette.primary);
    let resting = super::view::control_background(&palette, false, false);
    let hovered = super::view::control_background(&palette, false, true);
    let border = super::view::control_border(&palette, false, false);
    assert_eq!(
        (resting.r, resting.g, resting.b),
        (palette.surface_container.r, palette.surface_container.g, palette.surface_container.b,)
    );
    assert_eq!(
        (hovered.r, hovered.g, hovered.b),
        (palette.surface_variant.r, palette.surface_variant.g, palette.surface_variant.b,)
    );
    assert_eq!(
        (border.r, border.g, border.b),
        (palette.outline.r, palette.outline.g, palette.outline.b)
    );
}

#[test]
fn orient_cycles_all_wide_tall() {
    assert_eq!(next_orient(""), "landscape");
    assert_eq!(next_orient("landscape"), "portrait");
    assert_eq!(next_orient("portrait"), "");
    assert_eq!(next_orient("bogus"), "landscape");
    assert_eq!(orient_label(""), "SHAPE");
    assert_eq!(orient_label("landscape"), "WIDE");
    assert_eq!(orient_label("portrait"), "TALL");
}

#[test]
fn visual_style_keys() {
    assert_eq!(super::BarVisualStyle::from_key("slices"), super::BarVisualStyle::Slices);
    assert_eq!(super::BarVisualStyle::from_key("hex"), super::BarVisualStyle::Hex);
    assert_eq!(super::BarVisualStyle::from_key("wall"), super::BarVisualStyle::Wall);
    assert_eq!(super::BarVisualStyle::from_key("sandy"), super::BarVisualStyle::Slices);
    assert_eq!(super::BarVisualStyle::from_key("unknown"), super::BarVisualStyle::Slices);
}

#[test]
fn resolution_chip_cycle() {
    let presets = vec![
        ResolutionPreset {
            label: "FHD".into(),
            orientation: "wide".into(),
            from_width: 1920,
            from_height: 1080,
            to_width: Some(2559),
            to_height: Some(1439),
        },
        ResolutionPreset {
            label: "4K".into(),
            orientation: "wide".into(),
            from_width: 3840,
            from_height: 2160,
            to_width: None,
            to_height: None,
        },
        ResolutionPreset {
            label: "FHD TALL".into(),
            orientation: "tall".into(),
            from_width: 1080,
            from_height: 1920,
            to_width: Some(1439),
            to_height: Some(2559),
        },
    ];
    assert_eq!(next_resolution("", &presets, ""), "wide:1920x1080..2559x1439");
    assert_eq!(
        next_resolution("wide:1920x1080..2559x1439", &presets, "landscape"),
        "wide:3840x2160.."
    );
    assert_eq!(next_resolution("wide:3840x2160..", &presets, "landscape"), "");
    assert_eq!(next_resolution("", &presets, "portrait"), "tall:1080x1920..1439x2559");
    assert_eq!(next_resolution("tall:1080x1920..1439x2559", &presets, "portrait"), "");
    assert_eq!(resolution_label("", &presets), "SIZE");
    assert_eq!(resolution_label("wide:3840x2160..", &presets), "4K");

    let mut show = BarShow::all();
    show.resolution_presets = presets;
    let model = bar_with(&show);
    let chip = model
        .items
        .iter()
        .find(|item| matches!(item.action, Some(BarAction::Resolution(_))))
        .expect("resolution chip");
    assert_eq!(chip.label, "SIZE");
    assert!(matches!(
        chip.action,
        Some(BarAction::Resolution(ref value)) if value == "wide:1920x1080..2559x1439"
    ));
}

#[test]
fn dropdown_chip_glyph_budget() {
    use crate::frontend::ui::theme_bar::ThemeBar as Bar;

    let scale = 1.0;
    let glyph = super::super::misc::glyph_width(10.0 * scale);
    let folders = ["", "*", "anime"].map(String::from);
    let filters = Filters { folder: String::from("anime"), ..Default::default() };
    let model = build_bar(
        &filters,
        &folders,
        false,
        0,
        0,
        scale,
        false,
        false,
        false,
        false,
        false,
        None,
        "",
        &BarShow::all(),
        9999.0,
        false,
        false,
        Some(&Bar::default()),
    );
    let expected = |current: &str| {
        super::super::misc::text_width(current, 10.0 * scale, false) + 2.0 * glyph + 44.0 * scale
    };
    let folder = model
        .items
        .iter()
        .find(|item| item.label.contains("ANIME") && item.nerd)
        .expect("folder dropdown");
    assert!((folder.w - expected("ANIME")).abs() < 0.01);
    let backend = model
        .items
        .iter()
        .find(|item| item.label.starts_with('\u{f03d8}'))
        .expect("backend dropdown");
    assert!(backend.w >= super::super::misc::text_width("skwd", 10.0, false) + 2.0 * glyph);
    let (icon, name, arrow) = super::split_icon_label(&folder.label);
    assert_eq!((icon, name, arrow), ("\u{f024b}", "ANIME", super::DROP_ARROW));
    assert!((folder.w - super::super::misc::text_width(name, 10.0, false) - 44.0) / glyph == 2.0);
}

#[test]
fn skewed_bar_canvas_contains_every_item_tip() {
    for scale in [0.5, 1.0, 1.65, 2.0] {
        for max_width in [0.0, 320.0 * scale, 1600.0 * scale] {
            let model = build_bar_with_tasks(
                &Filters::default(),
                &[],
                false,
                0,
                0,
                scale,
                true,
                true,
                false,
                &BarShow::all(),
                max_width,
                false,
                false,
                None,
                &[],
            );
            for item in &model.items {
                assert!(
                    item.x + item.w <= model.width + 0.01,
                    "item {} ends at {}, outside {} at scale {scale}, max width {max_width}",
                    item.label,
                    item.x + item.w,
                    model.width
                );
            }
            if max_width == 0.0 {
                let last = model.items.last().unwrap();
                assert!(last.skew > 0.0);
                assert!((model.width - last.x - last.w).abs() < 0.01);
            }
        }
    }
}
