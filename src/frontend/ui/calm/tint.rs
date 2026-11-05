#[cfg(test)]
mod tests;

use iced::Color;

pub const ROW_TINT: f32 = 0.06;
pub const SELECTED_TINT: f32 = 0.18;

pub fn tint(plane: Color, ink: Color, alpha: f32) -> Color {
    let amount = (alpha * ink.a).clamp(0.0, 1.0);
    let mix = |from: f32, to: f32| from + (to - from) * amount;
    Color { r: mix(plane.r, ink.r), g: mix(plane.g, ink.g), b: mix(plane.b, ink.b), a: plane.a }
}
