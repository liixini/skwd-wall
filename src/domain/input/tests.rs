use super::action::InputScope;
use super::{
    ActiveScopes, InputAction, InputMap, KeyId, KeySpec, Mods, MouseButton, MouseSpec, NavKeys,
    Trigger, binding_config, binding_label, parse_binding, slot_triggers, with_slot, without,
};

const PICKER: ActiveScopes = ActiveScopes { fields: false, search: false, downloads: false };
const SEARCH: ActiveScopes = ActiveScopes { fields: true, search: true, downloads: false };
const DOWNLOADS: ActiveScopes = ActiveScopes { fields: true, search: false, downloads: true };

fn holders(map: &InputMap, action: InputAction, trigger: &str) -> Vec<InputAction> {
    map.trigger_holders(action, &key(trigger)).collect()
}

fn key(text: &str) -> Trigger {
    Trigger::parse(text).expect("valid trigger")
}

#[test]
fn parser_accepts_triggers() {
    assert_eq!(
        Trigger::parse("p"),
        Some(Trigger::Key(KeySpec { mods: Mods::NONE, id: KeyId::Char("p".into()) }))
    );
    assert_eq!(
        Trigger::parse(" Shift + LEFT "),
        Some(Trigger::Key(KeySpec { mods: Mods::new(false, false, true), id: KeyId::Left }))
    );
    assert_eq!(
        Trigger::parse("shift++"),
        Some(Trigger::Key(KeySpec {
            mods: Mods::new(false, false, true),
            id: KeyId::Char("+".into())
        }))
    );
    assert_eq!(
        Trigger::parse("click"),
        Some(Trigger::Mouse(MouseSpec { mods: Mods::NONE, button: MouseButton::Left }))
    );
    assert_eq!(
        Trigger::parse("ctrl+right-click"),
        Some(Trigger::Mouse(MouseSpec {
            mods: Mods::new(true, false, false),
            button: MouseButton::Right
        }))
    );
    assert_eq!(
        Trigger::parse("middle-click"),
        Some(Trigger::Mouse(MouseSpec { mods: Mods::NONE, button: MouseButton::Middle }))
    );
}

#[test]
fn parser_rejects_invalid() {
    for invalid in ["", "banana+x", "pp", "shift+", "insert", "scroll", "click+ctrl"] {
        assert_eq!(Trigger::parse(invalid), None);
    }
}

#[test]
fn label_config_round_trip() {
    for (raw, shown) in [
        ("shift+left", "Shift+Left"),
        ("ctrl+alt+shift+d", "Ctrl+Alt+Shift+D"),
        ("enter", "Enter"),
        ("return", "Enter"),
        ("tab", "Tab"),
        ("space", "Space"),
        ("click", "Click"),
        ("right-click", "Right click"),
        ("ctrl+click", "Ctrl+Click"),
        ("shift+middle-click", "Shift+Middle click"),
    ] {
        let trigger = Trigger::parse(raw).expect(raw);
        assert_eq!(crate::i18n::with_language("en-US", || trigger.label()), shown);
        assert_eq!(Trigger::parse(&trigger.config()), Some(trigger));
    }
    assert_eq!(key("ctrl+alt+shift+d").config(), "ctrl+alt+shift+d");
}

#[test]
fn bindings_hold_several_triggers() {
    let triggers = parse_binding("i, right-click").expect("valid binding");
    assert_eq!(triggers, vec![key("i"), key("right-click")]);
    assert_eq!(binding_config(&triggers), "i, right-click");
    assert_eq!(
        crate::i18n::with_language("en-US", || binding_label(&triggers)),
        "I \u{b7} Right click"
    );
    assert_eq!(
        crate::i18n::with_language("fr-FR", || binding_label(&triggers)),
        "I \u{b7} Clic droit"
    );
    assert_eq!(parse_binding("none"), Some(Vec::new()));
    assert_eq!(binding_config(&[]), "none");
    assert_eq!(parse_binding(""), None);
    assert_eq!(parse_binding("i, banana"), None);
}

