use iced::{Point, Rectangle};
use serde_json::json;
use skwd_config::keys::{filter_bar, selector};

use super::{
    COLOR_GROUP, COLOR_PREFIX, CSV_SEPARATOR, STEP_BUTTON, STEP_DENSITY, STEP_MOVE, STEP_ORDER,
    STEP_SIZE, apply_rect, changed, controls, placement, record, restore, set_frame, snapshot,
    visibility_key,
};
use crate::app::App;
use crate::domain::strand::Rect;
use crate::frontend::arrange::{Effect, Step, Target};
use crate::frontend::scene::layout::Mode;

fn rectangle(rect: Rect) -> Rectangle {
    Rectangle { x: rect.x, y: rect.y, width: rect.w, height: rect.h }
}

pub(super) fn apply(app: &mut App, effects: Vec<Effect>) {
    for effect in effects {
        match effect {
            Effect::Place(target, rect) => {
                hold(app);
                apply_rect(app, target, rectangle(rect));
            }
            Effect::Grid(rect, cells) => {
                hold(app);
                set_frame(app, rectangle(rect));
                app.config.set_key(selector::GRID_COLUMNS, json!(cells[0]));
                app.config.set_key(selector::GRID_ROWS, json!(cells[1]));
                app.apply_layout();
            }
            Effect::Commit(target, anchor, step) => {
                placement::anchor(app, target, anchor);
                release(app);
                app.apply_layout();
                record(app, label(step));
            }
            Effect::Nudge(target, rect, anchor, amend) => {
                apply_rect(app, target, rectangle(rect));
                placement::anchor(app, target, anchor);
                app.apply_layout();
                if amend {
                    super::super::settings_policy::normalize_picker_frame(app);
                    let snapshot = snapshot(app);
                    app.panels.arrange.as_mut().unwrap().history.amend(snapshot);
                } else {
                    record(app, STEP_MOVE);
                }
            }
            Effect::Abandon => settle(app),
            Effect::Reorder(id, at) => reorder(app, &id, Point::new(at[0], at[1])),
            Effect::Drop => drop(app),
            Effect::Hide(id) => {
                app.chrome.bar.reorder = None;
                release(app);
                hide(app, &id);
                changed(app, STEP_BUTTON);
            }
            Effect::Show(id, at) => {
                show(app, &id);
                if let Some(at) = at {
                    hold(app);
                    app.panels.arrange.as_mut().unwrap().shown = true;
                    app.apply_layout();
                    reorder(app, &id, Point::new(at[0], at[1]));
                } else {
                    changed(app, STEP_BUTTON);
                }
            }
            Effect::Layout(forward) => controls::layout(app, forward),
            Effect::Spacing => controls::spacing(app),
            Effect::Corners => controls::corners(app),
            Effect::Reset(target) => controls::reset(app, target),
            Effect::Message(message) => {
                let _ = super::update(app, message);
            }
        }
    }
}

fn label(step: Step) -> &'static str {
    match step {
        Step::Move => STEP_MOVE,
        Step::Size => STEP_SIZE,
        Step::Grid => STEP_DENSITY,
    }
}

fn hold(app: &mut App) {
    if app.panels.arrange.as_ref().is_some_and(|session| session.gesture.is_none()) {
        let before = snapshot(app);
        app.panels.arrange.as_mut().unwrap().gesture = Some(before);
    }
}

fn release(app: &mut App) {
    if let Some(session) = app.panels.arrange.as_mut() {
        session.gesture = None;
        session.shown = false;
    }
}

pub(super) fn settle(app: &mut App) {
    app.chrome.bar.reorder = None;
    let Some(session) = app.panels.arrange.as_mut() else { return };
    session.shown = false;
    if let Some(before) = session.gesture.take() {
        restore(app, &before);
    }
}

