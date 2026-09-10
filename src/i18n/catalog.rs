use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{LazyLock, Mutex};

use fluent::{FluentArgs, FluentResource};

type Bundle = fluent::concurrent::FluentBundle<FluentResource>;

include!(concat!(env!("OUT_DIR"), "/embedded_locales.rs"));

pub struct Catalog {
    bundle: Bundle,
    static_text: Mutex<HashMap<&'static str, &'static str>>,
}

impl Catalog {
    pub(crate) fn for_locale(requested: &str) -> Self {
        let (locale, override_resources) = match normalized_locale(requested).unwrap_or("en-US") {
            "sv-SE" => ("sv-SE", SV_SE_RESOURCES),
            "es-ES" => ("es-ES", ES_ES_RESOURCES),
            _ => ("en-US", &[][..]),
        };
        let locale = locale.parse().expect("embedded locale identifier must be valid");
        let mut bundle = Bundle::new_concurrent(vec![locale]);
        bundle.set_use_isolating(false);
        for source in EN_US_RESOURCES {
            let resource =
                FluentResource::try_new((*source).to_owned()).unwrap_or_else(|(_, errors)| {
                    panic!("invalid embedded en-US translations: {errors:?}")
                });
            bundle.add_resource(resource).unwrap_or_else(|errors| {
                panic!("invalid embedded en-US translation bundle: {errors:?}")
            });
        }
        for source in override_resources {
            let resource =
                FluentResource::try_new((*source).to_owned()).unwrap_or_else(|(_, errors)| {
                    panic!("invalid embedded selected translations: {errors:?}")
                });
            bundle.add_resource_overriding(resource);
        }
        Self { bundle, static_text: Mutex::new(HashMap::new()) }
    }

    pub fn format(&self, key: &str, args: Option<&FluentArgs<'_>>) -> String {
        format_message(&self.bundle, key, args)
    }

    fn text(&self, key: &'static str) -> &'static str {
        let mut cache = self.static_text.lock().expect("translation cache poisoned");
        if let Some(value) = cache.get(key) {
            return value;
        }
        let value = Box::leak(self.format(key, None).into_boxed_str());
        cache.insert(key, value);
        value
    }
}

fn normalized_locale(raw: &str) -> Option<&'static str> {
    let base = raw.trim().split(['.', '@']).next().unwrap_or_default();
    let language = base.split(['-', '_']).next().unwrap_or_default();
    match language.to_ascii_lowercase().as_str() {
        "es" => Some("es-ES"),
        "sv" => Some("sv-SE"),
        "en" | "c" | "posix" => Some("en-US"),
        _ => None,
    }
}

fn selected_locale(values: [Option<&str>; 5]) -> &'static str {
    let [explicit, all, messages, lang, languages] =
        values.map(|value| value.map(str::trim).filter(|value| !value.is_empty()));
    if let Some(explicit) = explicit {
        return normalized_locale(explicit).unwrap_or("en-US");
    }
    let system = all.or(messages).or(lang).unwrap_or("C");
    let base = system.split(['.', '@']).next().unwrap_or_default();
    if base.eq_ignore_ascii_case("C") || base.eq_ignore_ascii_case("POSIX") {
        return "en-US";
    }
    languages
        .and_then(|list| list.split(':').find_map(normalized_locale))
        .or_else(|| normalized_locale(system))
        .unwrap_or("en-US")
}

fn format_message(bundle: &Bundle, key: &str, args: Option<&FluentArgs<'_>>) -> String {
    let message = bundle.get_message(key).unwrap_or_else(|| panic!("missing translation: {key}"));
    let pattern = message.value().unwrap_or_else(|| panic!("translation has no value: {key}"));
    let mut errors = Vec::new();
    let value = bundle.format_pattern(pattern, args, &mut errors);
    assert!(errors.is_empty(), "could not format translation {key}: {errors:?}");
    value.into_owned()
}

static SELECTED: AtomicU8 = AtomicU8::new(0);
static AUTOMATIC: LazyLock<u8> = LazyLock::new(|| {
    let vars = ["SKWD_WALL_LOCALE", "LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"]
        .map(|key| std::env::var(key).ok());
    language_index(selected_locale(vars.each_ref().map(|value| value.as_deref())))
});
static ENGLISH: LazyLock<Catalog> = LazyLock::new(|| Catalog::for_locale("en-US"));
static SWEDISH: LazyLock<Catalog> = LazyLock::new(|| Catalog::for_locale("sv-SE"));
static SPANISH: LazyLock<Catalog> = LazyLock::new(|| Catalog::for_locale("es-ES"));

