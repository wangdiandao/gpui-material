use std::{cell::RefCell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, Div, ElementId, Entity, FocusHandle, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Switch as BaseSwitch, TestSupportExt};
use gpui_material_foundation::{
    InteractionSource, animate,
    control_feedback::{FeedbackState, activate_feedback},
    current_material_theme,
};
use gpui_material_theme::{
    ColorRole, Density, Direction, MotionKind, MotionSpec, MotionSpeed, StateLayer,
};

use crate::action::{Change, ripple_ink};

use super::{
    base::{BoundValueAction, SelectionControlBase},
    focus_style, tone,
};

/// Optional pigments for one switch state. Omitted fields use the window theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SwitchStateColors {
    /// Background fill color of the 52 × 32 dp switch track.
    pub track: Option<Hsla>,
    /// Outline stroke color of the switch track (visible when unchecked).
    pub border: Option<Hsla>,
    /// Fill color of the circular switch thumb.
    pub thumb: Option<Hsla>,
    /// Foreground tint for thumb icons or custom thumb content.
    pub content: Option<Hsla>,
}

/// Per-instance colors for enabled and disabled switches in either checked state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SwitchColors {
    /// Colors when enabled and unchecked.
    pub unchecked: SwitchStateColors,
    /// Colors when enabled and checked.
    pub checked: SwitchStateColors,
    /// Colors when disabled and unchecked.
    pub disabled_unchecked: SwitchStateColors,
    /// Colors when disabled and checked.
    pub disabled_checked: SwitchStateColors,
}

struct SwitchPalette {
    track: Hsla,
    border: Hsla,
    thumb: Hsla,
    content: Hsla,
}

impl SwitchStateColors {
    fn resolve(self, defaults: SwitchPalette) -> SwitchPalette {
        SwitchPalette {
            track: self.track.unwrap_or(defaults.track),
            border: self.border.unwrap_or(defaults.border),
            thumb: self.thumb.unwrap_or(defaults.thumb),
            content: self.content.unwrap_or(defaults.content),
        }
    }
}

impl SwitchColors {
    fn resolve(
        self,
        checked: bool,
        disabled: bool,
        tint: f32,
        window: &Window,
        cx: &App,
    ) -> SwitchPalette {
        use ColorRole::*;
        let color = |role| tone(role, window, cx);
        if disabled {
            return if checked {
                self.disabled_checked.resolve(SwitchPalette {
                    track: color(OnSurface).opacity(0.12),
                    border: gpui::transparent_black(),
                    thumb: color(Surface),
                    content: color(OnSurface).opacity(0.38),
                })
            } else {
                self.disabled_unchecked.resolve(SwitchPalette {
                    track: color(SurfaceContainerHighest).opacity(0.12),
                    border: color(OnSurface).opacity(0.12),
                    thumb: color(OnSurface).opacity(0.38),
                    content: color(SurfaceContainerHighest).opacity(0.38),
                })
            };
        }

        let unchecked = self.unchecked.resolve(SwitchPalette {
            track: color(SurfaceContainerHighest),
            border: color(Outline),
            thumb: color(Outline),
            content: color(SurfaceContainerHighest),
        });
        let on = self.checked.resolve(SwitchPalette {
            track: color(Primary),
            border: unchecked.border.opacity(0.),
            thumb: color(OnPrimary),
            content: color(OnPrimaryContainer),
        });
        let mix = |off, on| match tint {
            0. => off,
            1. => on,
            _ => crate::control_style::mix_color(off, on, tint),
        };
        SwitchPalette {
            track: mix(unchecked.track, on.track),
            border: mix(unchecked.border, on.border),
            thumb: mix(unchecked.thumb, on.thumb),
            content: if checked {
                on.content
            } else {
                unchecked.content
            },
        }
    }
}

#[derive(Clone, Copy)]
struct SwitchMetrics {
    track_width: f32,
    track_height: f32,
    state_layer: f32,
    hit_target: f32,
    pressed_thumb: f32,
    checked_thumb: f32,
    unchecked_thumb: f32,
}

impl SwitchMetrics {
    fn from_density(density: Density) -> Self {
        let track_height = density.visual_size(32.).max(20.);
        let track_width = density.visual_size(52.).max(36.);
        let delta = track_height - 32.;
        Self {
            track_width,
            track_height,
            state_layer: density.visual_size(40.).max(24.),
            hit_target: density.interactive_size(32.),
            pressed_thumb: (28. + delta).max(20.),
            checked_thumb: (24. + delta).max(16.),
            unchecked_thumb: (16. + delta).max(12.),
        }
    }
}

