use crate::contracts::media::MediaKind;
use crate::contracts::picker::Mode;
use crate::contracts::settings::wallpaper_kind::{STATIC, VIDEO, WE};
use crate::contracts::settings::{LockScreenTarget, SettingsSource, keys};
use crate::frontend::settings::tables::category;
use crate::i18n::tr;

use super::folders_tab::tab_paths;
use super::launch_tab::tab_launch;
use super::media_tabs::{
    tab_paper, tab_paper_asleep, tab_search, tab_sources, tab_steam, tab_wallhaven,
    tab_wallpaper_engine,
};
use super::position_tab::tab_position;
use super::profiles_tab::tab_profiles;
use super::selector_tab::{selector_cards, selector_gated, tab_filter, tab_selector};
use super::system_tabs::{
    tab_general, tab_keybinds, tab_language, tab_performance, tab_postprocessing, tab_schedule,
    tab_weather,
};
use super::theme_tabs::{
    tab_integrations, tab_lock_asleep, tab_lockscreens, tab_matugen, tab_niri, tab_theme,
    tab_theme_asleep,
};
use super::transition_tab::{tab_drawing, tab_transitions, tab_transitions_asleep};
use super::{ActionId, Builder, Card, Control, Row};

pub(super) fn type_chip_label(kind: &str) -> &'static str {
    match kind {
        STATIC => tr("settings-filter-type-images"),
        VIDEO => tr("settings-filter-type-videos"),
        WE => tr("settings-filter-type-we"),
        _ => tr("settings-filter-type-all"),
    }
}

pub(super) fn folder_dropdown_options(folders: &[String]) -> Vec<(String, String)> {
    let mut out = vec![(String::from("*"), tr("settings-filter-all-folders").to_string())];
    for folder in folders {
        if folder.is_empty() || folder == "*" {
            continue;
        }
        out.push((folder.clone(), crate::frontend::ui::folder_short_label(folder, folders)));
    }
    out
}

#[derive(Clone, Copy, Default)]
pub(crate) struct DrawingJob {
    pub(crate) library: bool,
    pub(crate) running: bool,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct RuntimeStatus<'a> {
    pub(crate) outputs: &'a [String],
    pub(crate) statuses: &'a [crate::contracts::daemon::OutputStatus],
    pub(crate) previews: Option<&'a std::collections::HashMap<String, String>>,
    pub(crate) library_watch: Option<&'a crate::contracts::daemon::LibraryWatchStatus>,
    pub(crate) playback: Option<&'a crate::contracts::daemon::PlaybackStatus>,
    pub(crate) app_themes: Option<&'a crate::contracts::daemon::AppThemesResult>,
    pub(crate) weather: Option<&'a crate::contracts::daemon::WeatherStatus>,
    pub(crate) weather_pending: bool,
    pub(crate) playlists: &'a [crate::contracts::playlists::Playlist],
    pub(crate) sddm_themes: &'a [String],
    pub(crate) drawing: Option<&'a crate::contracts::settings::drawing::Status>,
    pub(crate) drawing_selection: Option<&'a str>,
    pub(crate) drawing_view_count: usize,
    pub(crate) drawing_job: DrawingJob,
    pub(crate) connected: bool,
    pub(crate) layout_premade: Option<usize>,
}

#[cfg(test)]
pub fn build_tab(
    tab: &str,
    cfg: &dyn SettingsSource,
    themes: &[String],
    folders: &[String],
    analysis: &str,
    backends: &[String],
) -> Vec<(Card, Vec<Row>)> {
    build_tab_with_runtime_status(
        tab,
        cfg,
        themes,
        folders,
        analysis,
        backends,
        RuntimeStatus::default(),
    )
}

pub(crate) fn build_tab_with_runtime_status(
    tab: &str,
    cfg: &dyn SettingsSource,
    themes: &[String],
    folders: &[String],
    analysis: &str,
    backends: &[String],
    runtime: RuntimeStatus<'_>,
) -> Vec<(Card, Vec<Row>)> {
    match tab {
        category::PICKER => compose_picker(cfg, themes, folders, runtime.outputs, analysis),
        category::DISPLAYS => compose_displays(
            cfg,
            runtime.statuses,
            runtime.previews.unwrap_or(&std::collections::HashMap::new()),
        ),
        category::TRANSITIONS => compose_transitions(cfg, runtime),
        category::PLAYBACK => compose_playback(cfg, themes, runtime.outputs, runtime.playback),
        category::LIBRARY => compose_library(cfg, themes, runtime.outputs, runtime.library_watch),
        category::SOURCES => compose_sources(cfg, themes, runtime.outputs),
        category::AUTOMATION => compose_automation(
            cfg,
            themes,
            runtime.outputs,
            runtime.weather,
            runtime.playlists,
            runtime.weather_pending,
        ),
        category::THEME => compose_theme(cfg, backends, runtime.app_themes),
        category::INTEGRATIONS => compose_integrations(cfg, runtime.sddm_themes),
        category::APP => compose_app(cfg, themes, runtime.outputs),
        _ => Vec::new(),
    }
}

