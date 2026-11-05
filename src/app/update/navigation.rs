use std::time::{Duration, Instant};

use iced::Task;

use crate::frontend::nav::{grid_step, step};
use crate::frontend::scene::layout::Mode;

#[allow(clippy::wildcard_imports)]
use super::super::*;
use super::browser::browser_apply_current;
use super::effects::effects_nav;
use super::update_inner;
use crate::app::helpers::browser_wraps;

const WHEEL_REST: Duration = Duration::from_millis(250);

fn loops(app: &App) -> bool {
    app.config.loop_around() && !app.input.held
}

fn hand_nav_closes_flip(app: &mut App) -> bool {
    if app.scene.mode == Mode::Hand && app.scene.flip_open() && !app.menu_capturing() {
        app.scene.close_flip();
        return true;
    }
    false
}

pub(super) fn wheel(app: &mut App, amount: f32) -> Task<Message> {
    if !hand_nav_closes_flip(app) && (app.detail_open() || app.menu_capturing()) {
        return Task::none();
    }
    app.scene.close_collection();
    let now = Instant::now();
    let rested = app.input.last_wheel.is_none_or(|at| now.duration_since(at) >= WHEEL_REST);
    app.input.last_wheel = Some(now);
    app.input.wheel_armed |= rested;
    match app.scene.mode {
        Mode::Slices | Mode::Depth | Mode::Sandy | Mode::Hand | Mode::Collection => {
            let steps = app.scene.slice_steps(amount);
            let wrap = steps != 0 && wheel_wraps(app);
            app.scene.slice_move(steps, app.library_session.filtered.len(), wrap);
        }
        Mode::Grid => app.scene.grid_scroll(-amount),
        Mode::Hex => {
            let wrap = wheel_wraps(app);
            hex_wheel(app, amount, wrap);
        }
    }
    app.retick();
    Task::none()
}

fn wheel_wraps(app: &mut App) -> bool {
    std::mem::take(&mut app.input.wheel_armed) && app.config.loop_around()
}

fn hex_wheel(app: &mut App, amount: f32, wrap: bool) {
    let count = app.library_session.filtered.len();
    if count == 0 {
        return;
    }
    let rows = app.scene.hp.rows.max(1);
    let delta = -(amount.signum() as i32) * app.scene.hp.scroll_step as i32;
    let col = app.scene.current / rows;
    let col = step(col, delta, count.div_ceil(rows), wrap).unwrap_or(col);
    app.scene.set_current((col * rows + app.scene.hex_row).min(count - 1), count);
}

pub(super) fn key_step(app: &mut App, delta: i32) -> Task<Message> {
    if app.source_browser.browser.is_some() {
        return browser_key_nav(app, delta, 0);
    }
    if let Some(task) = effects_nav(app, delta, 0) {
        return task;
    }
    if !hand_nav_closes_flip(app) && (app.menu_capturing() || app.detail_open()) {
        return Task::none();
    }
    app.scene.kb_nav = true;
    let count = app.library_session.filtered.len();
    if let Some(next) = step(app.scene.current, delta, count, loops(app)) {
        app.scene.set_current(next, count);
        app.retick();
    }
    Task::none()
}

fn page_step(scene: &crate::app::scene::SceneCore) -> usize {
    let stride = match scene.mode {
        Mode::Grid => scene.gp.cols.max(1),
        Mode::Hex => scene.hp.rows.max(1),
        _ => 1,
    };
    (scene.render.hits.len() / stride).max(1) * stride
}

pub(super) fn key_page(app: &mut App, backwards: bool) -> Task<Message> {
    let browsing = app.source_browser.browser.is_some();
    let scene = if browsing { &app.source_browser.wall.scene } else { &app.scene };
    let page = page_step(scene) as i32;
    let delta = if backwards { -page } else { page };
    if browsing {
        return browser_seek(app, delta, browser_wraps(app));
    }
    let current = app.scene.current;
    let count = app.library_session.filtered.len();
    seek(app, step(current, delta, count, loops(app)).unwrap_or(current))
}

pub(super) fn key_edge(app: &mut App, last: bool) -> Task<Message> {
    if let Some(browser) = app.source_browser.browser.as_ref() {
        let count = browser.session.items.len() as i32;
        return browser_seek(app, if last { count } else { -count }, false);
    }
    seek(app, if last { usize::MAX } else { 0 })
}

fn browser_seek(app: &mut App, delta: i32, wrap: bool) -> Task<Message> {
    let Some(session) = app.source_browser.browser.as_ref().map(|browser| &browser.session) else {
        return Task::none();
    };
    if session.preview.is_some() || session.items.is_empty() {
        return Task::none();
    }
    let count = session.items.len();
    let current = app.source_browser.wall.scene.hover.unwrap_or(0);
    browser_select(app, step(current, delta, count, wrap).unwrap_or(current))
}

