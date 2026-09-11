use crate::contracts::media::MediaKind;
use crate::contracts::settings::wallpaper_kind::{STATIC, VIDEO, WE};
use crate::contracts::settings::{SettingsSource, keys};
use crate::i18n::tr;

use super::launch_tab::tab_launch;
use super::media_tabs::{
    tab_paper, tab_search, tab_sources, tab_steam, tab_wallhaven, tab_wallpaper_engine,
};
use super::motion_tab::tab_motion;
use super::position_tab::tab_position;
use super::selector_tab::{tab_filter, tab_selector};
use super::system_tabs::{
    tab_general, tab_keybinds, tab_language, tab_paths, tab_performance, tab_postprocessing,
    tab_schedule,
};
use super::theme_tabs::{tab_integrations, tab_matugen, tab_niri, tab_theme};
use super::transition_tab::tab_transitions;
use super::{Builder, Card, Control, Row};

pub(super) fn type_chip_label(kind: &str) -> &'static str {
    match kind {
        STATIC => tr("settings-filter-type-images"),
        VIDEO => tr("settings-filter-type-videos"),
        WE => tr("settings-filter-type-we"),
        _ => tr("settings-filter-type-all"),
    }
}

pub(super) fn folder_dropdown_options(folders: &[String]) -> Vec<(String, String)> {
    let mut out = vec![
        (String::from("*"), tr("settings-filter-all-folders").to_string()),
        (String::new(), tr("settings-filter-main-folder").to_string()),
    ];
    for folder in folders {
        if matches!(folder.as_str(), "" | "*") {
            continue;
        }
        out.push((folder.clone(), folder.clone()));
    }
    out
}

pub fn build_tab(
    tab: &str,
    cfg: &dyn SettingsSource,
    themes: &[String],
    folders: &[String],
    analysis: &str,
    backends: &[String],
) -> Vec<(Card, Vec<Row>)> {
    build_tab_with_outputs(tab, cfg, themes, folders, analysis, backends, &[])
}

