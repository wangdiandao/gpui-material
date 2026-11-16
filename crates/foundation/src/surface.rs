use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, ElementId, Entity, FocusHandle, Hsla, IntoElement,
    ParentElement, Pixels, RenderOnce, Rgba, Role, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Toggled, Window, canvas, div, prelude::FluentBuilder, px,
};
use gpui_base::{Button, StyledExt};
use gpui_material_theme::{Argb, ColorRole, CornerShape, Direction, Elevation, MaterialTheme};

use crate::{
    InteractionFeedback, InteractionSource, Shape,
    control_feedback::{activate_feedback, feedback_target, ripple_state},
    window_material_theme,
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

impl RenderOnce for Surface {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self
            .theme
            .as_ref()
            .unwrap_or_else(|| window_material_theme(window, cx));
        let scheme = &theme.colors;
        let default_focus = color(scheme.color(ColorRole::Secondary));
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
        self.style.background = Some(background.into());
        if let Some(foreground) = foreground {
            self.style.text.color = Some(foreground);
        }
        let shadows = elevation_shadows(
            self.shadow_elevation.dp(),
            self.shadow_scale,
            color(scheme.color(ColorRole::Shadow)),
        );
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
        if let Some(border) = self.border {
            let border_width = if f32::from(border.width).is_finite() {
                border.width.max(px(0.))
            } else {
                px(0.)
            };
            let width = Some(border_width.into());
            self.style.border_widths.top = width;
            self.style.border_widths.right = width;
            self.style.border_widths.bottom = width;
            self.style.border_widths.left = width;
            self.style.border_color = Some(if border_width > px(0.) {
                border.color
            } else {
                gpui::transparent_black()
            });
        }
        let Some(control) = self.control.take() else {
            return div()
                .refine_style(&self.style)
                .children(self.children)
                .into_any_element();
        };
        self.render_control(control, direction, default_focus, window, cx)
    }
}

impl Surface {
    fn render_control(
        self,
        control: Control,
        direction: Direction,
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
        let tracker = self
            .interaction_source
            .as_ref()
            .map(|source| InteractionSource::track(source, &control.id, window, cx));
        let interaction = state
            .borrow_mut()
            .track(!self.disabled, &focus, tracker, window, cx);
        let (shape, shape_direction) = if let Some(shape) = self.shape {
            (shape, direction)
        } else {
            let radii = &self.style.corner_radii;
            let radius = |value: Option<gpui::AbsoluteLength>| {
                value.map_or(0., |value| value.to_pixels(window.rem_size()).into())
            };
            (
                CornerShape::Radii([
                    radius(radii.top_left),
                    radius(radii.top_right),
                    radius(radii.bottom_right),
                    radius(radii.bottom_left),
                ]),
                Direction::Ltr,
            )
        };
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
        let geometry = state.clone();
        let paint = state.clone();
        let visual = canvas(
            move |bounds, _, _| {
                geometry.borrow_mut().shape = Shape::rounded(bounds, shape, shape_direction);
            },
            move |_, _, window, cx| {
                let state = paint.borrow();
                if let Some(shape) = &state.shape {
                    let reduced = cx.reduce_motion();
                    let ripple = state.ripple.as_ref().filter(|_| appearance.ripple);
                    let layer = interaction
                        .layer_with_ripple(reduced || !appearance.ripple)
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
            .children(self.children)
            .child(visual)
            .into_any_element()
    }
}
