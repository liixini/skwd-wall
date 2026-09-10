use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Alignment, Color, Point, Size};

use crate::frontend::scene::BackPanel;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{UI_FONT, mid_text, with_alpha};
use crate::i18n::tr;

use super::geometry::BackGeometry;
use super::layout::{BackLayout, back_rise};

pub(super) fn draw_actions(
    frame: &mut Frame,
    palette: &Palette,
    panel: &BackPanel,
    layout: &BackLayout,
    progress: f32,
    fade: f32,
    overview_set: bool,
) {
    let geometry = BackGeometry::new(panel, layout);
    let (label_amount, label_delta_y) = back_rise(progress, 3.0);
    geometry.fill_text(
        frame,
        mid_text(
            tr("card-back-actions-label").to_string(),
            Point::new(layout.action_left, layout.actions_label_cy + label_delta_y),
            with_alpha(palette.surface_text, 0.52 * fade * label_amount),
            8.5,
            UI_FONT,
            Alignment::Start,
        ),
    );
    frame.stroke(
        &geometry.line(
            Point::new(layout.action_left + 74.0, layout.actions_label_cy),
            Point::new(layout.action_right, layout.actions_label_cy),
        ),
        Stroke::default()
            .with_color(with_alpha(palette.outline, 0.25 * fade * label_amount))
            .with_width(1.0),
    );

    if let Some(overview) = layout.overview {
        draw_action(
            frame,
            palette,
            geometry,
            progress,
            fade,
            overview,
            if overview_set { tr("card-back-overview-set") } else { tr("card-back-overview") },
            palette.primary,
            true,
            3.35,
        );
    }
    if let Some(effects) = layout.effects {
        draw_action(
            frame,
            palette,
            geometry,
            progress,
            fade,
            effects,
            tr("card-back-effects"),
            palette.primary,
            false,
            3.55,
        );
    }
    if let Some(scene_properties) = layout.scene_properties {
        draw_action(
            frame,
            palette,
            geometry,
            progress,
            fade,
            scene_properties,
            tr("card-back-scene-properties"),
            palette.primary,
            false,
            3.65,
        );
    }
    if let Some(reset) = layout.reset_thumbnail {
        draw_action(
            frame,
            palette,
            geometry,
            progress,
            fade,
            reset,
            tr("card-back-reset-thumbnail"),
            palette.primary,
            false,
            3.7,
        );
    }
    draw_action(
        frame,
        palette,
        geometry,
        progress,
        fade,
        layout.playlist,
        tr("card-back-playlist"),
        palette.primary,
        false,
        3.75,
    );
    draw_action(
        frame,
        palette,
        geometry,
        progress,
        fade,
        layout.delete,
        tr("card-back-delete"),
        palette.destructive(),
        false,
        3.95,
    );
}

fn draw_action(
    frame: &mut Frame,
    palette: &Palette,
    geometry: BackGeometry,
    progress: f32,
    fade: f32,
    rectangle: (f32, f32, f32, f32),
    label: &str,
    accent: Color,
    primary: bool,
    slot: f32,
) {
    let (amount, delta_y) = back_rise(progress, slot);
    let alpha = fade * amount;
    if alpha <= 0.01 {
        return;
    }
    let (x, y, width, height) = (rectangle.0, rectangle.1 + delta_y, rectangle.2, rectangle.3);
    let control = geometry.path(&Path::rectangle(Point::new(x, y), Size::new(width, height)));
    frame.fill(&control, with_alpha(palette.background, 0.58 * alpha));
    if primary {
        let slant = height * 0.72;
        let sweep = amount.mul_add(width + slant * 2.0, -slant);
        let top = (sweep - slant).clamp(0.0, width);
        let bottom = (sweep + slant).clamp(0.0, width);
        let wipe = geometry.path(&Path::new(|builder| {
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(x + top, y));
            builder.line_to(Point::new(x + bottom, y + height));
            builder.line_to(Point::new(x, y + height));
            builder.close();
        }));
        frame.fill(&wipe, with_alpha(accent, 0.94 * fade));
    } else {
        let underline = geometry.path(&Path::rectangle(
            Point::new(x, y + height - 2.0),
            Size::new(width * amount, 2.0),
        ));
        frame.fill(&underline, with_alpha(accent, 0.72 * fade));
    }
    frame.stroke(
        &control,
        Stroke::default()
            .with_color(with_alpha(accent, if primary { 0.92 } else { 0.42 } * alpha))
            .with_width(1.0),
    );
    geometry.fill_text(
        frame,
        mid_text(
            label.to_string(),
            Point::new(x + width * 0.5, y + height * 0.5),
            with_alpha(if primary { palette.primary_text } else { palette.surface_text }, alpha),
            9.0,
            UI_FONT,
            Alignment::Center,
        ),
    );
}
