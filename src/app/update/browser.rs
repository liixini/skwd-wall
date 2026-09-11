use iced::Task;

#[allow(clippy::wildcard_imports)]
use super::super::*;

use crate::contracts::browser::{ApplyKind, ApplyTarget};
use crate::frontend::browser::BrowserMsg;
use crate::infrastructure::browser::RpcCall;

pub(super) fn update(app: &mut App, msg: BrowserMsg) -> Task<Message> {
    match msg {
        BrowserMsg::SwitchSource(source) => switch_browser_source(app, source),
        BrowserMsg::SearchInput(query) => browser_edit(app, false, |br| br.request.query = query),
        BrowserMsg::SearchSubmit => {
            if let Some(browser) = app.source_browser.browser.as_mut() {
                browser.session.submitted_search = Some(browser.search_request(1, ""));
            }
            app.run_browser_search(false);
            Task::none()
        }
        BrowserMsg::SetSort(sort) => browser_edit(app, true, |br| br.request.sorting = sort),
        BrowserMsg::SteamFilter(kind, value) => browser_edit(app, true, |br| match kind.as_str() {
            "type" => br.request.steam.kind = value,
            "res" => br.request.steam.resolution = value,
            "cat" => br.request.steam.category = value,
            "days" => br.request.steam.trend_days = value,
            "nsfw" => br.request.nsfw = value == "1",
            _ => {}
        }),
        BrowserMsg::CatalogFilter(kind, value) => {
            browser_edit(app, true, |br| match kind.as_str() {
                "unsplash-order" => br.request.catalog.unsplash.order_by = value,
                "unsplash-orientation" => br.request.catalog.unsplash.orientation = value,
                "unsplash-color" => br.request.catalog.unsplash.color = value,
                "unsplash-safety" => br.request.catalog.unsplash.content_filter = value,
                "pexels-orientation" => br.request.catalog.pexels.orientation = value,
                "pexels-size" => br.request.catalog.pexels.size = value,
                "pexels-color" => br.request.catalog.pexels.color = value,
                _ => {}
            })
        }
        BrowserMsg::SetTopRange(range) => {
            browser_edit(app, true, |br| br.request.wallhaven.top_range = range)
        }
        BrowserMsg::SetAtleast(res) => browser_edit(app, true, |br| {
            if res.is_empty() {
                br.request.wallhaven.atleast.clear();
                br.request.wallhaven.resolutions.clear();
            } else if br.request.wallhaven.res_exact {
                crate::frontend::browser::toggle_selected(
                    &mut br.request.wallhaven.resolutions,
                    &res,
                );
            } else {
                br.request.wallhaven.atleast =
                    if br.request.wallhaven.atleast == res { String::new() } else { res };
            }
        }),
        BrowserMsg::SetAtmost(res) => browser_edit(app, true, |br| {
            br.request.wallhaven.atmost = if res.is_empty() || br.request.wallhaven.atmost == res {
                String::new()
            } else {
                res
            };
        }),
        BrowserMsg::SetRatios(ratio) => browser_edit(app, true, |br| {
            crate::frontend::browser::toggle_selected(&mut br.request.wallhaven.ratios, &ratio);
        }),
        BrowserMsg::SetMaxDuration(dur) => {
            browser_edit(app, true, |br| br.request.catalog.max_duration = dur)
        }
        BrowserMsg::ToggleResExact => browser_edit(app, true, |br| {
            br.request.wallhaven.res_exact = !br.request.wallhaven.res_exact;
            br.request.wallhaven.atleast.clear();
            br.request.wallhaven.atmost.clear();
            br.request.wallhaven.resolutions.clear();
        }),
        BrowserMsg::ClipStart(text) => {
            browser_edit(app, false, |br| br.request.catalog.clip_start = text)
        }
        BrowserMsg::ClipLen(text) => {
            browser_edit(app, false, |br| br.request.catalog.clip_len = text)
        }
        BrowserMsg::SetCollection(id) => browser_edit(app, true, |br| {
            br.request.wallhaven.collection =
                if br.request.wallhaven.collection == id { String::new() } else { id };
        }),
        BrowserMsg::WallInput(input) => match input {
            crate::frontend::ui::BrowserWallInput::Pointer(x, y) => {
                browser_wall_pointer(app, x, y);
                Task::none()
            }
            crate::frontend::ui::BrowserWallInput::Click(x, y) => {
                browser_wall_pointer(app, x, y);
                browser_wall_click(app, x, y)
            }
            crate::frontend::ui::BrowserWallInput::Wheel(amount) => browser_wall_wheel(app, amount),
        },
        BrowserMsg::TogglePurity(idx) => browser_edit(app, true, |br| match idx {
            0 => br.request.wallhaven.sfw = !br.request.wallhaven.sfw,
            1 => br.request.wallhaven.sketchy = !br.request.wallhaven.sketchy,
            _ => br.request.nsfw = !br.request.nsfw,
        }),
        BrowserMsg::ToggleCategory(idx) => browser_edit(app, true, |br| match idx {
            0 => br.request.wallhaven.general = !br.request.wallhaven.general,
            1 => br.request.wallhaven.anime = !br.request.wallhaven.anime,
            _ => br.request.wallhaven.people = !br.request.wallhaven.people,
        }),
        BrowserMsg::SetColor(color) => {
            browser_edit(app, true, |br| br.request.wallhaven.color = color)
        }
        BrowserMsg::Apply(id) => browser_apply(app, id),
        BrowserMsg::OpenPreview(idx) => browser_open_preview(app, idx),
        BrowserMsg::ClosePreview => browser_close_preview(app),
        BrowserMsg::Download(id) => browser_download(app, id),
    }
}

