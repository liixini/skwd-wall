use std::time::Instant;

use iced::Task;
use log::info;

#[allow(clippy::wildcard_imports)]
use super::super::*;

pub(crate) fn drive_transition_preview(app: &mut App) {
    if !app.wants_transition_preview() {
        app.panels.transition_preview.stop();
        return;
    }
    let shader = app.config.str_path(skwd_config::keys::transition::SHADER);
    let shader = if shader.is_empty() { String::from("random") } else { shader };
    let fill_mode = app.config.str_path(skwd_config::keys::display::FILL_MODE);
    let fill_mode = if fill_mode.is_empty() { String::from("fill") } else { fill_mode };
    let duration = app.config.transition_duration_ms();
    let Some((fallback, to)) = crate::infrastructure::preview::stock_pair(&app.config.cache_dir())
    else {
        return;
    };
    let selected = app.selected_settings_preview_path();
    let from = if !selected.is_empty() && std::path::Path::new(selected).is_file() {
        selected.to_string()
    } else {
        fallback
    };
    let frame_ms = app.config.transition_preview_interval().as_millis() as u64;
    app.panels.transition_preview.ensure(
        &from,
        &to,
        &shader,
        &fill_mode,
        duration,
        frame_ms.max(4),
    );
    let first = app.panels.transition_preview.handle.is_none();
    if app.panels.transition_preview.take_frame() && first {
        app.invalidate_settings();
    }
}

pub(super) fn tick(app: &mut App, now: Instant, w: f32, h: f32) -> Task<Message> {
    let animating = app.animating();
    let studio_due = app
        .panels
        .colour_studio
        .as_ref()
        .and_then(|studio| crate::app::runtime::studio_deadline(now, &studio.model))
        .is_some_and(|deadline| deadline <= now);
    let minute_due =
        app.panels.strand.as_ref().and_then(|strand| strand.minute).is_some_and(|due| due <= now);
    let wake_due = animating || app.scene.preview_deadline_due(now) || studio_due || minute_due;
    if !crate::app::scene::animation_tick_due(wake_due, app.runtime_state.next_animation_tick, now)
    {
        app.preview_resources.render_loop_active = animating;
        return Task::none();
    }
    if (w, h) != app.scene.viewport {
        app.scene.viewport = (w, h);
        if app.config.set_screen_width(w) {
            app.apply_layout();
        }
    }
    app.run_tick(now);
    let mut tasks: Vec<Task<Message>> = Vec::new();
    if app.tags.focus_pending
        && app.tags.editing
        && app.scene.render.back.as_ref().is_some_and(|back| back.add_open > 0.85)
    {
        app.tags.focus_pending = false;
        tasks.push(iced::widget::operation::focus(tag_input_id()));
    }
    for (key, pane) in &mut app.chrome.pane_scrolls {
        if pane.dirty {
            pane.dirty = false;
            tasks.push(iced::widget::operation::scroll_to(
                crate::frontend::ui::pane_id(key),
                iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: pane.spring.x },
            ));
        }
    }
    Task::batch(tasks)
}

pub(super) fn exit(app: &mut App) -> Task<Message> {
    if app.panels.strand.is_some() {
        return super::strand::escape(app);
    }
    if app.panels.arrange.is_some() {
        return super::arrange::update(app, crate::frontend::arrange::ArrangeMsg::Escape);
    }
    if app.theme.audition_open && app.theme.audition_focused {
        super::warm::exit_picker(app);
    }
    if app.close_topmost_overlay() {
        return Task::none();
    }
    if app.scene.close_collection() {
        app.retick();
        return Task::none();
    }
    app.runtime_state.metrics.exit_summary();
    info!("close requested, rss = {}", crate::infrastructure::observability::rss_label());
    super::warm::exit_picker(app)
}