#[test]
fn defaults_reachable() {
    for action in InputAction::ALL {
        assert!(parse_binding(action.default_binding()).is_some(), "{action:?}");
    }
    let map = InputMap::default();
    assert_eq!(
        map.lookup_key(&KeyId::Char("p".into()), Mods::NONE, PICKER),
        Some(InputAction::Playlists)
    );
    assert_eq!(
        map.lookup_key(&KeyId::Char("p".into()), Mods::new(false, false, true), PICKER),
        Some(InputAction::Playlists)
    );
    assert_eq!(
        map.lookup_key(&KeyId::Char("p".into()), Mods::new(true, false, false), PICKER),
        None
    );
    assert_eq!(map.lookup_key(&KeyId::Enter, Mods::NONE, PICKER), Some(InputAction::Apply));
    assert_eq!(
        map.lookup_key(&KeyId::Char("c".into()), Mods::NONE, PICKER),
        Some(InputAction::ColourStudio)
    );
    assert_eq!(
        map.lookup_key(&KeyId::Char("c".into()), Mods::new(false, false, true), PICKER),
        Some(InputAction::ThemePanel)
    );
    assert_eq!(map.lookup_key(&KeyId::Tab, Mods::NONE, PICKER), Some(InputAction::TypeNext));
    assert_eq!(map.lookup_key(&KeyId::Left, Mods::NONE, PICKER), Some(InputAction::NavLeft));
    assert_eq!(
        map.lookup_key(&KeyId::Left, Mods::new(false, false, true), PICKER),
        Some(InputAction::ColorPrev)
    );
}

#[test]
fn clicks_resolve_like_keys() {
    let map = InputMap::default();
    let click = |mods, button| map.lookup_mouse(MouseSpec { mods, button }, PICKER);
    assert_eq!(click(Mods::NONE, MouseButton::Left), Some(InputAction::Select));
    assert_eq!(click(Mods::NONE, MouseButton::Right), Some(InputAction::Flip));
    assert_eq!(click(Mods::new(true, false, false), MouseButton::Left), Some(InputAction::Effects));
    assert_eq!(click(Mods::new(false, false, true), MouseButton::Right), Some(InputAction::Studio));
    assert_eq!(click(Mods::NONE, MouseButton::Middle), None);
    assert_eq!(click(Mods::new(false, true, false), MouseButton::Left), None);
}

#[test]
fn any_trigger_kind() {
    let map = InputMap::from_bindings([
        (InputAction::Playlists, "middle-click".to_string()),
        (InputAction::Select, "u".to_string()),
        (InputAction::Studio, "ctrl+alt+shift+d".to_string()),
    ]);
    assert_eq!(
        map.lookup_mouse(MouseSpec { mods: Mods::NONE, button: MouseButton::Middle }, PICKER),
        Some(InputAction::Playlists)
    );
    assert_eq!(
        map.lookup_key(&KeyId::Char("u".into()), Mods::NONE, PICKER),
        Some(InputAction::Select)
    );
    assert_eq!(
        map.lookup_key(&KeyId::Char("d".into()), Mods::new(true, true, true), PICKER),
        Some(InputAction::Studio)
    );
    assert_eq!(
        map.lookup_mouse(MouseSpec { mods: Mods::NONE, button: MouseButton::Left }, PICKER),
        None
    );
}

#[test]
fn invalid_override_fallback() {
    let map = InputMap::from_bindings([
        (InputAction::Playlists, "f".to_string()),
        (InputAction::Flip, "banana+q".to_string()),
    ]);
    assert_eq!(
        map.lookup_mouse(MouseSpec { mods: Mods::NONE, button: MouseButton::Right }, PICKER),
        Some(InputAction::Flip)
    );
    assert_eq!(map.conflicts(), vec![(InputAction::Favourite, InputAction::Playlists)]);
    assert_eq!(holders(&map, InputAction::Playlists, "f"), vec![InputAction::Favourite]);
    assert_eq!(holders(&map, InputAction::Playlists, "z"), Vec::new());
}

