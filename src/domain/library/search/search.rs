use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use super::numeric::{NumericQuery, parse_numeric_query, strip_numeric_query};

include!(concat!(env!("OUT_DIR"), "/wallpaper_taxonomy_aliases.rs"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticQuery {
    pub positive: String,
    pub negative: Option<String>,
    pub exclusions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryQuery {
    pub tags: Vec<String>,
    pub partial: String,
    pub numeric: NumericQuery,
}

pub fn parse_library_search(text: &str, vocab: &HashSet<String>) -> LibraryQuery {
    let (numeric_tags, tokens): (Vec<_>, Vec<_>) = text.split_whitespace().partition(|token| {
        let bare = token.trim_start_matches('-');
        super::numeric::workshop_id(bare).is_some() && vocab.contains(bare)
    });
    let numeric_text = tokens.join(" ");
    let numeric = parse_numeric_query(&numeric_text);
    let tag_text = strip_numeric_query(text, &numeric_tags);
    let (tags, partial) = parse_search(&tag_text, vocab);
    LibraryQuery { tags, partial, numeric }
}

pub fn word_stem(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    if len < 3 {
        return chars.iter().collect();
    }
    if let Some(stem) = stem_noun(word, &chars, len) {
        return stem;
    }
    if let Some(stem) = stem_verb(word, &chars, len) {
        return stem;
    }
    stem_tail(word, &chars, len)
}

fn stem_noun(word: &str, chars: &[char], len: usize) -> Option<String> {
    let take = |end: usize| -> String { chars[..end].iter().collect() };
    if word.ends_with("ies") && len > 4 {
        return Some(format!("{}y", take(len - 3)));
    }
    if word.ends_with("ves") && len > 4 {
        return Some(format!("{}f", take(len - 3)));
    }
    if word.ends_with("ses")
        || word.ends_with("xes")
        || word.ends_with("zes")
        || word.ends_with("ches")
        || word.ends_with("shes")
    {
        return Some(take(len - 2));
    }
    if word.ends_with("ness") && len > 5 {
        return Some(take(len - 4));
    }
    if word.ends_with("ment") && len > 5 {
        return Some(take(len - 4));
    }
    None
}

fn stem_verb(word: &str, chars: &[char], len: usize) -> Option<String> {
    if word.ends_with("ing") && len > 4 {
        return Some(undouble(&chars[..len - 3]));
    }
    if word.ends_with("ed") && len > 3 {
        return Some(undouble(&chars[..len - 2]));
    }
    None
}

fn undouble(base: &[char]) -> String {
    let len = base.len();
    if len > 1 && base[len - 1] == base[len - 2] {
        return base[..len - 1].iter().collect();
    }
    base.iter().collect()
}

fn stem_tail(word: &str, chars: &[char], len: usize) -> String {
    let take = |end: usize| -> String { chars[..end].iter().collect() };
    if word.ends_with("er") && len > 3 {
        return take(len - 2);
    }
    if word.ends_with("ly") && len > 3 {
        return take(len - 2);
    }
    if word.ends_with('s') && !word.ends_with("ss") && len > 3 {
        return take(len - 1);
    }
    chars.iter().collect()
}

pub fn edit_distance(lhs: &str, rhs: &str) -> usize {
    if lhs == rhs {
        return 0;
    }
    let lhs: Vec<char> = lhs.chars().collect();
    let rhs: Vec<char> = rhs.chars().collect();
    let (llen, rlen) = (lhs.len(), rhs.len());
    if llen == 0 {
        return rlen;
    }
    if rlen == 0 {
        return llen;
    }
    let mut prev: Vec<usize> = (0..=rlen).collect();
    let mut curr: Vec<usize> = vec![0; rlen + 1];
    for row in 1..=llen {
        curr[0] = row;
        for col in 1..=rlen {
            curr[col] = if lhs[row - 1] == rhs[col - 1] {
                prev[col - 1]
            } else {
                1 + prev[col - 1].min(prev[col]).min(curr[col - 1])
            };
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[rlen]
}

pub fn fuzzy_match(tag: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    if tag.contains(query) {
        return true;
    }
    let tag_stem = word_stem(tag);
    let query_stem = word_stem(query);
    if tag_stem == query_stem || tag_stem.contains(&query_stem) || query_stem.contains(&tag_stem) {
        return true;
    }
    let max_dist =
        if tag_stem.chars().count().min(query_stem.chars().count()) <= 4 { 1 } else { 2 };
    edit_distance(&tag_stem, &query_stem) <= max_dist
}

pub fn parse_search(text: &str, vocab: &HashSet<String>) -> (Vec<String>, String) {
    let raw = text.to_lowercase();
    let words = query_words(&raw);
    let mut locked = Vec::new();
    let mut consumed = vec![false; words.len()];
    let mut exclude_next = false;
    let mut index = 0;
    while index < words.len() {
        let word = words[index].as_str();
        if NATURAL_EXCLUSIONS.contains(&word) {
            exclude_next = true;
            index += 1;
            continue;
        }
        if exclude_next && EXCLUSION_FILLERS.contains(&word) {
            index += 1;
            continue;
        }
        let (explicit_exclusion, first) = split_neg(word);
        let mut matched = None;
        for length in (1..=3.min(words.len() - index)).rev() {
            let phrase = std::iter::once(first)
                .chain(words[index + 1..index + length].iter().map(String::as_str))
                .collect::<Vec<_>>()
                .join("-");
            if let Some(tag) = resolve_tag(&phrase, vocab) {
                matched = Some((length, tag));
                break;
            }
        }
        if let Some((length, tag)) = matched {
            let excluded = explicit_exclusion || exclude_next;
            exclude_next = false;
            for used in &mut consumed[index..index + length] {
                *used = true;
            }
            push_selection(&mut locked, &tag, excluded);
            index += length;
        } else {
            index += 1;
        }
    }

    let partial = if !raw
        .ends_with(|character: char| !character.is_alphanumeric() && character != '-')
        && let Some((last_index, last)) = words.iter().enumerate().next_back()
        && !consumed[last_index]
    {
        let (excluded, bare) = split_neg(last);
        completion_prefix(bare, vocab).map(|prefix| join_neg(excluded, prefix)).unwrap_or_default()
    } else {
        String::new()
    };
    (locked, partial)
}

pub fn semantic_query(text: &str, vocab: &HashSet<String>) -> SemanticQuery {
    let (selected, _) = parse_search(text, vocab);
    let exclusions: Vec<String> =
        selected.iter().filter_map(|tag| tag.strip_prefix('-').map(str::to_string)).collect();
    let words = query_words(&text.to_lowercase());
    let marker = words.iter().position(|word| NATURAL_EXCLUSIONS.contains(&word.as_str()));
    let (positive_words, negative_words) = if let Some(marker) = marker {
        let mut prefix = words[..marker].to_vec();
        while prefix.last().is_some_and(|word| NEGATION_CONNECTORS.contains(&word.as_str())) {
            prefix.pop();
        }
        let suffix = words[marker + 1..]
            .iter()
            .skip_while(|word| EXCLUSION_FILLERS.contains(&word.as_str()))
            .cloned()
            .collect();
        (prefix, suffix)
    } else {
        let mut positive = Vec::new();
        let mut negative = Vec::new();
        for word in words {
            let (excluded, bare) = split_neg(&word);
            if excluded {
                negative.push(bare.to_string());
            } else {
                positive.push(word);
            }
        }
        (positive, negative)
    };
    let positive = match positive_words.join(" ").trim() {
        "" => String::from("wallpaper"),
        value => value.to_string(),
    };
    let negative = (!negative_words.is_empty()).then(|| negative_words.join(" "));
    SemanticQuery { positive, negative, exclusions }
}

fn query_words(text: &str) -> Vec<String> {
    text.split(|character: char| {
        !character.is_alphanumeric() && character != '-' && character != '_'
    })
    .map(|word| word.trim_matches('_').replace('_', "-"))
    .filter(|word| word.chars().any(char::is_alphanumeric))
    .collect()
}

fn resolve_tag(phrase: &str, vocab: &HashSet<String>) -> Option<String> {
    if vocab.contains(phrase) {
        return Some(phrase.to_string());
    }
    if let Some(tag) = taxonomy_alias(phrase)
        && vocab.contains(tag)
    {
        return Some(tag.to_string());
    }
    if phrase.contains('-') {
        return None;
    }
    let stem = word_stem(phrase);
    vocab.iter().filter(|tag| word_stem(tag) == stem).min().cloned()
}

fn completion_prefix<'a>(word: &'a str, vocab: &'a HashSet<String>) -> Option<&'a str> {
    if word.is_empty() || STOP_WORDS.contains(&word) || NATURAL_EXCLUSIONS.contains(&word) {
        return None;
    }
    if vocab.iter().any(|tag| tag.starts_with(word) && tag.len() > word.len()) {
        return Some(word);
    }
    TAXONOMY_ALIASES
        .iter()
        .filter(|(alias, tag)| {
            alias.starts_with(word) && alias.len() > word.len() && vocab.contains(*tag)
        })
        .map(|(_, tag)| *tag)
        .min()
}

fn push_selection(locked: &mut Vec<String>, tag: &str, excluded: bool) {
    locked.retain(|selected| selected != tag && selected.strip_prefix('-') != Some(tag));
    locked.push(join_neg(excluded, tag));
}

fn taxonomy_alias(phrase: &str) -> Option<&'static str> {
    TAXONOMY_ALIASES
        .binary_search_by_key(&phrase, |(alias, _)| alias)
        .ok()
        .map(|index| TAXONOMY_ALIASES[index].1)
}

const NATURAL_EXCLUSIONS: &[&str] = &["not", "no", "without", "excluding", "exclude", "except"];

const EXCLUSION_FILLERS: &[&str] = &["a", "an", "any", "the"];

const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "any", "for", "from", "i", "in", "me", "my", "of", "on", "please", "show",
    "the", "to", "want", "with",
];

const NEGATION_CONNECTORS: &[&str] = &["and", "but", "is", "that", "which", "who", "with"];

fn split_neg(word: &str) -> (bool, &str) {
    if word.len() > 1
        && let Some(bare) = word.strip_prefix('-')
    {
        (true, bare)
    } else {
        (false, word)
    }
}

fn join_neg(neg: bool, bare: &str) -> String {
    if neg { format!("-{bare}") } else { bare.to_string() }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagEntry {
    pub tag: String,
    pub count: usize,
    pub selected: bool,
    pub excluded: bool,
}

pub fn recompute_tag_cloud(
    tags_db: &HashMap<String, Vec<String>>,
    selected: &[String],
    query: &str,
    max_visible: usize,
) -> (Vec<TagEntry>, String) {
    let counts = count_tags(tags_db, selected);

    let query = query.trim();
    let mut result = build_entries(&counts, selected, query);
    result.sort_by(tag_order);
    result.truncate(max_visible + selected.len());

    let suggest = suggest_completion(&result, query);
    (result, suggest)
}

fn count_tags<'a>(
    tags_db: &'a HashMap<String, Vec<String>>,
    selected: &[String],
) -> HashMap<&'a str, usize> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    if selected.is_empty() {
        for wtags in tags_db.values() {
            for tag in wtags {
                *counts.entry(tag.as_str()).or_insert(0) += 1;
            }
        }
        return counts;
    }
    for wtags in tags_db.values() {
        if wtags.is_empty() {
            continue;
        }
        if !matches_all(wtags, selected) {
            continue;
        }
        for tag in wtags {
            *counts.entry(tag.as_str()).or_insert(0) += 1;
        }
    }
    counts
}

fn matches_all(wtags: &[String], selected: &[String]) -> bool {
    selected.iter().all(|sel| {
        if let Some(neg) = sel.strip_prefix('-').filter(|rest| !rest.is_empty()) {
            !wtags.iter().any(|tag| tag == neg)
        } else {
            wtags.iter().any(|tag| tag == sel)
        }
    })
}

fn build_entries(counts: &HashMap<&str, usize>, selected: &[String], query: &str) -> Vec<TagEntry> {
    let mut result: Vec<TagEntry> = Vec::new();
    for (&tag, &count) in counts {
        let is_selected = selected.iter().any(|sel| sel == tag);
        let is_excluded = selected.iter().any(|sel| sel.strip_prefix('-') == Some(tag));
        if query.is_empty() || fuzzy_match(tag, query) || is_selected || is_excluded {
            result.push(TagEntry {
                tag: tag.to_string(),
                count,
                selected: is_selected,
                excluded: is_excluded,
            });
        }
    }
    result
}

fn tag_order(lhs: &TagEntry, rhs: &TagEntry) -> Ordering {
    let lhs_active = lhs.selected || lhs.excluded;
    let rhs_active = rhs.selected || rhs.excluded;
    if lhs_active != rhs_active {
        return if lhs_active { Ordering::Less } else { Ordering::Greater };
    }
    rhs.count.cmp(&lhs.count).then_with(|| lhs.tag.cmp(&rhs.tag))
}

pub fn suggest_completion(entries: &[TagEntry], query: &str) -> String {
    if query.is_empty() {
        return String::new();
    }
    let (neg, bare) = split_neg(query);
    let mut best: i64 = -1;
    let mut out = String::new();
    for entry in entries {
        if !entry.selected
            && (entry.tag.starts_with(bare) || word_stem(&entry.tag) == word_stem(bare))
            && entry.count as i64 > best
        {
            out = join_neg(neg, &entry.tag);
            best = entry.count as i64;
        }
    }
    out
}
