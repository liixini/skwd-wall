#[cfg(test)]
mod tests;

use std::borrow::Cow;

use iced::advanced::graphics::text::Paragraph;
use iced::advanced::text::{self, Paragraph as _, Span};
use iced::advanced::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer, widget,
};
use iced::keyboard::{self, Key, key::Named};
use iced::{
    Background, Border, Color, Element, Event, Font, Length, Pixels, Point, Rectangle, Size, Theme,
    Vector, alignment,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::frontend::theme::Palette;
use crate::frontend::ui::{
    TWISTY_OPEN, UI_FONT, cached_line_width, has_rtl, legible_type_scale, shapes_across,
};

use super::popover::{Anchored, Popover, Side};
use super::switch::RING;
use super::tint::{SELECTED_TINT, tint};

pub const SAY_SIZE: f32 = 13.5;
pub const SAY_LEADING: f32 = 1.55;
pub const PLAIN_LEADING: f32 = 1.3;
const NUMBER_GLYPH: &str = "↕";
const TEXT_GLYPH: &str = "✎";
const KEY_GLYPH: &str = "⌨";
const ADD_GLYPH: &str = "+";
const ADD_GLYPH_SIZE: f32 = 10.0;
const NO_GLYPH: &str = "";
const PLAIN_ALPHA: f32 = 0.86;
const SOFT_ALPHA: f32 = 0.72;
const FAINT_ALPHA: f32 = 0.6;
const UNBOUND_ALPHA: f32 = 0.5;
const UNBOUND_RULE: f32 = 0.35;
const ADD_ALPHA: f32 = 0.42;
const ADD_RULE: f32 = 0.3;
const LINK_RULE: f32 = 0.5;
const MENTION_RULE: f32 = 0.3;
const DASH: f32 = 3.0;
pub const ASIDE_SIZE: f32 = 11.0;
const ASIDE_LEADING: f32 = 1.4;
const ASIDE_ALPHA: f32 = 0.5;
pub const ASIDE_GAP: &str = "\u{2003}";
const GLYPH_ALPHA: f32 = 0.8;
const UNDERLINE_ALPHA: f32 = 0.75;
const MUTED_ALPHA: f32 = 0.6;
const MUTED_LINE_ALPHA: f32 = 0.4;
const SMALL_GLYPH: f32 = 8.0;
const WIDE_GLYPH: f32 = 9.0;
const GLYPH_GAP: char = '\u{202f}';
const JOINER: char = '\u{2060}';
const ISOLATE: char = '\u{2068}';
const ISOLATE_END: char = '\u{2069}';
const NO_BREAK_SPACE: &str = "\u{a0}";
const LINE_BREAK: &str = "\n";
const RTL_MARK: &str = "\u{200f}";
const BREAKS: [&str; 4] = ["/", "-", "\u{2010}", "\u{2013}"];
const SENTINEL: u32 = 0xE000;
const SENTINEL_LAST: u32 = 0xF8FD;
pub const VALUE_OPEN: char = '\u{f8fe}';
pub const VALUE_CLOSE: char = '\u{f8ff}';
const ASCENT: f32 = 0.928;
const DESCENT: f32 = 0.244;
const DOT: f32 = 1.0;
const RING_OFFSET: f32 = 2.0;
const DRAG_STEP: f32 = 4.0;
const DRAG_SLOP: f32 = 3.0;

#[derive(Debug, Clone, Copy)]
pub struct Scrub {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

impl Scrub {
    pub fn nudge(self, steps: f64) -> f64 {
        let outward =
            (self.value > self.max && steps > 0.0) || (self.value < self.min && steps < 0.0);
        if outward {
            return self.value;
        }
        let value = self.value + steps * self.step;
        let snapped =
            if self.step > 0.0 { (value / self.step).round() * self.step } else { self.value };
        snapped.max(self.min).min(self.max)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KeyLook {
    #[default]
    Bound,
    Clash,
    Unbound,
    Add,
    Listening,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stroke {
    Dotted,
    Dashed,
    Solid,
}

struct Inks {
    text: Color,
    rule: Color,
    stroke: Stroke,
    fill: Option<Color>,
    clipped: bool,
}

fn text_alpha<Message>(piece: &Piece<'_, Message>) -> f32 {
    match piece {
        Piece::Aside(_) => ASIDE_ALPHA,
        Piece::Soft(_) => SOFT_ALPHA,
        Piece::Faint(_) => FAINT_ALPHA,
        _ => PLAIN_ALPHA,
    }
}

fn dashes(left: f32, right: f32, dash: f32, step: f32, clipped: bool) -> Vec<(f32, f32)> {
    let mut x = left;
    let mut out = Vec::new();
    while x < right {
        out.push((x, if clipped { dash.min(right - x) } else { dash }));
        x += step;
    }
    out
}

pub enum Token<'a, Message> {
    Menu,
    Number { scrub: Scrub, change: Box<dyn Fn(f64) -> Message + 'a>, commit: Message },
    Text,
    Key(KeyLook),
    KeyLead(KeyLook),
    Link,
    Mention { underlined: bool },
    Underlined(Color),
}

impl<Message> Token<'_, Message> {
    fn glyph(&self) -> (&'static str, f32) {
        match self {
            Self::Menu => (TWISTY_OPEN, SMALL_GLYPH),
            Self::Number { .. } => (NUMBER_GLYPH, SMALL_GLYPH),
            Self::Text => (TEXT_GLYPH, WIDE_GLYPH),
            Self::Key(KeyLook::Add) => (ADD_GLYPH, ADD_GLYPH_SIZE),
            Self::Key(_) => (KEY_GLYPH, WIDE_GLYPH),
            Self::KeyLead(_) | Self::Link | Self::Mention { .. } | Self::Underlined(_) => {
                (NO_GLYPH, SMALL_GLYPH)
            }
        }
    }

    fn inks(&self, palette: &Palette, plane: Color, muted: bool) -> Inks {
        let rule = tint(plane, palette.primary, UNDERLINE_ALPHA);
        let faded = |alpha| tint(plane, palette.surface_text, alpha);
        let clipped = matches!(self, Self::Key(_) | Self::KeyLead(_));
        let mut inks =
            Inks { text: palette.primary, rule, stroke: Stroke::Dotted, fill: None, clipped };
        match self {
            Self::Key(KeyLook::Clash) | Self::KeyLead(KeyLook::Clash) => {
                inks.fill = Some(tint(plane, palette.primary, SELECTED_TINT));
            }
            Self::Key(KeyLook::Unbound) | Self::KeyLead(KeyLook::Unbound) => {
                inks.text = faded(UNBOUND_ALPHA);
                inks.rule = faded(UNBOUND_RULE);
            }
            Self::Key(KeyLook::Add) | Self::KeyLead(KeyLook::Add) => {
                inks.text = faded(ADD_ALPHA);
                inks.rule = faded(ADD_RULE);
                inks.stroke = Stroke::Dashed;
            }
            Self::Key(KeyLook::Listening) | Self::KeyLead(KeyLook::Listening) => {
                inks.text = palette.surface_text;
                inks.stroke = Stroke::Solid;
            }
            Self::Link => {
                inks.text = palette.surface_text;
                inks.rule = faded(LINK_RULE);
                inks.stroke = Stroke::Solid;
            }
            Self::Mention { underlined } => {
                inks.text = palette.surface_text;
                inks.rule = if *underlined { faded(MENTION_RULE) } else { Color::TRANSPARENT };
                inks.stroke = Stroke::Solid;
            }
            Self::Underlined(ink) => {
                inks.text = *ink;
                inks.rule = *ink;
                inks.stroke = Stroke::Solid;
            }
            Self::Menu
            | Self::Number { .. }
            | Self::Text
            | Self::Key(KeyLook::Bound)
            | Self::KeyLead(KeyLook::Bound) => {}
        }
        if muted {
            inks.text = faded(MUTED_ALPHA);
            inks.rule = faded(MUTED_LINE_ALPHA);
        }
        inks
    }

    fn takes_press(&self) -> bool {
        !matches!(self, Self::Key(KeyLook::Listening) | Self::KeyLead(KeyLook::Listening))
    }

    fn cursor(&self) -> mouse::Interaction {
        match self {
            Self::Number { .. } => mouse::Interaction::ResizingVertically,
            Self::Text => mouse::Interaction::Text,
            Self::Menu
            | Self::Key(_)
            | Self::KeyLead(_)
            | Self::Link
            | Self::Mention { .. }
            | Self::Underlined(_) => mouse::Interaction::Pointer,
        }
    }
}

pub enum Piece<'a, Message> {
    Plain(Cow<'a, str>),
    Aside(Cow<'a, str>),
    Soft(Cow<'a, str>),
    Faint(Cow<'a, str>),
    Value(Cow<'a, str>),
    Inked {
        text: Cow<'a, str>,
        size: f32,
        ink: Option<Color>,
    },
    Token {
        key: u64,
        label: Cow<'a, str>,
        kind: Token<'a, Message>,
        on_press: Message,
        muted: bool,
    },
}

impl<'a, Message> Piece<'a, Message> {
    pub fn plain(text: impl Into<Cow<'a, str>>) -> Self {
        Self::Plain(text.into())
    }

    pub fn aside(text: impl Into<Cow<'a, str>>) -> Self {
        Self::Aside(text.into())
    }

    pub fn inked(text: impl Into<Cow<'a, str>>, size: f32, ink: Color) -> Self {
        Self::Inked { text: text.into(), size, ink: Some(ink) }
    }

    pub fn token(
        key: u64,
        label: impl Into<Cow<'a, str>>,
        kind: Token<'a, Message>,
        on_press: Message,
    ) -> Self {
        Self::Token { key, label: label.into(), kind, on_press, muted: false }
    }

    pub fn muted(self) -> Self {
        match self {
            Self::Token { key, label, kind, on_press, .. } => {
                Self::Token { key, label, kind, on_press, muted: true }
            }
            other => other,
        }
    }
}

pub fn key_width(label: &str, look: KeyLook, size: f32) -> f32 {
    let (glyph, glyph_size) = Token::<()>::Key(look).glyph();
    cached_line_width(&unbroken(label), size, UI_FONT)
        + cached_line_width(glyph, glyph_size * legible_type_scale(1.0), UI_FONT)
}

pub fn sentinel(slot: u32) -> Option<char> {
    SENTINEL.checked_add(slot).filter(|mark| *mark <= SENTINEL_LAST).and_then(char::from_u32)
}

pub fn is_sentinel(ch: char) -> bool {
    (SENTINEL..=SENTINEL_LAST).contains(&u32::from(ch))
}

pub fn split<'a, Message>(
    resolved: &str,
    tokens: Vec<Vec<Piece<'a, Message>>>,
) -> Vec<Piece<'a, Message>> {
    let mut tokens: Vec<Option<Vec<Piece<'a, Message>>>> = tokens.into_iter().map(Some).collect();
    let mut pieces = Vec::new();
    let mut rest = resolved;
    let marks = SENTINEL..=u32::from(VALUE_CLOSE);
    while let Some((at, mark)) = rest.char_indices().find(|(_, ch)| marks.contains(&u32::from(*ch)))
    {
        if at > 0 {
            pieces.push(Piece::plain(rest[..at].to_owned()));
        }
        rest = &rest[at + mark.len_utf8()..];
        if mark == VALUE_OPEN {
            let end = rest.find(VALUE_CLOSE).unwrap_or(rest.len());
            if end > 0 {
                pieces.push(Piece::Value(rest[..end].to_owned().into()));
            }
            rest = rest.get(end + VALUE_CLOSE.len_utf8()..).unwrap_or_default();
            continue;
        }
        let slot = (u32::from(mark) - SENTINEL) as usize;
        if let Some(slot) = tokens.get_mut(slot).and_then(Option::take) {
            pieces.extend(slot);
        }
    }
    if !rest.is_empty() {
        pieces.push(Piece::plain(rest.to_owned()));
    }
    pieces.extend(tokens.into_iter().flatten().flatten());
    pieces
}

pub struct SentenceSpec<'a, Message> {
    pub pieces: Vec<Piece<'a, Message>>,
    pub size: f32,
    pub leading: f32,
    pub scale: f32,
    pub palette: Palette,
    pub plane: Color,
    pub focused: Option<u64>,
    pub ring: bool,
    pub popover: Option<Popover<'a, Message>>,
    pub centred: bool,
}

struct Sentence<'a, Message> {
    spec: SentenceSpec<'a, Message>,
}

#[derive(Clone, Copy)]
pub(super) struct Line {
    pub left: f32,
    pub right: f32,
    pub baseline: f32,
    pub rtl: bool,
}

impl Line {
    fn glyph_box(self, size: f32) -> Rectangle {
        let top = self.baseline - ASCENT * size;
        let bottom = self.baseline + DESCENT * size + DOT;
        Rectangle::new(Point::new(self.left, top), Size::new(self.right - self.left, bottom - top))
    }
}

#[derive(Clone, Copy)]
struct Press {
    key: u64,
    origin: f32,
    start: Option<f64>,
    moved: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Cut {
    piece: usize,
    at: usize,
}

#[derive(Clone, Copy)]
struct Origin {
    cut: Cut,
    line: usize,
    start: usize,
    end: usize,
}

#[derive(Default)]
struct Laid<'a> {
    spans: Vec<Span<'a, (), Font>>,
    owners: Vec<Option<usize>>,
    origins: Vec<Option<Origin>>,
    line: usize,
    offset: usize,
}

impl<'a> Laid<'a> {
    fn push(&mut self, span: Span<'a, (), Font>, owner: Option<usize>, cut: Option<Cut>) {
        let start = self.offset;
        let end = start + span.text.len();
        self.origins.push(cut.map(|cut| Origin { cut, line: self.line, start, end }));
        if span.text == LINE_BREAK {
            self.line += 1;
            self.offset = 0;
        } else {
            self.offset = end;
        }
        self.spans.push(span);
        self.owners.push(owner);
    }
}

#[derive(Default)]
struct State {
    paragraph: Paragraph,
    spans: Vec<Span<'static, (), Font>>,
    owners: Vec<Option<usize>>,
    origins: Vec<Option<Origin>>,
    hovered: Option<usize>,
    press: Option<Press>,
}

impl State {
    fn hit(&self, point: Point) -> Option<usize> {
        self.paragraph.hit_span(point).and_then(|span| self.owners.get(span).copied().flatten())
    }

    fn lines(&self, token: usize) -> Vec<Line> {
        token_lines(&self.paragraph, &self.owners, token)
    }
}

pub(super) fn unbroken(label: &str) -> String {
    let mut joined = glued(label);
    joined.push(GLYPH_GAP);
    joined
}

fn shown<Message>(kind: &Token<'_, Message>, label: &str, rtl: bool) -> String {
    let word = if matches!(kind, Token::Text) { wrapping(label) } else { glued(label) };
    let mut spelled =
        if rtl && !has_rtl(label) { format!("{ISOLATE}{word}{ISOLATE_END}") } else { word };
    if !matches!(kind, Token::Mention { .. } | Token::Underlined(_)) {
        spelled.push(GLYPH_GAP);
    }
    spelled
}

fn wrapping(label: &str) -> String {
    let mut words = label.split(' ').map(glued);
    let mut joined = words.next().unwrap_or_default();
    for word in words {
        joined.push(' ');
        joined.push_str(&word);
    }
    joined
}

fn glued(label: &str) -> String {
    let glue = !shapes_across(label);
    let mut joined = String::with_capacity(label.len() * 2);
    let mut after_break = false;
    for (idx, cluster) in label.graphemes(true).enumerate() {
        if idx > 0 && (glue || after_break) {
            joined.push(JOINER);
        }
        joined.push_str(if cluster == " " { NO_BREAK_SPACE } else { cluster });
        after_break = BREAKS.contains(&cluster);
    }
    joined
}

fn overflow(state: &State, width: f32) -> Option<usize> {
    let run = state.paragraph.buffer().layout_runs().find(|run| run.line_w > width)?;
    run.glyphs.iter().filter_map(|glyph| state.owners.get(glyph.metadata).copied().flatten()).max()
}

fn cut(state: &State, width: f32) -> Option<Cut> {
    let run = state.paragraph.buffer().layout_runs().find(|run| run.line_w > width)?;
    let mut glyphs: Vec<_> = run.glyphs.iter().collect();
    glyphs.sort_by_key(|glyph| glyph.start);
    let mut used = 0.0;
    let spill = glyphs.into_iter().find(|glyph| {
        used += glyph.w;
        used > width
    })?;
    let head = run.text.get(..spill.start)?;
    let at = head.rfind(char::is_whitespace).filter(|at| *at > 0).unwrap_or(spill.start);
    if at == 0 {
        return None;
    }
    state.origins.iter().flatten().find_map(|origin| {
        let inside = origin.line == run.line_i && (origin.start..origin.end).contains(&at);
        inside.then(|| Cut { at: origin.cut.at + at - origin.start, ..origin.cut })
    })
}

pub(super) fn token_lines(
    paragraph: &Paragraph,
    owners: &[Option<usize>],
    token: usize,
) -> Vec<Line> {
    paragraph
        .buffer()
        .layout_runs()
        .filter_map(|run| {
            let mut glyphs =
                run.glyphs.iter().filter(|glyph| owners.get(glyph.metadata) == Some(&Some(token)));
            let first = glyphs.next()?;
            let (left, right) = glyphs
                .fold((first.x, first.x + first.w), |(left, right), glyph| {
                    (left.min(glyph.x), right.max(glyph.x + glyph.w))
                });
            Some(Line { left, right, baseline: run.line_y, rtl: run.rtl })
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn laid(tree: &widget::Tree) -> Option<&Paragraph> {
    (tree.tag == widget::tree::Tag::of::<State>())
        .then(|| &tree.state.downcast_ref::<State>().paragraph)
}

impl<'a, Message> Sentence<'a, Message> {
    fn type_size(&self) -> f32 {
        self.spec.size * legible_type_scale(self.spec.scale)
    }

    fn tokens(&self) -> impl Iterator<Item = (u64, &Token<'a, Message>, &Message)> {
        self.spec.pieces.iter().filter_map(|piece| match piece {
            Piece::Token { key, kind, on_press, .. } => Some((*key, kind, on_press)),
            Piece::Plain(_)
            | Piece::Aside(_)
            | Piece::Soft(_)
            | Piece::Faint(_)
            | Piece::Inked { .. }
            | Piece::Value(_) => None,
        })
    }

    fn token(&self, key: u64) -> Option<(&Token<'a, Message>, &Message)> {
        self.tokens().find(|(token, ..)| *token == key).map(|(_, kind, on_press)| (kind, on_press))
    }

    fn key_at(&self, ordinal: usize) -> Option<u64> {
        self.tokens().nth(ordinal).map(|(key, ..)| key)
    }

    fn ordinal(&self, key: u64) -> Option<usize> {
        self.tokens().position(|(token, ..)| token == key)
    }

    fn anchor_ordinal(&self, popover: &Popover<'a, Message>) -> Option<usize> {
        let mut holders = self
            .tokens()
            .enumerate()
            .filter(|(_, (token, ..))| *token == popover.key)
            .map(|(ordinal, _)| ordinal);
        if popover.side == Side::AboveEnd { holders.next() } else { holders.last() }
    }

    fn ringed(&self, key: u64) -> bool {
        self.spec.ring && self.spec.focused == Some(key)
    }

    fn muted(&self, key: u64) -> bool {
        self.spec.pieces.iter().any(
            |piece| matches!(piece, Piece::Token { key: token, muted: true, .. } if *token == key),
        )
    }

    fn covered(&self, key: u64) -> bool {
        self.spec
            .popover
            .as_ref()
            .is_some_and(|popover| popover.side == Side::Over && popover.key == key)
    }

    fn spans(&self, breaks: &[usize], cuts: &[Cut]) -> Laid<'_> {
        let palette = self.spec.palette;
        let plane = self.spec.plane;
        let glyph_scale = legible_type_scale(self.spec.scale);
        let line = text::LineHeight::Absolute(Pixels(self.type_size() * self.spec.leading));
        let mut laid = Laid::default();
        let rtl = crate::frontend::ui::rtl();
        let newline = |laid: &mut Laid<'_>| {
            laid.push(Span::new(LINE_BREAK), None, None);
            if rtl {
                laid.push(Span::new(RTL_MARK), None, None);
            }
        };
        let mut ordinal = 0;
        for (index, piece) in self.spec.pieces.iter().enumerate() {
            match piece {
                Piece::Plain(text)
                | Piece::Aside(text)
                | Piece::Soft(text)
                | Piece::Faint(text)
                | Piece::Inked { text, .. } => {
                    let aside = matches!(piece, Piece::Aside(_));
                    let (ink, size) = match piece {
                        Piece::Inked { size, ink, .. } => (*ink, *size),
                        _ => (
                            Some(tint(plane, palette.surface_text, text_alpha(piece))),
                            if aside { ASIDE_SIZE } else { self.spec.size } * glyph_scale,
                        ),
                    };
                    let mut at = 0;
                    let mut ends: Vec<_> =
                        cuts.iter().filter(|cut| cut.piece == index).map(|cut| cut.at).collect();
                    ends.sort_unstable();
                    for (nth, end) in ends.into_iter().chain([text.len()]).enumerate() {
                        let Some(part) = text.get(at..end) else {
                            continue;
                        };
                        if nth > 0 {
                            newline(&mut laid);
                        }
                        let kept = if nth > 0 { part.trim_start() } else { part };
                        let origin = Cut { piece: index, at: at + part.len() - kept.len() };
                        let mut span = Span::new(kept).color_maybe(ink).size(size);
                        if aside {
                            span = span.line_height(text::LineHeight::Absolute(Pixels(
                                size * ASIDE_LEADING,
                            )));
                        }
                        laid.push(span, None, Some(origin));
                        at = end;
                    }
                }
                Piece::Value(text) => {
                    laid.push(Span::new(text.as_ref()).color(palette.surface_text), None, None);
                }
                Piece::Token { key, label, kind, muted, .. } => {
                    let hidden = self.covered(*key);
                    let ink = |color: Color| if hidden { Color::TRANSPARENT } else { color };
                    let (glyph, size) = kind.glyph();
                    if breaks.contains(&ordinal) {
                        newline(&mut laid);
                    }
                    let inks = kind.inks(&palette, plane, *muted);
                    laid.push(
                        Span::new(shown(kind, label, rtl)).color(ink(inks.text)),
                        Some(ordinal),
                        None,
                    );
                    if !glyph.is_empty() {
                        laid.push(
                            Span::new(glyph)
                                .size(size * glyph_scale)
                                .line_height(line)
                                .color(ink(tint(plane, palette.primary, GLYPH_ALPHA))),
                            Some(ordinal),
                            None,
                        );
                    }
                    ordinal += 1;
                }
            }
        }
        laid
    }

    fn shape(&self, state: &mut State, limits: &layout::Limits) -> layout::Node {
        layout::sized(limits, Length::Shrink, Length::Shrink, |limits| {
            let bounds = limits.max();
            let mut breaks = Vec::new();
            let mut cuts = Vec::new();
            loop {
                self.reshape(state, bounds, &breaks, &cuts);
                if let Some(token) = overflow(state, bounds.width)
                    && !breaks.contains(&token)
                {
                    breaks.push(token);
                    continue;
                }
                match cut(state, bounds.width) {
                    Some(next) if !cuts.contains(&next) => cuts.push(next),
                    _ => break,
                }
            }
            state.paragraph.min_bounds()
        })
    }

    fn reshape(&self, state: &mut State, bounds: Size, breaks: &[usize], cuts: &[Cut]) {
        let Laid { spans, owners, origins, .. } = self.spans(breaks, cuts);
        let shape = |content| text::Text {
            content,
            bounds,
            size: Pixels(self.type_size()),
            line_height: text::LineHeight::Relative(self.spec.leading),
            font: UI_FONT,
            align_x: if self.spec.centred {
                alignment::Horizontal::Center.into()
            } else {
                text::Alignment::Default
            },
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::WordOrGlyph,
        };
        if state.spans == spans {
            match state.paragraph.compare(shape(spans.as_slice()).with_content(())) {
                text::Difference::None => {}
                text::Difference::Bounds => state.paragraph.resize(bounds),
                text::Difference::Shape => {
                    state.paragraph = Paragraph::with_spans(shape(spans.as_slice()));
                }
            }
        } else {
            state.paragraph = Paragraph::with_spans(shape(spans.as_slice()));
            state.spans = spans.iter().cloned().map(Span::to_static).collect();
        }
        state.owners = owners;
        state.origins = origins;
    }

    fn handle(
        &self,
        state: &mut State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) where
        Message: Clone,
    {
        let over = cursor.position_in(bounds).and_then(|point| state.hit(point));
        if over != state.hovered {
            state.hovered = over;
            shell.request_redraw();
        }
        let over = over.and_then(|ordinal| self.key_at(ordinal));
        let pointer = cursor.position().map(|point| point.y);
        match (event, pointer) {
            (Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), Some(y)) => {
                self.press(state, over, y, shell);
            }
            (Event::Mouse(mouse::Event::CursorMoved { .. }), Some(y)) => self.drag(state, y, shell),
            (Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)), _) => {
                self.release(state, over, shell);
            }
            (Event::Mouse(mouse::Event::WheelScrolled { delta }), _) => {
                self.wheel(over, *delta, shell);
            }
            (Event::Keyboard(keyboard::Event::KeyPressed { key, .. }), _) => self.key(key, shell),
            _ => {}
        }
    }

    fn press(&self, state: &mut State, over: Option<u64>, y: f32, shell: &mut Shell<'_, Message>) {
        let Some(key) =
            over.filter(|key| self.token(*key).is_some_and(|(kind, _)| kind.takes_press()))
        else {
            return;
        };
        let start = match self.token(key) {
            Some((Token::Number { scrub, .. }, _)) => Some(scrub.value),
            _ => None,
        };
        state.press = Some(Press { key, origin: y, start, moved: false });
        shell.capture_event();
        shell.request_redraw();
    }

    fn drag(&self, state: &mut State, y: f32, shell: &mut Shell<'_, Message>) {
        let Some(press) = state.press.as_mut() else {
            return;
        };
        let (Some(start), Some((Token::Number { scrub, change, .. }, _))) =
            (press.start, self.token(press.key))
        else {
            return;
        };
        let rise = press.origin - y;
        press.moved |= rise.abs() > DRAG_SLOP;
        if !press.moved {
            return;
        }
        let value = Scrub { value: start, ..*scrub }.nudge(f64::from((rise / DRAG_STEP).round()));
        if value != scrub.value {
            shell.publish(change(value));
        }
        shell.capture_event();
    }

    fn release(&self, state: &mut State, over: Option<u64>, shell: &mut Shell<'_, Message>)
    where
        Message: Clone,
    {
        let Some(press) = state.press.take() else {
            return;
        };
        shell.capture_event();
        shell.request_redraw();
        match self.token(press.key) {
            Some((Token::Number { commit, .. }, _)) if press.moved => shell.publish(commit.clone()),
            Some((_, on_press)) if !press.moved && over == Some(press.key) => {
                shell.publish(on_press.clone());
            }
            _ => {}
        }
    }

    fn wheel(&self, over: Option<u64>, delta: mouse::ScrollDelta, shell: &mut Shell<'_, Message>)
    where
        Message: Clone,
    {
        let Some(focused) = self.spec.focused.filter(|key| over == Some(*key)) else {
            return;
        };
        let Some((Token::Number { scrub, change, commit }, _)) = self.token(focused) else {
            return;
        };
        let (mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. }) = delta;
        if y == 0.0 {
            return;
        }
        shell.publish(change(scrub.nudge(f64::from(y.signum()))));
        shell.publish(commit.clone());
        shell.capture_event();
    }

    fn key(&self, key: &Key, shell: &mut Shell<'_, Message>)
    where
        Message: Clone,
    {
        let Key::Named(named) = key else {
            return;
        };
        if shell.is_event_captured() {
            return;
        }
        if let Some(popover) = &self.spec.popover {
            let steps = match named {
                Named::ArrowUp => 1.0,
                Named::ArrowDown => -1.0,
                _ => return,
            };
            let Some((Token::Number { scrub, change, .. }, _)) =
                self.token(popover.key).filter(|_| popover.side == Side::Over)
            else {
                return;
            };
            shell.publish(change(scrub.nudge(steps)));
            shell.capture_event();
            return;
        }
        let Some((kind, on_press)) = self.spec.focused.and_then(|focused| self.token(focused))
        else {
            return;
        };
        let number = matches!(kind, Token::Number { .. });
        if matches!(named, Named::Enter) || matches!(named, Named::Space) && !number {
            shell.publish(on_press.clone());
            shell.capture_event();
        }
    }

    fn decorate(&self, state: &State, renderer: &mut iced::Renderer, origin: Vector) {
        let size = self.type_size();
        let palette = self.spec.palette;
        let active = state.press.and_then(|press| self.ordinal(press.key)).or(state.hovered);
        for (ordinal, (key, kind, _)) in self.tokens().enumerate() {
            if self.covered(key) {
                continue;
            }
            let ring = self.ringed(key);
            let inks = kind.inks(&palette, self.spec.plane, self.muted(key));
            for line in state.lines(ordinal) {
                if let Some(fill) = inks.fill {
                    rule(renderer, line.glyph_box(size) + origin, fill);
                }
                let under = line.baseline + DESCENT * size;
                let lit = active == Some(ordinal);
                let linked =
                    matches!(kind, Token::Link | Token::Mention { .. } | Token::Underlined(_));
                let ink = if lit && linked { inks.text } else { inks.rule };
                let (dash, step) = match (lit, inks.stroke) {
                    (true, _) | (_, Stroke::Solid) => {
                        (line.right - line.left, line.right - line.left)
                    }
                    (_, Stroke::Dotted) => (DOT, DOT * 2.0),
                    (_, Stroke::Dashed) => (DASH, DASH * 2.0),
                };
                for (x, width) in dashes(line.left, line.right, dash, step, inks.clipped) {
                    let piece = Size::new(width, DOT);
                    rule(renderer, Rectangle::new(Point::new(x, under), piece) + origin, ink);
                }
                if ring {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: line.glyph_box(size).expand(RING + RING_OFFSET) + origin,
                            border: Border {
                                color: palette.primary,
                                width: RING,
                                ..Border::default()
                            },
                            ..renderer::Quad::default()
                        },
                        Background::Color(Color::TRANSPARENT),
                    );
                }
            }
        }
    }
}

fn rule(renderer: &mut iced::Renderer, bounds: Rectangle, color: Color) {
    renderer.fill_quad(renderer::Quad { bounds, ..renderer::Quad::default() }, color);
}

impl<Message: Clone> Widget<Message, Theme, iced::Renderer> for Sentence<'_, Message> {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.spec
            .popover
            .iter()
            .map(|popover| widget::Tree::new(popover.content.as_widget()))
            .collect()
    }

    fn diff(&self, tree: &mut widget::Tree) {
        match &self.spec.popover {
            Some(popover) => tree.diff_children(std::slice::from_ref(&popover.content)),
            None => tree.children.clear(),
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.shape(tree.state.downcast_mut::<State>(), limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if !bounds.intersects(viewport) {
            return;
        }
        let state = tree.state.downcast_ref::<State>();
        self.decorate(state, renderer, bounds.position() - Point::ORIGIN);
        iced::advanced::widget::text::draw(
            renderer,
            style,
            bounds,
            &state.paragraph,
            iced::advanced::widget::text::Style::default(),
            viewport,
        );
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        self.handle(tree.state.downcast_mut::<State>(), event, layout.bounds(), cursor, shell);
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let dragged = state.press.filter(|press| press.moved).map(|press| press.key);
        let hovered = || {
            cursor
                .position_in(layout.bounds())
                .and_then(|point| state.hit(point))
                .and_then(|ordinal| self.key_at(ordinal))
        };
        dragged
            .or_else(hovered)
            .and_then(|key| self.token(key))
            .map_or(mouse::Interaction::None, |(kind, _)| kind.cursor())
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        _renderer: &iced::Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        let ordinal = self.anchor_ordinal(self.spec.popover.as_ref()?)?;
        let size = self.type_size();
        let line = *tree.state.downcast_ref::<State>().lines(ordinal).last()?;
        let popover = self.spec.popover.as_mut()?;
        let anchor = line.glyph_box(size) + (layout.position() - Point::ORIGIN) + translation;
        Some(overlay::Element::new(Box::new(Anchored {
            content: &mut popover.content,
            tree: &mut tree.children[0],
            anchor,
            side: popover.side,
            rtl: line.rtl,
        })))
    }
}

pub fn sentence<'a, Message: Clone + 'a>(spec: SentenceSpec<'a, Message>) -> Element<'a, Message> {
    Element::new(Sentence { spec })
}

pub fn prose<Message>(pieces: Vec<Piece<'_, Message>>, leading: f32) -> SentenceSpec<'_, Message> {
    SentenceSpec {
        pieces,
        size: SAY_SIZE,
        leading,
        scale: 1.0,
        palette: Palette::default(),
        plane: Color::TRANSPARENT,
        focused: None,
        ring: false,
        popover: None,
        centred: false,
    }
}

pub fn worded<'a, Message: Clone + 'a>(
    copy: impl Into<Cow<'a, str>>,
    size: f32,
    scale: f32,
    ink: Color,
    leading: f32,
) -> Element<'a, Message> {
    let size = size * legible_type_scale(scale);
    sentence(prose(vec![Piece::inked(copy, size, ink)], leading))
}