#[test]
fn card_actions_need_target() {
    for action in [
        InputAction::Select,
        InputAction::Apply,
        InputAction::Flip,
        InputAction::Favourite,
        InputAction::Effects,
        InputAction::Studio,
    ] {
        assert!(action.targets_card(), "{action:?}");
    }
    for action in [
        InputAction::Playlists,
        InputAction::Settings,
        InputAction::ThemePanel,
        InputAction::NavLeft,
    ] {
        assert!(!action.targets_card(), "{action:?}");
    }
}

#[test]
fn defaults_have_no_conflicts() {
    assert_eq!(InputMap::default().conflicts(), Vec::new());
}

#[test]
fn scoped_defaults_resolve_by_context() {
    let map = InputMap::default();
    let ctrl = Mods::new(true, false, false);
    let alt = Mods::new(false, true, false);
    let shift = Mods::new(false, false, true);
    let tab = |mods, scopes| map.lookup_key(&KeyId::Tab, mods, scopes);
    assert_eq!(tab(Mods::NONE, PICKER), Some(InputAction::TypeNext));
    assert_eq!(tab(shift, PICKER), Some(InputAction::TypePrev));
    assert_eq!(tab(Mods::NONE, SEARCH), Some(InputAction::Autocomplete));
    assert_eq!(tab(Mods::NONE, DOWNLOADS), Some(InputAction::Autocomplete));
    assert_eq!(tab(shift, SEARCH), None);
    assert_eq!(tab(ctrl, SEARCH), Some(InputAction::SearchMode));
    assert_eq!(tab(ctrl, PICKER), None);
    assert_eq!(tab(ctrl, DOWNLOADS), None);
    assert_eq!(map.lookup_key(&KeyId::Left, alt, PICKER), Some(InputAction::SortPrev));
    assert_eq!(map.lookup_key(&KeyId::Right, alt, SEARCH), Some(InputAction::SortNext));
    let char_key =
        |text: &str, mods, scopes| map.lookup_key(&KeyId::Char(text.into()), mods, scopes);
    assert_eq!(char_key("r", ctrl, PICKER), Some(InputAction::RandomRotate));
    assert_eq!(char_key("r", ctrl, SEARCH), Some(InputAction::RandomRotate));
    assert_eq!(char_key("d", ctrl, PICKER), Some(InputAction::Downloads));
    assert_eq!(char_key("d", ctrl, DOWNLOADS), Some(InputAction::Downloads));
    for (digit, action) in [
        ("1", InputAction::SourceWallhaven),
        ("2", InputAction::SourceSteam),
        ("3", InputAction::SourceUnsplash),
        ("4", InputAction::SourcePexels),
        ("5", InputAction::SourceYoutube),
        ("6", InputAction::SourceBing),
    ] {
        assert_eq!(char_key(digit, Mods::NONE, DOWNLOADS), Some(action));
        assert_eq!(char_key(digit, Mods::NONE, PICKER), None);
        assert_eq!(char_key(digit, Mods::NONE, SEARCH), None);
    }
}

#[test]
fn scopes_limit_conflicts_and_stealing() {
    assert!(!InputScope::Picker.overlaps(InputScope::Fields));
    assert!(!InputScope::Downloads.overlaps(InputScope::Picker));
    assert!(InputScope::Everywhere.overlaps(InputScope::Picker));
    assert!(InputScope::Search.overlaps(InputScope::Fields));
    let map = InputMap::from_bindings([(InputAction::SourceSteam, "1".to_string())]);
    assert_eq!(map.conflicts(), vec![(InputAction::SourceWallhaven, InputAction::SourceSteam)]);
    let map = InputMap::from_bindings([(InputAction::Playlists, "1".to_string())]);
    assert_eq!(map.conflicts(), vec![(InputAction::Playlists, InputAction::SourceWallhaven)]);
    let map = InputMap::default();
    assert_eq!(holders(&map, InputAction::Autocomplete, "tab"), Vec::new());
    assert_eq!(holders(&map, InputAction::TypeNext, "ctrl+d"), vec![InputAction::Downloads]);
    assert_eq!(
        holders(&map, InputAction::Playlists, "tab"),
        vec![InputAction::Autocomplete, InputAction::TypeNext]
    );
}