/// Material switch. The caller owns the checked state and optional thumb content.
#[derive(IntoElement)]
pub struct Switch {
    base: SelectionControlBase,
    checked: bool,
    checked_icon: Option<AnyElement>,
    unchecked_icon: Option<AnyElement>,
    thumb_content: Option<AnyElement>,
    colors: SwitchColors,
    on_change: Option<Change>,
}

impl Switch {
    /// Creates a controlled switch with an element ID, accessible/visible label, and current checked state.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, checked: bool) -> Self {
        Self {
            base: SelectionControlBase::new(id, label),
            checked,
            checked_icon: None,
            unchecked_icon: None,
            thumb_content: None,
            colors: SwitchColors::default(),
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

    /// Disables pointer, keyboard, and accessibility interaction when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.base.disabled(disabled);
        self
    }

    /// Sets an icon rendered inside the thumb when `checked` is `true`.
    pub fn checked_icon(mut self, icon: impl IntoElement) -> Self {
        self.checked_icon = Some(icon.into_any_element());
        self
    }

    /// Sets an icon rendered inside the thumb when `checked` is `false`.
    pub fn unchecked_icon(mut self, icon: impl IntoElement) -> Self {
        self.unchecked_icon = Some(icon.into_any_element());
        self
    }

    /// Supplies decorative content in the centered 16 px thumb slot. The
    /// element takes precedence over the icon preset for this render.
    pub fn thumb_content(mut self, content: impl IntoElement) -> Self {
        self.thumb_content = Some(content.into_any_element());
        self
    }

    /// Overrides track, border, thumb, and thumb-content colors per switch state.
    pub fn colors(mut self, colors: SwitchColors) -> Self {
        self.colors = colors;
        self
    }

    /// Uses the caller's focus handle instead of an internal keyed handle.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.base.track_focus(focus);
        self
    }

    /// Reports presentation state to a caller-owned source.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.base.interaction_source(source);
        self
    }

    /// Invoked with the requested next checked state when toggled by pointer, keyboard, or assistive technology.
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn thumb_decoration(&mut self) -> Option<AnyElement> {
        let icon = if self.checked {
            self.checked_icon.take()
        } else {
            self.unchecked_icon.take()
        };
        self.thumb_content
            .take()
            .map(|content| {
                div()
                    .id("switch-thumb-content")
                    .role(gpui::Role::Group)
                    .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
                    .size(px(16.))
                    .overflow_hidden()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(content)
                    .into_any_element()
            })
            .or(icon)
    }

    fn thumb_geometry(
        &self,
        metrics: SwitchMetrics,
        has_decoration: bool,
        active: bool,
        motion: MotionSpec,
        window: &mut Window,
        cx: &mut App,
    ) -> (f32, Pixels) {
        let normal_size = if self.checked || has_decoration {
            metrics.checked_thumb
        } else {
            metrics.unchecked_thumb
        };
        let diameter = animate(
            (self.base.id.clone(), "thumb-diameter"),
            if active {
                metrics.pressed_thumb
            } else {
                normal_size
            },
            if active { MotionSpec::Instant } else { motion },
            window,
            cx,
        )
        .clamp(metrics.unchecked_thumb, metrics.pressed_thumb);
        let offset = animate(
            (self.base.id.clone(), "thumb-offset"),
            match (active, self.checked) {
                (true, false) => (metrics.track_height - metrics.pressed_thumb) / 2.,
                (true, true) => metrics.track_width - metrics.pressed_thumb - 2.,
                (false, true) => {
                    metrics.track_width - normal_size - (metrics.track_height - normal_size) / 2.
                }
                (false, false) => (metrics.track_height - normal_size) / 2.,
            },
            if active { MotionSpec::Instant } else { motion },
            window,
            cx,
        );
        // Offset and diameter are sampled independently, including a reversal on release.
        let max_offset = (metrics.track_width - 2. - diameter).max(2.);
        (diameter, px(offset.clamp(2., max_offset)))
    }
}

#[allow(clippy::too_many_arguments)]
fn render_state_layer(
    diameter: f32,
    state_layer_size: f32,
    checked: bool,
    disabled: bool,
    active: bool,
    group: &SharedString,
    ink: Rc<RefCell<FeedbackState>>,
    direction: Direction,
    window: &Window,
    cx: &App,
) -> Stateful<Div> {
    use ColorRole::*;
    let accent = tone(if checked { Primary } else { OnSurface }, window, cx);
    div()
        .id("switch-state-layer")
        .absolute()
        .size(px(state_layer_size))
        .left(px((diameter - state_layer_size) / 2.))
        .top(px((diameter - state_layer_size) / 2.))
        .rounded_full()
        .when(!disabled, |layer| {
            let static_pressed = active && cx.reduce_motion();
            layer
                .when(static_pressed, |layer| {
                    layer.bg(accent.opacity(StateLayer::Pressed.opacity()))
                })
                .when(!static_pressed, |layer| {
                    layer.group_hover(group.clone(), move |style| {
                        style.bg(accent.opacity(StateLayer::Hover.opacity()))
                    })
                })
                .when(cx.reduce_motion(), |layer| {
                    layer.group_active(group.clone(), move |style| {
                        style.bg(accent.opacity(StateLayer::Pressed.opacity()))
                    })
                })
        })
        .child(ripple_ink(
            ink,
            accent,
            gpui_material_theme::CornerRole::Full.shape(),
            direction,
        ))
}

