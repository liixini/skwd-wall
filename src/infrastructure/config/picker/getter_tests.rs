#![cfg(test)]

use super::*;
use crate::contracts::picker::{SANDY_SWAP_STYLES, sandy_style_index};
use serde_json::{Value, json};

fn cfg(data: Value) -> Config {
    Config {
        data,
        config_path: std::env::temp_dir().join("skwd_getter_test.json"),
        small: false,
        transient: false,
        on_battery: false,
    }
}

fn battery_cfg(data: Value) -> Config {
    let mut config = cfg(data);
    config.on_battery = true;
    config
}

#[allow(clippy::needless_pass_by_value)]
fn sel(map: Value) -> Config {
    cfg(json!({ "components": { "wallpaperSelector": map } }))
}

#[test]
fn slices_default_to_touching_edges_and_preserve_explicit_spacing() {
    for small in [false, true] {
        let mut config = cfg(json!({}));
        config.small = small;
        assert_eq!(config.slice_spacing(), 0.0);

        for spacing in [-70.0, -30.0, 0.0, 12.0] {
            let mut config = sel(json!({"sliceSpacing": spacing}));
            config.small = small;
            assert_eq!(config.slice_spacing(), spacing);
        }
    }
}

#[test]
fn battery_saver_fps_cap() {
    assert_eq!(cfg(json!({})).max_fps(), 120.0);
    assert!(!cfg(json!({})).battery_saver_active());
    assert_eq!(battery_cfg(json!({})).max_fps(), 60.0);
    assert!(battery_cfg(json!({})).battery_saver_active());
    assert_eq!(
        battery_cfg(json!({
            "general": {"maxFps": 144},
            "performance": {"batteryFps": 45}
        }))
        .max_fps(),
        45.0
    );
    assert_eq!(
        battery_cfg(json!({
            "general": {"maxFps": 144},
            "performance": {"batterySaver": false}
        }))
        .max_fps(),
        144.0
    );
    assert!(!battery_cfg(json!({"performance": {"batterySaver": false}})).battery_saver_active());
}

#[test]
fn array_slice_borrows_values_and_defaults_to_empty() {
    let conf = cfg(json!({"profiles": [{"key": "wallpaper", "dark": {"primary": "#123456"}}]}));
    let values = conf.array_slice("profiles");
    assert_eq!(values.as_ptr(), conf.root()["profiles"].as_array().unwrap().as_ptr());
    assert_eq!(values, conf.array_values("profiles"));
    for path in ["missing", "profiles.0", "profiles.0.key"] {
        assert!(conf.array_slice(path).is_empty());
    }
    assert!(cfg(json!({"profiles": null})).array_slice("profiles").is_empty());
}

#[test]
fn array_index_paths() {
    let mut conf = cfg(json!({
        "postProcessing": [
            {"command": "a", "type": "all"},
            {"command": "b", "type": "all"}
        ]
    }));
    assert_eq!(conf.str_path("postProcessing.0.command"), "a");
    assert_eq!(conf.str_path("postProcessing.1.command"), "b");

    conf.set_key("postProcessing.1.type", json!("video"));
    conf.set_key("postProcessing.0.command", json!("wal set %path%"));

    assert!(conf.get("postProcessing").and_then(Value::as_array).is_some());
    assert_eq!(conf.str_path("postProcessing.1.type"), "video");
    assert_eq!(conf.str_path("postProcessing.0.command"), "wal set %path%");
    assert_eq!(conf.str_path("postProcessing.1.command"), "b");
    assert_eq!(conf.array_len("postProcessing"), 2);
}

#[test]
fn open_fade_getters() {
    let conf = Config::from_data(json!({}));
    assert_eq!(conf.open_fade_ms(), 250.0);
    let conf = Config::from_data(json!({ "motion": { "standardMs": 450.0 } }));
    assert_eq!(conf.open_fade_ms(), 450.0);
    let conf = Config::from_data(json!({ "motion": { "launchSpeed": "fast", "fastMs": 120.0 } }));
    assert_eq!(conf.open_fade_ms(), 120.0);
    let conf = Config::from_data(json!({}));
    assert_eq!(conf.open_fade_from(), 0.0);
    let conf = Config::from_data(json!({ "general": { "openFadeFrom": 30.0 } }));
    assert!((conf.open_fade_from() - 0.3).abs() < 1e-6);
    let conf = Config::from_data(json!({ "general": { "openFadeFrom": 250.0 } }));
    assert_eq!(conf.open_fade_from(), 1.0);
}

