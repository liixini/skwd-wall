#![cfg(test)]

use super::state::*;

use crate::domain::effects::{
    EffectDefinition, EffectOption, EffectParam, EffectParamKind, EffectValue,
};
use crate::domain::library::catalog::WallpaperKind;

fn definition(id: &str, category: &str, params: Vec<EffectParam>) -> EffectDefinition {
    EffectDefinition {
        id: id.into(),
        label: id.into(),
        description: String::new(),
        category: category.into(),
        params,
    }
}

fn param(id: &str, kind: EffectParamKind, default: Option<EffectValue>) -> EffectParam {
    let step = if matches!(&kind, EffectParamKind::Integer) { 1.0 } else { 0.01 };
    EffectParam {
        id: id.into(),
        label: id.into(),
        kind,
        min: 0.0,
        max: 100.0,
        step,
        default,
        options: Vec::new(),
    }
}

fn mon(name: &str, current: Option<&str>) -> MonitorInfo {
    MonitorInfo {
        name: name.into(),
        target: name.into(),
        connected: true,
        width: 2560,
        height: 1440,
        current_thumb: current.map(String::from),
        kind: WallpaperKind::Static,
        mute: true,
        volume: 100,
        fill: String::new(),
        locked: false,
        paused: false,
        manual_paused: false,
        current: String::new(),
        we_id: String::new(),
    }
}

#[test]
fn preview_orientation() {
    let landscape = mon("DP-1", None);
    let portrait = MonitorInfo { width: 1440, height: 2560, ..mon("DP-2", None) };

    let (landscape_width, landscape_height) = landscape.preview_dimensions(132.0, 88.0);
    let (portrait_width, portrait_height) = portrait.preview_dimensions(132.0, 88.0);

    assert!((landscape_width - 132.0).abs() < 0.01);
    assert!(landscape_width > landscape_height);
    assert!((portrait_height - 88.0).abs() < 0.01);
    assert!(portrait_height > portrait_width);
    assert_eq!(landscape.orientation_label_key(), "effects-orientation-landscape");
    assert_eq!(portrait.orientation_label_key(), "effects-orientation-portrait");
}

fn audible_mon(
    name: &str,
    kind: WallpaperKind,
    source: &str,
    mute: bool,
    volume: u32,
) -> MonitorInfo {
    let (current, we_id) = match kind {
        WallpaperKind::We => (String::new(), source.to_string()),
        _ => (source.to_string(), String::new()),
    };
    MonitorInfo {
        name: name.into(),
        target: name.into(),
        connected: true,
        width: 2560,
        height: 1440,
        current_thumb: None,
        kind,
        mute,
        volume,
        fill: String::new(),
        locked: false,
        paused: false,
        manual_paused: false,
        current,
        we_id,
    }
}

fn fixture() -> Effects {
    let mut fx = Effects::new(
        Vec::new(),
        "src".into(),
        Some("incoming".into()),
        0,
        WallpaperKind::Video,
        true,
        100,
        "video:src".into(),
    );
    fx.displays.monitors = vec![mon("DP-1", Some("cur1")), mon("DP-2", None)];
    fx
}

#[test]
fn apply_targets_selection() {
    let mut fx = fixture();
    fx.displays.monitors = vec![mon("DP-1", None), mon("DP-2", None), mon("DP-3", None)];

    fx.displays.selected_outputs.clear();
    fx.displays.selected_outputs.insert("DP-2".into());
    assert_eq!(fx.apply_targets(), vec!["DP-2".to_string()]);

    fx.displays.selected_outputs.insert("DP-1".into());
    let mut targets = fx.apply_targets();
    targets.sort();
    assert_eq!(targets, vec!["DP-1".to_string(), "DP-2".to_string()]);

    fx.displays.selected_outputs.insert("DP-3".into());
    assert_eq!(fx.apply_targets(), vec!["*".to_string()]);

    fx.displays.selected_outputs.clear();
    assert_eq!(fx.apply_targets(), vec!["*".to_string()]);
}

