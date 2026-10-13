use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Alignment, Point, Size};

use crate::frontend::animation::smoothstep;
use crate::frontend::scene::BackPanel;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{UI_FONT, ellipsize_text, mid_text, with_alpha};
use crate::i18n::tr;

use super::super::super::bar::{ICON_FAV, ICON_HIDDEN};
use super::super::super::misc::NERD_FONT;
use super::geometry::BackGeometry;
use super::layout::{BackLayout, TOGGLE_INSET, back_rise};

const ICON_FAV_OUTLINE: &str = "\u{f02d5}";

struct MastheadToggle {
    at: (f32, f32, f32),
    fill: f32,
    on: &'static str,
    off: &'static str,
}

pub(super) fn draw_toggles(
    frame: &mut Frame,
    palette: &Palette,
    panel: &BackPanel,
    layout: &BackLayout,
    progress: f32,
    fade: f32,
) {
    let favourite = MastheadToggle {
        at: layout.fav,
        fill: smoothstep(panel.fav_fill),
        on: ICON_FAV,
        off: ICON_FAV_OUTLINE,
    };
    let hidden = MastheadToggle {
        at: layout.hide,
        fill: if panel.hidden { 1.0 } else { 0.0 },
        on: ICON_HIDDEN,
        off: ICON_HIDDEN,
    };
    for toggle in [favourite, hidden] {
        draw_toggle(frame, palette, panel, layout, (progress, fade), &toggle);
    }
}

fn draw_toggle(
    frame: &mut Frame,
    palette: &Palette,
    panel: &BackPanel,
    layout: &BackLayout,
    (progress, fade): (f32, f32),
    toggle: &MastheadToggle,
) {
    let geometry = BackGeometry::new(panel, layout);
    let (amount, delta_y) = back_rise(progress, 0.0);
    let (x, y, size) = (toggle.at.0, toggle.at.1 + delta_y, toggle.at.2);
    let alpha = fade * amount;
    let fill = toggle.fill;
    let button_size = layout.masthead.3 - TOGGLE_INSET;
    let button = geometry.path(&Path::rectangle(
        Point::new(x - button_size * 0.5, y - button_size * 0.5),
        Size::new(button_size, button_size),
    ));
    frame.fill(&button, with_alpha(palette.primary, (0.08 + 0.76 * fill) * alpha));
    frame.stroke(
        &button,
        Stroke::default()
            .with_color(with_alpha(palette.primary, (0.42 + 0.5 * fill) * alpha))
            .with_width(1.0),
    );
    geometry.fill_text(
        frame,
        mid_text(
            if fill > 0.5 { toggle.on } else { toggle.off }.to_string(),
            Point::new(x, y),
            with_alpha(if fill > 0.5 { palette.primary_text } else { palette.surface_text }, alpha),
            size,
            NERD_FONT,
            Alignment::Center,
        ),
    );
}

pub(super) fn draw_fields(
    frame: &mut Frame,
    palette: &Palette,
    panel: &BackPanel,
    layout: &BackLayout,
    progress: f32,
    fade: f32,
) {
    let geometry = BackGeometry::new(panel, layout);
    let kind = if panel.kind_label.is_empty() {
        tr("card-back-asset").to_string()
    } else {
        panel.kind_label.to_uppercase()
    };
    let (kicker_amount, kicker_delta_y) = back_rise(progress, 0.0);
    geometry.fill_text(
        frame,
        mid_text(
            crate::i18n::tr_args!("card-back-kicker", kind => &kind),
            Point::new(
                layout.lead(layout.content_left, layout.content_right),
                layout.kicker_cy + kicker_delta_y,
            ),
            with_alpha(palette.primary, fade * kicker_amount),
            10.0,
            UI_FONT,
            layout.lead_align(),
        ),
    );
    let (title_amount, title_delta_y) = back_rise(progress, 0.65);
    let title_left = layout.title_left;
    geometry.fill_text(
        frame,
        mid_text(
            ellipsize_text(
                &panel.title,
                layout.title_size,
                (layout.title_right - title_left).max(48.0),
            ),
            Point::new(
                layout.lead(title_left, layout.title_right),
                layout.title_cy + title_delta_y,
            ),
            with_alpha(palette.surface_text, fade * title_amount),
            layout.title_size,
            UI_FONT,
            layout.lead_align(),
        ),
    );

    for (index, ((label, value), &fact)) in panel.fields.iter().zip(layout.facts.iter()).enumerate()
    {
        let (amount, delta_y) = back_rise(progress, 1.15 + index as f32 * 0.18);
        let (x, y, height) = (layout.lead_edge(fact, 0.0), fact.1, fact.3);
        geometry.fill_text(
            frame,
            mid_text(
                label.to_uppercase(),
                Point::new(x, y + 6.0 + delta_y),
                with_alpha(palette.primary, 0.72 * fade * amount),
                8.5,
                UI_FONT,
                layout.lead_align(),
            ),
        );
        geometry.fill_text(
            frame,
            mid_text(
                value.clone(),
                Point::new(x, y + height - 7.0 + delta_y),
                with_alpha(palette.surface_text, 0.9 * fade * amount),
                12.0,
                UI_FONT,
                layout.lead_align(),
            ),
        );
    }
    let (rule_amount, _) = back_rise(progress, 1.55);
    frame.stroke(
        &geometry.line(
            Point::new(layout.content_left, layout.facts_rule_y),
            Point::new(layout.content_right, layout.facts_rule_y),
        ),
        Stroke::default()
            .with_color(with_alpha(palette.outline, 0.34 * fade * rule_amount))
            .with_width(1.0),
    );
}