pub(crate) fn build_tab_with_outputs(
    tab: &str,
    cfg: &dyn SettingsSource,
    themes: &[String],
    folders: &[String],
    analysis: &str,
    backends: &[String],
    outputs: &[String],
) -> Vec<(Card, Vec<Row>)> {
    build_tab_with_output_statuses(
        tab,
        cfg,
        themes,
        folders,
        analysis,
        backends,
        outputs,
        &[],
        &std::collections::HashMap::new(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_tab_with_output_statuses(
    tab: &str,
    cfg: &dyn SettingsSource,
    themes: &[String],
    folders: &[String],
    analysis: &str,
    backends: &[String],
    outputs: &[String],
    statuses: &[crate::contracts::daemon::OutputStatus],
    previews: &std::collections::HashMap<String, String>,
) -> Vec<(Card, Vec<Row>)> {
    build_tab_with_runtime_status(
        tab, cfg, themes, folders, analysis, backends, outputs, statuses, previews, None, None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_tab_with_runtime_status(
    tab: &str,
    cfg: &dyn SettingsSource,
    themes: &[String],
    folders: &[String],
    analysis: &str,
    backends: &[String],
    outputs: &[String],
    statuses: &[crate::contracts::daemon::OutputStatus],
    previews: &std::collections::HashMap<String, String>,
    library_watch: Option<&crate::contracts::daemon::LibraryWatchStatus>,
    playback: Option<&crate::contracts::daemon::PlaybackStatus>,
) -> Vec<(Card, Vec<Row>)> {
    match tab {
        "picker" => compose_picker(cfg, themes, outputs),
        "filter" => compose_filter_search(cfg, folders, analysis),
        "position" => compose_position(cfg),
        "displays" => compose_displays(cfg, statuses, previews),
        "motion" => compose_motion(cfg),
        "playback" => compose_playback(cfg, themes, outputs, playback),
        "performance" => compose_performance(cfg),
        "library" => compose_library(cfg, themes, outputs, library_watch),
        "sources" => compose_sources(cfg, themes, outputs),
        "automation" => compose_automation(cfg, themes, folders, outputs),
        "theme" => compose_theme(cfg, backends),
        "integrations" => compose_integrations(cfg, themes),
        "language" => collect(cfg, tab_language),
        _ => Vec::new(),
    }
}

fn compose_displays(
    cfg: &dyn SettingsSource,
    statuses: &[crate::contracts::daemon::OutputStatus],
    previews: &std::collections::HashMap<String, String>,
) -> Vec<(Card, Vec<Row>)> {
    let mut rows = Vec::new();
    for output in statuses {
        let current = if output.kind == MediaKind::WallpaperEngine {
            output.we_id.as_str()
        } else if !output.current.is_empty() {
            output.current.as_str()
        } else {
            output.path.as_str()
        };
        let current = std::path::Path::new(current)
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or(current);
        let (width, height) = output.logical_size();
        let metadata = if output.is_connected() {
            crate::i18n::tr_args!(
                "settings-displays-monitor-desc",
                width => width,
                height => height,
                kind => output.kind.as_key()
            )
        } else {
            crate::i18n::tr_args!(
                "settings-displays-monitor-offline-desc",
                width => width,
                height => height
            )
        };
        let fill_path = format!("{}.{name}", keys::display::FILL_MODES, name = output.name);
        let fill = if output.fill.is_empty() {
            cfg.text(keys::display::FILL_MODE)
        } else {
            output.fill.clone()
        };
        let fill_modes = super::super::tables::FILL_MODES.map(|key| {
            let label = match key {
                "fit" => "settings-paper-fill-fit",
                "stretch" => "settings-paper-fill-stretch",
                "center" => "settings-paper-fill-center",
                "tile" => "settings-paper-fill-tile",
                "span" => "settings-paper-fill-span",
                _ => "settings-paper-fill-fill",
            };
            (key.to_string(), tr(label).to_string())
        });
        let placement = Row {
            title: tr("settings-displays-placement-label").to_string(),
            desc: tr("settings-displays-placement-desc").to_string(),
            control: Control::Chips {
                path: fill_path,
                options: fill_modes.into(),
                current: fill,
                disabled: Vec::new(),
            },
        };
        let lock_path = format!("{}.{name}", keys::display::OUTPUT_LOCKS, name = output.name);
        let lock = Row {
            title: tr("settings-displays-lock-label").to_string(),
            desc: tr("settings-displays-lock-desc").to_string(),
            control: Control::Toggle { value: cfg.flag(&lock_path), path: lock_path },
        };
        rows.push(Row {
            title: output.name.clone(),
            desc: metadata,
            control: Control::StackBar {
                id: format!("display:{}", output.target()),
                summary: current.to_string(),
                preview: previews.get(&output.name).cloned(),
                rows: vec![placement, lock],
            },
        });
    }
    if rows.is_empty() {
        rows.push(Row {
            title: tr("settings-displays-empty-label").to_string(),
            desc: tr("settings-displays-empty-desc").to_string(),
            control: Control::Static,
        });
    }
    vec![(
        Card { title: tr("settings-displays-card"), subtitle: tr("settings-displays-card-desc") },
        rows,
    )]
}

fn collect(
    cfg: &dyn SettingsSource,
    build: impl FnOnce(&mut Builder<'_>),
) -> Vec<(Card, Vec<Row>)> {
    let mut builder = Builder { cfg, cards: Vec::new() };
    build(&mut builder);
    builder.cards
}

fn take_card(cards: &mut Vec<(Card, Vec<Row>)>, title: &str) -> Vec<Row> {
    cards
        .iter()
        .position(|(card, _)| card.title == title)
        .map_or_else(Vec::new, |index| cards.remove(index).1)
}

fn take_rows(rows: &mut Vec<Row>, titles: &[&str]) -> Vec<Row> {
    let (selected, remaining): (Vec<_>, Vec<_>) =
        std::mem::take(rows).into_iter().partition(|row| titles.contains(&row.title.as_str()));
    *rows = remaining;
    selected
}

fn section(
    cards: &mut Vec<(Card, Vec<Row>)>,
    title: &'static str,
    subtitle: &'static str,
    rows: Vec<Row>,
) {
    if !rows.is_empty() {
        cards.push((Card { title, subtitle }, rows));
    }
}

fn prefixed(mut rows: Vec<Row>, prefix: &str) -> Vec<Row> {
    for row in &mut rows {
        row.title = format!("{prefix} {}", row.title);
    }
    rows
}

fn details(title: &str, desc: &str, id: &str, summary: &str, rows: Vec<Row>) -> Row {
    Row {
        title: title.to_string(),
        desc: desc.to_string(),
        control: Control::Details { id: id.to_string(), summary: summary.to_string(), rows },
    }
}

fn enabled_status(enabled: bool) -> &'static str {
    tr(if enabled { "settings-control-enabled" } else { "settings-control-disabled" })
}

fn compose_picker(
    cfg: &dyn SettingsSource,
    themes: &[String],
    outputs: &[String],
) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut general = collect(cfg, |builder| tab_general(builder, themes, outputs));
    let mut selector = collect(cfg, tab_selector);
    let mut keys = collect(cfg, tab_keybinds);

    let mut general_rows = take_card(&mut general, tr("settings-general-general-card"));
    let mut layout = take_card(&mut selector, tr("settings-selector-layout-card"));
    general_rows.extend(take_rows(
        &mut take_card(&mut general, tr("settings-general-behaviour-card")),
        &[tr("settings-general-close-on-selection-label")],
    ));
    section(&mut out, tr("settings-section-general"), "", general_rows);

    let _filter_motion = take_rows(&mut layout, &[tr("settings-selector-filter-motion-label")]);
    layout.extend(take_card(&mut selector, tr("settings-selector-presets-card")));
    for title in [
        tr("settings-selector-composition-card"),
        tr("settings-selector-cells-card"),
        tr("settings-selector-cards-card"),
        tr("settings-selector-performance-card"),
        tr("settings-selector-sandy-card"),
        tr("settings-selector-slice-size-card"),
        tr("settings-selector-corners-card"),
    ] {
        let rows = take_card(&mut selector, title);
        if rows.is_empty() {
            continue;
        }
        layout.extend(prefixed(rows, &format!("{title} ·")));
    }
    section(&mut out, tr("settings-section-layout-presets"), "", layout);

    let mut previews = take_card(&mut selector, tr("settings-selector-live-preview-card"));
    previews.extend(take_card(&mut selector, tr("settings-selector-video-preview-card")));
    section(&mut out, tr("settings-section-cards-previews"), "", previews);

    let tag_panel = take_card(&mut selector, tr("settings-selector-tag-cloud-card"));
    section(
        &mut out,
        tr("settings-section-tag-panel"),
        tr("settings-selector-tag-cloud-card-desc"),
        tag_panel,
    );

    let mut controls = Vec::new();
    for title in [
        tr("settings-keybinds-custom-card"),
        tr("settings-keybinds-fixed-card"),
        tr("settings-keybinds-reset-card"),
    ] {
        controls.extend(take_card(&mut keys, title));
    }
    section(
        &mut out,
        tr("settings-section-controls"),
        tr("settings-keybinds-custom-card-desc"),
        controls,
    );
    out
}

fn compose_filter_search(
    cfg: &dyn SettingsSource,
    folders: &[String],
    analysis: &str,
) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut filter = collect(cfg, |builder| tab_filter(builder, folders));

    section(
        &mut out,
        tr("settings-section-appearance"),
        tr("settings-filter-appearance-card-desc"),
        take_card(&mut filter, tr("settings-filter-appearance-card")),
    );
    let mut buttons = take_card(&mut filter, tr("settings-filter-type-chips-card"));
    buttons.extend(take_card(&mut filter, tr("settings-filter-sort-options-card")));
    buttons.extend(take_card(&mut filter, tr("settings-filter-other-controls-card")));
    section(
        &mut out,
        tr("settings-section-buttons"),
        tr("settings-filter-type-chips-card-desc"),
        buttons,
    );
    section(
        &mut out,
        tr("settings-section-resolution-presets"),
        tr("settings-filter-resolution-presets-card-desc"),
        take_card(&mut filter, tr("settings-filter-resolution-presets-card")),
    );
    section(
        &mut out,
        tr("settings-section-defaults"),
        tr("settings-filter-default-folder-card-desc"),
        take_card(&mut filter, tr("settings-filter-default-folder-card")),
    );

    section(
        &mut out,
        tr("settings-section-visibility"),
        tr("settings-filter-visibility-card-desc"),
        take_card(&mut filter, tr("settings-filter-visibility-card")),
    );
    let mut search = collect(cfg, tab_search);
    let search_summary = match cfg.text(keys::tagging::DEFAULT_SEARCH_MODE).as_str() {
        "describe" => tr("settings-tagging-mode-describe"),
        _ => tr("settings-tagging-mode-tags"),
    };
    let rows = vec![details(
        tr("settings-tagging-search-card"),
        tr("settings-tagging-search-card-desc"),
        "search.discovery",
        search_summary,
        take_card(&mut search, tr("settings-tagging-search-card")),
    )];
    section(
        &mut out,
        tr("settings-section-search-tagging"),
        tr("settings-tagging-overview-desc"),
        rows,
    );
    let mut model_rows = take_card(&mut search, tr("settings-tagging-models-card"));
    if !analysis.trim().is_empty() {
        model_rows.insert(
            0,
            Row {
                title: tr("settings-tagging-model-import-status-label").to_string(),
                desc: analysis.to_string(),
                control: Control::Static,
            },
        );
    }
    section(
        &mut out,
        tr("settings-section-semantic-models"),
        tr("settings-tagging-models-card-desc"),
        model_rows,
    );
    out
}

fn compose_position(cfg: &dyn SettingsSource) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut position = collect(cfg, tab_position);
    for (title, subtitle) in [
        (tr("settings-position-slices-card"), tr("settings-position-picker-card-desc")),
        (tr("settings-position-hex-card"), tr("settings-position-picker-card-desc")),
        (tr("settings-position-wall-card"), tr("settings-position-picker-card-desc")),
        (tr("settings-position-sandy-card"), tr("settings-position-picker-card-desc")),
        (tr("settings-filter-position-card"), tr("settings-filter-position-card-desc")),
        (
            tr("settings-selector-tag-cloud-position-card"),
            tr("settings-selector-tag-cloud-position-card-desc"),
        ),
    ] {
        section(&mut out, title, subtitle, take_card(&mut position, title));
    }
    out
}

fn compose_motion(cfg: &dyn SettingsSource) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut motion = collect(cfg, tab_motion);
    let mut launch = collect(cfg, tab_launch);
    let mut selector = collect(cfg, tab_selector);
    let mut transitions = collect(cfg, tab_transitions);
    let mut paper = collect(cfg, tab_paper);

    section(
        &mut out,
        tr("settings-section-shared-speeds"),
        tr("settings-motion-motion-card-desc"),
        take_card(&mut motion, tr("settings-motion-motion-card")),
    );
    section(
        &mut out,
        tr("settings-section-launch"),
        tr("settings-launch-launch-card-desc"),
        take_card(&mut launch, tr("settings-launch-launch-card")),
    );
    let mut picker_motion = take_rows(
        &mut take_card(&mut selector, tr("settings-selector-layout-card")),
        &[tr("settings-selector-filter-motion-label")],
    );
    picker_motion.extend(take_card(&mut selector, tr("settings-selector-card-flip-card")));
    section(
        &mut out,
        tr("settings-section-picker-motion"),
        tr("settings-selector-card-flip-card-desc"),
        picker_motion,
    );

    let mut wallpaper_transitions =
        take_card(&mut transitions, tr("settings-transitions-transitions-card"));
    wallpaper_transitions
        .extend(take_card(&mut transitions, tr("settings-transitions-preview-card")));
    section(
        &mut out,
        tr("settings-section-wallpaper-transitions"),
        tr("settings-transitions-transitions-card-desc"),
        wallpaper_transitions,
    );
    section(
        &mut out,
        tr("settings-section-transition-performance"),
        tr("settings-transitions-sand-card-desc"),
        take_card(&mut transitions, tr("settings-transitions-sand-card")),
    );
    section(
        &mut out,
        tr("settings-section-awww-transition"),
        "",
        take_card(&mut paper, tr("settings-paper-awww-card")),
    );

    let mut sandy = take_card(&mut selector, tr("settings-selector-sand-card"));
    sandy.extend(take_card(&mut selector, tr("settings-selector-ring-card")));
    section(
        &mut out,
        tr("settings-section-sandy-motion"),
        tr("settings-selector-sand-card-desc"),
        sandy,
    );
    out
}

fn compose_playback(
    cfg: &dyn SettingsSource,
    themes: &[String],
    outputs: &[String],
    playback: Option<&crate::contracts::daemon::PlaybackStatus>,
) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut paper = collect(cfg, tab_paper);
    let mut general = collect(cfg, |builder| tab_general(builder, themes, outputs));
    let mut workshop = collect(cfg, tab_wallpaper_engine);

    let mut display = take_card(&mut paper, tr("settings-paper-engine-card"));
    if let Some(row) = display.iter_mut().find(|row| row.title == tr("settings-paper-engine-label"))
    {
        row.title = tr("settings-paper-static-backend-label").to_string();
    }
    section(
        &mut out,
        tr("settings-section-display"),
        tr("settings-paper-engine-card-desc"),
        display,
    );
    section(
        &mut out,
        tr("settings-section-video"),
        tr("settings-paper-video-engine-card-desc"),
        take_card(&mut paper, tr("settings-paper-video-engine-card")),
    );
    section(
        &mut out,
        tr("settings-section-audio"),
        tr("settings-general-audio-card-desc"),
        take_card(&mut general, tr("settings-general-audio-card")),
    );

    if cfg.flag_default_true(keys::features::STEAM) {
        let mut rendering =
            take_card(&mut workshop, tr("settings-wallpaper-engine-rendering-card"));
        rendering.extend(take_card(&mut workshop, tr("settings-wallpaper-engine-effects-card")));
        section(
            &mut out,
            tr("settings-section-workshop-rendering"),
            tr("settings-wallpaper-engine-rendering-card-desc"),
            rendering,
        );
    }
    let mut pause = take_card(&mut paper, tr("settings-playback-pause-title"));
    if let Some(status) = playback {
        let detection = if !(cfg.flag(keys::playback::FULLSCREEN)
            || cfg.flag(keys::playback::MAXIMIZED)
            || cfg.is_niri() && cfg.flag(keys::niri::FULL_WIDTH_PAUSE))
        {
            tr("settings-playback-detection-off")
        } else if (!cfg.flag(keys::playback::FULLSCREEN) || status.fullscreen_supported)
            && (!cfg.flag(keys::playback::MAXIMIZED) || status.maximized_supported)
            && (!(cfg.is_niri() && cfg.flag(keys::niri::FULL_WIDTH_PAUSE))
                || status.full_width_supported)
        {
            tr("settings-playback-detection-ready")
        } else {
            tr("settings-playback-detection-unavailable")
        };
        pause.push(Row {
            title: detection.to_string(),
            desc: String::new(),
            control: Control::Static,
        });
        let reason = if !status.processes.is_empty() {
            crate::i18n::tr_args!("settings-playback-paused-process", names => status.processes.join(", "))
        } else if status.overview_paused {
            tr("settings-playback-paused-overview").to_string()
        } else if status.resume_pending {
            tr("settings-playback-resuming").to_string()
        } else if status.full_width_paused {
            tr("settings-playback-paused-full-width").to_string()
        } else if status.maximized_paused {
            tr("settings-playback-paused-maximized").to_string()
        } else if status.automatic_paused {
            tr("settings-playback-paused-fullscreen").to_string()
        } else {
            tr("settings-playback-no-rule").to_string()
        };
        pause.push(Row {
            title: reason,
            desc: status.outputs.join(", "),
            control: Control::Static,
        });
        if !status.available_processes.is_empty() {
            pause.push(Row {
                title: tr("settings-playback-choose-process").to_string(),
                desc: String::new(),
                control: Control::Dropdown {
                    path: "playback.addProcess".to_string(),
                    current: String::new(),
                    palettes: Vec::new(),
                    options: status
                        .available_processes
                        .iter()
                        .map(|name| (name.clone(), name.clone()))
                        .collect(),
                },
            });
        }
    }
    section(
        &mut out,
        tr("settings-playback-pause-title"),
        tr("settings-playback-pause-desc"),
        pause,
    );
    out
}

