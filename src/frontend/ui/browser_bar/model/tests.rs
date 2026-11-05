use crate::frontend::browser::{Browser, Source};

use super::builder::browser_bar_items;
use super::cache::{browser_bar_items_compact, compact_reflow};

#[test]
fn provider_bars_fit_floor() {
    for source in Source::ALL {
        let browser = Browser::new(source);
        let items = browser_bar_items(&browser, 1.0);
        let width = items.iter().map(|(item, _)| item.x + item.w).fold(0.0f32, f32::max);
        let height = items.iter().map(|(item, _)| item.y + item.h).fold(0.0f32, f32::max);
        let (min_w, min_h) = crate::frontend::ui::MIN_VIEWPORT;
        assert!(width + 294.0 + 72.0 <= min_w, "{} bar is {width}px wide", source.label());
        assert!(height + 208.0 + 24.0 <= min_h, "{} bar is {height}px tall", source.label());
    }
}

#[test]
fn compact_filters_in_rail() {
    let max_width = 254.0;
    for source in Source::ALL {
        let browser = Browser::new(source);
        let items = browser_bar_items_compact(&browser, 1.0, max_width);
        if matches!(source, Source::Bing | Source::Steam | Source::Wallhaven) {
            assert!(items.is_empty());
            continue;
        }
        assert!(!items.is_empty(), "{} has no filter controls", source.label());
        assert!(
            items.iter().all(|(item, _)| item.x >= 0.0 && item.x + item.w <= max_width + 0.01),
            "{} overflows the compact rail",
            source.label()
        );
        assert!(items.iter().all(|(item, _)| item.y >= 0.0), "{} above the rail", source.label());
    }
}

#[test]
fn compact_reflow_memoized() {
    let mut browser = Browser::new(Source::Unsplash);
    let first = browser_bar_items_compact(&browser, 1.0, 254.0);
    let repeat = browser_bar_items_compact(&browser, 1.0, 254.0);
    assert!(std::rc::Rc::ptr_eq(&first, &repeat));

    let wider = browser_bar_items_compact(&browser, 1.0, 300.0);
    assert!(!std::rc::Rc::ptr_eq(&first, &wider));

    browser.request.catalog.unsplash.orientation = String::from("portrait");
    let refiltered = browser_bar_items_compact(&browser, 1.0, 300.0);
    assert!(!std::rc::Rc::ptr_eq(&wider, &refiltered));

    let settled = browser_bar_items_compact(&browser, 1.0, 300.0);
    assert!(std::rc::Rc::ptr_eq(&refiltered, &settled));
}

#[test]
fn compact_reflow_mirrors_in_rail() {
    let max_width = 254.0;
    for source in Source::ALL {
        let browser = Browser::new(source);
        let ltr = compact_reflow(&browser, 1.0, max_width, false);
        let rtl = compact_reflow(&browser, 1.0, max_width, true);
        assert_eq!(ltr.len(), rtl.len());
        for ((left, _), (right, _)) in ltr.iter().zip(&rtl) {
            assert_eq!(left.y, right.y);
            assert_eq!(left.w, right.w);
            assert!((right.x - (max_width - left.x - left.w)).abs() < 0.01, "{}", source.label());
            assert!(right.x >= -0.01 && right.x + right.w <= max_width + 0.01);
        }
        if let Some(((first, _), (mirrored, _))) = ltr.get(1).zip(rtl.get(1)) {
            assert!((first.x).abs() < 0.01);
            assert!((mirrored.x + mirrored.w - max_width).abs() < 0.01, "{}", source.label());
        }
    }
}
