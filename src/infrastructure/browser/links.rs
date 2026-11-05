use crate::contracts::browser::WorkshopLink;

const OPEN_COMMAND: &str = "xdg-open";
const ITEM_URL: &str = "https://steamcommunity.com/sharedfiles/filedetails/?id=";
const CREATOR_URL: &str = "https://steamcommunity.com/profiles/";
const VIDEO_URL: &str = "https://www.youtube.com/watch?v=";
const VIDEO_ID_LENGTH: usize = 11;
const INVALID_LINK: &str = "invalid Workshop link";

pub fn open_workshop_link(link: &WorkshopLink) -> Result<(), String> {
    let url = workshop_url(link).ok_or_else(|| INVALID_LINK.to_owned())?;
    let status = std::process::Command::new(OPEN_COMMAND)
        .arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() { Ok(()) } else { Err(status.to_string()) }
}

fn workshop_url(link: &WorkshopLink) -> Option<String> {
    let (prefix, id, video) = match link {
        WorkshopLink::Item(id) => (ITEM_URL, id, false),
        WorkshopLink::Creator(id) => (CREATOR_URL, id, false),
        WorkshopLink::Video(id) => (VIDEO_URL, id, true),
    };
    let valid = if video {
        id.len() == VIDEO_ID_LENGTH
            && id.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    } else {
        id.parse::<u64>().is_ok_and(|id| id > 0)
    };
    valid.then(|| format!("{prefix}{id}"))
}

#[cfg(test)]
mod tests;
