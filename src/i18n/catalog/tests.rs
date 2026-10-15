use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::{Catalog, EN_US_RESOURCES, LANGUAGES};

fn locales() -> impl Iterator<Item = (&'static str, &'static [&'static str])> {
    LANGUAGES.iter().map(|language| {
        let resources = if language.tag == "en-US" { EN_US_RESOURCES } else { language.resources };
        (language.tag, resources)
    })
}

fn plural_categories(tag: &str) -> &'static [&'static [i64]] {
    match tag {
        "pt-BR" | "fr-FR" | "bn-BD" | "hi-IN" | "fa-IR" => &[&[0, 1], &[2, 5, 21]],
        "ar-SA" => &[&[0], &[1], &[2], &[3, 10], &[11, 25, 99], &[100, 103, 1000]],
        "ru-RU" => &[&[1, 21], &[2, 3, 22], &[0, 5, 11, 25]],
        "zh-CN" | "ja-JP" => &[&[0, 1, 2, 5, 21]],
        _ => &[&[1], &[0, 2, 5, 21]],
    }
}

fn resource_keys(resources: &[&str]) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for resource in resources {
        for line in resource.lines() {
            if line.starts_with([' ', '\t', '#', '-']) {
                continue;
            }
            let Some((candidate, _)) = line.split_once('=') else {
                continue;
            };
            let key = candidate.trim();
            if key.is_empty() {
                continue;
            }
            assert!(keys.insert(key.to_owned()), "duplicate key {key}");
        }
    }
    keys
}

fn message_blocks(resources: &[&str]) -> Vec<(String, String)> {
    let mut blocks: Vec<(String, String)> = Vec::new();
    for resource in resources {
        for line in resource.lines() {
            if line.starts_with([' ', '\t']) {
                let (_, block) = blocks.last_mut().expect("continuation line without a message");
                block.push('\n');
                block.push_str(line);
                continue;
            }
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let (key, rest) = line.split_once('=').expect("top-level line must be a message");
            blocks.push((key.trim().to_owned(), rest.to_owned()));
        }
    }
    blocks
}

fn placeable_variables(block: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut variables = BTreeSet::new();
    let mut selectors = BTreeSet::new();
    let mut rest = block;
    while let Some(index) = rest.find('$') {
        rest = &rest[index + 1..];
        let end = rest
            .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
            .unwrap_or(rest.len());
        assert!(end > 0, "dangling $: {block}");
        let name = rest[..end].to_owned();
        if rest[end..].trim_start().starts_with("->") {
            selectors.insert(name.clone());
        }
        variables.insert(name);
        rest = &rest[end..];
    }
    (variables, selectors)
}

#[test]
fn language_table_is_consistent() {
    let mut prefixes = BTreeSet::new();
    for language in LANGUAGES {
        assert!(language.tag.starts_with(language.prefix), "{}", language.tag);
        assert!(prefixes.insert(language.prefix), "duplicate prefix {}", language.prefix);
        assert_eq!(super::language_choice(language.tag), language.tag);
        assert_eq!(language.resources.is_empty(), language.tag == "en-US", "{}", language.tag);
    }
}

#[test]
fn right_to_left_follows_script() {
    for language in LANGUAGES {
        assert_eq!(language.rtl, language.script == super::Script::Arabic, "{}", language.tag);
    }
    assert!(LANGUAGES.iter().filter(|language| language.rtl).count() == 3);
}

#[test]
fn static_text_interned() {
    let first = super::tr("tags-filter-title");
    let second = super::tr("tags-filter-title");
    assert_eq!(first, super::catalog().format("tags-filter-title", None));
    assert_eq!(first.as_ptr(), second.as_ptr());
}

#[test]
fn tr_args_named_variables() {
    assert_eq!(
        crate::i18n::tr_args!("status-apply-detail", heading => "Apply failed", detail => "decode"),
        "Apply failed (decode)"
    );
}

