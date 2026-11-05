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
            background: crate::frontend::settings::background::BackgroundControl::default(),
            background_reveal: 0.0,
            background_picker: 0.0,
            locked: false,
            theme_source: false,
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

#[test]
fn settings_audio_groups_current_sources_and_commits_without_another_panel() {
    use crate::contracts::daemon::{OutputStatus, OutputsResult};
    use crate::contracts::media::MediaKind;
    use crate::frontend::audio_panel::AudioMsg;
    for kind in [MediaKind::Video, MediaKind::WallpaperEngine] {
        let mut app = test_app();
        app.panels.settings.open = true;
        app.panels.settings.tab = "wallpapers".into();
        let status = |name: &str, source: &str, connected| OutputStatus {
            name: name.into(),
            target: format!("@monitor:{name}"),
            kind: kind.clone(),
            connected,
            current: source.into(),
            path: source.into(),
            we_id: source.into(),
            volume: 80,
            mute: false,
            ..Default::default()
        };
        let outputs = vec![
            status("DP-1", "shared", true),
            status("DP-2", "shared", true),
            status("DP-3", "other", true),
            status("DP-4", "shared", false),
        ];
        app.on_outputs(OutputsResult { outputs: outputs.clone() });
        assert!(app.panels.effects.is_none() && app.panels.audio.is_none());
        let _ = drain_calls(&app);
        let _ = update(&mut app, Message::Audio(AudioMsg::MonMute("@monitor:DP-1".into(), true)));
        let calls = drain_calls(&app);
        assert!(calls.iter().any(|(method, params)| method == "wall.set_audio"
            && params["outputs"] == json!(["@monitor:DP-1", "@monitor:DP-2"])
            && params["mute"] == true));
        assert!(app.daemon.output_statuses[0].mute && app.daemon.output_statuses[1].mute);
        assert!(!app.daemon.output_statuses[2].mute && !app.daemon.output_statuses[3].mute);
        for volume in [0, 37] {
            let _ = update(
                &mut app,
                Message::Audio(AudioMsg::MonVolume("@monitor:DP-1".into(), volume)),
            );
            app.on_outputs(OutputsResult { outputs: outputs.clone() });
            assert_eq!(app.daemon.output_statuses[0].volume, volume);
            assert_eq!(app.daemon.output_statuses[1].volume, volume);
            assert_eq!(app.daemon.output_statuses[2].volume, 80);
            assert_eq!(app.daemon.output_statuses[3].volume, 80);
            let _ = drain_calls(&app);
            let _ = update(
                &mut app,
                Message::Audio(AudioMsg::MonVolumeRelease("@monitor:DP-1".into())),
            );
            let calls = drain_calls(&app);
            assert!(calls.iter().any(|(method, params)| method == "wall.set_audio"
                && params["volume"] == volume
                && params["mute"] == (volume == 0)
                && params["outputs"] == json!(["@monitor:DP-1", "@monitor:DP-2"])));
            assert!(app.panels.audio_volumes.is_empty());
        }
    }
}

#[test]
fn settings_playback_refreshes_effective_and_manual_pause() {
    use crate::frontend::audio_panel::AudioMsg;
    let mut app = test_app();
    app.panels.settings.open = true;
    let _ = drain_calls(&app);
    let _ = update(&mut app, Message::Audio(AudioMsg::MonPause("@monitor:Panel".into(), true)));
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| method == "wall.set_paused"
        && params["output"] == "@monitor:Panel"
        && params["paused"] == true));
    for manual in [true, false] {
        app.daemon.pending.insert(991, Pending::AudioOutputs);
        respond(
            &mut app,
            991,
            json!({"outputs":[{"name":"DP-1","target":"@monitor:Panel","type":"video","paused":true,"manual_paused":manual}]}),
        );
        assert!(app.daemon.output_statuses[0].paused);
        assert_eq!(app.daemon.output_statuses[0].manual_paused, manual);
    }
    let _ = drain_calls(&app);
    app.on_event(wall_proto::ev::PLAYBACK, &json!({}));
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.outputs"));
}

