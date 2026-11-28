use gpui::StatefulInteractiveElement;
use std::rc::Rc;

use gpui::{
    AccessibleAction, App, BorderStyle, Bounds, ContentMask, ElementId, Entity, FocusHandle, Hsla,
    InteractiveElement, IntoElement, ParentElement, Path, PathBuilder, PathStyle, Pixels,
    RenderOnce, Rgba, SharedString, StrokeOptions, Styled, Window, canvas, div, point,
    prelude::FluentBuilder, px, quad,
};
use gpui_base::{Checkbox as BaseCheckbox, TestSupportExt};
use gpui_material_foundation::{
    InteractionSource, animate, control_feedback::activate_feedback, current_material_theme,
};
use gpui_material_theme::{ColorRole, MotionKind, MotionSpec, MotionSpeed};

use super::{
    base::{BoundValueAction, SelectionControlBase},
    focus_style, selection_target, selection_tone, tone,
};

/// The semantic checked state of a controlled [`Checkbox`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CheckboxState {
    /// Not selected (default).
    #[default]
    Unchecked,
    /// Selected.
    Checked,
    /// Partially selected (mixed), typically derived from child states via [`Checkbox::aggregate`].
    Indeterminate,
}

impl CheckboxState {
    /// Returns `true` when the state is [`CheckboxState::Checked`].
    pub const fn is_checked(self) -> bool {
        matches!(self, Self::Checked)
    }

    /// Returns `true` when the state is [`CheckboxState::Indeterminate`].
    pub const fn is_indeterminate(self) -> bool {
        matches!(self, Self::Indeterminate)
    }

    /// Returns the next state requested when the user activates the checkbox.
    ///
    /// Both [`CheckboxState::Unchecked`] and [`CheckboxState::Indeterminate`]
    /// transition to [`CheckboxState::Checked`]; [`CheckboxState::Checked`]
    /// transitions to [`CheckboxState::Unchecked`].
    pub const fn next(self) -> Self {
        match self {
            Self::Unchecked | Self::Indeterminate => Self::Checked,
            Self::Checked => Self::Unchecked,
        }
    }

    pub(crate) const fn to_base(self) -> gpui_base::CheckboxState {
        match self {
            Self::Unchecked => gpui_base::CheckboxState::Unchecked,
            Self::Checked => gpui_base::CheckboxState::Checked,
            Self::Indeterminate => gpui_base::CheckboxState::Indeterminate,
        }
    }

    pub(crate) const fn from_base(state: gpui_base::CheckboxState) -> Self {
        match state {
            gpui_base::CheckboxState::Unchecked => Self::Unchecked,
            gpui_base::CheckboxState::Checked => Self::Checked,
            gpui_base::CheckboxState::Indeterminate => Self::Indeterminate,
        }
    }
}

impl From<bool> for CheckboxState {
    fn from(checked: bool) -> Self {
        if checked {
            Self::Checked
        } else {
            Self::Unchecked
        }
    }
}

type CheckboxChange = Rc<dyn Fn(CheckboxState, &mut Window, &mut App)>;

/// Optional pigments for one checkbox state. Unset fields use the window theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CheckboxStateColors {
    /// Stroke color of the checkmark or indeterminate dash.
    pub checkmark: Option<Hsla>,
    /// Interior fill color of the 18 dp checkbox square.
    pub box_color: Option<Hsla>,
    /// Outline stroke color of the checkbox square.
    pub border: Option<Hsla>,
}

/// Per-instance checkbox colors. Disabled entries take precedence over error entries.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CheckboxColors {
    /// Colors when enabled and unchecked.
    pub unchecked: CheckboxStateColors,
    /// Colors when enabled and checked.
    pub checked: CheckboxStateColors,
    /// Colors when enabled and indeterminate (mixed).
    pub indeterminate: CheckboxStateColors,
    /// Colors when enabled, in error state, and unchecked.
    pub error_unchecked: CheckboxStateColors,
    /// Colors when enabled, in error state, and checked.
    pub error_checked: CheckboxStateColors,
    /// Colors when enabled, in error state, and indeterminate.
    pub error_indeterminate: CheckboxStateColors,
    /// Colors when disabled and unchecked.
    pub disabled_unchecked: CheckboxStateColors,
    /// Colors when disabled and checked.
    pub disabled_checked: CheckboxStateColors,
    /// Colors when disabled and indeterminate.
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
    base: SelectionControlBase,
    state: CheckboxState,
    error: bool,
    colors: CheckboxColors,
    checkmark_stroke: StrokeOptions,
    outline_stroke: Pixels,
    on_change: Option<CheckboxChange>,
}