#[test]
fn filter_swap_ms_floor() {
    let conf = sel(json!({}));
    assert_eq!(conf.filter_swap_ms(), 450.0);
    let conf = Config::from_data(json!({ "motion": { "slowMs": 900.0 } }));
    assert_eq!(conf.filter_swap_ms(), 900.0);
    let conf = Config::from_data(json!({ "motion": { "filterSwapSpeed": "fast", "fastMs": 0.0 } }));
    assert_eq!(conf.filter_swap_ms(), 35.0);
}

#[test]
fn card_flip_defaults() {
    let conf = Config::from_data(json!({}));
    assert_eq!(conf.card_flip_duration_ms(), 1_500.0);
    assert!(conf.card_flip_shader());
    assert!(conf.card_flip_back_reveal());

    let conf = Config::from_data(json!({
        "components": { "wallpaperSelector": {
            "flipDurationMs": 12.0,
            "flipShader": false,
            "flipBackReveal": false
        }}
    }));
    assert_eq!(conf.card_flip_duration_ms(), 100.0);
    assert!(!conf.card_flip_shader());
    assert!(!conf.card_flip_back_reveal());
}

#[test]
fn slice_getters() {
    let conf = sel(json!({
        "sliceHeight": 400.0, "visibleCount": 9, "expandedWidth": 800.0,
        "sliceWidth": 150.0, "sliceSpacing": -10.0, "skewOffset": 20.0,
        "sliceEdgeTilt": 42.0
    }));
    assert_eq!(conf.slice_height(), 400.0);
    assert_eq!(conf.visible_count(), 9);
    assert_eq!(conf.expanded_width(), 800.0);
    assert_eq!(conf.slice_width(), 150.0);
    assert_eq!(conf.slice_spacing(), -10.0);
    assert_eq!(conf.skew_offset(), 20.0);
    assert_eq!(conf.slice_edge_tilt(), 42.0);
}

#[test]
fn corners_round_toggle() {
    let off = sel(json!({"roundCorners": false, "cornerTL": 12.0}));
    assert_eq!(off.slice_corners(), [0.0; 4]);
    let on = sel(json!({
        "roundCorners": true, "cornerTL": 1.0, "cornerTR": 2.0, "cornerBR": 3.0, "cornerBL": 4.0
    }));
    assert_eq!(on.slice_corners(), [1.0, 2.0, 3.0, 4.0]);
    let base = sel(json!({"roundCorners": true, "cornerRadius": 8.0}));
    assert_eq!(base.slice_corners(), [8.0; 4]);
}

#[test]
fn hex_getters() {
    let conf = sel(json!({
        "hexRadius": 120.0, "hexRows": 4, "hexCols": 8, "hexScrollStep": 2,
        "hexArc": false, "hexArcIntensity": 1.5, "hexCurveFrequency": 2.5,
        "hexGapX": 20, "hexGapY": 12, "hexAspect": 135, "hexStagger": 65,
        "hexLens": 30, "hexLensRadius": 800, "hexStageX": 20, "hexStageY": -10,
        "hexStageScale": 125, "hexStageRotation": 8, "hexStagePerspective": 22,
        "hexStageShearX": 15, "hexStageShearY": -8, "hexStageDepthAngle": 42,
        "hexOrbit": -35, "hexOrbitRadius": 900, "hexTwist": 27, "hexScatter": 19,
        "hexShape": "diamond"
    }));
    assert_eq!(conf.hex_radius(), 120.0);
    assert_eq!(conf.hex_rows(), 4);
    assert_eq!(conf.hex_cols(), 8);
    assert_eq!(conf.hex_scroll_step(), 2);
    assert!(!conf.hex_arc());
    assert_eq!(conf.hex_arc_intensity(), 1.5);
    assert_eq!(conf.hex_curve(), "flat");
    assert_eq!(conf.hex_curve_frequency(), 2.5);
    assert_eq!(conf.hex_shape(), "diamond");
    assert_eq!((conf.hex_gap_x(), conf.hex_gap_y()), (20.0, 12.0));
    assert_eq!(conf.hex_aspect(), 1.35);
    assert_eq!(conf.hex_stagger(), 0.65);
    assert_eq!((conf.hex_lens(), conf.hex_lens_radius()), (0.3, 800.0));
    assert_eq!(conf.hex_stage(), [0.2, -0.1, 1.25, 8.0, 0.22, 0.15, -0.08, 42.0]);
    assert_eq!((conf.hex_orbit(), conf.hex_orbit_radius()), (-0.35, 900.0));
    assert_eq!((conf.hex_twist(), conf.hex_scatter()), (27.0, 19.0));
    let floors = sel(json!({"hexRows": 0, "hexCols": 1, "hexScrollStep": 0}));
    assert_eq!(floors.hex_rows(), 1);
    assert_eq!(floors.hex_cols(), 3);
    assert_eq!(floors.hex_scroll_step(), 1);
    assert!(sel(json!({})).hex_arc());
    assert_eq!(sel(json!({})).hex_shape(), "hexagon");
    assert_eq!(sel(json!({"hexShape": "triangle"})).hex_shape(), "triangle");
    assert_eq!(sel(json!({"hexShape": "diamond"})).hex_shape(), "diamond");
    assert_eq!(sel(json!({"hexShape": "rhombus"})).hex_shape(), "rhombus");
    assert_eq!(sel(json!({"hexShape": "pentagram"})).hex_shape(), "hexagon");
    assert_eq!(sel(json!({"hexShape": "made-up"})).hex_shape(), "hexagon");
}

