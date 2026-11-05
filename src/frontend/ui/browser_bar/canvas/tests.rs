#![cfg(test)]

use iced::Point;

use super::super::model::BrowserAct;
use super::BrowserBar;
use crate::frontend::browser::{BrowserIntent, BrowserMsg};
use crate::frontend::ui::bar::BarItem;

fn item(x: f32, w: f32, skew: f32, z: i32) -> BarItem {
    BarItem {
        x,
        y: 0.0,
        w,
        h: 20.0,
        skew,
        label: String::new(),
        nerd: false,
        text_size: 9.0,
        swatch: None,
        notice: None,
        active: false,
        action: None,
        z,
    }
}

#[test]
fn hit_skew_geometry() {
    let items = vec![(item(100.0, 50.0, 10.0, 0), BrowserAct::Close)];
    assert_eq!(BrowserBar::hit(&items, Point::new(109.0, 0.0)), None);
    assert_eq!(BrowserBar::hit(&items, Point::new(111.0, 0.0)), Some(0));
    assert_eq!(BrowserBar::hit(&items, Point::new(149.0, 0.0)), Some(0));
    assert_eq!(BrowserBar::hit(&items, Point::new(151.0, 0.0)), None);
    assert_eq!(BrowserBar::hit(&items, Point::new(101.0, 20.0)), Some(0));
    assert_eq!(BrowserBar::hit(&items, Point::new(141.0, 20.0)), None);
}

#[test]
fn hit_topmost_skips_labels() {
    let items = vec![
        (item(0.0, 100.0, 0.0, 0), BrowserAct::Close),
        (item(0.0, 100.0, 0.0, 5), BrowserAct::MaxDuration("300")),
        (item(0.0, 100.0, 0.0, 9), BrowserAct::Label),
    ];
    assert_eq!(BrowserBar::hit(&items, Point::new(50.0, 10.0)), Some(1));
    assert_eq!(BrowserBar::hit(&items, Point::new(500.0, 10.0)), None);
}

#[test]
fn actions_emit_browser_feature_intents() {
    assert!(matches!(BrowserBar::message(&BrowserAct::Close), BrowserIntent::Close));
    assert!(matches!(
        BrowserBar::message(&BrowserAct::MaxDuration("300")),
        BrowserIntent::Update(BrowserMsg::SetMaxDuration(duration)) if duration == "300"
    ));
    assert!(matches!(BrowserBar::message(&BrowserAct::Label), BrowserIntent::Capture));
}

#[test]
fn hit_mirrored_skew_geometry() {
    let items = vec![(item(150.0, 50.0, -10.0, 0), BrowserAct::Close)];
    assert_eq!(BrowserBar::hit(&items, Point::new(151.0, 0.0)), Some(0));
    assert_eq!(BrowserBar::hit(&items, Point::new(189.0, 0.0)), Some(0));
    assert_eq!(BrowserBar::hit(&items, Point::new(191.0, 0.0)), None);
    assert_eq!(BrowserBar::hit(&items, Point::new(159.0, 20.0)), None);
    assert_eq!(BrowserBar::hit(&items, Point::new(161.0, 20.0)), Some(0));
    assert_eq!(BrowserBar::hit(&items, Point::new(199.0, 20.0)), Some(0));
}
