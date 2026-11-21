use std::{cell::Cell, rc::Rc};

use gpui::{
    AccessibleAction, App, BorderStyle, Bounds, ContentMask, ElementId, Entity, FocusHandle, Hsla,
    InteractiveElement, IntoElement, ParentElement, Path, PathBuilder, PathStyle, Pixels,
    RenderOnce, Rgba, SharedString, StatefulInteractiveElement, StrokeOptions, Styled, Window,
    canvas, div, point, prelude::FluentBuilder, px, quad,
};
use gpui_base::{Checkbox as BaseCheckbox, CheckboxState, TestSupportExt};
use gpui_material_foundation::{InteractionSource, ThemeScope, animate, current_material_theme};
use gpui_material_theme::{ColorRole, MotionKind, MotionSpec, MotionSpeed};

use crate::action::{feedback_target, ripple_state, ripple_target};

use super::{focus_style, row, selection_label, selection_target, selection_tone, tone};

type CheckboxChange = Rc<dyn Fn(CheckboxState, &mut Window, &mut App)>;

/// Optional pigments for one checkbox state. Unset fields use the window theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CheckboxStateColors {
    pub checkmark: Option<Hsla>,
    pub box_color: Option<Hsla>,
    pub border: Option<Hsla>,
}

/// Per-instance checkbox colors. Disabled entries take precedence over error entries.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CheckboxColors {
    pub unchecked: CheckboxStateColors,
    pub checked: CheckboxStateColors,
    pub indeterminate: CheckboxStateColors,
    pub error_unchecked: CheckboxStateColors,
    pub error_checked: CheckboxStateColors,
    pub error_indeterminate: CheckboxStateColors,
    pub disabled_unchecked: CheckboxStateColors,
    pub disabled_checked: CheckboxStateColors,
    pub disabled_indeterminate: CheckboxStateColors,
}

impl CheckboxColors {
    fn for_state(self, state: CheckboxState, error: bool, disabled: bool) -> CheckboxStateColors {
        use CheckboxState::{Checked, Indeterminate, Unchecked};
        match (disabled, error, state) {
            (true, _, Unchecked) => self.disabled_unchecked,
            (true, _, Checked) => self.disabled_checked,
            (true, _, Indeterminate) => self.disabled_indeterminate,
            (false, true, Unchecked) => self.error_unchecked,
            (false, true, Checked) => self.error_checked,
            (false, true, Indeterminate) => self.error_indeterminate,
            (false, false, Unchecked) => self.unchecked,
            (false, false, Checked) => self.checked,
            (false, false, Indeterminate) => self.indeterminate,
        }
    }
}

const CHECKBOX_SIZE: f32 = 18.;
const CHECKBOX_STROKE: f32 = 2.;

/// Controlled Material checkbox. Mixed activation requests Checked.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    label: SharedString,
    state: CheckboxState,
    error: bool,
    disabled: bool,
    colors: CheckboxColors,
    checkmark_stroke: StrokeOptions,
    outline_stroke: Pixels,
    focus: Option<FocusHandle>,
    interaction_source: Option<Entity<InteractionSource>>,
    on_change: Option<CheckboxChange>,
    decorative: bool,
}

