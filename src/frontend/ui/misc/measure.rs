use std::cell::RefCell;
use std::collections::HashMap;

use iced::Font;

use super::typography::measured_line_width;

const CACHE_CAP: usize = 512;

type WidthKey = (String, u32, Font);

thread_local! {
    static WIDTHS: RefCell<HashMap<WidthKey, f32>> = RefCell::new(HashMap::new());
}

fn remember<K: std::hash::Hash + Eq, V: Copy>(
    cell: &RefCell<HashMap<K, V>>,
    key: K,
    make: impl FnOnce() -> V,
) -> V {
    if let Some(found) = cell.borrow().get(&key) {
        return *found;
    }
    let value = make();
    let mut map = cell.borrow_mut();
    if map.len() >= CACHE_CAP {
        map.clear();
    }
    map.insert(key, value);
    value
}

pub(crate) fn cached_line_width(content: &str, size: f32, font: Font) -> f32 {
    let key = (content.to_string(), size.to_bits(), font);
    WIDTHS.with(|cell| remember(cell, key, || measured_line_width(content, size, font)))
}
