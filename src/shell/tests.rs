#![cfg(all(test, target_os = "linux"))]

use std::io::ErrorKind;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};

use super::shell::{
    ARABIC_UI_FONT_BYTES, BENGALI_UI_FONT_BYTES, DEVANAGARI_UI_FONT_BYTES, JAPANESE_UI_FONT_BYTES,
    NERD_FONT_BYTES, NVIDIA_COMPILER_RECLAIM_ADVICE, SIMPLIFIED_UI_FONT_BYTES, UI_FONT_BYTES,
    attach_reply, cold_nvidia_compiler_mapping, compressed_thumbnail_profile,
    layer_shell_global_usable, layershell_error_message, layershell_run_error_message,
    parse_control_command, picker_power_preference, remove_ui_faces, startup_control_command,
    startup_fonts, ui_font_bytes,
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
    assert_eq!(startup_control_command(Some("mixer"), None).as_deref(), Some("open mixer\n"));
    assert_eq!(
        startup_control_command(Some("theme-audition"), None).as_deref(),
        Some("open theme-audition\n")
    );
    assert_eq!(startup_control_command(Some("wallpaper"), None), None);
    assert_eq!(startup_control_command(None, None), None);
    assert_eq!(startup_control_command(None, Some("games")).as_deref(), Some("profile games\n"));
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
fn ui_font_follows_script() {
    use crate::i18n::Script;
    assert!(std::ptr::eq(ui_font_bytes(Script::Latin), UI_FONT_BYTES));
    assert!(std::ptr::eq(ui_font_bytes(Script::Cyrillic), UI_FONT_BYTES));
    assert!(std::ptr::eq(ui_font_bytes(Script::Simplified), SIMPLIFIED_UI_FONT_BYTES));
    assert!(std::ptr::eq(ui_font_bytes(Script::Japanese), JAPANESE_UI_FONT_BYTES));
    assert!(std::ptr::eq(ui_font_bytes(Script::Arabic), ARABIC_UI_FONT_BYTES));
    assert!(std::ptr::eq(ui_font_bytes(Script::Bengali), BENGALI_UI_FONT_BYTES));
    assert!(std::ptr::eq(ui_font_bytes(Script::Devanagari), DEVANAGARI_UI_FONT_BYTES));
}

fn shaped_glyphs(script: crate::i18n::Script, text: &str) -> Vec<u16> {
    use iced_wgpu::graphics::text::cosmic_text::{
        Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Stretch, Weight, fontdb,
    };
    let mut db = fontdb::Database::new();
    db.load_font_data(ui_font_bytes(script).to_vec());
    let mut system = FontSystem::new_with_locale_and_db(String::from("en-US"), db);
    let mut buffer = Buffer::new(&mut system, Metrics::new(14.0, 20.0));
    let attrs = Attrs::new()
        .family(Family::Name(crate::frontend::ui::UI_FONT_FAMILY))
        .weight(Weight::BOLD)
        .stretch(Stretch::Condensed);
    buffer.set_text(&mut system, text, &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(&mut system, false);
    let ids: Vec<u16> = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter().map(|glyph| glyph.glyph_id))
        .collect();
    assert!(!ids.is_empty() && ids.iter().all(|&id| id != 0), "{text}");
    ids
}

#[test]
fn ui_faces_shape_complex_scripts() {
    use crate::i18n::Script;
    let isolated = shaped_glyphs(Script::Arabic, "ب")[0];
    assert!(!shaped_glyphs(Script::Arabic, "ببب").contains(&isolated));
    let persian_isolated = shaped_glyphs(Script::Arabic, "پ")[0];
    assert!(!shaped_glyphs(Script::Arabic, "پپپ").contains(&persian_isolated));
    assert_eq!(shaped_glyphs(Script::Latin, "ÇçĞğİıÖöŞşÜü").len(), 12);
    let urdu_isolated = shaped_glyphs(Script::Arabic, "ٹ")[0];
    assert!(!shaped_glyphs(Script::Arabic, "ٹٹٹ").contains(&urdu_isolated));
    assert_eq!(shaped_glyphs(Script::Devanagari, "क्ष").len(), 1);
    assert_eq!(shaped_glyphs(Script::Bengali, "ক্ষ").len(), 1);
    assert_eq!(shaped_glyphs(Script::Latin, "Français").len(), 8);
}

#[test]
fn ui_faces_shape_persian_and_turkish_language_labels() {
    use crate::i18n::Script;
    for script in [
        Script::Latin,
        Script::Cyrillic,
        Script::Simplified,
        Script::Japanese,
        Script::Arabic,
        Script::Bengali,
        Script::Devanagari,
    ] {
        shaped_glyphs(script, "فارسی");
        assert_eq!(shaped_glyphs(script, "Türkçe").len(), 6);
    }
}

#[test]
fn ui_swap_preserves_icons() {
    use iced_wgpu::graphics::text::cosmic_text::fontdb::{Database, Source};
    let mut db = Database::new();
    db.load_font_data(UI_FONT_BYTES.to_vec());
    db.load_font_data(NERD_FONT_BYTES.to_vec());
    db.load_font_file(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/RobotoCondensed-Bold.ttf"))
        .expect("load file-backed UI family face");
    remove_ui_faces(&mut db);
    let remaining: Vec<(bool, bool)> = db
        .faces()
        .map(|face| {
            (
                matches!(face.source, Source::Binary(_)),
                face.families.iter().any(|(name, _)| name == crate::frontend::ui::UI_FONT_FAMILY),
            )
        })
        .collect();
    assert!(!remaining.contains(&(true, true)));
    assert!(!remaining.contains(&(false, true)));
    assert!(remaining.contains(&(true, false)));
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
fn queued_control_before_listener() {
    use std::io::Write;

    let name = format!("skwd-wall-queue.{}.{}", unsafe { libc::getuid() }, std::process::id());
    let addr = SocketAddr::from_abstract_name(name.as_bytes()).unwrap();
    let listener = UnixListener::bind_addr(&addr).unwrap();
    for command in ["", "open mixer\n", "profile games\n", "state\n"] {
        let mut conn = UnixStream::connect_addr(&addr).unwrap();
        conn.write_all(command.as_bytes()).unwrap();
    }
    for expected in ["", "open mixer", "profile games", "state"] {
        let (conn, _) = listener.accept().unwrap();
        let wake = super::shell::read_control_command(conn);
        match wake {
            crate::infrastructure::runtime::Wake::Toggle => assert_eq!(expected, ""),
            crate::infrastructure::runtime::Wake::Command(line)
            | crate::infrastructure::runtime::Wake::Query(line, _) => assert_eq!(line, expected),
            other => panic!("unexpected wake: {other:?}"),
        }
    }
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

#[test]
fn startup_fonts_match_language() {
    if std::env::var_os("SKWD_FONT_STARTUP_TEST").is_none() {
        for locale in ["en-US", "ja-JP"] {
            let root = tempfile::tempdir().unwrap();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "shell::tests::startup_fonts_match_language", "--nocapture"])
                .env("SKWD_FONT_STARTUP_TEST", locale)
                .env("SKWD_WALL_LOCALE", locale)
                .env("SKWD_WALL_V2_CONFIG", root.path())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{locale}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return;
    }
    for language in crate::i18n::LANGUAGES {
        let config = serde_json::json!({ "general": { "language": language.tag } });
        std::fs::write(skwd_config::config_path(), config.to_string()).unwrap();
        let fonts = startup_fonts();
        assert_eq!(crate::i18n::active_tag(), language.tag);
        assert_eq!(fonts[0].as_ref(), ui_font_bytes(language.script), "{}", language.tag);
        assert_eq!(fonts[1].as_ref(), NERD_FONT_BYTES);
    }
    std::fs::write(skwd_config::config_path(), r#"{"general":{"language":"auto"}}"#).unwrap();
    startup_fonts();
    assert_eq!(crate::i18n::active_tag(), std::env::var("SKWD_FONT_STARTUP_TEST").unwrap());
}

#[test]
fn ui_faces_shape_symbols() {
    for language in crate::i18n::LANGUAGES {
        assert_eq!(shaped_glyphs(language.script, "✓ ⚠ ◆ ◇").len(), 7);
    }
}

#[test]
fn initialized_face_wins() {
    use iced_wgpu::graphics::text::{cosmic_text::fontdb, font_system};

    if std::env::var_os("SKWD_FONT_PRIORITY_TEST").is_none() {
        for cached in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let fonts = root.path().join("fonts");
            std::fs::create_dir(&fonts).unwrap();
            if cached {
                std::fs::write(
                    fonts.join("system.ttf"),
                    include_bytes!("../../assets/RobotoCondensed-Bold.ttf"),
                )
                .unwrap();
                std::fs::write(fonts.join("symbols.ttf"), NERD_FONT_BYTES).unwrap();
            }
            let config = root.path().join("fonts.conf");
            std::fs::write(
                &config,
                format!(
                    "<fontconfig><dir>{}</dir><cachedir>{}</cachedir></fontconfig>",
                    fonts.display(),
                    root.path().display()
                ),
            )
            .unwrap();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "shell::tests::initialized_face_wins", "--nocapture"])
                .env("SKWD_FONT_PRIORITY_TEST", "1")
                .env("FONTCONFIG_FILE", config)
                .env("SKWD_WALL_V2_CONFIG", root.path())
                .env("SKWD_WALL_V2_CACHE", root.path())
                .env("SKWD_FONT_CACHED_TEST", if cached { "1" } else { "0" })
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "cached={cached}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return;
    }
    let mut system = font_system().write().unwrap();
    match super::fonts::database() {
        Ok(catalog) => super::shell::merge_catalog(&mut system, catalog),
        Err(_) => remove_ui_faces(system.raw().db_mut()),
    }
    for font in startup_fonts() {
        system.load_font(font);
    }
    let db = system.raw().db();
    assert_eq!(
        db.faces().filter(|face| matches!(face.source, fontdb::Source::File(_))).count(),
        usize::from(std::env::var("SKWD_FONT_CACHED_TEST").unwrap() == "1")
    );
    assert!(db.faces().any(|face| {
        matches!(face.source, fontdb::Source::Binary(_))
            && face.families.iter().any(|(name, _)| name == "Iced-Icons")
    }));
    let id = db
        .query(&fontdb::Query {
            families: &[fontdb::Family::Name(crate::frontend::ui::UI_FONT_FAMILY)],
            weight: fontdb::Weight::BOLD,
            stretch: fontdb::Stretch::Condensed,
            ..fontdb::Query::default()
        })
        .unwrap();
    assert!(matches!(db.face(id).unwrap().source, fontdb::Source::Binary(_)));
    assert_eq!(
        db.faces()
            .filter(|face| face
                .families
                .iter()
                .any(|(name, _)| name == crate::frontend::ui::UI_FONT_FAMILY))
            .count(),
        1
    );
}