fn compose_displays(
    cfg: &dyn SettingsSource,
    statuses: &[crate::contracts::daemon::OutputStatus],
    previews: &std::collections::HashMap<String, String>,
) -> Vec<(Card, Vec<Row>)> {
    let mut paper = collect(cfg, tab_paper);
    let mut rows = take_control_paths(
        &mut take_card(&mut paper, crate::i18n::text("settings-paper-engine-card")),
        &[keys::display::FILL_MODE],
    );
    if let Some(global) = rows.first_mut() {
        global.title = tr("settings-displays-default-placement-label").to_string();
        global.desc = tr("settings-displays-default-placement-desc").to_string();
    }
    let global_fill = cfg.text(keys::display::FILL_MODE);
    let global_fill_label = fill_mode_label(&global_fill);
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
        let mut fill_modes = vec![(
            String::new(),
            crate::i18n::tr_args!("settings-displays-use-global", mode => global_fill_label),
        )];
        fill_modes.extend(
            super::super::tables::FILL_MODES
                .map(|key| (key.to_string(), fill_mode_label(key).to_string())),
        );
        let placement = Row::of(
            tr("settings-displays-placement-label").to_string(),
            tr("settings-displays-placement-desc").to_string(),
            Control::Chips {
                path: fill_path,
                options: fill_modes,
                current: output.fill.clone(),
                disabled: Vec::new(),
                background: Some(crate::frontend::settings::background::BackgroundControl::new(
                    cfg,
                    &output.name,
                )),
            },
        );
        let lock_path = format!("{}.{name}", keys::display::OUTPUT_LOCKS, name = output.name);
        let lock = Row::of(
            tr("settings-displays-lock-label").to_string(),
            tr("settings-displays-lock-desc").to_string(),
            Control::Toggle { value: cfg.flag(&lock_path), path: lock_path },
        );
        let mut controls = vec![placement, lock];
        if output.is_connected() {
            use crate::frontend::display_control::DisplayControl;
            let mut live = vec![DisplayControl::Colours {
                target: output.target().to_string(),
                active: cfg.text(keys::display::THEME_OUTPUT) == output.name,
            }];
            if output.kind.has_audio_controls() {
                live.push(DisplayControl::Audio {
                    target: output.target().to_string(),
                    muted: output.mute,
                    volume: output.volume,
                    source_preview: false,
                });
                live.push(DisplayControl::Playback {
                    target: output.target().to_string(),
                    paused: output.paused,
                    manual: output.manual_paused,
                });
            }
            controls.extend(live.into_iter().map(|control| {
                Row::of(control.label().to_string(), String::new(), Control::Display(control))
            }));
        }
        rows.push(Row::of(
            output.name.clone(),
            metadata,
            Control::StackBar {
                id: format!("display:{}", output.target()),
                summary: current.to_string(),
                preview: previews.get(&output.name).cloned(),
                rows: controls,
            },
        ));
    }
    if statuses.is_empty() {
        rows.push(Row::of(
            tr("settings-displays-empty-label").to_string(),
            tr("settings-displays-empty-desc").to_string(),
            Control::Static,
        ));
    }
    vec![(
        Card::new(crate::i18n::text("settings-displays-card"), tr("settings-displays-card-desc")),
        rows,
    )]
}

fn fill_mode_label(mode: &str) -> &'static str {
    tr(match mode {
        "fit" => "settings-paper-fill-fit",
        "stretch" => "settings-paper-fill-stretch",
        "center" => "settings-paper-fill-center",
        "tile" => "settings-paper-fill-tile",
        "span" => "settings-paper-fill-span",
        _ => "settings-paper-fill-fill",
    })
}

fn collect(
    cfg: &dyn SettingsSource,
    build: impl FnOnce(&mut Builder<'_>),
) -> Vec<(Card, Vec<Row>)> {
    let mut builder = Builder { cfg, cards: Vec::new() };
    build(&mut builder);
    builder.cards
}

pub(in crate::frontend::settings) fn take_card(
    cards: &mut Vec<(Card, Vec<Row>)>,
    title: crate::i18n::Text,
) -> Vec<Row> {
    cards
        .iter()
        .position(|(card, _)| card.key == title.key)
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
    title: crate::i18n::Text,
    subtitle: &'static str,
    rows: Vec<Row>,
) {
    if !rows.is_empty() {
        cards.push((Card::new(title, subtitle), rows));
    }
}

fn segmented_rows(cards: Vec<(Card, Vec<Row>)>) -> Vec<Row> {
    let mut rows = Vec::new();
    for (card, mut segment_rows) in cards {
        if segment_rows.is_empty() {
            continue;
        }
        rows.push(Row::of(card.title.to_string(), card.subtitle.to_string(), Control::Segment));
        rows.append(&mut segment_rows);
    }
    rows
}

fn details(title: &str, desc: &str, id: &str, summary: &str, rows: Vec<Row>) -> Row {
    Row::of(
        title.to_string(),
        desc.to_string(),
        Control::Details { id: id.to_string(), summary: summary.to_string(), rows },
    )
}

fn enabled_status(enabled: bool) -> &'static str {
    tr(if enabled { "settings-control-enabled" } else { "settings-control-disabled" })
}