fn compose_performance(cfg: &dyn SettingsSource) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut performance = collect(cfg, tab_performance);
    let mut paper = collect(cfg, tab_paper);
    section(
        &mut out,
        tr("settings-section-power"),
        tr("settings-performance-power-card-desc"),
        take_card(&mut performance, tr("settings-performance-power-card")),
    );
    let mut rendering = take_card(&mut performance, tr("settings-performance-rendering-card"));
    let _duplicates = take_rows(
        &mut rendering,
        &[
            tr("settings-performance-preview-fps-label"),
            tr("settings-performance-hover-previews-label"),
        ],
    );
    section(
        &mut out,
        tr("settings-section-picker-rendering"),
        tr("settings-performance-rendering-card-desc"),
        rendering,
    );
    section(
        &mut out,
        tr("settings-section-wallpaper-efficiency"),
        tr("settings-paper-performance-card-desc"),
        take_card(&mut paper, tr("settings-paper-performance-card")),
    );
    section(
        &mut out,
        tr("settings-performance-bug-report-label"),
        tr("settings-performance-bug-report-desc"),
        take_card(&mut performance, tr("settings-performance-bug-report-label")),
    );
    out
}

fn compose_library(
    cfg: &dyn SettingsSource,
    themes: &[String],
    outputs: &[String],
    library_watch: Option<&crate::contracts::daemon::LibraryWatchStatus>,
) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut paths = collect(cfg, tab_paths);
    let mut performance = collect(cfg, tab_performance);
    let mut general = collect(cfg, |builder| tab_general(builder, themes, outputs));
    section(
        &mut out,
        tr("settings-section-folders"),
        tr("settings-paths-directories-card-desc"),
        take_card(&mut paths, tr("settings-paths-directories-card")),
    );
    section(
        &mut out,
        tr("settings-section-library-watching"),
        tr("settings-library-watch-section-desc"),
        library_watch_rows(cfg, library_watch),
    );
    let mut images = take_card(&mut performance, tr("settings-performance-image-opt-card"));
    images.extend(take_card(&mut performance, tr("settings-performance-trash-card")));
    section(
        &mut out,
        tr("settings-section-images-recovery"),
        tr("settings-performance-image-opt-card-desc"),
        images,
    );
    section(
        &mut out,
        tr("settings-section-generated-variants"),
        tr("settings-general-auto-recolour-card-desc"),
        take_card(&mut general, tr("settings-general-auto-recolour-card")),
    );
    section(
        &mut out,
        tr("settings-section-thumbnails-cache"),
        "",
        take_card(&mut performance, tr("settings-performance-thumbs-card")),
    );
    out
}