#[test]
fn grid_getters() {
    let conf = sel(json!({
        "gridColumns": 5,
        "gridRows": 2,
        "gridThumbWidth": 280.0,
        "gridThumbHeight": 160.0,
        "gridGapX": 14.0,
        "gridGapY": 22.0,
        "gridRoundCorners": false,
        "gridCornerRadius": 18.0,
        "gridBorderWidth": 4.0, "gridLayout": "masonry", "gridStagger": 35,
        "gridSelectedScale": 140, "gridStageX": -15, "gridStageY": 12,
        "gridStageScale": 90, "gridStageRotation": -6, "gridStagePerspective": 18,
        "gridStageShearX": -12, "gridStageShearY": 7, "gridStageDepthAngle": -55,
        "gridFlowWave": 32, "gridFlowFrequency": 2.5, "gridScatter": 17,
        "gridScaleVariance": 28, "gridCylinderBend": -65, "gridCylinderRadius": 840
    }));
    assert_eq!(conf.grid_columns(), 5);
    assert_eq!(conf.grid_rows(), 2);
    assert_eq!(conf.grid_thumb_width(), 280.0);
    assert_eq!(conf.grid_thumb_height(), 160.0);
    assert_eq!(conf.grid_layout(), "masonry");
    assert_eq!(conf.grid_stagger(), 0.35);
    assert_eq!(conf.grid_selected_scale(), 1.4);
    assert_eq!(conf.grid_stage(), [-0.15, 0.12, 0.9, -6.0, 0.18, -0.12, 0.07, -55.0]);
    assert_eq!((conf.grid_flow_wave(), conf.grid_flow_frequency()), (32.0, 2.5));
    assert_eq!((conf.grid_scatter(), conf.grid_scale_variance()), (17.0, 0.28));
    assert_eq!((conf.grid_cylinder_bend(), conf.grid_cylinder_radius()), (-0.65, 840.0));
    assert_eq!(conf.grid_gap_x(), 14.0);
    assert_eq!(conf.grid_gap_y(), 22.0);
    assert!(!conf.grid_round_corners());
    assert_eq!(conf.grid_corner_radius(), 18.0);
    assert_eq!(conf.grid_border_width(), 4.0);

    let defaults = sel(json!({}));
    assert_eq!(defaults.grid_gap_x(), 8.0);
    assert_eq!(defaults.grid_gap_y(), 8.0);
    assert!(defaults.grid_round_corners());
    assert_eq!(defaults.grid_corner_radius(), 6.0);
    assert_eq!(defaults.grid_border_width(), 2.0);
    assert_eq!((defaults.grid_cylinder_bend(), defaults.grid_cylinder_radius()), (0.65, 720.0));
    assert_eq!(sel(json!({"gridLayout": "cylinder"})).grid_layout(), "cylinder");
}

#[test]
fn tag_cloud_dims_clamp() {
    let conf = sel(json!({"tagCloudWidth": 600.0, "tagCloudHeight": 200.0}));
    assert_eq!(conf.tag_cloud_width(), 600.0);
    assert_eq!(conf.tag_cloud_height(), 200.0);
    assert_eq!(conf.tag_cloud_rows(), 2);
    let lo = sel(json!({"tagCloudWidth": 10.0, "tagCloudHeight": 10.0}));
    assert_eq!(lo.tag_cloud_width(), 360.0);
    assert_eq!(lo.tag_cloud_height(), 90.0);
    assert_eq!(sel(json!({"tagCloudRows": "1"})).tag_cloud_rows(), 1);
    assert_eq!(sel(json!({"tagCloudRows": "3"})).tag_cloud_rows(), 3);
    assert_eq!(sel(json!({"tagCloudRows": "12"})).tag_cloud_rows(), 3);
    assert_eq!(sel(json!({"tagCloudRows": 2})).tag_cloud_rows(), 2);
}

