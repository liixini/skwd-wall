use iced::Task;

use crate::app::{App, Message};
use crate::domain::input::MouseButton;
use crate::frontend::scene::layout::Mode;
use crate::frontend::tagcloud::TagMsg;

pub(in crate::app) fn active(app: &App) -> bool {
    app.panels.arrange.as_ref().is_some_and(|session| session.preview_config.is_some())
}

pub(in crate::app) fn enter(app: &mut App) {
    let Some(session) = app.panels.arrange.as_mut() else { return };
    if session.gesturing() || session.preview_config.is_some() {
        return;
    }
    session.preview_config = Some(app.config.document().clone());
    if let Some(output) = app.config.layout_output() {
        session.output_reviews.insert(
            output.to_owned(),
            crate::app::state::arrange::displays::OutputReview {
                viewport: skwd_config::picker_layout::Size {
                    width: app.scene.viewport.0,
                    height: app.scene.viewport.1,
                },
                save_disconnected: false,
            },
        );
    }
    app.chrome.bar.menu = None;
    app.scene.close_flip();
    app.retick();
}

pub(in crate::app) fn leave(app: &mut App) {
    if let Some(config) =
        app.panels.arrange.as_mut().and_then(|session| session.preview_config.take())
    {
        app.config.replace_document(config);
        app.chrome.bar.menu = None;
        app.scene.hand_drag_end();
        app.retick();
    }
}

pub(in crate::app) fn allows(message: &Message) -> bool {
    matches!(
        message,
        Message::Arrange(_)
            | Message::Daemon(
                crate::infrastructure::runtime::Wake::Frame(_)
                    | crate::infrastructure::runtime::Wake::Ipc(_)
                    | crate::infrastructure::runtime::Wake::Decoded
                    | crate::infrastructure::runtime::Wake::Query(..)
                    | crate::infrastructure::runtime::Wake::Semantic(_)
                    | crate::infrastructure::runtime::Wake::HudTick
            )
            | Message::WindowOpened { .. }
            | Message::Viewport(..)
            | Message::MouseMoved(..)
            | Message::Click(..)
            | Message::Wheel(_)
            | Message::KeyPrev
            | Message::KeyNext
            | Message::KeyUp
            | Message::KeyDown
            | Message::KeyPage { .. }
            | Message::KeyEdge { .. }
            | Message::SetMods(_)
            | Message::PointerUp
            | Message::KeyPressed(..)
            | Message::KeyReleased(_)
            | Message::Exit
            | Message::SetColorFilter(_)
            | Message::SetTypeFilter(_)
            | Message::SetFolder(_)
            | Message::CycleFolder { .. }
            | Message::ToggleFolder
            | Message::ToggleShowHidden
            | Message::SetSort(_)
            | Message::SetOrient(_)
            | Message::SetResolution(_)
            | Message::ToggleFavourites
            | Message::CycleType { .. }
            | Message::CycleSort { .. }
            | Message::BarHover(..)
            | Message::FolderMenuToggle
            | Message::FolderMenuScroll(_)
            | Message::OpenTagCloud
            | Message::CloseTagCloud
            | Message::ClearTags
            | Message::PaneWheel(..)
            | Message::PaneScrolled(..)
            | Message::Noop
            | Message::Tag(
                TagMsg::QueryInput(_)
                    | TagMsg::SearchMode(_)
                    | TagMsg::CycleSearchMode
                    | TagMsg::CloudClick(..)
                    | TagMsg::CloudScroll(_)
                    | TagMsg::Autocomplete
                    | TagMsg::MatchMode(_)
                    | TagMsg::SortAz(_)
                    | TagMsg::ToggleMatchingTags
            )
    )
}

pub(in crate::app) fn click(app: &mut App, x: f32, y: f32, button: MouseButton) -> Task<Message> {
    if button != MouseButton::Left || app.menu_capturing() {
        return Task::none();
    }
    if let Some(index) =
        app.scene.render.hits.iter().find(|hit| hit.contains(x, y)).map(|hit| hit.index)
    {
        if app.scene.mode == Mode::Collection {
            app.scene.select_collection(index, app.library_session.filtered.len());
        } else {
            app.scene.set_current(index, app.library_session.filtered.len());
        }
        app.retick();
    }
    Task::none()
}
