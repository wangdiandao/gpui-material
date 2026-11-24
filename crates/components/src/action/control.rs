use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AccessibleAction, AnyElement, App, ElementId, Entity, FocusHandle, Hsla, IntoElement,
    ParentElement, Pixels, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::{Button, Toggle};
use gpui_material_foundation::{
    InteractionSource, InteractionState, InteractionTracker, ThemeScope,
    control_feedback::{FeedbackState, activate_feedback, feedback_target},
    current_material_theme,
};
use gpui_material_theme::{MotionKind, MotionSpeed};

use super::{
    style::{ActionLayout, ActionStyle, animate_visuals, resolve_palette},
    surface::render_visual_surface,
};

pub(crate) type Activate = Rc<dyn Fn(&mut Window, &mut App)>;
pub(crate) type Change = Rc<dyn Fn(bool, &mut Window, &mut App)>;

pub(crate) enum InvokeAction {
    Full(Activate),
    AccessibleOnly(Activate),
}

impl InvokeAction {
    pub(crate) fn into_parts(self) -> (Activate, bool) {
        match self {
            Self::Full(callback) => (callback, false),
            Self::AccessibleOnly(callback) => (callback, true),
        }
    }

    pub(crate) fn into_callback(self) -> Activate {
        match self {
            Self::Full(callback) | Self::AccessibleOnly(callback) => callback,
        }
    }
}

pub(crate) enum Action {
    Invoke(Option<InvokeAction>),
    Toggle {
        selected: bool,
        on_change: Option<Change>,
    },
}

pub(crate) struct ActionControl {
    pub(crate) id: ElementId,
    pub(crate) label: SharedString,
    pub(crate) focus: Option<FocusHandle>,
    pub(crate) disabled: bool,
    pub(crate) action: Action,
    pub(crate) style: ActionStyle,
    pub(crate) content: AnyElement,
}

impl ActionControl {
    fn feedback(
        &self,
        window: &mut Window,
        cx: &mut App,
    ) -> (Rc<RefCell<FeedbackState>>, FocusHandle) {
        let (feedback, focus) = window
            .use_keyed_state((self.id.clone(), "material-feedback"), cx, |_, cx| {
                (
                    Rc::new(RefCell::new(FeedbackState::default())),
                    cx.focus_handle(),
                )
            })
            .read(cx);
        (
            feedback.clone(),
            self.focus.as_ref().unwrap_or(focus).clone(),
        )
    }

    #[cfg(feature = "selection")]
    pub(crate) fn focus_handle(&self, window: &mut Window, cx: &mut App) -> FocusHandle {
        self.feedback(window, cx).1
    }

    pub(crate) fn clear_interaction(&self, window: &mut Window, cx: &mut App) {
        let (feedback, focus) = self.feedback(window, cx);
        let mut state = feedback.borrow_mut();
        state.clear();
        if focus.is_focused(window) {
            window.defer(cx, move |window, cx| {
                if focus.is_focused(window) {
                    window.blur(cx);
                }
            });
        }
    }

    pub(crate) fn render_layout(
        self,
        scale: f32,
        layout: ActionLayout,
        interaction_source: Option<&Entity<InteractionSource>>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let tracker =
            interaction_source.map(|source| InteractionSource::track(source, &self.id, window, cx));
        self.render_with_tracker(scale, layout, tracker, window, cx)
    }

