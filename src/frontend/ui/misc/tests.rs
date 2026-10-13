#![cfg(test)]

#[test]
fn wrap_rows_break() {
    let items: Vec<(f32, u32)> = vec![(40.0, 0), (40.0, 1), (40.0, 2), (40.0, 3)];
    let rows = super::wrap_rows(items, 100.0, 8.0);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], vec![0, 1]);
    assert_eq!(rows[1], vec![2, 3]);
    let tight: Vec<(f32, u32)> = vec![(60.0, 0), (60.0, 1), (60.0, 2)];
    assert_eq!(super::wrap_rows(tight, 100.0, 8.0).len(), 3);
}

#[test]
fn wrap_rows_fit() {
    let items: Vec<(f32, u32)> = vec![(20.0, 0), (20.0, 1)];
    let rows = super::wrap_rows(items, 500.0, 8.0);
    assert_eq!(rows.len(), 1);
}

#[test]
fn ellipsize_width_budget() {
    let label = "fixing-resolution-for-red-blue-nord-black-3840x2160-v0-aznfxe7faquf1";
    let fitted = super::ellipsize_text(label, 38.0, 640.0);
    assert!(fitted.ends_with('…'));
    assert!(super::text_width(&fitted, 38.0, false) <= 640.0);
    assert_eq!(super::ellipsize_text("short title", 38.0, 640.0), "short title");
}

#[test]
fn folio_sheet_dims_cap() {
    let (w, h) = super::folio_sheet_dims(super::MIN_VIEWPORT, 1.0);
    assert_eq!((w, h), (1344.0, 746.0));
    assert!(w <= super::MIN_VIEWPORT.0 - 22.0 + 0.01);
    assert_eq!(super::folio_sheet_dims((1657.0, 1326.0), 0.5), (695.0, 435.0));
    assert_eq!(super::folio_sheet_dims((200.0, 150.0), 1.0), (220.0, 160.0));
}

#[test]
fn folio_sheet_dims_viewport() {
    assert_eq!(super::folio_sheet_dims((1657.0, 1326.0), 1.0), (1390.0, 870.0));
    assert_eq!(super::folio_sheet_dims((900.0, 700.0), 1.0), (878.0, 678.0));
}

#[test]
fn folio_scroll_padding_bottom() {
    let padding = super::folio_scroll_padding(23.0, 27.0, 1.0);
    assert_eq!(
        (padding.top, padding.right, padding.bottom, padding.left),
        (23.0, 27.0, 24.0, 27.0)
    );
    let scaled = super::folio_scroll_padding(35.0, 40.0, 1.5);
    assert_eq!((scaled.top, scaled.right, scaled.bottom, scaled.left), (52.5, 60.0, 54.0, 60.0));
}

#[test]
fn text_width_nerd_glyphs() {
    assert_eq!(super::text_width("\u{f0156}", 10.0, true), super::glyph_width(10.0));
    assert_eq!(super::text_width("\u{f0156}\u{f03d8}", 10.0, true), 2.0 * super::glyph_width(10.0));
    assert_eq!(super::text_width("abcd", 10.0, false), 4.0 * 10.0 * 0.64);
}

#[test]
fn type_scale_sentence_case() {
    assert_eq!(super::legible_type_scale(0.0), 0.95);
    assert_eq!(super::legible_type_scale(0.95), 0.95);
    assert_eq!(super::legible_type_scale(1.25), 1.25);
    assert_eq!(super::sentence_case("MOTION"), "Motion");
    assert_eq!(super::sentence_case(""), "");
}

#[test]
fn folio_rule_thickness() {
    assert_eq!(super::layout::FOLIO_RULE_THICKNESS, 1.0);
}

#[test]
fn folio_line_selected_focus() {
    let palette = crate::frontend::theme::Palette::default();
    let selected = super::folio_line_button_style(
        true,
        false,
        &palette,
        1.0,
        iced::widget::button::Status::Active,
    );
    let Some(iced::Background::Color(fill)) = selected.background else {
        panic!("no selection wash");
    };
    assert!((fill.a - 0.18).abs() < f32::EPSILON);
    assert_eq!(selected.border.width, 0.0);

    let focused = super::folio_line_button_style(
        true,
        true,
        &palette,
        1.0,
        iced::widget::button::Status::Active,
    );
    assert_eq!(focused.border.width, 2.0);
    assert_eq!(focused.border.color, palette.primary);
}

