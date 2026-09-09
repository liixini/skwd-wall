#![cfg(all(test, target_os = "linux"))]

use std::io::ErrorKind;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};

use super::shell::{
    NERD_FONT_BYTES, NVIDIA_COMPILER_RECLAIM_ADVICE, UI_FONT_BYTES, attach_reply,
    cold_nvidia_compiler_mapping, compressed_thumbnail_profile, layer_shell_global_usable,
    layershell_error_message, layershell_run_error_message, parse_control_command,
    picker_power_preference, startup_control_command,
};

#[test]
fn parse_control_lines() {
    assert!(matches!(parse_control_command(""), crate::infrastructure::runtime::Wake::Toggle));
    assert!(matches!(parse_control_command("\n"), crate::infrastructure::runtime::Wake::Toggle));
    match parse_control_command("mode hex\nignored second line") {
        crate::infrastructure::runtime::Wake::Command(cmd) => assert_eq!(cmd, "mode hex"),
        other => panic!("expected command, got {other:?}"),
    }
    match parse_control_command("  open theme  \n") {
        crate::infrastructure::runtime::Wake::Command(cmd) => assert_eq!(cmd, "open theme"),
        other => panic!("expected command, got {other:?}"),
    }
}

#[test]
fn panel_startup() {
    assert_eq!(startup_control_command(Some("mixer")), Some("open mixer\n"));
    assert_eq!(startup_control_command(Some("theme-audition")), Some("open theme-audition\n"));
    assert_eq!(startup_control_command(Some("wallpaper")), None);
    assert_eq!(startup_control_command(None), None);
}

#[test]
fn socket_name_namespaced() {
    let base = super::shell::control_socket_name();
    assert!(base.starts_with("skwd-wall-v2."));
    unsafe { std::env::set_var("SKWD_WALL_V2_INSTANCE", "e2e") };
    let tagged = super::shell::control_socket_name();
    unsafe { std::env::remove_var("SKWD_WALL_V2_INSTANCE") };
    assert_eq!(tagged, format!("{base}.e2e"));
    assert_eq!(super::shell::control_socket_name(), base);
}

#[test]
fn query_reply_channel() {
    use std::io::Read;

    let (local, mut peer) = UnixStream::pair().unwrap();
    match attach_reply(parse_control_command("state\n"), local) {
        crate::infrastructure::runtime::Wake::Query(cmd, reply) => {
            assert_eq!(cmd, "state");
            reply.send("{\"ok\":true}");
            let mut got = String::new();
            peer.set_read_timeout(Some(std::time::Duration::from_secs(2))).unwrap();
            peer.read_to_string(&mut got).unwrap();
            assert_eq!(got, "{\"ok\":true}\n");
        }
        other => panic!("expected query, got {other:?}"),
    }

    let (local, _peer) = UnixStream::pair().unwrap();
    assert!(matches!(
        attach_reply(parse_control_command("mode hex\n"), local),
        crate::infrastructure::runtime::Wake::Command(_)
    ));
    let (local, _peer) = UnixStream::pair().unwrap();
    assert!(matches!(
        attach_reply(parse_control_command("\n"), local),
        crate::infrastructure::runtime::Wake::Toggle
    ));
}

#[test]
fn bundled_fonts() {
    assert!(!UI_FONT_BYTES.is_empty());
    assert!(NERD_FONT_BYTES.len() > 100_000);
}

#[test]
fn layershell_failure_msg() {
    let msg = layershell_error_message("GNOME", "no such protocol");
    assert!(msg.contains("wlr-layer-shell"));
    assert!(msg.contains("GNOME"));
    assert!(msg.contains("SKWD_WALL_FORCE_WINIT"));
    assert!(msg.contains("no such protocol"));
}

#[test]
fn layershell_failure_unknown() {
    let unknown = layershell_error_message("weird-wm", "boom");
    assert!(unknown.contains("weird-wm"));
    assert!(unknown.contains("wlr-layer-shell"));
    let empty = layershell_error_message("", "boom");
    assert!(empty.contains("wlr-layer-shell"));
    assert!(!empty.contains("  "));
}