fn library_watch_rows(
    cfg: &dyn SettingsSource,
    status: Option<&crate::contracts::daemon::LibraryWatchStatus>,
) -> Vec<Row> {
    let (title, desc) = library_watch_message(status);
    vec![
        Row { title, desc, control: Control::Static },
        Row {
            title: tr("settings-library-watch-fallback-label").to_string(),
            desc: tr("settings-library-watch-fallback-desc").to_string(),
            control: Control::Toggle {
                value: cfg.flag(keys::library::POLLING_FALLBACK),
                path: keys::library::POLLING_FALLBACK.to_string(),
            },
        },
        Row {
            title: tr("settings-library-watch-interval-label").to_string(),
            desc: tr("settings-library-watch-interval-desc").to_string(),
            control: Control::Number {
                key: keys::library::POLLING_INTERVAL_SECONDS.to_string(),
                path: keys::library::POLLING_INTERVAL_SECONDS.to_string(),
                unit: "s",
            },
        },
    ]
}

fn library_watch_message(
    status: Option<&crate::contracts::daemon::LibraryWatchStatus>,
) -> (String, String) {
    let Some(status) = status else {
        return (
            tr("settings-library-watch-unknown-label").to_string(),
            tr("settings-library-watch-unknown-desc").to_string(),
        );
    };
    let convergence = convergence_label(status.last_successful_convergence_unix_ms);
    match status.mode.as_str() {
        crate::contracts::daemon::LibraryWatchMode::POLLING if !status.ok => (
            tr("settings-library-watch-poll-failed-label").to_string(),
            crate::i18n::tr_args!(
                "settings-library-watch-poll-failed-desc",
                interval => status.interval_seconds.unwrap_or(60)
            ),
        ),
        crate::contracts::daemon::LibraryWatchMode::POLLING => (
            tr("settings-library-watch-polling-label").to_string(),
            crate::i18n::tr_args!(
                "settings-library-watch-polling-desc",
                count => status.roots.iter().filter(|root| root.mode == crate::contracts::daemon::LibraryWatchMode::POLLING).count(),
                interval => status.interval_seconds.unwrap_or(60),
                budget => status.entry_budget_per_root.unwrap_or(4096),
                convergence => convergence
            ),
        ),
        crate::contracts::daemon::LibraryWatchMode::RECOVERING => (
            tr("settings-library-watch-recovering-label").to_string(),
            tr("settings-library-watch-recovering-desc").to_string(),
        ),
        crate::contracts::daemon::LibraryWatchMode::UNAVAILABLE => (
            tr("settings-library-watch-unavailable-label").to_string(),
            tr("settings-library-watch-unavailable-desc").to_string(),
        ),
        crate::contracts::daemon::LibraryWatchMode::NATIVE
            if status.last_successful_convergence_unix_ms.is_some() =>
        {
            (
                tr("settings-library-watch-recovered-label").to_string(),
                crate::i18n::tr_args!(
                    "settings-library-watch-recovered-desc",
                    convergence => convergence
                ),
            )
        }
        crate::contracts::daemon::LibraryWatchMode::NATIVE => (
            tr("settings-library-watch-native-label").to_string(),
            tr("settings-library-watch-native-desc").to_string(),
        ),
        _ => (
            tr("settings-library-watch-unknown-label").to_string(),
            tr("settings-library-watch-unknown-desc").to_string(),
        ),
    }
}

