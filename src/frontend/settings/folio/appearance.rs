#[cfg(test)]
mod tests;

use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::{Color, Point, Rectangle};

use crate::app::Message;
use crate::frontend::components::with_alpha;
use crate::frontend::settings::{SETTINGS_MOTION, place};
use crate::frontend::theme::Palette;

pub(super) struct Metrics {
    pub width: f32,
    pub height: f32,
    pub margin: f32,
    pub index: f32,
    pub emblem: f32,
    pub muted: f32,
    pub atmosphere: f32,
    pub sheet_rule: f32,
    pub index_rule: f32,
    pub editor: f32,
    pub row_gap: f32,
}

pub(super) const METRICS: Metrics = Metrics {
    width: 1_344.0,
    height: 746.0,
    margin: 22.0,
    index: 318.0,
    emblem: 46.0,
    muted: 0.64,
    atmosphere: 0.045,
    sheet_rule: 0.58,
    index_rule: 0.46,
    editor: super::cut_field::WIDTH,
    row_gap: 25.0,
};

pub(super) struct IndexMetrics {
    pub padding: [f32; 4],
    pub gap: f32,
    pub head_gap: f32,
    pub title: f32,
    pub note: f32,
    pub note_leading: f32,
    pub note_alpha: f32,
    pub row_padding: f32,
    pub row_gap: f32,
    pub row_size: f32,
    pub glyph: f32,
    pub glyph_width: f32,
    pub search_glyph: f32,
    pub search_size: f32,
    pub shortcut: f32,
    pub shortcut_alpha: f32,
    pub badge: f32,
    pub rail: f32,
    pub rail_gap: f32,
    pub fill: f32,
    pub travel: f32,
}

pub(super) const INDEX: IndexMetrics = IndexMetrics {
    padding: [25.0, 20.0, 20.0, 22.0],
    gap: 17.0,
    head_gap: 7.0,
    title: 20.0,
    note: 11.0,
    note_leading: 1.4,
    note_alpha: 0.56,
    row_padding: 8.0,
    row_gap: 9.0,
    row_size: 12.5,
    glyph: 10.0,
    glyph_width: 10.0,
    search_glyph: 15.0,
    search_size: 11.5,
    shortcut: 9.0,
    shortcut_alpha: 0.42,
    badge: 10.5,
    rail: 14.0,
    rail_gap: 9.0,
    fill: 0.9,
    travel: 6.0,
};

const EMBLEM_RADII: [f32; 3] = [23.0, 17.9, 13.4];
pub(super) const EMBLEM_ICON_SIZE: f32 = 13.0;
const SURFACE_ALPHA: f32 = 0.99;

pub(super) fn sheet_scale(viewport: (f32, f32), requested: f32) -> f32 {
    requested
        .min((viewport.0 - METRICS.margin) / METRICS.width)
        .min((viewport.1 - METRICS.margin) / METRICS.height)
}

pub(super) fn sheet_size(viewport: (f32, f32), scale: f32) -> (f32, f32) {
    (
        (METRICS.width * scale).min(viewport.0 - METRICS.margin),
        (METRICS.height * scale).min(viewport.1 - METRICS.margin),
    )
}

pub(super) fn surface(palette: &Palette, fade: f32) -> iced::widget::container::Style {
    crate::frontend::ui::bg_style(iced::Background::Color(with_alpha(
        palette.surface,
        SURFACE_ALPHA * fade,
    )))
}

const PLACE_ICONS: [(&str, usize); crate::frontend::settings::PLACE_COUNT] = [
    (place::HOME, 8),
    (place::LAYOUT, 2),
    (place::BAR, 5),
    (place::WALLPAPERS, 3),
    (place::COLOURS, 6),
    (place::LIBRARY, 0),
    (place::SOURCES, 1),
    (place::AUTOMATION, 7),
    (place::KEYBINDS, 9),
    (place::SYSTEM, 9),
];

pub(super) fn place_icon(key: &str) -> usize {
    PLACE_ICONS.iter().find(|(place, _)| *place == key).map_or(0, |(_, icon)| *icon)
}

pub(super) enum Ornament {
    Icon { index: usize, color: Color },
    Frame { palette: Palette, progress: f32, index_width: f32 },
    Emblem { palette: Palette, elapsed_ms: f32 },
}

