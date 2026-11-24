use gpui::Pixels;
#[cfg(any(feature = "inputs", feature = "feedback"))]
use gpui::{Bounds, Div, Hsla, Point, Styled, div, point, px};
#[cfg(any(feature = "inputs", feature = "feedback"))]
use gpui_material_foundation::{OverlayMotion, color, elevation_shadows, resolve_edges};
use gpui_material_theme::{ColorRole, CornerShape, Elevation, LogicalEdges};
#[cfg(any(feature = "inputs", feature = "feedback"))]
use gpui_material_theme::{CornerRole, Direction, MaterialTheme};

#[cfg(any(feature = "inputs", feature = "feedback"))]
pub(crate) fn menu_pivot(menu: Bounds<Pixels>, anchor: Bounds<Pixels>) -> Point<f32> {
    fn axis(start: Pixels, end: Pixels, anchor_start: Pixels, anchor_end: Pixels) -> f32 {
        if start >= anchor_end || start == end {
            0.
        } else if end <= anchor_start {
            1.
        } else {
            ((start.max(anchor_start) + end.min(anchor_end)) / 2. - start) / (end - start)
        }
    }
    point(
        axis(menu.left(), menu.right(), anchor.left(), anchor.right()),
        axis(menu.top(), menu.bottom(), anchor.top(), anchor.bottom()),
    )
}

/// Semantic color presets for Material vertical menus.
#[cfg(feature = "feedback")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MenuColor {
    /// Neutral surface container palette (`SurfaceContainer` / `SurfaceContainerLow`).
    #[default]
    Standard,
    /// Expressive tertiary container palette (`TertiaryContainer` / `OnTertiaryContainer`).
    Vibrant,
}

/// Per-surface menu styling. Omitted values use defaults appropriate to the surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct MenuSurfaceStyle {
    /// The surface shape in logical corner order.
    pub shape: Option<CornerShape>,
    /// Theme color role for the menu container background.
    pub container: Option<ColorRole>,
    /// The logical thickness and Material role of the outline.
    pub border: Option<(Pixels, ColorRole)>,
    /// Tonal elevation applied when the container role is `ColorRole::Surface`.
    pub tonal_elevation: Elevation,
    /// Drop-shadow elevation level. Defaults to `Elevation::Level2` for menus and `Elevation::Level0` for groups.
    pub shadow_elevation: Option<Elevation>,
    /// Inner logical padding around menu items. Defaults to `8 dp` vertical for menus and `4 dp` vertical for groups.
    pub padding: Option<LogicalEdges>,
    /// Vertical gap between child items or groups. Defaults to `0 px` for menus and `2 px` for groups.
    pub spacing: Option<Pixels>,
}

#[cfg(feature = "inputs")]
impl MenuSurfaceStyle {
    pub(crate) fn dropdown_surface(self, theme: &MaterialTheme, motion: OverlayMotion) -> Div {
        let palette = Palette {
            container: self.container.unwrap_or(ColorRole::SurfaceContainer),
            ink: ColorRole::OnSurface,
            #[cfg(feature = "feedback")]
            muted: ColorRole::OnSurfaceVariant,
            #[cfg(feature = "feedback")]
            selected: ColorRole::SecondaryContainer,
            #[cfg(feature = "feedback")]
            selected_ink: ColorRole::OnSecondaryContainer,
        };
        Paint {
            theme,
            motion,
            direction: theme.layout.direction,
        }
        .surface(self, palette, false)
    }

    pub(crate) fn popup_vertical_insets(self) -> Pixels {
        let padding = self
            .padding
            .unwrap_or(LogicalEdges::symmetric(0., 8.))
            .sanitize();
        let length = |value: Pixels| {
            if f32::from(value).is_finite() {
                value.max(px(0.))
            } else {
                px(0.)
            }
        };
        px(padding.top + padding.bottom)
            + self.border.map_or(px(0.), |(width, _)| length(width) * 2.)
    }
}

/// Per-row overrides. Unset shapes and colors follow the menu palette and layout.
#[cfg(feature = "feedback")]
#[derive(Clone, Copy, Debug, Default)]
pub struct MenuItemStyle {
    /// Logical corner shape for unselected menu rows.
    pub shape: Option<CornerShape>,
    /// Logical corner shape for selected menu rows.
    pub selected_shape: Option<CornerShape>,
    /// Inner logical padding of the menu row.
    pub padding: Option<LogicalEdges>,
    /// Horizontal gap between leading, label, and trailing slots.
    pub spacing: Option<Pixels>,
    /// Background color role for unselected rows.
    pub container: Option<ColorRole>,
    /// Primary label and icon color role for unselected rows.
    pub content: Option<ColorRole>,
    /// Supporting text and trailing metadata color role.
    pub supporting: Option<ColorRole>,
    /// Background color role for selected rows.
    pub selected_container: Option<ColorRole>,
    /// Primary label and icon color role for selected rows.
    pub selected_content: Option<ColorRole>,
}

#[cfg(any(feature = "inputs", feature = "feedback"))]
#[derive(Clone, Copy)]
pub(crate) struct Palette {
    pub(crate) container: ColorRole,
    pub(crate) ink: ColorRole,
    #[cfg(feature = "feedback")]
    pub(crate) muted: ColorRole,
    #[cfg(feature = "feedback")]
    pub(crate) selected: ColorRole,
    #[cfg(feature = "feedback")]
    pub(crate) selected_ink: ColorRole,
}

#[cfg(feature = "feedback")]
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

    pub(crate) fn padding<E: Styled>(self, element: E, padding: LogicalEdges) -> E {
        let edges = resolve_edges(padding, self.direction);
        element
            .pt(self.length(edges.top))
            .pr(self.length(edges.right))
            .pb(self.length(edges.bottom))
            .pl(self.length(edges.left))
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
                .unwrap_or_else(|| LogicalEdges::symmetric(0., if group { 4. } else { 8. })),
        )
    }
}
