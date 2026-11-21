use std::{cell::RefCell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, Bounds, ElementId, Entity, FocusHandle, Hsla, IntoElement,
    ParentElement, Pixels, RenderOnce, Rgba, Role, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Toggled, Window, canvas, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button, StyledExt};
use gpui_material_theme::{Argb, ColorRole, CornerShape, Direction, Elevation, MaterialTheme};

use crate::{
    InteractionFeedback, InteractionSource, Shape, ShapeFit, ShapePolygon,
    control_feedback::{activate_feedback, feedback_target, ripple_state},
    current_material_theme,
};

/// Converts Material's packed sRGB color and preserves its alpha channel.
pub fn color(argb: Argb) -> Hsla {
    let [a, r, g, b] = argb.channels().map(|channel| f32::from(channel) / 255.);
    Rgba { r, g, b, a }.into()
}

/// Resolves the Material scrim color for an overlay that owns dismissal and focus.
pub fn scrim_color(theme: &MaterialTheme, opacity: f32) -> Hsla {
    color(theme.colors.color(ColorRole::Scrim)).opacity(opacity.clamp(0., 1.))
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

/// An outline in logical pixels, drawn around the surface's shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceBorder {
    pub width: Pixels,
    pub color: Hsla,
}

/// Customizes the Material feedback painted by one interactive surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceFeedback {
    pub state_layer: bool,
    pub ripple: bool,
    pub focus_ring: bool,
    pub ink: Option<Hsla>,
    pub focus_color: Option<Hsla>,
}

impl Default for SurfaceFeedback {
    fn default() -> Self {
        Self {
            state_layer: true,
            ripple: true,
            focus_ring: true,
            ink: None,
            focus_color: None,
        }
    }
}

#[derive(Clone, Copy)]
enum SurfaceAction {
    Activate,
    Select(bool),
    Check(bool),
}

type Activate = Rc<dyn Fn(&mut Window, &mut App)>;

struct Control {
    id: ElementId,
    label: SharedString,
    action: SurfaceAction,
    activate: Option<Activate>,
}

/// A passive Material background or an interactive surface with arbitrary children.
#[derive(IntoElement)]
pub struct Surface {
    style: StyleRefinement,
    children: Vec<AnyElement>,
    role: ColorRole,
    container_color: Option<Hsla>,
    content_color: Option<Hsla>,
    inherit_content_color: bool,
    shape: Option<CornerShape>,
    polygon: Option<ShapePolygon>,
    shape_fit: ShapeFit,
    clip_backdrop: Option<Hsla>,
    shape_hit_test: Option<bool>,
    border: Option<SurfaceBorder>,
    tonal_elevation: Elevation,
    shadow_elevation: Elevation,
    shadow_scale: f32,
    theme: Option<MaterialTheme>,
    control: Option<Control>,
    disabled: bool,
    focus: Option<FocusHandle>,
    feedback: SurfaceFeedback,
    interaction_source: Option<Entity<InteractionSource>>,
}

impl Surface {
    pub fn new(role: ColorRole) -> Self {
        Self {
            style: StyleRefinement::default(),
            children: Vec::new(),
            role,
            container_color: None,
            content_color: None,
            inherit_content_color: false,
            shape: None,
            polygon: None,
            shape_fit: ShapeFit::Contain,
            clip_backdrop: None,
            shape_hit_test: None,
            border: None,
            tonal_elevation: Elevation::Level0,
            shadow_elevation: Elevation::Level0,
            shadow_scale: 1.,
            theme: None,
            control: None,
            disabled: false,
            focus: None,
            feedback: SurfaceFeedback::default(),
            interaction_source: None,
        }
    }

    pub fn clickable(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        role: ColorRole,
    ) -> Self {
        Self::interactive(id, label, role, SurfaceAction::Activate)
    }

    /// Renders the caller's selection and requests activation through on_activate.
    pub fn selectable(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        role: ColorRole,
        selected: bool,
    ) -> Self {
        Self::interactive(id, label, role, SurfaceAction::Select(selected))
    }

    /// Renders the caller's check state and requests the next value through on_checked_change.
    pub fn toggleable(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        role: ColorRole,
        checked: bool,
    ) -> Self {
        Self::interactive(id, label, role, SurfaceAction::Check(checked))
    }

