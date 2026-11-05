use skwd_palette::studio::{Area, COLOUR_RANGE, CONTRAST_RANGE, Method, SHADE_RANGE, WARMTH_RANGE};

use super::controls::{Edit, Hit, Knob, Phase, Plate, Row, StudioMsg, TUNABLE, View, cycle};
use super::motion::{BASE, FAST, SLOW};
use super::state::{ColourStudio, Picking};
use super::tokens::{PICK_MIN, POP_LEAVE, PREVIEW, SPLIT_RANGE, WHEEL_QUIET};
use crate::domain::studio::{Group, Mode, ModeChoice, Saved, Source, area, is_area, store};

const SLIDER_STEP: f32 = 0.01;
const SLIDER_KEY_STEP: f32 = 0.05;
const SLIDER_DETENT: f32 = 0.025;
const PERCENT: f32 = 100.0;

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    None,
    Close,
    Apply,
    Undo,
    Persist,
    Area(Area),
}

pub fn range(knob: Knob) -> ((f32, f32), f32) {
    match knob {
        Knob::Warmth => (WARMTH_RANGE, 0.0),
        Knob::Colour => (COLOUR_RANGE, 1.0),
        Knob::Contrast => (CONTRAST_RANGE, 0.0),
    }
}

pub fn snap(knob: Knob, value: f32) -> f32 {
    let ((low, high), neutral) = range(knob);
    let stepped = (value.clamp(low, high) / SLIDER_STEP).round() * SLIDER_STEP;
    let rounded = (stepped * PERCENT).round() / PERCENT;
    if (rounded - neutral).abs() < SLIDER_DETENT { neutral } else { rounded }
}

fn stepped(knob: Knob, current: f32, delta: f32) -> f32 {
    let ((low, high), neutral) = range(knob);
    let next = ((current + delta).clamp(low, high) * PERCENT).round() / PERCENT;
    if (current - neutral) * (next - neutral) < 0.0 { neutral } else { next }
}

fn rounded_dir(next: f32, current: f32) -> f32 {
    if next >= current { 1.0 } else { -1.0 }
}

impl ColourStudio {
    fn set_knob(&mut self, knob: Knob, value: f32) {
        let tuning = &mut self.model.tuning;
        let slot = match knob {
            Knob::Warmth => &mut tuning.warmth,
            Knob::Colour => &mut tuning.colour,
            Knob::Contrast => &mut tuning.contrast,
        };
        *slot = value;
    }

    fn select_row(&mut self, row: Row) {
        if row != self.row {
            self.row = row;
            self.row_at = self.now;
            self.hold(self.now + BASE);
        }
    }

    fn bump(&mut self, row: Row, dir: f32) {
        self.select_row(row);
        self.row_dirs[row.index()] = dir;
    }

    fn order_dir(&self, next: &Source) -> f32 {
        let order = self.model.order();
        let at = |source: &Source| order.iter().position(|item| item == source);
        match (at(next), at(&self.model.source)) {
            (Some(next), Some(current)) if next < current => -1.0,
            _ => 1.0,
        }
    }

    fn choose(&mut self, source: Source, dir: f32) {
        self.dir = dir;
        self.model.source = source;
        self.reveal_later();
    }

    fn reveal_later(&mut self) {
        let group = self.model.group();
        if group != self.group {
            self.open_group(group);
        }
    }

    fn open_group(&mut self, group: Group) {
        if group != self.group {
            self.group = group;
            self.group_at = self.now;
            self.hold(self.now + SLOW + super::tokens::KID_DELAY.0);
        }
    }

    fn earlier(before: &Source) -> Source {
        if is_area(before) {
            Source::Method(Method::Tonal.key().to_string())
        } else {
            before.clone()
        }
    }

    pub fn enter_pick(&mut self) {
        self.picking = Some(Picking {
            draft: None,
            anchor: None,
            prior: None,
            prior_source: self.model.source.clone(),
            before: self.model.source.clone(),
            dragging: false,
        });
        self.naming = None;
        self.applied = None;
        self.mounts.picking = self.now;
        self.hold(self.now + SLOW);
    }

    fn cancel_pick(&mut self) {
        let Some(pick) = self.picking.take() else {
            return;
        };
        self.mounts.main = self.now;
        self.dir = -1.0;
        self.model.source =
            if self.saved_area().is_some() && is_area(&pick.before) { area() } else { pick.before };
        self.refresh_area();
        self.hold(self.now + SLOW);
    }

