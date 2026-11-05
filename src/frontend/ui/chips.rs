use iced::widget::canvas::{self, Action, Frame, Stroke};
use iced::{Alignment, Color, Element, Event, Point, Rectangle, Size, mouse};

use crate::frontend::animation::smoothstep;
use crate::frontend::theme::Palette;

use super::bar::{BarItem, control_background, control_border, control_text, draw_button};
use super::misc::{NERD_FONT, UI_FONT, mid_text, text_width};
use super::{mirror_x_for, parallelogram, rtl, with_alpha};

pub struct SkwdChip<Message> {
    label: String,
    nerd: bool,
    text_size: f32,
    active: bool,
    tint: Option<Color>,
    msg: Message,
    pal: Palette,
}

impl<Message: Clone> canvas::Program<Message> for SkwdChip<Message> {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && cursor.is_over(bounds)
        {
            return Some(Action::publish(self.msg.clone()).and_capture());
        }
        None
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let hovered = cursor.is_over(bounds);
        let mut frame = Frame::new(renderer, bounds.size());
        let skew = chip_skew(bounds.height);
        if let Some(tint) = self.tint {
            let path = parallelogram(0.0, 0.0, bounds.width, bounds.height, skew);
            let fill = if self.active {
                tint
            } else if hovered {
                with_alpha(tint, 0.5)
            } else {
                with_alpha(tint, 0.22)
            };
            frame.fill(&path, fill);
            frame
                .stroke(&path, Stroke::default().with_color(with_alpha(tint, 0.6)).with_width(1.0));
            frame.fill_text(mid_text(
                self.label.clone(),
                Point::new(bounds.width / 2.0, bounds.height / 2.0),
                if self.active { self.pal.primary_text } else { tint },
                self.text_size,
                if self.nerd { NERD_FONT } else { UI_FONT },
                Alignment::Center,
            ));
            return vec![frame.into_geometry()];
        }
        let item = BarItem {
            x: 0.0,
            y: 0.0,
            w: bounds.width,
            h: bounds.height,
            skew,
            label: self.label.clone(),
            nerd: self.nerd,
            text_size: self.text_size,
            swatch: None,
            notice: None,
            active: self.active,
            action: None,
            z: 0,
        };
        draw_button(
            &mut crate::frontend::ui::FadeFrame::new(&mut frame, 1.0),
            &item,
            hovered,
            &self.pal,
        );
        vec![frame.into_geometry()]
    }
}

pub fn chip_skew(height: f32) -> f32 {
    (height * 0.4).min(10.0)
}

pub fn skwd_chip<'a, Message: Clone + 'a>(
    label: impl Into<String>,
    nerd: bool,
    text_size: f32,
    active: bool,
    width: iced::Length,
    height: f32,
    msg: Message,
    pal: &Palette,
) -> Element<'a, Message> {
    iced::widget::canvas(SkwdChip {
        label: label.into(),
        nerd,
        text_size,
        active,
        tint: None,
        msg,
        pal: *pal,
    })
    .width(width)
    .height(iced::Length::Fixed(height))
    .into()
}

pub fn chip_width(label: &str, text_size: f32, height: f32) -> f32 {
    label.chars().count() as f32 * text_size * 0.6 + chip_skew(height) * 2.0 + 16.0
}

pub fn pl_chip<'a, Message: Clone + 'a>(
    label: impl Into<String>,
    active: bool,
    msg: Message,
    scale: f32,
    pal: &Palette,
) -> Element<'a, Message> {
    let label = label.into();
    let ts = 12.0 * scale;
    let h = 26.0 * scale;
    let w = chip_width(&label, ts, h);
    skwd_chip(label, false, ts, active, iced::Length::Fixed(w), h, msg, pal)
}

pub struct TagChips<'a> {
    pub entries: std::rc::Rc<Vec<crate::domain::library::search::TagEntry>>,
    pub pal: &'a Palette,
    pub width: f32,
    pub max_h: f32,
    pub scale: f32,
    pub entrance: f32,
    pub scroll: f32,
    pub scroll_target: f32,
}