#[test]
#[should_panic(expected = "could not format translation status-apply-detail")]
fn tr_args_wrong_variable() {
    crate::i18n::tr_args!("status-apply-detail", wrong => 2);
}

#[test]
fn padded_counts_verbatim() {
    let catalog = Catalog::for_locale("en-US");
    let mut args = fluent::FluentArgs::new();
    args.set("count", 1);
    args.set("padded", "01");
    assert_eq!(catalog.format("playlists-state-ready", Some(&args)), "01 wallpaper ready");
    let mut args2 = fluent::FluentArgs::new();
    args2.set("count", 24);
    args2.set("padded", "24");
    assert_eq!(catalog.format("playlists-state-ready", Some(&args2)), "24 wallpapers ready");
}

#[test]
fn swedish_overrides_english() {
    let catalog = Catalog::for_locale("sv-SE");
    assert_eq!(catalog.format("tags-done", None), "Klar");

    let mut args = fluent::FluentArgs::new();
    args.set("heading", "Misslyckades");
    args.set("detail", "avkodning");
    assert_eq!(catalog.format("status-apply-detail", Some(&args)), "Misslyckades (avkodning)");
}

#[test]
fn saved_names_every_locale() {
    let mut args = fluent::FluentArgs::new();
    args.set("number", "1234");
    let cases = [
        ("en-US", "Playlist 1234", "Style 1234"),
        ("sv-SE", "Spellista 1234", "Stil 1234"),
        ("es-ES", "Lista 1234", "Estilo 1234"),
        ("pt-BR", "Playlist 1234", "Estilo 1234"),
        ("ru-RU", "Плейлист 1234", "Стиль 1234"),
        ("zh-CN", "播放列表 1234", "样式 1234"),
        ("ja-JP", "プレイリスト 1234", "スタイル 1234"),
        ("ar-SA", "قائمة تشغيل 1234", "النمط 1234"),
        ("fr-FR", "Liste de lecture 1234", "Style 1234"),
        ("bn-BD", "প্লেলিস্ট 1234", "স্টাইল 1234"),
        ("ur-PK", "پلے لسٹ 1234", "انداز 1234"),
        ("hi-IN", "प्लेलिस्ट 1234", "स्टाइल 1234"),
        ("fa-IR", "فهرست پخش 1234", "سبک 1234"),
        ("tr-TR", "Oynatma listesi 1234", "Stil 1234"),
    ];
    assert_eq!(cases.len(), LANGUAGES.len());
    for (locale, playlist, style) in cases {
        let catalog = Catalog::for_locale(locale);
        assert_eq!(catalog.format("playlists-generated-name", Some(&args)), playlist);
        assert_eq!(catalog.format("settings-selector-preset-generated-name", Some(&args)), style);
    }
}

#[test]
fn locale_keys_match() {
    let english = resource_keys(EN_US_RESOURCES);
    for (tag, resources) in locales() {
        let keys = resource_keys(resources);
        let missing: Vec<_> = english.difference(&keys).collect();
        let extra: Vec<_> = keys.difference(&english).collect();
        assert!(
            missing.is_empty() && extra.is_empty(),
            "{tag} missing {missing:?} extra {extra:?}"
        );
    }
}

#[test]
fn retired_brand_name_absent() {
    for (tag, resources) in locales() {
        for (key, block) in message_blocks(resources) {
            assert!(!block.to_ascii_lowercase().contains("folio"), "{tag} {key}");
        }
    }
}

#[test]
fn messages_format_every_locale() {
    let keys = resource_keys(EN_US_RESOURCES);
    let mut names = BTreeSet::new();
    for (_, resources) in locales() {
        for (_, block) in message_blocks(resources) {
            names.extend(placeable_variables(&block).0);
        }
    }
    assert!(names.len() >= 60, "variable scan collapsed to {} names", names.len());
    let mut args = fluent::FluentArgs::new();
    for name in &names {
        args.set(name.as_str(), 2);
    }
    for (tag, _) in locales() {
        let catalog = Catalog::for_locale(tag);
        for key in &keys {
            assert!(!catalog.format(key, Some(&args)).is_empty(), "{tag} {key}");
        }
    }
}