#[test]
fn typed_failure_messages() {
    let graphics = iced_layershell::Error::GraphicsCreationFailed(
        iced_wgpu::graphics::Error::BackendError(String::from("adapter failed")),
    );
    let graphics = layershell_run_error_message("niri", &graphics);
    assert!(graphics.contains("GPU rendering"));
    assert!(graphics.contains("adapter failed"));
    assert!(graphics.contains("SKWD_WALL_THUMBNAILS=rgba-poc"));
    assert!(!graphics.contains("does not implement"));

    let executor = iced_layershell::Error::ExecutorCreationFailed(std::io::Error::other("spawn"));
    let executor = layershell_run_error_message("niri", &executor);
    assert!(executor.contains("task executor"));
    assert!(executor.contains("spawn"));
    assert!(!executor.contains("does not implement"));

    let window = iced_layershell::Error::WindowCreationFailed(Box::new(std::io::Error::other(
        "surface failed",
    )));
    let window = layershell_run_error_message("niri", &window);
    assert!(window.contains("surface creation failed"));
    assert!(window.contains("surface failed"));
    assert!(!window.contains("does not implement"));
}

#[test]
fn layer_shell_version_floor() {
    assert!(layer_shell_global_usable("zwlr_layer_shell_v1", 3));
    assert!(layer_shell_global_usable("zwlr_layer_shell_v1", 4));
    assert!(!layer_shell_global_usable("zwlr_layer_shell_v1", 2));
    assert!(!layer_shell_global_usable("wl_shell", 4));
}

#[test]
fn abstract_socket_single() {
    let name = format!("skwd-wall-test.{}.{}", unsafe { libc::getuid() }, std::process::id());
    let addr = SocketAddr::from_abstract_name(name.as_bytes()).unwrap();
    let listener = UnixListener::bind_addr(&addr).expect("first bind succeeds");
    let second = UnixListener::bind_addr(&addr);
    assert!(matches!(&second, Err(err) if err.kind() == ErrorKind::AddrInUse));
    UnixStream::connect_addr(&addr).expect("relaunch can connect to primary");
    let _ = listener.accept();
}

#[test]
fn gpu_preference_battery() {
    let auto = serde_json::json!({});
    assert_eq!(picker_power_preference(&auto, false), "none");
    assert_eq!(picker_power_preference(&auto, true), "low");
    let high = serde_json::json!({"performance": {"gpuPreference": "high"}});
    assert_eq!(picker_power_preference(&high, true), "high");
    let disabled = serde_json::json!({"performance": {"batterySaver": false}});
    assert_eq!(picker_power_preference(&disabled, true), "none");
}

#[test]
fn thumbnail_profile_bc_default() {
    assert!(compressed_thumbnail_profile(None));
    assert!(compressed_thumbnail_profile(Some("bc")));
    assert!(compressed_thumbnail_profile(Some("unexpected")));
    assert!(!compressed_thumbnail_profile(Some("rgba-poc")));
    assert!(!compressed_thumbnail_profile(Some("RGBA-POC")));
}

#[test]
fn cold_reclaim_ro_pages() {
    assert_eq!(NVIDIA_COMPILER_RECLAIM_ADVICE, libc::MADV_COLD);
    assert_ne!(NVIDIA_COMPILER_RECLAIM_ADVICE, libc::MADV_DONTNEED);
    let line = "7f100000-7f200000 r-xp 00000000 00:01 42 /usr/lib/libnvidia-gpucomp.so.610";
    let (_, len) = cold_nvidia_compiler_mapping(line).expect("mapping");
    assert_eq!(len, 0x10_0000);
    assert!(
        cold_nvidia_compiler_mapping(
            "7f100000-7f200000 rw-p 0 00:01 42 /usr/lib/libnvidia-gpucomp.so.610"
        )
        .is_none()
    );
    assert!(
        cold_nvidia_compiler_mapping("7f100000-7f200000 r-xp 0 00:01 42 /usr/lib/libavcodec.so")
            .is_none()
    );
}