#[test]
fn folio_action_disabled() {
    let palette = crate::frontend::theme::Palette::default();
    let disabled =
        super::folio_button_style(false, false, &palette, iced::widget::button::Status::Disabled);
    assert!(disabled.background.is_none());
    assert!((disabled.text_color.a - 0.34).abs() < f32::EPSILON);
    assert!((disabled.border.color.a - 0.18).abs() < f32::EPSILON);
}

#[test]
fn scrim_neutral() {
    let col = super::scrim(0.5);
    assert_eq!((col.r, col.g, col.b), (0.0, 0.0, 0.0));
    assert!((col.a - 0.5).abs() < f32::EPSILON);
}

#[test]
fn style_ratchet() {
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    let allowed: HashMap<&str, [usize; 3]> = HashMap::from([
        ("frontend/tagcloud/tagcloud.rs", [1, 0, 1]),
        ("frontend/browser/view.rs", [1, 4, 1]),
        ("app/view.rs", [2, 0, 0]),
        ("frontend/effects/presentation/controls.rs", [1, 0, 1]),
        ("frontend/effects/presentation/monitor.rs", [1, 0, 0]),
        ("frontend/effects/presentation/stage.rs", [1, 0, 0]),
        ("frontend/settings/inputs.rs", [0, 0, 1]),
        ("frontend/settings/folio/folio.rs", [0, 2, 0]),
        ("frontend/settings/folio/control.rs", [0, 2, 0]),
    ]);
    let patterns = ["container::Style {", "button::Style {", "text_input::Style {"];
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let frontend = src.join("frontend");
    let mut offenders = Vec::new();
    let mut stack = vec![src.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read src") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                let shared_ui = path.file_name().is_some_and(|name| name == "ui")
                    && matches!(
                        path.parent(),
                        Some(parent) if parent == src || parent == frontend
                    );
                if shared_ui {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let rel = path.strip_prefix(&src).unwrap_or(&path).display().to_string();
            if rel.ends_with("tests.rs") {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap_or_default();
            let caps = allowed.get(rel.as_str()).copied().unwrap_or([0, 0, 0]);
            for (idx, pat) in patterns.iter().enumerate() {
                let count = text.matches(pat).count();
                if count > caps[idx] {
                    offenders.push(format!("{rel}: {count}x `{pat}` (allowed {})", caps[idx]));
                }
            }
        }
    }
    assert!(offenders.is_empty(), "new style closures: {offenders:#?}");
}

#[test]
fn no_hardcoded_colours() {
    let sources = [
        include_str!("../chrome/badges.rs"),
        include_str!("../chrome/canvas.rs"),
        include_str!("../chrome/panel.rs"),
        include_str!("../chrome/selection.rs"),
        include_str!("../chrome/back/actions.rs"),
        include_str!("../chrome/back/background.rs"),
        include_str!("../chrome/back/content.rs"),
        include_str!("../bar/view.rs"),
        include_str!("../theme_bar.rs"),
    ];
    for src in sources {
        for (num, line) in src.lines().enumerate() {
            let code = line.split("//").next().unwrap_or(line);
            assert!(!code.contains("Color::WHITE"), "line {} white: {}", num + 1, code.trim());
        }
    }
}

fn walk_sources(body: &mut dyn FnMut(&str, &str)) {
    use std::fs;
    use std::path::Path;

    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut stack = vec![src.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read src") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }
            let rel = path.strip_prefix(&src).unwrap_or(&path).display().to_string();
            if rel.ends_with("tests.rs") {
                continue;
            }
            body(&rel, &fs::read_to_string(&path).unwrap_or_default());
        }
    }
}

#[test]
fn folio_scrim_ratchet() {
    use std::collections::HashMap;

    assert_eq!(super::folio::FOLIO_SCRIM_ALPHA, 0.68);
    let allowed: HashMap<&str, usize> = HashMap::from([
        ("frontend/ui/misc/folio.rs", 1),
        ("frontend/ui/misc/style.rs", 1),
        ("frontend/ui/help.rs", 1),
        ("frontend/playlists/card_picker.rs", 1),
    ]);
    let mut offenders = Vec::new();
    walk_sources(&mut |rel, text| {
        let cleaned = text
            .replace("folio_scrim_style(", "")
            .replace("fn scrim_style(", "")
            .replace("fn scrim(", "");
        let count = cleaned.matches("scrim_style(").count() + cleaned.matches("scrim(").count();
        let cap = allowed.get(rel).copied().unwrap_or(0);
        if count > cap {
            offenders.push(format!("{rel}: {count} raw scrims (allowed {cap})"));
        }
    });
    assert!(offenders.is_empty(), "raw scrims: {offenders:#?}");
}

