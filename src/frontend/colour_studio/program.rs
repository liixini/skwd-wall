use std::time::{Duration, Instant};

use iced::widget::canvas::{self, Action, Event};
use iced::{Point, Rectangle, mouse};
use skwd_palette::studio::{Area, SHADE_RANGE};

use super::controls::{Hit, Knob, Phase, Plate, Row, StudioMsg};
use super::draw::{self, Chrome, Pen, Sheet};
use super::layout::{self, Frame, Head};
use super::state::ColourStudio;
use super::tokens::{
    DRAG_SLOP, INDEX_WIDTH, NODE_BOX, PREVIEW, SECTION_HEIGHT, SPLIT_GRAB, TRACK_REACH_X,
    TRACK_REACH_Y, WHEEL_NOTCH, WHEEL_STEP_GAP, WHEEL_STEP_GATE, WINDOW,
};
use crate::app::Message;
use crate::domain::studio::Group;

const NOTCH_PIXELS: f32 = 100.0;
const DOUBLE_CLICK: Duration = Duration::from_millis(400);

#[derive(Debug, Clone, PartialEq)]
enum Target {
    Plate(Plate),
    Section(Group),
    Kid(String),
    Saved(usize),
    Tab(Row),
    Value(Row),
    Option(Row, usize),
    Track(Knob),
    Node(i8),
    Switch,
    Pop,
    Pick,
    Split,
    Pane(Group),
    Inside,
    Outside,
}

impl Target {
    fn hover(&self) -> Option<Hit> {
        match self {
            Self::Plate(plate) => Some(Hit::Plate(*plate)),
            Self::Section(group) => Some(Hit::Section(*group)),
            Self::Kid(key) => Some(Hit::Kid(key.clone())),
            Self::Saved(index) => Some(Hit::Saved(*index)),
            Self::Tab(row) => Some(Hit::Tab(*row)),
            Self::Value(row) => Some(Hit::Value(*row)),
            Self::Option(row, index) => Some(Hit::Option(*row, *index)),
            Self::Track(knob) => Some(Hit::Track(*knob)),
            Self::Node(step) => Some(Hit::Node(*step)),
            Self::Switch | Self::Pop => Some(Hit::Pop),
            Self::Pick | Self::Split | Self::Pane(_) | Self::Inside | Self::Outside => None,
        }
    }

    fn pointer(&self) -> bool {
        matches!(
            self,
            Self::Plate(_)
                | Self::Section(_)
                | Self::Kid(_)
                | Self::Saved(_)
                | Self::Tab(_)
                | Self::Value(_)
                | Self::Option(..)
                | Self::Track(_)
                | Self::Node(_)
                | Self::Switch
        )
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
enum Drag {
    #[default]
    None,
    Slider {
        knob: Knob,
        start: f32,
        live: bool,
    },
    Split {
        offset: f32,
        start: f32,
        live: bool,
    },
    Pick,
}

#[derive(Default)]
pub struct Input {
    drag: Drag,
    step_slot: Option<Target>,
    step_sum: f32,
    step_at: Option<Instant>,
    notch: Option<(Knob, f32)>,
    clicked: Option<(Instant, Knob)>,
    pressed: Option<Plate>,
}

pub struct StudioCanvas<'a> {
    pub studio: &'a ColourStudio,
    pub chrome: Chrome,
    pub ui_scale: f32,
    pub part: (draw::Part, draw::Ink),
    pub top: bool,
}

fn emit(msg: StudioMsg) -> Action<Message> {
    Action::publish(Message::Studio(msg)).and_capture()
}

fn knob_value(studio: &ColourStudio, knob: Knob, at: Point) -> f32 {
    let rail = draw::track(studio, knob);
    let offset = draw::pop_offset(studio, knob.row());
    let share = ((at.x - offset.x - rail.x) / rail.width).clamp(0.0, 1.0);
    let ((low, high), _) = super::update::range(knob);
    low + share * (high - low)
}

fn split_at(at: Point) -> f32 {
    (at.x - PREVIEW.x) / PREVIEW.width
}

fn preview_point(at: Point) -> (f32, f32) {
    (
        ((at.x - PREVIEW.x) / PREVIEW.width).clamp(0.0, 1.0),
        ((at.y - PREVIEW.y) / PREVIEW.height).clamp(0.0, 1.0),
    )
}

fn dot_area(at: Point) -> Area {
    let (x, y) = preview_point(at);
    Area { left: x, top: y, right: x, bottom: y }
}

fn track_zone(studio: &ColourStudio, knob: Knob) -> Rectangle {
    let rail = draw::track(studio, knob);
    Rectangle {
        x: rail.x - TRACK_REACH_X,
        y: rail.y - TRACK_REACH_Y,
        width: rail.width + 2.0 * TRACK_REACH_X,
        height: 2.0 * TRACK_REACH_Y + 1.0,
    }
}

fn node_at(studio: &ColourStudio, at: Point) -> Option<i8> {
    let control = draw::pop(studio, Row::Shade).control;
    let middle = control.y + control.height / 2.0;
    if (at.y - middle).abs() > NODE_BOX / 2.0 {
        return None;
    }
    (SHADE_RANGE.0..=SHADE_RANGE.1)
        .enumerate()
        .find(|(index, _)| (at.x - draw::node_x(studio, *index)).abs() <= NODE_BOX / 2.0)
        .map(|(_, step)| step)
}

impl StudioCanvas<'_> {
    fn frame(&self, bounds: Rectangle) -> Frame {
        Frame::fit(bounds.size(), self.ui_scale)
    }