#[test]
fn wobble_offset_getters() {
    assert!(!sel(json!({})).slice_wobble());
    assert!(sel(json!({"sliceWobble": true})).slice_wobble());
    assert!(!sel(json!({"sliceWobble": false})).slice_wobble());
    assert_eq!(sel(json!({"sliceWobbleStrength": 150.0})).slice_wobble_strength(), 1.5);
    assert_eq!(sel(json!({"sliceWobbleStrength": 500.0})).slice_wobble_strength(), 2.0);
    assert_eq!(sel(json!({})).slice_wobble_strength(), 1.0);
    assert_eq!(
        sel(json!({"sliceStageX": 25.0, "sliceStageY": -40.0})).slice_position(),
        (0.25, -0.4)
    );
    assert_eq!(sel(json!({})).slice_position(), (0.0, 0.0));
    assert_eq!(
        sel(json!({"sandyStageX": -30.0, "sandyStageY": 45.0})).sandy_position(),
        (-0.3, 0.45)
    );
    assert_eq!(sel(json!({})).sandy_position(), (0.0, 0.0));
    assert_eq!(
        sel(json!({"tagCloudOffsetX": 15.0, "tagCloudOffsetY": 25.0})).tag_cloud_offset(),
        (15.0, 25.0)
    );
    assert_eq!(sel(json!({})).tag_cloud_offset(), (0.0, 0.0));
    assert_eq!(
        cfg(json!({"filterBar": {"offsetX": 10.0, "offsetY": 20.0}})).filter_bar_offset(),
        (10.0, 20.0)
    );
}

#[test]
fn display_mode_getter() {
    assert_eq!(cfg(json!({})).display_mode(), "slices");
    assert_eq!(sel(json!({"displayMode": "hex"})).display_mode(), "hex");
}

#[test]
fn filter_bar_style_follows() {
    assert_eq!(cfg(json!({})).filter_bar_visual_style(), "slices");
    assert_eq!(
        cfg(json!({"components": {"wallpaperSelector": {"displayMode": "hex"}}}))
            .filter_bar_visual_style(),
        "hex"
    );
    assert_eq!(
        cfg(json!({"components": {"wallpaperSelector": {"displayMode": "sandy"}}}))
            .filter_bar_visual_style(),
        "slices"
    );
    assert_eq!(
        cfg(json!({
            "filterBar": {"visualStyle": "wall"},
            "components": {"wallpaperSelector": {"displayMode": "hex"}}
        }))
        .filter_bar_visual_style(),
        "wall"
    );
    assert_eq!(
        cfg(json!({"filterBar": {"visualStyle": "unknown"}})).filter_bar_visual_style(),
        "slices"
    );
    assert_eq!(
        cfg(json!({"filterBar": {"visualStyle": "grid"}})).filter_bar_visual_style(),
        "wall"
    );
    assert_eq!(
        cfg(json!({"filterBar": {"visualStyle": "nova"}})).filter_bar_visual_style(),
        "slices"
    );
}

#[test]
fn filter_bar_sandy_fallback() {
    assert_eq!(
        cfg(json!({"filterBar": {"visualStyle": "sandy"}})).filter_bar_visual_style(),
        "slices"
    );
    assert_eq!(
        cfg(json!({"components": {"wallpaperSelector": {"displayMode": "nova"}}}))
            .filter_bar_visual_style(),
        "slices"
    );
}

#[test]
fn filter_bar_orientation_bounds() {
    assert_eq!(cfg(json!({})).filter_bar_orientation(), "horizontal");
    assert_eq!(
        cfg(json!({"filterBar": {"orientation": "vertical"}})).filter_bar_orientation(),
        "vertical"
    );
    assert_eq!(
        cfg(json!({"filterBar": {"orientation": "diagonal"}})).filter_bar_orientation(),
        "horizontal"
    );
}

#[test]
fn general_getters() {
    let conf = cfg(json!({
        "general": {"uiScale": 1.5, "maxFps": 165.0,
                    "closeOnSelection": true, "filterBarAlwaysVisible": false},
        "wallpaperVolume": 70, "wallpaperMute": false
    }));
    assert_eq!(conf.ui_scale(), 1.5);
    assert_eq!(conf.max_fps(), 165.0);
    assert!(conf.close_on_selection());
    assert!(!conf.filter_bar_always_visible());
    assert_eq!(conf.wallpaper_volume(), 70);
    assert!(!conf.wallpaper_mute());
}

#[test]
fn general_defaults_clamps() {
    let def = cfg(json!({}));
    assert_eq!(def.ui_scale(), 1.0);
    assert_eq!(def.max_fps(), 120.0);
    assert!(!def.close_on_selection());
    assert!(def.filter_bar_always_visible());
    assert_eq!(def.wallpaper_volume(), 100);
    assert!(def.wallpaper_mute());
    assert_eq!(cfg(json!({"general": {"uiScale": 9.0}})).ui_scale(), 2.0);
    assert_eq!(cfg(json!({"wallpaperVolume": 999})).wallpaper_volume(), 100);
}

