#![cfg(test)]

use super::audio_panel::{AudioMon, AudioPanel, label_for};

fn panel() -> AudioPanel {
    let mut out = AudioPanel::new();
    out.mons = vec![AudioMon {
        name: String::from("DP-1"),
        label: String::from("clip.mp4"),
        source: String::from("/video/clip.mp4"),
        wtype: crate::contracts::media::MediaKind::Video,
        mute: true,
        volume: 80,
        shared: false,
        paused: false,
        manual_paused: false,
    }];
    out
}

#[test]
fn label_for_types() {
    use crate::contracts::media::MediaKind;

    assert_eq!(label_for(&MediaKind::Video, "/a/b/clip.mp4", ""), "clip.mp4");
    assert_eq!(label_for(&MediaKind::Video, "", ""), "Video");
    assert_eq!(label_for(&MediaKind::WallpaperEngine, "/x", "12345"), "Wallpaper Engine (12345)");
    assert_eq!(label_for(&MediaKind::WallpaperEngine, "/x", ""), "Wallpaper Engine");
    assert_eq!(label_for(&MediaKind::Static, "/a/b.png", ""), "Static image");
    assert_eq!(label_for(&MediaKind::Other("bogus".into()), "/a/b.png", ""), "-");
}

#[test]
fn volume_clamp() {
    let mut audio = panel();
    audio.set_mon_volume("DP-1", 250);
    assert_eq!(audio.mons[0].volume, 100);
    audio.set_mon_volume("DP-1", 35);
    assert_eq!(audio.mons[0].volume, 35);
    audio.set_mon_volume("HDMI-1", 10);
    assert_eq!(audio.mons[0].volume, 35);
}

#[test]
fn mute_scoped() {
    let mut audio = panel();
    audio.set_mon_mute("DP-1", false);
    assert!(!audio.mons[0].mute);
    audio.set_mon_mute("HDMI-1", true);
    assert!(!audio.mons[0].mute);
}

#[test]
fn playing_requires_audio_channel() {
    let mut audio = panel();
    assert!(!audio.any_playing());

    audio.set_mon_mute("DP-1", false);
    assert!(audio.any_playing());

    audio.set_mon_volume("DP-1", 0);
    assert!(!audio.any_playing());

    audio.mons[0].wtype = crate::contracts::media::MediaKind::Static;
    audio.set_mon_volume("DP-1", 80);
    assert!(!audio.any_playing());
}

#[test]
fn uses_folio_index_shell() {
    let source = include_str!("audio_panel.rs");
    assert!(source.contains("crate::frontend::ui::folio_index_shell("));
    assert!(source.contains("folio_rule(palette)"));
    assert!(!source.contains("with_alpha(palette.background, 0.9)"));
    assert!(!source.contains("with_alpha(palette.outline, 0.58)"));
}
