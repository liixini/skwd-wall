#[allow(clippy::wildcard_imports)]
use super::*;
use crate::i18n::{settings_keybind_conflict, tr, tr_args};

pub(super) fn tab_general(
    builder: &mut Builder<'_>,
    themes: &[String],
    detected_outputs: &[String],
) {
    builder.card(tr("settings-general-general-card"), "");
    let configured = builder.cfg.text(keys::system::MONITOR);
    let mut outputs = detected_outputs.to_vec();
    outputs.sort();
    outputs.dedup();
    let mut choices = vec![(String::new(), String::from(tr("settings-general-monitor-focused")))];
    choices.extend(outputs.iter().cloned().map(|output| (output.clone(), output)));
    if !configured.is_empty() && !outputs.contains(&configured) {
        choices.push((
            configured.clone(),
            tr_args!("settings-general-monitor-unavailable", output => &configured),
        ));
    }
    builder.dynamic_chips(
        tr("settings-general-monitor-label"),
        tr("settings-general-monitor-desc"),
        keys::system::MONITOR,
        choices,
    );
    builder.num_setting(
        tr("settings-general-ui-scale-label"),
        tr("settings-general-ui-scale-desc"),
        crate::contracts::settings::schema::setting::general::UI_SCALE,
        "x",
    );
    builder.card(tr("settings-general-features-card"), tr("settings-general-features-card-desc"));
    builder.toggle(
        tr("settings-general-feature-steam-label"),
        tr("settings-general-feature-steam-desc"),
        keys::features::STEAM,
    );
    builder.toggle(
        tr("settings-general-feature-wallhaven-label"),
        tr("settings-general-feature-wallhaven-desc"),
        keys::features::WALLHAVEN,
    );
    builder.card(tr("settings-general-audio-card"), tr("settings-general-audio-card-desc"));
    builder.toggle(
        tr("settings-general-mute-label"),
        tr("settings-general-mute-desc"),
        keys::wallpaper::MUTE,
    );
    builder.num(
        tr("settings-general-volume-label"),
        tr("settings-general-volume-desc"),
        keys::wallpaper::VOLUME,
        "%",
    );
    builder.card(
        tr("settings-general-auto-recolour-card"),
        tr("settings-general-auto-recolour-card-desc"),
    );
    builder.toggle(
        tr("settings-general-auto-recolour-label"),
        tr("settings-general-auto-recolour-desc"),
        keys::effects::AUTO_RECOLOR,
    );
    builder.theme_dropdown(
        tr("settings-general-recolour-theme-label"),
        tr("settings-general-recolour-theme-desc"),
        keys::effects::AUTO_THEME,
        themes,
    );
    builder.card(tr("settings-general-behaviour-card"), "");
    builder.toggle(
        tr("settings-general-close-on-selection-label"),
        tr("settings-general-close-on-selection-desc"),
        keys::general::CLOSE_ON_SELECTION,
    );
    builder.toggle(
        tr("settings-general-notify-label"),
        tr("settings-general-notify-desc"),
        keys::general::NOTIFY_ON_WALLPAPER_CHANGE,
    );
    builder.toggle(
        tr("settings-general-restore-label"),
        tr("settings-general-restore-desc"),
        keys::system::RESTORE_ON_STARTUP,
    );
    builder.card(tr("settings-general-random-card"), "");
    builder.toggle(
        tr("settings-general-rotate-label"),
        tr("settings-general-rotate-desc"),
        keys::general::RANDOM_ROTATE,
    );
    builder.num(
        tr("settings-general-interval-label"),
        tr("settings-general-interval-desc"),
        keys::general::RANDOM_INTERVAL,
        "s",
    );
    builder.toggle(
        tr("settings-general-include-images-label"),
        tr("settings-general-include-images-desc"),
        keys::general::RANDOM_INCLUDE_STATIC,
    );
    builder.toggle(
        tr("settings-general-include-video-label"),
        tr("settings-general-include-video-desc"),
        keys::general::RANDOM_INCLUDE_VIDEO,
    );
    builder.toggle(
        tr("settings-general-include-we-label"),
        tr("settings-general-include-we-desc"),
        keys::general::RANDOM_INCLUDE_WE,
    );
    builder.toggle(
        tr("settings-general-favourites-only-label"),
        tr("settings-general-favourites-only-desc"),
        keys::general::RANDOM_INCLUDE_FAVOURITES,
    );
}

