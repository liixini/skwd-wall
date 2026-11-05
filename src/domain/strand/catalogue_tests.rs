use super::{KINDS, style_index};

#[test]
fn styles_indexed() {
    for kind in KINDS {
        for (index, face) in kind.styles().iter().enumerate() {
            assert_eq!(style_index(*face), index, "{face:?}");
        }
    }
}