fn seek(app: &mut App, target: usize) -> Task<Message> {
    if app.panels.effects.is_some()
        || (!hand_nav_closes_flip(app) && (app.menu_capturing() || app.detail_open()))
    {
        return Task::none();
    }
    let count = app.library_session.filtered.len();
    if count == 0 {
        return Task::none();
    }
    app.scene.kb_nav = true;
    app.scene.set_current(target.min(count - 1), count);
    app.retick();
    Task::none()
}

pub(super) fn random_other(count: usize, current: usize, seed: u64) -> usize {
    if count < 2 {
        return 0;
    }
    let pick = (seed % (count as u64 - 1)) as usize;
    if pick >= current { pick + 1 } else { pick }
}

pub(super) fn apply_random(app: &mut App) -> Task<Message> {
    use std::hash::BuildHasher;
    if app.menu_capturing() || app.detail_open() {
        return Task::none();
    }
    let count = app.library_session.filtered.len();
    if count == 0 {
        return Task::none();
    }
    let seed = std::collections::hash_map::RandomState::new().hash_one(std::time::Instant::now());
    let target = random_other(count, app.scene.current, seed);
    app.scene.kb_nav = true;
    app.scene.set_current(target, count);
    app.retick();
    apply_task(app, target)
}

pub(super) fn apply_current(app: &mut App) -> Task<Message> {
    if app.source_browser.browser.is_some() {
        return browser_apply_current(app);
    }
    if app.panels.effects.as_ref().is_some_and(|eff| {
        eff.mode() == crate::frontend::effects::EffectsMode::Studio && eff.has_effects_page()
    }) {
        return update_inner(app, Message::Effects(crate::frontend::effects::EffectsMsg::Apply));
    }
    if app.menu_capturing() {
        return Task::none();
    }
    apply_task(app, app.scene.current)
}

pub(super) fn key_favourite(app: &mut App) -> Task<Message> {
    if app.menu_capturing() {
        return Task::none();
    }
    toggle_favourite(app, app.scene.flipped().unwrap_or(app.scene.current));
    Task::none()
}

pub(super) fn key_reveal(app: &mut App) -> Task<Message> {
    if app.menu_capturing()
        || app.scene.mode != Mode::Hand
        || app.library_session.filtered.is_empty()
    {
        return Task::none();
    }
    app.scene.hand_reveal_toggle();
    app.retick();
    Task::none()
}

pub(super) fn key_flip(app: &mut App) -> Task<Message> {
    if app.menu_capturing() {
        return Task::none();
    }
    if app.detail_open() || app.scene.flip_open() {
        app.scene.close_flip();
        app.retick();
        return Task::none();
    }
    if app.library_session.filtered.is_empty() {
        return Task::none();
    }
    let idx = app.scene.current;
    match app.scene.mode {
        Mode::Slices | Mode::Depth | Mode::Hand => app.scene.toggle_flip(idx),
        Mode::Sandy => {
            app.scene.sandy_settle_now();
            app.scene.toggle_flip(app.scene.current);
        }
        _ => {
            let rect = app
                .scene
                .render
                .hits
                .iter()
                .find(|hit| hit.index == idx)
                .map(|hit| [hit.cx, hit.cy, hit.hw, hit.hh]);
            if let Some(rect) = rect {
                app.scene.open_detail(idx, rect);
            }
        }
    }
    app.retick();
    Task::none()
}

pub(super) fn key_effects(app: &mut App) -> Task<Message> {
    let Some(idx) = effects_target(app) else {
        return Task::none();
    };
    open_effects(app, idx, crate::frontend::effects::EffectsMode::Displays);
    Task::none()
}

pub(super) fn key_studio(app: &mut App) -> Task<Message> {
    let Some(idx) = effects_target(app) else {
        return Task::none();
    };
    super::picker::open_studio(app, idx)
}

fn effects_target(app: &mut App) -> Option<usize> {
    if app.panels.effects.is_some() {
        app.panels.effects = None;
        app.retick();
        return None;
    }
    if app.menu_capturing() || app.library_session.filtered.is_empty() {
        return None;
    }
    Some(app.scene.flipped().unwrap_or(app.scene.current))
}

pub(super) fn key_row(app: &mut App, delta: i32) -> Task<Message> {
    if app.source_browser.browser.is_some() {
        return browser_key_nav(app, 0, delta);
    }
    if let Some(task) = effects_nav(app, 0, delta) {
        return task;
    }
    if app.menu_capturing() || app.detail_open() {
        return Task::none();
    }
    let wrap = loops(app);
    let count = app.library_session.filtered.len();
    let cur = app.scene.current;
    let next = match app.scene.mode {
        Mode::Collection => return key_step(app, delta),
        Mode::Hex => {
            let rows = app.scene.hp.rows.max(1);
            let top = cur - cur % rows;
            step(cur % rows, delta, rows.min(count.saturating_sub(top)), wrap).map(|row| top + row)
        }
        Mode::Grid => grid_step(cur, delta, app.scene.gp.cols, count, wrap),
        _ => None,
    };
    if let Some(next) = next {
        app.scene.kb_nav = true;
        app.scene.set_current(next, count);
        app.retick();
    }
    Task::none()
}