impl Checkbox {
    /// Creates a controlled checkbox with an element ID, accessible/visible label, and current state.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: CheckboxState,
    ) -> Self {
        Self {
            base: SelectionControlBase::new(id, label),
            state,
            error: false,
            colors: CheckboxColors::default(),
            checkmark_stroke: StrokeOptions {
                line_width: CHECKBOX_STROKE,
                ..StrokeOptions::default()
            },
            outline_stroke: px(CHECKBOX_STROKE),
            on_change: None,
        }
    }

    #[cfg(feature = "surfaces")]
    pub(crate) fn decorative(mut self, decorative: bool) -> Self {
        self.base.decorative(decorative);
        self
    }

    /// Controls whether the visible label is rendered. Defaults to `true`.
    /// When `false`, the constructor's label remains the accessible name and the trailing gap is omitted.
    pub fn show_label(mut self, show: bool) -> Self {
        self.base.show_label(show);
        self
    }

    /// Replaces the visible label while keeping the constructor's label as the accessible name.
    pub fn label_content(mut self, content: impl IntoElement) -> Self {
        self.base.label_content(content);
        self
    }

    /// Marks the checkbox as invalid, applying error styling and `aria-invalid` semantics when enabled.
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// Disables pointer, keyboard, and accessibility interaction when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.base.disabled(disabled);
        self
    }

    /// Overrides checkmark, box fill, and border colors per checkbox state.
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
        self.base.track_focus(focus);
        self
    }

    /// Reports pointer, keyboard and disabled feedback to a caller-owned source.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.base.interaction_source(source);
        self
    }

    /// Invoked with the requested next [`CheckboxState`] when toggled by pointer, keyboard, or assistive technology.
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
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.base.disabled;
        let checked = self.state != CheckboxState::Unchecked;
        let accent = if self.error && !disabled {
            tone(ColorRole::Error, window, cx)
        } else {
            selection_tone(checked, disabled, window, cx)
        };
        let on_accent = tone(
            if disabled {
                ColorRole::Surface
            } else if self.error {
                ColorRole::OnError
            } else {
                ColorRole::OnPrimary
            },
            window,
            cx,
        );
        let theme = current_material_theme(window, cx);
        let motion = theme.motion;
        let direction = theme.layout.direction;
        let density = theme.layout.density;
        let reduced = disabled || cx.reduce_motion();
        let effects = motion.spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let spatial = motion.spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let draw = animate(
            (self.base.id.clone(), "check-draw"),
            f32::from(checked),
            spatial,
            window,
            cx,
        )
        .clamp(0., 1.);
        let mixed = animate(
            (self.base.id.clone(), "check-shape"),
            f32::from(self.state == CheckboxState::Indeterminate),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let inactive = if self.error && !disabled {
            accent
        } else {
            selection_tone(false, disabled, window, cx)
        };
        let (box_color, border, mark) = resolve_checkbox_colors(
            &self.base.id,
            self.colors,
            self.state,
            self.error,
            disabled,
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
        if self.base.decorative {
            return div()
                .id(self.base.id)
                .test_support()
                .flex_none()
                .size(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .child(indicator)
                .into_any_element();
        }
        let interaction = self.base.prepare_interaction(None, window, cx);
        let group = self.base.group_name("material-checkbox");
        let target = selection_target(
            group.clone(),
            indicator,
            accent,
            !disabled,
            interaction.ripple.clone(),
            direction,
            density,
            cx.reduce_motion(),
        );
        let click_ripple = interaction.ripple.clone();
        let label = self.base.label.clone();
        let control = BaseCheckbox::new(self.base.id.clone())
            .track_focus(&interaction.focus)
            .state(self.state.to_base())
            .when(self.error && !disabled, |control| {
                control.a11y_synthetic_children(|node| {
                    node.parent_node()
                        .set_invalid(gpui::accesskit::Invalid::True);
                })
            })
            .disabled(disabled)
            .accessibility_label(label)
            .group(group)
            .border_2()
            .border_color(gpui::transparent_black())
            .rounded_md()
            .focus_visible(focus_style(tone(ColorRole::Secondary, window, cx)))
            .child(self.base.render_row(direction, target, window, cx));
        let control = interaction.attach_feedback(control);
        if disabled {
            return control.into_any_element();
        }
        match self.on_change {
            Some(handler) => {
                let next = self.state.next();
                let bound = BoundValueAction::new(next, handler, window, cx);
                let a11y_bound = bound.clone();
                control
                    .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        a11y_bound.invoke_default(window, cx);
                    })
                    .on_change(move |value, event, window, cx| {
                        cx.stop_propagation();
                        activate_feedback(event, &click_ripple, window, cx);
                        bound.invoke(CheckboxState::from_base(value), window, cx);
                    })
                    .into_any_element()
            }
            None => control
                .on_change(move |_, event, window, cx| {
                    cx.stop_propagation();
                    activate_feedback(event, &click_ripple, window, cx);
                })
                .into_any_element(),
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
            crate::control_style::mix_color(inactive, accent, fill),
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
    if !stroke.miter_limit.is_finite() || stroke.miter_limit < 1. {
        stroke.miter_limit = StrokeOptions::default().miter_limit;
    }
    stroke
}

fn checkbox_mark(
    bounds: Bounds<Pixels>,
    draw: f32,
    mixed: f32,
    stroke: StrokeOptions,
) -> Option<Path> {
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
    fn checkbox_state_is_material_owned_and_preserves_helpers_and_base_roundtrip() {
        assert_ne!(
            std::any::TypeId::of::<CheckboxState>(),
            std::any::TypeId::of::<gpui_base::CheckboxState>()
        );
        assert_eq!(CheckboxState::default(), CheckboxState::Unchecked);
        assert_eq!(CheckboxState::from(false), CheckboxState::Unchecked);
        assert_eq!(CheckboxState::from(true), CheckboxState::Checked);
        assert!(!CheckboxState::Unchecked.is_checked());
        assert!(CheckboxState::Checked.is_checked());
        assert!(!CheckboxState::Indeterminate.is_checked());
        assert!(CheckboxState::Indeterminate.is_indeterminate());
        assert_eq!(CheckboxState::Unchecked.next(), CheckboxState::Checked);
        assert_eq!(CheckboxState::Indeterminate.next(), CheckboxState::Checked);
        assert_eq!(CheckboxState::Checked.next(), CheckboxState::Unchecked);
        assert_eq!(Checkbox::aggregate([]), CheckboxState::Unchecked);
        assert_eq!(
            Checkbox::aggregate([CheckboxState::Checked, CheckboxState::Checked]),
            CheckboxState::Checked
        );
        assert_eq!(
            Checkbox::aggregate([CheckboxState::Checked, CheckboxState::Unchecked]),
            CheckboxState::Indeterminate
        );
        for state in [
            CheckboxState::Unchecked,
            CheckboxState::Checked,
            CheckboxState::Indeterminate,
        ] {
            assert_eq!(CheckboxState::from_base(state.to_base()), state);
        }
    }

    #[test]
    fn checkbox_mark_strokes_fit_and_draw_checked_and_mixed_geometry() {
        let bounds = Bounds::new(
            point(px(0.), px(0.)),
            size(px(CHECKBOX_SIZE), px(CHECKBOX_SIZE)),
        );
        let default = checkbox_mark_stroke(StrokeOptions {
            line_width: 2.,
            ..StrokeOptions::default()
        });
        let wide = checkbox_mark_stroke(StrokeOptions {
            line_width: 4.,
            ..StrokeOptions::default()
        });
        assert_eq!(default.line_width, 2.);
        assert_eq!(wide.line_width, 4.);
        assert!(checkbox_mark(bounds, 1., 1., default).is_some());
        assert!(checkbox_mark(bounds, 1., 1., wide).is_some());
        assert!(checkbox_mark(bounds, 1., 0., default).is_some());
        assert!(checkbox_mark(bounds, 1., 0., wide).is_some());
        assert!(checkbox_mark(bounds, 0., 0., wide).is_none());
        assert!(
            checkbox_mark(
                bounds,
                1.,
                0.,
                StrokeOptions {
                    line_width: 0.,
                    ..wide
                }
            )
            .is_none()
        );
        assert_eq!(checkbox_stroke_width(f32::MAX, CHECKBOX_SIZE * 0.5), 9.);
        for invalid in [-1., f32::NAN, f32::INFINITY] {
            assert_eq!(checkbox_stroke_width(invalid, CHECKBOX_SIZE * 0.5), 2.);
            assert_eq!(
                checkbox_mark_stroke(StrokeOptions {
                    line_width: invalid,
                    ..wide
                })
                .line_width,
                2.
            );
        }
    }
}