pub(super) fn browser_apply_current(app: &mut App) -> Task<Message> {
    let Some(br) = app.source_browser.browser.as_ref() else {
        return Task::none();
    };
    let (preview, hover) = (br.session.preview, br.session.hover);
    if let Some(idx) = preview {
        let id = br.session.items.get(idx).map(|it| it.id.clone());
        return match id {
            Some(id) => update(app, BrowserMsg::Apply(id)),
            None => Task::none(),
        };
    }
    match hover {
        Some(idx) => update(app, BrowserMsg::OpenPreview(idx)),
        None => Task::none(),
    }
}

pub(super) fn open_browser(app: &mut App, source_key: &str) -> Task<Message> {
    app.close_settings();
    app.call_tracked("status", serde_json::json!({}), crate::app::state::Pending::Status);
    let requested = crate::frontend::browser::Source::from_key(source_key)
        .unwrap_or(crate::frontend::browser::Source::Wallhaven);
    let availability = crate::infrastructure::browser::source_availability(
        &app.config,
        app.daemon.steam_helper_available,
    );
    let source = availability
        .iter()
        .find(|entry| entry.source == requested && entry.unavailable.is_none())
        .or_else(|| availability.iter().find(|entry| entry.unavailable.is_none()))
        .map_or(crate::frontend::browser::Source::Wallhaven, |entry| entry.source);
    let opening = app.source_browser.browser.is_none();
    app.source_browser.wall.set_layout(browser_wall_params(&app.config));
    app.source_browser.activate(source);
    if opening {
        app.source_browser.entrance.run(0.0, 1.0);
    }
    adopt_active_source(app);
    app.retick();
    Task::none()
}

fn switch_browser_source(app: &mut App, source: crate::frontend::browser::Source) -> Task<Message> {
    let available = crate::infrastructure::browser::source_availability(
        &app.config,
        app.daemon.steam_helper_available,
    )
    .into_iter()
    .any(|entry| entry.source == source && entry.unavailable.is_none());
    if !available || !app.source_browser.activate(source) {
        return Task::none();
    }
    adopt_active_source(app);
    app.retick();
    Task::none()
}

fn adopt_active_source(app: &mut App) {
    app.source_browser.wall.begin_session();
    if let Some(browser) = app.source_browser.browser.as_ref() {
        app.source_browser.wall.rebuild_catalogue(browser, true);
    }
    let (source, needs_collections, needs_search) = app.source_browser.browser.as_ref().map_or(
        (crate::frontend::browser::Source::Wallhaven, false, false),
        |browser| {
            (
                browser.source,
                browser.source == crate::frontend::browser::Source::Wallhaven
                    && browser.request.wallhaven.collections.is_empty(),
                browser.session.items.is_empty() && !browser.session.loading,
            )
        },
    );
    if needs_collections {
        app.call_tracked(
            "wallhaven.collections",
            serde_json::json!({}),
            Pending::BrowserCollections { source },
        );
    }
    if needs_search {
        app.run_browser_search(false);
    }
}