pub(super) fn tab_schedule(builder: &mut Builder<'_>) {
    builder.card(tr("settings-schedule-schedule-card"), tr("settings-schedule-schedule-card-desc"));
    builder.toggle(
        tr("settings-schedule-enable-label"),
        tr("settings-schedule-enable-desc"),
        keys::schedule::ENABLED,
    );
    builder.toggle(
        tr("settings-schedule-apply-on-start-label"),
        tr("settings-schedule-apply-on-start-desc"),
        keys::schedule::APPLY_ON_START,
    );
    builder.action(
        tr("settings-schedule-editor-label"),
        tr("settings-schedule-editor-desc"),
        ActionId::OpenScheduleEditor,
        tr("settings-schedule-editor-action"),
    );
    builder.card(tr("settings-schedule-location-card"), tr("settings-schedule-location-card-desc"));
    builder.text_field(
        tr("settings-schedule-latitude-label"),
        tr("settings-schedule-latitude-desc"),
        keys::schedule::LATITUDE,
        "59.33",
    );
    builder.text_field(
        tr("settings-schedule-longitude-label"),
        tr("settings-schedule-longitude-desc"),
        keys::schedule::LONGITUDE,
        "18.06",
    );
}

pub(super) fn tab_paths(builder: &mut Builder<'_>) {
    builder.card(tr("settings-paths-directories-card"), tr("settings-paths-directories-card-desc"));
    builder.text_field(
        tr("settings-paths-wallpaper-dir-label"),
        tr("settings-paths-wallpaper-dir-desc"),
        keys::paths::WALLPAPER,
        "~/Pictures/Wallpapers",
    );
    builder.text_field(
        tr("settings-paths-video-dir-label"),
        tr("settings-paths-video-dir-desc"),
        keys::paths::VIDEO_WALLPAPER,
        tr("settings-paths-video-dir-placeholder"),
    );
}

