use crate::contracts::settings::{EFFECT_NAMES, GraphicsTier, keys};
use crate::i18n::{settings_sand_meter_detail, tr, tr_args};

use super::super::tables::{MODES, motion_speed_options};
use super::builder::{folder_dropdown_options, type_chip_label};
use super::{ActionId, Builder, Control, PRESET_NAME_KEY};

pub(super) fn tab_selector(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    let mode = cfg.display_mode();
    builder.card(tr("settings-selector-layout-card"), "");
    let display_mode = crate::contracts::picker::Mode::from_key(&mode).as_key();
    builder.row(
        tr("settings-selector-display-mode-label"),
        tr("settings-selector-display-mode-desc"),
        Control::Chips {
            path: keys::selector::DISPLAY_MODE.to_string(),
            options: MODES
                .iter()
                .map(|value| ((*value).to_string(), tr(mode_label_key(value)).to_string()))
                .collect(),
            current: display_mode.to_string(),
            disabled: Vec::new(),
        },
    );
    let filter_speed = match cfg.text(keys::motion::FILTER_SWAP_SPEED).as_str() {
        "fast" | "standard" | "slow" => cfg.text(keys::motion::FILTER_SWAP_SPEED),
        _ => String::from("slow"),
    };
    builder.dropdown_cur(
        tr("settings-selector-filter-motion-label"),
        tr("settings-selector-filter-motion-desc"),
        keys::motion::FILTER_SWAP_SPEED,
        &motion_speed_options(),
        filter_speed,
    );
    builder.card(tr("settings-selector-presets-card"), tr("settings-selector-presets-card-desc"));
    let pmode = cfg.selector_mode();
    let selected = cfg.selected_preset(&pmode);
    builder.text_field(
        tr("settings-selector-preset-name-label"),
        tr("settings-selector-preset-name-desc"),
        PRESET_NAME_KEY,
        tr("settings-selector-preset-name-placeholder"),
    );
    let items: Vec<(String, bool)> = cfg
        .selector_preset_names(&pmode)
        .into_iter()
        .map(|name| {
            let active = selected.as_deref() == Some(name.as_str());
            (name, active)
        })
        .collect();
    builder.row("", "", Control::Presets { mode: pmode, items });
    builder.card(tr("settings-selector-live-preview-card"), "");
    builder.toggle(
        tr("settings-selector-live-preview-label"),
        tr("settings-selector-live-preview-desc"),
        keys::selector::LIVE_PREVIEW,
    );
    builder.toggle_default_true(
        tr("settings-selector-type-badges-label"),
        tr("settings-selector-type-badges-desc"),
        keys::selector::SHOW_TYPE_BADGES,
    );
    match mode.as_str() {
        "hex" => {
            builder.card(
                tr("settings-selector-composition-card"),
                tr("settings-selector-composition-card-desc-hex"),
            );
            builder.chips(
                tr("settings-selector-hex-curve-label"),
                tr("settings-selector-hex-curve-desc"),
                keys::selector::HEX_CURVE,
                &[
                    ("flat", tr("settings-selector-hex-curve-flat")),
                    ("arc", tr("settings-selector-hex-curve-arc")),
                    ("wave", tr("settings-selector-hex-curve-wave")),
                    ("s", tr("settings-selector-hex-curve-s")),
                    ("cylinder", tr("settings-selector-hex-curve-cylinder")),
                ],
            );
            builder.chips(
                tr("settings-selector-hex-shape-label"),
                tr("settings-selector-hex-shape-desc"),
                keys::selector::HEX_SHAPE,
                &[
                    ("triangle", tr("settings-selector-hex-shape-triangle")),
                    ("hexagon", tr("settings-selector-hex-shape-hexagon")),
                    ("diamond", tr("settings-selector-hex-shape-diamond")),
                    ("rhombus", tr("settings-selector-hex-shape-rhombus")),
                ],
            );
            let curve = cfg.text(keys::selector::HEX_CURVE);
            if curve != "flat" {
                builder.num(
                    tr("settings-selector-hex-intensity-label"),
                    tr("settings-selector-hex-intensity-desc"),
                    keys::selector::HEX_ARC_INTENSITY_X10,
                    "",
                );
            }
            if curve == "wave" {
                builder.num(
                    tr("settings-selector-hex-waves-label"),
                    tr("settings-selector-hex-waves-desc"),
                    keys::selector::HEX_CURVE_FREQUENCY,
                    "x",
                );
            }
            if curve == "cylinder" {
                builder.num(
                    tr("settings-selector-hex-orbit-radius-label"),
                    tr("settings-selector-hex-orbit-radius-desc"),
                    keys::selector::HEX_ORBIT_RADIUS,
                    "px",
                );
            }
            builder.num(tr("settings-selector-hex-rows-label"), "", keys::selector::HEX_ROWS, "");
            builder.num(tr("settings-selector-hex-cols-label"), "", keys::selector::HEX_COLS, "");
            builder
                .card(tr("settings-selector-cells-card"), tr("settings-selector-cells-card-desc"));
            builder.num(
                tr("settings-selector-hex-cell-size-label"),
                "",
                keys::selector::HEX_RADIUS,
                "px",
            );
            builder.num(
                tr("settings-selector-hex-gap-x-label"),
                "",
                keys::selector::HEX_GAP_X,
                "px",
            );
            builder.num(
                tr("settings-selector-hex-gap-y-label"),
                "",
                keys::selector::HEX_GAP_Y,
                "px",
            );
        }
        "wall" | "grid" => {
            builder.card(
                tr("settings-selector-composition-card"),
                tr("settings-selector-composition-card-desc-wall"),
            );
            builder.chips(
                tr("settings-selector-wall-mode-label"),
                tr("settings-selector-wall-mode-desc"),
                keys::selector::GRID_LAYOUT,
                &[
                    ("uniform", tr("settings-selector-wall-mode-uniform")),
                    ("brick", tr("settings-selector-wall-mode-brick")),
                    ("masonry", tr("settings-selector-wall-mode-masonry")),
                    ("justified", tr("settings-selector-wall-mode-justified")),
                    ("editorial", tr("settings-selector-wall-mode-editorial")),
                    ("cylinder", tr("settings-selector-wall-mode-cylinder")),
                ],
            );
            builder.num(
                tr("settings-selector-grid-columns-label"),
                "",
                keys::selector::GRID_COLUMNS,
                "",
            );
            builder.num(tr("settings-selector-grid-rows-label"), "", keys::selector::GRID_ROWS, "");
            let wall_mode = cfg.text(keys::selector::GRID_LAYOUT);
            if matches!(wall_mode.as_str(), "brick" | "editorial") {
                builder.num(
                    tr("settings-selector-grid-stagger-label"),
                    tr("settings-selector-grid-stagger-desc"),
                    keys::selector::GRID_STAGGER,
                    "%",
                );
            }
            if wall_mode == "cylinder" {
                builder.num(
                    tr("settings-selector-grid-bend-label"),
                    tr("settings-selector-grid-bend-desc"),
                    keys::selector::GRID_CYLINDER_BEND,
                    "%",
                );
                builder.num(
                    tr("settings-selector-grid-cylinder-radius-label"),
                    tr("settings-selector-grid-cylinder-radius-desc"),
                    keys::selector::GRID_CYLINDER_RADIUS,
                    "px",
                );
            }
            builder
                .card(tr("settings-selector-cards-card"), tr("settings-selector-cards-card-desc"));
            builder.num(
                tr("settings-selector-card-width-label"),
                "",
                keys::selector::GRID_THUMB_WIDTH,
                "px",
            );
            builder.num(
                tr("settings-selector-card-height-label"),
                "",
                keys::selector::GRID_THUMB_HEIGHT,
                "px",
            );
            builder.num(
                tr("settings-selector-grid-gap-x-label"),
                tr("settings-selector-grid-gap-x-desc"),
                keys::selector::GRID_GAP_X,
                "px",
            );
            builder.num(
                tr("settings-selector-grid-gap-y-label"),
                tr("settings-selector-grid-gap-y-desc"),
                keys::selector::GRID_GAP_Y,
                "px",
            );
            builder.toggle(
                tr("settings-selector-grid-round-label"),
                tr("settings-selector-grid-round-desc"),
                keys::selector::GRID_ROUND_CORNERS,
            );
            builder.num(
                tr("settings-selector-grid-corner-radius-label"),
                tr("settings-selector-grid-corner-radius-desc"),
                keys::selector::GRID_CORNER_RADIUS,
                "px",
            );
        }
        "sandy" => {
            let grain = cfg.sandy_grain();
            let verts = crate::frontend::scene::sandy::grain_verts(
                grain,
                crate::frontend::scene::sandy::HERO_HALF_W,
                crate::frontend::scene::sandy::HERO_HALF_H,
            );
            let card = cfg.graphics_card();
            let note = match card.tier {
                GraphicsTier::Discrete => tr("settings-selector-sand-note-discrete"),
                _ => tr("settings-selector-sand-note-integrated"),
            };
            builder.card(
                tr("settings-selector-performance-card"),
                tr("settings-selector-performance-card-desc"),
            );
            let tier = match card.tier {
                GraphicsTier::Discrete => tr("settings-selector-gpu-tier-discrete"),
                GraphicsTier::Integrated => tr("settings-selector-gpu-tier-integrated"),
                GraphicsTier::Other => tr("settings-selector-gpu-tier-other"),
            };
            builder.info(
                &tr_args!(
                    "settings-selector-sand-meter-label",
                    count => compact_count(verts),
                    load => heaviness(verts),
                ),
                &settings_sand_meter_detail(grain as u32, &card.name, tier, note),
            );
            builder.num(
                tr("settings-selector-grain-size-label"),
                tr("settings-selector-grain-size-desc"),
                keys::selector::SANDY_GRAIN,
                "px",
            );
            builder.num(
                tr("settings-selector-render-scale-label"),
                tr("settings-selector-render-scale-desc"),
                keys::selector::SANDY_RES_SCALE,
                "%",
            );
            builder.toggle(
                tr("settings-selector-lod-auto-label"),
                tr("settings-selector-lod-auto-desc"),
                keys::selector::SANDY_LOD_AUTO,
            );
            builder.num(
                tr("settings-selector-lod-strength-label"),
                tr("settings-selector-lod-strength-desc"),
                keys::selector::SANDY_LOD,
                "x",
            );
            builder
                .card(tr("settings-selector-sandy-card"), tr("settings-selector-sandy-card-desc"));
            builder.num(
                tr("settings-selector-sandy-center-label"),
                "",
                keys::selector::SANDY_CENTER,
                "px",
            );
            builder.num(
                tr("settings-selector-sandy-slice-height-label"),
                "",
                keys::selector::SANDY_SLICE_HEIGHT,
                "px",
            );
            builder.num(
                tr("settings-selector-sandy-slice-width-label"),
                "",
                keys::selector::SANDY_SLICE_WIDTH,
                "px",
            );
            builder.num(
                tr("settings-selector-sandy-gap-label"),
                tr("settings-selector-sandy-gap-desc"),
                keys::selector::SANDY_SPACING,
                "px",
            );
            builder.num(
                tr("settings-selector-sandy-skew-label"),
                "",
                keys::selector::SANDY_SKEW,
                "",
            );
            builder.num(
                tr("settings-selector-sandy-edge-speed-label"),
                tr("settings-selector-sandy-edge-speed-desc"),
                keys::selector::SANDY_EDGE_SPEED,
                "%",
            );
            builder.card(tr("settings-selector-sand-card"), tr("settings-selector-sand-card-desc"));
            builder.num(
                tr("settings-selector-sandy-duration-label"),
                tr("settings-selector-sandy-duration-desc"),
                keys::selector::SANDY_DURATION,
                "ms",
            );
            builder.num(
                tr("settings-selector-sandy-blend-label"),
                tr("settings-selector-sandy-blend-desc"),
                keys::selector::SANDY_BLEND,
                "ms",
            );
            builder.num(
                tr("settings-selector-sandy-strands-label"),
                tr("settings-selector-sandy-strands-desc"),
                keys::selector::SANDY_STRANDS,
                "",
            );
            builder.num(
                tr("settings-selector-sandy-twist-label"),
                tr("settings-selector-sandy-twist-desc"),
                keys::selector::SANDY_TWIST,
                "%",
            );
            builder.num(
                tr("settings-selector-sandy-orbit-label"),
                tr("settings-selector-sandy-orbit-desc"),
                keys::selector::SANDY_ORBIT,
                "%",
            );
            builder.num(
                tr("settings-selector-sandy-turbulence-label"),
                tr("settings-selector-sandy-turbulence-desc"),
                keys::selector::SANDY_TURBULENCE,
                "%",
            );
            builder.num(
                tr("settings-selector-sandy-waist-label"),
                tr("settings-selector-sandy-waist-desc"),
                keys::selector::SANDY_WAIST,
                "%",
            );
            builder.num(
                tr("settings-selector-sandy-front-label"),
                tr("settings-selector-sandy-front-desc"),
                keys::selector::SANDY_FRONT,
                "%",
            );
            builder.num(
                tr("settings-selector-sandy-fan-label"),
                tr("settings-selector-sandy-fan-desc"),
                keys::selector::SANDY_FAN,
                "%",
            );
            let swap_styles = crate::contracts::picker::SANDY_SWAP_STYLES
                .map(|(key, _)| (key, tr(swap_style_label_key(key))));
            builder.dropdown_cur(
                tr("settings-selector-swap-style-label"),
                tr("settings-selector-swap-style-desc"),
                keys::selector::SANDY_SWAP_STYLE,
                &swap_styles,
                builder.cfg.sandy_swap_style(),
            );
            builder.num(
                tr("settings-selector-sandy-arc-label"),
                tr("settings-selector-sandy-arc-desc"),
                keys::selector::SANDY_ARC,
                "%",
            );
            builder.toggle(
                tr("settings-selector-swap-loop-label"),
                tr("settings-selector-swap-loop-desc"),
                keys::selector::SANDY_SWAP_LOOP,
            );
            builder.toggle(
                tr("settings-selector-outgoing-live-label"),
                tr("settings-selector-outgoing-live-desc"),
                keys::selector::SANDY_OUTGOING_LIVE,
            );
            builder.card(tr("settings-selector-ring-card"), tr("settings-selector-ring-card-desc"));
            builder.num(
                tr("settings-selector-ring-size-label"),
                tr("settings-selector-ring-size-desc"),
                keys::selector::SANDY_RING_SIZE,
                "%",
            );
            builder.num(
                tr("settings-selector-ring-spin-label"),
                tr("settings-selector-ring-spin-desc"),
                keys::selector::SANDY_RING_SPIN,
                "%",
            );
            builder.num(
                tr("settings-selector-ring-wave-label"),
                tr("settings-selector-ring-wave-desc"),
                keys::selector::SANDY_RING_WAVE,
                "%",
            );
            builder.num(
                tr("settings-selector-ring-soft-label"),
                tr("settings-selector-ring-soft-desc"),
                keys::selector::SANDY_RING_SOFT,
                "%",
            );
            builder.num(
                tr("settings-selector-ring-blend-label"),
                tr("settings-selector-ring-blend-desc"),
                keys::selector::SANDY_RING_BLEND,
                "%",
            );
            builder.num(
                tr("settings-selector-ring-hold-label"),
                tr("settings-selector-ring-hold-desc"),
                keys::selector::SANDY_RING_HOLD,
                "ms",
            );
        }
        _ => {
            builder.card(tr("settings-selector-slice-size-card"), "");
            builder.num(
                tr("settings-selector-slice-height-label"),
                "",
                keys::selector::SLICE_HEIGHT,
                "",
            );
            builder.num(
                tr("settings-selector-visible-count-label"),
                "",
                keys::selector::VISIBLE_COUNT,
                "",
            );
            builder.num(
                tr("settings-selector-expanded-width-label"),
                "",
                keys::selector::EXPANDED_WIDTH,
                "",
            );
            builder.num(
                tr("settings-selector-slice-width-label"),
                "",
                keys::selector::SLICE_WIDTH,
                "",
            );
            builder.num(
                tr("settings-selector-slice-gap-label"),
                tr("settings-selector-slice-gap-desc"),
                keys::selector::SLICE_SPACING,
                "px",
            );
            builder.num(tr("settings-selector-skew-label"), "", keys::selector::SKEW_OFFSET, "");
            builder.num(
                tr("settings-selector-edge-tilt-label"),
                tr("settings-selector-edge-tilt-desc"),
                keys::selector::SLICE_EDGE_TILT,
                "px",
            );
            builder.toggle(
                tr("settings-selector-wobble-label"),
                tr("settings-selector-wobble-desc"),
                keys::selector::SLICE_WOBBLE,
            );
            builder.num(
                tr("settings-selector-wobble-strength-label"),
                tr("settings-selector-wobble-strength-desc"),
                keys::selector::SLICE_WOBBLE_STRENGTH,
                "%",
            );
            builder.card(tr("settings-selector-corners-card"), "");
            builder.toggle(
                tr("settings-selector-round-corners-label"),
                tr("settings-selector-round-corners-desc"),
                keys::selector::ROUND_CORNERS,
            );
            builder.num(
                tr("settings-selector-corner-tl-label"),
                tr("settings-selector-corner-tl-desc"),
                keys::selector::CORNER_TL,
                "px",
            );
            builder.num(
                tr("settings-selector-corner-tr-label"),
                tr("settings-selector-corner-tr-desc"),
                keys::selector::CORNER_TR,
                "px",
            );
            builder.num(
                tr("settings-selector-corner-br-label"),
                tr("settings-selector-corner-br-desc"),
                keys::selector::CORNER_BR,
                "px",
            );
            builder.num(
                tr("settings-selector-corner-bl-label"),
                tr("settings-selector-corner-bl-desc"),
                keys::selector::CORNER_BL,
                "px",
            );
        }
    }
    builder
        .card(tr("settings-selector-card-flip-card"), tr("settings-selector-card-flip-card-desc"));
    let flip_opts: Vec<(String, String)> = EFFECT_NAMES
        .iter()
        .map(|name| ((*name).to_string(), tr(effect_label_key(name)).to_string()))
        .collect();
    let flip_cur = {
        let cur = cfg.text(keys::selector::FLIP_EFFECT);
        if EFFECT_NAMES.contains(&cur.as_str()) { cur } else { EFFECT_NAMES[0].to_string() }
    };
    builder.row(
        tr("settings-selector-flip-effect-label"),
        tr("settings-selector-flip-effect-desc"),
        Control::Dropdown {
            palettes: Vec::new(),
            path: keys::selector::FLIP_EFFECT.to_string(),
            options: flip_opts,
            current: flip_cur,
        },
    );
    builder.toggle_default_true(
        tr("settings-selector-flip-shader-label"),
        tr("settings-selector-flip-shader-desc"),
        keys::selector::FLIP_SHADER,
    );
    builder.toggle_default_true(
        tr("settings-selector-flip-back-label"),
        tr("settings-selector-flip-back-desc"),
        keys::selector::FLIP_BACK_REVEAL,
    );
    builder.num(
        tr("settings-selector-flip-duration-label"),
        tr("settings-selector-flip-duration-desc"),
        keys::selector::FLIP_DURATION_MS,
        "ms",
    );
    builder
        .card(tr("settings-selector-tag-cloud-card"), tr("settings-selector-tag-cloud-card-desc"));
    builder.num(
        tr("settings-selector-tag-cloud-width-label"),
        tr("settings-selector-tag-cloud-width-desc"),
        keys::selector::TAG_CLOUD_WIDTH,
        "px",
    );
    let tag_cloud_rows = match cfg.text(keys::selector::TAG_CLOUD_ROWS).as_str() {
        "1" => String::from("1"),
        "3" => String::from("3"),
        _ => String::from("2"),
    };
    builder.dropdown_cur(
        tr("settings-selector-tag-cloud-rows-label"),
        tr("settings-selector-tag-cloud-rows-desc"),
        keys::selector::TAG_CLOUD_ROWS,
        &[
            ("1", tr("settings-selector-tag-cloud-rows-one")),
            ("2", tr("settings-selector-tag-cloud-rows-two")),
            ("3", tr("settings-selector-tag-cloud-rows-three")),
        ],
        tag_cloud_rows,
    );
    builder.card(
        tr("settings-selector-video-preview-card"),
        tr("settings-selector-video-preview-card-desc"),
    );
    builder.toggle(
        tr("settings-selector-video-preview-label"),
        tr("settings-selector-video-preview-desc"),
        keys::video_preview::ENABLED,
    );
    builder.num(
        tr("settings-selector-preview-delay-label"),
        tr("settings-selector-preview-delay-desc"),
        keys::video_preview::DELAY_MS,
        "ms",
    );
}