    fn interactive(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        role: ColorRole,
        action: SurfaceAction,
    ) -> Self {
        Self {
            control: Some(Control {
                id: id.into(),
                label: label.into(),
                action,
                activate: None,
            }),
            ..Self::new(role)
        }
    }

    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        if let Some(control) = &mut self.control {
            control.activate = Some(Rc::new(handler));
        }
        self
    }

    pub fn on_checked_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        if let Some(control) = &mut self.control
            && let SurfaceAction::Check(checked) = control.action
        {
            control.activate = Some(Rc::new(move |window, cx| handler(!checked, window, cx)));
        }
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    pub fn feedback(mut self, feedback: SurfaceFeedback) -> Self {
        self.feedback = feedback;
        self
    }

    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.interaction_source = Some(source.clone());
        self
    }

    /// Uses an explicit fill and inherits the foreground unless one is supplied.
    pub fn container_color(mut self, color: Hsla) -> Self {
        self.container_color = Some(color);
        self
    }

    /// Sets the inherited foreground for this surface's children.
    pub fn content_color(mut self, color: Hsla) -> Self {
        self.content_color = Some(color);
        self
    }

    /// Uses the ancestor text color unless this surface supplies an explicit foreground.
    pub fn inherit_content_color(mut self, inherit: bool) -> Self {
        self.inherit_content_color = inherit;
        self
    }

    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self.polygon = None;
        self
    }

    /// Uses any [`ShapePolygon`] (including [`MaterialShape`](gpui_material_theme::MaterialShape),
    /// [`PolygonRole`](gpui_material_theme::PolygonRole), or sampled [`ShapeMorph`](crate::ShapeMorph)
    /// outlines) for background painting, border stroke, shadow, clipping, and feedback.
    ///
    /// Interactive polygon surfaces default to shape-aware pointer hit testing unless
    /// overridden with [`Self::shape_hit_test`].
    pub fn polygon_shape(mut self, polygon: impl Into<ShapePolygon>) -> Self {
        self.polygon = Some(polygon.into());
        self.shape = None;
        self
    }

    /// Chooses how a [`Self::polygon_shape`] maps into the laid-out surface bounds.
    pub fn shape_fit(mut self, fit: ShapeFit) -> Self {
        self.shape_fit = fit;
        self
    }

    /// Visually clips overflowing children to the resolved shape contour by applying a
    /// rectangular content mask and painting an EvenOdd exterior mask in `backdrop`.
    /// Match `backdrop` to the uniform background behind this surface.
    pub fn shape_clip_backdrop(mut self, backdrop: Hsla) -> Self {
        self.clip_backdrop = Some(backdrop);
        self
    }

    /// Controls whether pointer hover, press, and click activation require the cursor to
    /// lie inside the resolved shape contour. Defaults to `true` for [`Self::polygon_shape`]
    /// and `false` for rectangular or [`CornerShape`] surfaces. Keyboard and accessibility
    /// activation remain available regardless of this setting.
    pub fn shape_hit_test(mut self, enabled: bool) -> Self {
        self.shape_hit_test = Some(enabled);
        self
    }

    pub fn border(mut self, border: SurfaceBorder) -> Self {
        self.border = Some(border);
        self
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
        self.children.extend(elements);
    }
}

impl Styled for Surface {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

#[derive(Clone)]
enum ResolvedShape {
    Corners(CornerShape, Direction),
    Polygon(ShapePolygon, ShapeFit),
}

impl ResolvedShape {
    fn build(&self, bounds: Bounds<Pixels>) -> Option<Shape> {
        match self {
            Self::Corners(shape, direction) => Shape::rounded(bounds, *shape, *direction),
            Self::Polygon(polygon, fit) => Shape::polygon(bounds, polygon, *fit),
        }
    }
}

struct PolygonPaint {
    background: Hsla,
    shadow_elevation: f32,
    shadow_scale: f32,
    shadow_color: Hsla,
    border: Option<(Pixels, Hsla)>,
}

impl RenderOnce for Surface {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let current;
        let theme = match &self.theme {
            Some(theme) => theme,
            None => {
                current = current_material_theme(window, cx);
                &*current
            }
        };
        let scheme = &theme.colors;
        let default_focus = color(scheme.color(ColorRole::Secondary));
        let shadow_color = color(scheme.color(ColorRole::Shadow));
        let background = if self.role == ColorRole::Surface {
            scheme.surface_at_elevation(self.tonal_elevation)
        } else {
            scheme.color(self.role)
        };
        let direction = theme.layout.direction;
        let background = self.container_color.unwrap_or_else(|| color(background));
        let foreground = self.content_color.or_else(|| {
            if self.inherit_content_color || self.container_color.is_some() {
                return None;
            }
            scheme.pair(self.role).map(|pair| color(pair.foreground))
        });
        if let Some(foreground) = foreground {
            self.style.text.color = Some(foreground);
        }
        let border = self.border.map(|border| {
            let border_width = if f32::from(border.width).is_finite() {
                border.width.max(px(0.))
            } else {
                px(0.)
            };
            (border_width, border.color)
        });
        let polygon_paint = if self.polygon.is_some() {
            Some(PolygonPaint {
                background,
                shadow_elevation: self.shadow_elevation.dp(),
                shadow_scale: self.shadow_scale,
                shadow_color,
                border: border.filter(|(width, _)| *width > px(0.)),
            })
        } else {
            self.style.background = Some(background.into());
            let shadows =
                elevation_shadows(self.shadow_elevation.dp(), self.shadow_scale, shadow_color);
            if !shadows.is_empty() {
                self.style.box_shadow = Some(shadows);
            }
            if let Some(shape) = self.shape {
                if shape == CornerShape::Full {
                    self = self.rounded_full();
                } else {
                    let [tl, tr, br, bl] = shape.radii(f32::MAX, f32::MAX, direction);
                    self = self
                        .rounded_tl(px(tl))
                        .rounded_tr(px(tr))
                        .rounded_br(px(br))
                        .rounded_bl(px(bl));
                }
            }
            if let Some((border_width, border_color)) = border {
                let width = Some(border_width.into());
                self.style.border_widths.top = width;
                self.style.border_widths.right = width;
                self.style.border_widths.bottom = width;
                self.style.border_widths.left = width;
                self.style.border_color = Some(if border_width > px(0.) {
                    border_color
                } else {
                    gpui::transparent_black()
                });
            }
            None
        };
        let resolved_shape = self.resolve_shape(direction, window);
        let Some(control) = self.control.take() else {
            return self.render_passive(resolved_shape, polygon_paint);
        };
        self.render_control(
            control,
            resolved_shape,
            polygon_paint,
            default_focus,
            window,
            cx,
        )
    }
}