pub(super) fn draw_tags(
    frame: &mut Frame,
    palette: &Palette,
    panel: &BackPanel,
    layout: &BackLayout,
    progress: f32,
    fade: f32,
) {
    let geometry = BackGeometry::new(panel, layout);
    let (label_amount, label_delta_y) = back_rise(progress, 1.9);
    geometry.fill_text(
        frame,
        mid_text(
            crate::i18n::tr_args!("card-back-tags-label", count => panel.tags.len()),
            Point::new(
                layout.lead(layout.content_left, layout.content_right),
                layout.tags_label_cy + label_delta_y,
            ),
            with_alpha(palette.surface_text, 0.52 * fade * label_amount),
            8.5,
            UI_FONT,
            layout.lead_align(),
        ),
    );
    for (index, (tag, &(x, y, width, height))) in
        panel.tags.iter().zip(layout.tags.iter()).enumerate()
    {
        let (amount, delta_y) = back_rise(progress, 2.15 + index as f32 * 0.15);
        let mut alpha = fade * amount;
        let (mut x, mut y, mut width, mut height) = (x, y + delta_y, width, height);
        if index as i32 == panel.pop_idx && panel.chip_pop < 1.0 {
            let pop = smoothstep(panel.chip_pop);
            let scale = 0.62 + 0.38 * pop;
            let (center_x, center_y) = (x + width / 2.0, y + height / 2.0);
            width *= scale;
            height *= scale;
            x = center_x - width / 2.0;
            y = center_y - height / 2.0;
            alpha *= pop;
        }
        if alpha <= 0.01 {
            continue;
        }
        let chip = geometry.path(&Path::rectangle(Point::new(x, y), Size::new(width, height)));
        frame.fill(&chip, with_alpha(palette.background, 0.5 * alpha));
        frame.stroke(
            &chip,
            Stroke::default().with_color(with_alpha(palette.outline, 0.46 * alpha)).with_width(1.0),
        );
        geometry.fill_text(
            frame,
            mid_text(
                tag.to_uppercase(),
                Point::new(layout.lead_edge((x, y, width, height), 11.0), y + height / 2.0),
                with_alpha(palette.surface_text, 0.84 * alpha),
                9.0,
                UI_FONT,
                layout.lead_align(),
            ),
        );
        geometry.fill_text(
            frame,
            mid_text(
                String::from("×"),
                Point::new(layout.trail_edge((x, y, width, height), 11.0), y + height / 2.0),
                with_alpha(palette.primary, 0.82 * alpha),
                11.0,
                UI_FONT,
                Alignment::Center,
            ),
        );
    }
    if let Some((x, y, width, height, hidden)) = layout.tag_overflow {
        let (amount, delta_y) = back_rise(progress, 2.15 + layout.tags.len() as f32 * 0.15);
        let y = y + delta_y;
        let alpha = fade * amount;
        let chip = geometry.path(&Path::rectangle(Point::new(x, y), Size::new(width, height)));
        frame.fill(&chip, with_alpha(palette.surface_variant, 0.66 * alpha));
        frame.stroke(
            &chip,
            Stroke::default().with_color(with_alpha(palette.outline, 0.34 * alpha)).with_width(1.0),
        );
        geometry.fill_text(
            frame,
            mid_text(
                crate::i18n::tr_args!("card-back-more", count => hidden),
                Point::new(x + width / 2.0, y + height / 2.0),
                with_alpha(palette.surface_text, 0.62 * alpha),
                8.5,
                UI_FONT,
                Alignment::Center,
            ),
        );
    }
}

pub(super) fn draw_add(
    frame: &mut Frame,
    palette: &Palette,
    panel: &BackPanel,
    layout: &BackLayout,
    progress: f32,
    fade: f32,
) {
    let geometry = BackGeometry::new(panel, layout);
    let (amount, delta_y) = back_rise(progress, 2.2 + panel.tags.len() as f32 * 0.15);
    let (x, y, width, height) = layout.add;
    let y = y + delta_y;
    let open = smoothstep(panel.add_open);
    let background = geometry.path(&Path::rectangle(Point::new(x, y), Size::new(width, height)));
    frame.fill(&background, with_alpha(palette.background, (0.36 + 0.38 * open) * fade * amount));
    frame.stroke(
        &background,
        Stroke::default()
            .with_color(with_alpha(palette.primary, (0.46 + 0.44 * open) * fade * amount))
            .with_width(1.0),
    );
    if open < 0.5 {
        let label_alpha = (1.0 - open * 2.0) * fade * amount;
        geometry.fill_text(
            frame,
            mid_text(
                format!("+  {}", tr("card-back-add-tag")),
                Point::new(x + width / 2.0, y + height / 2.0),
                with_alpha(palette.primary, label_alpha),
                9.0,
                UI_FONT,
                Alignment::Center,
            ),
        );
    }
}
