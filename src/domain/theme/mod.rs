mod candidate;

pub use candidate::{
    Candidate, HoverColors, THEME_ROLE_COUNT, ThemeRole, hex_to_hsv, hsv_to_hex, hsv_to_rgb,
};

#[cfg(test)]
mod tests;
