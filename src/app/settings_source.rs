use crate::contracts::settings::{GraphicsCard, SettingsSource};
use crate::domain::input::InputMap;
use crate::infrastructure::config::Config;

const DARK_MODE: &str = "dark";
const LIGHT_MODE: &str = "light";

impl SettingsSource for Config {
    fn default_semantic_model_available(&self) -> bool {
        crate::infrastructure::semantic::SemanticPaths::manifest(&self.cache_dir(), "")
            .is_ok_and(|path| path.is_file())
    }

    fn flag(&self, path: &str) -> bool {
        self.flag_default_config(path)
    }

    fn flag_default_true(&self, path: &str) -> bool {
        Config::flag_default_true(self, path)
    }

    fn text(&self, path: &str) -> String {
        if let Some(index) = path
            .strip_prefix(skwd_config::keys::library::FOLDERS)
            .and_then(|suffix| suffix.strip_prefix('.'))
            .and_then(|index| index.parse::<usize>().ok())
        {
            return self.library_folders().get(index).cloned().unwrap_or_default();
        }
        if path == skwd_config::keys::paper::ENGINE {
            return skwd_config::paper_engine(self.root());
        }
        if path == skwd_config::keys::filter_bar::DEFAULT_FOLDER {
            return self.default_folder();
        }
        let value = self.str_path(path);
        if path == skwd_config::keys::theme::MODE && value.is_empty() {
            self.str_path(skwd_config::keys::matugen::MODE)
        } else {
            value
        }
    }

    fn number(&self, path: &str) -> f64 {
        self.num_path(path)
    }

    fn array_len(&self, path: &str) -> usize {
        Config::array_len(self, path)
    }

    fn library_folders(&self) -> Vec<String> {
        skwd_config::get(self.root(), skwd_config::keys::library::FOLDERS)
            .and_then(serde_json::Value::as_array)
            .map_or_else(
                || skwd_config::library_folders(self.root()),
                |folders| {
                    folders.iter().filter_map(|folder| folder.as_str().map(str::to_owned)).collect()
                },
            )
    }

    fn exclusion_scopes(&self) -> Vec<crate::contracts::settings::ExclusionScope> {
        use crate::contracts::settings::ExclusionScope;
        let roots = skwd_config::library_folders(self.root());
        skwd_config::excluded_folders(self.root())
            .iter()
            .map(|entry| {
                if entry.trim().is_empty() {
                    return ExclusionScope::Empty;
                }
                let path = std::path::PathBuf::from(skwd_config::resolve(entry));
                if !path.is_absolute() {
                    return ExclusionScope::Everywhere;
                }
                let Some(path) = skwd_config::resolve_folder(&path) else {
                    return ExclusionScope::Outside;
                };
                roots
                    .iter()
                    .filter(|root| path.starts_with(root))
                    .max_by_key(|root| root.len())
                    .map_or(ExclusionScope::Outside, |root| ExclusionScope::Inside(root.clone()))
            })
            .collect()
    }

    fn resolved_folder(&self, folder: &str) -> String {
        resolved_folder(folder)
    }

    fn profile_states(
        &self,
        index: usize,
        folders: &[String],
    ) -> crate::contracts::settings::ProfileStates {
        let profile = skwd_config::arr_ref(self.root(), skwd_config::keys::library::PROFILES)
            .get(index)
            .and_then(skwd_config::LibraryProfile::from_value);
        let state = |item| skwd_config::profile_state(profile.as_ref(), item);
        let states = folders
            .iter()
            .map(|folder| state(skwd_config::LibraryItem::Folder(std::path::Path::new(folder))))
            .collect();
        (states, state(skwd_config::LibraryItem::Workshop))
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
            self.array_slice(skwd_config::keys::theme::SAVED_THEMES),
        )
    }

    fn palette_colors(&self) -> Vec<(String, Vec<String>)> {
        let mode = skwd_config::theme_mode(self.root());
        let dark = match mode.as_str() {
            DARK_MODE => Some(true),
            LIGHT_MODE => Some(false),
            _ => None,
        };
        let mut palettes: Vec<_> = skwd_palette::PRESETS
            .iter()
            .filter_map(|(key, _)| {
                crate::domain::theme::Candidate::from_preset(key)
                    .map(|palette| (key.to_string(), palette.preview_colors()))
            })
            .collect();
        palettes.extend(
            crate::infrastructure::theme::saved_palettes(
                self.array_slice(skwd_config::keys::theme::SAVED_THEMES),
                dark,
            )
            .into_iter()
            .map(|(name, palette)| (name, palette.preview_colors())),
        );
        palettes
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

pub(in crate::app) fn resolved_folder(folder: &str) -> String {
    let resolved = skwd_config::resolve(folder);
    skwd_config::resolve_folder(std::path::Path::new(&resolved))
        .map_or(resolved, |path| path.to_string_lossy().into_owned())
}
