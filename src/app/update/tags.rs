use iced::Task;

#[allow(clippy::wildcard_imports)]
use super::super::*;

use crate::frontend::tagcloud::TagMsg;

pub(super) fn update(app: &mut App, msg: TagMsg) -> Task<Message> {
    match msg {
        TagMsg::InputChanged(text) => tag_input_changed(app, text),
        TagMsg::Submit => {
            commit_pending_tag(app);
            iced::widget::operation::focus(tag_input_id())
        }
        TagMsg::Remove(index) => remove_card_tag(app, index),
        TagMsg::ToggleCardDrawer => {
            toggle_card_tag_drawer(app);
            Task::none()
        }
        TagMsg::CloseCardDrawer => {
            if app.tags.card_drawer_open {
                toggle_card_tag_drawer(app);
            }
            Task::none()
        }
        TagMsg::QueryInput(text) => tag_query_input(app, text),
        TagMsg::SearchMode(mode) => set_search_mode(app, mode),
        TagMsg::CycleSearchMode => {
            let next = match app.tags.search_mode {
                SearchMode::Tags => SearchMode::Describe,
                SearchMode::Describe => SearchMode::Tags,
            };
            set_search_mode(app, next)
        }
        TagMsg::CloudClick(tag, right) => tag_cloud_click(app, tag, right),
        TagMsg::CloudScroll(pos) => {
            app.tags.cloud_scroll.retarget(pos.max(0.0));
            app.retick();
            Task::none()
        }
        TagMsg::Autocomplete => tag_autocomplete(app),
        TagMsg::MatchMode(match_any) => set_match_mode(app, match_any),
        TagMsg::SortAz(sort_az) => set_tag_sort(app, sort_az),
        TagMsg::ToggleMatchingTags => toggle_matching_tags(app),
    }
}

fn remove_card_tag(app: &mut App, index: usize) -> Task<Message> {
    let Some(key) = app.tags.card_key.clone() else { return Task::none() };
    remove_tag_at(app, &key, index);
    Task::none()
}

pub(super) fn tag_select_click(app: &mut App, hit: Option<usize>) -> Task<Message> {
    if let Some(fi) = hit
        && let Some(&si) = app.library_session.filtered.get(fi)
    {
        if !app.tags.select.remove(&si) {
            app.tags.select.insert(si);
        }
        app.retick();
    }
    Task::none()
}

pub(super) fn tag_input_changed(app: &mut App, s: String) -> Task<Message> {
    app.tags.input = s;
    sync_card_tag_input(app, false);
    Task::none()
}

pub(super) fn toggle_tag_mode(app: &mut App) -> Task<Message> {
    if app.menu_capturing() {
        return Task::none();
    }
    app.tags.mode = !app.tags.mode;
    if !app.tags.mode {
        app.tags.select.clear();
        app.tags.mass_tags.clear();
        app.tags.mass_input.clear();
    }
    app.chrome.bar.cache.clear();
    app.retick();
    Task::none()
}

pub(super) fn mass_tag_input(app: &mut App, s: String) -> Task<Message> {
    app.tags.mass_input = s;
    if app.tags.mass_input.chars().last().is_some_and(char::is_whitespace) {
        let tag = app.tags.mass_input.trim().to_lowercase();
        app.tags.mass_input.clear();
        if !tag.is_empty() && !app.tags.mass_tags.contains(&tag) {
            app.tags.mass_tags.push(tag);
        }
    }
    Task::none()
}

pub(super) fn mass_tag_add(app: &mut App, tag: &str) -> Task<Message> {
    let tag = tag.trim().to_lowercase();
    if !tag.is_empty() && !app.tags.mass_tags.contains(&tag) {
        app.tags.mass_tags.push(tag);
    }
    app.tags.mass_input.clear();
    iced::widget::operation::focus(mass_tag_input_id())
}