fn reorder(app: &mut App, id: &str, point: Point) {
    hold(app);
    let (width, height) = app.scene.viewport;
    if let Some(reorder) = app.chrome.bar.reorder.as_mut() {
        if reorder.move_to(point) {
            let (model, bar) = crate::app::view::filter_bar_target(app, width, height);
            app.chrome.bar.reorder.as_mut().unwrap().retarget(&model, bar);
        }
        return;
    }
    let (model, bar) = crate::app::view::filter_bar_target(app, width, height);
    let Some(index) = model.items.iter().position(|item| item.edit_id().as_deref() == Some(id))
    else {
        return;
    };
    let item = &model.items[index];
    let pulled = app.panels.arrange.as_ref().is_some_and(|session| session.shown);
    let origin = if pulled {
        Point::new(bar.x + item.x + item.w / 2.0, bar.y + item.y + item.h / 2.0)
    } else {
        point
    };
    app.chrome.bar.reorder = crate::frontend::ui::ReorderPreview::new(
        model,
        bar,
        index,
        origin,
        app.chrome.filter_bar_vertical && app.scene.mode != Mode::Sandy,
        app.config.ui_scale(),
        app.chrome.motion,
    );
    if pulled && let Some(reorder) = app.chrome.bar.reorder.as_mut() {
        reorder.move_to(point);
    }
}

fn drop(app: &mut App) {
    let shown = app.panels.arrange.as_ref().is_some_and(|session| session.shown);
    release(app);
    let Some(reorder) = app.chrome.bar.reorder.as_mut() else {
        if shown {
            record(app, STEP_BUTTON);
        }
        return;
    };
    reorder.finish();
    let moved = reorder.changed();
    let order = reorder.order();
    if reorder.finished() {
        app.chrome.bar.reorder = None;
    }
    if moved {
        let current = app.config.filter_bar_order();
        let mut visible = order.iter();
        let mut complete: Vec<_> = current
            .iter()
            .map(|id| if order.contains(id) { visible.next().unwrap().clone() } else { id.clone() })
            .collect();
        complete.extend(visible.cloned());
        app.config.set_key(filter_bar::BUTTON_ORDER, json!(complete.join(CSV_SEPARATOR)));
        app.chrome.bar.cache.clear();
    }
    if moved || shown {
        changed(app, if shown { STEP_BUTTON } else { STEP_ORDER });
    }
}

fn hide(app: &mut App, id: &str) {
    if id == crate::frontend::ui::edit_id::SETTINGS || id == crate::frontend::ui::edit_id::STASH {
        return;
    }
    if Target::floating(id).is_some() {
        if let Some(path) = visibility_key(id) {
            app.config.set_key(&path, json!(false));
        }
        return;
    }
    let mut hidden = app.config.filter_bar_hidden_buttons();
    if !hidden.iter().any(|item| item == id) {
        hidden.push(id.to_owned());
    }
    app.config.set_key(filter_bar::HIDDEN_BUTTONS, json!(hidden.join(CSV_SEPARATOR)));
}

fn show(app: &mut App, id: &str) {
    let mut hidden = app.config.filter_bar_hidden_buttons();
    if id.starts_with(COLOR_PREFIX) && !app.config.filter_show(COLOR_GROUP) {
        for bucket in 0..=crate::frontend::ui::edit_id::COLOR_CLEAR_INDEX {
            let key = if bucket == crate::frontend::ui::edit_id::COLOR_CLEAR_INDEX {
                crate::frontend::ui::edit_id::COLOR_CLEAR.to_owned()
            } else {
                format!("{COLOR_PREFIX}{bucket}")
            };
            if !hidden.contains(&key) {
                hidden.push(key);
            }
        }
    }
    hidden.retain(|item| item != id);
    app.config.set_key(filter_bar::HIDDEN_BUTTONS, json!(hidden.join(CSV_SEPARATOR)));
    if let Some(path) = visibility_key(id) {
        app.config.set_key(&path, json!(true));
    }
    app.chrome.bar.cache.clear();
}