fn compose_picker(
    cfg: &dyn SettingsSource,
    themes: &[String],
    folders: &[String],
    outputs: &[String],
    analysis: &str,
) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut general = collect(cfg, |builder| tab_general(builder, themes, outputs));
    let mut selector = collect(cfg, tab_selector);

    let mut general_rows = Vec::new();
    let mut layout = take_card(&mut selector, crate::i18n::text("settings-selector-layout-card"));
    general_rows.extend(take_rows(
        &mut take_card(&mut general, crate::i18n::text("settings-general-behaviour-card")),
        &[
            tr("settings-general-close-on-selection-label"),
            tr("settings-general-hover-selects-label"),
        ],
    ));
    general_rows.extend(take_rows(
        &mut layout,
        &[tr("settings-selector-start-position-label"), tr("settings-selector-loop-around-label")],
    ));
    section(&mut out, crate::i18n::text("settings-section-general"), "", general_rows);

    let _filter_motion = take_rows(&mut layout, &[tr("settings-selector-filter-motion-label")]);
    layout.extend(take_card(&mut selector, crate::i18n::text("settings-selector-presets-card")));
    section(&mut out, crate::i18n::text("settings-section-layout-presets"), "", layout);

    out.extend(compose_mode_sections(cfg, &mut selector));

    let mut previews =
        take_card(&mut selector, crate::i18n::text("settings-selector-live-preview-card"));
    previews.extend(take_card(
        &mut selector,
        crate::i18n::text("settings-selector-video-preview-card"),
    ));
    section(&mut out, crate::i18n::text("settings-section-cards-previews"), "", previews);

    let tag_panel = take_card(&mut selector, crate::i18n::text("settings-selector-tag-cloud-card"));
    section(
        &mut out,
        crate::i18n::text("settings-section-tag-panel"),
        tr("settings-selector-tag-cloud-card-desc"),
        tag_panel,
    );

    out.extend(compose_filter_picker(cfg, folders));
    out.extend(compose_search(cfg, analysis));
    out.extend(compose_picker_motion(cfg));
    out
}

const MODE_CARDS: [&str; 10] = [
    "settings-selector-cells-card",
    "settings-selector-cards-card",
    "settings-selector-performance-card",
    "settings-selector-depth-card",
    "settings-selector-mode-collection",
    "settings-selector-ribbons-card",
    "settings-selector-hand-effects-card",
    "settings-selector-corners-card",
    "settings-filter-position-card",
    "settings-selector-tag-cloud-position-card",
];

fn compose_mode_sections(
    cfg: &dyn SettingsSource,
    selector: &mut Vec<(Card, Vec<Row>)>,
) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut position = collect(cfg, tab_position);
    let mode = Mode::from_key(&cfg.display_mode());
    section(
        &mut out,
        crate::i18n::text("settings-position-frame-card"),
        tr("settings-position-frame-card-desc"),
        take_card(&mut position, crate::i18n::text("settings-position-frame-card")),
    );
    let active_position = match mode {
        Mode::Hex => Some(crate::i18n::text("settings-position-hex-card")),
        Mode::Grid => Some(crate::i18n::text("settings-position-wall-card")),
        Mode::Sandy => Some(crate::i18n::text("settings-position-sandy-card")),
        Mode::Hand => Some(crate::i18n::text("settings-position-hand-card")),
        Mode::Depth | Mode::Collection => None,
        Mode::Slices => Some(crate::i18n::text("settings-position-slices-card")),
    };

    for title in [
        crate::i18n::text("settings-position-slices-card"),
        crate::i18n::text("settings-position-hex-card"),
        crate::i18n::text("settings-position-wall-card"),
        crate::i18n::text("settings-position-sandy-card"),
        crate::i18n::text("settings-position-hand-card"),
    ] {
        let mut rows = take_card(&mut position, title);
        let mut section_title = title;
        let mut subtitle = tr("settings-position-picker-card-desc");
        let mut hand_motion = Vec::new();
        if Some(title) == active_position {
            let source_title = match mode {
                Mode::Hex | Mode::Grid => crate::i18n::text("settings-selector-composition-card"),
                Mode::Sandy => crate::i18n::text("settings-selector-sandy-card"),
                Mode::Hand => crate::i18n::text("settings-selector-hand-card"),
                Mode::Slices => crate::i18n::text("settings-selector-slice-size-card"),
                Mode::Depth | Mode::Collection => unreachable!(),
            };
            let mut source = take_card(selector, source_title);
            if mode == Mode::Hand {
                hand_motion = take_control_paths(&mut source, &HAND_MOTION_KEYS);
            }
            source.append(&mut rows);
            rows = source;
            if mode == Mode::Hand {
                section_title = source_title;
                subtitle = tr("settings-selector-hand-card-desc");
            }
        }
        section(&mut out, section_title, subtitle, rows);
        section(
            &mut out,
            crate::i18n::text("settings-section-hand-motion"),
            tr("settings-selector-hand-move-desc"),
            hand_motion,
        );
    }

    for title in MODE_CARDS.map(crate::i18n::text) {
        let subtitle = if title == crate::i18n::text("settings-filter-position-card") {
            tr("settings-filter-position-card-desc")
        } else if title == crate::i18n::text("settings-selector-tag-cloud-position-card") {
            tr("settings-selector-tag-cloud-position-card-desc")
        } else {
            selector
                .iter()
                .find(|(card, _)| card.key == title.key)
                .map_or("", |(card, _)| card.subtitle)
        };
        let rows = if title == crate::i18n::text("settings-filter-position-card")
            || title == crate::i18n::text("settings-selector-tag-cloud-position-card")
        {
            take_card(&mut position, title)
        } else {
            take_card(selector, title)
        };
        section(&mut out, title, subtitle, rows);
    }
    out
}