    fn clear_area(&mut self) {
        let before =
            self.picking.take().map_or_else(|| self.model.source.clone(), |pick| pick.before);
        self.set_saved_area(None);
        self.mounts.main = self.now;
        self.dir = -1.0;
        self.model.source = Self::earlier(&before);
        self.refresh_area();
        self.hold(self.now + SLOW);
    }

    fn drag(&mut self, shown: Area, phase: Phase) -> Effect {
        let ready = self.pixels().is_some();
        let Some(pick) = self.picking.as_mut() else {
            return Effect::None;
        };
        match phase {
            Phase::Down => {
                if !ready {
                    return Effect::None;
                }
                pick.anchor = Some((shown.left, shown.top));
                pick.prior = pick.draft;
                pick.prior_source = self.model.source.clone();
                pick.draft = Some(shown);
                pick.dragging = true;
                self.mounts.draft = self.now;
                self.model.source = area();
            }
            Phase::Drag => {
                let Some((x, y)) = pick.anchor.filter(|_| pick.dragging) else {
                    return Effect::None;
                };
                pick.draft = Some(Area {
                    left: x.min(shown.left),
                    top: y.min(shown.top),
                    right: x.max(shown.left),
                    bottom: y.max(shown.top),
                });
            }
            Phase::Up | Phase::Lost => {
                if !pick.dragging {
                    return Effect::None;
                }
                pick.dragging = false;
                pick.anchor = None;
                let small = pick.draft.is_none_or(|draft| {
                    (draft.right - draft.left) * PREVIEW.width < PICK_MIN
                        || (draft.bottom - draft.top) * PREVIEW.height < PICK_MIN
                });
                if phase == Phase::Lost || small {
                    pick.draft = pick.prior;
                    self.model.source = pick.prior_source.clone();
                    self.refresh_area();
                    return Effect::None;
                }
                let Some(draft) = pick.draft else {
                    return Effect::None;
                };
                let image = self.frame().to_image(draft);
                self.picking = None;
                self.set_saved_area(Some(image));
                self.mounts.main = self.now;
                self.dir = 1.0;
                self.model.source = area();
                self.refresh_area();
                self.hold(self.now + SLOW);
                return Effect::Area(image);
            }
        }
        self.refresh_area();
        Effect::None
    }

    fn step(&mut self, row: Row, dir: i8, fine: bool) {
        let sign = f32::from(dir);
        self.bump(row, sign);
        match row {
            Row::View => {
                let index = View::ALL.iter().position(|view| *view == self.view).unwrap_or(0);
                self.view = View::ALL[cycle(index, dir, View::ALL.len())];
            }
            Row::FineTune => {
                let index = TUNABLE.iter().position(|role| *role == self.model.role).unwrap_or(0);
                self.model.role = TUNABLE[cycle(index, dir, TUNABLE.len())];
            }
            Row::Warmth | Row::Colour | Row::Contrast => {
                let Some(knob) = Knob::of(row) else {
                    return;
                };
                let delta = sign * if fine { SLIDER_STEP } else { SLIDER_KEY_STEP };
                let next = stepped(knob, self.knob(knob), delta);
                self.set_knob(knob, next);
            }
            Row::Shade => {
                let slot = &mut self.model.tuning.shades[self.model.role.index()];
                *slot = (*slot + dir).clamp(SHADE_RANGE.0, SHADE_RANGE.1);
            }
            Row::Compare => {
                self.row_dirs[row.index()] = 1.0;
                self.model.compare = !self.model.compare;
                self.compared();
            }
        }
    }

    fn choose_option(&mut self, row: Row, index: usize) {
        match row {
            Row::View => {
                let at = View::ALL.iter().position(|view| *view == self.view).unwrap_or(0);
                self.bump(row, if index >= at { 1.0 } else { -1.0 });
                self.view = View::ALL[index.min(View::ALL.len() - 1)];
            }
            Row::FineTune => {
                let at = TUNABLE.iter().position(|role| *role == self.model.role).unwrap_or(0);
                self.bump(row, if index >= at { 1.0 } else { -1.0 });
                self.model.role = TUNABLE[index.min(TUNABLE.len() - 1)];
            }
            _ => {}
        }
    }

    fn compared(&mut self) {
        if self.model.compare {
            self.mounts.tags = self.now;
            self.hold(self.now + BASE + FAST);
        }
    }

    fn pick_source(&mut self, key: &str) {
        let source = Source::parse(key);
        if is_area(&source) {
            if self.picking.is_some() {
                return;
            }
            if self.saved_area().is_some() && !is_area(&self.model.source) {
                self.choose(source, 1.0);
                self.refresh_area();
            } else {
                self.enter_pick();
            }
            return;
        }
        self.end_pick();
        let dir = self.order_dir(&source);
        self.choose(source, dir);
    }

