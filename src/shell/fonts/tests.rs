use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use fontconfig::{self, Fontconfig, Pattern};
use iced_wgpu::graphics::text::cosmic_text::fontdb::{Database, Source, Stretch, Style, Weight};

use super::{database, face, stretch, weight};

fn pattern(config: &Fontconfig, index: i32) -> Pattern<'_> {
    let mut pattern = Pattern::new(config).unwrap();
    pattern.add_string(fontconfig::FC_FILE, c"/no/such/font.ttc").unwrap();
    pattern.add_integer(fontconfig::FC_INDEX, index).unwrap();
    pattern.add_string(fontconfig::FC_FAMILY, c"Test Family").unwrap();
    pattern
}

#[test]
fn registers_cached_face() {
    let config = Fontconfig::new().unwrap();
    let mut pattern = pattern(&config, 2);
    pattern.add_string(fontconfig::FC_FAMILY, c"Localized Family").unwrap();
    pattern.add_string(fontconfig::FC_POSTSCRIPT_NAME, c"TestFamily-Bold").unwrap();
    pattern.add_integer(fontconfig::FC_WEIGHT, fontconfig::FC_WEIGHT_BOLD).unwrap();
    pattern.add_integer(fontconfig::FC_WIDTH, fontconfig::FC_WIDTH_CONDENSED).unwrap();
    pattern.add_integer(fontconfig::FC_SLANT, fontconfig::FC_SLANT_ITALIC).unwrap();
    pattern.add_integer(fontconfig::FC_SPACING, fontconfig::FC_MONO).unwrap();
    let mut catalog = Database::new();
    catalog.push_face_info(face(&pattern).unwrap());
    let info = catalog.faces().next().unwrap();
    assert!(
        matches!(&info.source, Source::File(path) if path == &PathBuf::from("/no/such/font.ttc"))
    );
    assert_eq!(info.index, 2);
    assert_eq!(info.families.len(), 2);
    assert_eq!(info.families[1].0, "Localized Family");
    assert_eq!(info.post_script_name, "TestFamily-Bold");
    assert_eq!(info.style, Style::Italic);
    assert_eq!(info.weight, Weight::BOLD);
    assert_eq!(info.stretch, Stretch::Condensed);
    assert!(info.monospaced);
}

#[test]
fn keeps_nonutf8_path() {
    let config = Fontconfig::new().unwrap();
    let mut pattern = Pattern::new(&config).unwrap();
    pattern
        .add_string(fontconfig::FC_FILE, &CString::new(b"/fonts/\xff.ttf".to_vec()).unwrap())
        .unwrap();
    pattern.add_integer(fontconfig::FC_INDEX, 0).unwrap();
    pattern.add_string(fontconfig::FC_FAMILY, c"Test Family").unwrap();
    let info = face(&pattern).unwrap();
    assert!(
        matches!(info.source, Source::File(path) if path.as_os_str().as_bytes() == b"/fonts/\xff.ttf")
    );
}

#[test]
fn maps_font_metrics() {
    assert_eq!(weight(fontconfig::FC_WEIGHT_LIGHT), Weight::LIGHT);
    assert_eq!(weight(fontconfig::FC_WEIGHT_MEDIUM), Weight::MEDIUM);
    assert_eq!(weight(fontconfig::FC_WEIGHT_BLACK), Weight::BLACK);
    assert_eq!(stretch(fontconfig::FC_WIDTH_ULTRACONDENSED), Stretch::UltraCondensed);
    assert_eq!(stretch(fontconfig::FC_WIDTH_ULTRAEXPANDED), Stretch::UltraExpanded);
}

#[test]
fn skips_named_instance() {
    let config = Fontconfig::new().unwrap();
    let pattern = pattern(&config, 0x10002);
    assert!(face(&pattern).is_none());
}

#[test]
fn cached_catalog_shapes_text() {
    use iced_wgpu::graphics::text::cosmic_text::{
        Attrs, Buffer, Family, FontSystem, Metrics, Shaping,
    };

    if std::env::var_os("SKWD_FONT_CACHE_TEST").is_none() {
        for language in crate::i18n::LANGUAGES {
            let root = tempfile::tempdir().unwrap();
            let fonts = root.path().join("fonts");
            let cache = root.path().join("cache");
            std::fs::create_dir(&fonts).unwrap();
            std::fs::create_dir(&cache).unwrap();
            std::fs::write(fonts.join("ui.ttf"), crate::shell::ui_font_bytes(language.script))
                .unwrap();
            let config = root.path().join("fonts.conf");
            std::fs::write(
                &config,
                format!(
                    "<fontconfig><dir>{}</dir><cachedir>{}</cachedir></fontconfig>",
                    fonts.display(),
                    cache.display()
                ),
            )
            .unwrap();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "shell::fonts::tests::cached_catalog_shapes_text", "--nocapture"])
                .env("FONTCONFIG_FILE", config)
                .env("SKWD_FONT_CACHE_TEST", language.tag)
                .output()
                .unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stdout));
        }
        return;
    }
    let catalog = database().unwrap();
    assert_eq!(catalog.len(), 1);
    assert!(catalog.faces().all(|info| matches!(info.source, Source::File(_))));
    let mut system = FontSystem::new_with_locale_and_db(String::from("en-US"), catalog);
    let mut buffer = Buffer::new(&mut system, Metrics::new(14.0, 20.0));
    let tag = std::env::var("SKWD_FONT_CACHE_TEST").unwrap();
    let language = crate::i18n::LANGUAGES.iter().find(|language| language.tag == tag).unwrap();
    let catalog = crate::i18n::Catalog::for_locale(&tag);
    let text = format!("{} ✓ ⚠ ◆ ◇", catalog.format(language.label, None));
    for family in
        [Family::Name(crate::frontend::ui::UI_FONT_FAMILY), Family::SansSerif, Family::Monospace]
    {
        buffer.set_text(&mut system, &text, &Attrs::new().family(family), Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut system, false);
        let glyphs: Vec<_> = buffer.layout_runs().flat_map(|run| run.glyphs).collect();
        assert!(!glyphs.is_empty());
        assert!(glyphs.iter().all(|glyph| glyph.glyph_id != 0));
    }
}
