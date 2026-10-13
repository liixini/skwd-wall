#![cfg(test)]

use super::{desktop_is_niri, profile_argument};

#[test]
fn niri_detect_case_insensitive() {
    assert!(desktop_is_niri("niri"));
    assert!(desktop_is_niri("NIRI"));
    assert!(desktop_is_niri("niri:GNOME"));
    assert!(!desktop_is_niri("Hyprland"));
    assert!(!desktop_is_niri(""));
}

#[test]
fn profile_flag_forms() {
    let parse = |args: &[&str]| profile_argument(args.iter().map(|arg| (*arg).to_owned()));
    assert_eq!(parse(&["--profile", "games"]).as_deref(), Some("games"));
    assert_eq!(parse(&["--debug", "--profile=public"]).as_deref(), Some("public"));
    assert_eq!(parse(&["--profile"]), None);
    assert_eq!(parse(&["--profiles", "x"]), None);
}
