use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Alignment, Color, Point, Size};

use crate::frontend::scene::BackPanel;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{UI_FONT, mid_text, with_alpha};
use crate::i18n::tr;

use super::geometry::BackGeometry;
use super::layout::{BackLayout, back_rise};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActionStyle {
    Primary,
    Selected,
    Secondary,
    Disabled,
}

const DISABLED_ACTION_ALPHA: f32 = 0.38;

const REVEAL_STAGGER: f32 = 3.72;
const COPY_STAGGER: f32 = 3.74;
const COPY_LABELS: [&str; 2] = ["card-back-copy-image", "card-back-copy-path"];

#[expect(clippy::too_many_lines, reason = "card back action composition")]
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
            Point::new(
                layout.lead(layout.action_left, layout.action_right),
                layout.actions_label_cy + label_delta_y,
            ),
            with_alpha(palette.surface_text, 0.52 * fade * label_amount),
            8.5,
            UI_FONT,
            layout.lead_align(),
        ),
    );
    let (rule_from, rule_to) = actions_rule(layout.rtl, layout.action_left, layout.action_right);
    frame.stroke(
        &geometry.line(
            Point::new(rule_from, layout.actions_label_cy),
            Point::new(rule_to, layout.actions_label_cy),
        ),
        Stroke::default()
            .with_color(with_alpha(palette.outline, 0.25 * fade * label_amount))
            .with_width(1.0),
    );

    if let Some(overview) = layout.actions.overview {
        draw_action(
            frame,
            palette,
            geometry,
            progress,
            fade,
            overview,
            if overview_set { tr("card-back-overview-set") } else { tr("card-back-overview") },
            palette.primary,
            if overview_set { ActionStyle::Selected } else { ActionStyle::Primary },
            3.35,
        );
    }
    if let Some(effects) = layout.actions.effects {
        draw_action(
            frame,
            palette,
            geometry,
            progress,
            fade,
            effects,
            tr("card-back-effects"),
            palette.primary,
            ActionStyle::Secondary,
            3.55,
        );
    }
    if let Some(scene_properties) = layout.actions.scene_properties {
        draw_action(
            frame,
            palette,
            geometry,
            progress,
            fade,
            scene_properties,
            tr("card-back-scene-properties"),
            palette.primary,
            ActionStyle::Secondary,
            3.65,
        );
    }
    if let Some(reset) = layout.actions.reset_thumbnail {
        draw_action(
            frame,
            palette,
            geometry,
            progress,
            fade,
            reset,
            tr("card-back-reset-thumbnail"),
            palette.primary,
            ActionStyle::Secondary,
            3.7,
        );
    }
    if let Some(reveal) = layout.actions.reveal {
        draw_action(
            frame,
            palette,
            geometry,
            progress,
            fade,
            reveal,
            tr("card-back-reveal"),
            palette.primary,
            ActionStyle::Secondary,
            REVEAL_STAGGER,
        );
    }
    for (rectangle, label) in
        [(layout.actions.copy_image, COPY_LABELS[0]), (layout.actions.copy_path, COPY_LABELS[1])]
    {
        if let Some(rectangle) = rectangle {
            draw_action(
                frame,
                palette,
                geometry,
                progress,
                fade,
                rectangle,
                tr(label),
                palette.primary,
                ActionStyle::Secondary,
                COPY_STAGGER,
            );
        }
    }
    draw_action(
        frame,
        palette,
        geometry,
        progress,
        fade,
        layout.actions.playlist,
        tr("card-back-playlist"),
        palette.primary,
        ActionStyle::Secondary,
        3.75,
    );
    draw_action(
        frame,
        palette,
        geometry,
        progress,
        fade,
        layout.actions.delete,
        panel.removal.label(),
        palette.destructive(),
        if panel.removal.enabled() { ActionStyle::Secondary } else { ActionStyle::Disabled },
        3.95,
    );
}

pub(super) fn actions_rule(rtl: bool, left: f32, right: f32) -> (f32, f32) {
    if rtl { (left, right - 74.0) } else { (left + 74.0, right) }
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
    style: ActionStyle,
    slot: f32,
) {
    let (amount, delta_y) = back_rise(progress, slot);
    let alpha =
        fade * amount * if style == ActionStyle::Disabled { DISABLED_ACTION_ALPHA } else { 1.0 };
    if alpha <= 0.01 {
        return;
    }
    let rtl = geometry.rtl();
    let (x, y, width, height) = (rectangle.0, rectangle.1 + delta_y, rectangle.2, rectangle.3);
    let lead = |inset: f32| if rtl { x + width - inset } else { x + inset };
    let control = geometry.path(&Path::rectangle(Point::new(x, y), Size::new(width, height)));
    frame.fill(&control, with_alpha(palette.background, 0.58 * alpha));
    let primary = style == ActionStyle::Primary;
    let selected = style == ActionStyle::Selected;
    if selected {
        frame.fill(&control, with_alpha(accent, 0.16 * alpha));
        let check = geometry.path(&Path::new(|builder| {
            builder.move_to(Point::new(lead(10.0), y + height * 0.5));
            builder.line_to(Point::new(lead(13.0), y + height * 0.5 + 3.0));
            builder.line_to(Point::new(lead(19.0), y + height * 0.5 - 4.0));
        }));
        frame.stroke(
            &check,
            Stroke::default().with_color(with_alpha(accent, alpha)).with_width(1.8),
        );
    } else if primary {
        let slant = height * 0.72;
        let sweep = amount.mul_add(width + slant * 2.0, -slant);
        let top = (sweep - slant).clamp(0.0, width);
        let bottom = (sweep + slant).clamp(0.0, width);
        let wipe = geometry.path(&Path::new(|builder| {
            builder.move_to(Point::new(lead(0.0), y));
            builder.line_to(Point::new(lead(top), y));
            builder.line_to(Point::new(lead(bottom), y + height));
            builder.line_to(Point::new(lead(0.0), y + height));
            builder.close();
        }));
        frame.fill(&wipe, with_alpha(accent, 0.94 * fade));
    } else if style != ActionStyle::Disabled {
        let underline = geometry.path(&Path::rectangle(
            Point::new(if rtl { x + width - width * amount } else { x }, y + height - 2.0),
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
            Point::new(
                x + width * 0.5 + if selected { if rtl { -7.0 } else { 7.0 } } else { 0.0 },
                y + height * 0.5,
            ),
            with_alpha(if primary { palette.primary_text } else { palette.surface_text }, alpha),
            9.0,
            UI_FONT,
            Alignment::Center,
        ),
    );
}
