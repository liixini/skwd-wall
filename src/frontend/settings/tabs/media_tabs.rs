use crate::contracts::settings::keys;
use crate::i18n::tr;

use super::super::tables::{AWWW_FILTERS, AWWW_TYPES, ENGINES, FILL_MODES, VIDEO_ENGINES};
use super::{ActionId, Builder};

pub(super) fn tab_paper(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(tr("settings-paper-engine-card"), tr("settings-paper-engine-card-desc"));
    builder.chips(
        tr("settings-paper-engine-label"),
        tr("settings-paper-engine-desc"),
        keys::paper::ENGINE,
        &ENGINES,
    );
    let fill_modes = FILL_MODES.map(|key| {
        let label = match key {
            "fit" => "settings-paper-fill-fit",
            "stretch" => "settings-paper-fill-stretch",
            "center" => "settings-paper-fill-center",
            "tile" => "settings-paper-fill-tile",
            "span" => "settings-paper-fill-span",
            _ => "settings-paper-fill-fill",
        };
        (key, tr(label))
    });
    builder.chips(
        tr("settings-paper-fill-mode-label"),
        tr("settings-paper-fill-mode-desc"),
        keys::display::FILL_MODE,
        &fill_modes,
    );
    builder
        .card(tr("settings-paper-video-engine-card"), tr("settings-paper-video-engine-card-desc"));
    let video_engines = VIDEO_ENGINES.map(|key| {
        let label = match key {
            "tinier" => "settings-paper-video-engine-tinier",
            _ => "settings-paper-video-engine-vulkan",
        };
        (key, tr(label))
    });
    builder.chips_with_disabled(
        tr("settings-paper-video-engine-label"),
        tr("settings-paper-video-engine-desc"),
        keys::paper::VIDEO_ENGINE,
        &video_engines,
        &["tinier"],
    );
    builder.dropdown(
        tr("settings-paper-layer-label"),
        tr("settings-paper-layer-desc"),
        keys::paper::WALLPAPER_LAYER,
        &[
            ("bottom", tr("settings-paper-layer-bottom")),
            ("background", tr("settings-paper-layer-background")),
            ("top", tr("settings-paper-layer-top")),
            ("overlay", tr("settings-paper-layer-overlay")),
        ],
    );
    builder.toggle(
        tr("settings-paper-overview-only-label"),
        tr("settings-paper-overview-only-desc"),
        keys::niri::OVERVIEW_ONLY_PLAYBACK,
    );
    builder.card(tr("settings-paper-performance-card"), tr("settings-paper-performance-card-desc"));
    builder.toggle(
        tr("settings-paper-multi-process-label"),
        tr("settings-paper-multi-process-desc"),
        keys::paper::VIDEO_MULTI_PROCESS,
    );
    builder.toggle(
        tr("settings-paper-performance-mode-label"),
        tr("settings-paper-performance-mode-desc"),
        keys::paper::PERFORMANCE_MODE,
    );
    builder.card(tr("settings-playback-pause-title"), tr("settings-playback-pause-desc"));
    builder.toggle(
        tr("settings-playback-process-enabled"),
        tr("settings-playback-process-desc"),
        keys::playback::PROCESS_ENABLED,
    );
    builder.text_field(
        tr("settings-playback-processes"),
        tr("settings-playback-processes-desc"),
        keys::playback::PROCESSES,
        "Overwatch.exe, mpv",
    );
    builder.action(
        tr("settings-playback-choose-process"),
        "",
        ActionId::ChooseRunningProcess,
        tr("settings-playback-choose-process"),
    );
    builder.toggle(
        tr("settings-playback-fullscreen"),
        tr("settings-playback-fullscreen-desc"),
        keys::playback::FULLSCREEN,
    );
    builder.toggle(
        tr("settings-playback-maximized"),
        tr("settings-playback-maximized-desc"),
        keys::playback::MAXIMIZED,
    );
    if cfg.is_niri() {
        builder.toggle(
            tr("settings-playback-full-width"),
            tr("settings-playback-full-width-desc"),
            keys::niri::FULL_WIDTH_PAUSE,
        );
    }
    builder.dropdown(
        tr("settings-playback-scope"),
        tr("settings-playback-scope-desc"),
        keys::playback::FULLSCREEN_SCOPE,
        &[("all", tr("settings-playback-all")), ("display", tr("settings-playback-display"))],
    );
    builder.num(
        tr("settings-playback-resume"),
        tr("settings-playback-resume-desc"),
        keys::playback::RESUME_DELAY,
        "s",
    );
    builder.num(
        tr("settings-paper-idle-pause-label"),
        tr("settings-paper-idle-pause-desc"),
        keys::paper::IDLE_PAUSE_SECONDS,
        "s",
    );
    if cfg.text(keys::paper::ENGINE) == "awww" {
        builder.card(tr("settings-paper-awww-card"), "");
        let awww_types = AWWW_TYPES.map(|key| {
            let label = match key {
                "simple" => "settings-paper-awww-type-simple",
                "fade" => "settings-paper-awww-type-fade",
                "wipe" => "settings-paper-awww-type-wipe",
                "wave" => "settings-paper-awww-type-wave",
                "grow" => "settings-paper-awww-type-grow",
                "center" => "settings-paper-awww-type-center",
                "outer" => "settings-paper-awww-type-outer",
                "left" => "settings-paper-awww-type-left",
                "right" => "settings-paper-awww-type-right",
                "top" => "settings-paper-awww-type-top",
                "bottom" => "settings-paper-awww-type-bottom",
                "any" => "settings-paper-awww-type-any",
                "random" => "settings-paper-awww-type-random",
                _ => "settings-paper-awww-type-none",
            };
            (key, tr(label))
        });
        builder.dropdown(
            tr("settings-paper-awww-type-label"),
            "",
            keys::paper::AWWW_TRANSITION_TYPE,
            &awww_types,
        );
        builder.num(
            tr("settings-paper-awww-duration-label"),
            "",
            keys::paper::AWWW_TRANSITION_DURATION_MS,
            "ms",
        );
        builder.num(tr("settings-paper-awww-fps-label"), "", keys::paper::AWWW_TRANSITION_FPS, "");
        builder.num(
            tr("settings-paper-awww-step-label"),
            tr("settings-paper-awww-step-desc"),
            keys::paper::AWWW_TRANSITION_STEP,
            "",
        );
        builder.num(
            tr("settings-paper-awww-angle-label"),
            "",
            keys::paper::AWWW_TRANSITION_ANGLE,
            "deg",
        );
        builder.num(
            tr("settings-paper-awww-wave-width-label"),
            "",
            keys::paper::AWWW_TRANSITION_WAVE_WIDTH,
            "",
        );
        builder.num(
            tr("settings-paper-awww-wave-height-label"),
            "",
            keys::paper::AWWW_TRANSITION_WAVE_HEIGHT,
            "",
        );
        builder.text_field(
            tr("settings-paper-awww-pos-label"),
            tr("settings-paper-awww-pos-desc"),
            keys::paper::AWWW_TRANSITION_POS,
            "center",
        );
        builder.toggle(
            tr("settings-paper-awww-invert-y-label"),
            tr("settings-paper-awww-invert-y-desc"),
            keys::paper::AWWW_INVERT_Y,
        );
        builder.text_field(
            tr("settings-paper-awww-bezier-label"),
            tr("settings-paper-awww-bezier-desc"),
            keys::paper::AWWW_TRANSITION_BEZIER,
            "0.0,0.0,1.0,1.0",
        );
        builder.chips(
            tr("settings-paper-awww-filter-label"),
            tr("settings-paper-awww-filter-desc"),
            keys::paper::AWWW_FILTER,
            &AWWW_FILTERS,
        );
    }
}

