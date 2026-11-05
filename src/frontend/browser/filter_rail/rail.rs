use std::collections::HashMap;
use std::time::{Duration, Instant};

use iced::widget::canvas::{self, Action, Frame, Path, Stroke};
use iced::widget::{button, canvas as canvas_widget, column, container, keyed_column};
use iced::{Alignment, Color, Element, Event, Length, Point, Rectangle, Size, mouse, window};

use crate::frontend::animation::{MotionProfile, MotionTier, smoothstep};
use crate::frontend::components::lerp_color;
use crate::frontend::theme::Palette;
use crate::frontend::ui::browser_bar::{SECTION_RULE_ALPHA, SECTION_TEXT_ALPHA, SECTION_TEXT_SIZE};
use crate::frontend::ui::{
    UI_FONT, cached_line_width, diamond, folio_horizontal_rule, folio_line_button_style, label,
    mid_text, mirror_x_for, row, rtl, with_alpha,
};
use crate::i18n::tr;

use crate::frontend::browser::model::{Browser, BrowserIntent, BrowserMsg, Source, TagMark};

const COLUMNS: usize = 2;
const ROW_HEIGHT: f32 = 26.0;
const DIAMOND_RADIUS: f32 = 4.5;
const DIAMOND_INSET: f32 = 10.0;
const LABEL_INSET: f32 = 22.0;
const LABEL_SIZE: f32 = 10.0;
const LABEL_ALPHA: f32 = 0.72;
const EXCLUDED_ALPHA: f32 = 0.8;
const DIAMOND_ALPHA: f32 = 0.55;
const HOVER_ALPHA: f32 = 0.45;
const BAR_WIDTH: f32 = 1.5;
const BAR_SPAN: f32 = 0.7;
const SWATCH_SHARE: f32 = 0.3;
const RING_GAP: f32 = 3.0;
const RING_WIDTH: f32 = 1.5;
const GAP: f32 = 8.0;
const GROUP_GAP: f32 = 18.0;
const ACTION_PADDING: [f32; 2] = [1.0, 4.0];
const ACTION_SEPARATOR: &str = "\u{00b7}";
const FRAME: Duration = Duration::from_millis(16);
const SETTLE: f32 = 0.002;
const MAX_STEP: f32 = 0.1;

pub(super) struct Entry {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) mark: TagMark,
    pub(super) press: BrowserMsg,
    pub(super) exclude: Option<BrowserMsg>,
    pub(super) swatch: Option<Color>,
}

impl Entry {
    pub(super) fn choice(
        id: impl Into<String>,
        title: &str,
        chosen: bool,
        press: BrowserMsg,
    ) -> Self {
        let mark = if chosen { TagMark::Include } else { TagMark::Off };
        Self { id: id.into(), title: title.to_owned(), mark, press, exclude: None, swatch: None }
    }
}

pub(super) struct HeadingAction {
    pub(super) key: &'static str,
    pub(super) current: bool,
    pub(super) message: Option<BrowserMsg>,
}

pub(super) struct Section {
    pub(super) title_key: &'static str,
    pub(super) entries: Vec<Entry>,
    pub(super) actions: Vec<HeadingAction>,
    pub(super) columns: usize,
}

impl Section {
    pub(super) fn new(title_key: &'static str, entries: Vec<Entry>) -> Self {
        Self { title_key, entries, actions: Vec::new(), columns: COLUMNS }
    }
}

pub(super) fn choices(
    title_key: &'static str,
    keys: impl IntoIterator<Item = &'static str>,
    title: impl Fn(&'static str) -> &'static str,
    chosen: impl Fn(&str) -> bool,
    press: impl Fn(&'static str) -> BrowserMsg,
) -> Section {
    let entries = keys
        .into_iter()
        .map(|key| Entry::choice(key, title(key), chosen(key), press(key)))
        .collect();
    Section::new(title_key, entries)
}

pub(super) fn sections(browser: &Browser) -> Option<Vec<Section>> {
    match browser.source {
        Source::Steam => Some(super::steam::sections(browser)),
        Source::Wallhaven => Some(super::wallhaven::sections(browser)),
        _ => None,
    }
}

pub(in crate::frontend::browser) fn filter_rail<'a>(
    browser: &Browser,
    width: f32,
    scale: f32,
    fade: f32,
    pal: &'a Palette,
) -> Option<Element<'a, BrowserIntent>> {
    let group = Group { width, scale, fade, motion: browser.view.motion, pal };
    let sections = sections(browser)?;
    Some(
        keyed_column(
            sections.into_iter().map(|section| (section.title_key, group.section(section))),
        )
        .spacing(GROUP_GAP * scale)
        .into(),
    )
}

struct Group<'a> {
    width: f32,
    scale: f32,
    fade: f32,
    motion: MotionProfile,
    pal: &'a Palette,
}