#[test]
fn sticky_filter_bar_getters() {
    let def = cfg(json!({}));
    assert!(!def.filter_bar_sticky());
    assert_eq!(def.last_filter_color(), -1);
    assert_eq!(def.last_filter_kind(), "");
    assert_eq!(def.last_filter_folder(), "*");
    assert_eq!(def.last_filter_sort(), "color");
    assert!(!def.last_filter_favourites());
    let conf = cfg(json!({"filterBar": {
        "sticky": true,
        "defaultFolder": "anime",
        "last": {"color": 5, "kind": "video", "sort": "date", "favouritesOnly": true}
    }}));
    assert!(conf.filter_bar_sticky());
    assert_eq!(conf.last_filter_color(), 5);
    assert_eq!(conf.last_filter_kind(), "video");
    assert_eq!(conf.last_filter_folder(), "anime");
    assert_eq!(conf.last_filter_sort(), "date");
    assert!(conf.last_filter_favourites());
}

#[test]
fn default_folder_all() {
    assert_eq!(cfg(json!({})).default_folder(), "*");
    assert_eq!(cfg(json!({"filterBar": {"defaultFolder": "anime"}})).default_folder(), "anime");
    assert_eq!(cfg(json!({"filterBar": {"defaultFolder": ""}})).default_folder(), "*");
    assert_eq!(cfg(json!({"filterBar": {"last": {"folder": ""}}})).last_filter_folder(), "*");
}

#[test]
fn filter_show_defaults() {
    let conf = cfg(json!({}));
    assert!(conf.filter_show("type.all"));
    assert!(conf.filter_show("type.we"));
    assert!(conf.filter_show("sort.date"));
    assert!(conf.filter_show("sort.applied"));
    assert!(conf.filter_show("folder"));
    let off = cfg(json!({"filterBar": {"show": {"type": {"we": false}, "sort": {"date": false}}}}));
    assert!(!off.filter_show("type.we"));
    assert!(off.filter_show("type.static"));
    assert!(!off.filter_show("sort.date"));
    assert!(off.filter_show("sort.color"));
}

#[test]
fn resolution_presets_dedup() {
    let conf = cfg(json!({
        "filterBar": {
            "resolutionPresets": [
                {"label": "FHD", "orientation": "wide", "from": "1920x1080", "to": "2559x1439"},
                {"label": "Duplicate", "orientation": "wide", "from": "1920x1080", "to": "2559x1439"},
                {"label": "4K", "orientation": "wide", "from": "3840x2160", "to": ""},
                {"label": "", "width": 2560, "height": 1440},
                {"label": "Broken", "from": "nope", "to": ""},
                {"label": "Backwards", "from": "3840x2160", "to": "1920x1080"}
            ]
        }
    }));
    let presets = conf.resolution_presets();
    assert_eq!(presets.len(), 2);
    assert_eq!(presets[0].label, "FHD");
    assert_eq!(presets[0].key(), "wide:1920x1080..2559x1439");
    assert_eq!(presets[0].orientation, "wide");
    assert_eq!(presets[1].label, "4K");
    assert_eq!(presets[1].key(), "wide:3840x2160..");
}

#[test]
fn browser_grid_sizes() {
    let conf = sel(json!({
        "wallhavenColumns": 5, "wallhavenRows": 4, "wallhavenThumbWidth": 250.0, "wallhavenThumbHeight": 150.0,
        "steamColumns": 8, "steamRows": 2, "steamThumbWidth": 320.0, "steamThumbHeight": 180.0,
        "wallhavenGapX": 14.0, "wallhavenGapY": 18.0,
        "wallhavenCornerRadius": 12.0, "wallhavenBorderWidth": 3.0
    }));
    let wh = conf.browser_grid(false);
    assert_eq!((wh.cols, wh.rows, wh.thumb_w, wh.thumb_h), (5, 4, 250.0, 150.0));
    assert_eq!((wh.gap_x, wh.gap_y, wh.corner_radius, wh.border_width), (14.0, 18.0, 12.0, 3.0));
    let st = conf.browser_grid(true);
    assert_eq!(st, wh);
    assert_eq!(wh.cell_h(), 168.0);

    let separate = sel(json!({
        "gridGapX": 99.0,
        "gridCornerRadius": 99.0,
        "wallhavenGapX": 7.0,
        "wallhavenCornerRadius": 5.0
    }));
    assert_eq!(separate.browser_grid(false).gap_x, 7.0);
    assert_eq!(separate.browser_grid(false).corner_radius, 5.0);

    let named = sel(json!({
        "downloaderWallColumns": 7,
        "downloaderWallRows": 2,
        "downloaderWallGapX": 9.0
    }));
    assert_eq!(named.browser_grid(false).cols, 7);
    assert_eq!(named.browser_grid(false).rows, 2);
    assert_eq!(named.browser_grid(false).gap_x, 9.0);
}

