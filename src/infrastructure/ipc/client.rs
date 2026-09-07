use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_channel::mpsc::UnboundedSender;
use log::{debug, info, warn};
use serde_json::Value;

use crate::infrastructure::executable;
use crate::infrastructure::runtime::Wake;

const BACKOFF_BASE: Duration = Duration::from_millis(250);
const BACKOFF_MAX: Duration = Duration::from_secs(1);
const BACKOFF_RESET_AFTER: Duration = Duration::from_secs(5);
const LINE_BUF_MAX: usize = 64 * 1024;
const SOCKET_WAIT: Duration = Duration::from_secs(15);
const WRITE_TIMEOUT: Duration = Duration::from_secs(2);
const SPAWN_COOLDOWN: Duration = Duration::from_secs(30);
#[derive(Debug, Clone)]
pub enum IpcMsg {
    Connected { list_id: u64 },
    Disconnected,
    Response { id: u64, result: Option<Value>, error: Option<wall_proto::ErrorInfo> },
    Event { name: String, data: Value },
}

#[derive(Clone)]
pub struct IpcHandle {
    stream: Arc<Mutex<Option<UnixStream>>>,
    next_id: Arc<AtomicU64>,
}

fn resolve_socket_path() -> PathBuf {
    wall_proto::resolve_socket()
}

impl IpcHandle {
    #[cfg(test)]
    pub(super) fn disconnected() -> Self {
        Self { stream: Arc::new(Mutex::new(None)), next_id: Arc::new(AtomicU64::new(1)) }
    }

    #[cfg_attr(test, allow(dead_code))]
    pub fn start(tx: UnboundedSender<Wake>) -> Self {
        let handle =
            Self { stream: Arc::new(Mutex::new(None)), next_id: Arc::new(AtomicU64::new(1)) };
        let reader_handle = handle.clone();
        std::thread::Builder::new()
            .name("ipc-reader".into())
            .spawn(move || reader_loop(&reader_handle, &tx, None))
            .expect("spawn ipc reader");
        handle
    }

    pub fn call(&self, method: &str, params: Value) -> u64 {
        let Ok(mut guard) = self.stream.lock() else {
            return 0;
        };
        let Some(stream) = guard.as_mut() else {
            debug!("ipc call {method} dropped: not connected");
            return 0;
        };
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let req = wall_proto::Request { method: method.to_string(), params, id };
        let mut buf = serde_json::to_vec(&req).unwrap_or_default();
        buf.push(b'\n');
        if let Err(err) = stream.write_all(&buf).and_then(|()| stream.flush()) {
            warn!("ipc write failed: {err}");
            let _ = stream.shutdown(std::net::Shutdown::Both);
            *guard = None;
            return 0;
        }
        id
    }
}

fn spawn_walld() {
    let Some(bin) = executable::discover(&["skwd-walld"]) else {
        warn!("executable skwd-walld not found beside Wall or on PATH; start Deck manually");
        return;
    };
    let mut cmd = std::process::Command::new(&bin);
    cmd.stdin(std::process::Stdio::null());
    if crate::infrastructure::runtime::debug() {
        cmd.arg("--debug");
    }
    cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    match cmd.spawn() {
        Ok(_) => info!("spawned resident {}", bin.display()),
        Err(err) => warn!("failed to spawn skwd-walld: {err}"),
    }
}

fn reader_loop(handle: &IpcHandle, tx: &UnboundedSender<Wake>, socket: Option<&Path>) {
    let mut backoff = BACKOFF_BASE;
    let mut last_spawn: Option<std::time::Instant> = None;
    loop {
        let path = socket.map_or_else(resolve_socket_path, Path::to_path_buf);
        match UnixStream::connect(&path) {
            Ok(stream) => {
                info!("ipc connected to {}", path.display());
                match serve_connection(handle, tx, stream, &mut backoff) {
                    Served::Ended => {}
                    Served::CloneFailed => {
                        std::thread::sleep(backoff);
                        continue;
                    }
                    Served::WakeGone => return,
                }
            }
            Err(err) => {
                debug!("ipc connect {} failed: {err}", path.display());
                if try_spawn_walld(
                    &path,
                    socket.is_some() || std::env::var_os("SKWD_WALL_V2_SOCK").is_some(),
                    &mut last_spawn,
                ) {
                    continue;
                }
            }
        }
        std::thread::sleep(backoff);
        backoff = (backoff * 2).min(BACKOFF_MAX);
    }
}

