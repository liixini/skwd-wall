use super::*;
use crate::frontend::settings::places::drawn;
use crate::frontend::settings::test_source::{
    FakeSettingsSource, place_source as source, plain as shown, render,
};
use crate::frontend::settings::{Control, Slot, place};
use crate::frontend::ui::calm::sentinel;

fn sections(cfg: &dyn SettingsSource) -> Vec<Section> {
    crate::i18n::with_language("en-US", || source(cfg).place(place::SOURCES).sections)
}

fn section(cfg: &dyn SettingsSource, key: &str) -> Section {
    sections(cfg).into_iter().find(|section| section.id.key == key).expect("section")
}

fn bound(section: &Section) -> Vec<Vec<&str>> {
    section
        .say
        .tokens
        .iter()
        .map(|slot| slot.rows().filter_map(|at| section.row(at)?.control.path()).collect())
        .collect()
}

fn columns(section: &Section) -> Vec<Vec<&str>> {
    section
        .groups
        .iter()
        .map(|group| group.rows.iter().filter_map(|row| row.control.path()).collect())
        .collect()
}

fn titles(section: &Section) -> Vec<String> {
    section.groups.iter().filter_map(|group| group.heading.clone()).collect()
}

#[test]
fn board_sections_only() {
    let keys: Vec<_> =
        sections(&FakeSettingsSource::default()).iter().map(|section| section.id.key).collect();
    assert_eq!(keys, [WALLHAVEN, WORKSHOP, OTHERS]);
}

#[test]
fn wallhaven_sentence_states() {
    let sort = wallhaven::DEFAULT_SORT;
    for (cfg, expected) in [
        (
            FakeSettingsSource::default(),
            "New sessions sort by [0] over [1], ratio [2], showing [3] at [4].",
        ),
        (
            FakeSettingsSource::default().with_text(sort, "hot"),
            "New sessions sort by [0], ratio [1], showing [2] at [3].",
        ),
        (
            FakeSettingsSource::default().with_text(sort, "oldest"),
            "New sessions sort by [0], ratio [1], showing [2] at [3].",
        ),
        (
            FakeSettingsSource::default().with_flag(keys::features::WALLHAVEN, false),
            "Off. The Wallhaven browser is hidden.",
        ),
    ] {
        assert_eq!(shown(&section(&cfg, WALLHAVEN).say.text), expected);
    }
}

#[test]
fn wallhaven_tokens_bound() {
    let wallhaven = section(&FakeSettingsSource::default(), WALLHAVEN);
    assert_eq!(
        bound(&wallhaven),
        [
            vec![wallhaven::DEFAULT_SORT],
            vec![wallhaven::DEFAULT_TOP_RANGE],
            vec![wallhaven::DEFAULT_RATIOS],
            vec![wallhaven::DEFAULT_GENERAL, wallhaven::DEFAULT_ANIME, wallhaven::DEFAULT_PEOPLE],
            vec![wallhaven::DEFAULT_SFW, wallhaven::DEFAULT_SKETCHY, wallhaven::DEFAULT_NSFW],
        ]
    );
    let checks: Vec<_> = wallhaven
        .say
        .tokens
        .iter()
        .filter_map(|slot| match slot {
            Slot::Checks { heading, blank, .. } => Some((heading.as_str(), blank.as_str())),
            Slot::Value { .. } => None,
        })
        .collect();
    assert_eq!(checks, [("Default category", "no category"), ("Content", "no content level")]);
    assert_eq!(
        wallhaven.master.as_ref().and_then(|row| row.control.path()),
        Some(keys::features::WALLHAVEN)
    );
    let head: Vec<_> = wallhaven.head.iter().filter_map(|row| row.control.path()).collect();
    assert_eq!(head, bound(&wallhaven).concat());
}

