use super::*;
use crate::frontend::settings::test_source::{
    FakeSettingsSource, place_source as source, plain, render,
};
use crate::frontend::settings::{ActionId, place};
use crate::frontend::ui::calm::sentinel;

const SAVER: &str = keys::performance::BATTERY_SAVER;
const BATTERY_FPS: &str = keys::performance::BATTERY_FPS;
const BATTERY_IDLE: &str = keys::performance::BATTERY_VIDEO_IDLE_SECONDS;
const DETAIL: &str = keys::performance::BATTERY_WALLPAPER_PERFORMANCE;

fn sections(cfg: &dyn SettingsSource) -> Vec<Section> {
    crate::i18n::with_language("en-US", || source(cfg).place(place::SYSTEM).sections)
}

fn section(cfg: &dyn SettingsSource, key: &str) -> Section {
    sections(cfg).into_iter().find(|section| section.id.key == key).expect("section")
}

fn path(row: &Row) -> Option<&str> {
    row.control.path()
}

#[test]
fn board_sections_lead() {
    let keys: Vec<_> =
        sections(&FakeSettingsSource::default()).iter().map(|section| section.id.key).collect();
    assert_eq!(keys, [POWER, GPU, LANGUAGE, BUG_REPORT, TRANSFER, EFFICIENCY]);
}

#[test]
fn power_sentence_states() {
    for (saver, fps, idle, detail, expected) in [
        (
            true,
            "30",
            "60",
            true,
            "On battery the picker caps at [0], video stops after [1] without input, and wallpapers switch to performance mode.",
        ),
        (
            true,
            "0",
            "60",
            true,
            "On battery the picker has no extra cap ([0]), video stops after [1] without input, and wallpapers switch to performance mode.",
        ),
        (
            true,
            "30",
            "0",
            false,
            "On battery the picker caps at [0], only the normal Idle pause applies ([1]), and wallpapers keep full detail.",
        ),
        (
            false,
            "30",
            "60",
            true,
            "Off. On battery, the picker and wallpapers run with the same limits as on mains power.",
        ),
    ] {
        let cfg = FakeSettingsSource::default()
            .with_flag(SAVER, saver)
            .with_text(BATTERY_FPS, fps)
            .with_text(BATTERY_IDLE, idle)
            .with_flag(DETAIL, detail);
        assert_eq!(plain(&section(&cfg, POWER).say.text), expected);
    }
}

#[test]
fn power_tokens_bound() {
    let power = section(&FakeSettingsSource::default().with_flag(SAVER, true), POWER);
    let bound: Vec<_> = power.say.rows().filter_map(|at| power.row(at)).filter_map(path).collect();
    assert_eq!(bound, [BATTERY_FPS, BATTERY_IDLE]);
    let off = section(&FakeSettingsSource::default().with_flag(SAVER, false), POWER);
    assert!(off.say.tokens.is_empty());
}

#[test]
fn power_body_rows() {
    for saver in [true, false] {
        let power = section(&FakeSettingsSource::default().with_flag(SAVER, saver), POWER);
        assert_eq!(power.master.as_ref().and_then(path), Some(SAVER));
        assert_eq!(power.master_on(), Some(saver));
        let groups: Vec<Vec<_>> =
            power.groups.iter().map(|group| group.rows.iter().filter_map(path).collect()).collect();
        assert_eq!(
            groups,
            [
                vec![
                    keys::general::MAX_FPS,
                    keys::paper::IDLE_PAUSE_SECONDS,
                    keys::paper::PERFORMANCE_MODE
                ],
                vec![BATTERY_FPS, BATTERY_IDLE, DETAIL],
            ]
        );
        let follows: Vec<_> = power.groups.iter().map(|group| group.follows_master).collect();
        assert_eq!(follows, [false, true]);
        assert!(power.say.aside.is_some() && power.say.notes.is_empty());
    }
}

#[test]
fn gpu_sentence_modes() {
    for (value, note) in [
        ("auto", ": it prefers a low-power GPU while the battery saver is active."),
        ("low", ": it asks for an integrated GPU."),
        ("high", ": it asks for a discrete GPU."),
        ("none", ": the driver decides."),
        ("turbo", ", which it does not recognise, so it behaves like Auto."),
    ] {
        let cfg = FakeSettingsSource::default().with_text(keys::performance::GPU_PREFERENCE, value);
        let gpu = section(&cfg, GPU);
        assert_eq!(plain(&gpu.say.text), format!("The picker uses [0]{note}"), "{value}");
        assert_eq!(gpu.head.first().and_then(path), Some(keys::performance::GPU_PREFERENCE));
        assert!(!gpu.has_body());
    }
}

#[test]
fn language_sentence_follows() {
    for (value, expected) in [
        ("auto", "The interface is in [0], which follows your desktop language."),
        ("sv-SE", "The interface is in [0]."),
    ] {
        let cfg = FakeSettingsSource::default().with_text(keys::general::LANGUAGE, value);
        let language = section(&cfg, LANGUAGE);
        assert!(plain(&language.say.text).starts_with(expected), "{value}");
        assert_eq!(language.head.first().and_then(path), Some(keys::general::LANGUAGE));
        assert_eq!(language.say.rows().collect::<Vec<_>>(), [0]);
    }
}

