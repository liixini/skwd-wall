use std::hash::Hasher;

use crate::frontend::scene::{BackPanel, Chrome};
use crate::frontend::theme::Palette;

pub fn chrome_signature(chrome: &[Chrome], back: Option<&BackPanel>, palette: &Palette) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write_usize(chrome.len());
    for item in chrome {
        hasher.write_u8(item.view);
        for value in [
            item.cx,
            item.cy,
            item.hw,
            item.hh,
            item.skew,
            item.edge_tilt,
            item.radius,
            item.opacity,
        ] {
            hasher.write_u32(value.to_bits());
        }
        hasher.write_u8(item.kind);
        hasher.write_u8(item.has_video as u8);
        hasher.write_u8(item.favourite as u8);
    }
    match back {
        None => hasher.write_u8(0),
        Some(panel) => {
            hasher.write_u8(1);
            for value in [panel.cx, panel.cy, panel.hw, panel.hh, panel.skew, panel.edge_tilt] {
                hasher.write_u32(value.to_bits());
            }
            for radius in panel.radii {
                hasher.write_u32(radius.to_bits());
            }
            hasher.write_i32((panel.progress * 256.0) as i32);
            hasher.write_u8(panel.coordinated_flip as u8);
            hasher.write_u8(panel.embedded as u8);
            hasher.write_u8(panel.animate_flip_shader as u8);
            hasher.write_u8(panel.animate_flip_back as u8);
            hasher.write(panel.title.as_bytes());
            hasher.write(panel.kind_label.as_bytes());
            hasher.write_usize(panel.fields.len());
            for (key, value) in &panel.fields {
                hasher.write(key.as_bytes());
                hasher.write(value.as_bytes());
            }
            hasher.write_usize(panel.tags.len());
            for tag in &panel.tags {
                hasher.write(tag.as_bytes());
            }
            hasher.write_u8(panel.favourite as u8);
            hasher.write_i32((panel.fav_fill * 256.0) as i32);
            hasher.write_i32((panel.add_open * 256.0) as i32);
            hasher.write_i32((panel.chip_pop * 256.0) as i32);
            hasher.write_i32(panel.pop_idx);
            hasher.write_u8(panel.static_img as u8);
            hasher.write_u8(panel.overview_available as u8);
            hasher.write_u8(panel.reset_thumbnail as u8);
        }
    }
    for color in [
        palette.primary,
        palette.primary_text,
        palette.surface,
        palette.surface_text,
        palette.surface_variant,
        palette.surface_container,
        palette.background,
        palette.outline,
        palette.tertiary,
    ] {
        for value in [color.r, color.g, color.b, color.a] {
            hasher.write_u32(value.to_bits());
        }
    }
    hasher.finish()
}