fn mode_label_key(mode: &str) -> &'static str {
    match mode {
        "hex" => "settings-selector-mode-hex",
        "wall" => "settings-selector-mode-wall",
        "sandy" => "settings-selector-mode-sandy",
        _ => "settings-selector-mode-slices",
    }
}

fn swap_style_label_key(style: &str) -> &'static str {
    match style {
        "hourglass" => "settings-selector-swap-style-hourglass",
        "castle" => "settings-selector-swap-style-castle",
        "saltation" => "settings-selector-swap-style-saltation",
        "pour" => "settings-selector-swap-style-pour",
        "orbit" => "settings-selector-swap-style-orbit",
        "burst" => "settings-selector-swap-style-burst",
        "weave" => "settings-selector-swap-style-weave",
        "bloom" => "settings-selector-swap-style-bloom",
        "flock" => "settings-selector-swap-style-flock",
        "ring" => "settings-selector-swap-style-ring",
        _ => "settings-selector-swap-style-vortex",
    }
}

fn effect_label_key(name: &str) -> &'static str {
    match name {
        "Edge Fracture" => "effects-name-edge-fracture",
        "Tonal Wipe" => "effects-name-tonal-wipe",
        "Ash" => "effects-name-ash",
        "Depth Parallax" => "effects-name-depth-parallax",
        "Bokeh Bloom" => "effects-name-bokeh-bloom",
        "Light Streaks" => "effects-name-light-streaks",
        "Voxel Extrude" => "effects-name-voxel-extrude",
        "Pixel Sort" => "effects-name-pixel-sort",
        "Rack Focus" => "effects-name-rack-focus",
        "Topographic" => "effects-name-topographic",
        "Tonal Layers" => "effects-name-tonal-layers",
        _ => "effects-name-ignite",
    }
}

