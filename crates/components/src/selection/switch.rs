use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, Div, ElementId, Entity, FocusHandle, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Switch as BaseSwitch, TestSupportExt};
use gpui_material_foundation::{InteractionSource, ThemeScope, animate, current_material_theme};
use gpui_material_theme::{ColorRole, Direction, MotionKind, MotionSpec, MotionSpeed, StateLayer};

use crate::action::{Change, FeedbackState, feedback_target, ripple_ink, ripple_state};

use super::{focus_style, row, selection_label, tone};

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
            // Compose precomposites disabled pigments over the surface color.
            let muted = |role, opacity| color(Surface).blend(color(role).opacity(opacity));
            return if checked {
                self.disabled_checked.resolve(SwitchPalette {
                    track: muted(OnSurface, 0.12),
                    border: gpui::transparent_black(),
                    thumb: color(Surface),
                    content: muted(OnSurface, 0.38),
                })
            } else {
                self.disabled_unchecked.resolve(SwitchPalette {
                    track: muted(SurfaceContainerHighest, 0.12),
                    border: muted(OnSurface, 0.12),
                    thumb: muted(OnSurface, 0.38),
                    content: muted(SurfaceContainerHighest, 0.38),
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
            _ => crate::motion::mix(off, on, tint),
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

/// Material switch. The caller owns the checked state and optional thumb content.
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    label: SharedString,
    show_label: bool,
    label_content: Option<AnyElement>,
    checked: bool,
    disabled: bool,
    checked_icon: Option<AnyElement>,
    unchecked_icon: Option<AnyElement>,
    thumb_content: Option<AnyElement>,
    colors: SwitchColors,
    focus: Option<FocusHandle>,
    interaction_source: Option<Entity<InteractionSource>>,
    on_change: Option<Change>,
    decorative: bool,
}

impl Switch {
    /// Creates a controlled switch with an element ID, accessible/visible label, and current checked state.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, checked: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            show_label: true,
            label_content: None,
            checked,
            disabled: false,
            checked_icon: None,
            unchecked_icon: None,
            thumb_content: None,
            colors: SwitchColors::default(),
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

    /// Controls whether the visible label is rendered. Defaults to `true`.
    /// When `false`, the constructor's label remains the accessible name and the trailing gap is omitted.
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    /// Replaces the visible label while keeping the constructor's label as the accessible name.
    pub fn label_content(mut self, content: impl IntoElement) -> Self {
        self.label_content = Some(content.into_any_element());
        self
    }

    /// Disables pointer, keyboard, and accessibility interaction when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
        self.focus = Some(focus.clone());
        self
    }

    /// Reports presentation state to a caller-owned source.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.interaction_source = Some(source.clone());
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
        has_decoration: bool,
        active: bool,
        motion: MotionSpec,
        window: &mut Window,
        cx: &mut App,
    ) -> (f32, Pixels) {
        let normal_size = if self.checked || has_decoration {
            24.
        } else {
            16.
        };
        let diameter = animate(
            (self.id.clone(), "thumb-diameter"),
            if active { 28. } else { normal_size },
            if active { MotionSpec::Instant } else { motion },
            window,
            cx,
        )
        .clamp(16., 28.);
        let offset = animate(
            (self.id.clone(), "thumb-offset"),
            match (active, self.checked) {
                (true, false) => 2.,
                (true, true) => 22.,
                (false, true) => 24.,
                (false, false) => (32. - normal_size) / 2.,
            },
            if active { MotionSpec::Instant } else { motion },
            window,
            cx,
        );
        // Offset and diameter are sampled independently, including a reversal on release.
        (diameter, px(offset.clamp(2., 50. - diameter)))
    }
}

#[allow(clippy::too_many_arguments)]
fn render_state_layer(
    diameter: f32,
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
        .size(px(40.))
        .left(px((diameter - 40.) / 2.))
        .top(px((diameter - 40.) / 2.))
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
        .w(px(52.))
        .h(px(32.))
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
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let reduced = self.disabled || cx.reduce_motion();
        let motion = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let effects = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let tint = animate(
            (self.id.clone(), "switch-color"),
            f32::from(self.checked),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        let palette = self
            .colors
            .resolve(self.checked, self.disabled, tint, window, cx);
        let decoration = self.thumb_decoration();
        if self.decorative {
            let (diameter, offset) =
                self.thumb_geometry(decoration.is_some(), false, motion, window, cx);
            let track = render_track(
                &self.id,
                palette,
                diameter,
                offset,
                direction,
                div().id("switch-state-layer"),
                decoration,
            );
            return div()
                .id(self.id)
                .flex_none()
                .child(track)
                .into_any_element();
        }
        let ripple = ripple_state(&self.id, !self.disabled, window, cx);
        let focus = self.focus.take().unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let tracker = self
            .interaction_source
            .as_ref()
            .map(|source| InteractionSource::track(source, &self.id, window, cx));
        let observes_input = tracker.is_some();
        ripple
            .borrow_mut()
            .track(!self.disabled, &focus, tracker, window, cx);
        let active = ripple.borrow().pressed();
        let (diameter, offset) =
            self.thumb_geometry(decoration.is_some(), active, motion, window, cx);
        let group: SharedString = format!("material-switch/{}", self.id).into();
        let state_layer = render_state_layer(
            diameter,
            self.checked,
            self.disabled,
            active,
            &group,
            ripple.clone(),
            direction,
            window,
            cx,
        );
        let track = render_track(
            &self.id,
            palette,
            diameter,
            offset,
            direction,
            state_layer,
            decoration,
        );
        let control = BaseSwitch::new(self.id)
            .checked(self.checked)
            .disabled(self.disabled)
            .track_focus(&focus)
            .group(group)
            .accessibility_label(self.label.clone())
            .when(!self.disabled, |control| {
                feedback_target(
                    control,
                    ripple,
                    observes_input,
                    observes_input.then_some(focus),
                )
            })
            .border_2()
            .border_color(gpui::transparent_black())
            .rounded_md()
            .focus_visible(focus_style(tone(ColorRole::Secondary, window, cx)))
            .child(
                row(direction, self.show_label)
                    .id("switch-row")
                    .min_h(px(48.))
                    .px_1()
                    .child(track)
                    .children(self.show_label.then(|| {
                        selection_label(self.label, self.label_content, self.disabled, window, cx)
                    })),
            );
        match self.on_change {
            Some(handler) if !self.disabled => {
                let next = !self.checked;
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
                        change(window, cx)
                    })
                    .on_change(move |checked, _, window, cx| {
                        pending.set(checked);
                        handler(window, cx)
                    })
                    .into_any_element()
            }
            _ => control.into_any_element(),
        }
    }
}