#[test]
fn hidden_rows_parked() {
    let causes = |cfg: &FakeSettingsSource, key| {
        let section = section(cfg, key);
        let mut causes: Vec<_> = section
            .say
            .parked
            .iter()
            .filter_map(|parked| section.row(parked.cause)?.control.path().map(str::to_string))
            .collect();
        causes.dedup();
        (section.say.parked.len(), causes)
    };
    assert_eq!(causes(&FakeSettingsSource::default(), WALLHAVEN), (0, Vec::<String>::new()));
    let hot = FakeSettingsSource::default().with_text(wallhaven::DEFAULT_SORT, "hot");
    assert_eq!(causes(&hot, WALLHAVEN), (1, vec![wallhaven::DEFAULT_SORT.to_string()]));
    let off = FakeSettingsSource::default()
        .with_flag(keys::features::WALLHAVEN, false)
        .with_flag(keys::features::STEAM, false);
    assert_eq!(causes(&off, WALLHAVEN), (9, vec![keys::features::WALLHAVEN.to_string()]));
    assert_eq!(causes(&off, WORKSHOP), (5, vec![keys::features::STEAM.to_string()]));
}

#[test]
fn parked_rows_found() {
    let cfg = FakeSettingsSource::default().with_text(wallhaven::DEFAULT_SORT, "hot");
    let results = crate::i18n::with_language("en-US", || {
        crate::frontend::settings::search_settings("default time range", source(&cfg))
    });
    let wallhaven = section(&cfg, WALLHAVEN);
    let range = wallhaven
        .rows()
        .position(|row| row.control.path() == Some(wallhaven::DEFAULT_TOP_RANGE))
        .expect("range");
    assert!(results.iter().any(|result| result.tab == place::SOURCES && result.row == range));
}

#[test]
fn secrets_stay_masked() {
    let cfg = FakeSettingsSource::default().with_text(wallhaven::API_KEY, "0123456789abcdef");
    let results = crate::i18n::with_language("en-US", || {
        crate::frontend::settings::search_settings("api key", source(&cfg))
    });
    assert!(!results.is_empty());
    assert!(results.iter().all(|result| !result.value.contains("0123")), "{results:?}");
    let secret = section(&cfg, WALLHAVEN).groups[0].rows[0].control.clone();
    assert!(matches!(secret, Control::TextField { secret: true, .. }));
}

#[test]
fn head_rows_drawn() {
    for cfg in [
        FakeSettingsSource::default(),
        FakeSettingsSource::default()
            .with_text(wallhaven::DEFAULT_SORT, "random")
            .with_text(steam::DEFAULT_SORT, "9"),
        FakeSettingsSource::default()
            .with_flag(keys::features::WALLHAVEN, false)
            .with_flag(keys::features::STEAM, false),
    ] {
        let composed = crate::i18n::with_language("en-US", || compose(&source(&cfg)));
        assert!(composed.iter().all(drawn));
        assert_eq!(composed.len(), sections(&cfg).len());
    }
}

#[test]
fn wallhaven_body_rows() {
    let wallhaven = section(&FakeSettingsSource::default(), WALLHAVEN);
    assert_eq!(titles(&wallhaven), ["Account", "Browser"]);
    assert_eq!(
        columns(&wallhaven),
        [
            vec![wallhaven::API_KEY, wallhaven::USERNAME, providers::WALLHAVEN_SHOW_APPLY_BUTTON],
            vec![
                wallhaven::DEFAULT_ATLEAST,
                wallhaven::DEFAULT_ATMOST,
                grid::WALLHAVEN_COLUMNS,
                grid::WALLHAVEN_ROWS,
                grid::WALLHAVEN_THUMB_WIDTH,
                grid::WALLHAVEN_THUMB_HEIGHT,
            ],
        ]
    );
    let browser = &wallhaven.groups[1];
    let cells: Vec<_> = browser.rows.iter().map(|row| row.look.cell).collect();
    assert_eq!(
        cells,
        [Cell::Value, Cell::Value, Cell::Value, Cell::Joined, Cell::Value, Cell::Joined]
    );
    let paired: Vec<_> =
        [2, 4].map(|at| (browser.rows[at].title.as_str(), browser.rows[at].hint())).to_vec();
    assert_eq!(paired, [("Grid", "Columns × rows"), ("Thumbnail", "Width × height")]);
    assert_eq!(wallhaven.groups[0].blank.as_deref(), Some("not set"));
    let key = &wallhaven.groups[0].rows[0];
    assert_eq!(key.hint(), "Needed for NSFW results");
    assert!(key.desc.starts_with("Enter a Wallhaven API key"), "desc kept");
}