    fn plates(&self) -> Vec<(Plate, Rectangle)> {
        let studio = self.studio;
        let mut found: Vec<(Plate, Rectangle)> = draw::actions(studio)
            .into_iter()
            .filter_map(|action| Some((action.id?, action.plate.bounds())))
            .collect();
        if studio.content_age().is_finite() {
            for (mode, plate) in draw::mode_plates(studio) {
                found.push((Plate::Mode(mode), plate.bounds()));
            }
        }
        found
    }

    fn nav_target(&self, at: Point) -> Option<Target> {
        if at.x < 0.0 || at.x > INDEX_WIDTH {
            return None;
        }
        let studio = self.studio;
        for section in &studio.plan.sections {
            let top = studio.head_top(section.head);
            if (top..top + SECTION_HEIGHT).contains(&at.y) {
                return match section.head {
                    Head::Group(group) => Some(Target::Section(group)),
                    Head::Saved => Some(Target::Inside),
                };
            }
        }
        for kid in draw::saved_kids(studio) {
            if kid.rect.contains(at) {
                return Some(Target::Saved(kid.index));
            }
        }
        let pane = draw::pane_rect(studio);
        if let Some((_, rect)) = pane
            && !rect.contains(at)
            && studio.plan.opened.as_ref().is_some_and(|opened| opened.pane.is_some())
        {
            return None;
        }
        for kid in draw::kids(studio) {
            if kid.rect.contains(at)
                && let draw::Pick::Source(source) = &kid.pick
            {
                return Some(Target::Kid(source.key()));
            }
        }
        pane.filter(|(_, rect)| rect.contains(at)).map(|(group, _)| Target::Pane(group))
    }

    fn tab_target(&self, at: Point) -> Option<Target> {
        let studio = self.studio;
        let row = Row::ALL.into_iter().find(|row| studio.tabs[row.index()].contains(at))?;
        let steps = matches!(row, Row::View | Row::FineTune);
        if steps && draw::value_rect(studio, row).contains(at) {
            return Some(Target::Value(row));
        }
        Some(Target::Tab(row))
    }

    fn pop_target(&self, at: Point) -> Option<Target> {
        let studio = self.studio;
        if !studio.pop_open() {
            return None;
        }
        let row = studio.row;
        let offset = draw::pop_offset(studio, row);
        let at = Point::new(at.x - offset.x, at.y - offset.y);
        let pop = draw::pop(studio, row);
        if let Some(knob) = Knob::of(row)
            && track_zone(studio, knob).contains(at)
        {
            return Some(Target::Track(knob));
        }
        if !pop.rect.contains(at) {
            return None;
        }
        if let Some((index, _)) =
            draw::options(studio, row).into_iter().find(|(_, plate)| plate.bounds().contains(at))
        {
            return Some(Target::Option(row, index));
        }
        match row {
            Row::Shade => Some(node_at(studio, at).map_or(Target::Pop, Target::Node)),
            Row::Compare if pop.control.contains(at) => Some(Target::Switch),
            _ => Some(Target::Pop),
        }
    }

    fn target(&self, at: Point) -> Target {
        let window = Rectangle { x: 0.0, y: 0.0, width: WINDOW.width, height: WINDOW.height };
        if !window.contains(at) {
            return Target::Outside;
        }
        if let Some((plate, _)) = self.plates().into_iter().find(|(_, bounds)| bounds.contains(at))
        {
            return Target::Plate(plate);
        }
        let studio = self.studio;
        let covered = draw::backing(&draw::actions(studio)).is_some_and(|rect| rect.contains(at));
        if covered || !studio.content_age().is_finite() {
            return Target::Inside;
        }
        if let Some(found) = self.pop_target(at) {
            return found;
        }
        if studio.picking.is_some() && PREVIEW.contains(at) {
            return Target::Pick;
        }
        if let Some(found) = self.nav_target(at) {
            return found;
        }
        if let Some(found) = self.tab_target(at) {
            return found;
        }
        if PREVIEW.contains(at) {
            return Target::Split;
        }
        Target::Inside
    }