#[test]
fn plural_variants_follow_cldr_categories() {
    let selector_keys: Vec<(String, BTreeSet<String>, BTreeSet<String>)> =
        message_blocks(EN_US_RESOURCES)
            .into_iter()
            .filter_map(|(key, block)| {
                let (variables, selectors) = placeable_variables(&block);
                (!selectors.is_empty()).then_some((key, variables, selectors))
            })
            .collect();
    assert!(selector_keys.len() >= 16, "{} selectors", selector_keys.len());
    for (tag, _) in locales() {
        let catalog = Catalog::for_locale(tag);
        let categories = plural_categories(tag);
        let mut distinguishing = 0;
        for (key, variables, selectors) in &selector_keys {
            let render = |count: i64| {
                let mut args = fluent::FluentArgs::new();
                for name in variables {
                    if selectors.contains(name) {
                        args.set(name.as_str(), count);
                    } else {
                        args.set(name.as_str(), "x");
                    }
                }
                catalog.format(key, Some(&args)).replace(&count.to_string(), "#")
            };
            let rendered: Vec<Vec<String>> = categories
                .iter()
                .map(|samples| samples.iter().map(|&count| render(count)).collect())
                .collect();
            let shows_count = rendered.iter().flatten().any(|text| text.contains('#'));
            for (samples, texts) in categories.iter().zip(&rendered) {
                assert!(
                    texts.iter().all(|text| text == &texts[0]),
                    "{tag} {key} {samples:?}: {texts:?}"
                );
                if samples.len() > 1 && shows_count {
                    assert!(
                        texts[0].contains('#'),
                        "{tag} {key} {samples:?} hides the count: {}",
                        texts[0]
                    );
                }
            }
            let distinct: BTreeSet<&String> = rendered.iter().map(|texts| &texts[0]).collect();
            if distinct.len() == categories.len() {
                distinguishing += 1;
            }
        }
        if categories.len() > 1 && !matches!(tag, "fa-IR" | "tr-TR") {
            assert!(distinguishing > 0, "{tag} never distinguishes its plural categories");
        }
    }
}

fn rust_sources() -> Vec<(PathBuf, String)> {
    fn visit(directory: &Path, files: &mut Vec<(PathBuf, String)>) {
        for entry in std::fs::read_dir(directory).expect("read source directory") {
            let entry = entry.expect("source directory entry");
            let path = entry.path();
            if path.is_dir() {
                visit(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let source = std::fs::read_to_string(&path).expect("read source file");
                files.push((path, source));
            }
        }
    }
    let mut files = Vec::new();
    visit(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src")), &mut files);
    files
}

fn is_message_key(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn literal_keys_after(source: &str, pattern: &str, keys: &mut BTreeSet<String>) {
    let mut offset = 0;
    while let Some(index) = source[offset..].find(pattern) {
        let at = offset + index;
        let bounded = source[..at]
            .chars()
            .next_back()
            .is_none_or(|previous| !previous.is_alphanumeric() && previous != '_');
        let tail = source[at + pattern.len()..].trim_start();
        if bounded
            && let Some(tail) = tail.strip_prefix('"')
            && let Some(end) = tail.find('"')
            && is_message_key(&tail[..end])
        {
            keys.insert(tail[..end].to_owned());
        }
        offset = at + pattern.len();
    }
}

#[test]
fn source_lookups_resolve() {
    let english = resource_keys(EN_US_RESOURCES);
    let mut missing = BTreeSet::new();
    for (path, source) in rust_sources() {
        let mut used = BTreeSet::new();
        literal_keys_after(&source, "tr(", &mut used);
        literal_keys_after(&source, "tr_args!(", &mut used);
        if path.starts_with(concat!(env!("CARGO_MANIFEST_DIR"), "/src/i18n")) {
            literal_keys_after(&source, ".format(", &mut used);
            literal_keys_after(&source, "label: ", &mut used);
        }
        for key in used {
            if !english.contains(&key) {
                missing.insert(format!("{key} ({})", path.display()));
            }
        }
    }
    assert!(missing.is_empty(), "{missing:#?}");
}

#[test]
fn saved_names_not_embedded() {
    let forbidden = [
        concat!("\"", "Play", "list {"),
        concat!("\"", "Sty", "le {"),
        concat!("String::from(\"", "Style\")"),
        concat!("map_or(\"", "Playlist\""),
    ];
    let mut offenders = Vec::new();
    for (path, source) in rust_sources() {
        for fragment in forbidden {
            if source.contains(fragment) {
                offenders.push(format!("{}: {fragment}", path.display()));
            }
        }
    }
    assert!(offenders.is_empty(), "{offenders:#?}");
}

fn balanced_argument(source: &str) -> &str {
    let mut depth = 1usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in source.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &source[..index];
                }
            }
            _ => {}
        }
    }
    source
}

