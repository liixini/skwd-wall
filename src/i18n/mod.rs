mod catalog;

#[cfg(test)]
pub(crate) use catalog::Catalog;
pub use catalog::{
    audio_live_mix_summary, audio_outputs_summary, browser_downloading_count,
    browser_downloading_queued, browser_masthead, browser_queued_count, browser_results_page,
    card_back_applied_times, effects_apply_count, effects_of_total_displays, format,
    language_choice, playlists_active_assignments, playlists_count_detail, playlists_library_stats,
    playlists_state_ready, schedule_group_all_detail, schedule_group_any_detail,
    schedule_group_summary, set_language, settings_keybind_conflict, settings_sand_meter_detail,
    status_diagnostics_issues, status_diagnostics_passed, tags_selected_count, tags_wall_count,
    theme_designer_custom_count, theme_designer_index_subtitle, theme_designer_stats, tr,
};

macro_rules! tr_args {
    ($key:literal $(, $name:ident => $value:expr)+ $(,)?) => {{
        let mut args = ::fluent::FluentArgs::new();
        $(args.set(stringify!($name), $value);)+
        $crate::i18n::format($key, &args)
    }};
}
pub(crate) use tr_args;
