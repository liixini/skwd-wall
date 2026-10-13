use std::path::Path;

#[cfg(target_os = "linux")]
use std::{
    io::{BufRead, BufReader, Cursor, Read, Write},
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[cfg(target_os = "linux")]
use wl_clipboard_rs::{
    copy::{MimeSource, MimeType, Options, Source},
    paste::{ClipboardType, Seat, get_mime_types},
};

#[cfg(target_os = "linux")]
const SERVE_ARG: &str = "--skwd-internal-clipboard-serve";
#[cfg(target_os = "linux")]
const IMAGE_MIME: &str = "image/png";
#[cfg(target_os = "linux")]
const TEXT_MIME: &str = "text/plain;charset=utf-8";
#[cfg(target_os = "linux")]
const MARKER_MIME: &str = "application/x-skwd-wall-clipboard";
#[cfg(target_os = "linux")]
const READY: &str = "READY";
#[cfg(target_os = "linux")]
const ERROR: &str = "ERROR:";
#[cfg(target_os = "linux")]
const START_TIMEOUT: Duration = Duration::from_secs(5);
#[cfg(target_os = "linux")]
const READY_POLL: Duration = Duration::from_millis(10);

pub fn copy_image(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let source = std::fs::read(path).map_err(|err| format!("cannot read image: {err}"))?;
        copy(&source, IMAGE_MIME)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = path;
        Err("Wayland clipboard is unavailable on this platform".into())
    }
}

pub fn copy_path(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        copy(&path_bytes(path)?, TEXT_MIME)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = path;
        Err("Wayland clipboard is unavailable on this platform".into())
    }
}

#[cfg(target_os = "linux")]
fn image_sources(source: Vec<u8>) -> Result<Vec<MimeSource>, String> {
    let format = image::guess_format(&source).map_err(|err| format!("invalid image: {err}"))?;
    let decoded = image::load_from_memory_with_format(&source, format)
        .map_err(|err| format!("cannot decode image: {err}"))?;
    let mut sources = vec![MimeSource {
        source: Source::Bytes(source.into()),
        mime_type: MimeType::Specific(format.to_mime_type().into()),
    }];
    if format != image::ImageFormat::Png {
        let mut png = Cursor::new(Vec::new());
        decoded
            .write_to(&mut png, image::ImageFormat::Png)
            .map_err(|err| format!("cannot encode PNG: {err}"))?;
        sources.push(MimeSource {
            source: Source::Bytes(png.into_inner().into()),
            mime_type: MimeType::Specific(IMAGE_MIME.into()),
        });
    }
    Ok(sources)
}

#[cfg(target_os = "linux")]
fn path_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let absolute = path.canonicalize().map_err(|err| format!("cannot resolve path: {err}"))?;
    let text = absolute.to_str().ok_or("cannot copy a path that is not UTF-8")?;
    Ok(text.as_bytes().to_vec())
}

pub fn try_serve(arguments: &[String]) -> Option<Result<(), String>> {
    #[cfg(target_os = "linux")]
    {
        let [mode, mime] = arguments else {
            return None;
        };
        if mode != SERVE_ARG {
            return None;
        }
        let result = serve(mime);
        if let Err(err) = &result {
            println!("{ERROR}{}", err.replace('\n', " "));
            let _ = std::io::stdout().flush();
        }
        Some(result)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = arguments;
        None
    }
}

#[cfg(target_os = "linux")]
fn copy(bytes: &[u8], mime: &str) -> Result<(), String> {
    let executable =
        std::env::current_exe().map_err(|err| format!("cannot locate picker: {err}"))?;
    let mut child = Command::new(executable)
        .arg(SERVE_ARG)
        .arg(mime)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| format!("cannot start clipboard service: {err}"))?;

    let write = child.stdin.take().ok_or("clipboard service has no input")?.write_all(bytes);
    if let Err(err) = write {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("cannot send clipboard data: {err}"));
    }

    let stdout = child.stdout.take().ok_or("clipboard service has no output")?;
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut line = String::new();
        let result = BufReader::new(stdout).read_line(&mut line).map(|_| line);
        let _ = sender.send(result);
    });

    let result = match receiver.recv_timeout(START_TIMEOUT) {
        Ok(Ok(line)) if line.trim_end() == READY => Ok(()),
        Ok(Ok(line)) if line.starts_with(ERROR) => Err(line[ERROR.len()..].trim().to_owned()),
        Ok(Ok(_)) => Err("clipboard service exited before becoming ready".into()),
        Ok(Err(err)) => Err(format!("cannot read clipboard service status: {err}")),
        Err(_) => Err("clipboard service startup timed out".into()),
    };
    if result.is_ok() {
        std::thread::spawn(move || {
            let _ = child.wait();
        });
    } else {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

#[cfg(target_os = "linux")]
fn serve(mime: &str) -> Result<(), String> {
    if mime != IMAGE_MIME && mime != TEXT_MIME {
        return Err("invalid clipboard MIME type".into());
    }
    let mut bytes = Vec::new();
    std::io::stdin()
        .read_to_end(&mut bytes)
        .map_err(|err| format!("cannot read clipboard data: {err}"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("cannot create clipboard marker: {err}"))?
        .as_nanos();
    let marker = format!("{MARKER_MIME}-{}-{nonce}", std::process::id());
    let mut options = Options::new();
    options.foreground(true);
    let mut sources = if mime == IMAGE_MIME {
        image_sources(bytes)?
    } else {
        vec![MimeSource {
            source: Source::Bytes(bytes.into()),
            mime_type: MimeType::Specific(mime.into()),
        }]
    };
    sources.push(MimeSource {
        source: Source::Bytes(Box::new([])),
        mime_type: MimeType::Specific(marker.clone()),
    });
    let prepared = options
        .prepare_copy_multi(sources)
        .map_err(|err| format!("cannot select clipboard: {err}"))?;
    let serving = std::thread::spawn(move || prepared.serve());
    let started = Instant::now();
    loop {
        if get_mime_types(ClipboardType::Regular, Seat::Unspecified)
            .is_ok_and(|types| types.contains(&marker))
        {
            println!("{READY}");
            std::io::stdout().flush().map_err(|err| format!("cannot confirm clipboard: {err}"))?;
            return serving
                .join()
                .map_err(|_| "clipboard service panicked".to_owned())?
                .map_err(|err| format!("clipboard service failed: {err}"));
        }
        if serving.is_finished() || started.elapsed() >= START_TIMEOUT {
            return Err("clipboard selection was not confirmed".into());
        }
        std::thread::sleep(READY_POLL);
    }
}

#[path = "clipboard/tests.rs"]
#[cfg(test)]
mod tests;
