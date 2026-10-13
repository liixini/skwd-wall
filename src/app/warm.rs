use super::{App, Message};
use crate::infrastructure::runtime::Wake;

static SHOWN_AT: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);

fn note_show_requested() {
    *SHOWN_AT.lock().unwrap_or_else(std::sync::PoisonError::into_inner) =
        Some(std::time::Instant::now());
}

pub(crate) fn note_overlay_drawn() {
    let taken = SHOWN_AT.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();
    if let Some(shown) = taken {
        log::info!(
            "warm: overlay drawn {:.1} ms after show",
            shown.elapsed().as_secs_f64() * 1000.0
        );
    }
}

pub(crate) fn adopt_first_window(
    app: &mut App,
    id: iced::window::Id,
    width: f32,
    height: f32,
) -> iced::Task<Message> {
    if app.runtime_state.overlay.is_some() {
        return iced::Task::none();
    }
    app.runtime_state.overlay = Some(id);
    app.runtime_state.picker_output = crate::shell::picker_output(id);
    app.scene.viewport = (width, height);
    if app.config.set_screen_width(width) {
        app.snap_layout();
    }
    app.theme.suspended = false;
    note_show_requested();
    app.scene.begin_open_fade();
    app.retick();
    log::info!("adopted initial surface {id:?} as the overlay");
    iced::Task::none()
}

#[cfg(target_os = "linux")]
pub(crate) fn exit_picker(app: &mut App) -> ! {
    app.on_hidden();
    crate::hard_exit(0)
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn exit_picker(app: &mut App) -> ! {
    app.save_browse_position();
    crate::hard_exit(0)
}

pub(crate) fn toggle(app: &mut App) -> iced::Task<Message> {
    if app.runtime_state.stashed {
        return unstash(app);
    }
    if app.runtime_state.overlay.is_some() {
        exit_picker(app);
    }
    iced::Task::none()
}

pub(crate) fn toggle_stash(app: &mut App) -> iced::Task<Message> {
    if app.runtime_state.stashed { unstash(app) } else { stash(app) }
}

pub(crate) fn stash(app: &mut App) -> iced::Task<Message> {
    let Some(id) = app.runtime_state.overlay else { return iced::Task::none() };
    if app.runtime_state.stashed {
        return iced::Task::none();
    }
    app.runtime_state.stashed = true;
    app.panels.transition_preview.stop();
    app.scene.release_while_hidden();
    log::info!("stashed the overlay");
    crate::shell::stash_surface(id, true)
}

pub(crate) fn unstash(app: &mut App) -> iced::Task<Message> {
    let Some(id) = app.runtime_state.overlay else { return iced::Task::none() };
    if !app.runtime_state.stashed {
        return iced::Task::none();
    }
    app.runtime_state.stashed = false;
    app.conceal();
    app.runtime_state.last_tick = None;
    app.chrome.cache.clear();
    app.chrome.bar.cache.clear();
    app.scene.touch();
    app.retick();
    log::info!("restored the stashed overlay");
    crate::shell::stash_surface(id, false)
}

pub(crate) fn request_hide(app: &App) {
    let _ = app.runtime_state.wake_tx.unbounded_send(Wake::Hide);
}

#[cfg(test)]
mod tests;
