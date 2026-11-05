use paper_control::transitions::{
    CROP_RELAY, PAPER_HEART, PHOTO_CUTOUT, SLAT_TURN, STUDIO_TRANSITIONS, SUNLIT_ARCADE,
};

use crate::contracts::settings::drawing::Device;
use crate::contracts::settings::{SettingsSource, keys, saved_memory_mib, transitions};
use crate::i18n::{tr, tr_args};

use super::super::tables::{
    SHADER_FAMILIES, SHADER_FAMILY_KEY, SHADERS, shader_family, shaders_in_family,
};
use super::{ActionId, Builder, Control};

const DRAWING_SELECTED_ID: &str = "drawing:selected";
const DRAWING_PERCENT: f32 = 100.0;
const DRAWING_DURATION_UNIT: &str = "ms";
const DRAWING_PACK_SIZE: &str = "settings-drawing-pack-size";
const DRAWING_PROGRESS: &str = "settings-drawing-progress-value";
const DRAWING_VIEW_COUNT: &str = "settings-drawing-view-count";
pub(crate) const DRAWING_VIEW_LIMIT: &str = "settings-drawing-view-limit-desc";
pub(crate) const DRAWING_VIEW_EMPTY: &str = "settings-drawing-view-empty-desc";
pub(crate) const DRAWING_SKIPPED: &str = "settings-drawing-library-skipped";
pub(crate) const DRAWING_PACK_INSTALL: &str = "settings-drawing-pack-install";
const DRAWING_INKS: [(&str, &str); 2] = [
    (keys::drawing::INK_PLAIN, "settings-drawing-ink-style-plain"),
    (keys::drawing::INK_RAINBOW, "settings-drawing-ink-style-rainbow"),
];
const DRAWING_VIEW_PREPARE: &str = "settings-drawing-prepare-view-action";
const DRAWING_DEVICES: [(Device, &str); 4] = [
    (Device::Auto, "settings-drawing-device-auto"),
    (Device::Cuda, "settings-drawing-device-cuda"),
    (Device::Webgpu, "settings-drawing-device-webgpu"),
    (Device::Cpu, "settings-drawing-device-cpu"),
];
const DRAWING_MEMORY_GIB: [u64; 7] = [2, 4, 8, 12, 16, 24, 32];
const MIB_PER_GIB: u64 = 1024;
const DRAWING_MEMORY_SIZE: &str = "settings-drawing-memory-gib";
const DRAWING_MEMORY_SAVED: &str = "settings-drawing-memory-mib";
const DRAWING_STAGES: [(&str, &str); 8] = [
    ("checking", "settings-drawing-stage-checking"),
    ("ready", "settings-drawing-stage-ready"),
    ("starting", "settings-drawing-stage-starting"),
    ("source", "settings-drawing-stage-source"),
    ("lineart", "settings-drawing-stage-lineart"),
    ("depth", "settings-drawing-stage-depth"),
    ("subjects", "settings-drawing-stage-subjects"),
    ("complete", "settings-drawing-stage-complete"),
];

const RANDOM_SHADER: &str = "random";
const STUDIO_FAMILY: &str = transitions::Family::Studio.key();
const STUDIO_LABEL: &str = "settings-transitions-family-studio";
const STUDIO_DESC: &str = "settings-transitions-family-studio-desc";
const PRESET_TIMING_DESC: &str = "settings-transitions-preset-timing-desc";
const PRESET_LABELS: [(&str, &str); STUDIO_TRANSITIONS.len()] = [
    (PHOTO_CUTOUT, "settings-transitions-effect-photo-cutout"),
    (SLAT_TURN, "settings-transitions-effect-slat-turn"),
    (CROP_RELAY, "settings-transitions-effect-crop-relay"),
    (SUNLIT_ARCADE, "settings-transitions-effect-sunlit-arcade"),
    (PAPER_HEART, "settings-transitions-effect-paper-heart"),
];

