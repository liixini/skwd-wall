use iced::Color;
use skwd_palette::studio::{StudioPalette, from_theme};
use skwd_palette::{Rgb, ThemePalette};

use crate::contracts::picker::{PaletteSpec, palette_keys};
use crate::domain::theme::{Candidate, ThemeRole};

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub primary: Color,
    pub primary_text: Color,
    pub surface: Color,
    pub surface_text: Color,
    pub surface_variant: Color,
    pub surface_container: Color,
    pub background: Color,
    pub outline: Color,
    pub tertiary: Color,
    pub secondary: Color,
    pub secondary_text: Color,
    pub tertiary_text: Color,
    pub error: Color,
    pub error_text: Color,
    pub surface_variant_text: Color,
    pub shadow: Color,
    pub hover: Color,
    pub hover_text: Color,
}

pub fn rgb_color(rgb: Rgb) -> Color {
    Color::from_rgb8(rgb.0, rgb.1, rgb.2)
}

pub fn parse_hex(raw: &str) -> Option<Color> {
    hex(raw)
}

fn hex(raw: &str) -> Option<Color> {
    let digits = raw.trim().trim_start_matches('#');
    let parse = |idx: usize| u8::from_str_radix(digits.get(idx..idx + 2)?, 16).ok();
    match digits.len() {
        6 => Some(Color::from_rgb8(parse(0)?, parse(2)?, parse(4)?)),
        8 => Some(Color::from_rgba8(
            parse(2)?,
            parse(4)?,
            parse(6)?,
            u8::from_str_radix(&digits[0..2], 16).ok()? as f32 / 255.0,
        )),
        _ => None,
    }
}

impl Default for Palette {
    fn default() -> Self {
        let dark = hex("#1d100e").unwrap();
        let text = hex("#f1dedb").unwrap();
        let variant = hex("#534341").unwrap();
        let tertiary = hex("#dfc38c").unwrap();
        Self {
            primary: hex("#ffb4ab").unwrap(),
            primary_text: dark,
            surface: dark,
            surface_text: text,
            surface_variant: variant,
            surface_container: hex("#271815").unwrap(),
            background: dark,
            outline: hex("#a08c89").unwrap(),
            tertiary,
            secondary: tertiary,
            secondary_text: dark,
            tertiary_text: dark,
            error: tertiary,
            error_text: dark,
            surface_variant_text: text,
            shadow: Color::BLACK,
            hover: variant,
            hover_text: text,
        }
    }
}

use crate::frontend::components::lerp_color;

impl Palette {
    pub fn destructive(&self) -> Color {
        self.error
    }

    pub fn lerp(&self, other: &Palette, t: f32) -> Palette {
        let t = t.clamp(0.0, 1.0);
        Palette {
            primary: lerp_color(self.primary, other.primary, t),
            primary_text: lerp_color(self.primary_text, other.primary_text, t),
            surface: lerp_color(self.surface, other.surface, t),
            surface_text: lerp_color(self.surface_text, other.surface_text, t),
            surface_variant: lerp_color(self.surface_variant, other.surface_variant, t),
            surface_container: lerp_color(self.surface_container, other.surface_container, t),
            background: lerp_color(self.background, other.background, t),
            outline: lerp_color(self.outline, other.outline, t),
            tertiary: lerp_color(self.tertiary, other.tertiary, t),
            secondary: lerp_color(self.secondary, other.secondary, t),
            secondary_text: lerp_color(self.secondary_text, other.secondary_text, t),
            tertiary_text: lerp_color(self.tertiary_text, other.tertiary_text, t),
            error: lerp_color(self.error, other.error, t),
            error_text: lerp_color(self.error_text, other.error_text, t),
            surface_variant_text: lerp_color(
                self.surface_variant_text,
                other.surface_variant_text,
                t,
            ),
            shadow: lerp_color(self.shadow, other.shadow, t),
            hover: lerp_color(self.hover, other.hover, t),
            hover_text: lerp_color(self.hover_text, other.hover_text, t),
        }
    }

    pub fn from_spec(spec: &PaletteSpec) -> Self {
        let mut palette = Self::default();
        if let Some(col) = spec.primary.as_deref().and_then(hex) {
            palette.primary = col;
        }
        if let Some(col) = spec.primary_text.as_deref().and_then(hex) {
            palette.primary_text = col;
        }
        if let Some(col) = spec.surface.as_deref().and_then(hex) {
            palette.surface = col;
        }
        if let Some(col) = spec.surface_text.as_deref().and_then(hex) {
            palette.surface_text = col;
        }
        if let Some(col) = spec.surface_variant.as_deref().and_then(hex) {
            palette.surface_variant = col;
        }
        if let Some(col) = spec.surface_container.as_deref().and_then(hex) {
            palette.surface_container = col;
        }
        if let Some(col) = spec.background.as_deref().and_then(hex) {
            palette.background = col;
        }
        if let Some(col) = spec.outline.as_deref().and_then(hex) {
            palette.outline = col;
        }
        if let Some(col) = spec.tertiary.as_deref().and_then(hex) {
            palette.tertiary = col;
        }
        palette.secondary = spec.secondary.as_deref().and_then(hex).unwrap_or(palette.tertiary);
        palette.secondary_text =
            spec.secondary_text.as_deref().and_then(hex).unwrap_or(palette.primary_text);
        palette.tertiary_text =
            spec.tertiary_text.as_deref().and_then(hex).unwrap_or(palette.primary_text);
        palette.error = spec.error.as_deref().and_then(hex).unwrap_or(palette.tertiary);
        palette.error_text = spec.error_text.as_deref().and_then(hex).unwrap_or(palette.background);
        palette.surface_variant_text =
            spec.surface_variant_text.as_deref().and_then(hex).unwrap_or(palette.surface_text);
        palette.shadow = spec.shadow.as_deref().and_then(hex).unwrap_or(Color::BLACK);
        palette.hover = spec.hover.as_deref().and_then(hex).unwrap_or(palette.surface_variant);
        palette.hover_text =
            spec.hover_text.as_deref().and_then(hex).unwrap_or(palette.surface_text);
        palette
    }

