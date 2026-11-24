use gpui::{Edges, Hsla, Pixels, Rgba, px};
use gpui_material_theme::{Argb, ColorRole, ColorSpec, Direction, LogicalEdges, MaterialTheme};

/// Converts Material's packed sRGB color and preserves its alpha channel.
pub fn color(argb: Argb) -> Hsla {
    let [a, r, g, b] = argb.channels().map(|channel| f32::from(channel) / 255.);
    Rgba { r, g, b, a }.into()
}

/// Resolves a [`ColorSpec`] against `theme` and converts it to GPUI [`Hsla`].
pub fn resolve_color_spec(spec: ColorSpec, theme: &MaterialTheme) -> Hsla {
    color(spec.resolve(&theme.colors))
}

/// Resolves sanitized [`LogicalEdges`] into physical GPUI [`Edges<Pixels>`] for `direction`.
pub fn resolve_edges(edges: LogicalEdges, direction: Direction) -> Edges<Pixels> {
    let [top, right, bottom, left] = edges.resolve(direction);
    Edges {
        top: px(top),
        right: px(right),
        bottom: px(bottom),
        left: px(left),
    }
}

/// Resolves the Material scrim color for an overlay that owns dismissal and focus.
pub fn scrim_color(theme: &MaterialTheme, opacity: f32) -> Hsla {
    let clamped = if opacity.is_nan() {
        0.
    } else {
        opacity.clamp(0., 1.)
    };
    color(theme.colors.color(ColorRole::Scrim)).opacity(clamped)
}

/// Stack-allocated two-layer desktop shadow projection for custom canvas painters.
pub fn elevation_shadow_pair(
    elevation: f32,
    scale: f32,
    shadow: Hsla,
) -> Option<[gpui::BoxShadow; 2]> {
    let dp = elevation * scale;
    if !dp.is_finite() || dp <= 0. || !scale.is_finite() || scale <= 0. {
        return None;
    }
    Some([
        gpui_base::box_shadow(0., dp * 0.4, dp, 0., shadow.opacity(0.18)),
        gpui_base::box_shadow(0., scale, dp * 0.5, 0., shadow.opacity(0.13)),
    ])
}

/// Desktop shadow projection shared by surfaces and custom-painted controls.
/// `elevation` is in Material logical units; `scale` follows the component geometry.
/// Compose delegates shadows to its platform renderer; GPUI uses these two layers.
pub fn elevation_shadows(elevation: f32, scale: f32, shadow: Hsla) -> Vec<gpui::BoxShadow> {
    elevation_shadow_pair(elevation, scale, shadow).map_or_else(Vec::new, Vec::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_material_theme::{Appearance, ColorScheme};

    #[test]
    fn elevation_shadow_pair_rejects_non_positive_and_non_finite() {
        let shadow = color(Argb::from_rgb(0, 0, 0));
        for (elev, scale) in [
            (0., 1.),
            (-2., 1.),
            (3., 0.),
            (3., -1.),
            (f32::NAN, 1.),
            (3., f32::INFINITY),
        ] {
            assert!(elevation_shadow_pair(elev, scale, shadow).is_none());
            assert!(elevation_shadows(elev, scale, shadow).is_empty());
        }
        let pair = elevation_shadow_pair(6., 1., shadow).unwrap();
        assert_eq!(pair.len(), 2);
        assert_eq!(elevation_shadows(6., 1., shadow).len(), 2);
    }

    #[test]
    fn scrim_color_clamps_opacity_and_handles_nan() {
        let theme = MaterialTheme::baseline(Appearance::Light);
        assert_eq!(scrim_color(&theme, -0.5).a, 0.);
        assert_eq!(scrim_color(&theme, f32::NAN).a, 0.);
        assert!((scrim_color(&theme, 0.32).a - 0.32).abs() < 1e-4);
        assert_eq!(scrim_color(&theme, 2.0).a, 1.);
    }

    #[test]
    fn resolve_edges_and_color_spec_respect_direction_and_theme() {
        let edges = LogicalEdges::new(4., 8., 12., 16.);
        let ltr = resolve_edges(edges, Direction::Ltr);
        assert_eq!(
            (ltr.top, ltr.right, ltr.bottom, ltr.left),
            (px(4.), px(8.), px(12.), px(16.))
        );
        let rtl = resolve_edges(edges, Direction::Rtl);
        assert_eq!(
            (rtl.top, rtl.right, rtl.bottom, rtl.left),
            (px(4.), px(16.), px(12.), px(8.))
        );

        let theme = MaterialTheme::baseline(Appearance::Dark);
        let expected = color(ColorScheme::baseline(Appearance::Dark).color(ColorRole::Primary));
        assert_eq!(
            resolve_color_spec(ColorSpec::Role(ColorRole::Primary), &theme),
            expected
        );
        let custom = Argb::from_rgb(10, 20, 30);
        assert_eq!(
            resolve_color_spec(ColorSpec::Custom(custom), &theme),
            color(custom)
        );
    }
}
