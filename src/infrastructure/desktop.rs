use std::path::{Path, PathBuf};
use std::time::Duration;

use paper_control::{Background, DrawingManifest};
use paper_geom::{FillMode, Raster};
use paper_widgets::{Frame, Moment, Outline};

const SECONDS_PER_MINUTE: u64 = 60;

pub struct Request {
    pub wallpaper: Option<PathBuf>,
    pub fill: FillMode,
    pub background: Background,
    pub size: [u32; 2],
    pub outlines: Vec<PathBuf>,
    pub font: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Backdrop {
    pub size: [u32; 2],
    pub rgba: Vec<u8>,
    pub matte: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct Desktop {
    pub backdrop: Option<Backdrop>,
    pub font: Vec<u8>,
}

pub fn load(request: &Request) -> Desktop {
    let backdrop = request.wallpaper.as_deref().and_then(|path| backdrop(request, path));
    let font = request.font.as_deref().map_or_else(
        || paper_widgets::FONT.to_vec(),
        |path| {
            paper_widgets::read_font(path).unwrap_or_else(|error| {
                log::warn!("widgets: font {} unreadable: {error}", path.display());
                paper_widgets::FONT.to_vec()
            })
        },
    );
    Desktop { backdrop, font }
}

fn backdrop(request: &Request, path: &Path) -> Option<Backdrop> {
    if request.fill == FillMode::Span || request.size.contains(&0) {
        return None;
    }
    let picture = match image::open(path) {
        Ok(picture) => picture.into_rgba8(),
        Err(error) => {
            log::warn!("widgets: backdrop {} unreadable: {error}", path.display());
            return None;
        }
    };
    let (width, height) = picture.dimensions();
    let source = Raster { width, height, pixels: picture.as_raw() };
    let (screen, high, rgba) =
        paper_geom::compose(source, request.size, request.fill, request.background);
    drop(picture);
    let size = [screen, high];
    let matte = request
        .outlines
        .iter()
        .find(|manifest| DrawingManifest::describes(manifest, path))
        .and_then(|manifest| {
            Outline::load(manifest, path)
                .inspect_err(|error| {
                    log::warn!("widgets: no outline for {}: {error}", path.display());
                })
                .ok()
        })
        .and_then(|outline| {
            let frame = Frame { image: [width, height], buffer: size, mode: request.fill };
            outline.crop(frame, [0, 0], size.map(|side| side as usize))
        });
    Some(Backdrop { size, rgba, matte })
}

pub fn fill_mode(name: &str) -> FillMode {
    name.parse().unwrap_or_default()
}

pub fn moment() -> (Moment, Duration) {
    let elapsed =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let seconds = elapsed.as_secs();
    let local = skwd_log::local_time(seconds as i64);
    let moment =
        Moment::civil(local.tm_hour, local.tm_min, local.tm_wday, local.tm_mday, local.tm_mon + 1);
    let wait = Duration::from_secs(SECONDS_PER_MINUTE - seconds % SECONDS_PER_MINUTE);
    (moment, wait)
}

#[cfg(test)]
#[path = "desktop_tests.rs"]
mod tests;