#[test]
fn selector_mode_degrades() {
    for (mode, expected) in [
        ("slices", "slices"),
        ("grid", "grid"),
        ("wall", "grid"),
        ("hex", "hex"),
        ("sandy", "sandy"),
        ("nova", "sandy"),
        ("hand", "hand"),
        ("depth", "depth"),
    ] {
        assert_eq!(sel(json!({"displayMode": mode})).selector_mode(), expected);
    }
    for mode in ["spiral", "mosaic", "cascade", "warp", "fluid", "lens", "shelves"] {
        assert_eq!(sel(json!({"displayMode": mode})).selector_mode(), "slices");
    }
}

#[test]
fn hand_getters_default_and_clamp() {
    let def = cfg(json!({}));
    assert_eq!(def.hand_count(), 5);
    assert_eq!(def.hand_card_width(), 168.0);
    assert_eq!(def.hand_card_height(), 432.0);
    assert_eq!(def.hand_spread(), 126.0);
    assert_eq!(def.hand_ribbons(), 6);
    assert_eq!(def.hand_speed(), 1.0);
    assert_eq!(def.hand_tilt(), 1.0);
    assert_eq!(def.hand_perspective(), 1700.0);
    assert_eq!(def.hand_ribbon_axis(), "rows");
    assert_eq!(def.hand_cut(), "straight");
    assert_eq!(def.hand_cut_variance(), "none");
    assert_eq!(def.hand_move(), "cycle");
    assert!(def.hand_ghosts());
    assert!(!def.hand_bob());
    assert!(def.hand_backdrop());
    assert!(def.hand_reveal_fill());
    assert!(!sel(json!({"handRevealFill": false})).hand_reveal_fill());

    let wild = sel(json!({
        "handCount": 40, "handRibbons": 0, "handSpeed": 5, "handPerspective": 10,
        "handRibbonAxis": "columns", "handCut": "steep", "handCutVariance": "wild",
        "handMove": "spiral", "handBob": true, "handStageX": 30.0, "handStageY": -10.0
    }));
    assert_eq!(wild.hand_count(), 16);
    assert_eq!(def.hand_fan_angle(), 12.0);
    assert_eq!(def.hand_fan_roll(), 8.5);
    assert_eq!(def.hand_arch(), 20.0);
    assert_eq!(def.hand_corner_radius(), 0.0);
    assert_eq!(def.hand_skew(), 0.0);
    assert_eq!(def.hand_backdrop_blur(), 1.0);
    assert_eq!(sel(json!({"handCount": 1})).hand_count(), 2);
    assert_eq!(sel(json!({"handFanAngle": 90})).hand_fan_angle(), 60.0);
    assert_eq!(sel(json!({"handSkew": -500})).hand_skew(), -200.0);
    assert_eq!(sel(json!({"handBackdropBlur": 250})).hand_backdrop_blur(), 2.5);
    assert_eq!(wild.hand_ribbons(), 2);
    assert_eq!(wild.hand_speed(), 0.35);
    assert_eq!(wild.hand_perspective(), 400.0);
    assert_eq!(wild.hand_ribbon_axis(), "columns");
    assert_eq!(wild.hand_cut(), "steep");
    assert_eq!(wild.hand_cut_variance(), "wild");
    assert_eq!(wild.hand_move(), "cycle");
    assert_eq!(wild.hand_moves(), [false, false, false, false, true]);
    let random = sel(json!({"handMove": "random", "handMoveShuffle": false}));
    assert_eq!(random.hand_move(), "random");
    assert_eq!(random.hand_moves(), [true, true, false, true, true]);
    assert!(wild.hand_bob());
    assert_eq!(wild.hand_position(), (0.3, -0.1));
    assert_eq!(sel(json!({"handMove": "bogus"})).hand_move(), "cycle");
    assert_eq!(sel(json!({"handMove": "bogus"})).hand_moves(), [true; 5]);
}