pub(super) fn tab_wallhaven(builder: &mut Builder<'_>) {
    super::source_defaults::wallhaven(builder);
    builder.card(tr("settings-wallhaven-grid-card"), "");
    builder.num(
        tr("settings-wallhaven-columns-label"),
        tr("settings-wallhaven-columns-desc"),
        keys::selector::WALLHAVEN_COLUMNS,
        "",
    );
    builder.num(
        tr("settings-wallhaven-rows-label"),
        tr("settings-wallhaven-rows-desc"),
        keys::selector::WALLHAVEN_ROWS,
        "",
    );
    builder.card(tr("settings-wallhaven-thumb-card"), "");
    builder.num(
        tr("settings-wallhaven-width-label"),
        tr("settings-wallhaven-width-desc"),
        keys::selector::WALLHAVEN_THUMB_WIDTH,
        "px",
    );
    builder.num(
        tr("settings-wallhaven-height-label"),
        tr("settings-wallhaven-height-desc"),
        keys::selector::WALLHAVEN_THUMB_HEIGHT,
        "px",
    );
    builder.card(tr("settings-wallhaven-api-card"), "");
    builder.text_field(
        tr("settings-wallhaven-api-key-label"),
        tr("settings-wallhaven-api-key-desc"),
        keys::wallhaven::API_KEY,
        tr("settings-wallhaven-api-key-placeholder"),
    );
    builder.text_field(
        tr("settings-wallhaven-username-label"),
        tr("settings-wallhaven-username-desc"),
        keys::wallhaven::USERNAME,
        tr("settings-wallhaven-username-placeholder"),
    );
}