#[test]
fn second_nav_keys_and_mode_cycle_defaults() {
    let map = InputMap::default();
    let shift = Mods::new(false, false, true);
    let char_key =
        |text: &str, mods, scopes| map.lookup_key(&KeyId::Char(text.into()), mods, scopes);
    for (text, action) in [
        ("w", InputAction::NavUp),
        ("a", InputAction::NavLeft),
        ("s", InputAction::NavDown),
        ("d", InputAction::NavRight),
        ("/", InputAction::TagCloud),
        ("m", InputAction::ModeNext),
    ] {
        assert_eq!(char_key(text, Mods::NONE, PICKER), Some(action), "{text}");
    }
    assert_eq!(char_key("m", shift, PICKER), Some(InputAction::ModePrev));
    assert_eq!(char_key("s", shift, PICKER), Some(InputAction::Settings));
    assert_eq!(char_key("w", shift, PICKER), Some(InputAction::SceneProperties));
    assert_eq!(char_key("/", shift, PICKER), Some(InputAction::TagCloud));
    assert_eq!(char_key("m", Mods::NONE, SEARCH), None);
    assert_eq!(char_key("m", Mods::NONE, DOWNLOADS), None);
    assert_eq!(map.lookup_key(&KeyId::Left, Mods::NONE, PICKER), Some(InputAction::NavLeft));
    assert_eq!(map.lookup_key(&KeyId::Down, shift, PICKER), Some(InputAction::TagCloud));
}

#[test]
fn slots_split_primary_from_the_rest() {
    let triggers = parse_binding("left, a, h").expect("valid binding");
    assert_eq!(slot_triggers(&triggers, 0), &[key("left")]);
    assert_eq!(slot_triggers(&triggers, 1), &[key("a"), key("h")]);
    assert!(slot_triggers(&[], 0).is_empty());
    assert!(slot_triggers(&[], 1).is_empty());
    assert_eq!(slot_triggers(&triggers[..1], 1), &[] as &[Trigger]);
}

#[test]
fn replacing_one_slot_keeps_the_other() {
    let triggers = parse_binding("left, a").expect("valid binding");
    assert_eq!(with_slot(&triggers, 1, Some(key("h"))), vec![key("left"), key("h")]);
    assert_eq!(with_slot(&triggers, 0, Some(key("j"))), vec![key("j"), key("a")]);
    assert_eq!(with_slot(&triggers, 1, None), vec![key("left")]);
    assert_eq!(with_slot(&triggers, 0, None), vec![key("a"), key("a")]);
    assert_eq!(with_slot(&triggers, 1, Some(key("left"))), vec![key("left")]);
    assert_eq!(with_slot(&[], 1, Some(key("h"))), vec![key("h"), key("h")]);
    let extra = parse_binding("left, a, h").expect("valid binding");
    assert_eq!(with_slot(&extra, 1, Some(key("k"))), vec![key("left"), key("k")]);
    assert_eq!(with_slot(&extra, 0, Some(key("up"))), vec![key("up"), key("a"), key("h")]);
}

