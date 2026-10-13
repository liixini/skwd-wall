use std::sync::OnceLock;

const PROFILE_FLAG: &str = "--profile";
pub const PROFILE_COMMAND: &str = "profile";

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

pub fn startup_profile() -> Option<String> {
    profile_argument(std::env::args().skip(1))
}

fn profile_argument(mut arguments: impl Iterator<Item = String>) -> Option<String> {
    while let Some(argument) = arguments.next() {
        if argument == PROFILE_FLAG {
            return arguments.next();
        }
        if let Some(id) =
            argument.strip_prefix(PROFILE_FLAG).and_then(|rest| rest.strip_prefix('='))
        {
            return Some(id.to_owned());
        }
    }
    None
}

fn desktop_is_niri(desktop: &str) -> bool {
    desktop.to_lowercase().contains("niri")
}

#[cfg(test)]
mod tests;
