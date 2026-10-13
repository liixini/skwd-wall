use std::fmt::Write as _;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const DBUS_SEND: &str = "dbus-send";
const SHOW_ITEMS: [&str; 5] = [
    "--session",
    "--print-reply",
    "--dest=org.freedesktop.FileManager1",
    "/org/freedesktop/FileManager1",
    "org.freedesktop.FileManager1.ShowItems",
];
const URI_LIST: &str = "array:string:";
const STARTUP_ID: &str = "string:";
const XDG_OPEN: &str = "xdg-open";
const FILE_SCHEME: &str = "file://";
const KEEP: &[u8] = b"-._~/";
const OPEN_GRACE: Duration = Duration::from_secs(2);
const OPEN_POLL: Duration = Duration::from_millis(50);

pub fn show(path: &Path) -> Result<Option<Child>, String> {
    let shown = Command::new(DBUS_SEND)
        .args(SHOW_ITEMS)
        .arg(format!("{URI_LIST}{}", file_uri(path)))
        .arg(STARTUP_ID)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if shown.is_ok_and(|status| status.success()) {
        return Ok(None);
    }
    let folder = if path.is_dir() { path } else { path.parent().unwrap_or(path) };
    let mut opener = Command::new(XDG_OPEN)
        .arg(folder)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| format!("{XDG_OPEN}: {err}"))?;
    let started = Instant::now();
    while started.elapsed() < OPEN_GRACE {
        match opener.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(None),
            Ok(Some(status)) => return Err(format!("{XDG_OPEN}: {status}")),
            Ok(None) => std::thread::sleep(OPEN_POLL),
            Err(err) => return Err(format!("{XDG_OPEN}: {err}")),
        }
    }
    Ok(Some(opener))
}

fn file_uri(path: &Path) -> String {
    let mut uri = String::from(FILE_SCHEME);
    for byte in path.as_os_str().as_encoded_bytes() {
        if byte.is_ascii_alphanumeric() || KEEP.contains(byte) {
            uri.push(char::from(*byte));
        } else {
            let _ = write!(uri, "%{byte:02X}");
        }
    }
    uri
}

#[cfg(test)]
mod tests;
