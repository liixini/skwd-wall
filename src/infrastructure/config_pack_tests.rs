use std::path::Path;

use super::*;

#[test]
fn adds_pack_extension_once() {
    assert_eq!(pack_path(PathBuf::from("settings")), Path::new("settings.skwdpack"));
    assert_eq!(pack_path(PathBuf::from("settings.skwdpack")), Path::new("settings.skwdpack"));
}
