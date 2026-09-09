use std::sync::OnceLock;

pub fn is_niri() -> bool {
    static IS_NIRI: OnceLock<bool> = OnceLock::new();
    *IS_NIRI.get_or_init(|| {
        std::env::var("XDG_CURRENT_DESKTOP").is_ok_and(|desktop| desktop_is_niri(&desktop))
    })
}

pub fn startup_panel() -> Option<String> {
    if std::env::args().skip(1).any(|argument| argument == "--mixer") {
        Some("mixer".to_string())
    } else {
        std::env::var("SKWD_WALL_START").ok()
    }
}

fn desktop_is_niri(desktop: &str) -> bool {
    desktop.to_lowercase().contains("niri")
}

#[cfg(test)]
mod tests;
