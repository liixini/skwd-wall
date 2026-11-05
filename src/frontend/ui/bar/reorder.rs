use iced::{Color, Point, Rectangle, Vector};

use crate::frontend::animation::{MotionProfile, MotionTier, Spring};

use super::action::BarAction;
use super::model::{BarItem, BarModel, BarVisualStyle};

const SLOT_HYSTERESIS: f32 = 8.0;
const SLOT_ANCHOR: f32 = 0.28;

struct ItemMotion {
    x: Spring,
    y: Spring,
}

pub(crate) struct ReorderPreview {
    model: BarModel,
    motion: Vec<ItemMotion>,
    swatch: Option<ItemMotion>,
    source: usize,
    ids: Vec<(usize, String)>,
    others: Vec<usize>,
    slot: usize,
    pointer: Point,
    grab: Vector,
    vertical: bool,
    dragging: bool,
    scale: f32,
}

#[derive(Clone)]
pub(crate) struct EditBar {
    pub(crate) items: Vec<BarItem>,
    pub(crate) lifted: usize,
    pub(crate) swatch_at: Option<(f32, f32)>,
    pub(crate) swatches: Vec<Color>,
    pub(crate) style: BarVisualStyle,
    pub(crate) rtl: bool,
    pub(crate) scale: f32,
}

