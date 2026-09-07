use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::contracts::capabilities::{GraphicsCard, GraphicsProbe};

static GRAPHICS_DEVICES: OnceLock<Vec<crate::contracts::capabilities::GraphicsDevice>> =
    OnceLock::new();

pub fn graphics_devices(
    probe: &impl GraphicsProbe,
) -> &[crate::contracts::capabilities::GraphicsDevice] {
    GRAPHICS_DEVICES.get_or_init(|| probe.devices())
}

static GRAPHICS_CARD: OnceLock<GraphicsCard> = OnceLock::new();

pub fn graphics_card(probe: &impl GraphicsProbe) -> &'static GraphicsCard {
    GRAPHICS_CARD.get_or_init(|| {
        let Some((cache_path, cache_key)) = gpu_cache_location() else {
            return probe.probe();
        };
        load_or_probe(&cache_path, &cache_key, probe)
    })
}

pub(super) fn load_or_probe(path: &Path, key: &str, probe: &impl GraphicsProbe) -> GraphicsCard {
    if let Ok(text) = std::fs::read_to_string(path)
        && let Some(cached) = decode(&text, key)
    {
        return cached;
    }

    let probed = probe.probe();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, encode(&probed, key));
    probed
}

#[derive(Serialize, Deserialize)]
struct CachedCard<'a> {
    #[serde(borrow)]
    key: Cow<'a, str>,
    #[serde(borrow)]
    name: Cow<'a, str>,
    #[serde(borrow)]
    tier: Cow<'a, str>,
}

pub(super) fn decode(text: &str, key: &str) -> Option<GraphicsCard> {
    let cached: CachedCard<'_> = serde_json::from_str(text).ok()?;
    if cached.key != key {
        return None;
    }
    Some(GraphicsCard { name: cached.name.into_owned(), tier: cached.tier.parse().ok()? })
}

pub(super) fn encode(info: &GraphicsCard, key: &str) -> String {
    serde_json::to_string(&CachedCard {
        key: Cow::Borrowed(key),
        name: Cow::Borrowed(&info.name),
        tier: Cow::Borrowed(info.tier.label()),
    })
    .unwrap_or_default()
}

#[cfg(target_os = "linux")]
#[allow(clippy::unnecessary_wraps)]
fn gpu_cache_location() -> Option<(PathBuf, String)> {
    let preference = std::env::var("WGPU_POWER_PREF").unwrap_or_else(|_| String::from("none"));
    Some((probe_cache_dir().join("gpu-tier"), format!("{}|power={preference}", driver_signature())))
}

#[cfg(not(target_os = "linux"))]
fn gpu_cache_location() -> Option<(PathBuf, String)> {
    None
}

#[cfg(target_os = "linux")]
pub(crate) fn probe_cache_dir() -> PathBuf {
    std::env::var("XDG_CACHE_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("HOME").ok().map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(std::env::temp_dir)
        .join("skwd-wall")
}

#[cfg(target_os = "linux")]
pub(crate) fn driver_signature() -> String {
    let icd = std::fs::metadata("/usr/share/vulkan/icd.d")
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|mtime| mtime.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |duration| duration.as_secs());
    let driver = std::fs::read_to_string("/proc/driver/nvidia/version").unwrap_or_default();
    format!("{icd}|{}", driver.split_whitespace().collect::<Vec<_>>().join(" "))
}
