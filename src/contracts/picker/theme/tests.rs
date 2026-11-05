#![cfg(test)]

use super::PaletteSpec;
use crate::domain::theme::ThemeRole;

#[test]
fn indexed_roles_stay_compatible() {
    let PaletteSpec {
        primary,
        primary_text,
        surface,
        surface_text,
        surface_variant,
        surface_container,
        background,
        outline,
        tertiary,
        ..
    } = PaletteSpec::default();
    let fields = [
        ("Primary", primary),
        ("PrimaryText", primary_text),
        ("Surface", surface),
        ("SurfaceText", surface_text),
        ("SurfaceVariant", surface_variant),
        ("SurfaceContainer", surface_container),
        ("Background", background),
        ("Outline", outline),
        ("Tertiary", tertiary),
    ];
    let mut field_roles: Vec<&str> = fields.iter().map(|(role, _)| *role).collect();
    field_roles.sort_unstable();
    let mut roles: Vec<String> = ThemeRole::ALL.iter().map(|role| format!("{role:?}")).collect();
    roles.sort_unstable();
    assert_eq!(field_roles, roles);
}