fn convergence_label(timestamp_ms: Option<u64>) -> String {
    let Some(timestamp_ms) = timestamp_ms else {
        return tr("settings-library-watch-convergence-never").to_string();
    };
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |value| u64::try_from(value.as_millis()).unwrap_or(u64::MAX));
    let seconds = now_ms.saturating_sub(timestamp_ms) / 1000;
    if seconds < 60 {
        crate::i18n::tr_args!("settings-library-watch-convergence-seconds", value => seconds)
    } else if seconds < 3600 {
        crate::i18n::tr_args!("settings-library-watch-convergence-minutes", value => seconds / 60)
    } else {
        crate::i18n::tr_args!("settings-library-watch-convergence-hours", value => seconds / 3600)
    }
}

fn compose_sources(
    cfg: &dyn SettingsSource,
    themes: &[String],
    outputs: &[String],
) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut general = collect(cfg, |builder| tab_general(builder, themes, outputs));
    let mut wallhaven = collect(cfg, tab_wallhaven);
    let mut steam = collect(cfg, tab_steam);
    let mut providers = collect(cfg, tab_sources);

    let mut features = take_card(&mut general, tr("settings-general-features-card"));
    let mut wallhaven_rows =
        take_rows(&mut features, &[tr("settings-general-feature-wallhaven-label")]);
    wallhaven_rows.extend(take_card(&mut wallhaven, tr("settings-wallhaven-api-card")));
    wallhaven_rows.extend(take_card(&mut wallhaven, tr("settings-wallhaven-grid-card")));
    wallhaven_rows.extend(take_card(&mut wallhaven, tr("settings-wallhaven-thumb-card")));
    wallhaven_rows.extend(take_card(&mut wallhaven, tr("settings-source-defaults-card")));

    let mut steam_rows = take_rows(&mut features, &[tr("settings-general-feature-steam-label")]);
    steam_rows.extend(take_card(&mut steam, tr("settings-steam-backend-card")));
    steam_rows.extend(take_card(&mut steam, tr("settings-steam-paths-card")));
    steam_rows.extend(take_card(&mut steam, tr("settings-steam-grid-card")));
    steam_rows.extend(take_card(&mut steam, tr("settings-steam-thumb-card")));
    steam_rows.extend(take_card(&mut steam, tr("settings-source-defaults-card")));

    let rows = vec![
        details(
            tr("settings-sources-wallhaven-card"),
            tr("settings-sources-wallhaven-card-desc"),
            "source.wallhaven",
            enabled_status(cfg.flag(keys::features::WALLHAVEN)),
            wallhaven_rows,
        ),
        details(
            tr("settings-sources-unsplash-card"),
            tr("settings-sources-unsplash-card-desc"),
            "source.unsplash",
            enabled_status(cfg.flag(keys::sources::UNSPLASH_ENABLED)),
            take_card(&mut providers, tr("settings-sources-unsplash-card")),
        ),
        details(
            tr("settings-sources-pexels-card"),
            tr("settings-sources-pexels-card-desc"),
            "source.pexels",
            enabled_status(cfg.flag(keys::sources::PEXELS_ENABLED)),
            take_card(&mut providers, tr("settings-sources-pexels-card")),
        ),
        details(
            tr("settings-sources-bing-card"),
            tr("settings-sources-bing-card-desc"),
            "source.bing",
            enabled_status(cfg.flag(keys::sources::BING_ENABLED)),
            take_card(&mut providers, tr("settings-sources-bing-card")),
        ),
        details(
            tr("settings-sources-workshop-card"),
            tr("settings-sources-workshop-card-desc"),
            "source.workshop",
            enabled_status(cfg.flag(keys::features::STEAM)),
            steam_rows,
        ),
        details(
            tr("settings-sources-youtube-card"),
            tr("settings-sources-youtube-card-desc"),
            "source.youtube",
            enabled_status(cfg.flag(keys::sources::YOUTUBE_ENABLED)),
            take_card(&mut providers, tr("settings-sources-youtube-card")),
        ),
    ];
    section(
        &mut out,
        tr("settings-section-source-providers"),
        tr("settings-sources-providers-section-desc"),
        rows,
    );
    out
}

