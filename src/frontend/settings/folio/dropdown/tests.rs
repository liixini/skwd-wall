use super::*;

#[test]
fn duplicate_labels_keep_keys() {
    let choices = vec![
        Choice { key: "first".into(), label: "Same".into() },
        Choice { key: "second".into(), label: "Same".into() },
    ];
    assert_eq!(selection(&choices, "second"), Some(choices[1].clone()));
    let Message::Settings(SettingsMsg::Pick(path, key)) =
        picked("picker.layout", choices[1].clone())
    else {
        panic!("choice message");
    };
    assert_eq!(path, "picker.layout");
    assert_eq!(key, "second");
    assert_eq!(choices[1].to_string(), "Same");
}

#[test]
fn missing_selection() {
    let choices = vec![Choice { key: "wave".into(), label: "Wave".into() }];
    assert_eq!(selection(&choices, "missing"), None);
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].key, "wave");
}