const HAND_MOTION_KEYS: [&str; 7] = [
    keys::selector::HAND_MOVE,
    keys::selector::HAND_MOVE_CORKSCREW,
    keys::selector::HAND_MOVE_CASCADE,
    keys::selector::HAND_MOVE_SHUFFLE,
    keys::selector::HAND_MOVE_RIBBON,
    keys::selector::HAND_MOVE_SPIRAL,
    keys::selector::HAND_SPEED,
];

pub(in crate::frontend::settings) fn take_control_paths(
    rows: &mut Vec<Row>,
    paths: &[&str],
) -> Vec<Row> {
    let (selected, remaining): (Vec<_>, Vec<_>) = std::mem::take(rows)
        .into_iter()
        .partition(|row| row.control.path().is_some_and(|path| paths.contains(&path)));
    *rows = remaining;
    selected
}

fn compose_filter_picker(cfg: &dyn SettingsSource, folders: &[String]) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut filter = collect(cfg, |builder| tab_filter(builder, folders));

    section(
        &mut out,
        crate::i18n::text("settings-section-appearance"),
        tr("settings-filter-appearance-card-desc"),
        take_card(&mut filter, crate::i18n::text("settings-filter-appearance-card")),
    );
    let mut buttons = take_card(&mut filter, crate::i18n::text("arrange-title"));
    buttons.extend(take_card(&mut filter, crate::i18n::text("settings-filter-type-chips-card")));
    buttons.extend(take_card(&mut filter, crate::i18n::text("settings-filter-sort-options-card")));
    buttons
        .extend(take_card(&mut filter, crate::i18n::text("settings-filter-other-controls-card")));
    section(
        &mut out,
        crate::i18n::text("settings-section-buttons"),
        tr("settings-filter-type-chips-card-desc"),
        buttons,
    );
    section(
        &mut out,
        crate::i18n::text("settings-section-resolution-presets"),
        tr("settings-filter-resolution-presets-card-desc"),
        take_card(&mut filter, crate::i18n::text("settings-filter-resolution-presets-card")),
    );
    section(
        &mut out,
        crate::i18n::text("settings-section-defaults"),
        tr("settings-filter-default-folder-card-desc"),
        take_card(&mut filter, crate::i18n::text("settings-filter-default-folder-card")),
    );

    section(
        &mut out,
        crate::i18n::text("settings-section-visibility"),
        tr("settings-filter-visibility-card-desc"),
        take_card(&mut filter, crate::i18n::text("settings-filter-visibility-card")),
    );
    out
}

fn compose_search(cfg: &dyn SettingsSource, analysis: &str) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
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
        take_card(&mut search, crate::i18n::text("settings-tagging-search-card")),
    )];
    section(
        &mut out,
        crate::i18n::text("settings-section-search-tagging"),
        tr("settings-tagging-overview-desc"),
        rows,
    );
    let mut model_rows = take_card(&mut search, crate::i18n::text("settings-tagging-models-card"));
    if !analysis.trim().is_empty() {
        model_rows.insert(
            0,
            Row::of(
                tr("settings-semantic-model-status-label").to_string(),
                analysis.to_string(),
                Control::Static,
            ),
        );
    }
    section(
        &mut out,
        crate::i18n::text("settings-section-semantic-models"),
        tr("settings-tagging-models-card-desc"),
        model_rows,
    );
    out
}

fn compose_picker_motion(cfg: &dyn SettingsSource) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut launch = collect(cfg, tab_launch);
    let mut selector = collect(cfg, tab_selector);

    section(
        &mut out,
        crate::i18n::text("settings-section-launch"),
        tr("settings-launch-launch-card-desc"),
        take_card(&mut launch, crate::i18n::text("settings-launch-launch-card")),
    );
    let mut picker_motion = take_rows(
        &mut take_card(&mut selector, crate::i18n::text("settings-selector-layout-card")),
        &[tr("settings-selector-filter-motion-label")],
    );
    picker_motion
        .extend(take_card(&mut selector, crate::i18n::text("settings-selector-card-flip-card")));
    section(
        &mut out,
        crate::i18n::text("settings-section-picker-motion"),
        tr("settings-selector-card-flip-card-desc"),
        picker_motion,
    );

    let mut sandy = take_card(&mut selector, crate::i18n::text("settings-selector-sand-card"));
    sandy.extend(take_card(&mut selector, crate::i18n::text("settings-selector-ring-card")));
    section(
        &mut out,
        crate::i18n::text("settings-section-sandy-motion"),
        tr("settings-selector-sand-card-desc"),
        sandy,
    );
    out
}

