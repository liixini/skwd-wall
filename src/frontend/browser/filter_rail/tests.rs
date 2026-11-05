use std::time::{Duration, Instant};

use iced::{Point, Size};

use super::rail::{Entry, GridState, Layout, OptionGrid, Section, cell_at, fitted_size};
use crate::frontend::animation::MotionProfile;
use crate::frontend::browser::{Browser, Source};
use crate::frontend::browser::{BrowserMsg, TagMark};
use crate::frontend::theme::Palette;

const GRID: Size = Size::new(200.0, 100.0);
const PAIRS: Layout = Layout { rows: 4, columns: 2, count: 7 };

fn sections(browser: &Browser) -> Vec<Section> {
    super::rail::sections(browser).expect("structured rail")
}

fn titles(browser: &Browser) -> Vec<&'static str> {
    sections(browser).iter().map(|section| section.title_key).collect()
}

fn section(browser: &Browser, key: &str) -> Section {
    sections(browser).into_iter().find(|section| section.title_key == key).expect("section")
}

fn chosen(section: &Section) -> Vec<String> {
    section
        .entries
        .iter()
        .filter(|entry| entry.mark == TagMark::Include)
        .map(|entry| entry.id.clone())
        .collect()
}

#[test]
fn cells_fill_rows() {
    assert_eq!(cell_at(Point::new(10.0, 10.0), GRID, &PAIRS, false), Some(0));
    assert_eq!(cell_at(Point::new(150.0, 10.0), GRID, &PAIRS, false), Some(1));
    assert_eq!(cell_at(Point::new(10.0, 90.0), GRID, &PAIRS, false), Some(6));
    assert_eq!(cell_at(Point::new(150.0, 90.0), GRID, &PAIRS, false), None);
}

#[test]
fn rtl_mirrors_columns() {
    assert_eq!(cell_at(Point::new(150.0, 10.0), GRID, &PAIRS, true), Some(0));
    assert_eq!(cell_at(Point::new(10.0, 10.0), GRID, &PAIRS, true), Some(1));
}

#[test]
fn swatch_columns_hit() {
    let swatches = Layout { rows: 2, columns: 7, count: 13 };
    let size = Size::new(140.0, 52.0);
    assert_eq!(cell_at(Point::new(130.0, 10.0), size, &swatches, false), Some(6));
    assert_eq!(cell_at(Point::new(10.0, 40.0), size, &swatches, false), Some(7));
    assert_eq!(cell_at(Point::new(130.0, 40.0), size, &swatches, false), None);
}

#[test]
fn outside_misses() {
    assert_eq!(cell_at(Point::new(-1.0, 10.0), GRID, &PAIRS, false), None);
    let empty = Layout { rows: 0, columns: 2, count: 0 };
    assert_eq!(cell_at(Point::new(10.0, 10.0), GRID, &empty, false), None);
}

fn grid(mark: TagMark) -> OptionGrid {
    OptionGrid {
        entries: vec![Entry::choice(
            "Anime",
            "Anime",
            mark == TagMark::Include,
            BrowserMsg::SteamTag(String::from("Anime"), false),
        )],
        rows: 1,
        columns: 2,
        scale: 1.0,
        fade: 1.0,
        motion: MotionProfile::default(),
        palette: Palette::default(),
    }
}

fn excluded_grid() -> OptionGrid {
    let mut grid = grid(TagMark::Off);
    grid.entries[0].mark = TagMark::Exclude;
    grid
}

#[test]
fn first_sight_instant() {
    let mut state = GridState::default();
    assert!(!grid(TagMark::Include).advance(&mut state, Instant::now()));
    assert_eq!(state.mixes["Anime"].include, 1.0);
}

#[test]
fn mark_change_moves() {
    let mut state = GridState::default();
    let start = Instant::now();
    grid(TagMark::Off).advance(&mut state, start);
    let step = start + Duration::from_millis(16);
    assert!(excluded_grid().advance(&mut state, step));
    assert!(excluded_grid().advance(&mut state, step + Duration::from_millis(16)));
    let exclude = state.mixes["Anime"].exclude;
    assert!(exclude > 0.0 && exclude < 1.0);
}