pub fn language_choice(requested: &str) -> &'static str {
    normalized_locale(requested).unwrap_or("auto")
}

fn language_index(requested: &str) -> u8 {
    match language_choice(requested) {
        "en-US" => 1,
        "sv-SE" => 2,
        "es-ES" => 3,
        _ => 0,
    }
}

pub fn set_language(requested: &str) {
    SELECTED.store(language_index(requested), Ordering::Relaxed);
}

pub fn catalog() -> &'static Catalog {
    let selected = SELECTED.load(Ordering::Relaxed);
    match if selected == 0 { *AUTOMATIC } else { selected } {
        2 => &SWEDISH,
        3 => &SPANISH,
        _ => &ENGLISH,
    }
}

pub fn format(key: &str, args: &FluentArgs<'_>) -> String {
    catalog().format(key, Some(args))
}

pub fn tr(key: &'static str) -> &'static str {
    catalog().text(key)
}

pub fn settings_keybind_conflict(first: &str, second: &str, binding: &str) -> String {
    let mut args = FluentArgs::new();
    args.set("first", first);
    args.set("second", second);
    args.set("binding", binding);
    catalog().format("settings-keybinds-conflict-desc", Some(&args))
}

pub fn settings_sand_meter_detail(grain: u32, card: &str, tier: &str, note: &str) -> String {
    let mut args = FluentArgs::new();
    args.set("grain", grain);
    args.set("card", card);
    args.set("tier", tier);
    args.set("note", note);
    catalog().format("settings-selector-sand-meter-desc", Some(&args))
}

pub fn schedule_group_summary(operator: &str, count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("operator", operator);
    args.set("count", count);
    catalog().format("schedule-summary-group", Some(&args))
}

pub fn schedule_group_all_detail(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    catalog().format("schedule-group-all-detail", Some(&args))
}

pub fn schedule_group_any_detail(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    catalog().format("schedule-group-any-detail", Some(&args))
}

pub fn playlists_count_detail(kind: &str, count: i64) -> String {
    let mut args = FluentArgs::new();
    args.set("kind", kind);
    args.set("count", count);
    catalog().format("playlists-index-count-detail", Some(&args))
}

pub fn playlists_active_assignments(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    catalog().format("playlists-active-assignments", Some(&args))
}

pub fn playlists_library_stats(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    catalog().format("playlists-library-stats", Some(&args))
}

pub fn playlists_state_ready(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    args.set("padded", format!("{count:02}"));
    catalog().format("playlists-state-ready", Some(&args))
}

pub fn theme_designer_index_subtitle(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    catalog().format("theme-designer-index-subtitle", Some(&args))
}

pub fn theme_designer_custom_count(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    catalog().format("theme-designer-custom-count", Some(&args))
}

pub fn browser_downloading_count(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count.to_string());
    catalog().format("browser-downloading-count", Some(&args))
}

pub fn browser_queued_count(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count.to_string());
    catalog().format("browser-queued-count", Some(&args))
}

pub fn browser_downloading_queued(active: usize, queued: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("active", active.to_string());
    args.set("queued", queued.to_string());
    catalog().format("browser-downloading-queued", Some(&args))
}

pub fn browser_results_page(page: u32) -> String {
    let mut args = FluentArgs::new();
    args.set("page", format!("{page:02}"));
    catalog().format("browser-results-page", Some(&args))
}

pub fn browser_masthead(source: &str, results: usize, page: u32) -> String {
    let mut args = FluentArgs::new();
    args.set("source", source);
    args.set("results", results);
    args.set("page", page.to_string());
    catalog().format("browser-masthead", Some(&args))
}

pub fn effects_of_total_displays(total: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("total", total);
    catalog().format("effects-of-total-displays", Some(&args))
}

pub fn effects_apply_count(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    catalog().format("effects-apply-count", Some(&args))
}

pub fn audio_live_mix_summary(audible: usize, available: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("audible", audible);
    args.set("available", available);
    catalog().format("audio-live-mix-summary", Some(&args))
}

pub fn audio_outputs_summary(total: usize, available: usize, sounding: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("total", total);
    args.set("available", available);
    args.set("sounding", sounding);
    catalog().format("audio-outputs-summary", Some(&args))
}

pub fn tags_wall_count(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    catalog().format("tags-wall-count", Some(&args))
}

pub fn tags_selected_count(count: usize) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    catalog().format("tags-selected-count", Some(&args))
}

pub fn card_back_applied_times(count: i64) -> String {
    let mut args = FluentArgs::new();
    args.set("count", count);
    catalog().format("card-back-applied-times", Some(&args))
}

#[cfg(test)]
mod tests;
