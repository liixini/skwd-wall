use super::*;

#[test]
fn bar_reveal_survives_close() {
    let motion = MotionProfile::default();
    let mut state = SettingsState::default();
    state.toggle_bar(String::from("field"), motion);
    state.tick_bars(0.05);
    state.tick_bars(0.05);
    let open_x = state.bar_reveals["field"].x;
    assert!(open_x > 0.0 && open_x < 1.0);

    state.toggle_bar(String::from("field"), motion);
    state.tick_bars(0.05);
    assert!(state.bar_reveals["field"].x < open_x);
    assert!(state.bar_reveals.contains_key("field"));
    state.tick_bars(0.05);
    assert!(!state.bar_reveals.contains_key("field"));
}

const LIBRARY: SectionId = SectionId::new("library", "settings-section-folders");
const PROFILES: SectionId = SectionId::new("library", "settings-profiles-card");
const FOLD_SECONDS: f32 = 0.39;
const FRAME: f32 = 0.016;

fn run(state: &mut SettingsState, seconds: f32) {
    let mut left = seconds;
    while left > 0.0 {
        state.tick_sections(FRAME.min(left));
        left -= FRAME;
    }
}

#[test]
fn one_section_open() {
    let mut state = SettingsState::default();
    state.toggle_section(LIBRARY, false);
    assert_eq!(state.open_section, Some(LIBRARY));
    run(&mut state, FOLD_SECONDS);
    assert!(!state.sections_animating());
    state.toggle_section(PROFILES, false);
    assert_eq!((state.open_section, state.queued_section), (None, Some(PROFILES)));
    run(&mut state, FOLD_SECONDS - FRAME * 2.0);
    assert_eq!(state.open_section, None, "still closing");
    run(&mut state, FRAME * 3.0);
    assert_eq!((state.open_section, state.queued_section), (Some(PROFILES), None));
    run(&mut state, FOLD_SECONDS);
    assert!(!state.sections_animating());
}

#[test]
fn toggle_closes_open() {
    let mut state = SettingsState::default();
    state.toggle_section(LIBRARY, false);
    run(&mut state, FOLD_SECONDS);
    state.toggle_section(LIBRARY, false);
    assert_eq!((state.open_section, state.queued_section), (None, None));
    run(&mut state, FOLD_SECONDS);
    assert!(!state.sections_animating());
}

#[test]
fn busy_follows_open() {
    let mut state = SettingsState::default();
    state.toggle_section(LIBRARY, false);
    run(&mut state, FOLD_SECONDS / 2.0);
    assert!(state.busy.x > 0.4 && state.busy.x < 0.6, "busy {}", state.busy.x);
    run(&mut state, FOLD_SECONDS);
    assert_eq!(state.busy.x, 1.0);
    state.toggle_section(PROFILES, false);
    run(&mut state, FOLD_SECONDS - FRAME * 2.0);
    assert!(state.busy.x < 0.1, "rows regrow while closing");
    run(&mut state, FOLD_SECONDS + FRAME * 3.0);
    assert_eq!(state.busy.x, 1.0);
}

#[test]
fn instant_skips_queue() {
    let mut state = SettingsState::default();
    state.toggle_section(LIBRARY, true);
    state.toggle_section(PROFILES, true);
    assert_eq!((state.open_section, state.queued_section), (Some(PROFILES), None));
    assert!(!state.sections_animating());
}

#[test]
fn toggling_clears_popovers() {
    let mut state = SettingsState { pop: Some(1), edit: Some(2), ..SettingsState::default() };
    state.toggle_section(LIBRARY, false);
    assert_eq!((state.pop, state.edit), (None, None));
}

#[test]
fn third_click_keeps_closing() {
    let images = SectionId::new("library", "settings-section-images-recovery");
    let mut state = SettingsState::default();
    state.toggle_section(LIBRARY, false);
    run(&mut state, FOLD_SECONDS);
    state.toggle_section(PROFILES, false);
    run(&mut state, FOLD_SECONDS / 2.0);
    state.toggle_section(images, false);
    assert_eq!(state.closing_section, Some(LIBRARY));
    assert_eq!((state.open_section, state.queued_section), (None, Some(images)));
    run(&mut state, FOLD_SECONDS / 2.0 + FRAME * 2.0);
    assert_eq!((state.open_section, state.closing_section), (Some(images), None));
    run(&mut state, FOLD_SECONDS);
    assert!(!state.sections_animating());
}

#[test]
fn pending_tab_canonical() {
    let state = SettingsState {
        pending: Some(SettingsNavigation::Page { tab: String::from("performance") }),
        ..SettingsState::default()
    };
    assert_eq!(state.navigation_tab(), crate::frontend::settings::place::SYSTEM);
}
