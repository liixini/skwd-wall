#![cfg(test)]

use std::collections::HashMap;

use super::numeric::*;
use super::search::*;

#[test]
fn stem_suffixes() {
    assert_eq!(word_stem("cats"), "cat");
    assert_eq!(word_stem("boxes"), "box");
    assert_eq!(word_stem("parties"), "party");
    assert_eq!(word_stem("leaves"), "leaf");
    assert_eq!(word_stem("running"), "run");
    assert_eq!(word_stem("jumped"), "jump");
    assert_eq!(word_stem("darkness"), "dark");
    assert_eq!(word_stem("quickly"), "quick");
    assert_eq!(word_stem("sky"), "sky");
    assert_eq!(word_stem("glass"), "glass");
}

#[test]
fn edit_distance_basics() {
    assert_eq!(edit_distance("blue", "blue"), 0);
    assert_eq!(edit_distance("blue", "blu"), 1);
    assert_eq!(edit_distance("kitten", "sitting"), 3);
    assert_eq!(edit_distance("", "abc"), 3);
}

#[test]
fn fuzzy_match_kinds() {
    assert!(fuzzy_match("landscape", "land"));
    assert!(fuzzy_match("cats", "cat"));
    assert!(fuzzy_match("mountain", "montain"));
    assert!(!fuzzy_match("ocean", "forest"));
}

#[test]
fn numeric_exact_closed_and_open_ranges() {
    let query = parse_numeric_query(
        "width:1920 height:1080..2160 duration:<30s size:>=1.5MiB res:1920x1080..3840x2160",
    );
    assert!(query.is_valid());
    assert_eq!(query.groups().len(), 5);
    assert_eq!(query.groups()[0][0].field, NumericField::Width);
    assert_eq!(
        query.groups()[0][0].constraint,
        NumericConstraint::Scalar(NumericRange::exact(1920))
    );
    assert_eq!(query.groups()[2][0].field, NumericField::Duration);
    assert_eq!(
        query.groups()[2][0].constraint,
        NumericConstraint::Scalar(NumericRange {
            lower: None,
            upper: Some(NumericBound { value: 30_000, inclusive: false }),
        })
    );
    assert_eq!(
        query.groups()[3][0].constraint,
        NumericConstraint::Scalar(NumericRange {
            lower: Some(NumericBound { value: 1_572_864, inclusive: true }),
            upper: None,
        })
    );
}

#[test]
fn numeric_aliases_or_and_exclusion_are_typed() {
    let query = parse_numeric_query("w:>1920 or h:>1920 -dur:..10s bytes:1kb..");
    assert!(query.is_valid());
    assert_eq!(query.groups().len(), 3);
    assert_eq!(query.groups()[0].len(), 2);
    assert_eq!(query.groups()[0][0].field, NumericField::Width);
    assert_eq!(query.groups()[0][1].field, NumericField::Height);
    assert!(query.groups()[1][0].excluded);
    assert_eq!(query.groups()[1][0].field, NumericField::Duration);
    assert_eq!(query.groups()[2][0].field, NumericField::FileSize);
    assert_eq!(query.chips()[0].label, "width:>1920 OR height:>1920");
}

#[test]
fn numeric_boundaries_are_inclusive_or_exclusive_as_written() {
    let item = crate::domain::library::catalog::Wallpaper {
        width: 1920,
        height: 1080,
        duration_ms: 30_000,
        filesize: 10 * 1024 * 1024,
        ..Default::default()
    };
    for query in [
        "width:1920",
        "width:1920..",
        "width:..1920",
        "width:>=1920",
        "width:<=1920",
        "duration:<=30s",
        "size:..10MiB",
    ] {
        assert!(parse_numeric_query(query).matches(&item), "{query}");
    }
    for query in ["width:>1920", "width:<1920", "duration:<30s", "size:>10MiB"] {
        assert!(!parse_numeric_query(query).matches(&item), "{query}");
    }
}

#[test]
fn malformed_numeric_predicates_fail_closed() {
    for text in [
        "width:",
        "width:..",
        "width:3840..1920",
        "duration:later",
        "size:4parsecs",
        "res:1920x",
        "res:3840x2160..1920x1080",
    ] {
        let query = parse_numeric_query(text);
        assert!(!query.is_valid(), "{text}");
        assert_eq!(query.invalid(), [text]);
        assert!(!query.matches(&crate::domain::library::catalog::Wallpaper::default()));
        assert!(query.chips()[0].invalid);
    }
}

#[test]
fn library_query_keeps_tags_and_numeric_syntax_separate() {
    let vocab = ["forest", "ocean"].into_iter().map(str::to_string).collect();
    let parsed = parse_library_search("forest width:>=1920 | height:>=1920 without ocean", &vocab);
    assert_eq!(parsed.tags, ["forest", "-ocean"]);
    assert_eq!(parsed.numeric.groups().len(), 1);
    assert_eq!(parsed.numeric.groups()[0].len(), 2);
    assert!(parsed.partial.is_empty());
}

