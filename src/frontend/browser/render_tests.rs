use iced::Size;

use super::model::{Browser, BrowserItem, Source, SourceAvailability};
use crate::contracts::browser::{BrowserSearchResult, WorkshopDetails, WorkshopPreview};
use crate::contracts::picker::BrowserGrid;
use crate::frontend::theme::Palette;

const IMAGES: &str = "SKWD_BROWSER_IMAGES";
const OUT: &str = "SKWD_BROWSER_OUT";
const SCREENS: [(f32, f32, &str, usize); 3] =
    [(1366.0, 768.0, "small", 3), (2560.0, 1440.0, "large", 4), (2560.0, 2160.0, "tall", 6)];
const DESCRIPTION: &str = "A slow drift through a neon harbour at dusk. Rain on the glass, lanterns swaying in the wind and a train crossing the bridge every forty seconds.\n\nAudio reactive: the skyline pulses with your music. Settings for rain density, fog and lantern colour live in the properties panel.";

fn grid(small: bool, rows: usize) -> BrowserGrid {
    if small {
        BrowserGrid {
            cols: 4,
            rows,
            thumb_w: 220.0,
            thumb_h: 124.0,
            gap_x: 12.0,
            gap_y: 12.0,
            corner_radius: 8.0,
            border_width: 1.0,
        }
    } else {
        BrowserGrid {
            cols: 4,
            rows,
            thumb_w: 300.0,
            thumb_h: 169.0,
            gap_x: 16.0,
            gap_y: 16.0,
            corner_radius: 12.0,
            border_width: 1.0,
        }
    }
}

fn browser(images: &[String]) -> Browser {
    let mut browser = Browser::new(Source::Steam);
    browser.request.steam.tags = vec!["Anime".into(), "Landscape".into()];
    browser.request.steam.excluded_tags = vec!["Girls".into()];
    for (idx, path) in images.iter().enumerate() {
        let mut item = BrowserItem::from(BrowserSearchResult {
            id: format!("37{idx:08}"),
            full_url: format!("https://example.invalid/{idx}.jpg"),
            thumb_path: path.clone(),
            title: format!("Harbour lights {idx}"),
            resolution: "3840 x 2160".into(),
            category: "Scene, Anime, Landscape, Relaxing".into(),
            thumb_ready: true,
            ..BrowserSearchResult::default()
        });
        item.preview_paths.insert(item.full_url.clone(), path.clone());
        browser.session.items.push(item);
    }
    let previews = images
        .iter()
        .skip(1)
        .take(4)
        .map(|path| WorkshopPreview::Image { url: format!("file://{path}") })
        .chain(std::iter::once(WorkshopPreview::Youtube { id: "workshop-trailer".into() }))
        .collect::<Vec<_>>();
    let first = &mut browser.session.items[0];
    for (path, preview) in images.iter().skip(1).zip(&previews) {
        first.preview_handles.insert(
            preview.poster_url().into_owned(),
            iced::widget::image::Handle::from_path(path),
        );
    }
    first.title = "Neon harbour at dusk, rain and lanterns".into();
    first.workshop = Some(WorkshopDetails {
        description: DESCRIPTION.into(),
        creator_id: "76561198000000000".into(),
        creator_name: "lanternfield".into(),
        subscriptions: Some(184_233),
        votes_up: Some(9_812),
        votes_down: Some(214),
        time_created: 1_726_000_000,
        time_updated: 1_760_000_000,
        previews,
    });
    first.details_ready = true;
    browser
}

fn render(browser: &Browser, screen: Size, grid: BrowserGrid, preview: f32, out: &str) {
    let palette = Palette::default();
    let cache = iced::widget::canvas::Cache::new();
    let render = std::sync::Arc::new(crate::frontend::scene::RenderSnapshot::default());
    let uploads = crate::contracts::preview::UploadQueue::default();
    let pool = std::sync::Arc::new(crate::contracts::preview::BufPool::new(1, 1));
    let availability = [Source::Steam, Source::Wallhaven]
        .map(|source| SourceAvailability { source, unavailable: None });
    crate::frontend::headless::capture(screen, crate::frontend::ui::UI_FONT, out, || {
        super::view(
            browser,
            &availability,
            false,
            (screen.width, screen.height),
            grid,
            render.clone(),
            uploads.clone(),
            pool.clone(),
            &cache,
            1.0,
            0.0,
            0.0,
            preview,
            1.0,
            &palette,
            None,
        )
    });
}

#[test]
#[ignore = "manual offscreen render of the Workshop browser"]
fn render_workshop() {
    let images: Vec<String> =
        std::env::var(IMAGES).expect(IMAGES).split(':').map(str::to_owned).collect();
    let out = std::env::var(OUT).expect(OUT);
    iced_wgpu::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(crate::i18n::Script::Latin).into());
    let mut browser = browser(&images);
    for (width, height, name, rows) in SCREENS {
        let screen = Size::new(width, height);
        let grid = grid(name == "small", rows);
        browser.session.preview = None;
        render(&browser, screen, grid, 0.0, &format!("{out}/browser-{name}.png"));
        browser.session.preview = Some(0);
        render(&browser, screen, grid, 1.0, &format!("{out}/detail-{name}.png"));
        let mut wallhaven = Browser::new(Source::Wallhaven);
        wallhaven.request.wallhaven.color = 7;
        wallhaven.request.wallhaven.atleast = String::from("2560x1440");
        wallhaven.request.wallhaven.ratios = String::from("16x9");
        render(&wallhaven, screen, grid, 0.0, &format!("{out}/wallhaven-{name}.png"));
    }
}
