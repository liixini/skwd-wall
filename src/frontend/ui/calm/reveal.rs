#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use iced::advanced::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer, widget,
};
use iced::{Color, Element, Event, Length, Rectangle, Size, Theme, Vector, window};

use crate::frontend::animation::{CALM, CSS_EASE, Timing, Transition};
use crate::frontend::components::lerp_color;

pub const GROW: Timing = Timing::new(Duration::from_millis(390), CALM);
const FADE: Timing = Timing::new(Duration::from_millis(160), CSS_EASE);
const RISE: Timing = Timing::new(Duration::from_millis(220), CALM);
const LEAD: Duration = Duration::from_millis(90);
const STAGGER: Duration = Duration::from_millis(50);
const STAGGERED_ROWS: u32 = 6;
const RISE_DISTANCE: f32 = 6.0;

#[derive(Clone, Copy)]
pub struct RevealSpec {
    pub identity: u64,
    pub open: bool,
    pub instant: bool,
    pub floor: f32,
}

#[derive(Clone, Copy)]
pub struct StaggerSpec {
    pub identity: u64,
    pub index: usize,
    pub open: bool,
    pub instant: bool,
    pub planes: [Color; 2],
}

#[derive(Clone, Copy)]
pub struct PlaneSpec {
    pub identity: u64,
    pub open: bool,
    pub instant: bool,
    pub planes: [Color; 2],
}

#[derive(Clone, Copy)]
pub(super) enum Kind {
    Grow(RevealSpec),
    Rise(StaggerSpec),
    Plane(PlaneSpec),
}

impl Kind {
    fn identity(self) -> u64 {
        match self {
            Self::Grow(spec) => spec.identity,
            Self::Rise(spec) => spec.identity,
            Self::Plane(spec) => spec.identity,
        }
    }
}

struct Reveal<'a, Message> {
    content: Element<'a, Message>,
    kind: Kind,
}

pub(super) fn row_delay(index: usize) -> Duration {
    let step = u32::try_from(index).unwrap_or(u32::MAX).min(STAGGERED_ROWS - 1);
    LEAD + STAGGER * step
}

pub(super) struct Rise {
    pub fade: Transition,
    pub lift: Transition,
    pub plane: Transition,
}

impl Rise {
    pub(super) fn new(open: bool) -> Self {
        Self {
            fade: Transition::flag(open),
            lift: Transition::flag(open),
            plane: Transition::flag(open),
        }
    }

    pub(super) fn sync(&mut self, spec: StaggerSpec, now: Instant) -> bool {
        if spec.instant {
            *self = Self::new(spec.open);
            return false;
        }
        let delay = if spec.open { row_delay(spec.index) } else { Duration::ZERO };
        self.fade.toward_flag(spec.open, now, FADE.after(delay));
        self.lift.toward_flag(spec.open, now, RISE.after(delay));
        self.plane.toward_flag(spec.open, now, GROW);
        let fading = self.fade.tick(now);
        let rising = self.lift.tick(now);
        let tinting = self.plane.tick(now);
        fading || rising || tinting
    }

    pub(super) fn offset(&self) -> Vector {
        Vector::new(0.0, -RISE_DISTANCE * (1.0 - self.lift.x))
    }

    pub(super) fn veil(&self, spec: StaggerSpec) -> Color {
        let [closed, open] = spec.planes;
        let plane = lerp_color(closed, open, self.plane.x);
        Color { a: plane.a * (1.0 - self.fade.x), ..plane }
    }
}

pub(super) fn grow(height: &mut Transition, open: bool, instant: bool, now: Instant) -> bool {
    if instant {
        *height = Transition::flag(open);
        return false;
    }
    height.toward_flag(open, now, GROW);
    height.tick(now)
}

pub(super) struct Phase {
    identity: u64,
    pub height: Transition,
    pub rise: Rise,
    laid_out: bool,
    short: f32,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Tick {
    pub redraw: bool,
    pub relayout: bool,
}

impl Phase {
    pub(super) fn new(kind: Kind) -> Self {
        let (height, rise) = match kind {
            Kind::Grow(spec) => (Transition::flag(spec.open), Rise::new(true)),
            Kind::Rise(spec) => (Transition::flag(true), Rise::new(spec.open)),
            Kind::Plane(spec) => (Transition::flag(true), Rise::new(spec.open)),
        };
        Self { identity: kind.identity(), height, rise, laid_out: false, short: 0.0 }
    }

    pub(super) fn tick(&mut self, kind: Kind, now: Instant) -> Tick {
        let (height, folded) = (self.height.x, self.folded());
        let redraw = match kind {
            Kind::Grow(spec) => grow(&mut self.height, spec.open, spec.instant, now),
            Kind::Rise(spec) => self.rise.sync(spec, now),
            Kind::Plane(spec) => grow(&mut self.rise.plane, spec.open, spec.instant, now),
        };
        let relayout = self.height.x != height || self.folded() != folded;
        Tick { redraw, relayout }
    }

    pub(super) fn folded(&self) -> bool {
        self.height.x <= 0.0 && !self.height.moving()
    }

