use super::*;

#[test]
fn signature_inks_match() {
    let found = spans("pub fn derive(dominant: &[Rgb], dark: bool) -> ThemePalette {");
    let ink = |text: &str| found.iter().find(|(part, _)| *part == text).map(|(_, ink)| *ink);
    assert_eq!(ink("fn"), Some(Syntax::Keyword));
    assert_eq!(ink("derive"), Some(Syntax::Function));
    assert_eq!(ink("dominant"), Some(Syntax::Punctuation));
    assert_eq!(ink("Rgb"), Some(Syntax::Kind));
    assert_eq!(ink("bool"), Some(Syntax::Kind));
}

#[test]
fn selection_splits_line() {
    let found = marked("    let (hue, saturation) = pick(dominant);", Some("(hue, saturation)"));
    assert!(found.contains(&("(hue, saturation)", Syntax::Selection)));
    assert!(found.contains(&("dominant", Syntax::Plain)));
}

#[test]
fn strings_and_numbers() {
    let found = spans("log(\"dark palette\", 0.22);");
    assert!(found.contains(&("\"dark palette\"", Syntax::Text)));
    assert!(found.contains(&("0.22", Syntax::Number)));
}

#[test]
fn ragged_lines_stay_whole() {
    assert!(spans("").is_empty());
    assert_eq!(
        spans("log(\"open"),
        [("log", Syntax::Function), ("(", Syntax::Punctuation), ("\"open", Syntax::Text)]
    );
}
