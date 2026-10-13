#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum ThemeRole {
    Primary,
    PrimaryText,
    Tertiary,
    Surface,
    SurfaceText,
    SurfaceVariant,
    SurfaceContainer,
    Background,
    Outline,
}

impl ThemeRole {
    pub const ALL: [Self; 9] = [
        Self::Primary,
        Self::PrimaryText,
        Self::Tertiary,
        Self::Surface,
        Self::SurfaceText,
        Self::SurfaceVariant,
        Self::SurfaceContainer,
        Self::Background,
        Self::Outline,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }
}

pub const THEME_ROLE_COUNT: usize = skwd_palette::material::ROLE_KEYS.len();
const PREVIEW_COLOR_LIMIT: usize = 6;
const PREVIEW_ROLES: [ThemeRole; ThemeRole::ALL.len()] = [
    ThemeRole::Background,
    ThemeRole::Surface,
    ThemeRole::SurfaceContainer,
    ThemeRole::SurfaceText,
    ThemeRole::Primary,
    ThemeRole::Tertiary,
    ThemeRole::SurfaceVariant,
    ThemeRole::PrimaryText,
    ThemeRole::Outline,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub colors: [String; THEME_ROLE_COUNT],
    pub alternate: [String; THEME_ROLE_COUNT],
    pub dark: bool,
}

impl Default for Candidate {
    fn default() -> Self {
        Self {
            colors: std::array::from_fn(|_| "#808080".to_string()),
            alternate: std::array::from_fn(|_| "#808080".to_string()),
            dark: true,
        }
    }
}

impl Candidate {
    pub fn preview_colors(&self) -> Vec<String> {
        let mut colors = Vec::with_capacity(PREVIEW_COLOR_LIMIT);
        let mut seen = Vec::with_capacity(PREVIEW_COLOR_LIMIT);
        for role in PREVIEW_ROLES {
            let color = &self.colors[role.index()];
            if let Some(rgb) = parse_rgb(color)
                && !seen.contains(&rgb)
            {
                seen.push(rgb);
                colors.push(color.clone());
                if colors.len() == PREVIEW_COLOR_LIMIT {
                    break;
                }
            }
        }
        colors
    }

    pub fn from_preset(name: &str) -> Option<Self> {
        skwd_palette::preset(name).map(Self::from_palette)
    }

    pub fn from_seed(hex: &str, dark: bool) -> Option<Self> {
        let (colors, alternate) = skwd_palette::material::generate_colors(hex)?;
        let mut candidate = Self { colors, alternate, dark: true };
        candidate.set_dark(dark);
        Some(candidate)
    }

    pub fn set_dark(&mut self, dark: bool) {
        if self.dark != dark {
            std::mem::swap(&mut self.colors, &mut self.alternate);
            self.dark = dark;
        }
    }

    pub fn same_colors(&self, other: &Self) -> bool {
        if self.dark == other.dark {
            self.colors == other.colors && self.alternate == other.alternate
        } else {
            self.colors == other.alternate && self.alternate == other.colors
        }
    }

    fn from_palette(palette: skwd_palette::ThemePalette) -> Self {
        let dark = palette.background.lum() < 128.0;
        let mut candidate = Self::from_seed(&palette.primary.hex(), dark).unwrap_or_default();
        candidate.colors[ThemeRole::Primary.index()] = palette.primary.hex();
        candidate.colors[ThemeRole::PrimaryText.index()] = palette.on_primary.hex();
        candidate.colors[ThemeRole::Tertiary.index()] = palette.tertiary.hex();
        candidate.colors[ThemeRole::Surface.index()] = palette.surface.hex();
        candidate.colors[ThemeRole::SurfaceText.index()] = palette.on_surface.hex();
        candidate.colors[ThemeRole::SurfaceVariant.index()] = palette.surface_variant.hex();
        candidate.colors[ThemeRole::SurfaceContainer.index()] = palette.surface_container.hex();
        candidate.colors[ThemeRole::Background.index()] = palette.background.hex();
        candidate.colors[ThemeRole::Outline.index()] = palette.outline.hex();
        candidate
    }
}

pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> (f32, f32, f32) {
    let hue = hue.rem_euclid(360.0);
    let chroma = value * saturation;
    let intermediate = chroma * (1.0 - ((hue / 60.0).rem_euclid(2.0) - 1.0).abs());
    let offset = value - chroma;
    let (red, green, blue) = match hue as u32 {
        0..=59 => (chroma, intermediate, 0.0),
        60..=119 => (intermediate, chroma, 0.0),
        120..=179 => (0.0, chroma, intermediate),
        180..=239 => (0.0, intermediate, chroma),
        240..=299 => (intermediate, 0.0, chroma),
        _ => (chroma, 0.0, intermediate),
    };
    (red + offset, green + offset, blue + offset)
}

pub fn hsv_to_hex(hue: f32, saturation: f32, value: f32) -> String {
    let (red, green, blue) = hsv_to_rgb(hue, saturation, value);
    format!(
        "#{:02x}{:02x}{:02x}",
        (red * 255.0).round() as u8,
        (green * 255.0).round() as u8,
        (blue * 255.0).round() as u8
    )
}

pub fn hex_to_hsv(hex: &str) -> Option<(f32, f32, f32)> {
    let (red, green, blue) = parse_rgb(hex)?;
    let (red, green, blue) = (red as f32 / 255.0, green as f32 / 255.0, blue as f32 / 255.0);
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let hue = if delta == 0.0 {
        0.0
    } else if max == red {
        60.0 * (((green - blue) / delta).rem_euclid(6.0))
    } else if max == green {
        60.0 * ((blue - red) / delta + 2.0)
    } else {
        60.0 * ((red - green) / delta + 4.0)
    };
    let saturation = if max == 0.0 { 0.0 } else { delta / max };
    Some((hue, saturation, max))
}

fn parse_rgb(raw: &str) -> Option<(u8, u8, u8)> {
    let digits = raw.trim().trim_start_matches('#');
    let parse = |start: usize| u8::from_str_radix(digits.get(start..start + 2)?, 16).ok();
    match digits.len() {
        6 => Some((parse(0)?, parse(2)?, parse(4)?)),
        8 => {
            parse(0)?;
            Some((parse(2)?, parse(4)?, parse(6)?))
        }
        _ => None,
    }
}