    pub(super) fn live(&self) -> bool {
        self.height.x >= 1.0 && !self.height.moving()
    }

    pub(super) fn height(&self, child: f32) -> f32 {
        child * self.height.x
    }
}

pub(super) fn short(tree: &widget::Tree) -> f32 {
    let own = if tree.tag == widget::tree::Tag::of::<Phase>() {
        tree.state.downcast_ref::<Phase>().short
    } else {
        0.0
    };
    own + tree.children.iter().map(short).sum::<f32>()
}

fn first(layout: Layout<'_>) -> Layout<'_> {
    layout.children().next().expect("reveal child")
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Reveal<'_, Message> {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Phase>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Phase::new(self.kind))
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(self.content.as_widget())]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        if tree.state.downcast_ref::<Phase>().identity != self.kind.identity() {
            tree.state = widget::tree::State::new(Phase::new(self.kind));
        }
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        match self.kind {
            Kind::Grow(_) => Size::new(Length::Fill, Length::Shrink),
            Kind::Rise(_) | Kind::Plane(_) => self.content.as_widget().size(),
        }
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let phase = tree.state.downcast_mut::<Phase>();
        let child_tree = &mut tree.children[0];
        let content = self.content.as_widget_mut();
        let Kind::Grow(spec) = self.kind else {
            phase.laid_out = true;
            let child = content.layout(child_tree, renderer, limits);
            return layout::Node::with_children(child.size(), vec![child]);
        };
        phase.laid_out = !phase.folded();
        phase.short = 0.0;
        if !phase.laid_out {
            return layout::Node::new(Size::ZERO);
        }
        let Size { width, height: offered } = limits.max();
        let child = content.layout(
            child_tree,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width, offered.max(spec.floor))),
        );
        phase.short = (child.size().height - offered).max(0.0);
        let size = Size::new(width, phase.height(child.size().height));
        layout::Node::with_children(size, vec![child])
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let phase = tree.state.downcast_ref::<Phase>();
        let bounds = layout.bounds();
        let child = &tree.children[0];
        let content = self.content.as_widget();
        if let Kind::Plane(spec) = self.kind {
            let [closed, open] = spec.planes;
            renderer.fill_quad(
                renderer::Quad { bounds, ..renderer::Quad::default() },
                lerp_color(closed, open, phase.rise.plane.x),
            );
            content.draw(child, renderer, theme, style, first(layout), cursor, viewport);
            return;
        }
        if phase.rise.fade.x <= 0.0 || bounds.height <= 0.0 || !phase.laid_out {
            return;
        }
        if !phase.live() {
            let Some(clip) = bounds.intersection(viewport) else {
                return;
            };
            renderer.with_layer(clip, |renderer| {
                let idle = mouse::Cursor::Unavailable;
                content.draw(child, renderer, theme, style, first(layout), idle, &clip);
            });
            return;
        }
        let offset = phase.rise.offset();
        renderer.with_translation(offset, |renderer| {
            content.draw(
                child,
                renderer,
                theme,
                style,
                first(layout),
                cursor,
                &(*viewport - offset),
            );
        });
        let Kind::Rise(spec) = self.kind else {
            return;
        };
        if phase.rise.fade.x < 1.0
            && let Some(veil) = (bounds + offset).intersection(viewport)
        {
            renderer.with_layer(veil, |renderer| {
                renderer.fill_quad(
                    renderer::Quad { bounds: veil, ..renderer::Quad::default() },
                    phase.rise.veil(spec),
                );
            });
        }
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        if tree.state.downcast_ref::<Phase>().live() {
            let child = &mut tree.children[0];
            self.content.as_widget_mut().operate(child, first(layout), renderer, operation);
        }
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let phase = tree.state.downcast_mut::<Phase>();
        let laid_out = phase.laid_out;
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let tick = phase.tick(self.kind, *now);
            if tick.redraw {
                shell.request_redraw();
            }
            if tick.relayout {
                shell.invalidate_layout();
            }
        }
        if laid_out && (phase.live() || matches!(event, Event::Window(_))) {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                first(layout),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if !tree.state.downcast_ref::<Phase>().live() {
            return mouse::Interaction::None;
        }
        let child = &tree.children[0];
        self.content.as_widget().mouse_interaction(child, first(layout), cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        offset: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        if !tree.state.downcast_ref::<Phase>().live() {
            return None;
        }
        let child = &mut tree.children[0];
        self.content.as_widget_mut().overlay(child, first(layout), renderer, viewport, offset)
    }
}

pub fn reveal<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    spec: RevealSpec,
) -> Element<'a, Message> {
    Element::new(Reveal { content: content.into(), kind: Kind::Grow(spec) })
}

pub fn plane<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    spec: PlaneSpec,
) -> Element<'a, Message> {
    Element::new(Reveal { content: content.into(), kind: Kind::Plane(spec) })
}

pub fn stagger<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    spec: StaggerSpec,
) -> Element<'a, Message> {
    Element::new(Reveal { content: content.into(), kind: Kind::Rise(spec) })
}