fn compose_transitions(
    cfg: &dyn SettingsSource,
    runtime: RuntimeStatus<'_>,
) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut transitions = collect(cfg, tab_transitions);
    let mut drawing = collect(cfg, |builder| {
        tab_drawing(
            builder,
            runtime.drawing,
            runtime.drawing_selection,
            runtime.drawing_view_count,
            runtime.drawing_job.library,
        );
    });
    let mut paper = collect(cfg, tab_paper);
    let mut wallpaper_transitions =
        take_card(&mut transitions, crate::i18n::text("settings-transitions-transitions-card"));
    wallpaper_transitions.extend(take_card(
        &mut transitions,
        crate::i18n::text("settings-transitions-preview-card"),
    ));
    section(
        &mut out,
        crate::i18n::text("settings-section-wallpaper-transitions"),
        tr("settings-transitions-transitions-card-desc"),
        wallpaper_transitions,
    );
    section(
        &mut out,
        crate::i18n::text("settings-drawing-card"),
        tr("settings-drawing-card-desc"),
        take_card(&mut drawing, crate::i18n::text("settings-drawing-card")),
    );
    section(
        &mut out,
        crate::i18n::text("settings-section-transition-performance"),
        tr("settings-transitions-sand-card-desc"),
        take_card(&mut transitions, crate::i18n::text("settings-transitions-sand-card")),
    );
    section(
        &mut out,
        crate::i18n::text("settings-section-awww-transition"),
        "",
        take_card(&mut paper, crate::i18n::text("settings-paper-awww-card")),
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

    let mut display = take_card(&mut paper, crate::i18n::text("settings-paper-engine-card"));
    let _global_placement = take_control_paths(&mut display, &[keys::display::FILL_MODE]);
    if let Some(row) = display.iter_mut().find(|row| row.title == tr("settings-paper-engine-label"))
    {
        row.title = tr("settings-paper-static-backend-label").to_string();
    }
    section(
        &mut out,
        crate::i18n::text("settings-section-display"),
        tr("settings-paper-engine-card-desc"),
        display,
    );
    let mut audio = take_card(&mut general, crate::i18n::text("settings-general-audio-card"));
    if let Some(status) = playback.filter(|_| cfg.flag(keys::playback::MUTE_ON_OTHER_AUDIO)) {
        let title = if status.audio_ducked {
            tr("settings-general-other-audio-ducked")
        } else if status.other_audio_supported {
            tr("settings-general-other-audio-listening")
        } else {
            tr("settings-general-other-audio-unavailable")
        };
        audio.push(Row::of(title.to_string(), String::new(), Control::Static));
    }
    section(
        &mut out,
        crate::i18n::text("settings-section-audio"),
        tr("settings-general-audio-card-desc"),
        audio,
    );

    if cfg.flag_default_true(keys::features::STEAM) {
        let mut rendering =
            take_card(&mut workshop, crate::i18n::text("settings-wallpaper-engine-rendering-card"));
        rendering.extend(take_card(
            &mut workshop,
            crate::i18n::text("settings-wallpaper-engine-effects-card"),
        ));
        section(
            &mut out,
            crate::i18n::text("settings-section-workshop-rendering"),
            tr("settings-wallpaper-engine-rendering-card-desc"),
            rendering,
        );
    }
    let mut pause = take_card(&mut paper, crate::i18n::text("settings-playback-pause-title"));
    let _idle = take_control_paths(&mut pause, &[keys::paper::IDLE_PAUSE_SECONDS]);
    if let Some(status) = playback {
        pause.extend(pause_status(cfg, status));
    }
    section(
        &mut out,
        crate::i18n::text("settings-playback-pause-title"),
        tr("settings-playback-pause-desc"),
        pause,
    );
    out.rotate_right(1);
    out
}

fn pause_status(
    cfg: &dyn SettingsSource,
    status: &crate::contracts::daemon::PlaybackStatus,
) -> [Row; 2] {
    let detection = if !(cfg.flag(keys::playback::FULLSCREEN)
        || cfg.flag(keys::playback::MAXIMIZED)
        || cfg.flag(keys::playback::FOCUSED)
        || cfg.is_niri() && cfg.flag(keys::niri::FULL_WIDTH_PAUSE))
    {
        tr("settings-playback-detection-off")
    } else if (!cfg.flag(keys::playback::FULLSCREEN) || status.fullscreen_supported)
        && (!cfg.flag(keys::playback::MAXIMIZED) || status.maximized_supported)
        && (!cfg.flag(keys::playback::FOCUSED) || status.focused_supported)
        && (!(cfg.is_niri() && cfg.flag(keys::niri::FULL_WIDTH_PAUSE))
            || status.full_width_supported)
    {
        tr("settings-playback-detection-ready")
    } else {
        tr("settings-playback-detection-unavailable")
    };
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
    } else if status.focused_paused {
        tr("settings-playback-paused-focused").to_string()
    } else if status.automatic_paused {
        tr("settings-playback-paused-fullscreen").to_string()
    } else {
        tr("settings-playback-no-rule").to_string()
    };
    [
        Row::of(detection.to_string(), String::new(), Control::Static),
        Row::of(reason, status.outputs.join(", "), Control::Static),
    ]
}

fn compose_performance(cfg: &dyn SettingsSource) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut performance = collect(cfg, tab_performance);
    let mut paper = collect(cfg, tab_paper);
    section(
        &mut out,
        crate::i18n::text("settings-section-power"),
        tr("settings-performance-power-card-desc"),
        take_card(&mut performance, crate::i18n::text("settings-performance-power-card")),
    );
    let mut rendering =
        take_card(&mut performance, crate::i18n::text("settings-performance-rendering-card"));
    let _duplicates = take_rows(
        &mut rendering,
        &[
            tr("settings-performance-preview-fps-label"),
            tr("settings-performance-hover-previews-label"),
        ],
    );
    section(
        &mut out,
        crate::i18n::text("settings-section-picker-rendering"),
        tr("settings-performance-rendering-card-desc"),
        rendering,
    );
    let mut efficiency =
        take_card(&mut paper, crate::i18n::text("settings-paper-performance-card"));
    efficiency.extend(take_control_paths(
        &mut take_card(&mut paper, crate::i18n::text("settings-playback-pause-title")),
        &[keys::paper::IDLE_PAUSE_SECONDS],
    ));
    section(
        &mut out,
        crate::i18n::text("settings-section-wallpaper-efficiency"),
        tr("settings-paper-performance-card-desc"),
        efficiency,
    );
    section(
        &mut out,
        crate::i18n::text("settings-performance-bug-report-label"),
        tr("settings-performance-bug-report-desc"),
        take_card(&mut performance, crate::i18n::text("settings-performance-bug-report-label")),
    );
    out
}