pub(super) fn mass_tag_remove(app: &mut App, i: usize) -> Task<Message> {
    if i < app.tags.mass_tags.len() {
        app.tags.mass_tags.remove(i);
    }
    Task::none()
}

pub(super) fn open_tag_cloud(app: &mut App) -> Task<Message> {
    if app.menu_capturing() {
        return Task::none();
    }
    app.tags.cloud_open = !app.tags.cloud_open;
    app.tags.matching_tags_open = false;
    app.tags.tag_search.clear();
    app.tags.cloud_scroll.snap(0.0);
    app.chrome.bar.cache.clear();
    if !app.tags.cloud_open {
        app.tags.semantic.search.clear();
        app.clear_semantic_search();
        return Task::none();
    }
    app.tags.cloud_entrance.run(0.0, 1.0);
    app.tags.search_mode = if app.config.flag_default_true(skwd_config::keys::semantic::ENABLED) {
        SearchMode::from_config(
            &app.config.str_path(skwd_config::keys::tagging::DEFAULT_SEARCH_MODE),
        )
    } else {
        SearchMode::Tags
    };
    if app.tags.search_mode == SearchMode::Tags {
        app.tags.tag_search = sync_library_search(
            &app.library_session.filters.tags,
            &app.library_session.filters.numeric,
        );
    }
    app.tags.semantic.search.clear();
    iced::widget::operation::focus(tag_query_id())
}

pub(super) fn tag_cloud_click(app: &mut App, tag: String, right: bool) -> Task<Message> {
    let was_inc = app.library_session.filters.tags.iter().any(|cur| cur == &tag);
    let neg = format!("-{tag}");
    let was_exc = app.library_session.filters.tags.iter().any(|cur| cur == &neg);
    app.library_session.filters.tags.retain(|cur| cur != &tag && cur != &neg);
    if right {
        if !was_exc {
            app.library_session.filters.tags.push(neg);
        }
    } else if !was_inc {
        app.library_session.filters.tags.push(tag);
    }
    app.tags.tag_search = sync_library_search(
        &app.library_session.filters.tags,
        &app.library_session.filters.numeric,
    );
    app.tags.matching_tags_open = !app.library_session.filters.tags.is_empty()
        || !app.library_session.filters.numeric.is_empty();
    app.refilter_from_start();
    app.chrome.bar.cache.clear();
    app.retick();
    iced::widget::operation::focus(tag_query_id())
}

pub(super) fn tag_autocomplete(app: &mut App) -> Task<Message> {
    if !app.tags.cloud_open {
        return tag_autocomplete_input(app);
    }
    if app.tags.search_mode == SearchMode::Tags {
        let tag = app
            .tag_cloud_entries()
            .iter()
            .find(|entry| !entry.selected && !entry.excluded)
            .map(|entry| entry.tag.clone());
        if let Some(tag) = tag {
            return tag_cloud_click(app, tag, false);
        }
    } else {
        app.request_semantic_search();
    }
    iced::widget::operation::focus(tag_query_id())
}

pub(super) fn tag_autocomplete_input(app: &mut App) -> Task<Message> {
    if app.tags.editing {
        return iced::widget::operation::focus(tag_input_id());
    }
    if app.menu_capturing() {
        return iced::widget::operation::focus_next();
    }
    Task::none()
}

