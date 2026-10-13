use std::os::unix::process::ExitStatusExt;

use super::*;

#[test]
fn preserves_path_spaces() {
    let output = Output {
        status: std::process::ExitStatus::from_raw(0),
        stdout: b"/walls/ padded folder /\n".to_vec(),
        stderr: Vec::new(),
    };
    assert_eq!(selected(output).unwrap(), Some(PathBuf::from("/walls/ padded folder /")));
}

#[test]
fn cancel_leaves_selection_empty() {
    let output = Output {
        status: std::process::ExitStatus::from_raw(1 << 8),
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    assert_eq!(selected(output).unwrap(), None);
}
