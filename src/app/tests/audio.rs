use super::*;

#[test]
fn linked_sources_stay_rows() {
    let mon = |name: &str, wtype: &str, source: &str, mute: bool, volume: u32| {
        crate::frontend::audio_panel::AudioMon {
            name: name.to_string(),
            label: format!("{wtype} thing"),
            source: source.to_string(),
            wtype: crate::contracts::media::MediaKind::from_key(wtype),
            mute,
            volume,
            shared: false,
            paused: false,
            manual_paused: false,
        }
    };
    let mut monitors = vec![
        mon("DP-1", "video", "/same.mp4", true, 30),
        mon("DP-2", "video", "/same.mp4", false, 45),
        mon("DP-3", "static", "/still.png", true, 0),
    ];
    crate::frontend::audio_panel::align_shared_audio(&mut monitors);
    assert_eq!(monitors.len(), 3);
    for monitor in &monitors[..2] {
        assert!(monitor.shared);
        assert!(!monitor.mute);
        assert_eq!(monitor.volume, 45);
    }
    assert!(!monitors[2].shared);

    let mut panel = crate::frontend::audio_panel::AudioPanel::new();
    panel.mons = monitors;
    assert_eq!(panel.audio_group_outputs("DP-2"), ["DP-1", "DP-2"]);

    let mut we = vec![
        mon("DP-1", "we", "2285115188", false, 24),
        mon("DP-2", "we", "2285115188", true, 79),
        mon("DP-3", "we", "other", false, 27),
    ];
    crate::frontend::audio_panel::align_shared_audio(&mut we);
    assert!(we[0].shared && we[1].shared);
    assert!(!we[2].shared);
}

#[test]
fn zero_volume_restores_mute() {
    let mut app = test_app();
    app.panels.effects = Some(crate::frontend::effects::Effects::new(
        Vec::new(),
        String::from("/video.mp4"),
        None,
        0,
        WallpaperKind::Video,
        false,
        45,
        String::from("video:/video.mp4"),
    ));

    let _ = update(&mut app, Message::Effects(crate::frontend::effects::EffectsMsg::SrcVolume(0)));
    assert_eq!(app.panels.effects.as_ref().unwrap().source_audio(), (true, 0));
    let _ = update(&mut app, Message::Effects(crate::frontend::effects::EffectsMsg::SrcVolume(35)));
    assert_eq!(app.panels.effects.as_ref().unwrap().source_audio(), (false, 35));

    app.panels.effects = None;
    let mut audio = crate::frontend::audio_panel::AudioPanel::new();
    audio.mons.push(crate::frontend::audio_panel::AudioMon {
        name: String::from("DP-1"),
        label: String::from("video"),
        source: String::from("/video.mp4"),
        wtype: crate::contracts::media::MediaKind::Video,
        mute: false,
        volume: 45,
        shared: false,
        paused: false,
        manual_paused: false,
    });
    app.panels.audio = Some(audio);
    let _ = drain_calls(&app);

    let _ = update(
        &mut app,
        Message::Audio(crate::frontend::audio_panel::AudioMsg::MonVolume(String::from("DP-1"), 0)),
    );
    assert!(app.panels.audio.as_ref().unwrap().mons[0].mute);
    let _ = update(
        &mut app,
        Message::Audio(crate::frontend::audio_panel::AudioMsg::MonVolumeRelease(String::from(
            "DP-1",
        ))),
    );
    let calls = drain_calls(&app);
    let (_, params) =
        calls.iter().find(|(method, _)| method == "wall.set_audio").expect("set_audio sent");
    assert_eq!(params["volume"], 0);
    assert_eq!(params["mute"], true);

    let _ = update(
        &mut app,
        Message::Audio(crate::frontend::audio_panel::AudioMsg::MonVolume(String::from("DP-1"), 35)),
    );
    assert!(!app.panels.audio.as_ref().unwrap().mons[0].mute);
}

