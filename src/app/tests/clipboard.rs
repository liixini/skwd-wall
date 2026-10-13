use super::*;
use crate::app::message::CopyKind;

#[test]
fn copy_feedback_preserves_picker() {
    let mut app = test_app();
    seed(&mut app, &[wall("image.png", "static", 1, 0)]);
    app.scene.set_flipped_for_test(Some(0));
    for (kind, result, expected) in [
        (CopyKind::Image, Ok(()), "Image copied"),
        (CopyKind::Path, Ok(()), "Absolute path copied"),
        (CopyKind::Image, Err("decode failed".into()), "Could not copy image: decode failed"),
        (CopyKind::Path, Err("missing file".into()), "Could not copy path: missing file"),
    ] {
        let _ = update(&mut app, Message::WallpaperCopied { kind, result });
        assert_eq!(app.runtime_state.toast.as_ref().unwrap().0, expected);
        assert_eq!(app.scene.flipped(), Some(0));
        assert!(drain_calls(&app).is_empty());
    }
}

#[test]
fn copy_rejects_unavailable_items() {
    let mut app = test_app();
    seed(&mut app, &[wall("video.mp4", "video", 1, 0), wall("image.png", "static", 1, 0)]);
    edit_catalog(&mut app, |catalog| {
        catalog.items[0].path = "/video.mp4".into();
        catalog.items[1].path.clear();
    });
    for (index, kind) in
        [(0, CopyKind::Image), (1, CopyKind::Image), (1, CopyKind::Path), (2, CopyKind::Path)]
    {
        let task =
            crate::app::update::update_inner(&mut app, Message::CopyWallpaper { index, kind });
        assert!(iced_runtime::task::into_stream(task).is_none());
        assert!(app.runtime_state.toast.is_none());
    }
}