#[test]
fn tile_audio_states() {
    let mut fx = fixture();
    let mut playing = mon("DP-1", Some("cur1"));
    playing.kind = WallpaperKind::Video;
    assert_eq!(fx.tile_audio(&playing, false), TileAudio::Live);
    assert_eq!(fx.tile_audio(&playing, true), TileAudio::Live);

    let idle = mon("DP-2", None);
    assert_eq!(fx.tile_audio(&idle, false), TileAudio::None);
    assert_eq!(fx.tile_audio(&idle, true), TileAudio::PreApply);

    fx.preview.kind = WallpaperKind::Static;
    assert_eq!(fx.tile_audio(&idle, true), TileAudio::None);
    let mut we = mon("DP-3", None);
    we.kind = WallpaperKind::We;
    assert_eq!(fx.tile_audio(&we, false), TileAudio::Live);
}

#[test]
fn toggle_select_all() {
    let mut fx = fixture();
    assert!(!fx.all_selected());
    fx.toggle_output("DP-1");
    assert!(fx.displays.selected_outputs.contains("DP-1"));
    assert!(!fx.all_selected());
    fx.toggle_output("DP-1");
    assert!(!fx.displays.selected_outputs.contains("DP-1"));
    fx.toggle_all();
    assert!(fx.all_selected());
    assert_eq!(fx.displays.selected_outputs.len(), 2);
    fx.toggle_all();
    assert!(fx.displays.selected_outputs.is_empty());
}

#[test]
fn output_lock_updates_monitor_state() {
    let mut fx = fixture();
    assert!(!fx.monitors()[0].locked);
    fx.set_mon_locked("DP-1", true);
    assert!(fx.monitors()[0].locked);
    assert!(!fx.monitors()[1].locked);
}

#[test]
fn shared_audio_group() {
    let mut effects = fixture();
    effects.set_monitors(vec![
        audible_mon("DP-1", WallpaperKind::Video, "/same.mp4", false, 37),
        audible_mon("DP-2", WallpaperKind::Video, "/same.mp4", true, 90),
        audible_mon("DP-3", WallpaperKind::Video, "/other.mp4", false, 62),
    ]);
    assert_eq!(effects.audio_group_outputs("DP-2"), ["DP-1", "DP-2"]);
    assert_eq!(effects.mon_volume("DP-1"), Some(37));
    assert_eq!(effects.mon_volume("DP-2"), Some(37));
    assert!(!effects.monitors()[0].mute && !effects.monitors()[1].mute);
    assert_eq!(effects.mon_volume("DP-3"), Some(62));

    effects.set_monitors(vec![
        audible_mon("DP-1", WallpaperKind::We, "2057951800", true, 12),
        audible_mon("DP-2", WallpaperKind::We, "2057951800", false, 48),
        audible_mon("DP-3", WallpaperKind::We, "999", false, 73),
    ]);
    assert_eq!(effects.audio_group_outputs("DP-1"), ["DP-1", "DP-2"]);
    assert_eq!(effects.mon_volume("DP-1"), Some(48));
    assert_eq!(effects.mon_volume("DP-2"), Some(48));
    assert!(!effects.monitors()[0].mute && !effects.monitors()[1].mute);
    assert_eq!(effects.audio_group_outputs("DP-3"), ["DP-3"]);
}

#[test]
fn open_easing() {
    let mut fx = fixture();
    assert_eq!(fx.panel.animation.get("open"), 0.0);
    assert!(fx.animating());
    for _ in 0..120 {
        fx.tick(1.0 / 60.0);
    }
    assert!(fx.panel.animation.get("open") > 0.999);
    assert!(!fx.animating());
    assert!(fx.open_ease() >= 0.999);
}

#[test]
fn preview_crossfade() {
    let mut fx = fixture();
    assert_eq!(fx.panel.animation.get("fade"), 1.0);
    fx.begin_fade("a.png".into());
    assert_eq!(fx.panel.animation.get("fade"), 0.0);
    assert_eq!(fx.preview.fade_from.as_deref(), Some("incoming"));
    assert!(fx.animating());
    for _ in 0..200 {
        fx.tick(1.0 / 60.0);
    }
    assert!(fx.panel.animation.get("fade") > 0.999);
    fx.begin_fade("a.png".into());
    assert!(fx.panel.animation.get("fade") > 0.999);
    fx.begin_fade("b.png".into());
    assert_eq!(fx.panel.animation.get("fade"), 0.0);
    assert_eq!(fx.preview.fade_from.as_deref(), Some("a.png"));
}