#[test]
fn sandy_arc_getters() {
    let def = cfg(json!({}));
    assert_eq!(def.sandy_arc(), 1.0);
    assert_eq!(def.sandy_blend(), 700.0);
    assert_eq!(def.sandy_slice_width(), 96.0);
    assert_eq!(def.sandy_slice_height(), 180.0);
    assert_eq!(def.sandy_skew(), 12.0);
    assert_eq!(def.sandy_edge_speed(), 14.0);
    assert_eq!(def.sandy_ring_size(), 1.0);
    assert_eq!(def.num_path(skwd_config::keys::selector::SANDY_RING_SIZE), 100.0);
    assert_eq!(def.sandy_ring_spin(), 1.0);
    assert_eq!(def.sandy_ring_soft(), 1.0);
    assert!((def.sandy_ring_hold() - 0.2).abs() < 1e-6);
    assert_eq!(def.sandy_grain(), 3.0);
    assert_eq!(sel(json!({"sandyGrain": 0.0})).sandy_grain(), 1.0);
    assert_eq!(sel(json!({"sandyGrain": 500.0})).sandy_grain(), 32.0);
    assert_eq!(def.sandy_res_scale(), 1.0);
    assert_eq!(sel(json!({"sandyResScale": 50.0})).sandy_res_scale(), 0.5);
    assert_eq!(sel(json!({"sandyResScale": 10.0})).sandy_res_scale(), 0.25);
    assert_eq!(sel(json!({"sandyResScale": 500.0})).sandy_res_scale(), 1.0);
    assert_eq!(def.sandy_lod(), 2.0);
    assert_eq!(sel(json!({"sandyLod": 0.0})).sandy_lod(), 1.0);
    assert_eq!(sel(json!({"sandyLod": 9.0})).sandy_lod(), 4.0);
    assert!(def.sandy_lod_auto());
    assert!(!sel(json!({"sandyLodAuto": false})).sandy_lod_auto());
    assert!(def.sandy_video_out_live());
    assert!(!sel(json!({"sandyOutgoingLive": false})).sandy_video_out_live());
    assert_eq!(sel(json!({"sandyRingSoft": 300.0})).sandy_ring_soft(), 3.0);
    assert_eq!(sel(json!({"sandyRingSize": 10.0})).sandy_ring_size(), 0.25);
    assert_eq!(sel(json!({"sandyRingSize": 400.0})).sandy_ring_size(), 3.0);
    assert_eq!(sel(json!({"sandyEdgeSpeed": 0.0})).sandy_edge_speed(), 0.0);
    assert_eq!(sel(json!({"sandySpacing": -30.0})).sandy_spacing(), -30.0);
    assert!(!def.sandy_swap_loop());
    assert_eq!(def.sandy_swap_style(), "vortex");
    assert_eq!(def.sandy_swap_style_index(), 1.0);
    assert_eq!(sandy_style_index("vortex"), 1.0);
    assert_eq!(sandy_style_index("hourglass"), 2.0);
    assert_eq!(sandy_style_index("castle"), 3.0);
    assert_eq!(sandy_style_index("saltation"), 6.0);
    assert_eq!(sandy_style_index("pour"), 7.0);
    assert_eq!(sandy_style_index("orbit"), 8.0);
    assert_eq!(sandy_style_index("burst"), 10.0);
    assert_eq!(sandy_style_index("weave"), 11.0);
    assert_eq!(sandy_style_index("bloom"), 13.0);
    assert_eq!(sandy_style_index("serpent"), 1.0);
    assert_eq!(sandy_style_index("geyser"), 1.0);
    assert_eq!(sandy_style_index("flock"), 16.0);
    assert_eq!(sandy_style_index("ring"), 17.0);
    for (key, _) in SANDY_SWAP_STYLES {
        assert!(sandy_style_index(key) > 0.0, "{key} unmapped");
    }
    assert_eq!(sandy_style_index("twister"), 1.0);
    assert_eq!(sandy_style_index("storm"), 1.0);
    assert_eq!(sandy_style_index("nonsense"), 1.0);
    assert_eq!(sandy_style_index("sweep"), 1.0);
    assert_eq!(sel(json!({"sandySwapStyle": "hourglass"})).sandy_swap_style(), "hourglass");
    assert_eq!(
        sel(json!({"sandyVortex": true, "sandySwapStyle": "storm"})).sandy_swap_style(),
        "storm"
    );
    assert_eq!(sel(json!({"sandyArc": 500.0})).sandy_arc(), 3.0);
    assert_eq!(sel(json!({"sandyArc": 0.0})).sandy_arc(), 0.0);
}