#[test]
fn workshop_sentence_states() {
    for (cfg, expected) in [
        (
            FakeSettingsSource::default(),
            "New sessions sort by [0] over [1], type [2], resolution [3], category [4].",
        ),
        (
            FakeSettingsSource::default().with_text(steam::DEFAULT_SORT, "1"),
            "New sessions sort by [0], type [1], resolution [2], category [3].",
        ),
        (
            FakeSettingsSource::default().with_flag(keys::features::STEAM, false),
            "Off. The Steam Workshop browser and the Wallpaper Engine scene settings are hidden.",
        ),
    ] {
        assert_eq!(shown(&section(&cfg, WORKSHOP).say.text), expected);
    }
    let workshop = section(&FakeSettingsSource::default(), WORKSHOP);
    assert_eq!(
        bound(&workshop).concat(),
        [
            steam::DEFAULT_SORT,
            steam::DEFAULT_TREND_DAYS,
            steam::DEFAULT_TYPE,
            steam::DEFAULT_RESOLUTION,
            steam::DEFAULT_CATEGORY
        ]
    );
}

#[test]
fn workshop_body_rows() {
    let workshop = section(&FakeSettingsSource::default(), WORKSHOP);
    assert_eq!(titles(&workshop), ["Account", "Browser and folders"]);
    assert_eq!(
        columns(&workshop),
        [
            vec![steam::BACKEND, steam::USERNAME, steam::API_KEY, steam::DEFAULT_NSFW],
            vec![
                grid::STEAM_COLUMNS,
                grid::STEAM_ROWS,
                grid::STEAM_THUMB_WIDTH,
                grid::STEAM_THUMB_HEIGHT,
                providers::STEAM_SHOW_APPLY_BUTTON,
                keys::paths::STEAM_WORKSHOP,
                keys::paths::STEAM_WE_ASSETS,
                keys::paths::STEAM,
            ],
        ]
    );
    let [account, folders] = &workshop.groups[..] else { panic!("two columns") };
    assert_eq!(account.rows[0].look.cell, Cell::Buttons(&METHODS));
    let cells: Vec<_> = folders.rows.iter().map(|row| row.look.cell).collect();
    assert_eq!(
        cells,
        [
            Cell::Value,
            Cell::Joined,
            Cell::Value,
            Cell::Joined,
            Cell::Value,
            Cell::Field,
            Cell::Field,
            Cell::Field
        ]
    );
    assert_eq!(folders.blank.as_deref(), Some("Detected from Steam"));
}

#[test]
fn method_follows_runtime() {
    for (stored, lit, hint) in [
        ("steam", "steam", "Needs skwd-deck-steamworks and running Steam"),
        ("", "steam", "Needs skwd-deck-steamworks and running Steam"),
        ("steamcmd", "steamcmd", "Needs steamcmd and a one-time login"),
        ("legacy", "steamcmd", "Needs steamcmd and a one-time login"),
    ] {
        let cfg = FakeSettingsSource::default().with_text(steam::BACKEND, stored);
        let workshop = section(&cfg, WORKSHOP);
        let row = &workshop.groups[0].rows[0];
        assert_eq!(row.hint(), hint, "{stored:?}");
        let Control::Dropdown { options, current, .. } = &row.control else { panic!("menu") };
        assert_eq!(current, lit, "{stored:?}");
        let values: Vec<_> = options.iter().map(|(value, _)| value.as_str()).collect();
        assert_eq!(values, METHODS.map(|(value, _)| value));
    }
}

#[test]
fn others_sentence_states() {
    let keyed = |cfg: FakeSettingsSource| {
        cfg.with_text(providers::UNSPLASH_ACCESS_KEY, "u-key")
            .with_text(providers::PEXELS_API_KEY, "p-key")
    };
    let all = |on: bool| {
        FakeSettingsSource::default()
            .with_flag(providers::UNSPLASH_ENABLED, on)
            .with_flag(providers::PEXELS_ENABLED, on)
            .with_flag(providers::BING_ENABLED, on)
            .with_flag(providers::YOUTUBE_ENABLED, on)
    };
    for (cfg, expected) in [
        (all(false), "No source is searching. Unsplash, Pexels, Bing daily and YouTube are off."),
        (keyed(all(true)), "Searching Unsplash, Pexels, Bing daily and YouTube."),
        (
            all(true),
            "Searching Bing daily and YouTube. Unsplash needs an access key. Pexels needs an API key.",
        ),
        (
            keyed(all(true)).with_flag(providers::YOUTUBE_ENABLED, false),
            "Searching Unsplash, Pexels and Bing daily. YouTube is off.",
        ),
        (
            all(false).with_flag(providers::UNSPLASH_ENABLED, true),
            "No source is searching. Unsplash needs an access key. Pexels, Bing daily and YouTube are off.",
        ),
    ] {
        let others = section(&cfg, OTHERS);
        assert_eq!(others.say.text, expected);
        assert!(others.say.tokens.is_empty() && others.head.is_empty() && others.master.is_none());
    }
}

