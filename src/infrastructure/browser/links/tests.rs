use super::{WorkshopLink, workshop_url};

#[test]
fn restricts_link_targets() {
    assert_eq!(
        workshop_url(&WorkshopLink::Video("XUvDFsz4ABk".into())).as_deref(),
        Some("https://www.youtube.com/watch?v=XUvDFsz4ABk")
    );
    for link in [
        WorkshopLink::Item("0".into()),
        WorkshopLink::Item("12&evil=1".into()),
        WorkshopLink::Creator("../bad".into()),
        WorkshopLink::Video("https://bad/".into()),
    ] {
        assert!(workshop_url(&link).is_none());
    }
}
