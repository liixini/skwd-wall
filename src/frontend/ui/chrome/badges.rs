use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Alignment, Color, Font, Point};

use crate::frontend::scene::Chrome;
use crate::frontend::theme::Palette;
use crate::i18n::tr;

use super::super::bar::{ICON_FAV, ICON_HIDDEN};
use super::super::misc::{NERD_FONT, UI_FONT, mid_text};
use super::super::{mirror_x_for, parallelogram, with_alpha};

fn badge_label(kind: u8) -> &'static str {
    match kind % 3 {
        1 => tr("filter-bar-type-vid"),
        2 => tr("filter-bar-type-we"),
        _ => tr("filter-bar-type-pic"),
    }
}

fn badge_width(label: &str, text_size: f32, height: f32) -> f32 {
    label.chars().count() as f32 * text_size * 0.7 + height
}

pub(super) fn slice_corners(rtl: bool, chrome: &Chrome, badge_width: f32) -> (f32, f32) {
    let cut = chrome.skew.abs();
    let right = (chrome.skew >= 0.0) != rtl;
    let indicator_x = match (right, chrome.skew >= 0.0) {
        (true, true) => chrome.cx + chrome.hw - 21.0,
        (true, false) => chrome.cx + chrome.hw - cut - 21.0,
        (false, true) => chrome.cx - chrome.hw + cut + 21.0,
        (false, false) => chrome.cx - chrome.hw + 21.0,
    };
    let badge_x = match (right, chrome.skew >= 0.0) {
        (true, true) => chrome.cx + chrome.hw - badge_width - cut - 8.0,
        (true, false) => chrome.cx + chrome.hw - badge_width - 8.0,
        (false, true) => chrome.cx - chrome.hw + 8.0,
        (false, false) => chrome.cx - chrome.hw + cut + 8.0,
    };
    (indicator_x, badge_x)
}

pub(super) fn grid_corners(rtl: bool, chrome: &Chrome, badge_width: f32) -> (f32, f32, f32) {
    let extent = chrome.cx * 2.0;
    (
        mirror_x_for(rtl, chrome.cx - chrome.hw + 4.0, badge_width, extent),
        mirror_x_for(rtl, chrome.cx - chrome.hw + 13.0, 0.0, extent),
        mirror_x_for(rtl, chrome.cx + chrome.hw - 11.0, 0.0, extent),
    )
}

pub(super) fn hex_indicator_x(rtl: bool, chrome: &Chrome) -> f32 {
    mirror_x_for(rtl, chrome.cx + chrome.hw * 0.5 - 14.0, 0.0, chrome.cx * 2.0)
}

fn badge_text(
    frame: &mut Frame,
    content: &str,
    center_x: f32,
    center_y: f32,
    size: f32,
    color: Color,
) {
    frame.fill_text(mid_text(
        content.to_string(),
        Point::new(center_x, center_y),
        color,
        size,
        Font { weight: iced::font::Weight::Bold, ..UI_FONT },
        Alignment::Center,
    ));
}

fn video_indicator(
    frame: &mut Frame,
    palette: &Palette,
    center_x: f32,
    center_y: f32,
    diameter: f32,
    opacity: f32,
) {
    let circle = Path::circle(Point::new(center_x, center_y), diameter / 2.0);
    frame.fill(&circle, with_alpha(palette.surface, 0.92 * opacity));
    frame.stroke(
        &circle,
        Stroke::default().with_color(with_alpha(palette.primary, 0.6 * opacity)).with_width(1.0),
    );
    badge_text(
        frame,
        "\u{25b6}",
        center_x + 1.0,
        center_y,
        diameter * 0.41,
        with_alpha(palette.primary, opacity),
    );
}

fn type_badge(
    frame: &mut Frame,
    palette: &Palette,
    kind: u8,
    x: f32,
    y: f32,
    height: f32,
    skew: f32,
    text_size: f32,
    background_alpha: f32,
    opacity: f32,
) {
    let label = badge_label(kind);
    let width = badge_width(label, text_size, height);
    let background = parallelogram(x, y, width, height, skew.min(height * 0.4));
    frame.fill(
        &background,
        with_alpha(palette.surface_container, background_alpha.max(0.82) * opacity),
    );
    frame.stroke(
        &background,
        Stroke::default().with_color(with_alpha(palette.outline, 0.58 * opacity)).with_width(1.0),
    );
    badge_text(
        frame,
        label,
        x + width / 2.0,
        y + height / 2.0,
        text_size,
        with_alpha(palette.surface_text, opacity),
    );
}