#[test]
fn selector_audio_groups() {
    let monitor = |name: &str, kind: WallpaperKind, source: &str, mute: bool, volume: u32| {
        let (current, we_id) = match kind {
            WallpaperKind::We => (String::new(), source.to_string()),
            _ => (source.to_string(), String::new()),
        };
        crate::frontend::effects::MonitorInfo {
            name: name.to_string(),
            target: name.to_string(),
            connected: true,
            width: 1920,
            height: 1080,
            current_thumb: None,
            kind,
            mute,
            volume,
            fill: String::new(),
            locked: false,
            paused: false,
            manual_paused: false,
            current,
            we_id,
        }
    };
    let mut app = test_app();
    let mut effects = crate::frontend::effects::Effects::new(
        Vec::new(),
        String::from("/incoming.mp4"),
        None,
        0,
        WallpaperKind::Video,
        true,
        100,
        String::from("video:/incoming.mp4"),
    );
    effects.set_monitors(vec![
        monitor("DP-1", WallpaperKind::Video, "/same.mp4", false, 42),
        monitor("DP-2", WallpaperKind::Video, "/same.mp4", true, 90),
        monitor("DP-3", WallpaperKind::Video, "/other.mp4", false, 75),
    ]);
    app.panels.effects = Some(effects);
    let _ = drain_calls(&app);

    let _ = update(
        &mut app,
        Message::Audio(crate::frontend::audio_panel::AudioMsg::MonVolume(String::from("DP-2"), 28)),
    );
    let effects = app.panels.effects.as_ref().unwrap();
    assert_eq!(effects.mon_volume("DP-1"), Some(28));
    assert_eq!(effects.mon_volume("DP-2"), Some(28));
    assert_eq!(effects.mon_volume("DP-3"), Some(75));
    let _ = update(
        &mut app,
        Message::Audio(crate::frontend::audio_panel::AudioMsg::MonVolumeRelease(String::from(
            "DP-2",
        ))),
    );
    let calls = drain_calls(&app);
    let (_, params) =
        calls.iter().find(|(method, _)| method == "wall.set_audio").expect("set_audio sent");
    assert_eq!(params["outputs"], json!(["DP-1", "DP-2"]));
    assert_eq!(params["volume"], 28);

    app.panels.effects.as_mut().unwrap().set_monitors(vec![
        monitor("DP-1", WallpaperKind::We, "2057951800", false, 51),
        monitor("DP-2", WallpaperKind::We, "2057951800", true, 80),
        monitor("DP-3", WallpaperKind::We, "999", false, 75),
    ]);
    let _ = update(
        &mut app,
        Message::Audio(crate::frontend::audio_panel::AudioMsg::MonMute(String::from("DP-1"), true)),
    );
    let effects = app.panels.effects.as_ref().unwrap();
    assert!(effects.monitors()[0].mute && effects.monitors()[1].mute);
    assert!(!effects.monitors()[2].mute);
    let calls = drain_calls(&app);
    let (_, params) =
        calls.iter().find(|(method, _)| method == "wall.set_audio").expect("set_audio sent");
    assert_eq!(params["outputs"], json!(["DP-1", "DP-2"]));
    assert_eq!(params["mute"], true);
}

#[test]
fn monitor_pause_targets_only_one_linked_wallpaper() {
    use crate::frontend::audio_panel::{AudioMon, AudioMsg, AudioPanel};
    let mut app = test_app();
    let mut panel = AudioPanel::new();
    panel.mons = ["DP-1", "DP-2"]
        .into_iter()
        .map(|name| AudioMon {
            name: name.into(),
            label: "video".into(),
            source: "/same.mp4".into(),
            wtype: crate::contracts::media::MediaKind::Video,
            mute: false,
            volume: 35,
            shared: true,
            paused: false,
            manual_paused: false,
        })
        .collect();
    app.panels.audio = Some(panel);
    drain_calls(&app);
    let _ = update(&mut app, Message::Audio(AudioMsg::MonPause("DP-1".into(), true)));
    let calls = drain_calls(&app);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], ("wall.set_paused".to_string(), json!({"output":"DP-1","paused":true})));
    assert!(!app.panels.audio.as_ref().unwrap().mons[1].paused);
    app.on_result(Pending::AudioOutputs, &json!({"outputs":[
        {"name":"DP-1","type":"video","path":"/same.mp4","paused":true,"manual_paused":true,"mute":false,"volume":35},
        {"name":"DP-2","type":"video","path":"/same.mp4","paused":false,"manual_paused":false,"mute":false,"volume":35}
    ]}));
    let panel = app.panels.audio.as_ref().unwrap();
    assert!(!panel.mons[0].playing());
    assert!(panel.mons[1].playing());
    assert!(panel.mons[0].manual_paused);
}

#[test]
fn open_mixer_keeps_existing_panel() {
    let mut app = test_app();
    let _ = update(
        &mut app,
        Message::Daemon(crate::infrastructure::runtime::Wake::Command("open mixer".into())),
    );
    assert!(app.panels.audio.is_some());
    let _ = update(
        &mut app,
        Message::Daemon(crate::infrastructure::runtime::Wake::Command("open mixer".into())),
    );
    assert!(app.panels.audio.is_some());
}