enum Served {
    Ended,
    CloneFailed,
    WakeGone,
}

fn serve_connection(
    handle: &IpcHandle,
    tx: &UnboundedSender<Wake>,
    stream: UnixStream,
    backoff: &mut Duration,
) -> Served {
    let read_stream = match stream.try_clone() {
        Ok(conn) => conn,
        Err(err) => {
            warn!("ipc clone failed: {err}");
            return Served::CloneFailed;
        }
    };
    let connected_at = std::time::Instant::now();
    if let Err(err) = stream.set_write_timeout(Some(WRITE_TIMEOUT)) {
        warn!("ipc write timeout unavailable: {err}");
    }
    if let Ok(mut guard) = handle.stream.lock() {
        *guard = Some(stream);
    }
    handle.call("subscribe", serde_json::json!({"events": ["skwd."]}));
    let list_id = handle.call("wall.list", serde_json::json!({"favourites": false}));
    let _ = tx.unbounded_send(Wake::Ipc(IpcMsg::Connected { list_id }));

    read_session(read_stream, tx);

    info!("ipc disconnected");
    if connected_at.elapsed() > BACKOFF_RESET_AFTER {
        *backoff = BACKOFF_BASE;
    }
    if let Ok(mut guard) = handle.stream.lock() {
        *guard = None;
    }
    if tx.unbounded_send(Wake::Ipc(IpcMsg::Disconnected)).is_ok() {
        Served::Ended
    } else {
        Served::WakeGone
    }
}

fn read_session(read_stream: UnixStream, tx: &UnboundedSender<Wake>) {
    let mut reader = BufReader::new(read_stream);
    let mut line = String::new();
    loop {
        line.clear();
        if line.capacity() > LINE_BUF_MAX {
            line.shrink_to(8192);
        }
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        dispatch_line(trimmed, tx);
    }
}

fn dispatch_line(line: &str, tx: &UnboundedSender<Wake>) {
    match serde_json::from_str::<wall_proto::ServerMessage>(line) {
        Ok(wall_proto::ServerMessage::Event(ev)) => {
            let _ = tx.unbounded_send(Wake::Ipc(IpcMsg::Event { name: ev.event, data: ev.data }));
        }
        Ok(wall_proto::ServerMessage::Response(res)) => {
            let _ = tx.unbounded_send(Wake::Ipc(IpcMsg::Response {
                id: res.id,
                result: res.result,
                error: res.error,
            }));
        }
        Err(err) => warn!("ipc parse error: {err}; line = {line}"),
    }
}

fn try_spawn_walld(
    path: &std::path::Path,
    env_override: bool,
    last_spawn: &mut Option<std::time::Instant>,
) -> bool {
    if !may_spawn_walld(env_override, last_spawn.map(|inst| inst.elapsed())) {
        return false;
    }
    *last_spawn = Some(std::time::Instant::now());
    spawn_walld();
    info!("waiting for skwd-walld to bind {}", path.display());
    wait_for_socket(path, SOCKET_WAIT)
}

fn may_spawn_walld(env_override: bool, since_last_spawn: Option<Duration>) -> bool {
    !env_override && since_last_spawn.is_none_or(|elapsed| elapsed >= SPAWN_COOLDOWN)
}

fn wait_for_socket(path: &std::path::Path, timeout: Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if UnixStream::connect(path).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

mod tests;