#[test]
fn nav_key_sets_cover_every_direction() {
    for keys in NavKeys::ALL {
        let mut actions: Vec<_> = keys.second_keys().map(|(action, _)| action).to_vec();
        actions.sort_by_key(|action| InputAction::ALL.iter().position(|each| each == action));
        assert_eq!(
            actions,
            [InputAction::NavLeft, InputAction::NavRight, InputAction::NavUp, InputAction::NavDown]
        );
        for (action, text) in keys.second_keys() {
            assert!(Trigger::parse(text).is_some(), "{keys:?} {action:?}");
        }
    }
    let wasd = NavKeys::Wasd.second_keys();
    let map = InputMap::default();
    for (action, text) in wasd {
        assert_eq!(slot_triggers(map.triggers(action), 1), &[key(text)], "{action:?}");
    }
}

#[test]
fn page_and_edge_keys_parse_and_round_trip() {
    for (text, id, label) in [
        ("home", KeyId::Home, "Home"),
        ("end", KeyId::End, "End"),
        ("pageup", KeyId::PageUp, "Page Up"),
        ("pagedown", KeyId::PageDown, "Page Down"),
    ] {
        let trigger = key(text);
        assert_eq!(trigger, Trigger::Key(KeySpec { mods: Mods::NONE, id: id.clone() }));
        assert_eq!(trigger.label(), label);
        assert_eq!(trigger.config(), text);
    }
    let map = InputMap::default();
    for (id, action) in [
        (KeyId::Home, InputAction::JumpFirst),
        (KeyId::End, InputAction::JumpLast),
        (KeyId::PageUp, InputAction::PageBack),
        (KeyId::PageDown, InputAction::PageForward),
    ] {
        assert_eq!(map.lookup_key(&id, Mods::NONE, PICKER), Some(action));
    }
}

#[test]
fn empty_first_slot() {
    let lone = with_slot(&[], 1, Some(key("h")));
    assert!(slot_triggers(&lone, 0).is_empty());
    assert_eq!(slot_triggers(&lone, 1), &[key("h")]);
    let both = with_slot(&lone, 0, Some(key("left")));
    assert_eq!(both, vec![key("left"), key("h")]);
    let cleared = with_slot(&both, 0, None);
    assert!(slot_triggers(&cleared, 0).is_empty());
    assert_eq!(slot_triggers(&cleared, 1), &[key("h")]);
    assert!(with_slot(&cleared, 1, None).is_empty());
    assert_eq!(binding_label(&lone), "H");
}

#[test]
fn stealing_keeps_slot_places() {
    let pair = parse_binding("left, a").expect("valid binding");
    let robbed = without(&pair, &key("left"));
    assert!(slot_triggers(&robbed, 0).is_empty());
    assert_eq!(slot_triggers(&robbed, 1), &[key("a")]);
    let robbed = without(&pair, &key("a"));
    assert_eq!(slot_triggers(&robbed, 0), &[key("left")]);
    assert!(slot_triggers(&robbed, 1).is_empty());
    assert!(without(&robbed, &key("left")).is_empty());
}

#[test]
fn empty_slot_config() {
    let lone = with_slot(&[], 1, Some(key("h")));
    let text = binding_config(&lone);
    assert_eq!(text, "h, h");
    assert_eq!(parse_binding(&text), Some(lone));
    for text in ["left, a", "left", "none", "left, a, h"] {
        let read = parse_binding(text).expect(text);
        let kept = if text == "none" { Vec::new() } else { read.clone() };
        assert_eq!(binding_config(&kept), text);
        assert_eq!(slot_triggers(&read, 0).len(), usize::from(text != "none"));
    }
}

#[test]
fn older_reader_key() {
    let lone = with_slot(&[], 1, Some(key("h")));
    let map = InputMap::from_bindings([(InputAction::NavLeft, binding_config(&lone))]);
    assert_eq!(
        map.lookup_key(&KeyId::Char("h".into()), Mods::NONE, PICKER),
        Some(InputAction::NavLeft)
    );
    assert_eq!(map.triggers(InputAction::NavLeft), &[key("h"), key("h")]);
    assert_eq!(parse_binding("none, h"), None);
}