impl Surface {
    fn resolve_shape(&self, direction: Direction, window: &Window) -> ResolvedShape {
        if let Some(polygon) = &self.polygon {
            ResolvedShape::Polygon(polygon.clone(), self.shape_fit)
        } else if let Some(shape) = self.shape {
            ResolvedShape::Corners(shape, direction)
        } else {
            let radii = &self.style.corner_radii;
            let radius = |value: Option<gpui::AbsoluteLength>| {
                value.map_or(0., |value| value.to_pixels(window.rem_size()).into())
            };
            ResolvedShape::Corners(
                CornerShape::Radii([
                    radius(radii.top_left),
                    radius(radii.top_right),
                    radius(radii.bottom_right),
                    radius(radii.bottom_left),
                ]),
                Direction::Ltr,
            )
        }
    }

    fn render_passive(
        self,
        resolved_shape: ResolvedShape,
        polygon_paint: Option<PolygonPaint>,
    ) -> AnyElement {
        let clip_backdrop = self.clip_backdrop;
        if polygon_paint.is_none() && clip_backdrop.is_none() {
            return div()
                .refine_style(&self.style)
                .children(self.children)
                .into_any_element();
        }
        let prepared: Rc<RefCell<Option<Shape>>> = Rc::new(RefCell::new(None));
        let border = polygon_paint.as_ref().and_then(|paint| paint.border);
        let prepaint_shape = prepared.clone();
        let bg_shape = prepared.clone();
        let background_layer = polygon_paint.map(|paint| {
            let spec = resolved_shape.clone();
            canvas(
                move |bounds, _, _| {
                    *prepaint_shape.borrow_mut() = spec.build(bounds);
                },
                move |_, _, window, _| {
                    if let Some(shape) = bg_shape.borrow().as_ref() {
                        shape.paint_shadow(
                            paint.shadow_elevation,
                            paint.shadow_scale,
                            paint.shadow_color,
                            window,
                        );
                        shape.paint(paint.background, window);
                    }
                },
            )
            .absolute()
            .inset_0()
        });
        let needs_fg = clip_backdrop.is_some() || border.is_some();
        let fg_prepaint = prepared.clone();
        let fg_paint = prepared;
        let has_bg = background_layer.is_some();
        let foreground_layer = needs_fg.then(|| {
            canvas(
                move |bounds, _, _| {
                    if !has_bg {
                        *fg_prepaint.borrow_mut() = resolved_shape.build(bounds);
                    }
                },
                move |bounds, _, window, _| {
                    if let Some(shape) = fg_paint.borrow().as_ref() {
                        if let Some(backdrop) = clip_backdrop {
                            shape.paint_backdrop_mask(bounds, backdrop, window);
                        }
                        if let Some((width, color)) = border {
                            shape.paint_stroke(width, color, window);
                        }
                    }
                },
            )
            .absolute()
            .inset_0()
        });
        div()
            .relative()
            .when(clip_backdrop.is_some(), Styled::overflow_hidden)
            .refine_style(&self.style)
            .children(background_layer)
            .children(self.children)
            .children(foreground_layer)
            .into_any_element()
    }

