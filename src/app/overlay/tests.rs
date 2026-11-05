#![cfg(test)]

use super::Overlay;

#[test]
fn esc_order_permutes_all() {
    assert_eq!(Overlay::ESC_ORDER.len(), Overlay::ALL.len());
    for overlay in Overlay::ALL {
        match overlay {
            Overlay::Help
            | Overlay::CardPicker
            | Overlay::AudioPanel
            | Overlay::Playlists
            | Overlay::TagMode
            | Overlay::TagEditing
            | Overlay::Detail
            | Overlay::BrowserPreview
            | Overlay::Browser
            | Overlay::ThemeDesigner
            | Overlay::ColourStudio
            | Overlay::ScheduleEditor
            | Overlay::SceneProperties
            | Overlay::Effects
            | Overlay::BackendsMenu
            | Overlay::ThemeAudition
            | Overlay::ThemeBar
            | Overlay::TagCloud
            | Overlay::FoldersMenu
            | Overlay::Settings => {}
        }
        assert_eq!(
            Overlay::ALL.iter().filter(|&&other| other == overlay).count(),
            1,
            "{overlay:?} ALL"
        );
        assert_eq!(
            Overlay::ESC_ORDER.iter().filter(|&&other| other == overlay).count(),
            1,
            "{overlay:?} ESC_ORDER"
        );
    }
}

#[test]
fn esc_order_nesting() {
    let rank =
        |overlay: Overlay| Overlay::ESC_ORDER.iter().position(|&item| item == overlay).unwrap();
    assert!(rank(Overlay::BrowserPreview) < rank(Overlay::Browser));
    assert!(rank(Overlay::TagEditing) < rank(Overlay::Detail));
    assert!(rank(Overlay::ColourStudio) < rank(Overlay::ThemeBar));
}

#[test]
fn obscure_policy_set() {
    const OBSCURING: [Overlay; 11] = [
        Overlay::Help,
        Overlay::CardPicker,
        Overlay::AudioPanel,
        Overlay::Playlists,
        Overlay::Browser,
        Overlay::ThemeDesigner,
        Overlay::ColourStudio,
        Overlay::ScheduleEditor,
        Overlay::SceneProperties,
        Overlay::Effects,
        Overlay::ThemeAudition,
    ];
    for overlay in Overlay::ALL {
        let policy = overlay.policy();
        assert_eq!(policy.obscures, OBSCURING.contains(&overlay), "{overlay:?}");
        if policy.obscures {
            assert!(policy.captures, "{overlay:?}");
        }
    }
    assert!(!Overlay::BrowserPreview.policy().captures);
    assert!(!Overlay::Detail.policy().captures);
    assert!(Overlay::Browser.policy().captures);
}
