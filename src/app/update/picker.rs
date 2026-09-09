use std::time::Instant;

use iced::Task;
use serde_json::json;

use crate::domain::library::catalog::WallpaperKind;
use crate::frontend::scene::layout::Mode;

#[allow(clippy::wildcard_imports)]
use super::super::*;
use super::tags::tag_select_click;

pub(super) fn mouse_moved(app: &mut App, x: f32, y: f32) -> Task<Message> {
    if app.detail_open() || app.menu_capturing() {
        return Task::none();
    }
    if app.scene.last_mouse.is_none() {
        app.scene.last_mouse = Some((x, y));
        return Task::none();
    }
    app.input.last_activity = Instant::now();
    app.scene.input_idle = false;
    let moved = match app.scene.last_mouse {
        Some((lx, ly)) => (x - lx).abs() > 2.0 || (y - ly).abs() > 2.0,
        None => true,
    };
    if !moved {
        return Task::none();
    }
    app.scene.last_mouse = Some((x, y));
    app.scene.kb_nav = false;
    let hit = app.scene.render.hits.iter().find(|hit| hit.contains(x, y)).map(|hit| hit.index);
    let changed = hit != app.scene.hover;
    match app.scene.mode {
        Mode::Slices | Mode::Hex => {
            if let Some(idx) = hit {
                app.scene.set_current(idx, app.library_session.filtered.len());
            }
            app.scene.hover = hit;
        }
        Mode::Sandy => {
            app.scene.hover = hit;
            app.scene.sandy_pointer(x, y, hit, app.library_session.filtered.len());
        }
        Mode::Grid => app.scene.hover = hit,
    }
    if changed {
        app.scene.touch();
        app.retick();
    }
    Task::none()
}

fn card_is_static(app: &App, idx: usize) -> bool {
    app.library_session
        .filtered
        .get(idx)
        .and_then(|&si| app.library_session.library.catalog().items.get(si as usize))
        .is_some_and(|it| it.kind == WallpaperKind::Static)
}

pub(super) fn click(
    app: &mut App,
    x: f32,
    y: f32,
    button: crate::domain::input::MouseButton,
) -> Task<Message> {
    use crate::domain::input::MouseButton;
    if app.menu_capturing() && !app.tags.editing {
        return Task::none();
    }
    app.input.last_activity = Instant::now();
    app.scene.input_idle = false;
    let hit_rect = app
        .scene
        .render
        .hits
        .iter()
        .find(|hit| hit.contains(x, y))
        .map(|hit| (hit.index, [hit.cx, hit.cy, hit.hw, hit.hh]));
    let hit = hit_rect.map(|(idx, _)| idx);
    if app.tags.mode && button == MouseButton::Left {
        return tag_select_click(app, hit);
    }
    if let Some(fi) = app.scene.flipped() {
        if button == MouseButton::Left {
            return back_panel_click(app, fi, x, y);
        }
        if button == MouseButton::Right && remove_back_panel_tag(app, fi, x, y) {
            return Task::none();
        }
    }
    if app.detail_open() && button == MouseButton::Right {
        app.scene.close_flip();
        app.retick();
        return Task::none();
    }
    let spec = crate::domain::input::MouseSpec { mods: app.input.mods, button };
    let Some(action) = app.input.bindings.lookup_mouse(spec) else {
        return Task::none();
    };
    run_action(app, action, hit_rect)
}

fn run_action(
    app: &mut App,
    action: crate::domain::input::InputAction,
    target: Option<(usize, [f32; 4])>,
) -> Task<Message> {
    use crate::domain::input::InputAction;
    if !action.targets_card() {
        return super::update_inner(app, crate::app::input::action_message(action));
    }
    let Some((idx, rect)) = target else {
        return Task::none();
    };
    if app.tags.editing {
        commit_pending_tag(app);
    }
    app.tags.editing = false;
    app.tags.card_drawer_open = false;
    app.scene.set_tag_editing(false);
    match action {
        InputAction::Select => select_click(app, idx),
        InputAction::Apply => apply_task(app, idx),
        InputAction::Flip => flip_click(app, idx, rect),
        InputAction::Favourite => {
            toggle_favourite(app, idx);
            Task::none()
        }
        InputAction::Effects => {
            if app.panels.effects.is_none() {
                open_effects(app, idx, crate::frontend::effects::EffectsMode::Displays);
            }
            Task::none()
        }
        InputAction::Studio => open_studio(app, idx),
        InputAction::SceneProperties => {
            app.open_scene_properties();
            Task::none()
        }
        _ => Task::none(),
    }
}

pub(super) fn open_studio(app: &mut App, idx: usize) -> Task<Message> {
    if app.panels.effects.is_some() {
        return Task::none();
    }
    if card_is_static(app, idx) {
        open_effects(app, idx, crate::frontend::effects::EffectsMode::Studio);
    } else {
        app.show_toast(crate::i18n::tr("status-effects-static-only"));
        app.retick();
    }
    Task::none()
}

fn remove_back_panel_tag(app: &mut App, fi: usize, x: f32, y: f32) -> bool {
    let (Some(bp), Some(&si)) =
        (app.scene.render.back.clone(), app.library_session.filtered.get(fi))
    else {
        return false;
    };
    let lay = crate::frontend::ui::back_layout(&bp);
    let within = |rect| crate::frontend::ui::back_contains(&bp, &lay, rect, x, y);
    let Some(ti) = lay.tags.iter().position(|&rect| within(rect)) else {
        return false;
    };
    let key = app.library_session.library.catalog().items[si as usize].key.clone();
    remove_tag_at(app, &key, ti);
    true
}