fn compose_app(
    cfg: &dyn SettingsSource,
    themes: &[String],
    outputs: &[String],
) -> Vec<(Card, Vec<Row>)> {
    let mut out = collect(cfg, tab_language);
    let mut general = collect(cfg, |builder| tab_general(builder, themes, outputs));
    section(
        &mut out,
        crate::i18n::text("settings-general-general-card"),
        "",
        take_card(&mut general, crate::i18n::text("settings-general-general-card")),
    );
    let mut performance = collect(cfg, tab_performance);
    section(
        &mut out,
        crate::i18n::text("settings-configuration-transfer-card"),
        tr("settings-configuration-transfer-card-desc"),
        take_card(&mut performance, crate::i18n::text("settings-configuration-transfer-card")),
    );
    out.extend(compose_performance(cfg));
    section(
        &mut out,
        crate::i18n::text("settings-section-controls"),
        tr("settings-keybinds-controls-card-desc"),
        segmented_rows(collect(cfg, tab_keybinds)),
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
    out.push((
        Card::new(
            crate::i18n::text("settings-section-folders"),
            tr("settings-paths-source-card-desc"),
        ),
        take_card(&mut paths, crate::i18n::text("settings-paths-source-card")),
    ));
    out.push((
        Card::new(
            crate::i18n::text("settings-paths-excluded-card"),
            tr("settings-paths-excluded-card-desc"),
        ),
        take_card(&mut paths, crate::i18n::text("settings-paths-excluded-card")),
    ));
    let mut profiles = collect(cfg, tab_profiles);
    out.push((
        Card::new(crate::i18n::text("settings-profiles-card"), tr("settings-profiles-card-desc")),
        take_card(&mut profiles, crate::i18n::text("settings-profiles-card")),
    ));
    section(
        &mut out,
        crate::i18n::text("settings-section-library-watching"),
        tr("settings-library-watch-section-desc"),
        library_watch_rows(cfg, library_watch),
    );
    let mut images =
        take_card(&mut performance, crate::i18n::text("settings-performance-image-opt-card"));
    images
        .extend(take_card(&mut performance, crate::i18n::text("settings-performance-trash-card")));
    section(
        &mut out,
        crate::i18n::text("settings-section-images-recovery"),
        tr("settings-performance-image-opt-card-desc"),
        images,
    );
    section(
        &mut out,
        crate::i18n::text("settings-section-generated-variants"),
        tr("settings-general-auto-recolour-card-desc"),
        take_card(&mut general, crate::i18n::text("settings-general-auto-recolour-card")),
    );
    section(
        &mut out,
        crate::i18n::text("settings-section-thumbnails-cache"),
        "",
        take_card(&mut performance, crate::i18n::text("settings-performance-thumbs-card")),
    );
    out
}

fn library_watch_rows(
    cfg: &dyn SettingsSource,
    status: Option<&crate::contracts::daemon::LibraryWatchStatus>,
) -> Vec<Row> {
    let (title, desc) = library_watch_message(status);
    vec![
        Row::of(title, desc, Control::Static),
        Row::of(
            tr("settings-library-watch-fallback-label").to_string(),
            tr("settings-library-watch-fallback-desc").to_string(),
            Control::Toggle {
                value: cfg.flag(keys::library::POLLING_FALLBACK),
                path: keys::library::POLLING_FALLBACK.to_string(),
            },
        ),
        Row::of(
            tr("settings-library-watch-interval-label").to_string(),
            tr("settings-library-watch-interval-desc").to_string(),
            Control::Number {
                key: keys::library::POLLING_INTERVAL_SECONDS.to_string(),
                path: keys::library::POLLING_INTERVAL_SECONDS.to_string(),
                unit: "s",
            },
        ),
    ]
}

pub(in crate::frontend::settings) fn library_watch_label(
    status: Option<&crate::contracts::daemon::LibraryWatchStatus>,
) -> String {
    library_watch_message(status).0
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
    let Some(timestamp_ms) = timestamp_ms.filter(|ms| *ms > 0) else {
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

    let mut features = take_card(&mut general, crate::i18n::text("settings-general-features-card"));
    let mut wallhaven_rows =
        take_rows(&mut features, &[tr("settings-general-feature-wallhaven-label")]);
    wallhaven_rows
        .extend(take_card(&mut wallhaven, crate::i18n::text("settings-wallhaven-api-card")));
    wallhaven_rows
        .extend(take_card(&mut wallhaven, crate::i18n::text("settings-wallhaven-grid-card")));
    wallhaven_rows
        .extend(take_card(&mut wallhaven, crate::i18n::text("settings-wallhaven-thumb-card")));
    wallhaven_rows
        .extend(take_card(&mut wallhaven, crate::i18n::text("settings-source-defaults-card")));

    let mut steam_rows = take_rows(&mut features, &[tr("settings-general-feature-steam-label")]);
    steam_rows.extend(take_card(&mut steam, crate::i18n::text("settings-steam-backend-card")));
    steam_rows.extend(take_card(&mut steam, crate::i18n::text("settings-steam-paths-card")));
    steam_rows.extend(take_card(&mut steam, crate::i18n::text("settings-steam-grid-card")));
    steam_rows.extend(take_card(&mut steam, crate::i18n::text("settings-steam-thumb-card")));
    steam_rows.extend(take_card(&mut steam, crate::i18n::text("settings-source-defaults-card")));

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
            take_card(&mut providers, crate::i18n::text("settings-sources-unsplash-card")),
        ),
        details(
            tr("settings-sources-pexels-card"),
            tr("settings-sources-pexels-card-desc"),
            "source.pexels",
            enabled_status(cfg.flag(keys::sources::PEXELS_ENABLED)),
            take_card(&mut providers, crate::i18n::text("settings-sources-pexels-card")),
        ),
        details(
            tr("settings-sources-bing-card"),
            tr("settings-sources-bing-card-desc"),
            "source.bing",
            enabled_status(cfg.flag(keys::sources::BING_ENABLED)),
            take_card(&mut providers, crate::i18n::text("settings-sources-bing-card")),
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
            take_card(&mut providers, crate::i18n::text("settings-sources-youtube-card")),
        ),
    ];
    section(
        &mut out,
        crate::i18n::text("settings-section-source-providers"),
        tr("settings-sources-providers-section-desc"),
        rows,
    );
    out
}

