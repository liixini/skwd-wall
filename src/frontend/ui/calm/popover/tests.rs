use iced::{Point, Rectangle, Size};

use super::{Side, position};

const SCREEN: Size = Size::new(1366.0, 768.0);
const MENU: Size = Size::new(180.0, 120.0);

fn token() -> Rectangle {
    Rectangle::new(Point::new(400.0, 200.0), Size::new(60.0, 21.0))
}

#[test]
fn menu_hangs_below() {
    assert_eq!(position(token(), Side::Below, false, MENU, SCREEN), Point::new(390.0, 227.0));
    assert_eq!(position(token(), Side::Below, true, MENU, SCREEN), Point::new(290.0, 227.0));
}

#[test]
fn body_menu_ends_flush() {
    assert_eq!(position(token(), Side::BelowEnd, false, MENU, SCREEN), Point::new(280.0, 227.0));
    assert_eq!(position(token(), Side::BelowEnd, true, MENU, SCREEN), Point::new(400.0, 227.0));
}

#[test]
fn lower_menu_rises() {
    assert_eq!(position(token(), Side::AboveEnd, false, MENU, SCREEN), Point::new(280.0, 74.0));
    assert_eq!(position(token(), Side::AboveEnd, true, MENU, SCREEN), Point::new(400.0, 74.0));
}

#[test]
fn editor_covers_token() {
    let editor = Size::new(60.0, 21.0);
    assert_eq!(position(token(), Side::Over, false, editor, SCREEN), Point::new(400.0, 200.0));
    assert_eq!(
        position(token(), Side::Over, true, Size::new(80.0, 21.0), SCREEN),
        Point::new(380.0, 200.0)
    );
}

#[test]
fn stays_on_screen() {
    let edge = Rectangle::new(Point::new(1340.0, 740.0), Size::new(20.0, 21.0));
    assert_eq!(position(edge, Side::Below, false, MENU, SCREEN), Point::new(1186.0, 648.0));
    let start = Rectangle::new(Point::new(4.0, 4.0), Size::new(20.0, 21.0));
    assert_eq!(position(start, Side::Below, false, MENU, SCREEN).x, 0.0);
}