impl canvas::Program<Message> for Ornament {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        match self {
            Self::Icon { index, color } => icon_trace(&mut frame, bounds, *index, *color),
            Self::Frame { palette, progress, index_width } => {
                frame_trace(&mut frame, bounds, palette, *progress, *index_width);
            }
            Self::Emblem { palette, elapsed_ms } => {
                emblem_trace(&mut frame, bounds, palette, *elapsed_ms);
            }
        }
        vec![frame.into_geometry()]
    }
}

fn frame_trace(
    frame: &mut Frame,
    bounds: Rectangle,
    palette: &Palette,
    progress: f32,
    index_width: f32,
) {
    let end = Point::new(bounds.width - 1.0, bounds.height - 1.0);
    let center = Point::new(bounds.width * 0.5, 0.5);
    let bottom = Point::new(center.x, end.y);
    let left = [center, Point::new(0.5, 0.5), Point::new(0.5, end.y), bottom];
    let right = [center, Point::new(end.x, 0.5), end, bottom];
    let rule = with_alpha(palette.outline, METRICS.sheet_rule);
    trace(frame, &left, progress, rule);
    trace(frame, &right, progress, rule);
    let divider = divider(crate::frontend::ui::rtl(), index_width, bounds.width, end.y);
    trace(frame, &divider, progress, with_alpha(palette.outline, METRICS.index_rule));
}

fn divider(rtl: bool, index_width: f32, width: f32, bottom: f32) -> [Point; 2] {
    let x = crate::frontend::ui::mirror_x_for(rtl, index_width, 0.0, width);
    [Point::new(x, 0.0), Point::new(x, bottom)]
}

fn emblem_trace(frame: &mut Frame, bounds: Rectangle, palette: &Palette, elapsed_ms: f32) {
    let center = Point::new(bounds.width * 0.5, bounds.height * 0.5);
    let scale = bounds.width / METRICS.emblem;
    for (index, radius) in EMBLEM_RADII.into_iter().enumerate() {
        let progress = SETTINGS_MOTION
            .stroke
            .at(((elapsed_ms - index as f32 * SETTINGS_MOTION.emblem_stagger_ms)
                / SETTINGS_MOTION.content_ms)
                .clamp(0.0, 1.0));
        let radius = radius * scale;
        let points = [
            Point::new(center.x, center.y - radius),
            Point::new(center.x + radius, center.y),
            Point::new(center.x, center.y + radius),
            Point::new(center.x - radius, center.y),
            Point::new(center.x, center.y - radius),
        ];
        let color = if index == 1 {
            palette.primary
        } else {
            with_alpha(palette.surface_text, METRICS.muted)
        };
        trace(frame, &points, progress, color);
    }
}

fn trace(frame: &mut Frame, points: &[Point], progress: f32, color: Color) {
    let length: f32 = points.windows(2).map(|edge| edge[0].distance(edge[1])).sum();
    let mut remaining = length * progress.clamp(0.0, 1.0);
    for edge in points.windows(2) {
        let distance = edge[0].distance(edge[1]);
        if remaining <= 0.0 {
            break;
        }
        let amount = (remaining / distance.max(f32::EPSILON)).min(1.0);
        let end = edge[0] + (edge[1] - edge[0]) * amount;
        frame.stroke(&Path::line(edge[0], end), Stroke::default().with_color(color));
        remaining -= distance;
    }
}

