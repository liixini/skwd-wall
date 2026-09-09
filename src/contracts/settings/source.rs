use crate::domain::input::InputMap;

pub use crate::contracts::capabilities::{GraphicsCard, GraphicsTier};

pub trait SettingsSource {
    fn flag(&self, path: &str) -> bool;
    fn flag_default_true(&self, path: &str) -> bool;
    fn text(&self, path: &str) -> String;
    fn number(&self, path: &str) -> f64;
    fn array_len(&self, path: &str) -> usize;

    fn display_mode(&self) -> String;
    fn selector_mode(&self) -> String;
    fn selector_preset_names(&self, mode: &str) -> Vec<String>;
    fn selected_preset(&self, mode: &str) -> Option<String>;
    fn sandy_grain(&self) -> f32;
    fn sandy_swap_style(&self) -> String;
    fn launch_animation(&self) -> String;
    fn theme_backend(&self) -> String;

    fn motion_fast_ms(&self) -> f32;
    fn motion_standard_ms(&self) -> f32;
    fn motion_slow_ms(&self) -> f32;

    fn saved_theme_names(&self) -> Vec<String>;
    fn palette_colors(&self) -> Vec<(String, Vec<String>)>;
    fn palette_presets(&self) -> Vec<(String, String)>;
    fn bindings(&self) -> InputMap;
    fn graphics_card(&self) -> GraphicsCard;
    fn graphics_devices(&self) -> Vec<crate::contracts::capabilities::GraphicsDevice>;
    fn is_niri(&self) -> bool;
    fn on_battery_power(&self) -> bool;
}

#[cfg(test)]
mod tests;
