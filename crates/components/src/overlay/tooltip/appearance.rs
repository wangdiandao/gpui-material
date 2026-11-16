use gpui::{
    Bounds, Div, Hsla, Path, PathBuilder, Pixels, Point, Size, Styled, Window, point, px, size,
};
use gpui_base::Placement;
use gpui_material_foundation::{color, elevation_shadows};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Direction, Elevation, MaterialTheme,
};

use crate::{button_style::finite_length, motion::OverlayMotion};

/// Preferred tooltip side. Start and End resolve against the window's direction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TooltipPlacement {
    #[default]
    Above,
    Below,
    Left,
    Right,
    Start,
    End,
}

impl TooltipPlacement {
    pub(super) fn physical(self, direction: Direction) -> Placement {
        match (self, direction) {
            (Self::Above, _) => Placement::Top,
            (Self::Below, _) => Placement::Bottom,
            (Self::Left, _) | (Self::Start, Direction::Ltr) | (Self::End, Direction::Rtl) => {
                Placement::Left
            }
            (Self::Right, _) | (Self::End, Direction::Ltr) | (Self::Start, Direction::Rtl) => {
                Placement::Right
            }
        }
    }
}

/// Window-theme color overrides for a tooltip's individual slots.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TooltipColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub title_content: Option<Hsla>,
    pub action_content: Option<Hsla>,
}

type CaretPath = Box<dyn Fn(Bounds<Pixels>, TooltipPlacement) -> Option<Path<Pixels>>>;

/// Tooltip caret geometry. The default triangle is 16 px wide and 8 px deep.
pub struct TooltipCaret {
    size: Size<Pixels>,
    path: Option<CaretPath>,
}

impl Default for TooltipCaret {
    fn default() -> Self {
        Self::new(size(px(16.), px(8.)))
    }
}

impl TooltipCaret {
    /// Width along the tooltip edge and depth toward the trigger.
    pub fn new(size: Size<Pixels>) -> Self {
        Self {
            size: size.map(finite_length),
            path: None,
        }
    }

    /// Supplies a path in window coordinates. The side is the tooltip's resolved
    /// physical side relative to the trigger.
    pub fn path(
        mut self,
        path: impl Fn(Bounds<Pixels>, TooltipPlacement) -> Option<Path<Pixels>> + 'static,
    ) -> Self {
        self.path = Some(Box::new(path));
        self
    }

    pub(super) fn depth(&self, viewport: Size<Pixels>) -> Pixels {
        self.size
            .height
            .min(viewport.width.min(viewport.height) * 0.125)
    }

    pub(super) fn paint(
        &self,
        surface: Bounds<Pixels>,
        trigger: Bounds<Pixels>,
        side: Option<Placement>,
        style: SurfaceStyle,
        window: &mut Window,
    ) {
        let Some(side) = side.or_else(|| infer_side(surface, trigger)) else {
            return;
        };
        let [tl, tr, br, bl] = style
            .shape
            .radii(
                (surface.size.width / style.motion.scale).into(),
                (surface.size.height / style.motion.scale).into(),
                style.direction,
            )
            .map(|radius| style.motion.px(radius));
        let anchor = trigger.center();
        let (lower, upper, center) = match side {
            Placement::Top => (surface.left() + bl, surface.right() - br, anchor.x),
            Placement::Bottom => (surface.left() + tl, surface.right() - tr, anchor.x),
            Placement::Left => (surface.top() + tr, surface.bottom() - br, anchor.y),
            Placement::Right => (surface.top() + tl, surface.bottom() - bl, anchor.y),
        };
        let span = (self.size.width * style.motion.scale).min(upper - lower);
        let depth = self.depth(window.viewport_size()) * style.motion.scale;
        if span <= px(0.) || depth <= px(0.) {
            return;
        }
        let half = span * 0.5;
        let center = center.max(lower + half).min(upper - half);
        let (origin, dimensions, base_start, tip, base_end, side) = match side {
            Placement::Top => (
                point(center - half, surface.bottom()),
                size(span, depth),
                point(center - half, surface.bottom()),
                point(center, surface.bottom() + depth),
                point(center + half, surface.bottom()),
                TooltipPlacement::Above,
            ),
            Placement::Bottom => (
                point(center - half, surface.top() - depth),
                size(span, depth),
                point(center - half, surface.top()),
                point(center, surface.top() - depth),
                point(center + half, surface.top()),
                TooltipPlacement::Below,
            ),
            Placement::Left => (
                point(surface.right(), center - half),
                size(depth, span),
                point(surface.right(), center - half),
                point(surface.right() + depth, center),
                point(surface.right(), center + half),
                TooltipPlacement::Left,
            ),
            Placement::Right => (
                point(surface.left() - depth, center - half),
                size(depth, span),
                point(surface.left(), center - half),
                point(surface.left() - depth, center),
                point(surface.left(), center + half),
                TooltipPlacement::Right,
            ),
        };
        let path = self.path.as_ref().map_or_else(
            || triangle(base_start, tip, base_end),
            |path| path(Bounds::new(origin, dimensions), side),
        );
        if let Some(path) = path {
            window.paint_path(path, style.container);
        }
    }
}

