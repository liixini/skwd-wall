use super::*;

#[test]
fn normalizes_hex() {
    for (raw, expected) in [
        ("f1eee5", Some("#f1eee5")),
        ("#141816", Some("#141816")),
        ("000000ff", Some("#000000")),
        ("chalk", None),
        ("", None),
    ] {
        assert_eq!(normalized(raw).as_deref(), expected, "{raw}");
    }
}

#[test]
fn picker_per_path() {
    assert_ne!(picker_id("drawing.paperColor"), picker_id("drawing.inkColor"));
}