fn family_label_key(family: &str) -> &'static str {
    match family {
        "fade" => "settings-transitions-family-fade",
        "wipe" => "settings-transitions-family-wipe",
        "warp" => "settings-transitions-family-warp",
        "break" => "settings-transitions-family-break",
        "sand" => "settings-transitions-family-sand",
        STUDIO_FAMILY => STUDIO_LABEL,
        _ => "settings-transitions-family-random",
    }
}

fn family_desc_key(family: &str) -> &'static str {
    match family {
        "fade" => "settings-transitions-family-fade-desc",
        "wipe" => "settings-transitions-family-wipe-desc",
        "warp" => "settings-transitions-family-warp-desc",
        "break" => "settings-transitions-family-break-desc",
        "sand" => "settings-transitions-family-sand-desc",
        STUDIO_FAMILY => STUDIO_DESC,
        _ => "settings-transitions-family-random-desc",
    }
}

pub(super) fn pretty(shader: &str) -> String {
    if let Some((_, key)) = PRESET_LABELS.iter().find(|(id, _)| *id == shader) {
        return tr(key).to_string();
    }
    let mut out = String::with_capacity(shader.len());
    for (idx, word) in shader.split('-').enumerate() {
        if idx > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

fn shader_scope_path(shader: &str) -> String {
    format!("{}.{shader}", keys::transition::SHADER_SCOPES)
}

fn shader_scope(cfg: &dyn SettingsSource, shader: &str) -> String {
    let scope = cfg.text(&shader_scope_path(shader));
    if matches!(scope.as_str(), "all" | "primary") {
        return scope;
    }
    if shader.starts_with("sand-") && cfg.text(keys::transition::SAND_SCOPE) == "primary" {
        return String::from("primary");
    }
    String::from("all")
}

pub(super) fn tab_transitions(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    let current = cfg.text(keys::transition::SHADER);
    let current = if current.is_empty() { String::from("random") } else { current };
    let family = shader_family(&current);

    builder.card(
        crate::i18n::text("settings-transitions-transitions-card"),
        tr("settings-transitions-transitions-card-desc"),
    );
    builder.toggle(
        tr("settings-transitions-enable-label"),
        tr("settings-transitions-enable-desc"),
        keys::transition::ENABLED,
    );
    let preset = transitions::preset_duration_ms(&current);
    if let Some(duration) = preset {
        preset_timing(builder, duration);
    }
    if preset.is_none() || !cfg.flag_default_true(keys::transition::PRESET_TIMING) {
        duration(builder);
    }

    builder.num(
        tr("settings-transitions-rate-label"),
        tr("settings-transitions-rate-desc"),
        keys::transition::FPS,
        "fps",
    );

    let family_opts: Vec<(String, String)> = SHADER_FAMILIES
        .iter()
        .map(|(key, _, _)| ((*key).to_string(), tr(family_label_key(key)).to_string()))
        .collect();
    builder.row(
        tr("settings-transitions-family-label"),
        tr(family_desc_key(family)),
        Control::Chips {
            path: String::from(SHADER_FAMILY_KEY),
            options: family_opts,
            current: family.to_string(),
            disabled: Vec::new(),
            background: None,
        },
    );
    if family != "random" {
        shader(builder, family, current);
    }

    builder.card(
        crate::i18n::text("settings-transitions-preview-card"),
        tr("settings-transitions-preview-card-desc"),
    );
    builder.toggle(
        tr("settings-transitions-show-preview-label"),
        tr("settings-transitions-show-preview-desc"),
        keys::transition::PREVIEW,
    );
    if cfg.flag_default_true(keys::transition::PREVIEW) {
        preview_rate(builder);
        builder.row("", "", Control::Preview);
    }

    builder.card(
        crate::i18n::text("settings-transitions-sand-card"),
        tr("settings-transitions-sand-card-desc"),
    );
    builder.text_field(
        tr("settings-transitions-primary-label"),
        tr("settings-transitions-primary-desc"),
        keys::transition::SAND_PRIMARY,
        "DP-1",
    );
    for shader in shaders_in_family(family) {
        scope(builder, shader);
    }
    if family == "sand" {
        sand(builder);
    }
}

pub(super) fn tab_transitions_asleep(builder: &mut Builder<'_>) {
    builder.card(
        crate::i18n::text("settings-transitions-transitions-card"),
        tr("settings-transitions-transitions-card-desc"),
    );
    let timed = transitions::preset_duration_ms(PRESET_LABELS[0].0);
    preset_timing(builder, timed.unwrap_or_default());
    duration(builder);
    let current = builder.cfg.text(keys::transition::SHADER);
    shader(builder, "", current);
    preview_rate(builder);
    for name in SHADERS {
        if name != RANDOM_SHADER {
            scope(builder, name);
        }
    }
    sand(builder);
}

fn preset_timing(builder: &mut Builder<'_>, duration: u64) {
    builder.toggle_default_true(
        tr("settings-transitions-preset-timing-label"),
        &tr_args!(PRESET_TIMING_DESC, duration => duration),
        keys::transition::PRESET_TIMING,
    );
}

fn duration(builder: &mut Builder<'_>) {
    builder.num(
        tr("settings-transitions-duration-label"),
        tr("settings-transitions-duration-desc"),
        keys::transition::DURATION_MS,
        "ms",
    );
}

fn shader(builder: &mut Builder<'_>, family: &str, current: String) {
    let options: Vec<(String, String)> = shaders_in_family(family)
        .into_iter()
        .map(|name| (name.to_string(), pretty(name)))
        .collect();
    let count = options.len();
    builder.row(
        tr("settings-transitions-shader-label"),
        &tr_args!("settings-transitions-shader-count-desc", count => count),
        Control::Dropdown {
            palettes: Vec::new(),
            path: keys::transition::SHADER.to_string(),
            options,
            current,
        },
    );
}

fn preview_rate(builder: &mut Builder<'_>) {
    builder.num(
        tr("settings-transitions-preview-fps-label"),
        tr("settings-transitions-preview-fps-desc"),
        keys::transition::PREVIEW_FPS,
        "fps",
    );
}

fn scope(builder: &mut Builder<'_>, shader: &str) {
    let current = shader_scope(builder.cfg, shader);
    builder.dropdown_cur(
        &pretty(shader),
        tr("settings-transitions-scope-desc"),
        &shader_scope_path(shader),
        &[
            ("all", tr("settings-transitions-scope-all")),
            ("primary", tr("settings-transitions-scope-primary")),
        ],
        current,
    );
}

fn sand(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    let quality = cfg.text(keys::transition::SAND_QUALITY);
    let quality = if quality.is_empty() { String::from("auto") } else { quality };
    builder.dropdown_cur(
        tr("settings-transitions-quality-label"),
        tr("settings-transitions-quality-desc"),
        keys::transition::SAND_QUALITY,
        &[
            ("auto", tr("settings-transitions-quality-auto")),
            ("full", tr("settings-transitions-quality-full")),
            ("low", tr("settings-transitions-quality-low")),
        ],
        quality,
    );
    builder.toggle(
        tr("settings-transitions-sharp-label"),
        tr("settings-transitions-sharp-desc"),
        keys::transition::SAND_SHARP,
    );
}

#[expect(clippy::too_many_lines, reason = "Drawing settings composition")]
pub(super) fn tab_drawing(
    builder: &mut Builder<'_>,
    status: Option<&crate::contracts::settings::drawing::Status>,
    selected: Option<&str>,
    view_count: usize,
    library: bool,
) {
    use crate::contracts::settings::drawing::{PackState, Phase};

    builder.card(crate::i18n::text("settings-drawing-card"), tr("settings-drawing-card-desc"));
    let enabled = builder.cfg.flag(keys::drawing::ENABLED);
    let scope = builder.cfg.text(keys::drawing::SCOPE);
    let scope = if scope.is_empty() { keys::drawing::SCOPE_SELECTED } else { scope.as_str() };

    let pack = status.map_or(PackState::Missing, |status| status.pack);
    let checking = status.is_some_and(|status| status.phase == Phase::Checking);
    let pack_desc = match pack {
        _ if checking => tr("settings-drawing-pack-checking"),
        PackState::Missing if status.is_some() => tr(DRAWING_PACK_INSTALL),
        PackState::Missing => tr("settings-drawing-pack-missing"),
        PackState::Available => tr("settings-drawing-pack-ready"),
        PackState::Invalid => tr("settings-drawing-pack-invalid"),
    };
    let pack_detail = if pack == PackState::Invalid && !checking {
        status.and_then(|status| status.error.as_deref()).unwrap_or(pack_desc)
    } else {
        pack_desc
    };
    builder.info(tr("settings-drawing-pack-label"), pack_detail);
    if let Some(status) = status.filter(|status| !status.pack_path.is_empty()) {
        builder.info(
            &status.pack_path,
            &tr_args!(DRAWING_PACK_SIZE, size => format_bytes(status.pack_bytes)),
        );
    }
    builder.action(
        tr("settings-drawing-pack-action-label"),
        tr("settings-drawing-pack-action-desc"),
        ActionId::ChooseDrawingPack,
        tr("settings-drawing-pack-action"),
    );
    if pack != PackState::Missing {
        builder.action(
            tr("settings-drawing-detach-label"),
            tr("settings-drawing-detach-desc"),
            ActionId::DetachDrawingPack,
            tr("settings-drawing-detach-action"),
        );
    }

    let selected_status = status.filter(|status| selected == Some(status.key.as_str()));
    let selected_title = selected.unwrap_or(tr("settings-drawing-selection-none"));
    let other_active = selected_status.is_some_and(|status| {
        !status.active_key.is_empty()
            && status.active_key != status.key
            && matches!(status.phase, Phase::Preparing | Phase::Paused)
    });
    let selection = selected_status.map_or(tr("settings-drawing-selection-unprepared"), |status| {
        match (status.phase, other_active) {
            (Phase::Checking, _) => tr("settings-drawing-selection-checking"),
            (Phase::Preparing, true) if scope == keys::drawing::SCOPE_VIEW && status.total > 1 => {
                tr("settings-drawing-selection-view-preparing")
            }
            (Phase::Paused, true) if scope == keys::drawing::SCOPE_VIEW && status.total > 1 => {
                tr("settings-drawing-selection-view-paused")
            }
            (Phase::Preparing, true) => tr("settings-drawing-selection-other-preparing"),
            (Phase::Paused, true) => tr("settings-drawing-selection-other-paused"),
            (Phase::Preparing, false) => tr("settings-drawing-selection-preparing"),
            (Phase::Paused, false) => tr("settings-drawing-selection-paused"),
            (Phase::Cancelled, _) => tr("settings-drawing-selection-cancelled"),
            (Phase::Failed, _) => tr("settings-drawing-selection-failed"),
            (Phase::Idle | Phase::Ready, _) if status.prepared => {
                tr("settings-drawing-selection-prepared")
            }
            (Phase::Idle | Phase::Ready, _) => tr("settings-drawing-selection-unprepared"),
        }
    });
    let detail = selected_status.and_then(|status| status.error.as_deref()).unwrap_or(selection);
    builder.row(
        tr("settings-drawing-selection-label"),
        detail,
        Control::StackBar {
            id: String::from(DRAWING_SELECTED_ID),
            summary: selected_title.to_string(),
            preview: selected_status.and_then(|status| status.preview_path.clone()),
            rows: Vec::new(),
        },
    );
    builder.dropdown_cur(
        tr("settings-drawing-scope-label"),
        tr("settings-drawing-scope-desc"),
        keys::drawing::SCOPE,
        &[
            (keys::drawing::SCOPE_SELECTED, tr("settings-drawing-scope-selected")),
            (keys::drawing::SCOPE_VIEW, tr("settings-drawing-scope-view")),
        ],
        scope.to_string(),
    );
    if scope == keys::drawing::SCOPE_VIEW && !library {
        builder.info(
            tr("settings-drawing-view-count-label"),
            &tr_args!(DRAWING_VIEW_COUNT, count => view_count),
        );
    }
    if let Some(status) = status {
        if matches!(status.phase, Phase::Preparing | Phase::Checking) {
            if checking {
                builder.info(
                    tr("settings-drawing-checking-label"),
                    tr("settings-drawing-checking-desc"),
                );
            } else {
                builder.info(
                    tr("settings-drawing-progress-label"),
                    &tr_args!(DRAWING_PROGRESS, stage => drawing_stage(&status.stage), percent => (status.progress * DRAWING_PERCENT).round() as u32, completed => status.completed, total => status.total),
                );
            }
            if !checking {
                builder.action(
                    "",
                    tr("settings-drawing-pause-desc"),
                    ActionId::PauseDrawing,
                    tr("settings-drawing-pause-action"),
                );
            }
            builder.action(
                "",
                tr("settings-drawing-cancel-desc"),
                ActionId::CancelDrawing,
                tr("settings-drawing-cancel-action"),
            );
        } else if status.phase == Phase::Paused {
            builder.action(
                "",
                tr("settings-drawing-resume-desc"),
                ActionId::ResumeDrawing,
                tr("settings-drawing-resume-action"),
            );
            builder.action(
                "",
                tr("settings-drawing-cancel-desc"),
                ActionId::CancelDrawing,
                tr("settings-drawing-cancel-action"),
            );
        } else if pack == PackState::Available && !library {
            if scope == keys::drawing::SCOPE_VIEW {
                if view_count == 0 {
                    builder.info(tr("settings-drawing-view-empty-label"), tr(DRAWING_VIEW_EMPTY));
                } else if view_count > crate::contracts::settings::drawing::MAX_PREPARE_KEYS {
                    builder.info(
                        tr("settings-drawing-view-limit-label"),
                        &tr_args!(DRAWING_VIEW_LIMIT, count => view_count, limit => crate::contracts::settings::drawing::MAX_PREPARE_KEYS),
                    );
                } else {
                    builder.action(
                        "",
                        tr("settings-drawing-prepare-view-desc"),
                        ActionId::PrepareDrawing,
                        &tr_args!(DRAWING_VIEW_PREPARE, count => view_count),
                    );
                }
            } else if selected.is_some() {
                builder.action(
                    "",
                    tr("settings-drawing-prepare-desc"),
                    ActionId::PrepareDrawing,
                    tr(if selected_status.is_some_and(|status| status.prepared) {
                        "settings-drawing-prepare-again-action"
                    } else {
                        "settings-drawing-prepare-action"
                    }),
                );
            }
        }
    }
    if pack == PackState::Available && !checking {
        builder.action(
            tr("settings-drawing-library-label"),
            tr("settings-drawing-library-desc"),
            ActionId::PrepareAllDrawings,
            tr("settings-drawing-library-action"),
        );
    }
    if selected_status.is_some_and(|status| status.prepared) {
        builder.action(
            "",
            tr("settings-drawing-preview-desc"),
            ActionId::PreviewDrawing,
            tr("settings-drawing-preview-action"),
        );
    }
    builder.toggle(
        tr("settings-drawing-enable-label"),
        tr("settings-drawing-enable-desc"),
        keys::drawing::ENABLED,
    );
    builder.colour(
        tr("settings-drawing-paper-label"),
        tr("settings-drawing-paper-desc"),
        keys::drawing::PAPER_COLOR,
    );
    builder.colour(
        tr("settings-drawing-ink-label"),
        tr("settings-drawing-ink-desc"),
        keys::drawing::INK_COLOR,
    );
    let ink = builder.cfg.text(keys::drawing::INK);
    builder.dropdown_cur(
        tr("settings-drawing-ink-style-label"),
        tr("settings-drawing-ink-style-desc"),
        keys::drawing::INK,
        &DRAWING_INKS.map(|(value, label)| (value, tr(label))),
        if ink == keys::drawing::INK_RAINBOW { ink } else { keys::drawing::INK_PLAIN.to_string() },
    );
    builder.num(
        tr("settings-drawing-duration-label"),
        tr("settings-drawing-duration-desc"),
        keys::drawing::DURATION_MS,
        DRAWING_DURATION_UNIT,
    );
    builder.toggle(
        tr("settings-drawing-battery-label"),
        tr("settings-drawing-battery-desc"),
        keys::drawing::ON_BATTERY,
    );
    builder.toggle(
        tr("settings-drawing-auto-label"),
        tr("settings-drawing-auto-desc"),
        keys::drawing::AUTO_PREPARE,
    );
    drawing_runtime(builder);
    if let Some(status) = status {
        builder.info(tr("settings-drawing-cache-label"), &format_bytes(status.cache_bytes));
    }
    if status.is_some_and(|status| status.cache_bytes > 0) {
        builder.action(
            "",
            tr("settings-drawing-clear-desc"),
            ActionId::ClearDrawingCache,
            tr("settings-drawing-clear-action"),
        );
    }
    if enabled
        && pack != PackState::Available
        && !selected_status.is_some_and(|status| status.prepared)
    {
        builder.info(
            tr("settings-drawing-unavailable-label"),
            tr("settings-drawing-unavailable-desc"),
        );
    }
}

fn drawing_runtime(builder: &mut Builder<'_>) {
    let device = Device::parse(&builder.cfg.text(keys::drawing::DEVICE)).unwrap_or_default();
    builder.dropdown_cur(
        tr("settings-drawing-device-label"),
        tr("settings-drawing-device-desc"),
        keys::drawing::DEVICE,
        &DRAWING_DEVICES.map(|(device, label)| (device.label(), tr(label))),
        device.label().to_string(),
    );
    let saved = builder.cfg.text(keys::drawing::MEMORY_LIMIT_MIB);
    let mut limits = vec![(
        keys::drawing::MEMORY_LIMIT_AUTO.to_string(),
        tr("settings-drawing-memory-auto").to_string(),
    )];
    limits.extend(
        DRAWING_MEMORY_GIB.map(|gib| {
            ((gib * MIB_PER_GIB).to_string(), tr_args!(DRAWING_MEMORY_SIZE, gib => gib))
        }),
    );
    let current = match saved_memory_mib(&saved) {
        Some(mib) => {
            let value = mib.to_string();
            if !limits.iter().any(|(preset, _)| *preset == value) {
                limits.push((value.clone(), tr_args!(DRAWING_MEMORY_SAVED, mib => mib)));
            }
            value
        }
        None => keys::drawing::MEMORY_LIMIT_AUTO.to_string(),
    };
    let options: Vec<_> =
        limits.iter().map(|(value, label)| (value.as_str(), label.as_str())).collect();
    builder.dropdown_cur(
        tr("settings-drawing-memory-label"),
        tr("settings-drawing-memory-desc"),
        keys::drawing::MEMORY_LIMIT_MIB,
        &options,
        current,
    );
}

fn drawing_stage(stage: &str) -> &'static str {
    DRAWING_STAGES
        .iter()
        .find_map(|(key, label)| (*key == stage).then(|| tr(label)))
        .unwrap_or_else(|| tr("settings-drawing-stage-other"))
}

fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * KIB;
    if bytes >= MIB {
        format!("{} MiB", bytes / MIB)
    } else {
        format!("{} KiB", bytes.div_ceil(KIB))
    }
}

mod tests;