pub(super) fn tab_performance(builder: &mut Builder<'_>) {
    let battery_saver = builder.cfg.flag(keys::performance::BATTERY_SAVER);
    let power_source = match (builder.cfg.on_battery_power(), battery_saver) {
        (true, true) => tr("settings-performance-power-battery-limited"),
        (true, false) => tr("settings-performance-power-battery-unlimited"),
        (false, _) => tr("settings-performance-power-external"),
    };
    builder.card(tr("settings-performance-power-card"), tr("settings-performance-power-card-desc"));
    builder.info(tr("settings-performance-power-source-label"), power_source);
    let devices = builder.cfg.graphics_devices();
    let mut device_options =
        vec![(String::from("auto"), tr("settings-performance-device-auto").to_string())];
    for device in &devices {
        let name = if devices.iter().filter(|candidate| candidate.name == device.name).count() > 1 {
            format!("{} ({})", device.name, device.id)
        } else {
            device.name.clone()
        };
        device_options.push((device.id.clone(), name));
    }
    let current = builder.cfg.text(keys::performance::GPU_DEVICE);
    if !current.is_empty()
        && current != "auto"
        && !devices.iter().any(|device| device.id == current)
    {
        device_options.push((current, tr("settings-performance-device-unavailable").to_string()));
    }
    let options = device_options
        .iter()
        .map(|(value, label)| (value.as_str(), label.as_str()))
        .collect::<Vec<_>>();
    builder.dropdown(
        tr("settings-performance-device-label"),
        tr("settings-performance-device-desc"),
        keys::performance::GPU_DEVICE,
        &options,
    );
    builder.toggle(
        tr("settings-performance-battery-saver-label"),
        tr("settings-performance-battery-saver-desc"),
        keys::performance::BATTERY_SAVER,
    );
    builder.dropdown(
        tr("settings-performance-gpu-label"),
        tr("settings-performance-gpu-desc"),
        keys::performance::GPU_PREFERENCE,
        &[
            ("auto", tr("settings-performance-gpu-auto")),
            ("low", tr("settings-performance-gpu-low")),
            ("high", tr("settings-performance-gpu-high")),
            ("none", tr("settings-performance-gpu-none")),
        ],
    );
    builder.num(
        tr("settings-performance-battery-fps-label"),
        tr("settings-performance-battery-fps-desc"),
        keys::performance::BATTERY_FPS,
        "fps",
    );
    builder.num(
        tr("settings-performance-battery-idle-label"),
        tr("settings-performance-battery-idle-desc"),
        keys::performance::BATTERY_VIDEO_IDLE_SECONDS,
        "s",
    );
    builder.toggle(
        tr("settings-performance-battery-wallpaper-label"),
        tr("settings-performance-battery-wallpaper-desc"),
        keys::performance::BATTERY_WALLPAPER_PERFORMANCE,
    );
    builder.card(
        tr("settings-performance-rendering-card"),
        tr("settings-performance-rendering-card-desc"),
    );
    builder.info(
        tr("settings-performance-we-renderer-label"),
        tr("settings-performance-we-renderer-value"),
    );
    builder.num(
        tr("settings-performance-max-fps-label"),
        tr("settings-performance-max-fps-desc"),
        keys::general::MAX_FPS,
        "fps",
    );
    builder.num(
        tr("settings-performance-preview-fps-label"),
        tr("settings-performance-preview-fps-desc"),
        keys::transition::PREVIEW_FPS,
        "fps",
    );
    builder.toggle(
        tr("settings-performance-hover-previews-label"),
        tr("settings-performance-hover-previews-desc"),
        keys::video_preview::ENABLED,
    );
    builder.card(
        tr("settings-performance-image-opt-card"),
        tr("settings-performance-image-opt-card-desc"),
    );
    builder.toggle(
        tr("settings-performance-auto-optimize-label"),
        tr("settings-performance-auto-optimize-desc"),
        keys::performance::AUTO_OPTIMIZE_IMAGES,
    );
    builder.dropdown(
        tr("settings-performance-quality-label"),
        tr("settings-performance-quality-desc"),
        keys::performance::IMAGE_OPTIMIZE_PRESET,
        &[
            ("light", tr("settings-performance-quality-light")),
            ("balanced", tr("settings-performance-quality-balanced")),
            ("quality", tr("settings-performance-quality-quality")),
        ],
    );
    builder.dropdown(
        tr("settings-performance-resolution-label"),
        tr("settings-performance-resolution-desc"),
        keys::performance::IMAGE_OPTIMIZE_RESOLUTION,
        &[("1080p", "1080p"), ("2k", "2K"), ("4k", "4K")],
    );
    builder.action(
        tr("settings-performance-optimize-all-label"),
        tr("settings-performance-optimize-all-desc"),
        ActionId::OptimizeImages,
        tr("settings-performance-optimize-action"),
    );
    builder.card(tr("settings-performance-trash-card"), tr("settings-performance-trash-card-desc"));
    builder.num(
        tr("settings-performance-retention-label"),
        tr("settings-performance-retention-desc"),
        keys::performance::IMAGE_TRASH_DAYS,
        tr("settings-performance-retention-unit"),
    );
    builder.toggle(
        tr("settings-performance-auto-delete-label"),
        tr("settings-performance-auto-delete-desc"),
        keys::performance::AUTO_DELETE_IMAGE_TRASH,
    );
    builder.card(tr("settings-performance-thumbs-card"), "");
    builder.num(
        tr("settings-performance-thumb-jobs-label"),
        tr("settings-performance-thumb-jobs-desc"),
        keys::performance::MAX_THUMB_JOBS,
        "",
    );
    builder.action(
        tr("settings-performance-clear-cache-label"),
        tr("settings-performance-clear-cache-desc"),
        ActionId::ClearCache,
        tr("settings-performance-clear-cache-action"),
    );
    builder.action(
        tr("settings-performance-recompute-label"),
        tr("settings-performance-recompute-desc"),
        ActionId::RecomputeColors,
        tr("settings-performance-recompute-action"),
    );
    builder.card(
        tr("settings-performance-diagnostics-card"),
        tr("settings-performance-diagnostics-card-desc"),
    );
    builder.action(
        tr("settings-performance-doctor-label"),
        tr("settings-performance-doctor-desc"),
        ActionId::RunDoctor,
        tr("settings-performance-doctor-action"),
    );
    builder.action(
        tr("settings-performance-bug-report-label"),
        tr("settings-performance-bug-report-desc"),
        ActionId::GenerateBugReport,
        tr("settings-performance-bug-report-action"),
    );
}