const NAV_PATHS: &[&[&[[f32; 2]]]] = &[
    &[&[[1.0, 3.0], [6.0, 3.0], [8.0, 5.0], [15.0, 5.0], [15.0, 13.0], [1.0, 13.0], [1.0, 3.0]]],
    &[
        &[[1.0, 5.0], [4.0, 3.0], [8.0, 2.0], [12.0, 3.0], [15.0, 5.0]],
        &[[3.0, 8.0], [5.0, 6.0], [8.0, 5.0], [11.0, 6.0], [13.0, 8.0]],
        &[[6.0, 11.0], [8.0, 9.0], [10.0, 11.0]],
        &[[8.0, 13.0], [8.0, 14.0]],
    ],
    &[
        &[[2.0, 2.0], [6.0, 2.0], [6.0, 6.0], [2.0, 6.0], [2.0, 2.0]],
        &[[10.0, 2.0], [14.0, 2.0], [14.0, 6.0], [10.0, 6.0], [10.0, 2.0]],
        &[[2.0, 10.0], [6.0, 10.0], [6.0, 14.0], [2.0, 14.0], [2.0, 10.0]],
        &[[10.0, 10.0], [14.0, 10.0], [14.0, 14.0], [10.0, 14.0], [10.0, 10.0]],
    ],
    &[
        &[[1.0, 2.0], [15.0, 2.0], [15.0, 11.0], [1.0, 11.0], [1.0, 2.0]],
        &[[8.0, 11.0], [8.0, 15.0]],
        &[[5.0, 15.0], [11.0, 15.0]],
    ],
    &[
        &[[6.0, 12.0], [6.0, 3.0], [14.0, 1.0], [14.0, 10.0]],
        &[[6.0, 6.0], [14.0, 4.0]],
        &[
            [6.0, 12.0],
            [5.0, 10.0],
            [2.0, 10.0],
            [1.0, 12.0],
            [2.0, 14.0],
            [5.0, 14.0],
            [6.0, 12.0],
        ],
        &[
            [14.0, 10.0],
            [13.0, 8.0],
            [10.0, 8.0],
            [9.0, 10.0],
            [10.0, 12.0],
            [13.0, 12.0],
            [14.0, 10.0],
        ],
    ],
    &[
        &[[2.0, 2.0], [14.0, 2.0], [14.0, 7.0], [2.0, 7.0], [2.0, 2.0]],
        &[[2.0, 9.0], [14.0, 9.0], [14.0, 14.0], [2.0, 14.0], [2.0, 9.0]],
    ],
    &[
        &[[1.0, 4.0], [5.0, 4.0]],
        &[[9.0, 4.0], [15.0, 4.0]],
        &[[1.0, 12.0], [9.0, 12.0]],
        &[[13.0, 12.0], [15.0, 12.0]],
        &[[5.0, 2.0], [9.0, 2.0], [9.0, 6.0], [5.0, 6.0], [5.0, 2.0]],
        &[[9.0, 10.0], [13.0, 10.0], [13.0, 14.0], [9.0, 14.0], [9.0, 10.0]],
    ],
    &[
        &[[1.0, 3.0], [15.0, 3.0], [15.0, 15.0], [1.0, 15.0], [1.0, 3.0]],
        &[[1.0, 7.0], [15.0, 7.0]],
        &[[5.0, 1.0], [5.0, 5.0]],
        &[[11.0, 1.0], [11.0, 5.0]],
    ],
    &[
        &[[1.0, 2.0], [15.0, 2.0], [15.0, 6.0], [1.0, 6.0], [1.0, 2.0]],
        &[[1.0, 10.0], [15.0, 10.0]],
        &[[1.0, 14.0], [11.0, 14.0]],
    ],
    &[
        &[[1.0, 4.0], [5.0, 4.0]],
        &[[9.0, 4.0], [15.0, 4.0]],
        &[[1.0, 12.0], [9.0, 12.0]],
        &[[13.0, 12.0], [15.0, 12.0]],
        &[[5.0, 2.0], [9.0, 2.0], [9.0, 6.0], [5.0, 6.0], [5.0, 2.0]],
        &[[9.0, 10.0], [13.0, 10.0], [13.0, 14.0], [9.0, 14.0], [9.0, 10.0]],
    ],
];

pub(super) fn navigation_icon(
    index: usize,
    size: f32,
    color: Color,
) -> iced::Element<'static, Message> {
    iced::widget::canvas(Ornament::Icon { index, color })
        .width(iced::Length::Fixed(size))
        .height(iced::Length::Fixed(size))
        .into()
}

fn icon_trace(frame: &mut Frame, bounds: Rectangle, index: usize, color: Color) {
    const ICON_GRID: f32 = 16.0;
    const ICON_STROKE: f32 = 1.2;
    let Some(paths) = NAV_PATHS.get(index) else { return };
    let scale = bounds.width / ICON_GRID;
    for points in *paths {
        let path = Path::new(|path| {
            for (index, [x, y]) in points.iter().copied().enumerate() {
                let point = Point::new(x * scale, y * scale);
                if index == 0 {
                    path.move_to(point);
                } else {
                    path.line_to(point);
                }
            }
        });
        frame.stroke(&path, Stroke::default().with_color(color).with_width(ICON_STROKE * scale));
    }
}
