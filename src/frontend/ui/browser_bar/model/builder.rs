use crate::frontend::browser::{Browser, Source};

use super::super::super::bar::BarItem;
use super::item::push_item;
use super::types::BrowserAct;

pub(super) fn browser_bar_items(browser: &Browser, scale: f32) -> Vec<(BarItem, BrowserAct)> {
    let mut output = Vec::new();
    let mut x = 0.0;
    push_item(
        &mut output,
        scale,
        &mut x,
        0.0,
        "\u{f0156}",
        true,
        14.0 * scale,
        false,
        BrowserAct::Close,
    );
    x += 12.0 * scale;
    match browser.source {
        Source::Unsplash => super::unsplash::build_rows(browser, scale, x, &mut output),
        Source::Pexels => super::pexels::build_rows(browser, scale, x, &mut output),
        Source::Youtube => super::youtube::build_row(browser, scale, x, &mut output),
        Source::Wallhaven | Source::Steam | Source::Bing => {}
    }
    output
}