    fn end_pick(&mut self) {
        if self.picking.take().is_some() {
            self.mounts.main = self.now;
            self.refresh_area();
        }
    }

    fn pick_saved(&mut self, index: usize) -> Effect {
        let Some(entry) = self.shown_saved().get(index).cloned() else {
            return Effect::None;
        };
        self.end_pick();
        if let Some(area) = entry.area {
            self.set_saved_area(Some(area));
        }
        self.model.tuning = entry.tuning;
        self.model.source = entry.source;
        self.open_group(self.model.group());
        self.refresh_area();
        entry.area.filter(|_| is_area(&self.model.source)).map_or(Effect::None, Effect::Area)
    }

    fn scheme_step(&mut self, dir: i8) {
        if self.picking.is_some() {
            return;
        }
        let order = self.model.order();
        if order.is_empty() {
            return;
        }
        let at = order.iter().position(|item| *item == self.model.source).unwrap_or(0);
        let next = order[cycle(at, dir, order.len())].clone();
        self.choose(next, f32::from(dir));
        self.refresh_area();
    }

    fn save(&mut self) -> Effect {
        let Some(typed) = self.naming.take() else {
            return Effect::None;
        };
        let name = match typed.trim() {
            "" => super::text::save_placeholder(self),
            trimmed => trimmed.to_string(),
        };
        let entry = Saved {
            name: name.clone(),
            area: is_area(&self.model.source).then(|| self.saved_area()).flatten(),
            source: self.model.source.clone(),
            tuning: self.model.tuning,
        };
        self.saved = store(&self.saved, entry);
        self.invalidate();
        self.saved_at = self.now;
        self.mounts.main = self.now;
        self.hold(self.now + SLOW);
        Effect::Persist
    }

    fn edit(&mut self, edit: &Edit) {
        let Some(name) = self.naming.as_mut() else {
            return;
        };
        match edit {
            Edit::Insert(text) => name.push_str(text),
            Edit::Erase => {
                name.pop();
            }
        }
    }

    fn slide(&mut self, knob: Knob, value: f32, phase: Phase) {
        let next = snap(knob, value);
        let current = self.knob(knob);
        match phase {
            Phase::Down => {
                self.bump(knob.row(), rounded_dir(next, current));
                self.set_knob(knob, next);
            }
            Phase::Drag => {
                self.knob_drag = Some(knob);
                self.set_knob(knob, next);
            }
            Phase::Up | Phase::Lost => self.knob_drag = None,
        }
    }

    fn notch(&mut self, knob: Knob, steps: i32) {
        let current = self.knob(knob);
        let next = stepped(knob, current, steps as f32 * SLIDER_STEP);
        self.select_row(knob.row());
        self.row_dirs[knob.row().index()] = if steps > 0 { 1.0 } else { -1.0 };
        self.quiet_until = self.now + WHEEL_QUIET;
        self.hold(self.quiet_until + BASE);
        self.set_knob(knob, next);
    }

    fn reset(&mut self, knob: Knob) {
        let ((_, _), neutral) = range(knob);
        let current = self.knob(knob);
        self.bump(knob.row(), rounded_dir(neutral, current));
        self.set_knob(knob, neutral);
    }

    fn shade(&mut self, step: i8) {
        let current = self.model.tuning.shades[self.model.role.index()];
        self.bump(Row::Shade, if step >= current { 1.0 } else { -1.0 });
        self.model.tuning.shades[self.model.role.index()] = step;
    }

    fn hover(&mut self, hit: Option<Hit>) {
        if hit == self.hover {
            return;
        }
        if let Some(Hit::Tab(row) | Hit::Value(row)) = hit {
            self.select_row(row);
            self.sync_controls();
        }
        let inside = hit.as_ref().is_some_and(Hit::in_bar);
        if inside != self.pop_hover {
            self.pop_hover = inside;
            if inside {
                self.open_pop();
            } else {
                self.pop_until = self.now + POP_LEAVE;
                self.hold(self.pop_until);
            }
        }
        self.unhover = std::mem::replace(&mut self.hover, hit);
        self.unhover_at = self.now;
        self.hover_at = self.now;
        self.hold(self.now + FAST);
    }

    fn press(&mut self, plate: Option<Plate>) {
        if plate != self.pressed {
            self.pressed = plate;
            self.pressed_at = self.now;
            self.hold(self.now + FAST);
        }
    }