#[test]
fn begin_fade_orphans() {
    let mut fx = fixture();
    assert_eq!(fx.display_source(), "incoming");
    assert_eq!(fx.begin_fade("a.png".into()), None);
    assert_eq!(fx.begin_fade("b.png".into()), None);
    assert_eq!(fx.begin_fade("c.png".into()), Some("a.png".to_string()));
    assert_eq!(fx.begin_fade("c.png".into()), None);
    assert!(fx.is_protected("src") && fx.is_protected("incoming"));
    assert!(!fx.is_protected("a.png"));
}

#[test]
fn reset_params_defaults() {
    let defs = vec![
        definition(
            "border",
            "Transform",
            vec![
                param("thickness", EffectParamKind::Integer, Some(EffectValue::Number(12.0))),
                param("color", EffectParamKind::Color, Some(EffectValue::Text("#000000".into()))),
                param("broken", EffectParamKind::Other(String::new()), None),
            ],
        ),
        definition("bare", "", Vec::new()),
    ];
    let mut fx = Effects::new(
        defs,
        "src".into(),
        None,
        0,
        WallpaperKind::Static,
        true,
        100,
        "static:src".into(),
    );
    assert_eq!(fx.editor.selected_id, "border");
    assert_eq!(fx.editor.values.get("thickness"), Some(&EffectValue::Number(12.0)));
    assert_eq!(fx.editor.values.get("color"), Some(&EffectValue::Text("#000000".into())));
    assert_eq!(fx.editor.values.len(), 2);

    fx.set_num("thickness", 40.0);
    fx.editor.selected_id = String::from("bare");
    fx.reset_params();
    assert!(fx.editor.values.is_empty());
    assert!(fx.parameter_values().is_empty());

    fx.editor.selected_id = String::from("missing");
    fx.reset_params();
    assert!(fx.editor.values.is_empty());
}

#[test]
fn saved_effects_order() {
    let mut radius = param("radius", EffectParamKind::Integer, Some(EffectValue::Number(4.0)));
    radius.max = 10.0;
    let defs = vec![
        definition("kuwahara", "Distort", vec![radius]),
        definition(
            "brightness",
            "Adjust",
            vec![param("factor", EffectParamKind::Number, Some(EffectValue::Number(1.1)))],
        ),
    ];
    let mut effects = Effects::new(
        defs,
        "src".into(),
        None,
        0,
        WallpaperKind::Static,
        true,
        100,
        "static:src".into(),
    );

    effects.set_num("radius", 8.0);
    effects.toggle_saved();
    effects.select_effect("brightness".into());
    effects.set_num("factor", 1.7);
    assert_eq!(
        effects.preview_effects().iter().map(|step| step.effect.as_str()).collect::<Vec<_>>(),
        ["kuwahara", "brightness"]
    );

    effects.toggle_saved();
    effects.select_effect("kuwahara".into());
    assert_eq!(effects.parameter_values().get("radius"), Some(&EffectValue::Number(8.0)));
    effects.set_num("radius", 6.0);
    assert_eq!(effects.saved_effects()[0].params["radius"], EffectValue::Number(6.0));

    effects.toggle_saved();
    assert!(!effects.selected_is_saved());
    assert_eq!(effects.saved_effects().len(), 1);
    assert_eq!(effects.saved_effects()[0].effect, "brightness");
    assert_eq!(effects.preview_effects()[0].effect, "brightness");
    effects.set_num("radius", 5.0);
    assert_eq!(
        effects.preview_effects().iter().map(|step| step.effect.as_str()).collect::<Vec<_>>(),
        ["brightness", "kuwahara"]
    );
}

#[test]
fn tile_thumb_swap() {
    let mut fx = fixture();
    let dp1 = fx.displays.monitors[0].clone();
    let dp2 = fx.displays.monitors[1].clone();
    assert_eq!(fx.tile_thumb(&dp1).as_deref(), Some("cur1"));
    assert_eq!(fx.tile_thumb(&dp2).as_deref(), Some("incoming"));
    fx.toggle_output("DP-1");
    assert_eq!(fx.tile_thumb(&dp1).as_deref(), Some("incoming"));
    fx.toggle_output("DP-1");
    fx.set_hover(Some("DP-1".into()));
    assert_eq!(fx.tile_thumb(&dp1).as_deref(), Some("incoming"));
    fx.set_hover(None);
    assert_eq!(fx.tile_thumb(&dp1).as_deref(), Some("cur1"));
}

