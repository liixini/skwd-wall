use crate::contracts::settings::{GraphicsCard, SettingsSource};
use crate::domain::input::InputMap;
use crate::infrastructure::config::Config;

impl SettingsSource for Config {
    fn flag(&self, path: &str) -> bool {
        self.flag_default_config(path)
    }

    fn flag_default_true(&self, path: &str) -> bool {
        Config::flag_default_true(self, path)
    }

    fn text(&self, path: &str) -> String {
        if path == skwd_config::keys::paper::ENGINE {
            return skwd_config::paper_engine(self.root());
        }
        self.str_path(path)
    }

    fn number(&self, path: &str) -> f64 {
        self.num_path(path)
    }

    fn array_len(&self, path: &str) -> usize {
        Config::array_len(self, path)
    }

    fn display_mode(&self) -> String {
        Config::display_mode(self)
    }

    fn selector_mode(&self) -> String {
        Config::selector_mode(self)
    }

    fn selector_preset_names(&self, mode: &str) -> Vec<String> {
        self.selector_presets(mode).into_iter().map(|(name, _)| name).collect()
    }

    fn selected_preset(&self, mode: &str) -> Option<String> {
        Config::selected_preset(self, mode)
    }

    fn sandy_grain(&self) -> f32 {
        Config::sandy_grain(self)
    }

    fn sandy_swap_style(&self) -> String {
        Config::sandy_swap_style(self)
    }

    fn launch_animation(&self) -> String {
        Config::launch_animation(self)
    }

    fn theme_backend(&self) -> String {
        Config::theme_backend(self)
    }

    fn motion_fast_ms(&self) -> f32 {
        Config::motion_fast_ms(self)
    }

    fn motion_standard_ms(&self) -> f32 {
        Config::motion_standard_ms(self)
    }

    fn motion_slow_ms(&self) -> f32 {
        Config::motion_slow_ms(self)
    }

    fn saved_theme_names(&self) -> Vec<String> {
        crate::infrastructure::theme::saved_names(
            &self.array_values(skwd_config::keys::theme::SAVED_THEMES),
        )
    }

    fn palette_presets(&self) -> Vec<(String, String)> {
        skwd_palette::PRESETS
            .iter()
            .map(|(key, label)| ((*key).to_string(), (*label).to_string()))
            .collect()
    }

    fn bindings(&self) -> InputMap {
        crate::infrastructure::config::load_bindings(self)
    }

    fn graphics_card(&self) -> GraphicsCard {
        crate::infrastructure::capabilities::graphics_card(
            &crate::rendering::capabilities::WgpuGraphicsProbe,
        )
        .clone()
    }

    fn graphics_devices(&self) -> Vec<crate::contracts::capabilities::GraphicsDevice> {
        crate::infrastructure::capabilities::graphics_devices(
            &crate::rendering::capabilities::WgpuGraphicsProbe,
        )
        .to_vec()
    }

    fn is_niri(&self) -> bool {
        crate::infrastructure::runtime::is_niri()
    }

    fn on_battery_power(&self) -> bool {
        Config::on_battery_power(self)
    }
}
