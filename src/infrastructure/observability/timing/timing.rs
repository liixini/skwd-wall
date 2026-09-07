use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) static TIMING_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
static CHECKPOINTS_SEEN: Mutex<Option<std::collections::HashSet<String>>> = Mutex::new(None);

pub fn checkpoint_is_new(label: &str) -> bool {
    CHECKPOINTS_SEEN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_or_insert_with(std::collections::HashSet::new)
        .insert(label.to_string())
}

#[cfg_attr(test, allow(dead_code))]
pub fn init(cache_dir: &str) {
    let dir = PathBuf::from(cache_dir).join("wallpaper");
    let _ = std::fs::create_dir_all(&dir);
    *TIMING_PATH.lock().unwrap_or_else(std::sync::PoisonError::into_inner) =
        Some(dir.join("selector-timing.log"));
}

pub fn line(message: &str) {
    let Some(path) = TIMING_PATH.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    else {
        return;
    };
    append_line(&path, message);
}

pub(super) fn append_line(path: &std::path::Path, message: &str) {
    let epoch_ms = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |dur| dur.as_millis());
    let (rss_mb, pss_mb, vss_mb) = self_mem_mb();
    let full = format!(
        "{epoch_ms} wallpaper-selector timing: {message} (rss: {rss_mb:.1} MB, pss: {pss_mb:.1} MB, vss: {vss_mb:.1} MB) [rust]\n"
    );
    if let Ok(mut file) = skwd_log::RotatingWriter::new(path) {
        let _ = file.write_all(full.as_bytes());
    }
}

#[cfg(target_os = "linux")]
pub(super) fn self_mem_mb() -> (f64, f64, f64) {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let mut rss = 0.0;
    let mut vss = 0.0;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            rss = parse_kb(rest);
        } else if let Some(rest) = line.strip_prefix("VmSize:") {
            vss = parse_kb(rest);
        }
    }
    let pss = std::fs::read_to_string("/proc/self/smaps_rollup")
        .unwrap_or_default()
        .lines()
        .find_map(|line| line.strip_prefix("Pss:").map(parse_kb))
        .unwrap_or(0.0);
    (rss / 1024.0, pss / 1024.0, vss / 1024.0)
}

#[cfg(not(target_os = "linux"))]
pub(super) fn self_mem_mb() -> (f64, f64, f64) {
    (0.0, 0.0, 0.0)
}

#[cfg(target_os = "linux")]
pub(super) fn parse_kb(text: &str) -> f64 {
    text.trim().trim_end_matches("kB").trim().parse().unwrap_or(0.0)
}
