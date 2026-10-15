use std::ffi::{CStr, CString, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use fontconfig::{self, Fontconfig, ObjectSet, Pattern};
use fontconfig_sys as sys;
use iced_wgpu::graphics::text::cosmic_text::fontdb::{
    Database, FaceInfo, ID, Language, Source, Stretch, Style, Weight,
};

const FACE_INDEX_MASK: i32 = 0xffff;
const NO_FACES: &str = "fontconfig returned no usable faces";
const WEIGHT_STOPS: &[(i32, u16)] = &[
    (fontconfig::FC_WEIGHT_THIN, 100),
    (fontconfig::FC_WEIGHT_EXTRALIGHT, 200),
    (fontconfig::FC_WEIGHT_LIGHT, 300),
    (fontconfig::FC_WEIGHT_BOOK, 380),
    (fontconfig::FC_WEIGHT_REGULAR, 400),
    (fontconfig::FC_WEIGHT_MEDIUM, 500),
    (fontconfig::FC_WEIGHT_DEMIBOLD, 600),
    (fontconfig::FC_WEIGHT_BOLD, 700),
    (fontconfig::FC_WEIGHT_EXTRABOLD, 800),
    (fontconfig::FC_WEIGHT_BLACK, 900),
    (fontconfig::FC_WEIGHT_EXTRABLACK, 1000),
];
const WIDTH_STOPS: &[(i32, Stretch)] = &[
    (fontconfig::FC_WIDTH_ULTRACONDENSED, Stretch::UltraCondensed),
    (fontconfig::FC_WIDTH_EXTRACONDENSED, Stretch::ExtraCondensed),
    (fontconfig::FC_WIDTH_CONDENSED, Stretch::Condensed),
    (fontconfig::FC_WIDTH_SEMICONDENSED, Stretch::SemiCondensed),
    (fontconfig::FC_WIDTH_NORMAL, Stretch::Normal),
    (fontconfig::FC_WIDTH_SEMIEXPANDED, Stretch::SemiExpanded),
    (fontconfig::FC_WIDTH_EXPANDED, Stretch::Expanded),
    (fontconfig::FC_WIDTH_EXTRAEXPANDED, Stretch::ExtraExpanded),
    (fontconfig::FC_WIDTH_ULTRAEXPANDED, Stretch::UltraExpanded),
];
const GENERIC_FAMILIES: &[(Generic, &str)] = &[
    (Generic::Serif, "serif"),
    (Generic::SansSerif, "sans-serif"),
    (Generic::Cursive, "cursive"),
    (Generic::Fantasy, "fantasy"),
    (Generic::Monospace, "monospace"),
];

#[derive(Clone, Copy)]
enum Generic {
    Serif,
    SansSerif,
    Cursive,
    Fantasy,
    Monospace,
}

pub(super) fn database() -> Result<Database, String> {
    let config = Fontconfig::new().ok_or("fontconfig unavailable")?;
    let pattern = Pattern::new(&config).map_err(|err| err.to_string())?;
    let mut objects = ObjectSet::new(&config).map_err(|err| err.to_string())?;
    for key in [
        fontconfig::FC_FILE,
        fontconfig::FC_INDEX,
        fontconfig::FC_FAMILY,
        fontconfig::FC_POSTSCRIPT_NAME,
        fontconfig::FC_SLANT,
        fontconfig::FC_WEIGHT,
        fontconfig::FC_WIDTH,
        fontconfig::FC_SPACING,
    ] {
        objects.add(key).map_err(|err| err.to_string())?;
    }
    let fonts = fontconfig::list_fonts(&pattern, Some(&objects)).map_err(|err| err.to_string())?;
    let mut catalog = Database::new();
    for pattern in fonts.iter() {
        if let Some(info) = face(&pattern) {
            catalog.push_face_info(info);
        }
    }
    if catalog.is_empty() {
        return Err(String::from(NO_FACES));
    }
    for (generic, family) in GENERIC_FAMILIES.iter().filter_map(|(kind, generic)| {
        let mut pattern = Pattern::new(&config).ok()?;
        pattern.add_string(fontconfig::FC_FAMILY, &CString::new(*generic).ok()?).ok()?;
        let matched = pattern.font_match().ok()?;
        Some((*kind, matched.get_string(fontconfig::FC_FAMILY).ok()?.to_owned()))
    }) {
        match generic {
            Generic::Serif => catalog.set_serif_family(family),
            Generic::SansSerif => catalog.set_sans_serif_family(family),
            Generic::Cursive => catalog.set_cursive_family(family),
            Generic::Fantasy => catalog.set_fantasy_family(family),
            Generic::Monospace => catalog.set_monospace_family(family),
        }
    }
    Ok(catalog)
}

fn face(pattern: &Pattern<'_>) -> Option<FaceInfo> {
    let index = pattern.face_index().ok()?;
    if index < 0 || index & !FACE_INDEX_MASK != 0 {
        return None;
    }
    let families: Vec<_> = (0..)
        .map_while(|index| indexed_string(pattern, fontconfig::FC_FAMILY, index))
        .filter_map(|name| name.to_str().ok())
        .map(|name| (name.to_owned(), Language::English_UnitedStates))
        .collect();
    if families.is_empty() {
        return None;
    }
    let path = PathBuf::from(OsStr::from_bytes(
        indexed_string(pattern, fontconfig::FC_FILE, 0)?.to_bytes(),
    ));
    let style = match pattern.slant().unwrap_or(fontconfig::FC_SLANT_ROMAN) {
        fontconfig::FC_SLANT_ITALIC => Style::Italic,
        fontconfig::FC_SLANT_OBLIQUE => Style::Oblique,
        _ => Style::Normal,
    };
    let weight = weight(pattern.weight().unwrap_or(fontconfig::FC_WEIGHT_REGULAR));
    let stretch = stretch(pattern.width().unwrap_or(fontconfig::FC_WIDTH_NORMAL));
    let monospaced = matches!(
        pattern.get_int(fontconfig::FC_SPACING),
        Ok(fontconfig::FC_MONO | fontconfig::FC_CHARCELL)
    );
    Some(FaceInfo {
        id: ID::dummy(),
        source: Source::File(path),
        index: index as u32,
        families,
        post_script_name: indexed_string(pattern, fontconfig::FC_POSTSCRIPT_NAME, 0)
            .and_then(|name| name.to_str().ok())
            .unwrap_or_default()
            .to_owned(),
        style,
        weight,
        stretch,
        monospaced,
    })
}

fn indexed_string<'a>(pattern: &'a Pattern<'_>, key: &CStr, index: i32) -> Option<&'a CStr> {
    let mut raw = std::ptr::null_mut();
    let result = unsafe {
        (sys::statics::LIB.FcPatternGetString)(
            pattern.as_ptr().cast_mut(),
            key.as_ptr(),
            index,
            &raw mut raw,
        )
    };
    if result != sys::FcResultMatch || raw.is_null() {
        return None;
    }
    Some(unsafe { CStr::from_ptr(raw.cast()) })
}

fn weight(value: i32) -> Weight {
    let (min, low) = WEIGHT_STOPS[0];
    if value <= min {
        return Weight(low);
    }
    let mut previous = (min, low);
    for &(stop, high) in WEIGHT_STOPS.iter().skip(1) {
        if value <= stop {
            let (start, low) = previous;
            let interpolated = i32::from(low)
                + (value - start) * (i32::from(high) - i32::from(low)) / (stop - start);
            return Weight(interpolated as u16);
        }
        previous = (stop, high);
    }
    Weight(previous.1)
}

fn stretch(value: i32) -> Stretch {
    WIDTH_STOPS
        .iter()
        .min_by_key(|(stop, _)| stop.abs_diff(value))
        .map_or(Stretch::Normal, |(_, stretch)| *stretch)
}

#[cfg(test)]
mod tests;
