use gpui::{
    AnyElement, App, Hsla, IntoElement, ParentElement, RenderOnce, Rgba, Styled, Window, div,
};
use gpui_material_theme::{Argb, ColorRole, Elevation, MaterialTheme};

use crate::window_material_theme;

/// Converts Material's packed sRGB color and preserves its alpha channel.
pub fn color(argb: Argb) -> Hsla {
    let [a, r, g, b] = argb.channels().map(|channel| f32::from(channel) / 255.);
    Rgba { r, g, b, a }.into()
}

/// Resolves the Material scrim color for an overlay that owns dismissal and focus.
pub fn scrim_color(theme: &MaterialTheme, opacity: f32) -> Hsla {
    color(theme.colors.color(ColorRole::Scrim)).opacity(opacity.clamp(0., 1.))
}

/// Desktop shadow projection shared by surfaces and custom-painted controls.
/// `elevation` is in Material logical units; `scale` follows the component geometry.
/// Compose delegates shadows to its platform renderer; GPUI uses these two layers.
pub fn elevation_shadows(elevation: f32, scale: f32, shadow: Hsla) -> Vec<gpui::BoxShadow> {
    let dp = elevation * scale;
    if !dp.is_finite() || dp <= 0. || !scale.is_finite() || scale <= 0. {
        return Vec::new();
    }
    vec![
        gpui_base::box_shadow(0., dp * 0.4, dp, 0., shadow.opacity(0.18)),
        gpui_base::box_shadow(0., scale, dp * 0.5, 0., shadow.opacity(0.13)),
    ]
}

/// A Material surface with semantic foreground, tonal elevation, and optional shadow.
#[derive(IntoElement)]
pub struct Surface {
    element: gpui::Div,
    role: ColorRole,
    tonal_elevation: Elevation,
    shadow_elevation: Elevation,
    shadow_scale: f32,
    theme: Option<MaterialTheme>,
}

impl Surface {
    pub fn new(role: ColorRole) -> Self {
        Self {
            element: div(),
            role,
            tonal_elevation: Elevation::Level0,
            shadow_elevation: Elevation::Level0,
            shadow_scale: 1.,
            theme: None,
        }
    }

    pub fn tonal_elevation(mut self, elevation: Elevation) -> Self {
        self.tonal_elevation = elevation;
        self
    }

    pub fn shadow_elevation(mut self, elevation: Elevation) -> Self {
        self.shadow_elevation = elevation;
        self
    }

    /// Scales the offset and blur radius with an animated surface.
    pub fn shadow_scale(mut self, scale: f32) -> Self {
        self.shadow_scale = scale;
        self
    }

    /// Uses an explicit window or subtree theme for this surface.
    pub fn with_theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl ParentElement for Surface {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.element.extend(elements);
    }
}

impl Styled for Surface {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.element.style()
    }
}

impl RenderOnce for Surface {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self
            .theme
            .as_ref()
            .unwrap_or_else(|| window_material_theme(window, cx));
        let scheme = &theme.colors;
        let background = if self.role == ColorRole::Surface {
            scheme.surface_at_elevation(self.tonal_elevation)
        } else {
            scheme.color(self.role)
        };
        let mut element = self.element.bg(color(background));
        if let Some(pair) = scheme.pair(self.role) {
            element = element.text_color(color(pair.foreground));
        }
        let shadows = elevation_shadows(
            self.shadow_elevation.dp(),
            self.shadow_scale,
            color(scheme.color(ColorRole::Shadow)),
        );
        if !shadows.is_empty() {
            element = element.shadow(shadows);
        }
        element
    }
}
