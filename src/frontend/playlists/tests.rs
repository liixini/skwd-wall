#![cfg(test)]

use super::filter::{source_colors, source_folders};
use super::{Playlists, toggle_color_clause, toggle_folder_clause};
use crate::contracts::playlists::Playlist;

fn two_lists() -> Playlists {
    Playlists {
        lists: vec![
            Playlist {
                id: 1,
                name: "morning".into(),
                dwell: 120,
                source: Some("favourites".into()),
                ..Playlist::default()
            },
            Playlist { id: 2, name: "evening".into(), dwell: 600, ..Playlist::default() },
        ],
        ..Playlists::default()
    }
}

#[test]
fn sync_buffers_defaults() {
    let mut panel = two_lists();
    panel.selected = Some(1);
    panel.sync_buffers();
    assert_eq!(panel.name_buf, "morning");
    assert_eq!(panel.dwell_buf, "120");
    assert_eq!(panel.source_buf, "favourites");

    panel.selected = Some(2);
    panel.sync_buffers();
    assert_eq!(panel.name_buf, "evening");
    assert_eq!(panel.dwell_buf, "600");
    assert_eq!(panel.source_buf, "");
}

#[test]
fn sync_buffers_switch() {
    let mut panel = two_lists();
    panel.selected = Some(1);
    panel.sync_buffers();
    panel.name_buf = String::from("morning RENAMED");
    panel.selected = Some(2);
    panel.sync_buffers();
    assert_eq!(panel.name_buf, "evening");
    assert_eq!(panel.dwell_buf, "600");
}

#[test]
fn sync_buffers_noselect() {
    let mut panel = two_lists();
    panel.name_buf = String::from("draft");
    panel.dwell_buf = String::from("42");
    panel.selected = None;
    panel.sync_buffers();
    assert_eq!(panel.name_buf, "draft");
    assert_eq!(panel.dwell_buf, "42");

    panel.selected = Some(99);
    panel.sync_buffers();
    assert_eq!(panel.name_buf, "draft");
}

#[test]
fn color_clause_toggle() {
    assert_eq!(toggle_color_clause("", 8), "color:blue");
    assert_eq!(toggle_color_clause("tag:cat", 8), "tag:cat color:blue");
    assert_eq!(toggle_color_clause("tag:cat color:blue", 0), "tag:cat color:blue,red");
    assert_eq!(toggle_color_clause("color:blue,red", 8), "color:red");
    assert_eq!(toggle_color_clause("tag:cat color:blue", 8), "tag:cat");
    assert_eq!(toggle_color_clause("color:red type:video", 99), "type:video color:red,gray");
}

#[test]
fn source_color_buckets() {
    assert_eq!(source_colors("tag:cat color:blue,red"), vec![8, 0]);
    assert_eq!(source_colors("colour:8 tag:sky"), vec![8]);
    assert_eq!(source_colors("color:gray"), vec![99]);
    assert!(source_colors("tag:cat").is_empty());
}

#[test]
fn index_uses_folio_shell() {
    let source = include_str!("view.rs");
    assert!(source.contains("crate::frontend::ui::folio_index_shell("));
    assert_eq!(source.matches("folio_rule(palette)").count(), 6);
    assert!(!source.contains("with_alpha(palette.background, 0.9)"));
}

#[test]
fn folder_toggles_roundtrip() {
    let source = toggle_folder_clause("all", "/w/Gwenni good");
    assert_eq!(source, r#"folder:"/w/Gwenni good""#);
    let source = toggle_folder_clause(&format!("{source} color:blue"), "/w/anime");
    assert_eq!(source_folders(&source), ["/w/Gwenni good", "/w/anime"]);
    assert_eq!(source_colors(&source), [8]);
    let source = toggle_folder_clause(&source, "/w/Gwenni good");
    assert_eq!(source, "color:blue folder:/w/anime");
    assert_eq!(toggle_folder_clause("folder:/w/anime", "/w/anime"), "all");
}
