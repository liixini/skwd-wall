use iced::widget::canvas::Frame;

use crate::frontend::animation::smoothstep;
use crate::frontend::scene::{BackPanel, card_flip_phases};
use crate::frontend::theme::Palette;

use super::actions::draw_actions;
use super::background::draw_surface;
use super::content::{draw_add, draw_fields, draw_tags, draw_toggles};
use super::layout::back_layout;

pub(crate) fn draw_back(
    frame: &mut Frame,
    panel: &BackPanel,
    palette: &Palette,
    overview_set: bool,
    global_fade: f32,
) {
    let (surface_fade, progress) = if panel.coordinated_flip {
        let phases =
            card_flip_phases(panel.progress, panel.animate_flip_shader, panel.animate_flip_back);
        (phases.surface, phases.content)
    } else {
        (
            smoothstep(((panel.progress - 0.46) / 0.3).clamp(0.0, 1.0)),
            ((panel.progress - 0.56) / 0.44).clamp(0.0, 1.0),
        )
    };
    let fade = smoothstep(progress) * global_fade;
    let surface_fade = surface_fade * global_fade;
    if surface_fade <= 0.01 {
        return;
    }
    let layout = back_layout(panel);
    draw_surface(frame, palette, panel, &layout, surface_fade);
    if fade <= 0.01 {
        return;
    }
    draw_toggles(frame, palette, panel, &layout, progress, fade);
    draw_fields(frame, palette, panel, &layout, progress, fade);
    draw_tags(frame, palette, panel, &layout, progress, fade);
    draw_add(frame, palette, panel, &layout, progress, fade);
    draw_actions(frame, palette, panel, &layout, progress, fade, overview_set);
}

#[cfg(test)]
mod tests;