#[test]
fn settled_grid_idles() {
    let mut state = GridState::default();
    let mut now = Instant::now();
    grid(TagMark::Off).advance(&mut state, now);
    let settled = (0..240).any(|_| {
        now += Duration::from_millis(16);
        !grid(TagMark::Include).advance(&mut state, now)
    });
    assert!(settled);
    assert!(state.last_frame.is_none());
}

#[test]
fn long_labels_shrink() {
    assert_eq!(fitted_size("CGI", 100.0, 10.0), 10.0);
    let shrunk = fitted_size("Пиксельная графика и ретро", 60.0, 10.0);
    assert!(shrunk < 10.0 && shrunk > 0.0);
}

#[test]
fn other_sources_use_bar() {
    assert!(super::rail::sections(&Browser::new(Source::Unsplash)).is_none());
}

#[test]
fn period_follows_trend() {
    let mut browser = Browser::new(Source::Steam);
    assert!(titles(&browser).contains(&"browser-group-period"));
    browser.request.sorting = String::from("1");
    assert!(!titles(&browser).contains(&"browser-group-period"));
}

#[test]
fn single_choice_marks() {
    let mut browser = Browser::new(Source::Steam);
    browser.request.nsfw = true;
    let safety = section(&browser, "browser-group-safety");
    assert_eq!(chosen(&safety), ["1"]);
    assert!(safety.entries.iter().all(|entry| entry.exclude.is_none()));
}

#[test]
fn tags_offer_clear() {
    let mut browser = Browser::new(Source::Steam);
    assert!(section(&browser, "browser-group-tags").actions.is_empty());
    browser.request.steam.cycle_tag("Anime");
    browser.request.steam.cycle_tag("Anime");
    let tags = section(&browser, "browser-group-tags");
    assert!(tags.actions.iter().any(|action| action.message.is_some()));
    let anime = tags.entries.iter().find(|entry| entry.id == "Anime").expect("anime");
    assert_eq!(anime.mark, TagMark::Exclude);
    assert!(anime.title.starts_with('\u{2212}'));
}

#[test]
fn wallhaven_sections() {
    let browser = Browser::new(Source::Wallhaven);
    assert_eq!(
        titles(&browser),
        [
            "browser-group-content",
            "browser-group-order",
            "browser-group-top-range",
            "browser-group-safety",
            "browser-group-colour",
            "browser-group-resolution",
            "browser-group-ratio",
            "browser-max-size",
        ]
    );
    assert_eq!(chosen(&section(&browser, "browser-group-content")), ["general", "anime", "people"]);
    assert_eq!(chosen(&section(&browser, "browser-group-safety")), ["sfw"]);
}

#[test]
fn collection_narrows_rail() {
    let mut browser = Browser::new(Source::Wallhaven);
    browser.request.wallhaven.collections = vec![("12".into(), "Saved".into())];
    browser.request.wallhaven.collection = "12".into();
    assert_eq!(
        titles(&browser),
        ["browser-group-safety", "browser-max-size", "browser-group-collection"]
    );
    assert_eq!(chosen(&section(&browser, "browser-group-collection")), ["12"]);
}

#[test]
fn exact_sizes_multi_select() {
    let mut browser = Browser::new(Source::Wallhaven);
    browser.request.wallhaven.res_exact = true;
    browser.request.wallhaven.resolutions = String::from("1920x1080,3840x2160");
    assert!(!titles(&browser).contains(&"browser-max-size"));
    let sizes = section(&browser, "browser-group-resolution");
    assert_eq!(chosen(&sizes), ["1920x1080", "3840x2160"]);
    let modes: Vec<_> =
        sizes.actions.iter().map(|action| (action.current, action.message.is_some())).collect();
    assert_eq!(modes, [(false, true), (true, false)]);
}

#[test]
fn colour_toggles_off() {
    let mut browser = Browser::new(Source::Wallhaven);
    browser.request.wallhaven.color = 3;
    let colour = section(&browser, "browser-group-colour");
    assert_eq!(colour.columns, 7);
    assert!(colour.entries.iter().all(|entry| entry.swatch.is_some()));
    assert!(matches!(colour.entries[3].press, BrowserMsg::SetColor(-1)));
    assert!(matches!(colour.entries[4].press, BrowserMsg::SetColor(4)));
}