fn compose_automation(
    cfg: &dyn SettingsSource,
    themes: &[String],
    folders: &[String],
    outputs: &[String],
) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut general = collect(cfg, |builder| tab_general(builder, themes, outputs));
    let mut schedule = collect(cfg, tab_schedule);
    let mut filter = collect(cfg, |builder| tab_filter(builder, folders));
    let mut external = collect(cfg, tab_postprocessing);
    section(
        &mut out,
        tr("settings-section-random-rotation"),
        "",
        take_card(&mut general, tr("settings-general-random-card")),
    );
    section(
        &mut out,
        tr("settings-section-schedule"),
        tr("settings-schedule-schedule-card-desc"),
        take_card(&mut schedule, tr("settings-schedule-schedule-card")),
    );
    let mut location = take_card(&mut schedule, tr("settings-schedule-location-card"));
    location.extend(take_card(&mut filter, tr("settings-filter-weather-card")));
    section(
        &mut out,
        tr("settings-section-location-weather"),
        tr("settings-schedule-location-card-desc"),
        location,
    );
    let startup = take_rows(
        &mut take_card(&mut general, tr("settings-general-behaviour-card")),
        &[tr("settings-general-notify-label"), tr("settings-general-restore-label")],
    );
    section(&mut out, tr("settings-section-startup-notifications"), "", startup);
    section(
        &mut out,
        tr("settings-section-post-apply-behaviour"),
        "",
        take_card(&mut external, tr("settings-postprocessing-behaviour-card")),
    );
    section(
        &mut out,
        tr("settings-section-post-apply-commands"),
        tr("settings-postprocessing-commands-card-desc"),
        take_card(&mut external, tr("settings-postprocessing-commands-card")),
    );
    out
}