#[test]
fn video_preview_getters() {
    let conf = cfg(json!({"videoPreview": {"enabled": false, "delayMs": 500.0}}));
    assert!(!conf.video_preview_enabled());
    assert_eq!(conf.video_preview_delay_ms(), 500);
    assert_eq!(conf.video_preview_fps(), 30);
    let def = cfg(json!({}));
    assert!(def.video_preview_enabled());
    assert_eq!(cfg(json!({"videoPreview": {"delayMs": 99999.0}})).video_preview_delay_ms(), 3000);

    let mut battery = cfg(json!({}));
    battery.on_battery = true;
    assert_eq!(battery.video_preview_fps(), 20);

    let mut limited = cfg(json!({"general": {"maxFps": 12}}));
    limited.on_battery = true;
    assert_eq!(limited.video_preview_fps(), 12);
    assert_eq!(cfg(json!({"general": {"maxFps": 12.9}})).video_preview_fps(), 12);

    let mut disabled = cfg(json!({"performance": {"batterySaver": false}}));
    disabled.on_battery = true;
    assert_eq!(disabled.video_preview_fps(), 30);

    let mut uncapped = cfg(json!({"performance": {"batteryFps": 0}}));
    uncapped.on_battery = true;
    assert_eq!(uncapped.video_preview_fps(), 30);
}

#[test]
fn reload_partial_writes() {
    let mut conf = cfg(json!({"transition": {"durationMs": 4000.0}}));
    conf.config_path =
        std::env::temp_dir().join(format!("skwd_reload_test_{}.json", std::process::id()));
    conf.set_screen_width(1024.0);
    std::fs::write(&conf.config_path, r#"{"transition": {"durationMs": 4000.0}}"#).unwrap();
    assert!(!conf.reload());
    std::fs::write(&conf.config_path, r#"{"transition": {"durationMs": 750.0}}"#).unwrap();
    assert!(conf.reload());
    assert_eq!(conf.num_path("transition.durationMs"), 750.0);
    assert_eq!(conf.slice_height(), 360.0);
    std::fs::write(&conf.config_path, r#"{"transition": {"durat"#).unwrap();
    assert!(!conf.reload());
    assert_eq!(conf.num_path("transition.durationMs"), 750.0);
    let _ = std::fs::remove_file(&conf.config_path);
}

#[test]
fn transition_preview_fps_cap() {
    let conf = Config::from_data(json!({}));
    assert_eq!(conf.transition_preview_fps(), 30.0);
    assert_eq!(conf.transition_preview_interval(), std::time::Duration::from_secs_f32(1.0 / 30.0));

    let conf = Config::from_data(json!({ "transition": { "previewFps": 15.0 } }));
    assert_eq!(conf.transition_preview_fps(), 15.0);

    let conf = Config::from_data(json!({
        "transition": { "previewFps": 240.0 },
        "general": { "maxFps": 60.0 }
    }));
    assert_eq!(conf.transition_preview_fps(), 60.0);

    let conf = Config::from_data(json!({ "transition": { "previewFps": 0.0 } }));
    assert_eq!(conf.transition_preview_fps(), 1.0);

    let conf = Config::from_data(json!({ "transition": { "previewFps": -8.0 } }));
    assert_eq!(conf.transition_preview_fps(), 1.0);
}

#[test]
fn shared_config_parity() {
    let data = json!({
        "paths": { "wallpaper": "~/wp", "videoWallpaper": "~/vids", "cache": "/c" },
        "wallpaperVolume": 50.0,
        "wallpaperMute": false,
        "features": { "wallhaven": false, "steam": true },
        "sources": { "bing": { "enabled": true }, "unsplash": { "enabled": true, "accessKey": "k" } },
        "videoPreview": { "enabled": false, "delayMs": 250.0 },
        "theme": { "backend": "wallust" }
    });
    let gui = Config::from_data(data);
    let root = gui.root();

    assert_eq!(gui.wallpaper_volume(), skwd_config::wallpaper_volume(root));
    assert_eq!(gui.wallpaper_volume(), 50);
    assert_eq!(gui.source_enabled("wallhaven"), skwd_config::wallhaven_enabled(root));
    assert!(!gui.source_enabled("wallhaven"));
    assert_eq!(gui.source_enabled("steam"), skwd_config::steam_enabled(root));
    assert!(gui.source_enabled("bing"));
    assert!(gui.source_enabled("unsplash"));
    assert!(!gui.source_enabled("pexels"));
    assert!(!gui.source_enabled("youtube"));
    assert_eq!(gui.wallpaper_dir(), skwd_config::wallpaper_dir(root));
    assert_eq!(gui.video_dir(), skwd_config::video_dir(root));
    assert_eq!(gui.cache_dir(), skwd_config::cache_dir_of(root));
    assert_eq!(gui.theme_backend(), skwd_config::theme_backend(root));
    assert_eq!(gui.unsplash_access_key(), skwd_config::unsplash_access_key(root));
    assert_eq!(gui.wallpaper_mute(), skwd_config::wallpaper_mute(root));
    assert_eq!(gui.video_preview_enabled(), skwd_config::video_preview_enabled(root));
    assert_eq!(gui.video_preview_delay_ms(), skwd_config::video_preview_delay_ms(root));
}
