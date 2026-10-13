use iced::{Color, Event, Subscription, event, keyboard, mouse, window};

#[allow(clippy::wildcard_imports)]
use super::*;

pub fn subscription(_app: &App) -> Subscription<Message> {
    Subscription::batch([event::listen_with(on_event), Subscription::run(wake_stream)])
}

pub(crate) fn wake_stream() -> impl iced::futures::Stream<Item = Message> {
    use iced::futures::StreamExt;
    let rx = take_wake_receiver();
    match rx {
        Some(rx) => rx.map(Message::Daemon).boxed(),
        None => iced::futures::stream::pending::<Message>().boxed(),
    }
}

pub(crate) fn on_event(event: Event, status: event::Status, window: window::Id) -> Option<Message> {
    if let Event::Window(window::Event::Opened { size, .. }) = event {
        return Some(Message::WindowOpened { id: window, width: size.width, height: size.height });
    }
    if let Event::Keyboard(keyboard::Event::ModifiersChanged(m)) = event {
        return Some(Message::SetMods(mods(m)));
    }
    if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) = event {
        return Some(Message::PointerUp);
    }
    if status == event::Status::Captured
        && let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = &event
    {
        if matches!(
            key,
            keyboard::Key::Named(keyboard::key::Named::Escape | keyboard::key::Named::Tab)
        ) || reaches_bindings_from_text(key, *modifiers)
        {
            return Some(Message::KeyPressed(key.clone(), *modifiers));
        }
        return None;
    } else if status == event::Status::Captured
        && let Event::Keyboard(_) = event
    {
        return None;
    }
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            Some(Message::KeyPressed(key, modifiers))
        }
        _ => None,
    }
}

pub(crate) fn reaches_bindings_from_text(
    key: &keyboard::Key,
    modifiers: keyboard::Modifiers,
) -> bool {
    use keyboard::key::Named;
    if !modifiers.control() && !modifiers.alt() {
        return false;
    }
    let edits_text = match key {
        keyboard::Key::Character(character) => match character.as_str() {
            "a" | "c" | "x" => modifiers.control(),
            "v" => modifiers.control() && !modifiers.alt(),
            _ => false,
        },
        keyboard::Key::Named(
            Named::Backspace | Named::Delete | Named::ArrowLeft | Named::ArrowRight,
        ) => modifiers.control(),
        _ => false,
    };
    !edits_text
}

pub(crate) fn settings_key_message(
    key: &keyboard::Key,
    modifiers: keyboard::Modifiers,
) -> Option<Message> {
    use crate::frontend::settings::{SettingsKey, SettingsMsg};
    use keyboard::key::Named;

    let key = match key {
        keyboard::Key::Character(character)
            if character == "/" || (modifiers.control() && character.eq_ignore_ascii_case("f")) =>
        {
            SettingsKey::Search
        }
        keyboard::Key::Named(Named::Escape) => SettingsKey::Cancel,
        keyboard::Key::Named(Named::Tab) if modifiers.control() => {
            SettingsKey::CategoryNext { backwards: modifiers.shift() }
        }
        keyboard::Key::Named(Named::Tab) => SettingsKey::FocusNext { backwards: modifiers.shift() },
        keyboard::Key::Named(Named::ArrowLeft) => SettingsKey::Previous,
        keyboard::Key::Named(Named::ArrowRight) => SettingsKey::Next,
        keyboard::Key::Named(Named::ArrowUp) => SettingsKey::Up,
        keyboard::Key::Named(Named::ArrowDown) => SettingsKey::Down,
        keyboard::Key::Named(Named::Enter) => SettingsKey::Activate,
        keyboard::Key::Character(character) if character == " " => SettingsKey::Activate,
        _ => return None,
    };
    Some(Message::Settings(SettingsMsg::Key(key)))
}

pub(crate) fn mods(modifiers: keyboard::Modifiers) -> crate::domain::input::Mods {
    crate::domain::input::Mods::new(modifiers.control(), modifiers.alt(), modifiers.shift())
}

fn source_message(source: crate::frontend::browser::Source) -> Message {
    Message::Browser(crate::frontend::browser::BrowserMsg::SwitchSource(source))
}