    fn press(&self, input: &mut Input, at: Point) -> Action<Message> {
        let studio = self.studio;
        let target = self.target(at);
        match target {
            Target::Outside => emit(StudioMsg::Close),
            Target::Plate(plate) => {
                input.pressed = Some(plate);
                emit(StudioMsg::Press(Some(plate)))
            }
            Target::Section(group) => emit(StudioMsg::OpenGroup(group)),
            Target::Kid(key) => emit(StudioMsg::Pick(key)),
            Target::Saved(index) => emit(StudioMsg::PickSaved(index)),
            Target::Value(row) => emit(StudioMsg::Step(row, 1, false)),
            Target::Track(knob) => {
                let now = Instant::now();
                let twice = input.clicked.is_some_and(|(when, last)| {
                    last == knob && now.duration_since(when) < DOUBLE_CLICK
                });
                input.clicked = Some((now, knob));
                if twice {
                    input.clicked = None;
                    return emit(StudioMsg::Reset(knob));
                }
                input.drag = Drag::Slider { knob, start: at.x, live: false };
                emit(StudioMsg::Slide(knob, knob_value(studio, knob, at), Phase::Down))
            }
            Target::Node(step) => emit(StudioMsg::Shade(step)),
            Target::Option(row, index) => emit(StudioMsg::Choose(row, index)),
            Target::Switch => emit(StudioMsg::Compare),
            Target::Tab(row) => emit(StudioMsg::Row(row)),
            Target::Pick => {
                input.drag = Drag::Pick;
                emit(StudioMsg::Drag(dot_area(at), Phase::Down))
            }
            Target::Split if studio.model.compare => {
                let cut = PREVIEW.x + PREVIEW.width * studio.model.split;
                let near = (at.x - cut).abs() <= SPLIT_GRAB;
                if near {
                    input.drag = Drag::Split { offset: cut - at.x, start: at.x, live: true };
                    return emit(StudioMsg::Split(studio.model.split, Phase::Drag));
                }
                input.drag = Drag::Split { offset: 0.0, start: at.x, live: false };
                emit(StudioMsg::Split(split_at(at), Phase::Down))
            }
            Target::Split | Target::Pane(_) | Target::Pop | Target::Inside => {
                emit(StudioMsg::SkipIntro)
            }
        }
    }

    fn moved(&self, input: &mut Input, at: Point) -> Option<Action<Message>> {
        match input.drag {
            Drag::Slider { knob, start, live } => {
                if !live && (at.x - start).abs() < DRAG_SLOP {
                    return Some(Action::capture());
                }
                input.drag = Drag::Slider { knob, start, live: true };
                Some(emit(StudioMsg::Slide(knob, knob_value(self.studio, knob, at), Phase::Drag)))
            }
            Drag::Split { offset, start, live } => {
                if !live && (at.x - start).abs() < DRAG_SLOP {
                    return Some(Action::capture());
                }
                input.drag = Drag::Split { offset, start, live: true };
                Some(emit(StudioMsg::Split(split_at(Point::new(at.x + offset, at.y)), Phase::Drag)))
            }
            Drag::Pick => Some(emit(StudioMsg::Drag(dot_area(at), Phase::Drag))),
            Drag::None => {
                let hover = self.target(at).hover();
                (hover != self.studio.hover)
                    .then(|| Action::publish(Message::Studio(StudioMsg::Hover(hover))))
            }
        }
    }

    fn released(&self, input: &mut Input, at: Option<Point>) -> Option<Action<Message>> {
        if let Some(plate) = input.pressed.take() {
            let inside = at.is_some_and(|at| self.target(at) == Target::Plate(plate));
            if !inside {
                return Some(emit(StudioMsg::Press(None)));
            }
            return Some(emit(plate_message(plate)));
        }
        match std::mem::take(&mut input.drag) {
            Drag::None => None,
            Drag::Slider { knob, .. } => {
                Some(emit(StudioMsg::Slide(knob, self.studio.knob(knob), Phase::Up)))
            }
            Drag::Split { .. } => Some(emit(StudioMsg::Split(self.studio.model.split, Phase::Up))),
            Drag::Pick => {
                let phase = if at.is_some() { Phase::Up } else { Phase::Lost };
                Some(emit(StudioMsg::Drag(
                    at.map_or(Area { left: 0.0, top: 0.0, right: 0.0, bottom: 0.0 }, dot_area),
                    phase,
                )))
            }
        }
    }

