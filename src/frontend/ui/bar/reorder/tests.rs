use iced::{Point, Rectangle};

use super::*;
use crate::contracts::daemon::{TaskControl, TaskState};
use crate::frontend::ui::bar::action::BarAction;
use crate::frontend::ui::bar::edit_id;
use crate::frontend::ui::bar::model::BarNotice;

const BAR_X: f32 = 100.0;
const BAR_Y: f32 = 200.0;
const ITEM_HEIGHT: f32 = 24.0;
const MOTION_STEP: f32 = 1.0 / 60.0;
const SETTLE_STEPS: usize = 90;

fn item(x: f32, y: f32, width: f32, action: BarAction) -> BarItem {
    BarItem {
        x,
        y,
        w: width,
        h: ITEM_HEIGHT,
        skew: 0.0,
        label: String::from("Button"),
        nerd: false,
        text_size: 12.0,
        swatch: None,
        notice: None,
        active: false,
        action: Some(action),
        z: 0,
    }
}

fn model(vertical: bool, rtl: bool) -> BarModel {
    let positions = if vertical {
        [(0.0, 0.0), (0.0, 27.0), (0.0, 54.0)]
    } else if rtl {
        [(130.0, 0.0), (70.0, 0.0), (0.0, 0.0)]
    } else {
        [(0.0, 0.0), (50.0, 0.0), (120.0, 0.0)]
    };
    BarModel {
        items: [
            item(positions[0].0, positions[0].1, 50.0, BarAction::Favs),
            item(positions[1].0, positions[1].1, 70.0, BarAction::Settings),
            item(positions[2].0, positions[2].1, 60.0, BarAction::Stash),
        ]
        .into(),
        width: 180.0,
        height: if vertical { 81.0 } else { ITEM_HEIGHT },
        menu_up: false,
        menu_visible_rows: 0,
        swatch_at: None,
        rtl,
    }
}

fn preview(vertical: bool, rtl: bool) -> ReorderPreview {
    let model = model(vertical, rtl);
    let bar = Rectangle { x: BAR_X, y: BAR_Y, width: model.width, height: model.height };
    let first = &model.items[0];
    let point = Point::new(BAR_X + first.x + 10.0, BAR_Y + first.y + 8.0);
    ReorderPreview::new(model, bar, 0, point, vertical, 1.0, MotionProfile::default()).unwrap()
}

#[test]
fn dragged_item_keeps_offset() {
    let mut reorder = preview(false, false);
    let pointer = Point::new(275.0, 260.0);
    reorder.move_to(pointer);
    let drawn = reorder.edit_bar(BarVisualStyle::Slices, &[]);
    assert_eq!(drawn.items[0].x, pointer.x - 10.0);
    assert_eq!(drawn.items[0].y, pointer.y - 8.0);
    assert_eq!(drawn.lifted, 0);
}

#[test]
fn insertion_resists_jitter() {
    let mut reorder = preview(false, false);
    reorder.move_to(Point::new(190.0, 208.0));
    let order = reorder.order();
    for x in [187.0, 191.0, 188.0, 192.0] {
        reorder.move_to(Point::new(x, 208.0));
        assert_eq!(reorder.order(), order);
    }
}

#[test]
fn reverse_drag_tracks_gap() {
    let mut reorder = preview(false, false);
    reorder.move_to(Point::new(270.0, 208.0));
    let mut target = model(false, false);
    target.items[0].x = 130.0;
    target.items[1].x = 0.0;
    target.items[2].x = 70.0;
    let bar = Rectangle { x: BAR_X, y: BAR_Y, width: target.width, height: target.height };
    reorder.retarget(&target, bar);
    let pointer = Point::new(160.0, 208.0);
    reorder.move_to(pointer);
    assert_eq!(reorder.order(), ["settings", "favourites", "stash"]);
    target.items[0].x = 70.0;
    target.items[2].x = 120.0;
    reorder.retarget(&target, bar);
    for _ in 0..SETTLE_STEPS {
        reorder.tick(MOTION_STEP);
        assert!(!reorder.move_to(pointer));
        assert_eq!(reorder.order(), ["settings", "favourites", "stash"]);
    }
    reorder.move_to(Point::new(110.0, 208.0));
    assert_eq!(reorder.order(), ["favourites", "settings", "stash"]);
    target.items[0].x = 0.0;
    target.items[1].x = 50.0;
    reorder.retarget(&target, bar);
    reorder.move_to(Point::new(270.0, 208.0));
    assert_eq!(reorder.order(), ["settings", "stash", "favourites"]);
}

