#[cfg(any(feature = "inputs", feature = "feedback"))]
use gpui::{Div, Hsla, Styled, div, px};
use gpui::{Edges, Pixels};
#[cfg(any(feature = "inputs", feature = "feedback"))]
use gpui_material_foundation::{color, elevation_shadows};
use gpui_material_theme::{ColorRole, CornerShape, Elevation};
#[cfg(any(feature = "inputs", feature = "feedback"))]
use gpui_material_theme::{CornerRole, Direction, MaterialTheme};

#[cfg(any(feature = "inputs", feature = "feedback"))]
use crate::motion::OverlayMotion;

/// Semantic color presets for Material vertical menus.
#[cfg(any(feature = "inputs", feature = "feedback"))]
#[cfg_attr(not(feature = "feedback"), allow(dead_code))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MenuColor {
    #[default]
    Standard,
    Vibrant,
}

/// Per-surface menu styling. Omitted values use defaults appropriate to the surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct MenuSurfaceStyle {
    /// The surface shape in logical corner order.
    pub shape: Option<CornerShape>,
    pub container: Option<ColorRole>,
    /// The logical thickness and Material role of the outline.
    pub border: Option<(Pixels, ColorRole)>,
    pub tonal_elevation: Elevation,
    pub shadow_elevation: Option<Elevation>,
    pub padding: Option<Edges<Pixels>>,
    pub spacing: Option<Pixels>,
}

#[cfg(feature = "inputs")]
impl MenuSurfaceStyle {
    pub(crate) fn dropdown_surface(self, theme: &MaterialTheme, motion: OverlayMotion) -> Div {
        let palette = MenuColor::Standard.palette(false, self.container);
        Paint {
            theme,
            motion,
            direction: theme.layout.direction,
        }
        .surface(self, palette, false)
    }

    pub(crate) fn popup_vertical_insets(self) -> Pixels {
        let padding = self.padding.unwrap_or_else(|| axis_padding(px(0.), px(8.)));
        let length = |value: Pixels| {
            if f32::from(value).is_finite() {
                value.max(px(0.))
            } else {
                px(0.)
            }
        };
        length(padding.top)
            + length(padding.bottom)
            + self.border.map_or(px(0.), |(width, _)| length(width) * 2.)
    }
}

/// Per-row overrides. Unset shapes and colors follow the menu palette and layout.
#[cfg(feature = "feedback")]
#[derive(Clone, Copy, Debug, Default)]
pub struct MenuItemStyle {
    pub shape: Option<CornerShape>,
    pub selected_shape: Option<CornerShape>,
    pub padding: Option<Edges<Pixels>>,
    pub spacing: Option<Pixels>,
    pub container: Option<ColorRole>,
    pub content: Option<ColorRole>,
    pub supporting: Option<ColorRole>,
    pub selected_container: Option<ColorRole>,
    pub selected_content: Option<ColorRole>,
}

#[cfg(any(feature = "inputs", feature = "feedback"))]
#[derive(Clone, Copy)]
#[cfg_attr(not(feature = "feedback"), allow(dead_code))]
pub(crate) struct Palette {
    pub(crate) container: ColorRole,
    pub(crate) ink: ColorRole,
    pub(crate) muted: ColorRole,
    pub(crate) selected: ColorRole,
    pub(crate) selected_ink: ColorRole,
}

#[cfg(any(feature = "inputs", feature = "feedback"))]
impl MenuColor {
    pub(crate) fn palette(self, grouped: bool, container: Option<ColorRole>) -> Palette {
        use ColorRole::*;
        let (default_container, ink, muted, selected, selected_ink) = match (self, grouped) {
            (Self::Standard, false) => (
                SurfaceContainer,
                OnSurface,
                OnSurfaceVariant,
                SecondaryContainer,
                OnSecondaryContainer,
            ),
            (Self::Standard, true) => (
                SurfaceContainerLow,
                OnSurface,
                OnSurfaceVariant,
                TertiaryContainer,
                OnTertiaryContainer,
            ),
            (Self::Vibrant, _) => (
                TertiaryContainer,
                OnTertiaryContainer,
                OnTertiaryContainer,
                Tertiary,
                OnTertiary,
            ),
        };
        Palette {
            container: container.unwrap_or(default_container),
            ink,
            muted,
            selected,
            selected_ink,
        }
    }
}

#[cfg(any(feature = "inputs", feature = "feedback"))]
#[derive(Clone, Copy)]
pub(crate) struct Paint<'a> {
    pub(crate) theme: &'a MaterialTheme,
    pub(crate) motion: OverlayMotion,
    pub(crate) direction: Direction,
}

#[cfg(any(feature = "inputs", feature = "feedback"))]
impl Paint<'_> {
    pub(crate) fn color(self, role: ColorRole) -> Hsla {
        color(self.theme.colors.color(role))
    }

    pub(crate) fn padding<E: Styled>(self, element: E, padding: Edges<Pixels>) -> E {
        element
            .pt(self.length(padding.top))
            .pr(self.length(padding.right))
            .pb(self.length(padding.bottom))
            .pl(self.length(padding.left))
    }

    pub(crate) fn length(self, value: Pixels) -> Pixels {
        let value = f32::from(value);
        self.motion
            .px(if value.is_finite() { value.max(0.) } else { 0. })
    }

    pub(crate) fn shape<E: Styled>(self, element: E, shape: CornerShape) -> E {
        if shape == CornerShape::Full {
            return element.rounded_full();
        }
        let [tl, tr, br, bl] = shape.radii(f32::MAX, f32::MAX, self.direction);
        element
            .rounded_tl(self.motion.px(tl))
            .rounded_tr(self.motion.px(tr))
            .rounded_br(self.motion.px(br))
            .rounded_bl(self.motion.px(bl))
    }

    pub(crate) fn surface(self, style: MenuSurfaceStyle, palette: Palette, group: bool) -> Div {
        let background = if palette.container == ColorRole::Surface {
            color(
                self.theme
                    .colors
                    .surface_at_elevation(style.tonal_elevation),
            )
        } else {
            self.color(palette.container)
        };
        let shadow = style.shadow_elevation.unwrap_or(if group {
            Elevation::Level0
        } else {
            Elevation::Level2
        });
        let surface = div()
            .flex()
            .flex_col()
            .gap(self.length(style.spacing.unwrap_or(if group { px(2.) } else { px(0.) })))
            .bg(background)
            .text_color(self.color(palette.ink))
            .shadow(elevation_shadows(
                shadow.dp(),
                self.motion.scale,
                self.color(ColorRole::Shadow),
            ));
        let surface = if let Some((width, role)) = style.border {
            let mut surface = surface.border_color(self.color(role));
            let width = Some(self.length(width).into());
            let borders = &mut surface.style().border_widths;
            borders.top = width;
            borders.right = width;
            borders.bottom = width;
            borders.left = width;
            surface
        } else {
            surface
        };
        let surface = self.shape(
            surface,
            style.shape.unwrap_or_else(|| {
                if group {
                    CornerRole::Large.shape()
                } else {
                    CornerRole::ExtraSmall.shape()
                }
            }),
        );
        self.padding(
            surface,
            style
                .padding
                .unwrap_or_else(|| axis_padding(px(0.), if group { px(4.) } else { px(8.) })),
        )
    }
}

#[cfg(any(feature = "inputs", feature = "feedback"))]
pub(crate) fn axis_padding(x: Pixels, y: Pixels) -> Edges<Pixels> {
    Edges {
        top: y,
        right: x,
        bottom: y,
        left: x,
    }
}