pub(super) fn close_browser(app: &mut App) -> Task<Message> {
    app.clear_browser_previews();
    app.source_browser.close();
    Task::none()
}

pub(super) fn browser_edit(
    app: &mut App,
    search: bool,
    func: impl FnOnce(&mut crate::frontend::browser::Browser),
) -> Task<Message> {
    if let Some(br) = app.source_browser.browser.as_mut() {
        func(br);
    }
    if search
        && app
            .source_browser
            .browser
            .as_ref()
            .is_some_and(|browser| !app.config.browser_apply_button(browser.source))
    {
        app.run_browser_search(false);
    }
    Task::none()
}

pub(super) fn browser_wall_pointer(app: &mut App, x: f32, y: f32) {
    let hit = app
        .source_browser
        .wall
        .scene
        .render
        .hits
        .iter()
        .find(|hit: &&crate::frontend::scene::layout::Hit| hit.contains(x, y))
        .map(|hit| hit.index);
    if app.source_browser.wall.scene.hover != hit {
        app.source_browser.wall.scene.hover = hit;
        app.source_browser.wall.scene.touch();
        app.source_browser.wall.chrome_cache.clear();
        browser_hover(app, hit);
    }
}

pub(super) fn browser_wall_click(app: &mut App, x: f32, y: f32) -> Task<Message> {
    let Some(hit) =
        app.source_browser.wall.scene.render.hits.iter().find(|hit| hit.contains(x, y)).copied()
    else {
        return Task::none();
    };
    let action =
        crate::frontend::browser::browser_card_action_at(&hit, x, y, app.config.ui_scale());
    let item = app.source_browser.browser.as_ref().and_then(|browser| {
        browser.session.items.get(hit.index).map(|item| {
            (
                item.id.clone(),
                item.downloaded,
                item.downloading,
                browser.session.pending_apply.as_deref() == Some(item.id.as_str()),
            )
        })
    });
    match (action, item) {
        (Some(crate::frontend::browser::BrowserCardAction::Save), Some((id, false, false, _))) => {
            update(app, BrowserMsg::Download(id))
        }
        (Some(crate::frontend::browser::BrowserCardAction::Save), Some(_)) => Task::none(),
        (Some(crate::frontend::browser::BrowserCardAction::Apply), Some((_, _, _, true))) => {
            Task::none()
        }
        (Some(crate::frontend::browser::BrowserCardAction::Apply), Some((id, _, _, false))) => {
            update(app, BrowserMsg::Apply(id))
        }
        _ => browser_open_preview(app, hit.index),
    }
}

pub(super) fn browser_wall_wheel(app: &mut App, amount: f32) -> Task<Message> {
    if amount == 0.0 {
        return Task::none();
    }
    app.source_browser.wall.scene.grid_scroll(-amount);
    app.retick();
    Task::none()
}

pub(super) fn browser_hover(app: &mut App, idx: Option<usize>) {
    let motion = app.source_browser.motion;
    let Some(br) = app.source_browser.browser.as_mut() else {
        return;
    };
    if br.session.hover == idx {
        return;
    }
    if let Some(prev) = br.session.hover
        && Some(prev) != idx
        && let Some(spring) = br.view.hover_fades.get_mut(&prev)
    {
        spring.retarget(0.0);
    }
    if let Some(cur) = idx {
        let spring =
            br.view.hover_fades.entry(cur).or_insert_with(|| {
                motion.spring(0.0, crate::frontend::animation::MotionTier::Fast)
            });
        spring.retarget(1.0);
    }
    br.session.hover = idx;
    app.retick();
}

pub(super) fn browser_close_preview(app: &mut App) -> Task<Message> {
    if app.source_browser.browser.as_ref().is_some_and(|br| br.session.preview.is_some()) {
        app.source_browser.preview_closing = true;
        app.retick();
    }
    Task::none()
}

pub(super) fn browser_download(app: &mut App, id: String) -> Task<Message> {
    let request = app.source_browser.browser.as_ref().and_then(|br| {
        br.session
            .items
            .iter()
            .find(|item| item.id == id)
            .map(|item| (br.source, br.download_request(item)))
    });
    let Some((source, request)) = request else {
        return Task::none();
    };
    let call = crate::infrastructure::browser::encode_download(&request);
    if let Some(br) = app.source_browser.browser.as_mut()
        && let Some(item) = br.item_mut(&id)
    {
        item.downloading = true;
    }
    app.call_tracked(call.method, call.params, Pending::BrowserDownload { source, id });
    Task::none()
}