#[test]
fn workshop_queries_match_exactly() {
    let item = crate::domain::library::catalog::Wallpaper {
        key: "we:1234567890".into(),
        ..Default::default()
    };
    for text in ["1234567890", "id:1234567890", "workshop:1234567890"] {
        let query = parse_numeric_query(text);
        assert!(query.matches(&item), "{text}");
        assert_eq!(query.query_text(), "workshop:1234567890");
        assert!(!query.matches(&crate::domain::library::catalog::Wallpaper::default()));
        assert!(!parse_numeric_query("123456789").matches(&item));
    }
    assert!(!parse_numeric_query("-1234567890").matches(&item));
    assert!(
        parse_numeric_query("-1234567890")
            .matches(&crate::domain::library::catalog::Wallpaper::default())
    );
}

#[test]
fn workshop_query_keeps_tags() {
    let parsed = parse_library_search("forest 1234567890", &vocab());
    assert_eq!(parsed.tags, ["forest"]);
    assert_eq!(parsed.numeric.query_text(), "workshop:1234567890");
    assert!(parsed.partial.is_empty());
    for text in ["id:0", "workshop:later", "workshop:>123", "id:18446744073709551616"] {
        assert!(!parse_numeric_query(text).is_valid(), "{text}");
    }
}

#[test]
fn workshop_ids_use_u64() {
    let item = crate::domain::library::catalog::Wallpaper {
        we_id: u64::MAX.to_string(),
        ..Default::default()
    };
    assert!(parse_numeric_query(&u64::MAX.to_string()).matches(&item));
}

#[test]
fn numeric_tags_stay_tags() {
    let vocab = ["2024".into(), "forest".into()].into_iter().collect();
    for (text, tags) in [
        ("2024", vec!["2024"]),
        ("-2024 forest", vec!["-2024", "forest"]),
        ("forest not 2024", vec!["forest", "-2024"]),
        ("forest not 2024 width:1920", vec!["forest", "-2024"]),
    ] {
        let parsed = parse_library_search(text, &vocab);
        assert_eq!(parsed.tags, tags);
        assert_eq!(parsed.numeric.is_empty(), !text.contains("width:"));
    }
    let explicit = parse_library_search("id:2024", &vocab);
    assert_eq!(explicit.numeric.query_text(), "workshop:2024");
    assert!(explicit.tags.is_empty());
}

fn db() -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    map.insert("a".into(), vec!["nature".into(), "green".into(), "forest".into()]);
    map.insert("b".into(), vec!["nature".into(), "blue".into(), "ocean".into()]);
    map.insert("c".into(), vec!["nature".into(), "green".into(), "hills".into()]);
    map.insert("d".into(), vec!["city".into(), "night".into()]);
    map
}

#[test]
fn popular_counts() {
    let (tags, _) = recompute_tag_cloud(&db(), &[], "", 60);
    let nature = tags.iter().find(|entry| entry.tag == "nature").unwrap();
    assert_eq!(nature.count, 3);
    assert_eq!(tags[0].tag, "nature");
    let green = tags.iter().find(|entry| entry.tag == "green").unwrap();
    assert_eq!(green.count, 2);
}

#[test]
fn cooccurrence_selected() {
    let selected = vec!["nature".to_string()];
    let (tags, _) = recompute_tag_cloud(&db(), &selected, "", 60);
    assert!(tags.iter().any(|entry| entry.tag == "green" && entry.count == 2));
    assert!(!tags.iter().any(|entry| entry.tag == "city"));
    assert!(tags.iter().find(|entry| entry.tag == "nature").unwrap().selected);
    assert_eq!(tags[0].tag, "nature");
}

#[test]
fn exclusion_cooccurrence() {
    let selected = vec!["-green".to_string()];
    let (tags, _) = recompute_tag_cloud(&db(), &selected, "", 60);
    assert!(tags.iter().any(|entry| entry.tag == "ocean"));
    assert!(!tags.iter().any(|entry| entry.tag == "forest"));
}

#[test]
fn query_suggests() {
    let (tags, suggest) = recompute_tag_cloud(&db(), &[], "natur", 60);
    assert!(tags.iter().all(|entry| entry.tag == "nature"));
    assert_eq!(suggest, "nature");
}

fn vocab() -> std::collections::HashSet<String> {
    ["forest", "green", "ocean", "nature"].iter().map(std::string::ToString::to_string).collect()
}

#[test]
fn locks_completed_words() {
    let vocab = vocab();
    let (locked, partial) = parse_search("forest gr", &vocab);
    assert_eq!(locked, vec!["forest"]);
    assert_eq!(partial, "gr");
}

#[test]
fn trailing_space_locks() {
    let vocab = vocab();
    let (locked, partial) = parse_search("forest green ", &vocab);
    assert_eq!(locked, vec!["forest", "green"]);
    assert_eq!(partial, "");
}

