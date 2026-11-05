use std::collections::HashMap;

use crate::contracts::settings::{SettingsSource, keys};
use crate::frontend::settings::test_source::{FakeSettingsSource, place_source};
use crate::frontend::settings::{ActionId, Control, SettingsSearchResult, search_hits};
use crate::i18n::{set_language, tr};

use super::super::super::search_page::{Found, SUGGESTIONS, own_words, result_id};
use super::super::{TokenSide, token_key};
use super::{Extent, FIT_MARGIN, Live, OUT, SCREEN, Shot, capture_in, localized};

const LONG_QUERY: &str = "wallpapers that keep playing their sound while other apps play music on every monitor of a very long desk";
const SHOT_PLACE: &str = crate::frontend::settings::place::SYSTEM;
const BROAD_QUERY: &str = "a";
const WIDE_QUERY: &str = "e";
const ASLEEP_QUERY: &str = "awww";
const SECOND: usize = 1;

pub(super) struct Probe<'a> {
    pub found: Option<Found<'a>>,
    pub landed: Option<(usize, usize)>,
}

struct PageState {
    name: &'static str,
    cfg: FakeSettingsSource,
    query: String,
    at: Option<usize>,
    pop: Option<u64>,
    edit: Option<u64>,
    armed: Option<ActionId>,
}

fn state(name: &'static str, query: &str) -> PageState {
    PageState {
        name,
        cfg: FakeSettingsSource::default().with_niri(true),
        query: query.to_string(),
        at: None,
        pop: None,
        edit: None,
        armed: None,
    }
}

fn page_states() -> Vec<PageState> {
    let sentence = |path: &str| Some(token_key(path, 0, TokenSide::Sentence));
    let mut states =
        vec![state("empty", ""), state("none", "zzzzzz"), state("long-query", LONG_QUERY)];
    for (name, key) in
        ["sound", "fps", "blur", "lock", "gpu", "colour"].into_iter().zip(SUGGESTIONS)
    {
        states.push(state(name, tr(key)));
    }
    states.push(PageState {
        cfg: FakeSettingsSource::default(),
        ..state("off-niri", tr(SUGGESTIONS[2]))
    });
    states.push(state("top", BROAD_QUERY));
    states.push(state("top-wide", WIDE_QUERY));
    states.push(PageState { at: Some(SECOND), ..state("selected", tr(SUGGESTIONS[1])) });
    states.push(PageState {
        cfg: FakeSettingsSource::default().with_flag(keys::semantic::ENABLED, false),
        ..state("parked", "active model")
    });
    states.push(PageState {
        cfg: FakeSettingsSource::default()
            .with_niri(true)
            .with_text(keys::paper::ENGINE, "builtin"),
        ..state("asleep", ASLEEP_QUERY)
    });
    states.push(PageState {
        pop: sentence(keys::plasma::LOCK_SCREEN_MODE),
        ..state("menu", tr(SUGGESTIONS[3]))
    });
    states.push(PageState { edit: sentence(keys::general::MAX_FPS), ..state("number", "max fps") });
    states.push(PageState { armed: Some(ActionId::ClearCache), ..state("armed", "clear") });
    states
}

pub(super) fn seed(
    cfg: &dyn SettingsSource,
    found: Found<'_>,
    values: &mut HashMap<String, String>,
) {
    let rows = found.results.iter().flat_map(|result| {
        let home = result.home.iter().flat_map(crate::frontend::settings::Section::rows);
        result.setting.iter().chain(home)
    });
    for row in rows {
        match &row.control {
            Control::Number { key, path, .. } => {
                values.insert(
                    key.clone(),
                    crate::contracts::picker::format_config_number(cfg.number(path)),
                );
            }
            Control::TextField { key, path, .. } | Control::Resolution { key, path, .. } => {
                values.insert(key.clone(), cfg.text(path));
            }
            _ => {}
        }
    }
}