#[test]
fn effect_switch_reveal() {
    let mut fx = fixture();
    fx.editor.definitions =
        vec![definition("a", "c", Vec::new()), definition("b", "c", Vec::new())];
    fx.select_effect(String::from("a"));
    assert_eq!(fx.panel.animation.get("params"), 0.0);
    assert!(fx.animating());
    for _ in 0..200 {
        fx.tick(1.0 / 60.0);
    }
    assert!(fx.panel.animation.get("params") > 0.999);
    let before = fx.panel.animation.get("params");
    fx.select_effect(String::from("a"));
    assert_eq!(fx.panel.animation.get("params"), before);
}

#[test]
fn tile_fade_settles() {
    let mut fx = fixture();
    for _ in 0..200 {
        fx.tick(1.0 / 60.0);
    }
    assert!(!fx.animating());
    fx.toggle_output("DP-1");
    fx.tick(1.0 / 60.0);
    let fade = fx.displays.tile_animation["DP-1"];
    assert!(fade > 0.0 && fade < 1.0);
    assert!(fx.animating());
    for _ in 0..200 {
        fx.tick(1.0 / 60.0);
    }
    assert_eq!(fx.displays.tile_animation["DP-1"], 1.0);
    assert!(!fx.animating());
}

#[test]
fn busy_spinner() {
    let mut fx = fixture();
    for _ in 0..200 {
        fx.tick(1.0 / 60.0);
    }
    assert!(!fx.animating());
    fx.panel.busy = true;
    fx.tick(1.0 / 60.0);
    assert!(fx.panel.busy_spin > 0.0);
    assert!(fx.animating());
    fx.panel.busy = false;
    fx.tick(1.0 / 60.0);
    assert_eq!(fx.panel.busy_spin, 0.0);
}

#[test]
fn effects_page_static_only() {
    let mut fx = fixture();
    assert_eq!(fx.preview.kind, WallpaperKind::Video);
    assert!(!fx.has_effects_page());
    fx.preview.kind = WallpaperKind::We;
    assert!(!fx.has_effects_page());
    fx.preview.kind = WallpaperKind::Static;
    assert!(fx.has_effects_page());
}

fn static_effects() -> Effects {
    Effects::new(
        Vec::new(),
        "src".into(),
        Some("t".into()),
        0,
        WallpaperKind::Static,
        true,
        100,
        "static:src".into(),
    )
}

#[test]
fn shader_effect_maps_ids() {
    let mut fx = static_effects();
    for (id, code) in [
        ("invert", 5),
        ("grayscale", 6),
        ("brightness", 1),
        ("contrast", 2),
        ("saturation", 3),
        ("gamma", 4),
        ("sepia", 7),
        ("hue", 8),
        ("temperature", 9),
        ("tint", 10),
        ("posterize", 11),
        ("solarize", 12),
        ("threshold", 13),
        ("duotone", 14),
        ("vignette", 15),
        ("dim", 16),
        ("flip", 17),
        ("mirror", 18),
    ] {
        fx.editor.selected_id = id.into();
        let Some(spec) = fx.shader_effect() else {
            panic!("{id}");
        };
        assert_eq!(spec.effect, code, "{id}");
    }
    for id in ["recolor", "blur", "gradientmap", "scanlines", "kaleidoscope"] {
        fx.editor.selected_id = id.into();
        assert!(fx.shader_effect().is_none(), "{id}");
    }
}