#[test]
fn stable_audio_targets_keep_mixer_and_settings_in_sync_while_dragging() {
    use crate::contracts::daemon::OutputStatus;
    use crate::contracts::media::MediaKind;
    use crate::frontend::audio_panel::{AudioMsg, AudioPanel};
    let mut app = test_app();
    app.panels.audio = Some(AudioPanel::default());
    let outputs = vec![OutputStatus {
        name: "DP-1".into(),
        target: "@monitor:Panel".into(),
        kind: MediaKind::Video,
        current: "/movie.mp4".into(),
        path: "/movie.mp4".into(),
        volume: 70,
        ..Default::default()
    }];
    app.on_outputs(crate::contracts::daemon::OutputsResult { outputs });
    app.daemon.pending.insert(992, Pending::AudioOutputs);
    respond(
        &mut app,
        992,
        json!({"outputs":[{"name":"DP-1","target":"@monitor:Panel","type":"video","path":"/movie.mp4","volume":70}]}),
    );
    let _ = update(&mut app, Message::Audio(AudioMsg::MonVolume("DP-1".into(), 19)));
    assert_eq!(app.panels.audio.as_ref().unwrap().mons[0].volume, 19);
    assert_eq!(app.daemon.output_statuses[0].volume, 19);
    let _ = update(&mut app, Message::Audio(AudioMsg::MonMute("DP-1".into(), true)));
    assert!(app.panels.audio.as_ref().unwrap().mons[0].mute);
    assert!(app.daemon.output_statuses[0].mute);
}

#[test]
fn settings_normalizes_linked_audio_and_cancels_abandoned_volume_edits() {
    use crate::contracts::daemon::{OutputStatus, OutputsResult};
    use crate::contracts::media::MediaKind;
    use crate::frontend::audio_panel::AudioMsg;
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.tab = "wallpapers".into();
    let source = |name: &str, mute, volume, connected| OutputStatus {
        name: name.into(),
        target: format!("@monitor:{name}"),
        kind: MediaKind::Video,
        current: "shared.mp4".into(),
        mute,
        volume,
        connected,
        ..Default::default()
    };
    let outputs = vec![
        source("DP-1", true, 20, true),
        source("DP-2", false, 80, true),
        source("DP-3", true, 5, false),
    ];
    app.on_outputs(OutputsResult { outputs: outputs.clone() });
    assert_eq!(
        app.daemon.output_statuses.iter().map(|out| (out.mute, out.volume)).collect::<Vec<_>>(),
        [(false, 80), (false, 80), (true, 5)]
    );
    let _ = update(&mut app, Message::Audio(AudioMsg::MonVolume("@monitor:DP-1".into(), 43)));
    assert!(!app.panels.audio_volumes.is_empty());
    let _ = update(&mut app, Message::ToggleSettings);
    assert!(app.panels.audio_volumes.is_empty());
    app.on_outputs(OutputsResult { outputs });
    assert_eq!(app.daemon.output_statuses[0].volume, 80);
}

#[test]
fn pending_volume_does_not_follow_replaced_wallpaper() {
    use crate::contracts::daemon::{OutputStatus, OutputsResult};
    use crate::contracts::media::MediaKind;
    use crate::frontend::audio_panel::AudioMsg;
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.tab = "wallpapers".into();
    let mut output = OutputStatus {
        name: "DP-1".into(),
        target: "@monitor:first".into(),
        kind: MediaKind::Video,
        current: "first.mp4".into(),
        volume: 80,
        connected: true,
        ..Default::default()
    };
    app.on_outputs(OutputsResult { outputs: vec![output.clone()] });
    let _ = update(&mut app, Message::Audio(AudioMsg::MonVolume("@monitor:first".into(), 43)));
    output.current = "replacement.mp4".into();
    output.volume = 25;
    app.on_outputs(OutputsResult { outputs: vec![output] });
    assert!(app.panels.audio_volumes.is_empty());
    assert_eq!(app.daemon.output_statuses[0].volume, 25);
}

#[test]
fn leaving_volume_view_cancels_drag_with_another_panel_retained() {
    use crate::contracts::daemon::{OutputStatus, OutputsResult};
    use crate::contracts::media::MediaKind;
    use crate::frontend::audio_panel::{AudioMsg, AudioPanel};
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.tab = "wallpapers".into();
    app.panels.audio = Some(AudioPanel::new());
    app.on_outputs(OutputsResult {
        outputs: vec![OutputStatus {
            name: "DP-1".into(),
            kind: MediaKind::Video,
            current: "first.mp4".into(),
            volume: 80,
            connected: true,
            ..Default::default()
        }],
    });
    let _ = update(&mut app, Message::Audio(AudioMsg::MonVolume("DP-1".into(), 43)));
    assert!(!app.panels.audio_volumes.is_empty());
    let _ = update(&mut app, Message::ToggleSettings);
    assert!(app.panels.audio.is_some());
    assert!(app.panels.audio_volumes.is_empty());
}