impl ReorderPreview {
    pub(crate) fn new(
        model: BarModel,
        bar: Rectangle,
        source: usize,
        pointer: Point,
        vertical: bool,
        scale: f32,
        profile: MotionProfile,
    ) -> Option<Self> {
        let item = model.items.get(source)?;
        item.edit_id()?;
        let grab = pointer - Point::new(bar.x + item.x, bar.y + item.y);
        let ids: Vec<_> = model
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| item.edit_id().map(|id| (index, id)))
            .collect();
        let slot = ids.iter().position(|(index, _)| *index == source)?;
        let others =
            ids.iter().filter(|(index, _)| *index != source).map(|(index, _)| *index).collect();
        let motion = model
            .items
            .iter()
            .map(|item| ItemMotion {
                x: profile.spring(bar.x + item.x, MotionTier::Standard),
                y: profile.spring(bar.y + item.y, MotionTier::Standard),
            })
            .collect();
        let swatch = model.swatch_at.map(|(x, y)| ItemMotion {
            x: profile.spring(bar.x + x, MotionTier::Standard),
            y: profile.spring(bar.y + y, MotionTier::Standard),
        });
        Some(Self {
            model,
            motion,
            swatch,
            source,
            ids,
            others,
            slot,
            pointer,
            grab,
            vertical,
            dragging: true,
            scale,
        })
    }

    pub(crate) fn move_to(&mut self, point: Point) -> bool {
        self.pointer = point;
        let center = self.lifted_center();
        let current = self.slot_score(self.slot, center).min(self.gap_distance(center));
        let Some((slot, distance)) = (0..=self.others.len())
            .map(|slot| (slot, self.slot_score(slot, center)))
            .min_by(|left, right| left.1.total_cmp(&right.1))
        else {
            return false;
        };
        if slot != self.slot && distance + SLOT_HYSTERESIS * self.scale < current {
            self.slot = slot;
            return true;
        }
        false
    }

    fn lifted_center(&self) -> Point {
        let item = &self.model.items[self.source];
        Point::new(
            self.pointer.x - self.grab.x + item.w * 0.5,
            self.pointer.y - self.grab.y + item.h * 0.5,
        )
    }

    fn slot_score(&self, slot: usize, point: Point) -> f32 {
        let mut distance = f32::INFINITY;
        if let Some(index) = slot.checked_sub(1).and_then(|index| self.others.get(index)) {
            distance = distance.min(self.anchor_distance(*index, point, true));
        }
        if let Some(index) = self.others.get(slot) {
            distance = distance.min(self.anchor_distance(*index, point, false));
        }
        distance
    }

    fn gap_distance(&self, point: Point) -> f32 {
        let item = &self.model.items[self.source];
        Rectangle {
            x: self.motion[self.source].x.target,
            y: self.motion[self.source].y.target,
            width: item.w,
            height: item.h,
        }
        .distance(point)
    }

    fn anchor_distance(&self, index: usize, point: Point, after: bool) -> f32 {
        let item = &self.model.items[index];
        let direction = if after { 1.0 } else { -1.0 };
        let x = self.motion[index].x.target + item.w * 0.5;
        let y = self.motion[index].y.target + item.h * 0.5;
        let anchor = if self.vertical {
            Point::new(x, y + direction * item.h * SLOT_ANCHOR)
        } else {
            let reading = if self.model.rtl { -direction } else { direction };
            Point::new(x + reading * item.w * SLOT_ANCHOR, y)
        };
        anchor.distance(point)
    }

    pub(crate) fn order(&self) -> Vec<String> {
        let source = self.ids.iter().find(|(index, _)| *index == self.source).unwrap().1.clone();
        let mut order: Vec<_> = self
            .ids
            .iter()
            .filter(|(index, _)| *index != self.source)
            .map(|(_, id)| id.clone())
            .collect();
        order.insert(self.slot, source);
        order
    }

    pub(crate) fn changed(&self) -> bool {
        self.ids.iter().map(|(_, id)| id).ne(self.order().iter())
    }

    pub(crate) fn retarget(&mut self, target: &BarModel, bar: Rectangle) {
        for (index, item) in self.model.items.iter().enumerate() {
            let next = if let Some(id) = item.edit_id() {
                target.items.iter().find(|next| next.edit_id().as_deref() == Some(&id))
            } else if item.notice.is_some() {
                let ordinal = self.model.items[..index]
                    .iter()
                    .filter(|earlier| earlier.notice.is_some())
                    .count();
                target.items.iter().filter(|next| next.notice.is_some()).nth(ordinal)
            } else {
                target
                    .items
                    .get(index)
                    .filter(|next| same_auxiliary(item, next))
                    .or_else(|| target.items.iter().find(|next| same_auxiliary(item, next)))
            };
            let Some(next) = next else {
                continue;
            };
            self.motion[index].x.retarget(bar.x + next.x);
            self.motion[index].y.retarget(bar.y + next.y);
        }
        if let (Some(swatch), Some((x, y))) = (self.swatch.as_mut(), target.swatch_at) {
            swatch.x.retarget(bar.x + x);
            swatch.y.retarget(bar.y + y);
        }
    }

    pub(crate) fn finish(&mut self) {
        self.dragging = false;
        let origin = self.lifted_origin();
        let x = self.motion[self.source].x.target;
        let y = self.motion[self.source].y.target;
        self.motion[self.source].x.snap(origin.x);
        self.motion[self.source].y.snap(origin.y);
        self.motion[self.source].x.retarget(x);
        self.motion[self.source].y.retarget(y);
    }

    fn lifted_origin(&self) -> Point {
        Point::new(self.pointer.x - self.grab.x, self.pointer.y - self.grab.y)
    }

    pub(crate) fn tick(&mut self, dt: f32) {
        for (index, item) in self.motion.iter_mut().enumerate() {
            if self.dragging && index == self.source {
                continue;
            }
            item.x.tick(dt);
            item.y.tick(dt);
        }
        if let Some(swatch) = self.swatch.as_mut() {
            swatch.x.tick(dt);
            swatch.y.tick(dt);
        }
    }

    pub(crate) fn animating(&self) -> bool {
        self.motion.iter().enumerate().any(|(index, item)| {
            (!self.dragging || index != self.source) && (!item.x.settled() || !item.y.settled())
        }) || self.swatch.as_ref().is_some_and(|swatch| !swatch.x.settled() || !swatch.y.settled())
    }

    pub(crate) fn finished(&self) -> bool {
        !self.dragging && !self.animating()
    }

    pub(crate) fn retime(&mut self, profile: MotionProfile) {
        for item in &mut self.motion {
            profile.retime_spring(&mut item.x, MotionTier::Standard);
            profile.retime_spring(&mut item.y, MotionTier::Standard);
        }
        if let Some(swatch) = self.swatch.as_mut() {
            profile.retime_spring(&mut swatch.x, MotionTier::Standard);
            profile.retime_spring(&mut swatch.y, MotionTier::Standard);
        }
    }

    pub(crate) fn edit_bar(&self, style: BarVisualStyle, swatches: &[Color]) -> EditBar {
        let mut items = self.model.items.clone();
        for (index, item) in items.iter_mut().enumerate() {
            let point = if self.dragging && index == self.source {
                self.lifted_origin()
            } else {
                Point::new(self.motion[index].x.value(), self.motion[index].y.value())
            };
            item.x = point.x;
            item.y = point.y;
        }
        EditBar {
            items,
            lifted: self.source,
            swatch_at: self.swatch.as_ref().map(|swatch| (swatch.x.value(), swatch.y.value())),
            swatches: swatches.to_vec(),
            style,
            rtl: self.model.rtl,
            scale: self.scale,
        }
    }
}

fn same_auxiliary(first: &BarItem, second: &BarItem) -> bool {
    if second.edit_id().is_some() {
        return false;
    }
    match (first.action.as_ref(), second.action.as_ref()) {
        (
            Some(BarAction::TaskControl { id: first_id, action: first_action }),
            Some(BarAction::TaskControl { id: second_id, action: second_action }),
        ) => first_id == second_id && first_action == second_action,
        (
            Some(BarAction::ThemeOpt(first_key, first_value)),
            Some(BarAction::ThemeOpt(second_key, second_value)),
        ) => first_key == second_key && first_value == second_value,
        (Some(BarAction::ThemeBackendToggle), Some(BarAction::ThemeBackendToggle))
        | (Some(BarAction::ThemePinSettings), Some(BarAction::ThemePinSettings)) => true,
        (None, None) if first.notice.is_none() && second.notice.is_none() => {
            first.label == second.label
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;