pub(crate) fn action_message(action: crate::domain::input::InputAction) -> Message {
    use crate::domain::input::InputAction;
    use crate::frontend::browser::Source;
    match action {
        InputAction::Playlists => Message::OpenPlaylists,
        InputAction::Favourite => Message::KeyFavourite,
        InputAction::Flip => Message::KeyFlip,
        InputAction::Reveal => Message::KeyReveal,
        InputAction::Effects => Message::KeyEffects,
        InputAction::Studio => Message::KeyStudio,
        InputAction::SceneProperties => Message::OpenSceneProps,
        InputAction::Settings => Message::ToggleSettings,
        InputAction::Help => Message::ToggleHelp,
        InputAction::ThemePanel => Message::ToggleThemePanel,
        InputAction::ColorPrev => Message::SetColorFilter(i64::MIN),
        InputAction::ColorNext => Message::SetColorFilter(i64::MAX),
        InputAction::FilterBar => Message::ToggleFilterBar,
        InputAction::FolderPrev => Message::CycleFolder { backwards: true },
        InputAction::FolderNext => Message::CycleFolder { backwards: false },
        InputAction::FolderToggle => Message::ToggleFolder,
        InputAction::ShowHidden => Message::ToggleShowHidden,
        InputAction::TagCloud => Message::OpenTagCloud,
        InputAction::TagMode => Message::ToggleTagMode,
        InputAction::NavLeft => Message::KeyPrev,
        InputAction::NavRight => Message::KeyNext,
        InputAction::NavUp => Message::KeyUp,
        InputAction::NavDown => Message::KeyDown,
        InputAction::PageBack => Message::KeyPage { backwards: true },
        InputAction::PageForward => Message::KeyPage { backwards: false },
        InputAction::JumpFirst => Message::KeyEdge { last: false },
        InputAction::JumpLast => Message::KeyEdge { last: true },
        InputAction::Select | InputAction::Apply => Message::ApplyCurrent,
        InputAction::Autocomplete => Message::Tag(crate::frontend::tagcloud::TagMsg::Autocomplete),
        InputAction::TypePrev => Message::CycleType { backwards: true },
        InputAction::TypeNext => Message::CycleType { backwards: false },
        InputAction::SortPrev => Message::CycleSort { backwards: true },
        InputAction::SortNext => Message::CycleSort { backwards: false },
        InputAction::ModePrev => Message::CycleMode { backwards: true },
        InputAction::ModeNext => Message::CycleMode { backwards: false },
        InputAction::RandomRotate => Message::ToggleRandomRotate,
        InputAction::RandomApply => Message::ApplyRandom,
        InputAction::Downloads => Message::ToggleDownloads,
        InputAction::SearchMode => Message::Tag(crate::frontend::tagcloud::TagMsg::CycleSearchMode),
        InputAction::SourceWallhaven => source_message(Source::Wallhaven),
        InputAction::SourceSteam => source_message(Source::Steam),
        InputAction::SourceUnsplash => source_message(Source::Unsplash),
        InputAction::SourcePexels => source_message(Source::Pexels),
        InputAction::SourceYoutube => source_message(Source::Youtube),
        InputAction::SourceBing => source_message(Source::Bing),
    }
}

pub(crate) fn key_message(
    map: &crate::domain::input::InputMap,
    key: &keyboard::Key,
    modifiers: keyboard::Modifiers,
    scopes: crate::domain::input::ActiveScopes,
) -> Option<Message> {
    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
        return Some(Message::Exit);
    }
    let id = key_id(key)?;
    map.lookup_key(&id, mods(modifiers), scopes).map(action_message)
}

pub(crate) fn mouse_button(
    button: iced::mouse::Button,
) -> Option<crate::domain::input::MouseButton> {
    use crate::domain::input::MouseButton;
    match button {
        iced::mouse::Button::Left => Some(MouseButton::Left),
        iced::mouse::Button::Right => Some(MouseButton::Right),
        iced::mouse::Button::Middle => Some(MouseButton::Middle),
        _ => None,
    }
}

pub(crate) fn key_id(key: &keyboard::Key) -> Option<crate::domain::input::KeyId> {
    use crate::domain::input::KeyId;
    match key {
        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => Some(KeyId::Left),
        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => Some(KeyId::Right),
        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(KeyId::Up),
        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some(KeyId::Down),
        keyboard::Key::Named(keyboard::key::Named::Enter) => Some(KeyId::Enter),
        keyboard::Key::Named(keyboard::key::Named::Tab) => Some(KeyId::Tab),
        keyboard::Key::Named(keyboard::key::Named::Home) => Some(KeyId::Home),
        keyboard::Key::Named(keyboard::key::Named::End) => Some(KeyId::End),
        keyboard::Key::Named(keyboard::key::Named::PageUp) => Some(KeyId::PageUp),
        keyboard::Key::Named(keyboard::key::Named::PageDown) => Some(KeyId::PageDown),
        keyboard::Key::Named(keyboard::key::Named::Space) => Some(KeyId::Char(" ".to_string())),
        keyboard::Key::Character(ch) => Some(KeyId::Char(ch.to_lowercase())),
        _ => None,
    }
}

pub fn style(_app: &App, theme: &iced::Theme) -> iced::theme::Style {
    iced::theme::Style { background_color: Color::TRANSPARENT, text_color: theme.palette().text }
}