    pub fn from_candidate(candidate: &Candidate) -> Self {
        let mut palette = Self::default();
        let color = |role: ThemeRole| candidate.colors[role.index()].as_str();
        if let Some(value) = hex(color(ThemeRole::Primary)) {
            palette.primary = value;
        }
        if let Some(value) = hex(color(ThemeRole::PrimaryText)) {
            palette.primary_text = value;
        }
        if let Some(value) = hex(color(ThemeRole::Surface)) {
            palette.surface = value;
        }
        if let Some(value) = hex(color(ThemeRole::SurfaceText)) {
            palette.surface_text = value;
        }
        if let Some(value) = hex(color(ThemeRole::SurfaceVariant)) {
            palette.surface_variant = value;
        }
        if let Some(value) = hex(color(ThemeRole::SurfaceContainer)) {
            palette.surface_container = value;
        }
        if let Some(value) = hex(color(ThemeRole::Background)) {
            palette.background = value;
        }
        if let Some(value) = hex(color(ThemeRole::Outline)) {
            palette.outline = value;
        }
        if let Some(value) = hex(color(ThemeRole::Tertiary)) {
            palette.tertiary = value;
        }
        let named = |key: &str| {
            skwd_palette::material::ROLE_KEYS
                .iter()
                .position(|role| *role == key)
                .and_then(|index| hex(&candidate.colors[index]))
        };
        palette.secondary = named(palette_keys::SECONDARY[0]).unwrap_or(palette.tertiary);
        palette.secondary_text =
            named(palette_keys::SECONDARY_TEXT[1]).unwrap_or(palette.primary_text);
        palette.tertiary_text =
            named(palette_keys::TERTIARY_TEXT[1]).unwrap_or(palette.primary_text);
        palette.error = named(palette_keys::ERROR[0]).unwrap_or(palette.tertiary);
        palette.error_text = named(palette_keys::ERROR_TEXT[1]).unwrap_or(palette.background);
        palette.surface_variant_text =
            named(palette_keys::SURFACE_VARIANT_TEXT[1]).unwrap_or(palette.surface_text);
        palette.shadow = named(palette_keys::SHADOW[0]).unwrap_or(Color::BLACK);
        palette.hover = candidate
            .hover
            .as_ref()
            .and_then(|hover| hex(&hover.fill))
            .unwrap_or(palette.surface_variant);
        palette.hover_text = candidate
            .hover
            .as_ref()
            .and_then(|hover| hex(&hover.text))
            .unwrap_or(palette.surface_text);
        palette
    }
}

impl Palette {
    pub fn from_studio(studio: &StudioPalette) -> Self {
        Self {
            primary: rgb_color(studio.accent),
            primary_text: rgb_color(studio.on_accent),
            surface: rgb_color(studio.base),
            surface_text: rgb_color(studio.text),
            surface_variant: rgb_color(studio.raised),
            surface_container: rgb_color(studio.panel),
            background: rgb_color(studio.base),
            outline: rgb_color(studio.quiet),
            tertiary: rgb_color(studio.third),
            secondary: rgb_color(studio.second),
            secondary_text: rgb_color(studio.on_accent),
            tertiary_text: rgb_color(studio.on_accent),
            error: rgb_color(studio.third),
            error_text: rgb_color(studio.base),
            surface_variant_text: rgb_color(studio.text),
            shadow: Color::BLACK,
            hover: rgb_color(studio.raised),
            hover_text: rgb_color(studio.text),
        }
    }

    pub fn studio(&self) -> StudioPalette {
        let rgb = |color: Color| {
            let [red, green, blue, _] = color.into_rgba8();
            Rgb(red, green, blue)
        };
        from_theme(&ThemePalette {
            primary: rgb(self.primary),
            on_primary: rgb(self.primary_text),
            surface: rgb(self.surface_container),
            on_surface: rgb(self.surface_text),
            surface_variant: rgb(self.surface_variant),
            surface_container: rgb(self.surface_container),
            background: rgb(self.surface),
            outline: rgb(self.outline),
            tertiary: rgb(self.tertiary),
        })
    }
}