impl<'a> Group<'a> {
    fn section(&self, section: Section) -> Element<'a, BrowserIntent> {
        let (scale, fade, pal) = (self.scale, self.fade, self.pal);
        let mut heading = row![
            label(
                tr(section.title_key),
                SECTION_TEXT_SIZE,
                scale,
                with_alpha(pal.surface_text, SECTION_TEXT_ALPHA * fade),
            ),
            container(folio_horizontal_rule(with_alpha(pal.outline, SECTION_RULE_ALPHA * fade)))
                .width(Length::Fill),
        ]
        .spacing(GAP * scale)
        .align_y(Alignment::Center);
        let idle = with_alpha(pal.surface_text, SECTION_TEXT_ALPHA * fade);
        for (index, action) in section.actions.into_iter().enumerate() {
            if index > 0 {
                heading = heading.push(label(ACTION_SEPARATOR, SECTION_TEXT_SIZE, scale, idle));
            }
            let tone = if action.current { with_alpha(pal.primary, fade) } else { idle };
            heading = heading.push(
                button(label(tr(action.key), SECTION_TEXT_SIZE, scale, tone))
                    .padding(ACTION_PADDING.map(|side| side * scale))
                    .on_press_maybe(action.message.map(BrowserIntent::Update))
                    .style(move |_, status| {
                        folio_line_button_style(false, false, pal, fade, status)
                    }),
            );
        }
        let columns = section.columns.max(1);
        let rows = section.entries.len().div_ceil(columns);
        let grid = canvas_widget(OptionGrid {
            entries: section.entries,
            rows,
            columns,
            scale,
            fade,
            motion: self.motion,
            palette: *pal,
        })
        .width(Length::Fixed(self.width))
        .height(Length::Fixed(rows as f32 * ROW_HEIGHT * scale));
        column![heading, grid].spacing(GAP * scale).into()
    }
}

pub(super) fn fitted_size(title: &str, room: f32, size: f32) -> f32 {
    let width = cached_line_width(title, size, UI_FONT);
    if width <= room { size } else { size * room.max(0.0) / width }
}

pub(super) struct Layout {
    pub(super) rows: usize,
    pub(super) columns: usize,
    pub(super) count: usize,
}

pub(super) fn cell_at(position: Point, size: Size, layout: &Layout, rtl: bool) -> Option<usize> {
    if position.x < 0.0 || position.y < 0.0 || position.x >= size.width || layout.rows == 0 {
        return None;
    }
    let column_width = size.width / layout.columns as f32;
    let row = (position.y / (size.height / layout.rows as f32)) as usize;
    let column = (position.x / column_width) as usize;
    let column = if rtl { layout.columns - 1 - column } else { column };
    let index = row * layout.columns + column;
    (row < layout.rows && index < layout.count).then_some(index)
}

#[derive(Clone, Copy, Default)]
pub(super) struct Mix {
    pub(super) include: f32,
    pub(super) exclude: f32,
}

impl Mix {
    fn of(mark: TagMark) -> Self {
        Self {
            include: f32::from(u8::from(mark == TagMark::Include)),
            exclude: f32::from(u8::from(mark == TagMark::Exclude)),
        }
    }
}

#[derive(Default)]
pub(super) struct GridState {
    pub(super) mixes: HashMap<String, Mix>,
    pub(super) last_frame: Option<Instant>,
    hovered: Option<usize>,
}

pub(super) struct OptionGrid {
    pub(super) entries: Vec<Entry>,
    pub(super) rows: usize,
    pub(super) columns: usize,
    pub(super) scale: f32,
    pub(super) fade: f32,
    pub(super) motion: MotionProfile,
    pub(super) palette: Palette,
}

impl OptionGrid {
    pub(super) fn advance(&self, state: &mut GridState, now: Instant) -> bool {
        let dt = state
            .last_frame
            .replace(now)
            .map_or(0.0, |last| now.saturating_duration_since(last).as_secs_f32().min(MAX_STEP));
        let mut moving = false;
        for entry in &self.entries {
            let target = Mix::of(entry.mark);
            let mix = state.mixes.entry(entry.id.clone()).or_insert(target);
            for (value, goal) in
                [(&mut mix.include, target.include), (&mut mix.exclude, target.exclude)]
            {
                *value = self.motion.approach(*value, goal, dt, MotionTier::Fast);
                if (*value - goal).abs() < SETTLE {
                    *value = goal;
                }
                moving |= *value != goal;
            }
        }
        if !moving {
            state.last_frame = None;
        }
        moving
    }

    fn hit(&self, bounds: Rectangle, cursor: mouse::Cursor) -> Option<usize> {
        let position = cursor.position_in(bounds)?;
        let layout = Layout { rows: self.rows, columns: self.columns, count: self.entries.len() };
        cell_at(position, bounds.size(), &layout, rtl())
    }

    fn draw_swatch(&self, frame: &mut Frame, cell: Rectangle, colour: Color, include: f32) {
        let centre = cell.center();
        let radius = cell.width.min(cell.height) * SWATCH_SHARE;
        frame.fill(&diamond(centre, radius), with_alpha(colour, self.fade));
        if include > 0.0 {
            frame.stroke(
                &diamond(centre, radius + RING_GAP * self.scale),
                Stroke::default()
                    .with_color(with_alpha(self.palette.primary, include * self.fade))
                    .with_width(RING_WIDTH * self.scale),
            );
        }
    }