pub(super) fn tab_steam(builder: &mut Builder<'_>) {
    super::source_defaults::steam(builder);
    builder.card(tr("settings-steam-grid-card"), "");
    builder.num(
        tr("settings-steam-columns-label"),
        tr("settings-steam-columns-desc"),
        keys::selector::STEAM_COLUMNS,
        "",
    );
    builder.num(
        tr("settings-steam-rows-label"),
        tr("settings-steam-rows-desc"),
        keys::selector::STEAM_ROWS,
        "",
    );
    builder.card(tr("settings-steam-thumb-card"), "");
    builder.num(
        tr("settings-steam-width-label"),
        tr("settings-steam-width-desc"),
        keys::selector::STEAM_THUMB_WIDTH,
        "px",
    );
    builder.num(
        tr("settings-steam-height-label"),
        tr("settings-steam-height-desc"),
        keys::selector::STEAM_THUMB_HEIGHT,
        "px",
    );
    builder.card(tr("settings-steam-backend-card"), "");
    builder.dropdown(
        tr("settings-steam-backend-label"),
        tr("settings-steam-backend-desc"),
        keys::steam::BACKEND,
        &[
            ("steam", tr("settings-steam-backend-steam")),
            ("steamcmd", tr("settings-steam-backend-steamcmd")),
        ],
    );
    builder.text_field(
        tr("settings-steam-username-label"),
        tr("settings-steam-username-desc"),
        keys::steam::USERNAME,
        tr("settings-steam-username-placeholder"),
    );
    builder.text_field(
        tr("settings-steam-api-key-label"),
        tr("settings-steam-api-key-desc"),
        keys::steam::API_KEY,
        tr("settings-steam-api-key-placeholder"),
    );
    builder.card(tr("settings-steam-paths-card"), tr("settings-steam-paths-card-desc"));
    builder.text_field(
        tr("settings-steam-workshop-dir-label"),
        tr("settings-steam-workshop-dir-desc"),
        keys::paths::STEAM_WORKSHOP,
        tr("settings-steam-workshop-dir-placeholder"),
    );
    builder.text_field(
        tr("settings-steam-assets-dir-label"),
        tr("settings-steam-assets-dir-desc"),
        keys::paths::STEAM_WE_ASSETS,
        tr("settings-steam-assets-dir-placeholder"),
    );
    builder.text_field(
        tr("settings-steam-steam-dir-label"),
        tr("settings-steam-steam-dir-desc"),
        keys::paths::STEAM,
        tr("settings-steam-steam-dir-placeholder"),
    );
}