#[test]
fn actions_in_header() {
    let cfg = FakeSettingsSource::default();
    for (key, expected) in [
        (BUG_REPORT, vec![ActionId::GenerateBugReport]),
        (TRANSFER, vec![ActionId::ExportConfiguration, ActionId::ImportConfiguration]),
    ] {
        let section = section(&cfg, key);
        let actions: Vec<_> = section
            .head
            .iter()
            .filter_map(|row| match &row.control {
                Control::ActionBtn { id, .. } => Some(id.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(actions, expected, "{key}");
        assert!(section.groups.is_empty());
    }
}

#[test]
fn wallpaper_rows_interim() {
    let efficiency = section(&FakeSettingsSource::default(), EFFICIENCY);
    let paths: Vec<_> = efficiency.rows().filter_map(path).collect();
    assert!(paths.is_empty(), "{paths:?}");
    assert!(efficiency.rows().any(|row| row.title == "Wallpaper Engine scene renderer"));
}

#[test]
fn power_status_lines() {
    for (battery, saver, source, limits) in [
        (
            false,
            true,
            "External power, desktop, or unavailable (automatic limits are inactive)",
            "On external power",
        ),
        (true, false, "Battery (automatic limits are disabled)", "Battery saver is off"),
        (true, true, "Battery (automatic limits are active)", "On battery"),
    ] {
        let cfg = FakeSettingsSource::default().with_on_battery(battery).with_flag(SAVER, saver);
        let power = section(&cfg, POWER);
        assert_eq!(power.say.aside, Some(format!("Current power source: {source}")));
        let effective = power.groups[0].note.as_deref().unwrap_or_default();
        assert!(effective.starts_with("Effective limits: "), "{effective}");
        assert!(effective.ends_with(limits), "{effective}");
    }
}

#[test]
fn head_rows_drawn() {
    for saver in [true, false] {
        let cfg = FakeSettingsSource::default().with_flag(SAVER, saver);
        let composed = crate::i18n::with_language("en-US", || compose(&source(&cfg)));
        assert!(composed.iter().all(crate::frontend::settings::places::drawn));
        assert_eq!(composed.len(), sections(&cfg).len());
    }
}

#[test]
fn sentences_keep_slots() {
    let (fps, seconds) =
        (sentinel(0).expect("slot").to_string(), sentinel(1).expect("slot").to_string());
    for language in crate::i18n::LANGUAGES {
        let tag = language.tag;
        for cap in FPS_CAP.values {
            for idle in VIDEO_IDLE.values {
                for detail in BATTERY_DETAIL.values {
                    let text = render(
                        tag,
                        POWER_SAY,
                        &[
                            (FPS_CAP.arg, cap),
                            ("fps", &fps),
                            (VIDEO_IDLE.arg, idle),
                            ("seconds", &seconds),
                            (BATTERY_DETAIL.arg, detail),
                        ],
                    );
                    for slot in [fps.as_str(), seconds.as_str()] {
                        assert_eq!(text.matches(slot).count(), 1, "{tag} {text}");
                    }
                }
            }
        }
        for mode in GPU_VARIANTS {
            let text = render(tag, GPU_SAY, &[(GPU_SLOT.arg, &fps), (GPU_MODE, mode)]);
            assert_eq!(text.matches(&fps).count(), 1, "{tag} {mode}");
        }
        for follow in LANGUAGE_FOLLOWS.values {
            let text = render(
                tag,
                LANGUAGE_SAY,
                &[(LANGUAGE_SLOT.arg, &fps), (LANGUAGE_FOLLOWS.arg, follow)],
            );
            assert_eq!(text.matches(&fps).count(), 1, "{tag} {follow}");
        }
        assert!(render(tag, POWER_SOURCE, &[("source", &fps)]).contains(&fps), "{tag}");
    }
}

const GPU_VARIANTS: [&str; 5] = ["auto", "low", "high", "none", CUSTOM_GPU];

#[test]
fn gpu_variants_match_menu() {
    let gpu = section(&FakeSettingsSource::default(), GPU);
    let Control::Dropdown { options, .. } = &gpu.head[0].control else { panic!("menu") };
    let mut values: Vec<_> = options.iter().map(|(value, _)| value.as_str()).collect();
    values.push(CUSTOM_GPU);
    assert_eq!(values, GPU_VARIANTS);
    for language in crate::i18n::LANGUAGES {
        let say = |mode: &str| render(language.tag, GPU_SAY, &[("gpu", "x"), (GPU_MODE, mode)]);
        assert_eq!(say("turbo"), say(CUSTOM_GPU), "{}", language.tag);
    }
}

#[test]
fn selectors_change_text() {
    let power = |tag: &str, picks: [(&str, &str); 3]| {
        let mut pairs = vec![("fps", "x"), ("seconds", "y")];
        pairs.extend(picks);
        render(tag, POWER_SAY, &pairs)
    };
    for language in crate::i18n::LANGUAGES {
        let tag = language.tag;
        let choices = [FPS_CAP, VIDEO_IDLE, BATTERY_DETAIL];
        for (at, choice) in choices.iter().enumerate() {
            let picks = |on: bool| {
                let mut picks = choices.each_ref().map(|other| other.pick(true));
                picks[at] = choice.pick(on);
                picks
            };
            assert_ne!(power(tag, picks(true)), power(tag, picks(false)), "{tag} {}", choice.arg);
        }
        let gpu: std::collections::BTreeSet<_> = GPU_VARIANTS
            .into_iter()
            .map(|mode| render(tag, GPU_SAY, &[("gpu", "x"), (GPU_MODE, mode)]))
            .collect();
        assert_eq!(gpu.len(), GPU_VARIANTS.len(), "{tag}");
        let follow =
            |on| render(tag, LANGUAGE_SAY, &[("language", "x"), LANGUAGE_FOLLOWS.pick(on)]);
        assert_ne!(follow(true), follow(false), "{tag}");
    }
}
