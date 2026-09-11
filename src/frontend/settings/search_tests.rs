use super::*;
use crate::frontend::settings::test_source::FakeSettingsSource;

#[test]
fn search_aliases_across_tabs() {
    let cfg = FakeSettingsSource::default();
    let empty = Vec::new();
    let battery = search_settings("laptop power", &cfg, &empty, &empty, "", &empty);
    assert!(!battery.is_empty());
    assert!(battery.iter().any(|result| result.tab == "performance"));
    let frame_rate = search_settings("frame rate", &cfg, &empty, &empty, "", &empty);
    assert!(frame_rate.iter().any(|result| result.title.to_lowercase().contains("fps")));
    let engine = search_settings("we workshop", &cfg, &empty, &empty, "", &empty);
    assert!(engine.iter().any(|result| result.tab == "playback"));
}

#[test]
fn blank_search_empty() {
    let cfg = FakeSettingsSource::default();
    assert!(search_settings("   ", &cfg, &[], &[], "", &[]).is_empty());
}

#[test]
fn nested_source_controls_match_case_insensitive_search() {
    let cfg = FakeSettingsSource::default();
    for query in ["show apply button", "SHOW APPLY BUTTON", "Show Apply button"] {
        let results = search_settings(query, &cfg, &[], &[], "", &[]);
        assert_eq!(results.len(), 5, "{query}");
        assert!(results.iter().all(|result| result.tab == "sources"));
    }
}
