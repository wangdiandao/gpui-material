use std::{rc::Rc, sync::LazyLock};

use gpui::{App, Global};
use gpui_material_theme::MaterialStrings;

struct ActiveMaterialStrings(Rc<MaterialStrings>);

impl Global for ActiveMaterialStrings {}

static DEFAULT_STRINGS: LazyLock<MaterialStrings> = LazyLock::new(MaterialStrings::default);

pub(crate) fn init_default_strings(cx: &mut App) {
    if !cx.has_global::<ActiveMaterialStrings>() {
        cx.set_global(ActiveMaterialStrings(Rc::new(MaterialStrings::default())));
    }
}

pub(crate) fn app_strings_handle(cx: &App) -> Rc<MaterialStrings> {
    cx.try_global::<ActiveMaterialStrings>()
        .map(|active| active.0.clone())
        .unwrap_or_else(|| Rc::new(MaterialStrings::default()))
}

/// Returns the application-wide [`MaterialStrings`] table, or [`MaterialStrings::default`]
/// if no custom strings have been installed yet.
pub fn material_strings(cx: &App) -> &MaterialStrings {
    cx.try_global::<ActiveMaterialStrings>()
        .map(|active| active.0.as_ref())
        .unwrap_or(&DEFAULT_STRINGS)
}

/// Replaces the application-wide [`MaterialStrings`] table and refreshes open windows
/// when the values change.
///
/// Subtree overrides installed with [`crate::ThemeScope::strings`] take precedence over
/// the application global, and explicit per-component label setters always take precedence
/// over both.
pub fn set_material_strings(strings: MaterialStrings, cx: &mut App) {
    if cx
        .try_global::<ActiveMaterialStrings>()
        .is_some_and(|previous| *previous.0 == strings)
    {
        return;
    }
    cx.set_global(ActiveMaterialStrings(Rc::new(strings)));
    cx.refresh_windows();
}
