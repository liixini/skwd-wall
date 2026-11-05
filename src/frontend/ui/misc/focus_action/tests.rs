use super::*;

#[test]
fn focus_state_changes() {
    use widget::operation::Focusable;
    let mut state = State::default();
    assert!(!state.is_focused());
    state.focus();
    assert!(state.is_focused());
    state.unfocus();
    assert!(!state.is_focused());
}

#[test]
fn enter_activates_focus() {
    let mut event = Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: Key::Named(Named::Enter),
        modified_key: Key::Named(Named::Enter),
        physical_key: iced::keyboard::key::Physical::Unidentified(
            iced::keyboard::key::NativeCode::Unidentified,
        ),
        location: iced::keyboard::Location::Standard,
        modifiers: iced::keyboard::Modifiers::default(),
        text: None,
        repeat: false,
    });
    assert!(activates(&event, true));
    assert!(!activates(&event, false));
    if let Event::Keyboard(iced::keyboard::Event::KeyPressed { modifiers, .. }) = &mut event {
        *modifiers = iced::keyboard::Modifiers::CTRL;
    }
    assert!(!activates(&event, true));
}

#[test]
fn outside_press_blurs_focus() {
    let bounds = Rectangle { x: 10.0, y: 10.0, width: 40.0, height: 30.0 };
    let press = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
    let release = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
    let outside = mouse::Cursor::Available(iced::Point::new(80.0, 80.0));
    let inside = mouse::Cursor::Available(iced::Point::new(20.0, 20.0));
    let mut state = State { focused: true };
    blur_pointer_focus(&mut state, &press, inside, bounds);
    assert!(state.focused);
    blur_pointer_focus(&mut state, &release, outside, bounds);
    assert!(state.focused);
    blur_pointer_focus(&mut state, &press, outside, bounds);
    assert!(!state.focused);
}
