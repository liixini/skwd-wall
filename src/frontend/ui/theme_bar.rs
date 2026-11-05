pub const THEME_BACKENDS: [(&str, &str); 15] = [
    ("native", "theme-bar-skwd-colour"),
    ("static", "theme-bar-static"),
    ("skwd-iris", "theme-bar-skwd-iris"),
    ("skwd-pywal", "theme-bar-skwd-pywal"),
    ("skwd-wallust", "theme-bar-skwd-wallust"),
    (crate::contracts::picker::STUDIO_ENGINE, "theme-bar-skwd-studio"),
    ("matugen", "theme-bar-matugen"),
    ("wallust", "theme-bar-wallust"),
    ("pywal", "theme-bar-pywal"),
    ("iris", "theme-bar-iris"),
    ("caelestia", "theme-bar-caelestia"),
    ("noctalia", "theme-bar-noctalia"),
    ("dms", "theme-bar-dms"),
    ("end4", "theme-bar-end-4"),
    ("off", "theme-bar-off"),
];

pub const STATIC_THEMES: [(&str, &str); 7] = [
    ("nord", "theme-bar-nord"),
    ("dracula", "theme-bar-dracula"),
    ("tokyo-night", "theme-bar-tokyo-night"),
    ("catppuccin", "theme-bar-catppuccin"),
    ("gruvbox", "theme-bar-gruvbox"),
    ("rose-pine", "theme-bar-rose-pine"),
    ("custom", "theme-bar-custom"),
];

pub const THEME_SCHEMES: [(&str, &str); 8] = [
    ("scheme-tonal-spot", "theme-bar-tonal-spot"),
    ("scheme-content", "theme-bar-content"),
    ("scheme-expressive", "theme-bar-expressive"),
    ("scheme-fidelity", "theme-bar-fidelity"),
    ("scheme-fruit-salad", "theme-bar-fruit-salad"),
    ("scheme-monochrome", "theme-bar-monochrome"),
    ("scheme-neutral", "theme-bar-neutral"),
    ("scheme-rainbow", "theme-bar-rainbow"),
];

pub const THEME_MODES: [(&str, &str); 3] =
    [("dark", "theme-bar-dark"), ("light", "theme-bar-light"), ("auto", "theme-bar-auto")];

pub const SKWD_STYLES: [(&str, &str); 4] = [
    ("natural", "theme-bar-natural"),
    ("pastel", "theme-bar-pastel"),
    ("muted", "theme-bar-muted"),
    ("vibrant", "theme-bar-vibrant"),
];

pub const SKWD_SCHEMES: [(&str, &str); 9] = [
    ("tonal-spot", "theme-bar-tonal-spot"),
    ("vibrant", "theme-bar-vibrant"),
    ("expressive", "theme-bar-expressive"),
    ("neutral", "theme-bar-neutral"),
    ("monochrome", "theme-bar-mono"),
    ("fidelity", "theme-bar-fidelity"),
    ("content", "theme-bar-content"),
    ("rainbow", "theme-bar-rainbow"),
    ("fruit-salad", "theme-bar-fruit-salad"),
];

pub const WALLUST_PALETTES: [(&str, &str); 6] = [
    ("dark", "theme-bar-dark"),
    ("dark16", "theme-bar-dark-16"),
    ("harddark", "theme-bar-hard-dark"),
    ("softdark", "theme-bar-soft-dark"),
    ("light", "theme-bar-light"),
    ("softlight", "theme-bar-soft-light"),
];

pub const WALLUST_COLORSPACES: [(&str, &str); 4] = [
    ("lab", "theme-bar-lab"),
    ("labmixed", "theme-bar-lab-mixed"),
    ("lch", "theme-bar-lch"),
    ("lchmixed", "theme-bar-lch-mixed"),
];

pub const PYWAL_SATURATIONS: [(&str, &str); 5] = [
    ("", "theme-bar-natural"),
    ("0.4", "theme-bar-saturate-0-4"),
    ("0.6", "theme-bar-saturate-0-6"),
    ("0.8", "theme-bar-saturate-0-8"),
    ("1.0", "theme-bar-saturate-1-0"),
];

pub const NOCTALIA_SCHEMES: [(&str, &str); 10] = [
    ("m3-tonal-spot", "theme-bar-tonal-spot"),
    ("m3-content", "theme-bar-content"),
    ("m3-fruit-salad", "theme-bar-fruit-salad"),
    ("m3-rainbow", "theme-bar-rainbow"),
    ("m3-monochrome", "theme-bar-mono"),
    ("vibrant", "theme-bar-vibrant"),
    ("faithful", "theme-bar-faithful"),
    ("soft", "theme-bar-soft"),
    ("dysfunctional", "theme-bar-dysfunc"),
    ("muted", "theme-bar-muted"),
];

pub fn backend_menu_options(
    available: Option<&[String]>,
    current: &str,
) -> Vec<(&'static str, &'static str)> {
    THEME_BACKENDS
        .iter()
        .copied()
        .filter(|(key, _)| {
            *key == "off"
                || *key == "native"
                || *key == "static"
                || *key == current
                || available.is_none_or(|avail| avail.iter().any(|name| name == key))
        })
        .collect()
}

mod tests;

#[derive(Default)]
pub struct ThemeBar {
    pub backend: String,
    pub menu_open: bool,
    pub mode: String,
    pub static_theme: String,
    pub scheme: String,
    pub style: String,
    pub iris_scheme: String,
    pub color_index: u32,
    pub wallust_palette: String,
    pub wallust_colorspace: String,
    pub pywal_saturate: String,
    pub noctalia_scheme: String,
    pub noctalia_pure_black: bool,
    pub backend_count: usize,
    pub wallpaper_settings_pinned: bool,
}
