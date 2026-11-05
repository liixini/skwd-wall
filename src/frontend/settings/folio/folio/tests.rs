use super::*;

#[test]
fn tracker_spans_rail() {
    assert_eq!(tracker_position(0.0, 11), 0.0);
    assert_eq!(tracker_position(10.0, 11), 1.0);
    assert_eq!(tracker_position(99.0, 11), 1.0);
    assert_eq!(tracker_position(2.5, 11), 0.25);
}

#[test]
fn button_wipe_diagonal() {
    assert_eq!(crate::frontend::ui::folio_diagonal_edges(120.0, 30.0, -1.0), (0.0, 0.0));
    assert_eq!(crate::frontend::ui::folio_diagonal_edges(120.0, 30.0, 0.0), (0.0, 0.0));
    let middle = crate::frontend::ui::folio_diagonal_edges(120.0, 30.0, 0.5);
    assert!((middle.0 - 38.4).abs() < 0.001);
    assert!((middle.1 - 81.6).abs() < 0.001);
    assert_eq!(crate::frontend::ui::folio_diagonal_edges(120.0, 30.0, 1.0), (120.0, 120.0));
    assert_eq!(crate::frontend::ui::folio_diagonal_edges(120.0, 30.0, 2.0), (120.0, 120.0));
}

#[test]
fn category_copy_complete() {
    for tab in crate::frontend::settings::tables::TABS {
        assert!(!super::super::super::tables::category_label(tab).starts_with("settings-tab-"));
    }
}

#[test]
fn glyph_fade_opaque() {
    assert_eq!(super::GLYPH_FADE, 1.0);
}

#[test]
fn small_type_floor() {
    assert_eq!(crate::frontend::ui::TYPE_SMALL, 11.0);
}

#[test]
fn editor_bars_match_kind() {
    let source = include_str!("../folio.rs");
    assert!(source.contains("control.is_inline_editor()"));
    assert!(source.contains("crate::frontend::ui::folio_inline_bar("));
    assert!(source.contains("crate::frontend::ui::folio_stack_bar("));
    assert!(source.contains("SettingsMsg::ToggleBar(id, index)"));
}

#[test]
fn sheet_bounds_fit() {
    for (viewport, scale) in
        [((1366.0, 768.0), 1.0), ((3840.0, 2160.0), 1.0), ((1366.0, 768.0), 1.5)]
    {
        let (width, height) = super::super::appearance::sheet_size(viewport, scale);
        assert!(width <= viewport.0 - METRICS.margin);
        assert!(height <= viewport.1 - METRICS.margin);
        assert!(width <= METRICS.width * scale);
    }
    assert_eq!(super::super::appearance::sheet_size((3840.0, 2160.0), 1.0), (1344.0, 746.0));
    assert_eq!(super::super::appearance::sheet_size((1366.0, 768.0), 1.0), (1344.0, 746.0));
    let scale = super::super::appearance::sheet_scale((1366.0, 768.0), 2.0);
    assert_eq!(scale, 1.0);
    assert_eq!(super::super::appearance::sheet_scale((3840.0, 2160.0), 2.0), 2.0);
}

#[test]
fn inline_vs_group_editors() {
    let inline = [
        Control::Number { key: "number".into(), path: String::new(), unit: "px" },
        Control::TextField {
            key: "text".into(),
            path: String::new(),
            placeholder: "path",
            secret: false,
        },
        Control::KeyBinding { key: "key".into(), path: String::new(), default: "ctrl+f" },
    ];
    assert!(inline.iter().all(Control::is_inline_editor));
    let grouped = Control::StackBar {
        id: String::from("bar"),
        summary: String::new(),
        preview: None,
        rows: Vec::new(),
    };
    assert!(grouped.is_group_editor());
    assert!(grouped.compact_bar_id().is_some());
    let toggle = Control::Toggle { path: String::new(), value: true };
    assert!(toggle.compact_bar_id().is_none());
    assert!(toggle.is_compact_action());
}

#[test]
fn toggles_use_action_bar() {
    let source = include_str!("../folio.rs");
    assert!(source.contains("fn compact_action_field("));
    assert!(source.contains("crate::frontend::ui::folio_action_bar("));
    assert!(source.contains("crate::frontend::ui::folio_action("));
}

#[test]
fn folio_no_ordinals() {
    let source = include_str!("../folio.rs");
    assert!(!source.contains("format!(\"{:02}\", index + 1)"));
}

#[test]
fn railless_sheet() {
    let source = include_str!("../folio.rs");
    for shared in [
        "super::appearance::Ornament::Frame {",
        "crate::frontend::ui::folio_sheet_panel_style(",
        "crate::frontend::ui::folio_scrim_style(",
        "folio_horizontal_rule(",
    ] {
        assert!(source.contains(shared), "{shared}");
    }
    for retired in [
        "folio_masthead(",
        "folio_masthead_with_back(",
        "settings-nav-crumb-root",
        "settings-nav-footer-hint",
        "settings-studio-controls",
    ] {
        assert!(!source.contains(retired), "{retired} came back");
    }
}

#[test]
fn reading_area_stays_visible() {
    let (width, _) = super::super::appearance::sheet_size((1366.0, 768.0), 1.0);
    let body = width - METRICS.index - reading_inset(1.0);
    assert!(body > METRICS.editor * 2.0, "two value columns fit");
}

#[test]
fn studio_back_and_close() {
    let source = include_str!("../folio.rs");
    assert!(source.contains("tr(\"settings-studio-back\")"));
    assert!(source.contains("SettingsMsg::LeaveLayoutStudio"));
    assert!(source.contains("Message::ToggleSettings"));
}

#[test]
fn workbench_ctx_structs() {
    let source = include_str!("../folio.rs");
    for shaped in [
        "pub struct SourceCtx<",
        "pub struct FocusCtx<",
        "pub struct ChromeCtx<",
        "struct ReadingInput<",
        "fn picker_layout_workbench<'a>(\n    source: &SourceCtx<'a>,",
        "fn reading_surface<'a>(input: ReadingInput<'a, '_>)",
    ] {
        assert!(source.contains(shaped), "{shaped}");
    }
}

#[test]
fn unchanged_values_stay_still() {
    let old = Row::of(
        "Volume".into(),
        "Wallpaper audio".into(),
        Control::Number { key: "volume".into(), path: "volume".into(), unit: "%" },
    );
    let inputs = HashMap::from([("volume".into(), "40".into())]);
    assert!(unchanged_row(&old, &old, &inputs, &inputs));
    let changed = HashMap::from([("volume".into(), "50".into())]);
    assert!(!unchanged_row(&old, &old, &inputs, &changed));
    let mut renamed = old.clone();
    renamed.title = "Brightness".into();
    assert!(!unchanged_row(&old, &renamed, &inputs, &inputs));
}
