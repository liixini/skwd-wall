use iced::Color;

use crate::frontend::theme::Palette;

fn ensure_bright(c: Color) -> Color {
    let lum = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
    if lum >= 0.55 {
        return c;
    }
    let t = ((0.62 - lum) / (1.0 - lum)).clamp(0.0, 0.85);
    Color { r: c.r + (1.0 - c.r) * t, g: c.g + (1.0 - c.g) * t, b: c.b + (1.0 - c.b) * t, a: c.a }
}

pub fn dark_chrome(base: &Palette) -> Palette {
    Palette {
        primary: ensure_bright(base.primary),
        primary_text: Color { r: 0.05, g: 0.06, b: 0.08, a: 1.0 },
        surface: Color { r: 0.05, g: 0.055, b: 0.07, a: 1.0 },
        surface_text: Color { r: 0.93, g: 0.94, b: 0.96, a: 1.0 },
        surface_variant: Color { r: 0.12, g: 0.13, b: 0.16, a: 1.0 },
        surface_container: Color { r: 0.09, g: 0.10, b: 0.13, a: 1.0 },
        background: Color { r: 0.03, g: 0.035, b: 0.05, a: 1.0 },
        outline: Color { r: 0.42, g: 0.44, b: 0.5, a: 1.0 },
        tertiary: ensure_bright(base.tertiary),
        secondary: ensure_bright(base.secondary),
        error: ensure_bright(base.error),
        ..*base
    }
}