#[test]
fn shader_effect_reads_params() {
    let mut fx = static_effects();
    fx.editor.selected_id = "brightness".into();
    assert_eq!(fx.shader_effect().unwrap().params[0], 1.1);
    fx.editor.values.insert("factor".into(), EffectValue::Number(1.5));
    assert_eq!(fx.shader_effect().unwrap().params[0], 1.5);

    fx.editor.selected_id = "gamma".into();
    fx.editor.values.clear();
    fx.editor.values.insert("gamma".into(), EffectValue::Number(2.0));
    assert_eq!(fx.shader_effect().unwrap().params[0], 0.5);

    fx.editor.selected_id = "saturation".into();
    fx.editor.values.clear();
    fx.editor.values.insert("percentage".into(), EffectValue::Number(50.0));
    assert_eq!(fx.shader_effect().unwrap().params[0], 1.5);

    fx.editor.selected_id = "contrast".into();
    fx.editor.values.clear();
    fx.editor.values.insert("factor".into(), EffectValue::Number(50.0));
    assert_eq!(fx.shader_effect().unwrap().params[0], 1.5);
    fx.editor.values.insert("mode".into(), EffectValue::Text("sigmoid".into()));
    assert!(fx.shader_effect().is_none());

    fx.editor.selected_id = "duotone".into();
    fx.editor.values.clear();
    fx.editor.values.insert("shadow".into(), EffectValue::Text("#000000".into()));
    fx.editor.values.insert("highlight".into(), EffectValue::Text("#ffffff".into()));
    let spec = fx.shader_effect().unwrap();
    assert_eq!(spec.color_a, [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(spec.color_b, [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn shader_effect_gated_to_static() {
    let mut fx = static_effects();
    fx.editor.selected_id = "brightness".into();
    assert!(fx.shader_effect().is_some());
    fx.preview.kind = WallpaperKind::Video;
    assert!(fx.shader_effect().is_none());
}

#[test]
fn shader_live_texture_version() {
    let mut fx = static_effects();
    fx.editor.selected_id = "brightness".into();
    assert!(!fx.shader_live());
    fx.set_source_texture(vec![0u8; 16], 2, 2);
    assert!(fx.shader_live());
    assert_eq!((fx.preview.width, fx.preview.height), (2, 2));
    let v1 = fx.preview.version;
    fx.set_source_texture(vec![0u8; 16], 2, 2);
    assert!(fx.preview.version > v1);
    fx.editor.selected_id = "recolor".into();
    assert!(!fx.shader_live());
}

fn nav_effects() -> Effects {
    let mut theme =
        param("theme", EffectParamKind::Dropdown, Some(EffectValue::Text("Catppuccin".into())));
    theme.options = ["Catppuccin", "Nord", "Everforest"]
        .into_iter()
        .map(|name| EffectOption { mode: name.into(), label: name.into(), swatch: Vec::new() })
        .collect();
    let mut factor = param("factor", EffectParamKind::Number, Some(EffectValue::Number(1.1)));
    factor.max = 3.0;
    factor.step = 0.1;
    let defs = vec![
        definition("theme", "Colour", vec![theme]),
        definition(
            "duotone",
            "Colour",
            vec![param(
                "shadow",
                EffectParamKind::Color,
                Some(EffectValue::Text("#000000".into())),
            )],
        ),
        definition("brightness", "Adjust", vec![factor]),
        definition("invert", "Adjust", Vec::new()),
    ];
    Effects::new(
        defs,
        "src".into(),
        Some("t".into()),
        0,
        WallpaperKind::Static,
        true,
        100,
        "static:src".into(),
    )
}

#[test]
fn nav_move_clamp() {
    let mut fx = nav_effects();
    assert_eq!(fx.editor.selected_id, "theme");
    assert_eq!(fx.editor.nav_focus, NAV_EFFECT);
    assert_eq!(fx.nav_stops(), 3);

    fx.nav_move(-1);
    assert_eq!(fx.editor.nav_focus, NAV_CATEGORY);
    fx.nav_move(-1);
    assert_eq!(fx.editor.nav_focus, NAV_CATEGORY);
    fx.nav_move(1);
    fx.nav_move(1);
    assert_eq!(fx.editor.nav_focus, NAV_CONFIG_BASE);
    fx.nav_move(1);
    assert_eq!(fx.editor.nav_focus, NAV_CONFIG_BASE);
}

#[test]
fn nav_horizontal_steps() {
    let mut fx = nav_effects();

    fx.editor.nav_focus = NAV_EFFECT;
    assert!(matches!(fx.nav_horizontal(1), Some(EffectsMsg::Select(id)) if id == "duotone"));
    assert!(fx.nav_horizontal(-1).is_none());

    fx.editor.nav_focus = NAV_CATEGORY;
    assert!(matches!(fx.nav_horizontal(1), Some(EffectsMsg::Select(id)) if id == "brightness"));
    assert!(fx.nav_horizontal(-1).is_none());

    fx.editor.nav_focus = NAV_CONFIG_BASE;
    assert!(
        matches!(fx.nav_horizontal(1), Some(EffectsMsg::SetChoice(id, val)) if id == "theme" && val == "Nord")
    );
    assert!(fx.nav_horizontal(-1).is_none());
}

#[test]
fn nav_config_slider_steps() {
    let mut fx = nav_effects();
    fx.select_effect("brightness".into());
    fx.editor.nav_focus = NAV_CONFIG_BASE;
    let Some(EffectsMsg::SetNum(id, val)) = fx.nav_horizontal(1) else {
        panic!("expected slider step");
    };
    assert_eq!(id, "factor");
    assert!((val - 1.2).abs() < 1e-9);
}

#[test]
fn nav_focus_clamps() {
    let mut fx = nav_effects();
    fx.select_effect("theme".into());
    fx.editor.nav_focus = NAV_CONFIG_BASE;
    fx.select_effect("invert".into());
    assert_eq!(fx.nav_stops(), 2);
    assert_eq!(fx.editor.nav_focus, NAV_EFFECT);
}

#[test]
fn commit_veil_before_apply() {
    let mut fx = static_effects();
    assert!(fx.apply_ease() <= 0.003);
    fx.tick(0.2);
    assert!(fx.apply_ease() <= 0.003);
    fx.begin_apply();
    fx.panel.busy = true;
    fx.tick(0.05);
    assert!(fx.apply_ease() > 0.003);
}

#[test]
fn apply_holds_then_finishes() {
    let mut fx = static_effects();
    fx.editor.selected_id = "brightness".into();
    assert!(!fx.panel.committing);
    assert!(!fx.apply_finished());

    fx.begin_apply();
    assert!(fx.panel.committing);
    fx.panel.busy = true;
    for _ in 0..90 {
        fx.tick(1.0 / 60.0);
    }
    assert!(fx.apply_ease() > 0.99);
    assert!(!fx.apply_finished());

    fx.panel.busy = false;
    for _ in 0..40 {
        fx.tick(1.0 / 60.0);
    }
    assert!(fx.apply_finished());
}

#[test]
fn nav_scroll_follows_focus() {
    let mut fx = nav_effects();
    fx.editor.nav_focus = NAV_EFFECT;
    assert_eq!(fx.nav_scroll(), Some(("fx.reel", 0.0)));
    fx.select_effect("duotone".into());
    assert_eq!(fx.nav_scroll(), Some(("fx.reel", 1.0)));

    fx.select_effect("theme".into());
    fx.editor.nav_focus = NAV_CATEGORY;
    assert_eq!(fx.nav_scroll(), Some(("fx.cats", 0.0)));

    fx.editor.nav_focus = NAV_CONFIG_BASE;
    assert_eq!(fx.nav_scroll(), Some(("fx.params", 0.0)));
}

#[test]
fn index_shells_share_chrome() {
    let dock = include_str!("presentation/dock.rs");
    let displays = include_str!("presentation/displays.rs");
    assert!(dock.contains("crate::frontend::ui::folio_index_shell_tinted("));
    assert!(dock.contains("(0.78, 0.58),"));
    assert!(displays.contains("crate::frontend::ui::folio_index_shell("));
    for source in [dock, displays] {
        assert!(!source.contains("with_alpha(palette.background, 0.9)"));
        assert!(source.contains("folio_rule(palette)"));
    }
    let studio = include_str!("presentation/studio.rs");
    assert!(studio.contains("crate::frontend::ui::folio_scrim_style(ease)"));
    assert!(!studio.contains("folio_masthead("));
    assert_eq!(studio.matches("INDEX_WIDTH").count(), 2);
}

#[test]
fn editor_has_no_ordinals() {
    for source in [
        include_str!("presentation/dock.rs"),
        include_str!("presentation/stage.rs"),
        include_str!("presentation/studio.rs"),
        include_str!("presentation/displays.rs"),
        include_str!("presentation/monitor.rs"),
    ] {
        for retired in [
            "format!(\"{:02}\"",
            "label(\"01\"",
            "label(\"02\"",
            "effects-live-proof",
            "effects-live-canvas",
            "effects-controls-count",
        ] {
            assert!(!source.contains(retired), "{retired} came back");
        }
    }
}