const TAG_CHIP_H: f32 = 27.0;
const TAG_CHIP_GAP_X: f32 = 6.0;
const TAG_CHIP_GAP_Y: f32 = 6.0;
pub(crate) const TAG_CHIP_TEXT: f32 = 11.0;

pub(crate) const EXCLUDED_MARK: &str = "\u{2212} ";

fn chip_label(entry: &crate::domain::library::search::TagEntry) -> String {
    let up = entry.tag.to_uppercase();
    if entry.excluded { format!("{EXCLUDED_MARK}{up}") } else { up }
}

fn chip_count_label(entry: &crate::domain::library::search::TagEntry) -> String {
    if entry.count >= 1000 {
        format!("{:.1}k", entry.count as f32 / 1000.0)
    } else {
        entry.count.to_string()
    }
}

fn chip_contains(x: f32, y: f32, w: f32, h: f32, skew: f32, px: f32, py: f32) -> bool {
    if py < y || py > y + h {
        return false;
    }
    let frac = (py - y) / h;
    let left = x + skew * (1.0 - frac);
    let right = x + w - skew * frac;
    px >= left && px <= right
}

impl TagChips<'_> {
    fn layout(&self) -> (Vec<(f32, f32, f32)>, f32) {
        tag_cloud_chip_layout(&self.entries, self.width, self.scale, rtl())
    }

    fn eff_scroll(&self, total: f32) -> f32 {
        self.scroll.clamp(0.0, (total - self.max_h).max(0.0))
    }
}

fn tag_cloud_chip_layout(
    entries: &[crate::domain::library::search::TagEntry],
    width: f32,
    scale: f32,
    rtl: bool,
) -> (Vec<(f32, f32, f32)>, f32) {
    let h = TAG_CHIP_H * scale;
    let size = TAG_CHIP_TEXT * scale;
    let gap_x = TAG_CHIP_GAP_X * scale;
    let gap_y = TAG_CHIP_GAP_Y * scale;
    let mut rects = Vec::with_capacity(entries.len());
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    for entry in entries {
        let w = text_width(&chip_label(entry), size, false)
            + text_width(&chip_count_label(entry), size * 0.86, false)
            + 6.0 * scale
            + 22.0 * scale;
        if x > 0.0 && x + w > width.max(w) {
            x = 0.0;
            y += h + gap_y;
        }
        rects.push((mirror_x_for(rtl, x, w, width), y, w));
        x += w + gap_x;
    }
    let total = if rects.is_empty() { 0.0 } else { y + h };
    (rects, total)
}

pub fn tag_cloud_row_count(
    entries: &[crate::domain::library::search::TagEntry],
    width: f32,
    scale: f32,
) -> usize {
    let (_, total) = tag_cloud_chip_layout(entries, width, scale, false);
    if total <= 0.0 {
        return 1;
    }
    ((total + TAG_CHIP_GAP_Y * scale) / ((TAG_CHIP_H + TAG_CHIP_GAP_Y) * scale)).round().max(1.0)
        as usize
}

pub fn tag_cloud_body_height(rows: usize, scale: f32) -> f32 {
    let rows = rows.clamp(1, 3);
    rows as f32 * TAG_CHIP_H * scale + rows.saturating_sub(1) as f32 * TAG_CHIP_GAP_Y * scale
}