#[test]
fn last_word_partial() {
    let vocab = vocab();
    let (locked, partial) = parse_search("forest green", &vocab);
    assert_eq!(locked, vec!["forest", "green"]);
    assert_eq!(partial, "");
}

#[test]
fn negation_locks() {
    let vocab = vocab();
    let (locked, partial) = parse_search("-ocean ", &vocab);
    assert_eq!(locked, vec!["-ocean"]);
    assert_eq!(partial, "");
}

#[test]
fn free_form_extracts_tags() {
    let vocab = ["anime", "woman", "sword", "rainy", "city", "street"]
        .into_iter()
        .map(str::to_string)
        .collect();

    assert_eq!(
        parse_search("show me an anime woman with a sword", &vocab),
        (vec!["anime".into(), "woman".into(), "sword".into()], String::new())
    );
    assert_eq!(
        parse_search("I want a rainy city street please", &vocab),
        (vec!["rainy".into(), "city".into(), "street".into()], String::new())
    );
}

#[test]
fn taxonomy_alias_resolution() {
    let vocab =
        ["woman", "art", "ocean", "foggy", "animal"].into_iter().map(str::to_string).collect();

    assert_eq!(
        parse_search("women drawing animals by the sea in mist", &vocab),
        (
            vec!["woman".into(), "art".into(), "animal".into(), "ocean".into(), "foggy".into()],
            String::new()
        )
    );
}

#[test]
fn natural_exclusions_win() {
    let vocab = ["forest", "people", "animal", "cat"].into_iter().map(str::to_string).collect();

    assert_eq!(
        parse_search("a forest with no people", &vocab),
        (vec!["forest".into(), "-people".into()], String::new())
    );
    assert_eq!(
        parse_search("a cute animal, but not a cat", &vocab),
        (vec!["animal".into(), "-cat".into()], String::new())
    );
    assert_eq!(parse_search("cat but without cats", &vocab), (vec!["-cat".into()], String::new()));
}

#[test]
fn semantic_query_split() {
    let vocab =
        ["forest", "people", "woman", "anime", "cat"].into_iter().map(str::to_string).collect();

    assert_eq!(
        semantic_query("a forest without people", &vocab),
        SemanticQuery {
            positive: String::from("a forest"),
            negative: Some(String::from("people")),
            exclusions: vec![String::from("people")],
        }
    );
    assert_eq!(
        semantic_query("a woman who is not anime", &vocab),
        SemanticQuery {
            positive: String::from("a woman"),
            negative: Some(String::from("anime")),
            exclusions: vec![String::from("anime")],
        }
    );
    assert_eq!(
        semantic_query("cozy forest -cat", &vocab),
        SemanticQuery {
            positive: String::from("cozy forest"),
            negative: Some(String::from("cat")),
            exclusions: vec![String::from("cat")],
        }
    );
    assert_eq!(
        semantic_query("a forest without bicycles", &vocab),
        SemanticQuery {
            positive: String::from("a forest"),
            negative: Some(String::from("bicycles")),
            exclusions: Vec::new(),
        }
    );
}

#[test]
fn multi_word_alias_greedy() {
    let vocab =
        ["monochrome", "aerial", "sci-fi", "aurora"].into_iter().map(str::to_string).collect();

    assert_eq!(
        parse_search("black and white sci fi from above under northern lights", &vocab),
        (
            vec!["monochrome".into(), "sci-fi".into(), "aerial".into(), "aurora".into()],
            String::new()
        )
    );
}

#[test]
fn unmatched_sentence_keeps_cloud() {
    let vocab = vocab();
    assert_eq!(
        parse_search("something completely unknown please", &vocab),
        (Vec::new(), String::new())
    );
    assert_eq!(parse_search("forest gr", &vocab), (vec!["forest".into()], String::from("gr")));
}

fn xs(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn fuzz_string(seed: &mut u64) -> String {
    const POOL: &[&str] = &[
        "a", "Z", "0", "-", " ", "\t", "\u{0}", "\u{7f}", "\u{fe0f}", "é", "ß", "İ", "桜", "🌊",
        "\u{0301}", "\u{202e}", "\"", "\\", "\n", "ocean", "-ocean", "--", "ﬁ", "ẞ",
    ];
    let len = (xs(seed) % 20) as usize;
    (0..len).map(|_| POOL[(xs(seed) % POOL.len() as u64) as usize]).collect()
}

#[test]
fn unicode_fuzz() {
    let vocab = vocab();
    let mut seed = 0x5eed_cafe_f00d_0001u64;
    for _ in 0..8000 {
        let first = fuzz_string(&mut seed);
        let second = fuzz_string(&mut seed);
        let _ = word_stem(&first);
        let _ = edit_distance(&first, &second);
        let _ = fuzzy_match(&first, &second);
        let (locked, partial) = parse_search(&first, &vocab);
        assert!(
            locked.len() <= first.len() + 1 && partial.len() <= first.len() + 8,
            "input {first:?}"
        );
    }
}
