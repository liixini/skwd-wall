use super::*;

#[test]
fn hsv_round_trip() {
    for (hue, saturation, value) in
        [(0.0, 1.0, 1.0), (120.0, 0.5, 0.8), (265.0, 0.45, 0.6), (359.0, 1.0, 0.2)]
    {
        let hex = hsv_to_hex(hue, saturation, value);
        let (actual_hue, actual_saturation, actual_value) = hex_to_hsv(&hex).expect("valid hex");
        assert!((hue - actual_hue).abs() < 3.0 || (hue - actual_hue).abs() > 357.0, "{hex}");
        assert!((saturation - actual_saturation).abs() < 0.02);
        assert!((value - actual_value).abs() < 0.02);
    }
}

#[test]
fn hsv_known_anchors() {
    assert_eq!(hsv_to_hex(0.0, 1.0, 1.0), "#ff0000");
    assert_eq!(hsv_to_hex(120.0, 1.0, 1.0), "#00ff00");
    assert_eq!(hsv_to_hex(240.0, 1.0, 1.0), "#0000ff");
    assert_eq!(hsv_to_hex(0.0, 0.0, 1.0), "#ffffff");
    assert_eq!(hsv_to_hex(0.0, 0.0, 0.0), "#000000");
    let grey = hex_to_hsv("#808080").unwrap();
    assert!(grey.1 < 0.01 && (grey.2 - 0.5).abs() < 0.01);
}

#[test]
fn preset_seed_roles() {
    let nord = Candidate::from_preset("nord").expect("nord preset exists");
    assert!(nord.colors.iter().all(|color| color.starts_with('#') && color.len() == 7));

    let seeded = Candidate::from_seed("#b48ead", true).expect("valid seed");
    assert!(seeded.colors.iter().all(|color| hex_to_hsv(color).is_some()));
    assert!(Candidate::from_seed("nope", true).is_none());
}

#[test]
fn role_indices_stable() {
    for (index, role) in ThemeRole::ALL.into_iter().enumerate() {
        assert_eq!(role.index(), index);
    }
    assert_eq!(THEME_ROLE_COUNT, 50);
}

#[test]
fn light_presets_open_their_original_colours_in_the_light_variant() {
    for name in ["catppuccin-latte", "rose-pine-dawn", "solarized-light", "github-light"] {
        let mut candidate = Candidate::from_preset(name).unwrap();
        let preset = skwd_palette::preset(name).unwrap();
        assert!(!candidate.dark, "{name}");
        assert_eq!(candidate.colors[ThemeRole::Primary.index()], preset.primary.hex());
        assert_eq!(candidate.colors[ThemeRole::Background.index()], preset.background.hex());
        candidate.set_dark(true);
        assert_ne!(candidate.colors[ThemeRole::Background.index()], preset.background.hex());
        candidate.set_dark(false);
        assert_eq!(candidate.colors[ThemeRole::Background.index()], preset.background.hex());
    }
}
