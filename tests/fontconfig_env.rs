#[cfg(target_os = "linux")]
#[path = "../src/shell/fontconfig_env.rs"]
mod fontconfig_env;

#[cfg(target_os = "linux")]
use std::ffi::{OsStr, OsString};
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStringExt;

#[cfg(target_os = "linux")]
const FONTCONFIG_FILE: &str = "FONTCONFIG_FILE";

#[cfg(target_os = "linux")]
fn check(cache: &std::path::Path, original: Option<&OsStr>) {
    match original {
        Some(value) => unsafe { std::env::set_var(FONTCONFIG_FILE, value) },
        None => unsafe { std::env::remove_var(FONTCONFIG_FILE) },
    }
    let config = fontconfig_env::with_empty_config(cache, || {
        let path = std::env::var_os(FONTCONFIG_FILE).unwrap();
        let config = std::path::PathBuf::from(path);
        assert!(config.starts_with(cache));
        assert!(config.is_file());
        assert!(config.parent().unwrap().join("empty").is_dir());
        config
    })
    .unwrap();
    assert_eq!(std::env::var_os(FONTCONFIG_FILE).as_deref(), original);
    assert!(!config.exists());
    assert_eq!(std::fs::read_dir(cache).unwrap().count(), 0);
}

#[cfg(target_os = "linux")]
fn main() {
    let root = tempfile::tempdir().unwrap();
    let cache = root.path().join("cache<&");
    check(&cache, None);
    check(&cache, Some(OsStr::new("")));
    check(&cache, Some(OsStr::new("/some/fontconfig.conf")));
    let nonutf_original = OsString::from_vec(b"/nonutf8-\xff.conf".to_vec());
    check(&cache, Some(&nonutf_original));

    let original = OsString::from_vec(b"/before-\xfe.conf".to_vec());
    unsafe { std::env::set_var(FONTCONFIG_FILE, &original) };
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let panic = std::panic::catch_unwind(|| {
        let _ = fontconfig_env::with_empty_config(&cache, || panic!("unwind"));
    });
    std::panic::set_hook(hook);
    assert!(panic.is_err());
    assert_eq!(std::env::var_os(FONTCONFIG_FILE), Some(original.clone()));
    assert_eq!(std::fs::read_dir(&cache).unwrap().count(), 0);

    let blocked = root.path().join("blocked");
    std::fs::write(&blocked, b"blocked").unwrap();
    let scanned = fontconfig_env::with_empty_config(&blocked, || panic!("invalid cache"))
        .unwrap_or_else(|_| std::env::var_os(FONTCONFIG_FILE).unwrap());
    assert_eq!(scanned, original);
    assert_eq!(std::env::var_os(FONTCONFIG_FILE), Some(original.clone()));

    let nonutf = root.path().join(OsString::from_vec(b"cache-\xff".to_vec()));
    assert!(fontconfig_env::with_empty_config(&nonutf, || panic!("non-UTF-8 cache")).is_err());
    assert_eq!(std::env::var_os(FONTCONFIG_FILE), Some(original));
    assert_eq!(std::fs::read_dir(nonutf).unwrap().count(), 0);
}

#[cfg(not(target_os = "linux"))]
fn main() {}