impl Checkbox {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: CheckboxState,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state,
            error: false,
            disabled: false,
            colors: CheckboxColors::default(),
            checkmark_stroke: StrokeOptions::default().with_line_width(CHECKBOX_STROKE),
            outline_stroke: px(CHECKBOX_STROKE),
            focus: None,
            interaction_source: None,
            on_change: None,
            decorative: false,
        }
    }

    #[cfg(feature = "surfaces")]
    pub(crate) fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn colors(mut self, colors: CheckboxColors) -> Self {
        self.colors = colors;
        self
    }

    /// Sets the uniform path stroke used for the checkmark and the mixed mark.
    pub fn checkmark_stroke(mut self, stroke: StrokeOptions) -> Self {
        self.checkmark_stroke = stroke;
        self
    }

    /// Sets the width of the rounded box outline. Zero omits the outline.
    pub fn outline_stroke(mut self, width: Pixels) -> Self {
        self.outline_stroke = width;
        self
    }

    /// Uses the caller's focus handle instead of an internal keyed handle.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    /// Reports pointer, keyboard and disabled feedback to a caller-owned source.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.interaction_source = Some(source.clone());
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(CheckboxState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Derives a parent value from its children. An empty set is unchecked.
    pub fn aggregate(children: impl IntoIterator<Item = CheckboxState>) -> CheckboxState {
        let mut children = children.into_iter();
        let first = children.next().unwrap_or_default();
        if children.all(|child| child == first) {
            first
        } else {
            CheckboxState::Indeterminate
        }
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.state != CheckboxState::Unchecked;
        let accent = if self.error && !self.disabled {
            tone(ColorRole::Error, window, cx)
        } else {
            selection_tone(checked, self.disabled, window, cx)
        };
        let on_accent = tone(
            if self.disabled {
                ColorRole::Surface
            } else if self.error {
                ColorRole::OnError
            } else {
                ColorRole::OnPrimary
            },
            window,
            cx,
        );
        let motion = current_material_theme(window, cx).motion;
        let reduced = self.disabled || cx.reduce_motion();
        let effects = motion.spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let spatial = motion.spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let draw = animate(
            (self.id.clone(), "check-draw"),
            f32::from(checked),
            spatial,
            window,
            cx,
        )
        .clamp(0., 1.);
        let mixed = animate(
            (self.id.clone(), "check-shape"),
            f32::from(self.state == CheckboxState::Indeterminate),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let inactive = if self.error && !self.disabled {
            accent
        } else {
            selection_tone(false, self.disabled, window, cx)
        };
        let (box_color, border, mark) = resolve_checkbox_colors(
            &self.id,
            self.colors,
            self.state,
            self.error,
            self.disabled,
            accent,
            inactive,
            on_accent,
            effects,
            window,
            cx,
        );
        let mark_stroke = checkbox_mark_stroke(self.checkmark_stroke);
        let outline = px(checkbox_stroke_width(
            self.outline_stroke.into(),
            CHECKBOX_SIZE * 0.5,
        ));
        let indicator = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                window.paint_quad(quad(
                    bounds,
                    px(2.),
                    box_color,
                    outline,
                    if outline == px(0.) {
                        gpui::transparent_black()
                    } else {
                        border
                    },
                    BorderStyle::Solid,
                ));
                if let Some(path) = checkbox_mark(bounds, draw, mixed, mark_stroke) {
                    window.with_content_mask(Some(ContentMask { bounds }), |window| {
                        window.paint_path(path, mark);
                    });
                }
            },
        )
        .size(px(CHECKBOX_SIZE))
        .flex_none();
        if self.decorative {
            return div()
                .id(self.id)
                .test_support()
                .flex_none()
                .size(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .child(indicator)
                .into_any_element();
        }
        let direction = current_material_theme(window, cx).layout.direction;
        let ripple = ripple_state(&self.id, !self.disabled, window, cx);
        let focus = self.focus.unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let tracker = self
            .interaction_source
            .as_ref()
            .map(|source| InteractionSource::track(source, &self.id, window, cx));
        let track_hover = tracker.is_some();
        ripple
            .borrow_mut()
            .track(!self.disabled, &focus, tracker, window, cx);
        let group: SharedString = format!("material-checkbox/{}", self.id).into();
        let label = self.label;
        let control = BaseCheckbox::new(self.id)
            .track_focus(&focus)
            .when(!self.disabled, |control| {
                if track_hover {
                    feedback_target(control, ripple.clone(), true, Some(focus.clone()))
                } else {
                    ripple_target(control, ripple.clone())
                }
            })
            .state(self.state)
            .when(self.error, |control| {
                control.a11y_synthetic_children(|node| {
                    node.parent_node()
                        .set_invalid(gpui::accesskit::Invalid::True);
                })
            })
            .disabled(self.disabled)
            .accessibility_label(label.clone())
            .group(group.clone())
            .border_2()
            .border_color(gpui::transparent_black())
            .rounded_md()
            .focus_visible(focus_style(tone(ColorRole::Secondary, window, cx)))
            .child(
                row(direction)
                    .child(selection_target(
                        group,
                        indicator,
                        accent,
                        !self.disabled,
                        ripple,
                        direction,
                        cx.reduce_motion(),
                    ))
                    .child(selection_label(label, self.disabled, window, cx)),
            );
        match self.on_change {
            Some(handler) if !self.disabled => {
                let next = if self.state == CheckboxState::Checked {
                    CheckboxState::Unchecked
                } else {
                    CheckboxState::Checked
                };
                let pending = Rc::new(Cell::new(next));
                let pending_arg = pending.clone();
                let handler = Rc::new(ThemeScope::bind_action(
                    move |window, cx| handler(pending_arg.get(), window, cx),
                    window,
                    cx,
                ));
                let change = handler.clone();
                let a11y_pending = pending.clone();
                control
                    .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        a11y_pending.set(next);
                        change(window, cx);
                    })
                    .on_change(move |value, _, window, cx| {
                        pending.set(value);
                        handler(window, cx);
                    })
                    .into_any_element()
            }
            _ => control.into_any_element(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_checkbox_colors(
    id: &ElementId,
    colors: CheckboxColors,
    state: CheckboxState,
    error: bool,
    disabled: bool,
    accent: Hsla,
    inactive: Hsla,
    on_accent: Hsla,
    effects: MotionSpec,
    window: &mut Window,
    cx: &mut App,
) -> (Hsla, Hsla, Hsla) {
    let checked = state != CheckboxState::Unchecked;
    if colors == CheckboxColors::default() {
        let fill = animate(
            (id.clone(), "check-fill"),
            f32::from(checked),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        (
            accent.opacity(fill),
            crate::motion::mix(inactive, accent, fill),
            on_accent,
        )
    } else {
        let colors = colors.for_state(state, error, disabled);
        let mut sample = |name, color| {
            animate_checkbox_color((id.clone(), name).into(), color, effects, window, cx)
        };
        (
            sample(
                "check-box-color",
                colors
                    .box_color
                    .unwrap_or(accent.opacity(f32::from(checked))),
            ),
            sample(
                "check-border-color",
                colors
                    .border
                    .unwrap_or(if checked { accent } else { inactive }),
            ),
            sample("check-mark-color", colors.checkmark.unwrap_or(on_accent)),
        )
    }
}

fn checkbox_stroke_width(width: f32, max: f32) -> f32 {
    if width.is_finite() && width >= 0. {
        width.min(max)
    } else {
        CHECKBOX_STROKE
    }
}

fn checkbox_mark_stroke(mut stroke: StrokeOptions) -> StrokeOptions {
    stroke.line_width = checkbox_stroke_width(stroke.line_width, CHECKBOX_SIZE);
    stroke.variable_line_width = None;
    if !stroke.miter_limit.is_finite() || stroke.miter_limit < StrokeOptions::MINIMUM_MITER_LIMIT {
        stroke.miter_limit = StrokeOptions::DEFAULT_MITER_LIMIT;
    }
    if !stroke.tolerance.is_finite() || stroke.tolerance <= 0. {
        stroke.tolerance = StrokeOptions::DEFAULT_TOLERANCE;
    } else {
        stroke.tolerance = stroke.tolerance.max(0.01);
    }
    stroke
}

fn checkbox_mark(
    bounds: Bounds<Pixels>,
    draw: f32,
    mixed: f32,
    stroke: StrokeOptions,
) -> Option<Path<Pixels>> {
    if draw == 0. || stroke.line_width == 0. {
        return None;
    }
    let side = bounds.size.width;
    let start = point(side * 0.2, side * 0.5);
    let knee = point(side * (0.4 + 0.1 * mixed), side * (0.7 - 0.2 * mixed));
    let end = point(side * 0.8, side * (0.3 + 0.2 * mixed));
    let first = f32::from(knee.x - start.x).hypot(f32::from(knee.y - start.y));
    let second = f32::from(end.x - knee.x).hypot(f32::from(end.y - knee.y));
    let distance = (first + second) * draw;
    let mut path = PathBuilder::default().with_style(PathStyle::Stroke(stroke));
    path.move_to(bounds.origin + start);
    if distance <= first {
        path.line_to(bounds.origin + start + (knee - start) * (distance / first));
    } else {
        path.line_to(bounds.origin + knee);
        path.line_to(bounds.origin + knee + (end - knee) * ((distance - first) / second));
    }
    path.build().ok()
}

fn animate_checkbox_color(
    id: ElementId,
    color: Hsla,
    spec: MotionSpec,
    window: &mut Window,
    cx: &mut App,
) -> Hsla {
    let target = Rgba::from(color);
    let mut sample = |channel, value| animate((id.clone(), channel), value, spec, window, cx);
    let current = Rgba {
        r: sample("red", target.r),
        g: sample("green", target.g),
        b: sample("blue", target.b),
        a: sample("alpha", target.a),
    };
    if current == target {
        color
    } else {
        current.into()
    }
}

#[cfg(test)]
mod checkbox_tests {
    use super::*;
    use gpui::size;

    #[test]
    fn checkbox_mark_strokes_fit_and_draw_checked_and_mixed_geometry() {
        let bounds = Bounds::new(
            point(px(0.), px(0.)),
            size(px(CHECKBOX_SIZE), px(CHECKBOX_SIZE)),
        );
        let default = checkbox_mark_stroke(StrokeOptions::default().with_line_width(2.));
        let wide = checkbox_mark_stroke(StrokeOptions::default().with_line_width(4.));
        let mixed = checkbox_mark(bounds, 1., 1., default).unwrap();
        assert_eq!(mixed.bounds.size.height, px(2.));
        let wider_mixed = checkbox_mark(bounds, 1., 1., wide).unwrap();
        assert_eq!(wider_mixed.bounds.size.height, px(4.));
        let checked = checkbox_mark(bounds, 1., 0., default).unwrap();
        let wider_checked = checkbox_mark(bounds, 1., 0., wide).unwrap();
        assert!(checked.bounds.size.height > mixed.bounds.size.height);
        assert!(wider_checked.bounds.size.height > checked.bounds.size.height);
        assert!(checkbox_mark(bounds, 0., 0., wide).is_none());
        assert!(checkbox_mark(bounds, 1., 0., wide.with_line_width(0.)).is_none());
        assert_eq!(checkbox_stroke_width(f32::MAX, CHECKBOX_SIZE * 0.5), 9.);
        for invalid in [-1., f32::NAN, f32::INFINITY] {
            assert_eq!(checkbox_stroke_width(invalid, CHECKBOX_SIZE * 0.5), 2.);
            assert_eq!(
                checkbox_mark_stroke(wide.with_line_width(invalid)).line_width,
                2.
            );
        }
    }
}
