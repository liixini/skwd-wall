use super::geometry::Rect;

pub const BASE: f32 = 42.0;
pub const BAND: f32 = 104.0;
pub const END: f32 = 24.0;
pub const BEAD: f32 = 9.0;
pub const HOVER: f32 = 12.0;
pub const STEP: f32 = 32.0;
pub const ROOM: f32 = 12.0;
pub const SPREAD: f32 = 14.0;
pub const ROW: f32 = 4.0;
pub const PLATE: f32 = 26.0;
pub const PLATE_GAP: f32 = 6.0;
pub const MARGIN: f32 = END + ROOM;
const PASSES: usize = 4;
const SLACK: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gaps {
    pub lead: f32,
    pub tail: f32,
    pub done: f32,
}

pub const NORMAL: Gaps = Gaps { lead: 24.0, tail: 40.0, done: 44.0 };
pub const COMPACT: Gaps = Gaps { lead: 14.0, tail: 22.0, done: 26.0 };

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KindSpec {
    pub plate: f32,
    pub group: f32,
    pub styles: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct StrandSpec<'a> {
    pub width: f32,
    pub kinds: &'a [KindSpec],
    pub cancel: f32,
    pub done: f32,
    pub extras: &'a [f32],
    pub open: Option<usize>,
    pub obstacles: &'a [[f32; 2]],
    pub margin: f32,
    pub dock: &'a [[f32; 2]],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item {
    Kind(usize),
    Group(usize),
    Bead(usize, usize),
    Extra(usize),
    Cancel,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    pub item: Item,
    pub x: f32,
    pub reach: [f32; 2],
    pub off: bool,
    pub top: Option<f32>,
}