fn compose_automation(
    cfg: &dyn SettingsSource,
    themes: &[String],
    outputs: &[String],
    weather: Option<&crate::contracts::daemon::WeatherStatus>,
    playlists: &[crate::contracts::playlists::Playlist],
    weather_pending: bool,
) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut general = collect(cfg, |builder| tab_general(builder, themes, outputs));
    let mut schedule = collect(cfg, tab_schedule);
    let mut weather_rows =
        collect(cfg, |builder| tab_weather(builder, weather, playlists, weather_pending));
    let mut external = collect(cfg, tab_postprocessing);
    section(
        &mut out,
        crate::i18n::text("settings-section-random-rotation"),
        "",
        take_card(&mut general, crate::i18n::text("settings-general-random-card")),
    );
    section(
        &mut out,
        crate::i18n::text("settings-section-schedule"),
        tr("settings-schedule-schedule-card-desc"),
        take_card(&mut schedule, crate::i18n::text("settings-schedule-schedule-card")),
    );
    let mut location =
        take_card(&mut schedule, crate::i18n::text("settings-schedule-location-card"));
    let mut outdoor_rows = take_card(&mut weather_rows, crate::i18n::text("settings-weather-card"));
    outdoor_rows.append(&mut location);
    section(
        &mut out,
        crate::i18n::text("settings-section-location-weather"),
        tr("settings-schedule-location-card-desc"),
        outdoor_rows,
    );
    let startup = take_rows(
        &mut take_card(&mut general, crate::i18n::text("settings-general-behaviour-card")),
        &[tr("settings-general-notify-label"), tr("settings-general-restore-label")],
    );
    section(&mut out, crate::i18n::text("settings-section-startup-notifications"), "", startup);
    section(
        &mut out,
        crate::i18n::text("settings-section-post-apply-behaviour"),
        "",
        take_card(&mut external, crate::i18n::text("settings-postprocessing-behaviour-card")),
    );
    section(
        &mut out,
        crate::i18n::text("settings-section-post-apply-commands"),
        tr("settings-postprocessing-commands-card-desc"),
        take_card(&mut external, crate::i18n::text("settings-postprocessing-commands-card")),
    );
    out
}

const PLASMA_LOCK: [&str; 3] = [
    keys::plasma::LOCK_SCREEN_MODE,
    keys::plasma::LOCK_SCREEN_IMAGE,
    keys::plasma::LOCK_SCREEN_DYNAMIC,
];

const THEME_SECTIONS: [(&str, Option<&str>, &str); 10] = [
    (
        "settings-section-behaviour",
        Some("settings-theme-behaviour-card-desc"),
        "settings-theme-behaviour-card",
    ),
    (
        "settings-section-colour-driver",
        Some("settings-theme-driver-card-desc"),
        "settings-theme-driver-card",
    ),
    (
        "settings-section-colour-source",
        Some("settings-theme-engine-card-desc"),
        "settings-theme-engine-card",
    ),
    (
        "settings-section-appearance",
        Some("settings-theme-appearance-card-desc"),
        "settings-theme-appearance-card",
    ),
    (
        "settings-section-palette",
        Some("settings-theme-fixed-card-desc"),
        "settings-theme-fixed-card",
    ),
    (
        "settings-theme-designer-label",
        Some("settings-theme-profile-desc"),
        "settings-theme-designer-label",
    ),
    (
        "settings-section-appearance",
        Some("settings-theme-matugen-card-desc"),
        "settings-theme-matugen-card",
    ),
    ("settings-section-imported-result", None, "settings-theme-external-card"),
    (
        "settings-section-engine-note",
        Some("settings-theme-native-card-desc"),
        "settings-theme-native-card",
    ),
    ("settings-section-engine-options", None, "settings-theme-wallust-card"),
];

