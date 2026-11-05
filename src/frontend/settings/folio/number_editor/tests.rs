use super::*;

#[test]
fn bounded_steps_keep_endpoints() {
    let spec = schema::find(crate::contracts::settings::keys::selector::DEPTH_SKEW).unwrap();
    let bounds = finite_bounds(spec).unwrap();
    let step = step_size(spec, bounds);
    assert!(step > 0.0);
    assert!(!can_step(bounds.min, -step, bounds));
    assert!(can_step(bounds.min, step, bounds));
    assert!(!can_step(bounds.max, step, bounds));
    assert!(can_step(bounds.max, -step, bounds));
    assert!(can_step(-0.5, -step, bounds));
}

#[test]
fn fractional_bounds_stay_precise() {
    for path in [
        "general.uiScale",
        "components.wallpaperSelector.gridFlowFrequency",
        "components.wallpaperSelector.hexCurveFrequency",
    ] {
        let spec = schema::find(path).unwrap();
        assert_eq!(step_size(spec, finite_bounds(spec).unwrap()), FINE_STEP);
    }
}

#[test]
fn millesimal_defaults_survive_drag() {
    for value in [0.005, 0.022] {
        let spec = SettingSpec::number("fractional", value);
        let bounds = NumberBounds { min: 0.0, max: 1.0 };
        let step = step_size(&spec, bounds);
        assert_eq!(step, 0.001);
        let Message::Settings(SettingsMsg::Input(_, raw)) =
            slider_input("fractional".into(), value, step, bounds)
        else {
            panic!("slider message")
        };
        assert_eq!(raw.parse::<f64>().unwrap(), value);
    }
}

#[test]
fn fractional_input() {
    let spec =
        schema::find(crate::contracts::settings::keys::selector::DEPTH_FALLOFF_FACTOR).unwrap();
    let bounds = finite_bounds(spec).unwrap();
    let step = step_size(spec, bounds);
    assert_eq!(step, FINE_STEP);
    assert!(can_step(0.99, step, bounds));
    assert!(!can_step(1.0, step, bounds));
    let key = spec.path.to_string();
    let Message::Settings(SettingsMsg::Input(actual, raw)) =
        slider_input(key.clone(), 0.29 + step, step, bounds)
    else {
        panic!("slider input message");
    };
    assert_eq!(actual, key);
    assert_eq!(raw, "0.30");
    assert_eq!(number_value("-0.12345", false), Some(-0.12345));
}

#[test]
fn automatic_and_invalid_drafts() {
    assert_eq!(number_value("", true), Some(0.0));
    assert_eq!(number_value("0", true), Some(0.0));
    assert_eq!(number_value("", false), None);
    assert_eq!(number_value("-", false), None);
    assert_eq!(number_value("NaN", false), None);
    assert_eq!(number_value("1.25", false), Some(1.25));
    assert!(finite_bounds(&SettingSpec::number("unbounded", 1.0)).is_none());
}

#[test]
fn step_text_keeps_precision() {
    assert_eq!(number_step_text(0.1 + 0.01, FINE_STEP, Some("0.10")), Some("0.11".into()));
    assert_eq!(
        number_step_text(0.12345 + FINE_STEP, FINE_STEP, Some("0.12345")),
        Some("0.13345".into())
    );
    assert_eq!(number_step_text(2.2, WHOLE_STEP, Some("1.20")), Some("2.20".into()));
    assert_eq!(number_step_text(0.0101, FINE_STEP, Some("1e-4")), Some("0.0101".into()));
    assert_eq!(number_step_text(f64::INFINITY, FINE_STEP, None), None);
}

#[test]
fn durations_read_whole() {
    crate::i18n::with_language("en-US", || {
        for (raw, expected) in [
            ("300", Some("5 min")),
            ("7200", Some("2 h")),
            ("90", None),
            ("0", None),
            ("1.5", None),
        ] {
            assert_eq!(duration_caption(raw).as_deref(), expected, "{raw}");
        }
    });
}

#[test]
fn unbounded_paths_scrub() {
    use crate::contracts::settings::keys;
    assert_eq!(scrub_bounds(keys::selector::WALLHAVEN_THUMB_WIDTH), (120.0, 960.0, 10.0));
    assert_eq!(scrub_bounds(keys::sources::YOUTUBE_MAX_HEIGHT), (144.0, 4320.0, 72.0));
    assert_eq!(step_number(keys::selector::STEAM_ROWS, 40.0, 1.0), None);
    assert_eq!(step_number(keys::selector::STEAM_ROWS, 40.0, -1.0), Some(8.0));
    assert_eq!(step_number(keys::selector::STEAM_ROWS, 3.0, 1.0), Some(4.0));
    for range in &SCRUB_RANGES {
        let spec = schema::find(range.path).expect("schema path");
        assert!(finite_bounds(spec).is_none(), "{}", range.path);
        assert!(range.min < range.max && range.step > 0.0, "{}", range.path);
    }
}