    fn gated(input: &mut Input, slot: Target, pixels: f32) -> Option<i8> {
        if input.step_slot.as_ref() != Some(&slot) {
            input.step_slot = Some(slot);
            input.step_sum = 0.0;
        }
        input.step_sum += pixels;
        let now = Instant::now();
        let recent =
            input.step_at.is_some_and(|at| now.duration_since(at).as_secs_f32() < WHEEL_STEP_GAP);
        if input.step_sum.abs() < WHEEL_STEP_GATE || recent {
            return None;
        }
        let dir = if input.step_sum > 0.0 { 1 } else { -1 };
        input.step_sum = 0.0;
        input.step_at = Some(now);
        Some(dir)
    }

    fn wheel(&self, input: &mut Input, at: Point, pixels: f32) -> Option<Action<Message>> {
        let studio = self.studio;
        match self.target(at) {
            Target::Pane(_) | Target::Kid(_)
                if draw::pane_rect(studio).is_some_and(|(_, rect)| rect.contains(at)) =>
            {
                let (group, _) = draw::pane_rect(studio)?;
                let max = studio.plan.opened.as_ref().map_or(0.0, layout::Opened::max_scroll);
                let target = (studio.panes[group.index()].target() + pixels).clamp(0.0, max);
                Some(emit(StudioMsg::Scroll(group, target)))
            }
            Target::Value(row) => {
                let dir = Self::gated(input, Target::Value(row), pixels)?;
                Some(emit(StudioMsg::Step(row, dir, false)))
            }
            Target::Track(knob) => {
                let (kept, sum) =
                    input.notch.filter(|(kept, _)| *kept == knob).unwrap_or((knob, 0.0));
                let sum = sum - pixels;
                let steps = (sum / WHEEL_NOTCH).trunc();
                input.notch = Some((kept, sum - steps * WHEEL_NOTCH));
                (steps != 0.0).then(|| emit(StudioMsg::Notch(knob, steps as i32)))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;

fn plate_message(plate: Plate) -> StudioMsg {
    match plate {
        Plate::Close => StudioMsg::Close,
        Plate::Mode(mode) => StudioMsg::Mode(mode),
        Plate::Use => StudioMsg::Apply,
        Plate::SaveAs => StudioMsg::SaveAs,
        Plate::Save => StudioMsg::Save,
        Plate::Cancel => StudioMsg::CancelSave,
        Plate::ClearArea => StudioMsg::ClearArea,
        Plate::CancelPick => StudioMsg::CancelPick,
        Plate::Undo => StudioMsg::Undo,
    }
}

impl canvas::Program<Message> for StudioCanvas<'_> {
    type State = Input;

    fn update(
        &self,
        input: &mut Input,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        if !self.top {
            return None;
        }
        let frame = self.frame(bounds);
        let design = |position: Point| {
            frame.design(Point::new(position.x - bounds.x, position.y - bounds.y))
        };
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let at = design(cursor.position()?);
                Some(self.press(input, at))
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let at = design(cursor.position()?);
                self.moved(input, at)
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let at = cursor.position().map(design);
                self.released(input, at)
            }
            Event::Mouse(mouse::Event::CursorLeft) => self.released(input, None).or_else(|| {
                self.studio
                    .hover
                    .is_some()
                    .then(|| Action::publish(Message::Studio(StudioMsg::Hover(None))))
            }),
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let at = design(cursor.position()?);
                let pixels = match crate::frontend::components::norm_wheel(delta)? {
                    mouse::ScrollDelta::Lines { y, .. } => -y * NOTCH_PIXELS,
                    mouse::ScrollDelta::Pixels { y, .. } => -y,
                };
                self.wheel(input, at, pixels)
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _input: &Input,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut sheet = Sheet::new(renderer, Rectangle::with_size(bounds.size()));
        let frame = self.frame(bounds);
        let (part, ink) = self.part;
        let mut pen = Pen::new(&mut sheet, frame.origin, frame.scale, ink);
        draw::studio(&mut pen, self.studio, &self.chrome, &frame, part);
        sheet.finish()
    }

    fn mouse_interaction(
        &self,
        input: &Input,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if !self.top {
            return mouse::Interaction::default();
        }
        let frame = self.frame(bounds);
        let Some(at) = cursor.position_in(bounds).map(|at| frame.design(at)) else {
            return mouse::Interaction::default();
        };
        match input.drag {
            Drag::Split { .. } => return mouse::Interaction::ResizingHorizontally,
            Drag::Pick => return mouse::Interaction::Crosshair,
            Drag::Slider { .. } | Drag::None => {}
        }
        match self.target(at) {
            Target::Split if self.studio.model.compare => mouse::Interaction::ResizingHorizontally,
            Target::Pick | Target::Split => mouse::Interaction::Crosshair,
            found if found.pointer() => mouse::Interaction::Pointer,
            _ => mouse::Interaction::default(),
        }
    }
}