#[test]
fn folio_advance_ratchet() {
    let mut offenders = Vec::new();
    walk_sources(&mut |rel, text| {
        if !rel.starts_with("frontend/ui/misc/") && text.contains("chars().count() as f32 * 6.") {
            offenders.push(rel.to_string());
        }
    });
    assert!(offenders.is_empty(), "raw glyph advance: {offenders:#?}");
}

#[test]
fn folio_action_width_variants() {
    assert_eq!(super::folio_action_width("retag", 1.0), 5.0 * 6.4 + 28.0);
    assert_eq!(super::folio_action_width("a", 1.0), 52.0);
    assert_eq!(super::folio_action_width(&"x".repeat(80), 1.0), 190.0);
    assert_eq!(super::folio_action_width("retag", 2.0), 2.0 * (5.0 * 6.4 + 28.0));
    assert_eq!(super::folio_action_width_in("retag", 1.0, 52.0, 190.0), 5.0 * 6.4 + 28.0);
    assert_eq!(super::folio_action_width_in("a", 1.0, 92.0, f32::INFINITY), 92.0);
    assert_eq!(super::folio_action_width_in(&"x".repeat(40), 1.0, 52.0, 132.0), 132.0);
}

#[test]
fn folio_sheet_panel_chrome() {
    let palette = crate::frontend::theme::Palette::default();
    let opening = super::folio_sheet_panel_style(&palette, 0.0);
    let open = super::folio_sheet_panel_style(&palette, 1.0);
    assert_eq!(opening.shadow.color.a, 0.0);
    assert!((open.shadow.color.a - 0.52).abs() < f32::EPSILON);
    assert_eq!(opening.background, open.background);
    assert_eq!(opening.border.color, open.border.color);
    assert!((super::folio::FOLIO_INDEX_BG_ALPHA - 0.9).abs() < f32::EPSILON);
    assert!((super::folio::FOLIO_INDEX_OUTLINE_ALPHA - 0.46).abs() < f32::EPSILON);
}

#[test]
fn folio_rule_alpha() {
    assert_eq!(super::layout::FOLIO_RULE_ALPHA, 0.58);
}

#[test]
fn settings_editor_and_toggle_bars_allow_content_to_set_height() {
    let palette = crate::frontend::theme::Palette::default();
    for scale in [0.8, 1.0, 1.5, 2.0] {
        for inline in [false, true] {
            let content =
                iced::widget::text("A label\nHelp text\nthat wraps\nonto more\nthan three lines")
                    .width(120.0)
                    .into();
            let action = iced::widget::text("30").into();
            let bar = if inline {
                super::folio_inline_bar(content, false, (), action, scale, &palette, 1.0)
            } else {
                super::folio_action_bar(content, false, (), action, scale, &palette, 1.0)
            };
            assert_eq!(bar.as_widget().size_hint().height, iced::Length::Shrink);
        }
    }
}

#[test]
fn rtl_canvas_text_anchors_its_measured_width() {
    use iced::advanced::text::Alignment as TextAlignment;
    use iced::{Alignment, Color, Point};
    let latin = super::mid_text(
        "ALL",
        Point::new(100.0, 8.0),
        Color::WHITE,
        11.0,
        super::UI_FONT,
        Alignment::Center,
    );
    assert_eq!(latin.align_x, TextAlignment::Center);
    assert_eq!(latin.position, Point::new(100.0, 8.0));

    let arabic = super::mid_text(
        "الكل",
        Point::new(100.0, 8.0),
        Color::WHITE,
        11.0,
        super::UI_FONT,
        Alignment::Center,
    );
    let width = super::typography::measured_line_width("الكل", 11.0, super::UI_FONT);
    assert!(width > 0.0 && width.is_finite());
    assert_eq!(arabic.align_x, TextAlignment::Default);
    assert_eq!(arabic.position, Point::new(100.0 - width / 2.0, 8.0));

    let trailing = super::mid_text(
        "صور",
        Point::new(100.0, 8.0),
        Color::WHITE,
        11.0,
        super::UI_FONT,
        Alignment::End,
    );
    let width = super::typography::measured_line_width("صور", 11.0, super::UI_FONT);
    assert_eq!(trailing.align_x, TextAlignment::Default);
    assert_eq!(trailing.position.x, 100.0 - width);

    let multiline = super::mid_text(
        "الكل\nصور",
        Point::new(100.0, 8.0),
        Color::WHITE,
        11.0,
        super::UI_FONT,
        Alignment::Center,
    );
    assert_eq!(multiline.align_x, TextAlignment::Center);
    assert!(super::typography::has_rtl("اردو") && !super::typography::has_rtl("हिन्दी"));
}

