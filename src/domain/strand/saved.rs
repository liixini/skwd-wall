use std::collections::VecDeque;

use paper_control::Widget;

#[derive(Debug, Clone, PartialEq)]
pub enum Saved {
    Kept(usize),
    Placed(Widget),
}

pub fn merge<T>(
    raw: Vec<T>,
    slots: &[usize],
    saved: Vec<Saved>,
    encode: impl Fn(&Widget) -> T,
) -> Vec<T> {
    let mut cells: Vec<Option<T>> = raw.into_iter().map(Some).collect();
    let mut fresh: VecDeque<T> = saved
        .into_iter()
        .filter_map(|item| match item {
            Saved::Kept(index) => cells.get_mut(*slots.get(index)?)?.take(),
            Saved::Placed(widget) => Some(encode(&widget)),
        })
        .collect();
    let mut merged = Vec::with_capacity(cells.len() + fresh.len());
    for (position, cell) in cells.into_iter().enumerate() {
        if slots.contains(&position) {
            merged.extend(fresh.pop_front());
        } else {
            merged.extend(cell);
        }
    }
    merged.extend(fresh);
    merged
}

#[cfg(test)]
#[path = "saved_tests.rs"]
mod tests;