pub(super) fn tab_wallpaper_engine(builder: &mut Builder<'_>) {
    builder.card(
        tr("settings-wallpaper-engine-rendering-card"),
        tr("settings-wallpaper-engine-rendering-card-desc"),
    );
    builder.num(
        tr("settings-wallpaper-engine-fps-label"),
        tr("settings-wallpaper-engine-fps-desc"),
        keys::we_render::FPS,
        "",
    );
    builder.dropdown(
        tr("settings-wallpaper-engine-scaling-label"),
        tr("settings-wallpaper-engine-scaling-desc"),
        keys::we_render::SCALING,
        &[
            ("default", tr("settings-wallpaper-engine-scaling-default")),
            ("fill", tr("settings-wallpaper-engine-scaling-fill")),
            ("fit", tr("settings-wallpaper-engine-scaling-fit")),
            ("stretch", tr("settings-wallpaper-engine-scaling-stretch")),
            ("center", tr("settings-wallpaper-engine-scaling-center")),
            ("tile", tr("settings-wallpaper-engine-scaling-tile")),
            ("span", tr("settings-wallpaper-engine-scaling-span")),
        ],
    );
    builder.card(tr("settings-wallpaper-engine-effects-card"), "");
    builder.toggle(
        tr("settings-wallpaper-engine-particles-label"),
        tr("settings-wallpaper-engine-particles-desc"),
        keys::we_render::DISABLE_PARTICLES,
    );
}

pub(super) fn tab_sources(builder: &mut Builder<'_>) {
    builder.card(tr("settings-sources-unsplash-card"), tr("settings-sources-unsplash-card-desc"));
    super::source_defaults::apply_button(builder, keys::sources::UNSPLASH_SHOW_APPLY_BUTTON);
    builder.toggle(
        tr("settings-sources-unsplash-enable-label"),
        tr("settings-sources-unsplash-enable-desc"),
        keys::sources::UNSPLASH_ENABLED,
    );
    builder.text_field(
        tr("settings-sources-unsplash-key-label"),
        tr("settings-sources-unsplash-key-desc"),
        keys::sources::UNSPLASH_ACCESS_KEY,
        tr("settings-sources-unsplash-key-placeholder"),
    );
    builder.card(tr("settings-sources-pexels-card"), tr("settings-sources-pexels-card-desc"));
    super::source_defaults::apply_button(builder, keys::sources::PEXELS_SHOW_APPLY_BUTTON);
    builder.toggle(
        tr("settings-sources-pexels-enable-label"),
        tr("settings-sources-pexels-enable-desc"),
        keys::sources::PEXELS_ENABLED,
    );
    builder.text_field(
        tr("settings-sources-pexels-key-label"),
        tr("settings-sources-pexels-key-desc"),
        keys::sources::PEXELS_API_KEY,
        tr("settings-sources-pexels-key-placeholder"),
    );
    builder.card(tr("settings-sources-youtube-card"), tr("settings-sources-youtube-card-desc"));
    super::source_defaults::apply_button(builder, keys::sources::YOUTUBE_SHOW_APPLY_BUTTON);
    builder.toggle(
        tr("settings-sources-youtube-enable-label"),
        tr("settings-sources-youtube-enable-desc"),
        keys::sources::YOUTUBE_ENABLED,
    );
    builder.num(
        tr("settings-sources-youtube-height-label"),
        tr("settings-sources-youtube-height-desc"),
        keys::sources::YOUTUBE_MAX_HEIGHT,
        "px",
    );
    builder.num(
        tr("settings-sources-youtube-clip-label"),
        tr("settings-sources-youtube-clip-desc"),
        keys::sources::YOUTUBE_MAX_MINUTES,
        "min",
    );
    builder.card(tr("settings-sources-bing-card"), tr("settings-sources-bing-card-desc"));
    builder.toggle(
        tr("settings-sources-bing-enable-label"),
        tr("settings-sources-bing-enable-desc"),
        keys::sources::BING_ENABLED,
    );
    builder.text_field(
        tr("settings-sources-bing-market-label"),
        tr("settings-sources-bing-market-desc"),
        keys::sources::BING_MARKET,
        "en-US",
    );
}