#[test]
fn wrapped_gap_keeps_slot() {
    let initial = BarModel {
        items: vec![
            item(0.0, 0.0, 53.2, BarAction::SetType { key: "", next: String::new() }),
            item(222.4, 0.0, 155.6, BarAction::FolderToggle),
            item(0.0, 30.0, 49.4, BarAction::Download),
            item(39.4, 30.0, 49.4, BarAction::Playlists),
            item(78.8, 30.0, 49.4, BarAction::Settings),
            item(118.2, 30.0, 49.4, BarAction::Stash),
        ],
        width: 775.6,
        height: 54.0,
        menu_up: false,
        menu_visible_rows: 0,
        swatch_at: None,
        rtl: false,
    };
    let bar = Rectangle { x: 12.2, y: 978.0, width: initial.width, height: initial.height };
    let press = Point::new(266.0, 990.0);
    let mut reorder =
        ReorderPreview::new(initial.clone(), bar, 1, press, false, 1.0, MotionProfile::default())
            .unwrap();
    let pointer = Point::new(80.0, 1020.0);
    reorder.slot = 3;
    assert_eq!(reorder.order()[3], edit_id::FOLDER);

    let mut target = initial;
    target.items[0].x = 0.0;
    target.items[1].x = 0.0;
    target.items[1].y = 30.0;
    target.items[2].x = 614.0;
    target.items[2].y = 0.0;
    target.items[3].x = 653.4;
    target.items[3].y = 0.0;
    target.items[4].x = 145.6;
    target.items[5].x = 185.0;
    let target_bar = Rectangle { x: 48.6, y: 978.0, width: 702.8, height: 54.0 };
    reorder.retarget(&target, target_bar);
    for _ in 0..SETTLE_STEPS {
        reorder.tick(MOTION_STEP);
        assert!(!reorder.move_to(pointer));
    }
    reorder.finish();
    assert_eq!(
        reorder.order(),
        [
            edit_id::TYPE_ALL,
            edit_id::DOWNLOAD,
            edit_id::PLAYLISTS,
            edit_id::FOLDER,
            edit_id::SETTINGS,
            edit_id::STASH
        ]
    );
}

#[test]
fn vertical_rtl_reach_end() {
    for (vertical, rtl, point) in
        [(true, false, Point::new(130.0, 275.0)), (false, true, Point::new(110.0, 208.0))]
    {
        let mut reorder = preview(vertical, rtl);
        reorder.move_to(point);
        assert_eq!(reorder.order().last().map(String::as_str), Some("favourites"));
    }
}

#[test]
fn release_settles_motion() {
    let mut reorder = preview(false, false);
    reorder.move_to(Point::new(270.0, 208.0));
    let mut target = model(false, false);
    target.items[0].x = 130.0;
    target.items[1].x = 0.0;
    target.items[2].x = 70.0;
    let bar = Rectangle { x: BAR_X, y: BAR_Y, width: target.width, height: target.height };
    reorder.retarget(&target, bar);
    reorder.finish();
    assert!(reorder.animating());
    for _ in 0..SETTLE_STEPS {
        reorder.tick(MOTION_STEP);
    }
    assert!(reorder.finished());
    let drawn = reorder.edit_bar(BarVisualStyle::Slices, &[]);
    assert_eq!(drawn.items[0].x, BAR_X + target.items[0].x);
}

#[test]
fn auxiliary_items_follow_wrap() {
    let mut initial = model(false, false);
    let mut notice = item(180.0, 0.0, 90.0, BarAction::Favs);
    notice.action = None;
    notice.notice = Some(BarNotice { state: TaskState::Running, progress: Some(0.5) });
    notice.label = String::from("Indexing");
    let control = item(
        270.0,
        0.0,
        45.0,
        BarAction::TaskControl { id: String::from("index"), action: TaskControl::Stop },
    );
    let mut second_notice = item(315.0, 0.0, 80.0, BarAction::Favs);
    second_notice.action = None;
    second_notice.notice = Some(BarNotice { state: TaskState::Paused, progress: None });
    second_notice.label = String::from("Downloading");
    initial.items.extend([notice, control, second_notice]);
    initial.width = 395.0;
    initial.swatch_at = Some((150.0, 30.0));
    let bar = Rectangle { x: BAR_X, y: BAR_Y, width: initial.width, height: initial.height };
    let point = Point::new(BAR_X + 10.0, BAR_Y + 8.0);
    let mut reorder =
        ReorderPreview::new(initial.clone(), bar, 0, point, false, 1.0, MotionProfile::default())
            .unwrap();
    let mut target = initial;
    target.items[3].x = 25.0;
    target.items[3].y = 40.0;
    target.items[4].x = 115.0;
    target.items[4].y = 40.0;
    target.items[5].x = 160.0;
    target.items[5].y = 40.0;
    target.swatch_at = Some((10.0, 70.0));
    let shifted = Rectangle { x: 50.0, y: 240.0, ..bar };
    reorder.retarget(&target, shifted);
    reorder.tick(MOTION_STEP);
    let moving = reorder.edit_bar(BarVisualStyle::Slices, &[]);
    assert!(moving.items[3].x < BAR_X + 180.0);
    assert!(moving.items[3].y > BAR_Y);
    assert!(moving.items[4].x < BAR_X + 270.0);
    assert!(moving.items[5].x < BAR_X + 315.0);
    assert!(moving.swatch_at.unwrap().0 < BAR_X + 150.0);
    reorder.finish();
    for _ in 0..SETTLE_STEPS {
        reorder.tick(MOTION_STEP);
    }
    let settled = reorder.edit_bar(BarVisualStyle::Slices, &[]);
    assert_eq!(settled.items[3].x, shifted.x + target.items[3].x);
    assert_eq!(settled.items[3].y, shifted.y + target.items[3].y);
    assert_eq!(settled.items[4].x, shifted.x + target.items[4].x);
    assert_eq!(settled.items[5].x, shifted.x + target.items[5].x);
    assert_eq!(settled.swatch_at, Some((shifted.x + 10.0, shifted.y + 70.0)));
    assert!(reorder.finished());
}