fn compose_theme(cfg: &dyn SettingsSource, backends: &[String]) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut theme = collect(cfg, |builder| tab_theme(builder, backends));
    let mut matugen = collect(cfg, tab_matugen);
    section(
        &mut out,
        tr("settings-section-behaviour"),
        tr("settings-theme-behaviour-card-desc"),
        take_card(&mut theme, tr("settings-theme-behaviour-card")),
    );
    section(
        &mut out,
        tr("settings-section-colour-driver"),
        tr("settings-theme-driver-card-desc"),
        take_card(&mut theme, tr("settings-theme-driver-card")),
    );
    section(
        &mut out,
        tr("settings-section-colour-source"),
        tr("settings-theme-engine-card-desc"),
        take_card(&mut theme, tr("settings-theme-engine-card")),
    );
    section(
        &mut out,
        tr("settings-section-appearance"),
        tr("settings-theme-appearance-card-desc"),
        take_card(&mut theme, tr("settings-theme-appearance-card")),
    );
    section(
        &mut out,
        tr("settings-section-palette"),
        tr("settings-theme-fixed-card-desc"),
        take_card(&mut theme, tr("settings-theme-fixed-card")),
    );
    section(
        &mut out,
        tr("settings-theme-designer-label"),
        tr("settings-theme-profile-desc"),
        take_card(&mut theme, tr("settings-theme-designer-label")),
    );
    section(
        &mut out,
        tr("settings-section-appearance"),
        tr("settings-theme-matugen-card-desc"),
        take_card(&mut theme, tr("settings-theme-matugen-card")),
    );
    section(
        &mut out,
        tr("settings-section-imported-result"),
        "",
        take_card(&mut theme, tr("settings-theme-external-card")),
    );
    section(
        &mut out,
        tr("settings-section-engine-note"),
        tr("settings-theme-native-card-desc"),
        take_card(&mut theme, tr("settings-theme-native-card")),
    );
    section(
        &mut out,
        tr("settings-section-engine-options"),
        "",
        take_card(&mut theme, tr("settings-theme-wallust-card")),
    );
    section(
        &mut out,
        tr("settings-section-matugen-command"),
        tr("settings-matugen-external-card-desc"),
        take_card(&mut matugen, tr("settings-matugen-external-card")),
    );
    section(
        &mut out,
        tr("settings-section-theme-outputs"),
        tr("settings-matugen-integrations-card-desc"),
        take_card(&mut matugen, tr("settings-matugen-integrations-card")),
    );
    out
}

fn compose_integrations(cfg: &dyn SettingsSource, themes: &[String]) -> Vec<(Card, Vec<Row>)> {
    let mut out = collect(cfg, tab_integrations);
    if cfg.is_niri() {
        out.extend(collect(cfg, |builder| tab_niri(builder, themes)));
    }
    out
}

#[cfg(test)]
mod ownership_tests;