#[test]
fn toasts_are_translated() {
    let pattern = concat!(".show_", "toast(");
    let daemon_passthrough = ["backend_warning.to_string()"];
    let translated_locals = ["summary", "warning"];
    let translated_calls =
        ["crate::i18n::tr(", "crate::i18n::tr_args!(", "tr(", "tr_args!(", "apply_error_message("];
    let mut offenders = Vec::new();
    for (path, source) in rust_sources() {
        let mut offset = 0;
        while let Some(found) = source[offset..].find(pattern) {
            let start = offset + found + pattern.len();
            offset = start;
            let argument = balanced_argument(&source[start..]);
            let flat_argument = argument.split_whitespace().collect::<Vec<_>>().join(" ");
            let flat_argument = flat_argument.trim_end_matches(',').trim();
            if daemon_passthrough.contains(&flat_argument)
                || translated_locals.contains(&flat_argument)
            {
                continue;
            }
            if translated_calls.iter().any(|call| flat_argument.starts_with(call)) {
                if let Some(tail) = flat_argument.split_once('"').map(|(_, tail)| tail)
                    && let Some((literal, _)) = tail.split_once('"')
                    && !is_message_key(literal)
                {
                    offenders.push(format!("{}: show_toast({flat_argument})", path.display()));
                }
                continue;
            }
            offenders.push(format!("{}: show_toast({flat_argument})", path.display()));
        }
    }
    assert!(offenders.is_empty(), "{offenders:#?}");
}

#[test]
fn catalog_has_no_orphaned_keys() {
    let english = resource_keys(EN_US_RESOURCES);
    let corpus =
        rust_sources().into_iter().map(|(_, source)| source).collect::<Vec<_>>().join("\n");
    let orphaned: Vec<&String> =
        english.iter().filter(|key| !corpus.contains(&format!("\"{key}\""))).collect();
    assert!(orphaned.is_empty(), "{orphaned:#?}");
}