fn infer_side(surface: Bounds<Pixels>, trigger: Bounds<Pixels>) -> Option<Placement> {
    if surface.bottom() <= trigger.top() {
        Some(Placement::Top)
    } else if surface.top() >= trigger.bottom() {
        Some(Placement::Bottom)
    } else if surface.right() <= trigger.left() {
        Some(Placement::Left)
    } else if surface.left() >= trigger.right() {
        Some(Placement::Right)
    } else {
        None
    }
}

fn triangle(a: Point<Pixels>, tip: Point<Pixels>, b: Point<Pixels>) -> Option<Path<Pixels>> {
    let mut path = PathBuilder::fill();
    path.move_to(a);
    path.line_to(tip);
    path.line_to(b);
    path.close();
    path.build().ok()
}

#[derive(Default)]
pub(super) struct Appearance {
    pub(super) colors: TooltipColors,
    pub(super) shape: Option<CornerShape>,
    pub(super) max_width: Option<Pixels>,
    pub(super) tonal_elevation: Elevation,
    pub(super) shadow_elevation: Option<Elevation>,
    pub(super) caret: Option<TooltipCaret>,
}

#[derive(Clone, Copy)]
pub(super) struct SurfaceStyle {
    rich: bool,
    shape: CornerShape,
    direction: Direction,
    motion: OverlayMotion,
    container: Hsla,
    pub(super) content: Hsla,
    pub(super) title: Hsla,
    pub(super) action: Hsla,
    pub(super) action_override: Option<Hsla>,
    width: Pixels,
    height: Pixels,
    shadow: Hsla,
    elevation: Elevation,
}

impl Appearance {
    pub(super) fn margin(&self, viewport: Size<Pixels>) -> Pixels {
        let depth = self
            .caret
            .as_ref()
            .map_or(px(0.), |caret| caret.depth(viewport));
        (px(8.) + depth).min(viewport.width.min(viewport.height) * 0.25)
    }

    pub(super) fn resolve(
        &self,
        rich: bool,
        viewport: Size<Pixels>,
        motion: OverlayMotion,
        theme: &MaterialTheme,
    ) -> SurfaceStyle {
        let tint = |role| color(theme.colors.color(role));
        let default_container = if rich {
            ColorRole::SurfaceContainer
        } else {
            ColorRole::InverseSurface
        };
        let default_content = if rich {
            ColorRole::OnSurfaceVariant
        } else {
            ColorRole::InverseOnSurface
        };
        let corner = if rich {
            CornerRole::Medium
        } else {
            CornerRole::ExtraSmall
        };
        let margin = self.margin(viewport);
        let width = self
            .max_width
            .unwrap_or_else(|| viewport.width * 0.75)
            .min(viewport.width - margin * 2.)
            .min(self.max_width.unwrap_or(px(if rich { 320. } else { 200. })));
        SurfaceStyle {
            rich,
            shape: self.shape.unwrap_or_else(|| corner.shape()),
            direction: theme.layout.direction,
            motion,
            container: self.colors.container.unwrap_or_else(|| {
                if self.tonal_elevation == Elevation::Level0 {
                    tint(default_container)
                } else {
                    color(theme.colors.surface_at_elevation(self.tonal_elevation))
                }
            }),
            content: self.colors.content.unwrap_or_else(|| tint(default_content)),
            title: self
                .colors
                .title_content
                .unwrap_or_else(|| tint(default_content)),
            action: self
                .colors
                .action_content
                .unwrap_or_else(|| tint(ColorRole::Primary)),
            action_override: self.colors.action_content,
            width,
            height: viewport.height - margin * 2.,
            shadow: tint(ColorRole::Shadow),
            elevation: self.shadow_elevation.unwrap_or(if rich {
                Elevation::Level2
            } else {
                Elevation::Level0
            }),
        }
    }
}

impl SurfaceStyle {
    pub(super) fn apply(self, div: Div) -> Div {
        let scale = self.motion.scale;
        let surface = div
            .flex()
            .flex_col()
            .bg(self.container)
            .text_color(self.content)
            .min_w(self.width.min(px(40.)) * scale)
            .min_h(self.height.min(px(24.)) * scale)
            .max_w(self.width * scale)
            .max_h(self.height * scale)
            .overflow_hidden()
            .shadow(elevation_shadows(self.elevation.dp(), scale, self.shadow));
        let surface = if self.shape == CornerShape::Full {
            surface.rounded_full()
        } else {
            let [tl, tr, br, bl] = self.shape.radii(f32::MAX, f32::MAX, self.direction);
            surface
                .rounded_tl(self.motion.px(tl))
                .rounded_tr(self.motion.px(tr))
                .rounded_br(self.motion.px(br))
                .rounded_bl(self.motion.px(bl))
        };
        if self.rich {
            surface.gap(self.motion.px(8.)).p(self.motion.px(16.))
        } else {
            surface
                .px(self.motion.px(8.))
                .py(self.motion.px(4.))
                .justify_center()
        }
    }
}
