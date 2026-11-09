use super::*;

#[test]
fn fade_then_orientation_switch() {
    let motion = MotionProfile::new(100.0, 200.0, 300.0);
    let mut chrome =
        ChromeState::new(skwd_config::FilterBarVisibility::Always, false, false, motion);
    chrome.filter_bar_visible = false;
    chrome.sync_filter_bar(false);
    assert_eq!(chrome.filter_bar_reveal.target, 0.0);
    chrome.tick_filter_bar(0.05);
    assert!(chrome.filter_bar_fade() < 1.0);

    chrome.filter_bar_visible = true;
    chrome.sync_filter_bar(true);
    assert_eq!(chrome.filter_bar_orientation_reveal.target, 0.0);
    assert!(!chrome.filter_bar_vertical);
    for _ in 0..4 {
        chrome.tick_filter_bar(0.05);
    }
    assert!(chrome.filter_bar_vertical);
    assert_eq!(chrome.filter_bar_orientation_reveal.target, 1.0);
    for _ in 0..4 {
        chrome.tick_filter_bar(0.05);
    }
    assert!(!chrome.filter_bar_animating());
    assert_eq!(chrome.filter_bar_fade(), 1.0);
}