#[test]
fn locale_environment_respects_overrides_and_message_priority() {
    let cases = [
        ([None, None, None, None, None], "en-US"),
        ([None, None, None, Some("es_ES.UTF-8"), None], "es-ES"),
        ([None, Some("C"), None, Some("es_ES.UTF-8"), Some("es")], "en-US"),
        ([None, Some("C.UTF-8"), None, Some("es"), Some("es")], "en-US"),
        ([None, Some("POSIX"), None, Some("es"), None], "en-US"),
        ([None, Some("de_DE.UTF-8"), Some("es"), Some("sv"), None], "en-US"),
        ([None, None, Some("sv_SE.UTF-8"), Some("es"), None], "sv-SE"),
        ([None, None, None, Some("en_US.UTF-8"), Some("fr:es_MX:sv")], "fr-FR"),
        ([None, None, None, Some("en_US.UTF-8"), Some("de:es_MX:sv")], "es-ES"),
        ([None, None, Some("ar_EG.UTF-8"), None, None], "ar-SA"),
        ([None, None, None, Some("bn_IN.UTF-8"), None], "bn-BD"),
        ([None, None, None, Some("ur_IN.UTF-8"), Some("hi")], "hi-IN"),
        ([None, None, None, Some("es_ES.UTF-8"), Some("C:sv")], "en-US"),
        ([Some("sv-SE"), Some("C"), None, Some("es"), None], "sv-SE"),
        ([Some(" ES_mx.UTF-8 "), Some("C"), None, None, None], "es-ES"),
        ([Some("de"), None, None, Some("es"), None], "en-US"),
        ([Some(" "), Some(""), None, Some("es"), None], "es-ES"),
        ([None, None, None, Some("estonian"), None], "en-US"),
        ([None, None, None, Some("pt_BR.UTF-8"), None], "pt-BR"),
        ([None, None, None, Some("en_US.UTF-8"), Some("ja:en")], "ja-JP"),
        ([None, None, Some("ru_RU.UTF-8"), Some("zh_CN.UTF-8"), None], "ru-RU"),
        ([None, None, Some("fa_IR.UTF-8"), None, None], "fa-IR"),
        ([None, None, None, Some("tr_TR.UTF-8"), None], "tr-TR"),
        ([None, None, None, Some("en_US.UTF-8"), Some("fa:tr")], "fa-IR"),
        ([Some("tr"), Some("C"), None, None, None], "tr-TR"),
        ([Some("zh_CN.UTF-8"), None, None, Some("ru"), None], "zh-CN"),
        ([None, None, None, Some("zh_TW.UTF-8"), None], "en-US"),
        ([None, None, None, Some("en_US.UTF-8"), Some("zh_HK:ja")], "ja-JP"),
    ];
    for (values, expected) in cases {
        assert_eq!(super::selected_locale(values), expected, "{values:?}");
    }
}

#[test]
fn spanish_overrides_english_and_accepts_regional_locales() {
    for locale in ["es", "es-ES", "es_MX.UTF-8", "ES_ar"] {
        let catalog = Catalog::for_locale(locale);
        assert_eq!(catalog.format("browser-apply", None), "Aplicar");
        assert_eq!(catalog.format("settings-performance-device-label", None), "GPU del fondo");
    }
}

#[test]
fn added_locales_override_english_and_accept_regional_variants() {
    for (requested, expected) in [
        ("pt_BR.UTF-8", "Pausar"),
        ("pt-PT", "Pausar"),
        ("ru_RU.UTF-8", "Пауза"),
        ("RU", "Пауза"),
        ("zh_CN.UTF-8", "暂停"),
        ("zh-SG", "暂停"),
        ("ja_JP.UTF-8", "一時停止"),
        ("ja", "一時停止"),
        ("ar_EG.UTF-8", "إيقاف مؤقت"),
        ("ar", "إيقاف مؤقت"),
        ("fr_CA.UTF-8", "Pause"),
        ("fr-BE", "Pause"),
        ("bn_IN.UTF-8", "বিরতি"),
        ("bn", "বিরতি"),
        ("ur_IN.UTF-8", "روکیں"),
        ("ur", "روکیں"),
        ("hi_IN.UTF-8", "रोकें"),
        ("hi", "रोकें"),
        ("fa_IR.UTF-8", "مکث"),
        ("fa", "مکث"),
        ("tr_TR.UTF-8", "Duraklat"),
        ("TR", "Duraklat"),
    ] {
        let catalog = Catalog::for_locale(requested);
        assert_eq!(catalog.format("filter-bar-pause", None), expected, "{requested}");
    }
}