    fn draw_cell(
        &self,
        frame: &mut Frame,
        cell: Rectangle,
        entry: &Entry,
        mix: Mix,
        hovered: bool,
    ) {
        let pal = &self.palette;
        let rtl = rtl();
        if hovered {
            frame.fill_rectangle(
                cell.position(),
                cell.size(),
                with_alpha(pal.hover, HOVER_ALPHA * self.fade),
            );
        }
        let include = smoothstep(mix.include);
        let exclude = smoothstep(mix.exclude);
        if let Some(colour) = entry.swatch {
            self.draw_swatch(frame, cell, colour, include);
            return;
        }
        let lead = |inset: f32| {
            if rtl { cell.x + cell.width - inset * self.scale } else { cell.x + inset * self.scale }
        };
        let centre = Point::new(lead(DIAMOND_INSET), cell.center_y());
        let radius = DIAMOND_RADIUS * self.scale;
        let shape = diamond(centre, radius);
        let resting = 1.0 - include.max(exclude);
        frame.stroke(
            &shape,
            Stroke::default()
                .with_color(with_alpha(pal.outline, DIAMOND_ALPHA * resting * self.fade))
                .with_width(self.scale),
        );
        if include > 0.0 {
            frame.fill(&diamond(centre, radius * include), with_alpha(pal.primary, self.fade));
            frame.stroke(
                &shape,
                Stroke::default()
                    .with_color(with_alpha(pal.primary, include * self.fade))
                    .with_width(self.scale),
            );
        }
        if exclude > 0.0 {
            let destructive = pal.destructive();
            frame.stroke(
                &shape,
                Stroke::default()
                    .with_color(with_alpha(destructive, exclude * self.fade))
                    .with_width(self.scale),
            );
            let reach = radius * BAR_SPAN * exclude;
            frame.stroke(
                &Path::line(
                    Point::new(centre.x - reach, centre.y),
                    Point::new(centre.x + reach, centre.y),
                ),
                Stroke::default()
                    .with_color(with_alpha(destructive, exclude * self.fade))
                    .with_width(BAR_WIDTH * self.scale),
            );
        }
        let resting_text = with_alpha(pal.surface_text, LABEL_ALPHA);
        let excluded = with_alpha(pal.destructive(), EXCLUDED_ALPHA);
        let tone = lerp_color(lerp_color(resting_text, pal.primary, include), excluded, exclude);
        let anchor = Point::new(lead(LABEL_INSET), cell.center_y());
        let room = cell.width - (LABEL_INSET + DIAMOND_INSET) * self.scale;
        let size = fitted_size(&entry.title, room, LABEL_SIZE * self.scale);
        frame.with_clip(cell, |frame| {
            frame.fill_text(mid_text(
                entry.title.clone(),
                anchor,
                with_alpha(tone, tone.a * self.fade),
                size,
                UI_FONT,
                if rtl { Alignment::End } else { Alignment::Start },
            ));
        });
    }
}

impl canvas::Program<BrowserIntent> for OptionGrid {
    type State = GridState;

    fn update(
        &self,
        state: &mut GridState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<BrowserIntent>> {
        match event {
            Event::Window(window::Event::RedrawRequested(now)) => {
                self.advance(state, *now).then(|| Action::request_redraw_at(*now + FRAME))
            }
            Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft) => {
                let hovered = self.hit(bounds, cursor);
                (state.hovered != hovered).then(|| {
                    state.hovered = hovered;
                    Action::request_redraw()
                })
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                let entry = &self.entries[self.hit(bounds, cursor)?];
                let message = match button {
                    mouse::Button::Left => entry.press.clone(),
                    mouse::Button::Right => entry.exclude.clone()?,
                    _ => return None,
                };
                Some(Action::publish(BrowserIntent::Update(message)).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &GridState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let row_height = ROW_HEIGHT * self.scale;
        let column_width = bounds.width / self.columns as f32;
        for row in 1..=self.rows {
            let y = row as f32 * row_height - self.scale * 0.5;
            frame.stroke(
                &Path::line(Point::new(0.0, y), Point::new(bounds.width, y)),
                Stroke::default()
                    .with_color(with_alpha(self.palette.outline, SECTION_RULE_ALPHA * self.fade))
                    .with_width(self.scale),
            );
        }
        for (index, entry) in self.entries.iter().enumerate() {
            let row = index / self.columns;
            let column = index % self.columns;
            let x = mirror_x_for(rtl(), column as f32 * column_width, column_width, bounds.width);
            let cell = Rectangle::new(
                Point::new(x, row as f32 * row_height),
                Size::new(column_width, row_height),
            );
            let mix = state.mixes.get(&entry.id).copied().unwrap_or_else(|| Mix::of(entry.mark));
            self.draw_cell(&mut frame, cell, entry, mix, state.hovered == Some(index));
        }
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &GridState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.hit(bounds, cursor).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}