pub(super) fn browser_apply(app: &mut App, id: String) -> Task<Message> {
    if app
        .source_browser
        .browser
        .as_ref()
        .is_some_and(|br| br.session.items.iter().any(|item| item.id == id && item.downloading))
    {
        if let Some(br) = app.source_browser.browser.as_mut() {
            br.session.pending_apply = Some(id);
        }
        return Task::none();
    }
    if app.source_browser.browser.as_ref().is_some_and(crate::frontend::browser::Browser::is_steam)
    {
        return browser_apply_steam(app, id);
    }
    let kind =
        app.source_browser.browser.as_ref().map_or(ApplyKind::Static, |br| br.source.apply_kind());
    let item = app.source_browser.browser.as_ref().and_then(|br| {
        br.session.items.iter().find(|item| item.id == id).map(|item| {
            (item.downloaded_path.clone(), item.full_url.clone(), br.download_request(item))
        })
    });
    match item {
        Some((Some(path), _, _request)) => {
            let params =
                crate::infrastructure::browser::encode_apply(&ApplyTarget::Path { kind, path });
            app.daemon.client.call("wall.apply", params);
            if app.config.close_on_selection() {
                app.clear_browser_previews();
                crate::app::warm::request_hide(app);
            }
        }
        Some((None, url, request)) if !url.is_empty() => {
            let call = crate::infrastructure::browser::encode_download(&request);
            start_pending_download(app, id, call);
        }
        _ => {}
    }
    Task::none()
}

pub(super) fn browser_apply_steam(app: &mut App, id: String) -> Task<Message> {
    let item = app.source_browser.browser.as_ref().and_then(|br| {
        br.session
            .items
            .iter()
            .find(|item| item.id == id)
            .map(|item| (item.downloaded, br.download_request(item)))
    });
    let Some((downloaded, request)) = item else {
        return Task::none();
    };
    if !downloaded {
        let call = crate::infrastructure::browser::encode_download(&request);
        start_pending_download(app, id, call);
        return Task::none();
    }
    let params = crate::infrastructure::browser::encode_apply(&ApplyTarget::WallpaperEngine { id });
    app.daemon.client.call("wall.apply", params);
    if app.config.close_on_selection() {
        app.clear_browser_previews();
        crate::app::warm::request_hide(app);
    }
    Task::none()
}

pub(super) fn start_pending_download(app: &mut App, id: String, call: RpcCall) {
    let source = app.source_browser.browser.as_ref().map(|browser| browser.source);
    if let Some(br) = app.source_browser.browser.as_mut() {
        br.mark_pending_download(&id);
    }
    if let Some(source) = source {
        app.call_tracked(call.method, call.params, Pending::BrowserDownload { source, id });
    }
}

pub(super) fn browser_open_preview(app: &mut App, idx: usize) -> Task<Message> {
    let cap = app.config.youtube_max_minutes().saturating_mul(60);
    let req = app.source_browser.browser.as_mut().and_then(|br| {
        br.session.preview = Some(idx);
        if br.source == crate::frontend::browser::Source::Youtube
            && let Some(dur) = br.session.items.get(idx).map(|it| it.duration_secs)
            && dur > 0
        {
            let len = if cap == 0 { dur } else { dur.min(cap) };
            br.request.catalog.clip_start = String::from("0:00");
            br.request.catalog.clip_len = crate::frontend::browser::fmt_clock(len);
        }
        br.session
            .items
            .get(idx)
            .and_then(|item| (item.preview_path.is_none()).then(|| br.preview_request(item)))
    });
    if let Some(request) = req.filter(|request| !request.full_url.is_empty()) {
        let source = request.source;
        let id = request.id.clone();
        let call = crate::infrastructure::browser::encode_preview(&request);
        app.call_tracked(call.method, call.params, Pending::BrowserPreview { source, id });
    }
    app.source_browser.preview_animation = 0.0;
    app.source_browser.preview_closing = false;
    app.retick();
    Task::none()
}