    fn mode(&mut self, choice: crate::domain::studio::ModeChoice) {
        if choice == self.model.mode_choice {
            return;
        }
        let mode = match choice {
            ModeChoice::Dark => Mode::Dark,
            ModeChoice::Light => Mode::Light,
            ModeChoice::ByTime => self.time_mode,
        };
        let before = self.model.source.clone();
        let order = [ModeChoice::Dark, ModeChoice::Light, ModeChoice::ByTime];
        let rank = |item| order.iter().position(|entry| *entry == item);
        let dir = if rank(choice) > rank(self.model.mode_choice) { 1.0 } else { -1.0 };
        self.model.choose_mode(choice, mode);
        if self.model.source != before {
            self.dir = dir;
        }
        self.refresh_area();
        self.reveal_later();
    }

    pub fn update(&mut self, msg: &StudioMsg) -> Effect {
        if skips_intro(msg) {
            self.intro = false;
        }
        if !matches!(msg, StudioMsg::Press(_) | StudioMsg::Hover(_)) {
            self.pressed = None;
        }
        let before = (self.model.source.clone(), self.group);
        let effect = self.handle(msg);
        if flashes(msg) {
            self.flash();
        }
        if !matches!(msg, StudioMsg::Hover(_) | StudioMsg::Press(_)) {
            self.sync();
            if (self.model.source.clone(), self.group) != before {
                self.reveal();
            }
        }
        effect
    }

    fn handle(&mut self, msg: &StudioMsg) -> Effect {
        match msg {
            StudioMsg::Close => return Effect::Close,
            StudioMsg::Apply => return Effect::Apply,
            StudioMsg::Undo => return Effect::Undo,
            StudioMsg::Pick(key) => self.pick_source(key),
            StudioMsg::PickSaved(index) => return self.pick_saved(*index),
            StudioMsg::OpenGroup(group) => self.open_group(*group),
            StudioMsg::Mode(mode) => self.mode(*mode),
            StudioMsg::Row(row) => self.select_row(*row),
            StudioMsg::Step(row, dir, fine) => self.step(*row, *dir, *fine),
            StudioMsg::Slide(knob, value, phase) => self.slide(*knob, *value, *phase),
            StudioMsg::Notch(knob, steps) => self.notch(*knob, *steps),
            StudioMsg::Reset(knob) => self.reset(*knob),
            StudioMsg::Choose(row, index) => self.choose_option(*row, *index),
            StudioMsg::Shade(step) => self.shade(*step),
            StudioMsg::Compare => self.step(Row::Compare, 1, false),
            StudioMsg::Split(at, phase) => {
                self.model.split = at.clamp(SPLIT_RANGE.0, SPLIT_RANGE.1);
                self.sliding = *phase == Phase::Drag;
            }
            StudioMsg::Scroll(group, offset) => {
                let until = self.panes[group.index()].toward(offset.max(0.0), self.now);
                self.hold(until);
            }
            StudioMsg::SchemeStep(dir) => self.scheme_step(*dir),
            StudioMsg::Hover(hit) => self.hover(hit.clone()),
            StudioMsg::Press(plate) => self.press(*plate),
            StudioMsg::SaveAs => {
                self.naming = Some(String::new());
                self.mounts.naming = self.now;
                self.hold(self.now + BASE + FAST);
            }
            StudioMsg::Save => return self.save(),
            StudioMsg::CancelSave => {
                self.naming = None;
                self.mounts.main = self.now;
                self.hold(self.now + BASE + FAST);
            }
            StudioMsg::Name(edit) => self.edit(edit),
            StudioMsg::StartPick => {
                if self.picking.is_some() {
                    self.cancel_pick();
                } else {
                    self.enter_pick();
                }
            }
            StudioMsg::CancelPick => self.cancel_pick(),
            StudioMsg::ClearArea => self.clear_area(),
            StudioMsg::Drag(shown, phase) => return self.drag(*shown, *phase),
            StudioMsg::SkipIntro => self.intro = false,
        }
        Effect::None
    }
}

fn skips_intro(msg: &StudioMsg) -> bool {
    !matches!(msg, StudioMsg::Hover(_) | StudioMsg::Scroll(..) | StudioMsg::Notch(..))
}

fn flashes(msg: &StudioMsg) -> bool {
    matches!(
        msg,
        StudioMsg::Row(_)
            | StudioMsg::Step(..)
            | StudioMsg::Slide(_, _, Phase::Down)
            | StudioMsg::Notch(..)
            | StudioMsg::Reset(_)
            | StudioMsg::Choose(..)
            | StudioMsg::Shade(_)
            | StudioMsg::Compare
    )
}

#[cfg(test)]
mod tests;
