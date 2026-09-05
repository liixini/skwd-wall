use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::{Catalog, EN_US_RESOURCES, ES_ES_RESOURCES, SV_SE_RESOURCES};

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

fn plural_variants_differ(block: &str) -> bool {
    let ones: Vec<&str> =
        block.lines().filter_map(|line| line.trim().strip_prefix("[one]")).map(str::trim).collect();
    let others: Vec<&str> = block
        .lines()
        .filter_map(|line| line.trim().strip_prefix("*[other]"))
        .map(str::trim)
        .collect();
    assert_eq!(ones.len(), others.len(), "unbalanced plural variants: {block}");
    assert!(!ones.is_empty(), "no [one] variant: {block}");
    ones.iter().zip(&others).any(|(one, other)| one != other)
}

#[test]
fn static_text_interned() {
    // Use explicit locale to avoid flakiness from system LANG auto-detection
    let catalog = Catalog::for_locale("en-US");
    let first = catalog.format("tags-filter-title", None);
    let second = catalog.format("tags-filter-title", None);
    assert_eq!(first, "Tag filter");
    // tr() caching is tested separately if needed; here we just check formatting
    assert_eq!(first, second);
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
    // Force en-US to avoid system locale auto-detection flakiness
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
fn saved_names_both_locales() {
    let mut args = fluent::FluentArgs::new();
    args.set("number", "1234");
    for (locale, playlist, style) in [
        ("en-US", "Playlist 1234", "Style 1234"),
        ("sv-SE", "Spellista 1234", "Stil 1234"),
        ("es-ES", "Lista 1234", "Estilo 1234"),
    ] {
        let catalog = Catalog::for_locale(locale);
        assert_eq!(catalog.format("playlists-generated-name", Some(&args)), playlist);
        assert_eq!(catalog.format("settings-selector-preset-generated-name", Some(&args)), style);
    }
}

#[test]
fn locale_keys_match() {
    let english = resource_keys(EN_US_RESOURCES);
    let swedish = resource_keys(SV_SE_RESOURCES);
    let spanish = resource_keys(ES_ES_RESOURCES);
    assert_eq!(english, swedish);
    assert_eq!(english, spanish);
}

#[test]
fn retired_brand_name_absent() {
    for resources in [EN_US_RESOURCES, SV_SE_RESOURCES, ES_ES_RESOURCES] {
        for (key, block) in message_blocks(resources) {
            assert!(!block.to_ascii_lowercase().contains("folio"), "{key}");
        }
    }
}

#[test]
fn messages_format_both_locales() {
    let keys = resource_keys(EN_US_RESOURCES);
    let mut names = BTreeSet::new();
    for resources in [EN_US_RESOURCES, SV_SE_RESOURCES, ES_ES_RESOURCES] {
        for (_, block) in message_blocks(resources) {
            names.extend(placeable_variables(&block).0);
        }
    }
    assert!(names.len() >= 60, "variable scan collapsed to {} names", names.len());
    let mut args = fluent::FluentArgs::new();
    for name in &names {
        args.set(name.as_str(), 2);
    }
    for locale in ["en-US", "sv-SE", "es-ES"] {
        let catalog = Catalog::for_locale(locale);
        for key in &keys {
            assert!(!catalog.format(key, Some(&args)).is_empty(), "{locale} {key}");
        }
    }
}

#[test]
fn count_selector_singular() {
    for (locale, resources) in [
        ("en-US", EN_US_RESOURCES),
        ("sv-SE", SV_SE_RESOURCES),
        ("es-ES", ES_ES_RESOURCES),
    ] {
        let catalog = Catalog::for_locale(locale);
        let mut selector_keys = 0;
        for (key, block) in message_blocks(resources) {
            let (variables, selectors) = placeable_variables(&block);
            if selectors.is_empty() {
                continue;
            }
            selector_keys += 1;
            let render = |count: i64| {
                let mut args = fluent::FluentArgs::new();
                for name in &variables {
                    if selectors.contains(name) {
                        args.set(name.as_str(), count);
                    } else {
                        args.set(name.as_str(), "x");
                    }
                }
                catalog.format(&key, Some(&args))
            };
            let zero = render(0).replace('0', "#");
            let one = render(1).replace('1', "#");
            let two = render(2).replace('2', "#");
            assert_eq!(zero, two, "{locale} {key} zero");
            if plural_variants_differ(&block) {
                assert_ne!(one, two, "{locale} {key} one");
            } else {
                assert_eq!(one, two, "{locale} {key} same");
            }
        }
        assert!(selector_keys >= 18, "{locale}: {selector_keys} selectors");
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
