#![cfg(test)]

use crate::frontend::settings::build_tab;
use crate::frontend::settings::tables::TABS;
use crate::frontend::settings::test_source::FakeSettingsSource;

#[test]
fn launch_tab_builds() {
    assert!(TABS.contains(&"picker"));
    let cfg = FakeSettingsSource::default();
    assert_eq!(crate::frontend::settings::canonical_place("picker"), Some("layout"));
    let cards = build_tab("picker", &cfg, &[], &[], "", &[]);
    let rows =
        cards.iter().find(|(card, _)| card.title == "Launch").map_or(0, |(_, rows)| rows.len());
    assert!(rows >= 3);
}

#[test]
fn launch_defaults_to_fade() {
    let cfg = FakeSettingsSource::default();
    assert_eq!(crate::contracts::settings::SettingsSource::launch_animation(&cfg), "fade");
}