pub(super) fn draw_chrome_item(
    frame: &mut Frame,
    palette: &Palette,
    chrome: &Chrome,
    show_type_badge: bool,
    show_video_indicator: bool,
    rtl: bool,
) {
    match chrome.view {
        0 => draw_slice(frame, palette, chrome, show_type_badge, show_video_indicator, rtl),
        1 => draw_grid(frame, palette, chrome, show_type_badge, show_video_indicator, rtl),
        _ => draw_hex(frame, palette, chrome, show_type_badge, show_video_indicator, rtl),
    }
}

fn draw_slice(
    frame: &mut Frame,
    palette: &Palette,
    chrome: &Chrome,
    show_type_badge: bool,
    show_video_indicator: bool,
    rtl: bool,
) {
    let height = 16.0;
    let width = badge_width(badge_label(chrome.kind), 9.0, height);
    let (indicator_x, badge_x) = slice_corners(rtl, chrome, width);
    if show_video_indicator && chrome.has_video {
        let x = indicator_x;
        let edge_y = -chrome.edge_tilt * 0.5 * ((x - chrome.cx) / chrome.hw.max(1.0));
        video_indicator(
            frame,
            palette,
            x,
            chrome.cy - chrome.hh + 21.0 + edge_y,
            22.0,
            chrome.opacity,
        );
    }
    if show_type_badge {
        let x = badge_x;
        let edge_y = -chrome.edge_tilt * 0.5 * ((x + width * 0.5 - chrome.cx) / chrome.hw.max(1.0));
        type_badge(
            frame,
            palette,
            chrome.kind,
            x,
            chrome.cy + chrome.hh - height - 8.0 + edge_y,
            height,
            (height / 2.0).min(chrome.radius * 0.5).max(2.0),
            9.0,
            0.75,
            chrome.opacity,
        );
    }
}

fn draw_grid(
    frame: &mut Frame,
    palette: &Palette,
    chrome: &Chrome,
    show_type_badge: bool,
    show_video_indicator: bool,
    rtl: bool,
) {
    let (badge_x, indicator_x, fav_x) =
        grid_corners(rtl, chrome, badge_width(badge_label(chrome.kind), 8.0, 14.0));
    if show_type_badge {
        type_badge(
            frame,
            palette,
            chrome.kind,
            badge_x,
            chrome.cy + chrome.hh - 18.0,
            14.0,
            3.0,
            8.0,
            0.6,
            chrome.opacity,
        );
    }
    if show_video_indicator && chrome.has_video {
        video_indicator(
            frame,
            palette,
            indicator_x,
            chrome.cy - chrome.hh + 13.0,
            18.0,
            chrome.opacity,
        );
    }
    let glyph_y = chrome.cy - chrome.hh + CORNER_GLYPH_Y;
    if chrome.favourite {
        frame.fill_text(mid_text(
            ICON_FAV.to_string(),
            Point::new(fav_x, glyph_y),
            with_alpha(palette.primary, chrome.opacity),
            CORNER_GLYPH_SIZE,
            NERD_FONT,
            Alignment::Center,
        ));
    }
    if chrome.hidden {
        let step =
            if chrome.favourite { HIDDEN_BADGE_STEP * (badge_x - fav_x).signum() } else { 0.0 };
        frame.fill_text(mid_text(
            ICON_HIDDEN.to_string(),
            Point::new(fav_x + step, glyph_y),
            with_alpha(palette.surface_text, HIDDEN_BADGE_ALPHA * chrome.opacity),
            CORNER_GLYPH_SIZE,
            NERD_FONT,
            Alignment::Center,
        ));
    }
}

const CORNER_GLYPH_Y: f32 = 11.0;
const CORNER_GLYPH_SIZE: f32 = 14.0;
const HIDDEN_BADGE_STEP: f32 = 18.0;
const HIDDEN_BADGE_ALPHA: f32 = 0.75;

fn draw_hex(
    frame: &mut Frame,
    palette: &Palette,
    chrome: &Chrome,
    show_type_badge: bool,
    show_video_indicator: bool,
    rtl: bool,
) {
    if show_type_badge {
        let height = 18.0;
        let label = badge_label(chrome.kind);
        let width = badge_width(label, 9.0, 14.0);
        type_badge(
            frame,
            palette,
            chrome.kind,
            chrome.cx - width / 2.0,
            chrome.cy + chrome.hh - height - chrome.hw * 0.18,
            height,
            9.0,
            9.0,
            0.75,
            chrome.opacity,
        );
    }
    if show_video_indicator && chrome.has_video {
        video_indicator(
            frame,
            palette,
            hex_indicator_x(rtl, chrome),
            chrome.cy - chrome.hh * 0.866 + 18.0,
            20.0,
            chrome.opacity,
        );
    }
}