impl Placed {
    pub fn centre(&self) -> f32 {
        self.x + f32::midpoint(self.reach[0], self.reach[1])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub items: Vec<Placed>,
    pub folded: Vec<bool>,
    pub fits: bool,
}

pub fn thread_y(height: f32) -> f32 {
    height - BASE
}

pub const fn zone_top(height: f32) -> f32 {
    height - BAND
}

pub fn zone_of(lane: f32) -> [f32; 2] {
    [lane - (BAND - BASE), lane + BASE]
}

pub fn obstacles(boxes: impl Iterator<Item = Rect>, height: f32) -> Vec<[f32; 2]> {
    let reach = PLATE / 2.0 + ROW;
    spans_in(boxes, [thread_y(height) - reach, thread_y(height) + reach])
}

pub fn spans_in(boxes: impl Iterator<Item = Rect>, band: [f32; 2]) -> Vec<[f32; 2]> {
    let [top, bottom] = band;
    let mut spans: Vec<[f32; 2]> = boxes
        .filter(|bounds| bounds.bottom() >= top && bounds.y <= bottom)
        .map(|bounds| [bounds.x, bounds.right()])
        .collect();
    spans.sort_by(|one, other| one[0].total_cmp(&other[0]));
    let mut merged: Vec<[f32; 2]> = Vec::new();
    for span in spans {
        match merged.last_mut() {
            Some(last) if span[0] <= last[1] => last[1] = last[1].max(span[1]),
            _ => merged.push(span),
        }
    }
    merged
}

fn build(spec: &StrandSpec<'_>, folded: &[bool], gaps: Gaps) -> Vec<Placed> {
    let mut items = Vec::new();
    let mut x = 0.0;
    for (kind, row) in spec.kinds.iter().enumerate() {
        if kind > 0 {
            x += gaps.tail;
        }
        items.push(Placed {
            item: Item::Kind(kind),
            x,
            reach: [0.0, row.plate],
            off: false,
            top: None,
        });
        x += row.plate + gaps.lead;
        let shut = folded[kind];
        let half = HOVER.max(row.group / 2.0);
        items.push(Placed {
            item: Item::Group(kind),
            x,
            reach: [-half, half],
            off: !shut,
            top: None,
        });
        for style in 0..row.styles {
            items.push(Placed {
                item: Item::Bead(kind, style),
                x,
                reach: [-HOVER, HOVER],
                off: shut,
                top: None,
            });
            if !shut && style + 1 < row.styles {
                x += STEP;
            }
        }
    }
    if spec.dock.is_empty() {
        x += gaps.done;
        for (item, width) in commands(spec) {
            items.push(Placed { item, x, reach: [0.0, width], off: false, top: None });
            x += width + PLATE_GAP;
        }
    }
    items
}

fn commands(spec: &StrandSpec<'_>) -> Vec<(Item, f32)> {
    let extras = spec.extras.iter().enumerate().map(|(index, width)| (Item::Extra(index), *width));
    extras.chain([(Item::Cancel, spec.cancel), (Item::Done, spec.done)]).collect()
}

fn lay(spec: &StrandSpec<'_>, folded: &[bool], gaps: Gaps) -> (Vec<Placed>, bool) {
    let mut items = build(spec, folded, gaps);
    let fits = place(&mut items, spec.width, spec.margin, spec.obstacles);
    items.extend(commands(spec).into_iter().zip(spec.dock).map(|((item, width), spot)| Placed {
        item,
        x: spot[0],
        reach: [0.0, width],
        off: false,
        top: Some(spot[1]),
    }));
    (items, fits)
}

fn place(items: &mut [Placed], width: f32, margin: f32, obstacles: &[[f32; 2]]) -> bool {
    let (low, high) = (margin, width - margin);
    let visible: Vec<usize> = (0..items.len()).filter(|index| !items[*index].off).collect();
    let (Some(&first), Some(&last)) = (visible.first(), visible.last()) else {
        return true;
    };
    let left = |index: usize| items[index].x + items[index].reach[0];
    let right = |index: usize| items[index].x + items[index].reach[1];
    let unit = |index: usize| match (items[index].item, items.get(index + 1)) {
        (Item::Kind(_), Some(next)) if matches!(next.item, Item::Group(_)) && !next.off => {
            right(index + 1)
        }
        _ => right(index),
    };
    let mut start = (width - (right(last) - left(first))) / 2.0 - left(first);
    let mut shifts = vec![start; items.len()];
    let mut fits = false;
    for _ in 0..PASSES {
        let mut shift = start;
        for &index in &visible {
            for &[from, to] in obstacles {
                let (near, far) = (left(index) + shift, unit(index) + shift);
                if near < to + SPREAD && far > from - SPREAD {
                    shift += to + SPREAD - near;
                }
            }
            shifts[index] = shift;
        }
        let begin = left(first) + shifts[first];
        let end = right(last) + shifts[last];
        fits = begin >= low - SLACK && end <= high + SLACK;
        if fits {
            break;
        }
        if end > high {
            start -= end - high;
        }
        if left(first) + start < low {
            start = low - left(first);
        }
    }
    let mut shift = start;
    for (index, item) in items.iter_mut().enumerate() {
        if !item.off {
            shift = shifts[index];
        }
        item.x += shift;
    }
    fits
}

pub fn plan(spec: &StrandSpec<'_>) -> Plan {
    let count = spec.kinds.len();
    let mut order: Vec<usize> = (0..count).filter(|kind| Some(*kind) != spec.open).collect();
    order.sort_by_key(|kind| std::cmp::Reverse(spec.kinds[*kind].styles));
    let folding = |shut: &[usize]| {
        let mut folded = vec![false; count];
        for kind in shut {
            folded[*kind] = true;
        }
        folded
    };
    let every: Vec<usize> = (0..count).collect();
    let tries = (0..=order.len())
        .map(|taken| (folding(&order[..taken]), NORMAL))
        .chain([(folding(&order), COMPACT), (folding(&every), COMPACT)]);
    let mut last = None;
    for (folded, gaps) in tries {
        let (items, fits) = lay(spec, &folded, gaps);
        if fits {
            return Plan { items, folded, fits };
        }
        last = Some(Plan { items, folded, fits });
    }
    last.expect("strand plans")
}

pub fn shut(spec: &StrandSpec<'_>) -> Plan {
    let folded: Vec<bool> = (0..spec.kinds.len()).map(|kind| Some(kind) != spec.open).collect();
    let (items, fits) = lay(spec, &folded, COMPACT);
    Plan { items, folded, fits }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