#[test]
fn reading_direction_helpers_mirror_only_for_rtl() {
    use iced::alignment::Horizontal;
    assert_eq!(super::ordered(vec![1, 2, 3], false), vec![1, 2, 3]);
    assert_eq!(super::ordered(vec![1, 2, 3], true), vec![3, 2, 1]);
    assert_eq!(super::start_for(false), Horizontal::Left);
    assert_eq!(super::start_for(true), Horizontal::Right);
    assert_eq!(super::end_for(false), Horizontal::Right);
    assert_eq!(super::end_for(true), Horizontal::Left);
    let ltr = super::logical_padding_for(false, 1.0, 2.0, 3.0, 4.0);
    assert_eq!((ltr.top, ltr.right, ltr.bottom, ltr.left), (1.0, 2.0, 3.0, 4.0));
    let rtl = super::logical_padding_for(true, 1.0, 2.0, 3.0, 4.0);
    assert_eq!((rtl.top, rtl.right, rtl.bottom, rtl.left), (1.0, 4.0, 3.0, 2.0));
    assert_eq!(super::mirror_x_for(false, 10.0, 30.0, 100.0), 10.0);
    assert_eq!(super::mirror_x_for(true, 10.0, 30.0, 100.0), 60.0);
    let flow: super::Flow<'_, ()> = super::row![iced::widget::text("a"), iced::widget::text("b")];
    assert_eq!(flow.len(), 2);
    let empty: super::Flow<'_, ()> = super::row![];
    assert!(empty.push(iced::widget::text("c")).len() == 1);
}

#[test]
fn flow_rows_enclose_filling_children_like_iced_rows() {
    use iced::Length;
    let filling: iced::Element<'_, ()> =
        super::row![iced::widget::text("a").width(Length::Fill), iced::widget::text("b")].into();
    assert_eq!(filling.as_widget().size_hint().width, Length::Fill);
    let shrinking: iced::Element<'_, ()> = super::row![iced::widget::text("a")].into();
    assert_eq!(shrinking.as_widget().size_hint().width, Length::Shrink);
    let fixed: iced::Element<'_, ()> =
        super::row![iced::widget::text("a").width(Length::Fill)].width(Length::Fixed(40.0)).into();
    assert_eq!(fixed.as_widget().size_hint().width, Length::Fixed(40.0));
    let tall: iced::Element<'_, ()> =
        super::row![iced::widget::text("a").height(Length::Fill)].into();
    assert_eq!(tall.as_widget().size_hint().height, Length::Fill);
}

#[test]
fn view_rows_come_from_the_direction_aware_kit() {
    fn visit(directory: &std::path::Path, offenders: &mut Vec<String>) {
        for entry in std::fs::read_dir(directory).expect("read view directory") {
            let path = entry.expect("view entry").path();
            if path.is_dir() {
                visit(&path, offenders);
                continue;
            }
            if path.extension().is_none_or(|extension| extension != "rs")
                || path.ends_with("dir.rs")
                || path.file_name().is_some_and(|name| name == "tests.rs")
            {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("read view source");
            let imports_iced_row = source.lines().any(|line| {
                line.starts_with("use iced::widget::")
                    && line.split(['{', '}', ',', ' ']).any(|token| token == "row")
            }) || source.contains("iced::widget::row!")
                || source.contains("widget::row(")
                || source.contains("Row::with_children(")
                || source.contains("Row::new(");
            if imports_iced_row {
                offenders.push(path.display().to_string());
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders = Vec::new();
    visit(&root.join("src/frontend"), &mut offenders);
    visit(&root.join("src/app/view"), &mut offenders);
    assert!(offenders.is_empty(), "{offenders:#?}");
}
