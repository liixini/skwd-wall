use iced::advanced::text::Alignment as TextAlignment;
use iced::widget::canvas::Text;
use iced::{Alignment, Color, Font, Point, alignment};

pub const NERD_FONT: Font = Font::with_name("Symbols Nerd Font");

pub const UI_FONT_FAMILY: &str = "Roboto Condensed";

pub const UI_FONT: Font = Font {
    family: iced::font::Family::Name(UI_FONT_FAMILY),
    weight: iced::font::Weight::Bold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub const TYPE_SMALL: f32 = 11.0;

pub fn legible_type_scale(scale: f32) -> f32 {
    scale.max(0.95)
}

pub fn label<'a>(
    content: impl iced::widget::text::IntoFragment<'a>,
    size: f32,
    scale: f32,
    color: Color,
) -> iced::widget::Text<'a> {
    iced::widget::text(content).font(UI_FONT).size(size * legible_type_scale(scale)).color(color)
}

pub fn sentence_case(label: &str) -> String {
    let mut value = label.to_ascii_lowercase();
    if let Some(first) = value.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    value
}

pub(crate) fn glyph_width(size: f32) -> f32 {
    size * 1.1
}

pub(crate) fn text_width(label: &str, size: f32, nerd: bool) -> f32 {
    let count = label.chars().count() as f32;
    if nerd { count * glyph_width(size) } else { count * size * 0.64 }
}

pub(crate) fn ellipsize_text(label: &str, size: f32, max_width: f32) -> String {
    if text_width(label, size, false) <= max_width {
        return label.to_owned();
    }
    let ellipsis = '…';
    let char_width = size * 0.64;
    let keep = ((max_width / char_width).floor() as usize).saturating_sub(1);
    let mut fitted: String = label.chars().take(keep).collect();
    fitted.push(ellipsis);
    fitted
}

pub fn mid_text(
    content: impl Into<String>,
    position: Point,
    color: Color,
    size: f32,
    font: Font,
    align_x: Alignment,
) -> Text {
    let content = content.into();
    let (position, align_x) = anchor_rtl(&content, position, align_x.into(), size, font);
    Text {
        content,
        position,
        color,
        size: size.into(),
        font,
        align_x,
        align_y: alignment::Vertical::Center,
        ..Text::default()
    }
}

pub(crate) fn shapes_across(text: &str) -> bool {
    has_rtl(text) || text.chars().any(|ch| matches!(ch, '\u{0900}'..='\u{0DFF}'))
}

pub(crate) fn has_rtl(text: &str) -> bool {
    text.chars().any(|ch| {
        matches!(ch, '\u{0590}'..='\u{08FF}' | '\u{FB1D}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}')
    })
}

pub(crate) fn anchor_rtl(
    content: &str,
    position: Point,
    align_x: TextAlignment,
    size: f32,
    font: Font,
) -> (Point, TextAlignment) {
    if align_x == TextAlignment::Default || content.contains('\n') || !has_rtl(content) {
        return (position, align_x);
    }
    let width = measured_line_width(content, size, font);
    let x = match align_x {
        TextAlignment::Center => position.x - width / 2.0,
        TextAlignment::Right => position.x - width,
        _ => position.x,
    };
    (Point::new(x, position.y), TextAlignment::Default)
}

pub(crate) fn measured_line_width(content: &str, size: f32, font: Font) -> f32 {
    use iced::advanced::graphics::text::{cosmic_text, font_system, to_attributes};
    let mut system = font_system().write().expect("font system poisoned");
    let mut buffer =
        cosmic_text::Buffer::new(system.raw(), cosmic_text::Metrics::new(size, size * 1.2));
    buffer.set_size(system.raw(), Some(f32::INFINITY), Some(f32::INFINITY));
    buffer.set_text(
        system.raw(),
        content,
        &to_attributes(font),
        cosmic_text::Shaping::Advanced,
        None,
    );
    buffer.layout_runs().map(|run| run.line_w).fold(0.0, f32::max)
}