#[test]
fn static_text_is_cached_per_language() {
    let cases = [
        ("en-US", "Done"),
        ("sv-SE", "Klar"),
        ("es-ES", "Hecho"),
        ("pt-BR", "Concluído"),
        ("ru-RU", "Готово"),
        ("zh-CN", "完成"),
        ("ja-JP", "完了"),
        ("ar-SA", "تم"),
        ("fr-FR", "Terminé"),
        ("bn-BD", "সম্পন্ন"),
        ("ur-PK", "ہو گیا"),
        ("hi-IN", "हो गया"),
        ("fa-IR", "انجام شد"),
        ("tr-TR", "Tamam"),
    ];
    assert_eq!(cases.len(), LANGUAGES.len());
    for (locale, expected) in cases {
        let catalog = Catalog::for_locale(locale);
        let first = catalog.text("tags-done");
        assert_eq!(first, expected);
        for _ in 0..20 {
            assert_eq!(catalog.text("tags-done").as_ptr(), first.as_ptr());
        }
        assert_eq!(catalog.static_text.lock().unwrap().len(), 1);
    }
}

#[test]
fn saved_language_choices_normalize_to_supported_options() {
    for (requested, expected) in [
        ("auto", "auto"),
        ("", "auto"),
        ("de-DE", "auto"),
        ("fr-FR", "fr-FR"),
        ("fr_CA.UTF-8", "fr-FR"),
        ("ar", "ar-SA"),
        ("bn_IN", "bn-BD"),
        ("ur_IN.UTF-8", "ur-PK"),
        ("hi-IN", "hi-IN"),
        ("fa_IR.UTF-8", "fa-IR"),
        ("fa", "fa-IR"),
        ("tr_TR.UTF-8", "tr-TR"),
        (" TR ", "tr-TR"),
        ("en", "en-US"),
        ("sv_SE.UTF-8", "sv-SE"),
        (" ES_mx ", "es-ES"),
        ("pt_BR.UTF-8", "pt-BR"),
        (" ru ", "ru-RU"),
        ("zh-CN", "zh-CN"),
        ("zh_SG.UTF-8", "zh-CN"),
        ("zh-Hans-HK", "zh-CN"),
        ("zh_TW.UTF-8", "auto"),
        ("zh-Hant", "auto"),
        ("zh_HK", "auto"),
        ("zh_MO.UTF-8", "auto"),
        ("ja_JP.UTF-8", "ja-JP"),
    ] {
        assert_eq!(super::language_choice(requested), expected);
    }
}

#[test]
fn persian_and_turkish_preserve_message_arguments() {
    let english: std::collections::BTreeMap<_, _> = message_blocks(EN_US_RESOURCES)
        .into_iter()
        .map(|(key, block)| (key, placeable_variables(&block)))
        .collect();
    for tag in ["fa-IR", "tr-TR"] {
        for (key, block) in message_blocks(super::language(tag).resources) {
            assert_eq!(placeable_variables(&block), english[&key], "{tag} {key}");
        }
    }
}

#[test]
fn ui_fonts_cover_catalogs() {
    use iced_wgpu::graphics::text::cosmic_text::{Font, fontdb};
    for (tag, resources) in locales() {
        let language = super::language(tag);
        let mut db = fontdb::Database::new();
        db.load_font_data(crate::shell::ui_font_bytes(language.script).to_vec());
        let face = db.faces().next().expect("embedded UI face");
        assert!(
            face.families.iter().any(|(name, _)| name == crate::frontend::ui::UI_FONT_FAMILY),
            "{}",
            language.tag
        );
        assert_eq!(face.weight, fontdb::Weight::BOLD, "{}", language.tag);
        assert_eq!(face.stretch, fontdb::Stretch::Condensed, "{}", language.tag);
        let font = Font::new(&db, face.id, face.weight).expect("parse UI face");
        let charmap = font.as_swash().charmap();
        let missing: BTreeSet<char> = resources
            .iter()
            .flat_map(|resource| resource.chars())
            .filter(|ch| !ch.is_whitespace() && !ch.is_control())
            .filter(|ch| !('\u{200c}'..='\u{200f}').contains(ch))
            .filter(|ch| charmap.map(*ch) == 0)
            .collect();
        assert!(missing.is_empty(), "{} {missing:?}", language.tag);
    }
}
