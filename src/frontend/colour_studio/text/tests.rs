use super::*;
use skwd_palette::studio::Method;

#[test]
fn every_method_named() {
    for method in Method::ALL {
        assert_ne!(named(&METHODS, method.key()), method.key());
    }
}

#[test]
fn matugen_schemes_named() {
    for scheme in skwd_palette::studio::MATUGEN_SCHEMES {
        assert_ne!(named(&MATUGEN, scheme), *scheme);
    }
}

#[test]
fn readouts_stop_at_neutral() {
    assert_eq!(knob(Knob::Warmth, 0.0), tr("colour-studio-neutral"));
    assert!(knob(Knob::Warmth, 0.1).starts_with(tr("colour-studio-warmer")));
    assert!(knob(Knob::Colour, 1.6).ends_with("+60"));
    assert!(knob(Knob::Contrast, -1.0).ends_with("\u{2212}100"));
}

#[test]
fn shade_counts_steps() {
    assert_eq!(shade(0), tr("colour-studio-as-made"));
    assert!(shade(2).contains('2'));
    assert!(shade(-3).contains('3'));
    assert_ne!(shade(1), shade(-1));
}