    fn render_control(
        self,
        control: Control,
        resolved_shape: ResolvedShape,
        polygon_paint: Option<PolygonPaint>,
        default_focus: Hsla,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let focus = self.focus.unwrap_or_else(|| {
            window
                .use_keyed_state((control.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let state = ripple_state(&control.id, !self.disabled, window, cx);
        let shape_hit_test = self.shape_hit_test.unwrap_or(polygon_paint.is_some());
        state.borrow_mut().set_shape_hit_test(shape_hit_test);
        let tracker = self
            .interaction_source
            .as_ref()
            .map(|source| InteractionSource::track(source, &control.id, window, cx));
        let interaction = state
            .borrow_mut()
            .track(!self.disabled, &focus, tracker, window, cx);
        let appearance = self.feedback;
        let ink = appearance
            .ink
            .or(self.style.text.color)
            .unwrap_or_else(|| window.text_style().color);
        let ring = if appearance.focus_ring {
            appearance.focus_color.unwrap_or(default_focus)
        } else {
            gpui::transparent_black()
        };
        let clip_backdrop = self.clip_backdrop;
        let border = polygon_paint.as_ref().and_then(|paint| paint.border);
        let bg_geometry = state.clone();
        let bg_paint = state.clone();
        let background_layer = polygon_paint.map(|paint| {
            let spec = resolved_shape.clone();
            canvas(
                move |bounds, window, _| {
                    bg_geometry
                        .borrow_mut()
                        .set_shape(spec.build(bounds), window);
                },
                move |_, _, window, _| {
                    if let Some(shape) = &bg_paint.borrow().shape {
                        shape.paint_shadow(
                            paint.shadow_elevation,
                            paint.shadow_scale,
                            paint.shadow_color,
                            window,
                        );
                        shape.paint(paint.background, window);
                    }
                },
            )
            .absolute()
            .inset_0()
        });
        let has_bg = background_layer.is_some();
        let geometry = state.clone();
        let paint = state.clone();
        let visual = canvas(
            move |bounds, window, _| {
                if !has_bg {
                    match &resolved_shape {
                        ResolvedShape::Corners(corners, direction) => {
                            geometry
                                .borrow_mut()
                                .set_rounded_shape(bounds, *corners, *direction, window);
                        }
                        ResolvedShape::Polygon(polygon, fit) => {
                            geometry
                                .borrow_mut()
                                .set_shape(Shape::polygon(bounds, polygon, *fit), window);
                        }
                    }
                }
            },
            move |bounds, _, window, cx| {
                let state = paint.borrow();
                if let Some(shape) = &state.shape {
                    if let Some(backdrop) = clip_backdrop {
                        shape.paint_backdrop_mask(bounds, backdrop, window);
                    }
                    if let Some((width, color)) = border {
                        shape.paint_stroke(width, color, window);
                    }
                    let reduced = cx.reduce_motion();
                    let ripple = state.ripple.as_ref().filter(|_| appearance.ripple);
                    let layer = interaction
                        .layer_with_ripple(reduced || ripple.is_none())
                        .filter(|_| appearance.state_layer)
                        .map_or(0., |layer| layer.opacity());
                    InteractionFeedback::new(interaction, ink, ring).paint_with_opacity(
                        shape,
                        ripple,
                        layer,
                        cx.background_executor().now(),
                        reduced,
                        window,
                    );
                }
            },
        )
        .absolute()
        .inset_0();
        let button = Button::new(control.id)
            .block()
            .items_stretch()
            .justify_start()
            .when(clip_backdrop.is_some(), Styled::overflow_hidden)
            .line_height(window.text_style().line_height)
            .refine_style(&self.style)
            .accessibility_label(control.label)
            .track_focus(&focus)
            .disabled(self.disabled);
        let button = match control.action {
            SurfaceAction::Activate => button,
            SurfaceAction::Select(value) => button.role(Role::ListBoxOption).aria_selected(value),
            SurfaceAction::Check(value) => button.role(Role::CheckBox).aria_toggled(if value {
                Toggled::True
            } else {
                Toggled::False
            }),
        };
        let button = if let Some(activate) = control.activate.filter(|_| !self.disabled) {
            let accessible = activate.clone();
            let feedback = state.clone();
            button
                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    accessible(window, cx);
                })
                .on_click(move |event, window, cx| {
                    cx.stop_propagation();
                    if !feedback.borrow().blocks_activation(event) {
                        activate_feedback(event, &feedback, window, cx);
                        activate(window, cx);
                    }
                })
        } else {
            button
        };
        button
            .when(!self.disabled, |button| {
                feedback_target(button, state, true, Some(focus))
            })
            .children(background_layer)
            .children(self.children)
            .child(visual)
            .into_any_element()
    }
}