    pub(crate) fn interaction_snapshot(
        &self,
        dragged: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> InteractionState {
        let (feedback, focus) = self.feedback(window, cx);
        let mut state = feedback.borrow_mut();
        let enabled = !self.disabled;
        state.cancel_unavailable(enabled, window);
        state.set_dragged(enabled && dragged);
        state.interaction(enabled, &focus, window)
    }

    pub(crate) fn render_with_tracker(
        self,
        scale: f32,
        layout: ActionLayout,
        tracker: Option<InteractionTracker>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let (feedback, focus) = self.feedback(window, cx);
        let style = self.style;
        let theme = current_material_theme(window, cx);
        let density = theme.layout.density;
        let direction = theme.layout.direction;
        let reduced = cx.reduce_motion();
        let (corner_kind, corner_speed) = style.corner_motion;
        let motion = theme.motion.spec(corner_kind, corner_speed, reduced);
        let effects = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let selected = layout.presentation_selected.unwrap_or(matches!(
            &self.action,
            Action::Toggle { selected: true, .. }
        ));
        let palette = resolve_palette(style, layout, self.disabled, selected, window, cx);
        let visual_height = px(density.visual_size(style.height) * scale);
        let visual_width = layout
            .width
            .unwrap_or_else(|| px(density.visual_size(style.min_width) * scale));
        let target_height = px(density.interactive_size(style.height) * scale);
        let target_width = layout
            .width
            .unwrap_or_else(|| px(density.interactive_size(style.min_width.max(1.)) * scale));

        let interactive = !self.disabled && !layout.loading;
        let interaction = record_interaction(
            &feedback,
            &focus,
            tracker,
            self.disabled,
            layout.loading,
            layout.dragged,
            layout.decorative,
            window,
            cx,
        );
        let visuals = animate_visuals(
            &self.id,
            style,
            layout,
            interaction,
            selected,
            interactive,
            scale,
            motion,
            effects,
            reduced,
            window,
            cx,
        );
        let visual = render_visual_surface(
            style,
            layout,
            direction,
            scale,
            palette,
            visuals,
            interaction,
            reduced,
            &feedback,
        );

        // The visual may be smaller than the interactive target. GPUI and Base
        // retain a single target and semantic node as the shape changes.
        let content = div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .min_w(visual_width)
            .max_w_full()
            .min_h(visual_height)
            .child(visual)
            .child(self.content);
        if layout.decorative {
            return div()
                .min_h(target_height)
                .min_w(target_width)
                .max_w_full()
                .text_color(palette.foreground)
                .child(content)
                .into_any_element();
        }
        render_action_control(
            self.id,
            self.label,
            self.disabled,
            focus,
            layout,
            self.action,
            content,
            feedback,
            palette.foreground,
            target_height,
            (target_width, layout.width.is_some()),
            window,
            cx,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn record_interaction(
    feedback: &RefCell<FeedbackState>,
    focus: &FocusHandle,
    tracker: Option<InteractionTracker>,
    disabled: bool,
    loading: bool,
    dragged: bool,
    decorative: bool,
    window: &Window,
    cx: &mut App,
) -> InteractionState {
    let interactive = !disabled && !loading;
    let mut state = feedback.borrow_mut();
    state.cancel_unavailable(interactive, window);
    state.set_dragged(interactive && dragged);
    let interaction = if decorative {
        let mut interaction = InteractionState::default();
        interaction.disabled = disabled;
        interaction
    } else if loading {
        // Keep keyboard focus ring visible while busy unless also disabled.
        let focus_state = state.interaction(!disabled, focus, window);
        let mut interaction = InteractionState::default();
        interaction.disabled = disabled;
        interaction.focus_visible = focus_state.focus_visible;
        interaction
    } else {
        state.interaction(interactive, focus, window)
    };
    state.track_snapshot(interaction, focus, tracker, cx);
    interaction
}

#[allow(clippy::too_many_arguments)]
fn render_action_control(
    id: ElementId,
    label: SharedString,
    disabled: bool,
    focus: FocusHandle,
    layout: ActionLayout,
    action: Action,
    content: impl IntoElement,
    feedback: Rc<RefCell<FeedbackState>>,
    foreground: Hsla,
    target_height: Pixels,
    width: (Pixels, bool),
    window: &Window,
    cx: &App,
) -> AnyElement {
    let interactive = !disabled && !layout.loading;
    match action {
        Action::Invoke(on_activate) => {
            let control = Button::new(id)
                .track_focus(&focus)
                .tab_stop(layout.tab_stop)
                .disabled(disabled)
                .accessibility_label(label)
                .when(layout.loading, |control| {
                    control.a11y_synthetic_children(|tree| {
                        tree.parent_node().set_busy();
                    })
                });
            let control = match on_activate {
                Some(action) if interactive => {
                    let (on_activate, semantic_only) = action.into_parts();
                    let on_activate = Rc::new(ThemeScope::bind_action(
                        move |window, cx| on_activate(window, cx),
                        window,
                        cx,
                    ));
                    let click_feedback = feedback.clone();
                    let semantic_feedback = feedback.clone();
                    let invoke = on_activate.clone();
                    control
                        .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                            if !semantic_feedback.borrow().is_dragged() {
                                invoke(window, cx);
                            }
                        })
                        .when(!semantic_only, |control| {
                            control.on_click(move |event, window, cx| {
                                cx.stop_propagation();
                                if !click_feedback.borrow().blocks_activation(event) {
                                    activate_feedback(event, &click_feedback, window, cx);
                                    on_activate(window, cx);
                                }
                            })
                        })
                }
                _ => control,
            };
            decorate(
                control,
                content,
                feedback,
                foreground,
                target_height,
                width,
                interactive,
            )
        }
        Action::Toggle {
            selected,
            on_change,
        } => {
            let control = Toggle::new(id)
                .track_focus(&focus)
                .tab_stop(layout.tab_stop)
                .pressed(selected)
                .disabled(disabled)
                .accessibility_label(label)
                .when(layout.loading, |control| {
                    control.a11y_synthetic_children(|tree| {
                        tree.parent_node().set_busy();
                    })
                });
            let control = match on_change {
                Some(on_change) if interactive => {
                    let pending = Rc::new(Cell::new(!selected));
                    let pending_arg = pending.clone();
                    let on_change = Rc::new(ThemeScope::bind_action(
                        move |window, cx| on_change(pending_arg.get(), window, cx),
                        window,
                        cx,
                    ));
                    let click_feedback = feedback.clone();
                    let semantic_feedback = feedback.clone();
                    let toggle = on_change.clone();
                    let semantic_pending = pending.clone();
                    control
                        .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                            if !semantic_feedback.borrow().is_dragged() {
                                semantic_pending.set(!selected);
                                toggle(window, cx);
                            }
                        })
                        .on_change(move |selected, event, window, cx| {
                            cx.stop_propagation();
                            if !click_feedback.borrow().blocks_activation(event) {
                                activate_feedback(event, &click_feedback, window, cx);
                                pending.set(selected);
                                on_change(window, cx);
                            }
                        })
                }
                _ => control,
            };
            decorate(
                control,
                content,
                feedback,
                foreground,
                target_height,
                width,
                interactive,
            )
        }
    }
}

fn decorate(
    control: impl Styled + StatefulInteractiveElement + ParentElement + IntoElement,
    content: impl IntoElement,
    feedback: Rc<RefCell<FeedbackState>>,
    foreground: Hsla,
    height: Pixels,
    (width, fixed_width): (Pixels, bool),
    enabled: bool,
) -> AnyElement {
    control
        .min_h(height)
        .min_w(width)
        .when(fixed_width, |control| control.w(width))
        .max_w_full()
        .text_color(foreground)
        .child(content)
        .when(enabled, |control| {
            feedback_target(control, feedback, true, None)
        })
        .into_any_element()
}
