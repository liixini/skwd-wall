use iced::{Font, Size};
use skwd_palette::studio::Tuning;

use super::controls::{Hit, Row, StudioMsg, View};
use super::state::{ColourStudio, Load, Wallpaper};
use crate::domain::studio::{Mode, Source, Studio};
use crate::frontend::theme::Palette;

const SCREEN: Size = Size::new(1366.0, 768.0);
const SETTLE: f32 = 5.0;
const TICK: f32 = 1.0 / 60.0;
const IMAGE: &str = "SKWD_STUDIO_IMAGE";
const SCAN: &str = "SKWD_STUDIO_SCAN";
const OUT: &str = "SKWD_STUDIO_OUT";
const AT: &str = "SKWD_STUDIO_AT";

fn studio(image: &str, source: &str, view: View) -> ColourStudio {
    let wallpaper = Wallpaper {
        key: image.into(),
        label: std::path::Path::new(image)
            .file_stem()
            .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned()),
        image: iced::widget::image::Handle::from_path(image),
        current: true,
        aspect: None,
    };
    let mut model = Studio::new(Source::parse(source), Mode::Dark, Tuning::default());
    model.live = Some(Palette::default().studio());
    if let Ok(scan) = std::env::var(SCAN) {
        let output = std::process::Command::new(scan)
            .arg(wall_proto::STUDIO_SCAN_FLAG)
            .arg(image)
            .output()
            .expect("scan runs");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("scan json");
        model.snapshot =
            Some(crate::infrastructure::rpc_results::decode_studio(&value).expect("snapshot"));
    }
    let mut studio = ColourStudio::new(model, wallpaper, Vec::new());
    studio.set_decoded(super::decode(image).map_or(Load::Failed, Load::Ready));
    studio.sync();
    let at = std::env::var(AT).ok().and_then(|value| value.parse::<f32>().ok());
    let mut left = at.unwrap_or(SETTLE);
    while left > 0.0 {
        studio.tick(TICK.min(left));
        left -= TICK;
    }
    if at.is_none() {
        studio.update(&StudioMsg::SkipIntro);
        studio.view = view;
        studio.sync();
        let mut left = SETTLE;
        while left > 0.0 {
            studio.tick(TICK);
            left -= TICK;
        }
    }
    studio
}

fn render(studio: &ColourStudio, out: &str) {
    let palette = Palette::from_studio(&studio.model.live.expect("live palette"));
    crate::frontend::headless::capture(SCREEN, Font::DEFAULT, out, || studio.view(&palette, 1.0));
}

#[test]
#[ignore = "manual offscreen render of the colour studio"]
fn render_studio() {
    let image = std::env::var(IMAGE).expect("SKWD_STUDIO_IMAGE");
    let out = std::env::var(OUT).expect("SKWD_STUDIO_OUT");
    for (source, view, name) in [
        ("subject", View::Desktop, "desktop"),
        ("theme:nord", View::Browser, "browser"),
        ("tonal", View::Chat, "chat"),
        ("gamut", View::Music, "music"),
        ("material:content", View::Code, "code"),
    ] {
        render(&studio(&image, source, view), &format!("{out}/{name}.png"));
    }
    let mut open = studio(&image, "subject", View::Desktop);
    open.update(&StudioMsg::Hover(Some(Hit::Tab(Row::Colour))));
    let mut left = SETTLE;
    while left > 0.0 {
        open.tick(TICK);
        left -= TICK;
    }
    render(&open, &format!("{out}/popover.png"));
}