#[allow(clippy::too_many_arguments)]
fn render_track(
    id: &ElementId,
    metrics: SwitchMetrics,
    palette: SwitchPalette,
    diameter: f32,
    offset: Pixels,
    direction: Direction,
    state_layer: Stateful<Div>,
    decoration: Option<AnyElement>,
) -> impl IntoElement + use<> {
    let handle = div()
        .absolute()
        .top_0()
        .h_full()
        .w(px(diameter))
        .flex()
        .items_center()
        .justify_center()
        .when(direction == Direction::Ltr, |handle| handle.left(offset))
        .when(direction == Direction::Rtl, |handle| handle.right(offset))
        .child(
            div()
                .id((id.clone(), "switch-thumb"))
                .relative()
                .test_support()
                .size(px(diameter))
                .rounded_full()
                .bg(palette.thumb)
                .text_color(palette.content)
                .text_size(px(16.))
                .flex()
                .items_center()
                .justify_center()
                .child(state_layer)
                .children(decoration),
        );
    div()
        .id((id.clone(), "switch-track"))
        .test_support()
        .relative()
        .w(px(metrics.track_width))
        .h(px(metrics.track_height))
        .rounded_full()
        .bg(palette.track)
        .when(palette.border.a > 0., |track| {
            track.child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded_full()
                    .border_2()
                    .border_color(palette.border),
            )
        })
        .child(handle)
}

impl RenderOnce for Switch {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.base.disabled;
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let metrics = SwitchMetrics::from_density(theme.layout.density);
        let reduced = disabled || cx.reduce_motion();
        let motion = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let effects = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let tint = animate(
            (self.base.id.clone(), "switch-color"),
            f32::from(self.checked),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let palette = self
            .colors
            .resolve(self.checked, disabled, tint, window, cx);
        let decoration = self.thumb_decoration();
        if self.base.decorative {
            let (diameter, offset) =
                self.thumb_geometry(metrics, decoration.is_some(), false, motion, window, cx);
            let track = render_track(
                &self.base.id,
                metrics,
                palette,
                diameter,
                offset,
                direction,
                div().id("switch-state-layer"),
                decoration,
            );
            return div()
                .id(self.base.id)
                .flex_none()
                .child(track)
                .into_any_element();
        }
        let interaction = self.base.prepare_interaction(None, window, cx);
        let active = interaction.ripple.borrow().pressed();
        let (diameter, offset) =
            self.thumb_geometry(metrics, decoration.is_some(), active, motion, window, cx);
        let group = self.base.group_name("material-switch");
        let state_layer = render_state_layer(
            diameter,
            metrics.state_layer,
            self.checked,
            disabled,
            active,
            &group,
            interaction.ripple.clone(),
            direction,
            window,
            cx,
        );
        let track = render_track(
            &self.base.id,
            metrics,
            palette,
            diameter,
            offset,
            direction,
            state_layer,
            decoration,
        );
        let click_ripple = interaction.ripple.clone();
        let label = self.base.label.clone();
        let control = BaseSwitch::new(self.base.id.clone())
            .checked(self.checked)
            .disabled(disabled)
            .track_focus(&interaction.focus)
            .group(group)
            .accessibility_label(label)
            .border_2()
            .border_color(gpui::transparent_black())
            .rounded_md()
            .focus_visible(focus_style(tone(ColorRole::Secondary, window, cx)))
            .child(
                self.base
                    .render_row(direction, track, window, cx)
                    .id("switch-row")
                    .min_h(px(metrics.hit_target))
                    .px_1(),
            );
        let control = interaction.attach_feedback(control);
        if disabled {
            return control.into_any_element();
        }
        match self.on_change {
            Some(handler) => {
                let next = !self.checked;
                let bound = BoundValueAction::new(next, handler, window, cx);
                let a11y_bound = bound.clone();
                control
                    .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        a11y_bound.invoke_default(window, cx);
                    })
                    .on_change(move |checked, event, window, cx| {
                        cx.stop_propagation();
                        activate_feedback(event, &click_ripple, window, cx);
                        bound.invoke(checked, window, cx);
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