#[test]
fn others_body_rows() {
    let others = section(&FakeSettingsSource::default(), OTHERS);
    assert_eq!(titles(&others), ["Photos", "Daily images and video"]);
    assert_eq!(
        columns(&others),
        [PHOTO_ROWS.map(|(path, _)| path), DAILY_ROWS.map(|(path, _)| path)]
    );
    let hints: Vec<_> = others.body().map(Row::hint).filter(|hint| !hint.is_empty()).collect();
    assert_eq!(
        hints,
        [
            "Free from unsplash.com/developers",
            "Free from pexels.com/api",
            "The last 8 homepage images",
            "Needs yt-dlp on PATH",
            "0 downloads everything",
        ]
    );
}

#[test]
fn sentences_keep_slots() {
    let marks: Vec<_> = (0..5).map(|at| sentinel(at).expect("slot").to_string()).collect();
    let says = [
        (WALLHAVEN_SAY, TOP, WALLHAVEN_SLOTS.map(|slot| slot.arg())),
        (WORKSHOP_SAY, TREND, WORKSHOP_SLOTS.map(|slot| slot.arg)),
    ];
    for language in crate::i18n::LANGUAGES {
        let tag = language.tag;
        for (key, choice, args) in &says {
            for (on, value) in [false, true].into_iter().zip(choice.values) {
                let mut pairs: Vec<(&str, &str)> =
                    args.iter().zip(&marks).map(|(arg, mark)| (*arg, mark.as_str())).collect();
                pairs.push((choice.arg, value));
                let text = render(tag, key, &pairs);
                let counts: Vec<_> =
                    marks.iter().map(|mark| text.matches(mark.as_str()).count()).collect();
                let expected = if on { [1; 5] } else { [1, 0, 1, 1, 1] };
                assert_eq!(counts, expected, "{tag} {key} {value} {text}");
            }
        }
    }
}

#[test]
fn selectors_change_text() {
    for language in crate::i18n::LANGUAGES {
        let tag = language.tag;
        for (key, choice, slots) in [
            (WALLHAVEN_SAY, TOP, WALLHAVEN_SLOTS.map(|slot| slot.arg())),
            (WORKSHOP_SAY, TREND, WORKSHOP_SLOTS.map(|slot| slot.arg)),
        ] {
            let say = |on: bool| {
                let mut pairs: Vec<(&str, &str)> = slots.iter().map(|arg| (*arg, "x")).collect();
                pairs.push(choice.pick(on));
                render(tag, key, &pairs)
            };
            assert_ne!(say(true), say(false), "{tag} {key}");
        }
        let needs = |kind| render(tag, NEEDS_KEY, &[("source", "x"), ("key", kind)]);
        assert_ne!(needs("access"), needs("api"), "{tag}");
        let off = |count| render(tag, SWITCHED_OFF, &[("sources", "x"), ("count", count)]);
        assert!(off(MANY.values[0]).contains('x') && off(MANY.values[1]).contains('x'), "{tag}");
        assert!(render(tag, SEARCHING, &[("sources", "x")]).contains('x'), "{tag}");
    }
}

#[test]
fn lists_every_count() {
    let items = ["@1", "@2", "@3", "@4", "@5"];
    for language in crate::i18n::LANGUAGES {
        crate::i18n::with_language(language.tag, || {
            for count in 1..items.len() {
                let text = listed(LIST_KEY, &items[..count]);
                for item in &items[..count] {
                    assert_eq!(text.matches(item).count(), 1, "{} {text}", language.tag);
                }
            }
            let many = listed(LIST_KEY, &items);
            assert!(items[..3].iter().all(|item| many.contains(item)), "{} {many}", language.tag);
            assert_ne!(many, listed(LIST_KEY, &items[..4]), "{}", language.tag);
        });
    }
}