fn take(fit: &PageState, out: Option<&str>) -> (f32, f32) {
    let hits = search_hits(&fit.query, place_source(&fit.cfg));
    let found = Found { query: &fit.query, results: &hits.shown, total: hits.total, at: fit.at };
    let probe = Probe { found: Some(found), landed: None };
    let live = Live {
        search: Some(&probe),
        shelf: super::library::Shelf { armed: fit.armed.as_ref(), ..Default::default() },
        ..Live::default()
    };
    let shot = Shot {
        place: SHOT_PLACE,
        cfg: &fit.cfg,
        open: None,
        pop: fit.pop,
        edit: fit.edit,
        weather: None,
    };
    let last = hits.shown.len().checked_sub(1).map(result_id);
    let mut extent =
        Extent::new(crate::frontend::settings::settings_reading_id(), last, Vec::new());
    let view = capture_in(SCREEN, &shot, &live, out, &mut [&mut extent]).view;
    (view, extent.used())
}

#[test]
#[ignore = "manual offscreen render of the search page"]
fn render_search() {
    let out = localized();
    let mut heights = Vec::new();
    for fit in page_states() {
        let (view, used) = take(&fit, Some(&format!("{out}/{}.png", fit.name)));
        heights.push(format!("{}: used {used:.0} of {view:.0}", fit.name));
    }
    let off = FakeSettingsSource::default().with_flag(keys::semantic::ENABLED, false);
    let hit = every_hit(&off, tr("settings-tagging-model-active-label")).remove(0);
    let place = crate::frontend::settings::place::BAR;
    let open = hit.home.as_ref().map(|home| home.id);
    let shot = Shot { place, cfg: &off, open, pop: None, edit: None, weather: None };
    let probe = Probe { found: None, landed: Some((hit.section, hit.row)) };
    let live = Live { search: Some(&probe), ..Live::default() };
    capture_in(SCREEN, &shot, &live, Some(&format!("{out}/landed-parked.png")), &mut []);
    std::fs::write(format!("{out}/heights.txt"), heights.join("\n")).expect("heights");
}

#[test]
#[ignore = "offscreen layout probe of the search page in every locale"]
fn search_fits_every_locale() {
    let mut over = Vec::new();
    let mut worst = Vec::new();
    for language in crate::i18n::LANGUAGES {
        set_language(language.tag);
        iced_wgpu::graphics::text::font_system()
            .write()
            .expect("font system")
            .load_font(crate::shell::ui_font_bytes(language.script).into());
        let mut most = (0.0_f32, "");
        for fit in page_states() {
            let (view, used) = take(&fit, None);
            if used > most.0 {
                most = (used, fit.name);
            }
            if used > view - FIT_MARGIN {
                over.push(format!("{} {}: {:.0} of {view:.0}", language.tag, fit.name, used));
            }
        }
        worst.push(format!("{} {:.0} ({})", language.tag, most.0, most.1));
    }
    set_language("en-US");
    if let Ok(out) = std::env::var(OUT) {
        std::fs::write(format!("{out}/search-fit.txt"), worst.join("\n")).expect("fit report");
    }
    assert!(over.is_empty(), "{over:#?}");
}

fn every_hit(cfg: &FakeSettingsSource, query: &str) -> Vec<SettingsSearchResult> {
    search_hits(query, place_source(cfg)).shown
}

#[test]
fn suggestions_find_something() {
    let cfg = FakeSettingsSource::default().with_niri(true);
    for language in crate::i18n::LANGUAGES {
        crate::i18n::with_language(language.tag, || {
            for key in SUGGESTIONS {
                assert!(!every_hit(&cfg, tr(key)).is_empty(), "{} {key}", language.tag);
            }
        });
    }
}

#[test]
fn no_blank_results() {
    let cfg = FakeSettingsSource::default().with_niri(true);
    crate::i18n::with_language("en-US", || {
        let source = place_source(&cfg);
        let titles: Vec<String> = source
            .places()
            .iter()
            .flat_map(|place| &place.sections)
            .flat_map(crate::frontend::settings::Section::rows)
            .filter(|row| !matches!(row.control, Control::Segment | Control::Preview))
            .map(|row| row.title.clone())
            .collect();
        for title in titles.iter().filter(|title| !title.is_empty()) {
            for result in every_hit(&cfg, title) {
                let noted = result
                    .setting
                    .as_ref()
                    .is_some_and(|row| matches!(row.control, Control::Static));
                let spoken = noted || result.live().is_some() || !own_words(&result).is_empty();
                assert!(spoken, "{title}: {}", result.title);
            }
        }
    });
}