pub(super) fn tab_search(builder: &mut Builder<'_>) {
    builder.card(tr("settings-tagging-search-card"), tr("settings-tagging-search-card-desc"));
    builder.dropdown_cur(
        tr("settings-tagging-search-mode-label"),
        tr("settings-tagging-search-mode-desc"),
        keys::tagging::DEFAULT_SEARCH_MODE,
        &[
            ("tags", tr("settings-tagging-mode-tags")),
            ("describe", tr("settings-tagging-mode-describe")),
        ],
        match builder.cfg.text(keys::tagging::DEFAULT_SEARCH_MODE).as_str() {
            "describe" => String::from("describe"),
            _ => String::from("tags"),
        },
    );
    builder.info(
        tr("settings-tagging-describe-info-label"),
        tr("settings-tagging-describe-info-desc"),
    );
    builder.card(tr("settings-tagging-models-card"), tr("settings-tagging-models-card-desc"));
    builder.action(
        tr("settings-tagging-model-import-label"),
        tr("settings-tagging-model-import-desc"),
        ActionId::ImportSemanticModel,
        tr("settings-tagging-model-import-action"),
    );
    let count = builder.cfg.array_len(keys::semantic::MODELS);
    let mut options = vec![(String::new(), tr("settings-tagging-model-default").to_string())];
    for idx in 0..count {
        let base = format!("{}.{idx}", keys::semantic::MODELS);
        let name = builder.cfg.text(&format!("{base}.name"));
        let manifest = builder.cfg.text(&format!("{base}.manifest"));
        if !manifest.trim().is_empty() {
            let label = if name.trim().is_empty() {
                crate::i18n::tr_args!("settings-tagging-model-numbered", index => idx + 1)
            } else {
                name.clone()
            };
            options.push((manifest, label));
        }
    }
    let selected = builder.cfg.text(keys::semantic::MANIFEST);
    if !selected.is_empty() && !options.iter().any(|(manifest, _)| manifest == &selected) {
        options.push((
            selected.clone(),
            crate::i18n::tr_args!("settings-tagging-model-missing", path => &selected),
        ));
    }
    builder.dynamic_dropdown(
        tr("settings-tagging-model-active-label"),
        tr("settings-tagging-model-active-desc"),
        keys::semantic::MANIFEST,
        options,
    );
    builder.chips(
        tr("settings-tagging-index-profile-label"),
        tr("settings-tagging-index-profile-desc"),
        keys::semantic::INDEX_PROFILE,
        &[
            ("full", tr("settings-tagging-index-profile-full")),
            ("multiview", tr("settings-tagging-index-profile-multiview")),
        ],
    );
    builder.info(tr("settings-tagging-model-index-label"), tr("settings-tagging-model-index-desc"));
    for idx in 0..count {
        let base = format!("{}.{idx}", keys::semantic::MODELS);
        let name_path = format!("{base}.name");
        let manifest_path = format!("{base}.manifest");
        let name = builder.cfg.text(&name_path);
        let manifest = builder.cfg.text(&manifest_path);
        let title = if name.trim().is_empty() {
            crate::i18n::tr_args!("settings-tagging-model-numbered", index => idx + 1)
        } else {
            name
        };
        let summary = if manifest.trim().is_empty() {
            tr("settings-tagging-model-path-needed").to_string()
        } else {
            manifest
        };
        builder.details(
            &title,
            tr("settings-tagging-model-entry-desc"),
            base,
            summary,
            |builder| {
                builder.text_field(
                    tr("settings-tagging-model-name-label"),
                    tr("settings-tagging-model-name-desc"),
                    &name_path,
                    tr("settings-tagging-model-name-placeholder"),
                );
                builder.text_field(
                    tr("settings-tagging-model-manifest-label"),
                    tr("settings-tagging-model-manifest-desc"),
                    &manifest_path,
                    "/path/to/semantic-pack.json",
                );
                builder.action(
                    "",
                    tr("settings-tagging-model-forget-desc"),
                    ActionId::RemoveSemanticModel(idx as u16),
                    tr("settings-tagging-model-forget-action"),
                );
            },
        );
    }
    builder.action(
        tr("settings-tagging-model-manual-label"),
        tr("settings-tagging-model-manual-desc"),
        ActionId::AddSemanticModel,
        tr("settings-tagging-model-manual-action"),
    );
}