impl canvas::Program<crate::frontend::tagcloud::TagIntent> for TagChips<'_> {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<crate::frontend::tagcloud::TagIntent>> {
        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
            cursor.position_in(bounds)?;
            let step = match crate::frontend::components::norm_wheel(delta)? {
                mouse::ScrollDelta::Lines { y, .. } => y * 48.0 * self.scale,
                mouse::ScrollDelta::Pixels { y, .. } => y,
            };
            let (_, total) = self.layout();
            if total <= self.max_h {
                return Some(Action::capture());
            }
            let next =
                (self.scroll_target.min(total - self.max_h) - step).clamp(0.0, total - self.max_h);
            return Some(
                Action::publish(crate::frontend::tagcloud::TagIntent::Update(
                    crate::frontend::tagcloud::TagMsg::CloudScroll(next),
                ))
                .and_capture(),
            );
        }
        if let Event::Mouse(mouse::Event::ButtonPressed(btn)) = event {
            let right = *btn == mouse::Button::Right;
            if !right && *btn != mouse::Button::Left {
                return None;
            }
            let pos = cursor.position_in(bounds)?;
            let scale = self.scale;
            let h = TAG_CHIP_H * scale;
            let (rects, total) = self.layout();
            let off = self.eff_scroll(total);
            for (entry, &(x, y, w)) in self.entries.iter().zip(&rects) {
                if chip_contains(x, y - off, w, h, 0.0, pos.x, pos.y) {
                    return Some(
                        Action::publish(crate::frontend::tagcloud::TagIntent::Update(
                            crate::frontend::tagcloud::TagMsg::CloudClick(entry.tag.clone(), right),
                        ))
                        .and_capture(),
                    );
                }
            }
        }
        None
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let scale = self.scale;
        let h = TAG_CHIP_H * scale;
        let size = TAG_CHIP_TEXT * scale;
        let count_size = size * 0.86;
        let pal = self.pal;
        let excl = pal.destructive();
        let (rects, total) = self.layout();
        let off = self.eff_scroll(total);
        let mirrored = rtl();
        let ent = self.entrance.clamp(0.0, 1.0);
        let num = self.entries.len().max(1) as f32;
        for (idx, (entry, &(x, y, w))) in self.entries.iter().zip(&rects).enumerate() {
            let y = y - off;
            if y + h < 0.0 || y > self.max_h {
                continue;
            }
            let ease =
                if ent >= 0.999 { 1.0 } else { smoothstep((ent - (idx as f32 / num) * 0.4) / 0.6) };
            let fade = move |color: Color| with_alpha(color, color.a * ease);
            let yy = y + (1.0 - ease) * 8.0 * scale;
            let path = canvas::Path::rectangle(Point::new(x, yy), Size::new(w, h));
            let hovered = cursor
                .position_in(bounds)
                .is_some_and(|position| chip_contains(x, yy, w, h, 0.0, position.x, position.y));
            let (fill, stroke, txt) = if entry.excluded {
                (with_alpha(excl, 0.16), with_alpha(excl, 0.85), excl)
            } else if entry.selected {
                (
                    control_background(pal, true, false),
                    control_border(pal, true, false),
                    control_text(pal, true, hovered),
                )
            } else if hovered {
                (
                    control_background(pal, false, true),
                    control_border(pal, false, true),
                    control_text(pal, false, true),
                )
            } else {
                (
                    control_background(pal, false, false),
                    control_border(pal, false, false),
                    control_text(pal, false, false),
                )
            };
            frame.fill(&path, fade(fill));
            frame.stroke(&path, Stroke::default().with_color(fade(stroke)).with_width(1.0));
            let name = chip_label(entry);
            let count = chip_count_label(entry);
            let name_w = text_width(&name, size, false);
            let count_w = text_width(&count, count_size, false);
            let gap = 6.0 * scale;
            let start_x = x + w / 2.0 - (name_w + gap + count_w) / 2.0;
            let (name_x, count_x) = if mirrored {
                (start_x + count_w + gap, start_x)
            } else {
                (start_x, start_x + name_w + gap)
            };
            let cy = yy + h / 2.0;
            frame.fill_text(mid_text(
                name,
                Point::new(name_x, cy),
                fade(txt),
                size,
                UI_FONT,
                Alignment::Start,
            ));
            frame.fill_text(mid_text(
                count,
                Point::new(count_x, cy),
                fade(with_alpha(txt, 0.5)),
                count_size,
                UI_FONT,
                Alignment::Start,
            ));
        }
        vec![frame.into_geometry()]
    }
}

mod tests;
