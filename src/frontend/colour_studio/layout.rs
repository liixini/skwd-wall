use iced::{Point, Rectangle, Size, Vector};

use super::controls::Row;
use super::tokens::{
    GROUP_GAP, INDEX_BOTTOM, INDEX_WIDTH, KID_HEIGHT, KID_STEP, MODES_FLOOR, MODES_GAP, NAV_TOP,
    PANE_HEIGHT, SAVED_GAP, SCREEN, SECTION_HEIGHT, TAB_HEIGHT, TAB_WIDTHS, TABS,
    WHEEL_BELOW_MODES, WHEEL_CAPTION, WHEEL_MAX, WHEEL_MIN, WINDOW,
};
use crate::domain::studio::{Group, Source};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub origin: Vector,
    pub scale: f32,
    pub viewport: Size,
}

impl Frame {
    pub fn fit(viewport: Size, ui_scale: f32) -> Self {
        let room = (viewport.width / SCREEN.width).min(viewport.height / SCREEN.height);
        let scale = ui_scale.min(room);
        let origin = Vector::new(
            (viewport.width - WINDOW.width * scale) / 2.0,
            (viewport.height - WINDOW.height * scale) / 2.0,
        );
        Self { origin, scale, viewport }
    }

    pub fn design(&self, at: Point) -> Point {
        Point::new((at.x - self.origin.x) / self.scale, (at.y - self.origin.y) / self.scale)
    }

    pub fn viewport_design(&self) -> Rectangle {
        let corner = self.design(Point::ORIGIN);
        Rectangle {
            x: corner.x,
            y: corner.y,
            width: self.viewport.width / self.scale,
            height: self.viewport.height / self.scale,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Head {
    Saved,
    Group(Group),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub head: Head,
    pub top: f32,
    pub open: bool,
    pub holds: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Opened {
    pub group: Group,
    pub top: f32,
    pub kids: Vec<Source>,
    pub pane: Option<f32>,
}

impl Opened {
    pub fn content(&self) -> f32 {
        content(self.kids.len())
    }

    pub fn height(&self) -> f32 {
        self.pane.unwrap_or_else(|| self.content())
    }

    pub fn max_scroll(&self) -> f32 {
        (self.content() - self.height()).max(0.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub sections: Vec<Section>,
    pub saved: usize,
    pub opened: Option<Opened>,
    pub bottom: f32,
}

pub fn content(count: usize) -> f32 {
    if count == 0 { 0.0 } else { count as f32 * KID_STEP - (KID_STEP - KID_HEIGHT) }
}

pub fn kid_top(index: usize) -> f32 {
    index as f32 * KID_STEP
}

pub fn plan(
    saved: usize,
    open: Group,
    catalogue: impl Fn(Group) -> Vec<Source>,
    holds: Group,
) -> Plan {
    let mut sections = Vec::with_capacity(Group::ALL.len() + 1);
    let mut y = NAV_TOP;
    if saved > 0 {
        sections.push(Section { head: Head::Saved, top: y, open: true, holds: false });
        y += SECTION_HEIGHT + saved as f32 * KID_STEP + SAVED_GAP;
    }
    let mut opened = None;
    for group in Group::ALL {
        let is_open = group == open;
        sections.push(Section {
            head: Head::Group(group),
            top: y,
            open: is_open,
            holds: group == holds,
        });
        y += SECTION_HEIGHT;
        if !is_open {
            continue;
        }
        let kids = catalogue(group);
        let scrolls = matches!(group, Group::Themes | Group::Matugen);
        let pane = scrolls.then(|| content(kids.len()).min(PANE_HEIGHT));
        let found = Opened { group, top: y, kids, pane };
        y += found.height() + GROUP_GAP;
        opened = Some(found);
    }
    Plan { sections, saved, opened, bottom: y }
}

pub fn modes_top(bottom: f32) -> f32 {
    (bottom + MODES_GAP).max(MODES_FLOOR)
}

pub fn wheel_top(modes: f32) -> f32 {
    modes + WHEEL_BELOW_MODES
}

pub fn wheel_size(modes: f32) -> Option<f32> {
    let size = WHEEL_MAX.min(INDEX_BOTTOM - wheel_top(modes) - WHEEL_CAPTION);
    (size >= WHEEL_MIN).then_some(size)
}

pub fn wheel_rect(top: f32, size: f32) -> Rectangle {
    Rectangle { x: (INDEX_WIDTH - size) / 2.0, y: top, width: size, height: size }
}

pub fn tabs(needs: [f32; Row::ALL.len()], right: f32) -> [Rectangle; Row::ALL.len()] {
    let widths: [f32; Row::ALL.len()] =
        std::array::from_fn(|index| TAB_WIDTHS[index].max(needs[index]));
    let total: f32 = widths.iter().sum();
    let scale = ((right - TABS.x) / total).clamp(0.0, 1.0);
    let mut x = TABS.x;
    std::array::from_fn(|index| {
        let width = widths[index] * scale;
        let rect = Rectangle { x, y: TABS.y, width, height: TAB_HEIGHT };
        x += width;
        rect
    })
}

#[cfg(test)]
mod tests;