fn compose_theme(
    cfg: &dyn SettingsSource,
    backends: &[String],
    apps: Option<&crate::contracts::daemon::AppThemesResult>,
) -> Vec<(Card, Vec<Row>)> {
    let mut out = Vec::new();
    let mut theme = collect(cfg, |builder| tab_theme(builder, backends));
    let mut matugen = collect(cfg, tab_matugen);
    section(
        &mut out,
        crate::i18n::text("settings-app-themes-title"),
        tr("settings-app-themes-desc"),
        super::app_themes::rows(cfg, apps),
    );
    for (title, subtitle, card) in THEME_SECTIONS {
        let subtitle = subtitle.map_or("", tr);
        section(
            &mut out,
            crate::i18n::text(title),
            subtitle,
            take_card(&mut theme, crate::i18n::text(card)),
        );
    }
    section(
        &mut out,
        crate::i18n::text("settings-section-matugen-command"),
        tr("settings-matugen-external-card-desc"),
        take_card(&mut matugen, crate::i18n::text("settings-matugen-external-card")),
    );
    section(
        &mut out,
        crate::i18n::text("settings-section-theme-outputs"),
        tr("settings-matugen-integrations-card-desc"),
        vec![details(
            tr("settings-app-themes-custom"),
            tr("settings-app-themes-custom-desc"),
            "app-themes.custom",
            "",
            take_card(&mut matugen, crate::i18n::text("settings-matugen-integrations-card")),
        )],
    );
    out
}

fn compose_integrations(cfg: &dyn SettingsSource, sddm_themes: &[String]) -> Vec<(Card, Vec<Row>)> {
    let mut out = collect(cfg, tab_integrations);
    let (mut rows, advanced, _) = lock_parts(cfg);
    if !sddm_themes.is_empty() {
        let mut items: Vec<_> = sddm_themes
            .iter()
            .map(|theme| (ActionId::ChooseSddmTheme(Some(theme.clone())), theme.clone()))
            .collect();
        items.push((ActionId::ChooseSddmTheme(None), tr("settings-keybind-capture-cancel").into()));
        rows.push(Row::of(
            tr("settings-lockscreens-choose-theme").into(),
            tr("settings-lockscreens-choose-theme-desc").into(),
            Control::ActionChips { items },
        ));
    }
    rows.push(details(
        tr("settings-lockscreens-advanced-label"),
        "",
        keys::lock_screens::MODE,
        tr("settings-lockscreens-source-label"),
        advanced,
    ));
    section(&mut out, crate::i18n::text("settings-section-lockscreens"), "", rows);
    out
}

fn lock_parts(cfg: &dyn SettingsSource) -> (Vec<Row>, Vec<Row>, Vec<Row>) {
    let mut lockscreens = collect(cfg, tab_lockscreens);
    let mut rows = take_card(&mut lockscreens, crate::i18n::text("settings-section-lockscreens"));
    let mut advanced =
        take_card(&mut lockscreens, crate::i18n::text("settings-lockscreens-advanced-label"));
    let mut plasma = take_control_paths(&mut advanced, &PLASMA_LOCK);
    let at = rows.iter().position(|row| {
        matches!(
            row.control,
            Control::ActionBtn { id: ActionId::SetLockScreen(LockScreenTarget::Plasma, _), .. }
        )
    });
    plasma.extend(at.map(|at| rows.remove(at)));
    (rows, advanced, plasma)
}

pub(in crate::frontend::settings) fn plasma_lock_rows(cfg: &dyn SettingsSource) -> Vec<Row> {
    lock_parts(cfg).2
}

pub(in crate::frontend::settings) fn niri_rows(
    cfg: &dyn SettingsSource,
    themes: &[String],
) -> Vec<Row> {
    collect(cfg, |builder| tab_niri(builder, themes))
        .into_iter()
        .flat_map(|(_, rows)| rows)
        .collect()
}

pub(in crate::frontend::settings) fn asleep_rows(
    cfg: &dyn SettingsSource,
    themes: &[String],
) -> Vec<Row> {
    let mut cards = collect(cfg, tab_transitions_asleep);
    cards.extend(collect(cfg, tab_paper_asleep));
    cards.extend(collect(cfg, tab_wallpaper_engine));
    cards.extend(collect(cfg, |builder| tab_lock_asleep(builder, themes)));
    cards.into_iter().flat_map(|(_, rows)| rows).collect()
}

pub(in crate::frontend::settings) fn layout_asleep_rows(cfg: &dyn SettingsSource) -> Vec<Row> {
    let mut cards = collect(cfg, tab_position);
    for mode in Mode::ALL {
        cards.extend(collect(cfg, |builder| selector_cards(builder, mode.as_key())));
    }
    cards.extend(collect(cfg, selector_gated));
    cards.into_iter().flat_map(|(_, rows)| rows).collect()
}

pub(in crate::frontend::settings) fn colours_asleep_rows(
    cfg: &dyn SettingsSource,
    backends: &[String],
) -> Vec<Row> {
    collect(cfg, |builder| tab_theme_asleep(builder, backends))
        .into_iter()
        .flat_map(|(_, rows)| rows)
        .collect()
}

#[cfg(test)]
mod inventory_tests;
#[cfg(test)]
mod ownership_tests;