fn select_click(app: &mut App, idx: usize) -> Task<Message> {
    match app.scene.mode {
        Mode::Sandy => {
            if idx != app.scene.current {
                app.scene.set_current(idx, app.library_session.filtered.len());
            }
            apply_task(app, idx)
        }
        Mode::Slices => {
            if idx == app.scene.current {
                return apply_task(app, idx);
            }
            app.scene.set_current(idx, app.library_session.filtered.len());
            app.retick();
            Task::none()
        }
        _ => apply_task(app, idx),
    }
}

fn flip_click(app: &mut App, idx: usize, rect: [f32; 4]) -> Task<Message> {
    match app.scene.mode {
        Mode::Slices => {
            if idx == app.scene.current {
                app.scene.toggle_flip(idx);
            } else {
                app.scene.set_current(idx, app.library_session.filtered.len());
            }
        }
        Mode::Sandy => {
            if idx != app.scene.current {
                app.scene.set_current(idx, app.library_session.filtered.len());
            }
            app.scene.sandy_settle_now();
            app.scene.toggle_flip(app.scene.current);
        }
        _ => app.scene.open_detail(idx, rect),
    }
    app.retick();
    Task::none()
}

fn back_panel_click(app: &mut App, fi: usize, x: f32, y: f32) -> Task<Message> {
    let (Some(bp), Some(&si)) =
        (app.scene.render.back.clone(), app.library_session.filtered.get(fi))
    else {
        return Task::none();
    };
    let lay = crate::frontend::ui::back_layout(&bp);
    let key = app.library_session.library.catalog().items[si as usize].key.clone();
    let (fx, fy, _) = lay.fav;
    let within = |rect| crate::frontend::ui::back_contains(&bp, &lay, rect, x, y);

    if within((fx - 30.0, fy - 26.0, 60.0, 52.0)) {
        toggle_favourite(app, fi);
    } else if let Some(ti) = lay.tags.iter().position(|&rect| within(rect)) {
        remove_tag_at(app, &key, ti);
    } else if lay.tag_overflow.is_some_and(|rect| within((rect.0, rect.1, rect.2, rect.3))) {
        toggle_card_tag_drawer(app);
    } else if within(lay.add) {
        app.tags.editing = true;
        app.tags.focus_pending = true;
        let tags =
            app.library_session.library.catalog().tags.get(&key).cloned().unwrap_or_default();
        begin_card_tag_edit(app, &tags);
        app.scene.set_tag_editing(true);
        app.retick();
    } else if within(lay.playlist) {
        let name = app.library_session.library.catalog().items[si as usize].name.clone();
        app.open_card_picker(key, name);
        app.retick();
    } else if lay.effects.is_some_and(within) {
        open_effects(app, fi, crate::frontend::effects::EffectsMode::Studio);
        app.retick();
    } else if lay.scene_properties.is_some_and(within) {
        app.open_scene_properties();
    } else if lay.overview.is_some_and(within) {
        set_overview_backdrop(app, si);
        app.retick();
    } else if within(lay.delete) {
        delete_wallpaper(app, si);
        app.retick();
    } else if app.scene.mode != Mode::Slices {
        return close_flip_or_apply(app, fi, &bp, x, y);
    }
    Task::none()
}

fn set_overview_backdrop(app: &mut App, si: u32) {
    let item = &app.library_session.library.catalog().items[si as usize];
    let img = item.backdrop_source();
    if !img.is_empty() {
        app.config.save_key(skwd_config::keys::niri::BACKDROP, json!(img));
        app.config.save_key(skwd_config::keys::niri::BACKDROP_FOLLOW_WALLPAPER, json!(false));
        app.config.save_key(skwd_config::keys::niri::OVERVIEW_BACKDROP, json!(true));
        app.daemon.client.call("wall.refresh_overview_backdrop", json!({}));
    }
    app.chrome.cache.clear();
}

pub(crate) fn delete_wallpaper(app: &mut App, si: u32) {
    let item = &app.library_session.library.catalog().items[si as usize];
    let we_id = if item.we_id.is_empty() {
        item.key.strip_prefix("we:").unwrap_or("").to_string()
    } else {
        item.we_id.clone()
    };
    if we_id.is_empty() {
        app.daemon.client.call("wall.remove", json!({ "path": item.path.clone() }));
    } else {
        app.daemon.client.call("wall.remove", json!({ "we_id": we_id }));
    }
    app.call_tracked("wall.list", json!({"favourites": false}), Pending::List);
    app.tags.editing = false;
    app.tags.card_drawer_open = false;
    app.scene.close_flip();
}

fn close_flip_or_apply(
    app: &mut App,
    fi: usize,
    bp: &crate::frontend::scene::BackPanel,
    x: f32,
    y: f32,
) -> Task<Message> {
    if app.tags.editing {
        commit_pending_tag(app);
        app.tags.editing = false;
    }
    app.tags.card_drawer_open = false;
    let inside = crate::frontend::scene::layout::sheared_contains(
        bp.cx,
        bp.cy,
        bp.hw,
        bp.hh,
        bp.skew,
        bp.edge_tilt,
        x,
        y,
    );
    app.scene.close_flip();
    app.retick();
    if inside { apply_task(app, fi) } else { Task::none() }
}