fn tag_query_input(app: &mut App, text: String) -> Task<Message> {
    if app.tags.search_mode == SearchMode::Tags {
        let deleting = text.chars().count() < app.tags.tag_search.chars().count();
        let commits_token = text.chars().last().is_some_and(char::is_whitespace);
        app.tags.tag_search = text;
        let vocabulary: std::collections::HashSet<String> =
            app.library_session.library.catalog().tags.values().flatten().cloned().collect();
        let parsed: crate::domain::library::search::LibraryQuery =
            crate::domain::library::search::parse_library_search(
                &workshop_query(&app.tags.tag_search),
                &vocabulary,
            );
        let parsed_tags = if deleting || commits_token {
            parsed.tags
        } else {
            app.library_session.filters.tags.clone()
        };
        if parsed_tags != app.library_session.filters.tags
            || parsed.numeric != app.library_session.filters.numeric
        {
            app.library_session.filters.tags = parsed_tags;
            app.library_session.filters.numeric = parsed.numeric;
            app.refilter_from_start();
        }
        app.tags.matching_tags_open = !tag_search_partial(&app.tags.tag_search).is_empty()
            || !app.library_session.filters.tags.is_empty()
            || !app.library_session.filters.numeric.is_empty();
        *app.tags.cloud_cache.borrow_mut() = None;
    } else {
        app.tags.semantic.search = text;
        let numeric = crate::domain::library::search::parse_numeric_query(&workshop_query(
            &app.tags.semantic.search,
        ));
        if numeric.has_workshop() {
            app.clear_semantic_search();
            app.tags.semantic.ranked = app
                .library_session
                .library
                .catalog()
                .items
                .iter()
                .filter(|item| numeric.matches(item))
                .map(|item| item.key.clone())
                .collect();
            app.tags.semantic.resolved = true;
            app.refilter_semantic_from_start();
        } else {
            app.request_semantic_search();
        }
    }
    app.tags.cloud_scroll.snap(0.0);
    app.chrome.bar.cache.clear();
    app.retick();
    iced::widget::operation::focus(tag_query_id())
}

fn workshop_query(text: &str) -> String {
    if !text.trim().bytes().all(|byte| byte.is_ascii_digit())
        && let Some(id) = wall_proto::workshop_id(text)
    {
        format!("{}:{id}", crate::domain::library::search::WORKSHOP_KEY)
    } else {
        text.to_string()
    }
}

pub(super) fn clear_tags(app: &mut App) -> Task<Message> {
    app.library_session.filters.tags.clear();
    app.library_session.filters.numeric = crate::domain::library::search::NumericQuery::default();
    app.tags.tag_search.clear();
    app.refilter_from_start();
    app.retick();
    iced::widget::operation::focus(tag_query_id())
}

fn set_search_mode(app: &mut App, mode: SearchMode) -> Task<Message> {
    let mode = if app.config.flag_default_true(skwd_config::keys::semantic::ENABLED) {
        mode
    } else {
        SearchMode::Tags
    };
    if app.tags.search_mode == mode {
        return Task::none();
    }
    app.tags.search_mode = mode;
    app.tags.cloud_scroll.snap(0.0);
    app.tags.matching_tags_open =
        mode == SearchMode::Tags && !app.tags.tag_search.trim().is_empty();
    *app.tags.cloud_cache.borrow_mut() = None;
    if mode == SearchMode::Describe {
        app.request_semantic_search();
    } else {
        app.clear_semantic_search();
        app.tags.tag_search = sync_library_search(
            &app.library_session.filters.tags,
            &app.library_session.filters.numeric,
        );
        app.refilter_from_start();
    }
    app.chrome.bar.cache.clear();
    app.retick();
    iced::widget::operation::focus(tag_query_id())
}

fn set_match_mode(app: &mut App, match_any: bool) -> Task<Message> {
    if app.library_session.filters.tags_match_any != match_any {
        app.library_session.filters.tags_match_any = match_any;
        app.refilter_from_start();
        app.retick();
    }
    Task::none()
}

fn set_tag_sort(app: &mut App, sort_az: bool) -> Task<Message> {
    if app.tags.sort_az != sort_az {
        app.tags.sort_az = sort_az;
        app.retick();
    }
    Task::none()
}

fn toggle_matching_tags(app: &mut App) -> Task<Message> {
    app.tags.matching_tags_open = !app.tags.matching_tags_open;
    app.tags.cloud_scroll.snap(0.0);
    app.retick();
    Task::none()
}
