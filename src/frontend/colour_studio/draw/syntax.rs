#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syntax {
    Plain,
    Keyword,
    Function,
    Text,
    Number,
    Kind,
    Comment,
    Punctuation,
    Selection,
}

const KEYWORDS: [&str; 4] = ["pub", "fn", "let", "if"];
const PRIMITIVES: [&str; 1] = ["bool"];
const COMMENT: &str = "//";
const QUOTE: char = '"';
const CALL: char = '(';
const FIELD: char = ':';
const DECIMAL: char = '.';
const JOINER: char = '_';

pub fn marked<'a>(line: &'a str, selected: Option<&'a str>) -> Vec<(&'a str, Syntax)> {
    let Some((before, after)) = selected.and_then(|picked| line.split_once(picked)) else {
        return spans(line);
    };
    let mut out = spans(before);
    out.extend(selected.map(|picked| (picked, Syntax::Selection)));
    out.extend(spans(after));
    out
}

pub fn spans(line: &str) -> Vec<(&str, Syntax)> {
    if line.trim_start().starts_with(COMMENT) {
        return vec![(line, Syntax::Comment)];
    }
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(first) = rest.chars().next() {
        let (length, syntax) = token(rest, first);
        out.push((&rest[..length], syntax));
        rest = &rest[length..];
    }
    out
}

fn token(rest: &str, first: char) -> (usize, Syntax) {
    if first.is_whitespace() {
        return (run(rest, char::is_whitespace), Syntax::Plain);
    }
    if first == QUOTE {
        let mark = QUOTE.len_utf8();
        let close = rest[mark..].find(QUOTE).map_or(rest.len(), |at| mark + at + mark);
        return (close, Syntax::Text);
    }
    if first.is_ascii_digit() {
        return (run(rest, |next| next.is_ascii_digit() || next == DECIMAL), Syntax::Number);
    }
    if first.is_alphabetic() || first == JOINER {
        let length = run(rest, |next| next.is_alphanumeric() || next == JOINER);
        return (length, word(&rest[..length], rest[length..].chars().next()));
    }
    (first.len_utf8(), Syntax::Punctuation)
}

fn run(text: &str, keep: impl Fn(char) -> bool) -> usize {
    text.find(|next: char| !keep(next)).unwrap_or(text.len())
}

fn word(word: &str, next: Option<char>) -> Syntax {
    if KEYWORDS.contains(&word) {
        Syntax::Keyword
    } else if PRIMITIVES.contains(&word) || word.starts_with(char::is_uppercase) {
        Syntax::Kind
    } else if next == Some(CALL) {
        Syntax::Function
    } else if next == Some(FIELD) {
        Syntax::Punctuation
    } else {
        Syntax::Plain
    }
}

#[cfg(test)]
#[path = "syntax_tests.rs"]
mod tests;
