use super::*;
use crate::domain::studio::Source;

fn listed(group: Group) -> Vec<Source> {
    let count = match group {
        Group::Image => 6,
        Group::Theory => 7,
        Group::Themes => 30,
        Group::Matugen => 10,
    };
    (0..count).map(|index| Source::Method(format!("m{index}"))).collect()
}

#[test]
fn native_size_shrinks_late() {
    let wide = Frame::fit(Size::new(2556.0, 1440.0), 1.0);
    assert_eq!(wide.scale, 1.0);
    assert_eq!(
        wide.origin,
        Vector::new((2556.0 - WINDOW.width) / 2.0, (1440.0 - WINDOW.height) / 2.0)
    );
    let scaled = Frame::fit(Size::new(2732.0, 1536.0), 2.0);
    assert_eq!(scaled.scale, 2.0);
    let small = Frame::fit(Size::new(1024.0, 768.0), 1.0);
    assert!(small.scale < 1.0);
    assert_eq!(wide.design(Point::new(wide.origin.x, wide.origin.y)), Point::ORIGIN);
}

#[test]
fn image_group_lists_inline() {
    let plan = plan(0, Group::Image, listed, Group::Image);
    let opened = plan.opened.as_ref().expect("open");
    assert_eq!(opened.pane, None);
    assert_eq!(opened.top, NAV_TOP + SECTION_HEIGHT);
    assert_eq!(opened.height(), 5.0 * KID_STEP + KID_HEIGHT);
    assert_eq!(plan.sections.len(), 4);
    assert!(plan.sections[0].holds);
}

#[test]
fn themes_pane_scrolls() {
    let plan = plan(0, Group::Themes, listed, Group::Themes);
    let opened = plan.opened.as_ref().expect("open");
    assert_eq!(opened.pane, Some(PANE_HEIGHT));
    assert!(opened.max_scroll() > 0.0);
    let modes = modes_top(plan.bottom);
    assert!(wheel_size(modes).is_some());
}

#[test]
fn saved_sits_on_top() {
    let plan = plan(2, Group::Image, listed, Group::Image);
    assert_eq!(plan.sections[0].head, Head::Saved);
    assert_eq!(plan.sections[1].top, NAV_TOP + SECTION_HEIGHT + 2.0 * KID_STEP + SAVED_GAP);
}

#[test]
fn tabs_follow_design() {
    let tabs = tabs([0.0; Row::ALL.len()], f32::MAX);
    assert_eq!(tabs[Row::View.index()].x, TABS.x);
    let last = tabs[Row::Compare.index()];
    assert_eq!((last.x, last.width), (890.0, 126.0));
    assert!(last.y + last.height < crate::frontend::colour_studio::tokens::PREVIEW.y);
}

#[test]
fn long_labels_widen_tabs() {
    let mut needs = [0.0; Row::ALL.len()];
    needs[Row::Colour.index()] = 150.0;
    let right = f32::MAX;
    let tabs = tabs(needs, right);
    assert!(tabs[Row::Colour.index()].width > TAB_WIDTHS[Row::Colour.index()]);
    let last = tabs[Row::Compare.index()];
    assert!(last.x + last.width <= right);
}

#[test]
fn locale_tabs_clear_actions() {
    use crate::frontend::colour_studio::text::ROW_KEYS;
    use crate::frontend::colour_studio::tokens::{
        ACTIONS_PAD, BUTTON_GAP, CLOSE_RIGHT, PLATE_PAD, PLATE_TEXT, TAB_LABEL_SIZE, TAB_PAD,
    };
    use crate::frontend::ui::{UI_FONT, measured_line_width};
    use crate::i18n::{Catalog, LANGUAGES};

    for language in LANGUAGES {
        let catalog = Catalog::for_locale(language.tag);
        let needs = ROW_KEYS.map(|(_, label, _)| {
            TAB_PAD + measured_line_width(&catalog.format(label, None), TAB_LABEL_SIZE, UI_FONT)
        });
        let plate = |key| {
            measured_line_width(&catalog.format(key, None), PLATE_TEXT, UI_FONT) + 2.0 * PLATE_PAD
        };
        let apply = plate("colour-studio-use").max(plate("colour-studio-undo"));
        let limit = WINDOW.width
            - CLOSE_RIGHT
            - plate("colour-studio-close")
            - plate("colour-studio-save-as")
            - apply
            - 2.0 * BUTTON_GAP
            - ACTIONS_PAD;
        let tabs = tabs(needs, limit);
        let last = tabs[Row::Compare.index()];
        assert!(last.x + last.width <= limit + 0.01, "{}", language.tag);
    }
}
