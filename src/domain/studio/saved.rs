use skwd_palette::studio::{Area, Tuning};

use super::source::Source;

pub const SHOWN: usize = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct Saved {
    pub name: String,
    pub source: Source,
    pub area: Option<Area>,
    pub tuning: Tuning,
}

pub fn store(list: &[Saved], entry: Saved) -> Vec<Saved> {
    let mut kept: Vec<Saved> =
        list.iter().filter(|item| item.name != entry.name).cloned().collect();
    kept.push(entry);
    kept
}

pub fn shown(list: &[Saved]) -> &[Saved] {
    &list[list.len().saturating_sub(SHOWN)..]
}