pub(super) fn compact_count(count: u32) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1e6)
    } else {
        format!("{}K", (count as f64 / 1e3).round() as u32)
    }
}

pub(super) fn heaviness(vertices: u32) -> &'static str {
    match vertices {
        count if count > 800_000 => tr("settings-selector-sand-load-very-heavy"),
        count if count > 250_000 => tr("settings-selector-sand-load-heavy"),
        count if count > 80_000 => tr("settings-selector-sand-load-moderate"),
        _ => tr("settings-selector-sand-load-light"),
    }
}

pub(super) fn tab_filter(builder: &mut Builder<'_>, folders: &[String]) {
    builder.card(tr("settings-filter-appearance-card"), tr("settings-filter-appearance-card-desc"));
    let visual_style =
        match builder.cfg.text(crate::contracts::settings::keys::filter_bar::VISUAL_STYLE).as_str()
        {
            "slices" | "sandy" => "slices",
            "hex" => "hex",
            "wall" => "wall",
            _ => "match",
        };
    builder.dropdown_cur(
        tr("settings-filter-visual-style-label"),
        tr("settings-filter-visual-style-desc"),
        crate::contracts::settings::keys::filter_bar::VISUAL_STYLE,
        &[
            ("match", tr("settings-filter-visual-style-match")),
            ("slices", tr("settings-filter-visual-style-slices")),
            ("hex", tr("settings-filter-visual-style-hex")),
            ("wall", tr("settings-filter-visual-style-wall")),
        ],
        visual_style.to_string(),
    );
    builder.dropdown_setting(
        tr("settings-filter-orientation-label"),
        tr("settings-filter-orientation-desc"),
        crate::contracts::settings::schema::setting::filter_bar::ORIENTATION,
        &[
            ("horizontal", tr("settings-filter-orientation-horizontal")),
            ("vertical", tr("settings-filter-orientation-vertical")),
        ],
    );
    builder.card(tr("settings-filter-type-chips-card"), tr("settings-filter-type-chips-card-desc"));
    for key in crate::frontend::ui::TYPES {
        let tkey = crate::frontend::ui::BarShow::type_key(key);
        builder.toggle(type_chip_label(key), "", &format!("filterBar.show.type.{tkey}"));
    }
    builder.card(
        tr("settings-filter-sort-options-card"),
        tr("settings-filter-sort-options-card-desc"),
    );
    for (mode, _) in crate::frontend::ui::SORTS {
        builder.toggle(
            tr(crate::frontend::ui::sort_label_key(mode)),
            "",
            &format!("filterBar.show.sort.{mode}"),
        );
    }
    builder.card(
        tr("settings-filter-other-controls-card"),
        tr("settings-filter-other-controls-card-desc"),
    );
    builder.toggle(
        tr("settings-filter-show-folder-label"),
        tr("settings-filter-show-folder-desc"),
        keys::filter_bar::SHOW_FOLDER,
    );
    builder.toggle(
        tr("settings-filter-show-favourites-label"),
        tr("settings-filter-show-favourites-desc"),
        keys::filter_bar::SHOW_FAVOURITES,
    );
    builder.toggle(
        tr("settings-filter-show-random-label"),
        tr("settings-filter-show-random-desc"),
        keys::filter_bar::SHOW_RANDOM,
    );
    builder.toggle(
        tr("settings-filter-show-colors-label"),
        tr("settings-filter-show-colors-desc"),
        keys::filter_bar::SHOW_COLORS,
    );
    builder.toggle(
        tr("settings-filter-show-theme-label"),
        tr("settings-filter-show-theme-desc"),
        keys::filter_bar::SHOW_THEME,
    );
    builder.toggle(
        tr("settings-filter-show-tag-cloud-label"),
        tr("settings-filter-show-tag-cloud-desc"),
        keys::filter_bar::SHOW_TAG_CLOUD,
    );
    builder.toggle(
        tr("settings-filter-show-resolution-label"),
        tr("settings-filter-show-resolution-desc"),
        keys::filter_bar::SHOW_RESOLUTION,
    );
    builder.card(
        tr("settings-filter-resolution-presets-card"),
        tr("settings-filter-resolution-presets-card-desc"),
    );
    let resolution_presets = builder.cfg.array_len(keys::filter_bar::RESOLUTION_PRESETS);
    for idx in 0..resolution_presets {
        let base = format!("{}.{idx}", keys::filter_bar::RESOLUTION_PRESETS);
        let label_path = format!("{base}.label");
        let orientation_path = format!("{base}.orientation");
        let from_path = format!("{base}.from");
        let to_path = format!("{base}.to");
        let configured_label = builder.cfg.text(&label_path);
        let title = if configured_label.trim().is_empty() {
            tr_args!("settings-filter-preset-item-label", index => idx + 1)
        } else {
            configured_label
        };
        let from = builder.cfg.text(&from_path);
        let to = builder.cfg.text(&to_path);
        let orientation = builder.cfg.text(&orientation_path);
        let orientation_label = if orientation == "tall" {
            tr("settings-filter-preset-orientation-tall")
        } else {
            tr("settings-filter-preset-orientation-wide")
        };
        let dimensions =
            if to.trim().is_empty() { format!("{from}+") } else { format!("{from} - {to}") };
        let range = format!("{orientation_label} · {dimensions}");
        builder.details(&title, tr("settings-filter-preset-item-desc"), base, range, |builder| {
            builder.text_field(
                tr("settings-filter-preset-label-label"),
                tr("settings-filter-preset-name-desc"),
                &label_path,
                tr("settings-filter-preset-name-placeholder"),
            );
            builder.dropdown(
                tr("settings-filter-preset-orientation-label"),
                tr("settings-filter-preset-orientation-desc"),
                &orientation_path,
                &[
                    ("wide", tr("settings-filter-preset-orientation-wide")),
                    ("tall", tr("settings-filter-preset-orientation-tall")),
                ],
            );
            builder.text_field(
                tr("settings-filter-preset-from-label"),
                tr("settings-filter-preset-from-desc"),
                &from_path,
                tr("settings-filter-preset-resolution-placeholder"),
            );
            builder.text_field(
                tr("settings-filter-preset-to-label"),
                tr("settings-filter-preset-to-desc"),
                &to_path,
                tr("settings-filter-preset-resolution-placeholder"),
            );
            builder.action(
                "",
                tr("settings-filter-preset-remove-desc"),
                ActionId::RemoveResolutionPreset(idx as u16),
                tr("settings-filter-remove-action"),
            );
        });
    }
    builder.action(
        tr("settings-filter-add-preset-label"),
        tr("settings-filter-add-preset-desc"),
        ActionId::AddResolutionPreset,
        tr("settings-filter-add-action"),
    );
    builder.card(
        tr("settings-filter-default-folder-card"),
        tr("settings-filter-default-folder-card-desc"),
    );
    let folder_opts = folder_dropdown_options(folders);
    let folder_refs: Vec<(&str, &str)> =
        folder_opts.iter().map(|(key, label)| (key.as_str(), label.as_str())).collect();
    builder.dropdown_setting(
        tr("settings-filter-default-folder-label"),
        tr("settings-filter-default-folder-desc"),
        crate::contracts::settings::schema::setting::filter_bar::DEFAULT_FOLDER,
        &folder_refs,
    );
    builder.toggle_setting(
        tr("settings-filter-sticky-label"),
        tr("settings-filter-sticky-desc"),
        crate::contracts::settings::schema::setting::filter_bar::STICKY,
    );
    builder.card(tr("settings-filter-visibility-card"), tr("settings-filter-visibility-card-desc"));
    builder.toggle_setting(
        tr("settings-filter-always-visible-label"),
        tr("settings-filter-always-visible-desc"),
        crate::contracts::settings::schema::setting::general::FILTER_BAR_ALWAYS_VISIBLE,
    );
    builder.card(tr("settings-filter-weather-card"), tr("settings-filter-weather-card-desc"));
    builder.toggle(
        tr("settings-filter-weather-label"),
        tr("settings-filter-weather-desc"),
        keys::general::WEATHER_MATCH,
    );
    builder.text_field(
        tr("settings-filter-location-label"),
        tr("settings-filter-location-desc"),
        keys::general::LOCALE,
        tr("settings-filter-location-placeholder"),
    );
}