pub(super) fn tab_postprocessing(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(tr("settings-postprocessing-behaviour-card"), "");
    builder.toggle(
        tr("settings-postprocessing-pick-only-label"),
        tr("settings-postprocessing-pick-only-desc"),
        keys::system::PICK_ONLY_MODE,
    );
    builder.toggle(
        tr("settings-postprocessing-on-restore-label"),
        tr("settings-postprocessing-on-restore-desc"),
        keys::system::POST_PROCESS_ON_RESTORE,
    );
    builder.card(
        tr("settings-postprocessing-commands-card"),
        tr("settings-postprocessing-commands-card-desc"),
    );
    let post_types = POST_TYPES.map(|key| {
        let label = if key == crate::contracts::settings::wallpaper_kind::STATIC {
            "settings-postprocessing-type-image"
        } else if key == crate::contracts::settings::wallpaper_kind::VIDEO {
            "settings-postprocessing-type-video"
        } else if key == crate::contracts::settings::wallpaper_kind::WE {
            "settings-postprocessing-type-we"
        } else {
            "settings-postprocessing-type-all"
        };
        (key, tr(label))
    });
    let entries = cfg.array_len(keys::post_processing::LIST);
    for idx in 0..entries {
        builder.chips(
            tr("settings-postprocessing-runs-for-label"),
            "",
            &format!("postProcessing.{idx}.type"),
            &post_types,
        );
        let key = format!("postProcessing.{idx}.command");
        builder.row(
            &tr_args!("settings-postprocessing-command-label", index => idx + 1),
            "",
            Control::TextField {
                key: key.clone(),
                path: key,
                placeholder: tr("settings-postprocessing-command-placeholder"),
            },
        );
        builder.action(
            "",
            tr("settings-postprocessing-remove-desc"),
            ActionId::RemovePostCommand(idx as u16),
            tr("settings-postprocessing-remove-action"),
        );
    }
    builder.action(
        tr("settings-postprocessing-add-label"),
        tr("settings-postprocessing-add-desc"),
        ActionId::AddPostCommand,
        tr("settings-postprocessing-add-action"),
    );
}

pub(super) fn tab_keybinds(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(tr("settings-keybinds-custom-card"), tr("settings-keybinds-custom-card-desc"));
    let bindings = cfg.bindings();
    for descriptor in crate::contracts::picker::KEY_BINDINGS {
        builder.key_binding(
            tr(descriptor.title_key),
            descriptor.path,
            descriptor.action.default_binding(),
        );
    }
    for (first, second) in bindings.conflicts() {
        let title = |action| {
            crate::contracts::picker::KEY_BINDINGS
                .into_iter()
                .find(|entry| entry.action == action)
                .map_or("", |entry| tr(entry.title_key))
        };
        let msg = settings_keybind_conflict(title(first), title(second), &bindings.label(first));
        builder.row(tr("settings-keybinds-conflict-label"), &msg, Control::Static);
    }
    builder.card(tr("settings-keybinds-fixed-card"), tr("settings-keybinds-fixed-card-desc"));
    builder.row("Escape", tr("settings-keybinds-fixed-escape-desc"), Control::Static);
    builder.row(
        tr("settings-keybinds-mouse-scroll-label"),
        tr("settings-keybinds-mouse-scroll-desc"),
        Control::Static,
    );
    builder.row(
        tr("settings-keybinds-mouse-hover-label"),
        tr("settings-keybinds-mouse-hover-desc"),
        Control::Static,
    );
    builder.card(tr("settings-keybinds-reset-card"), "");
    builder.action(
        tr("settings-keybinds-reset-label"),
        tr("settings-keybinds-reset-desc"),
        ActionId::ResetKeybinds,
        tr("settings-keybinds-reset-action"),
    );
}

pub(super) fn tab_language(builder: &mut Builder<'_>) {
    builder.card(tr("settings-language-card"), tr("settings-language-card-desc"));
    let current = crate::i18n::language_choice(&builder.cfg.text(keys::general::LANGUAGE));
    builder.dropdown_cur(
        tr("settings-language-choice-label"),
        tr("settings-language-choice-desc"),
        keys::general::LANGUAGE,
        &[
            ("auto", tr("settings-language-system")),
            ("en-US", tr("settings-language-english")),
            ("sv-SE", tr("settings-language-swedish")),
            ("es-ES", tr("settings-language-spanish")),
        ],
        current.to_string(),
    );
}
