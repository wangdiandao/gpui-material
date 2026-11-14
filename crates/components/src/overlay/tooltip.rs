use std::time::Duration;

use gpui::{
    Anchor, AnyElement, App, Context, ElementId, FocusHandle, InteractiveElement, IntoElement,
    KeyBinding, ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    Subscription, Task, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Popup, TestSupportExt};
use gpui_material_foundation::Surface;
use gpui_material_theme::{ColorRole, Elevation, TypeRole};

use crate::{Button, ButtonVariant, Text};

gpui::actions!(
    material_tooltip,
    [
        FocusTooltipAction,
        FocusTooltipTrigger,
        DismissTooltip,
        ActivateTooltipAction
    ]
);

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", FocusTooltipAction, Some("MaterialTooltipTrigger")),
        KeyBinding::new(
            "shift-tab",
            FocusTooltipTrigger,
            Some("MaterialTooltipAction"),
        ),
        KeyBinding::new("escape", DismissTooltip, Some("MaterialTooltip")),
        KeyBinding::new(
            "enter",
            ActivateTooltipAction,
            Some("MaterialTooltipAction"),
        ),
        KeyBinding::new(
            "space",
            ActivateTooltipAction,
            Some("MaterialTooltipAction"),
        ),
    ]);
}

/// A delayed plain or interactive rich Material tooltip anchored to a trigger.
#[derive(IntoElement)]
pub struct Tooltip {
    id: ElementId,
    trigger: AnyElement,
    text: SharedString,
    title: Option<SharedString>,
    action: Option<TooltipAction>,
    rich: bool,
    trigger_focus: Option<FocusHandle>,
}

type TooltipCallback = std::rc::Rc<dyn Fn(&mut Window, &mut App)>;

struct TooltipAction {
    label: SharedString,
    focus: FocusHandle,
    callback: TooltipCallback,
}

impl Tooltip {
    pub fn plain(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        text: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            text: text.into(),
            title: None,
            action: None,
            rich: false,
            trigger_focus: None,
        }
    }

    pub fn rich(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        title: impl Into<SharedString>,
        text: impl Into<SharedString>,
    ) -> Self {
        Self {
            title: Some(title.into()),
            rich: true,
            ..Self::plain(id, trigger, text)
        }
    }

    /// Adds a control to a rich tooltip.
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        focus: &FocusHandle,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.rich = true;
        self.action = Some(TooltipAction {
            label: label.into(),
            focus: focus.clone(),
            callback: std::rc::Rc::new(callback),
        });
        self
    }

    /// Uses the focus handle associated with the trigger for keyboard activation.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.trigger_focus = Some(focus.clone());
        self
    }
}

#[derive(Default)]
struct TooltipState {
    open: bool,
    trigger_hover: bool,
    surface_hover: bool,
    keyboard: bool,
    suppress_focus: bool,
    surface_focus: bool,
    task: Option<Task<()>>,
    focus: Option<TooltipFocus>,
}

struct TooltipFocus {
    trigger: FocusHandle,
    surface: FocusHandle,
    _subscriptions: [Subscription; 4],
}

impl TooltipState {
    fn schedule(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let open = self.trigger_hover || self.surface_hover || self.keyboard || self.surface_focus;
        self.task = None;
        if open == self.open {
            return;
        }
        if self.keyboard && open {
            self.open = true;
            window.refresh();
            return;
        }
        let delay = if open {
            Duration::from_millis(500)
        } else {
            Duration::from_millis(200)
        };
        self.task = Some(cx.spawn_in(window, async move |state, cx| {
            cx.background_executor().timer(delay).await;
            let _ = state.update_in(cx, |state, window, _| {
                state.task = None;
                state.open = open;
                window.refresh();
            });
        }));
    }

    fn dismiss(&mut self, window: &mut Window) {
        self.open = false;
        self.task = None;
        self.surface_hover = false;
        self.keyboard = false;
        self.surface_focus = false;
        self.suppress_focus = true;
        window.refresh();
    }
}

impl RenderOnce for Tooltip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let width = window.viewport_size().width * 0.75;
        let role = if self.rich {
            ColorRole::SurfaceContainer
        } else {
            ColorRole::InverseSurface
        };
        let state = window.use_keyed_state((self.id.clone(), "tooltip-state"), cx, |_, _| {
            TooltipState::default()
        });
        if let Some(focus) = self.trigger_focus {
            state.update(cx, |state, cx| {
                if state
                    .focus
                    .as_ref()
                    .is_some_and(|previous| previous.trigger == focus)
                {
                    return;
                }
                let enter = cx.entity().downgrade();
                let leave = enter.clone();
                let enter_surface = enter.clone();
                let leave_surface = enter.clone();
                let surface = cx.focus_handle();
                let on_enter = window.on_focus_in(&focus, cx, move |window, cx| {
                    let _ = enter.update(cx, |state, cx| {
                        state.keyboard = !state.suppress_focus;
                        state.schedule(window, cx);
                    });
                });
                let on_leave = window.on_focus_out(&focus, cx, move |_, window, cx| {
                    let _ = leave.update(cx, |state, cx| {
                        state.keyboard = false;
                        state.suppress_focus = false;
                        state.schedule(window, cx);
                    });
                });
                let on_enter_surface = window.on_focus_in(&surface, cx, move |window, cx| {
                    let _ = enter_surface.update(cx, |state, cx| {
                        state.surface_focus = true;
                        state.schedule(window, cx);
                    });
                });
                let on_leave_surface = window.on_focus_out(&surface, cx, move |_, window, cx| {
                    let _ = leave_surface.update(cx, |state, cx| {
                        state.surface_focus = false;
                        state.schedule(window, cx);
                    });
                });
                state.focus = Some(TooltipFocus {
                    trigger: focus,
                    surface,
                    _subscriptions: [on_enter, on_leave, on_enter_surface, on_leave_surface],
                });
            });
        }
        let observed = state.read(cx);
        let open = observed.open
            || (!observed.suppress_focus
                && observed.focus.as_ref().is_some_and(|focus| {
                    focus.trigger.is_focused(window) || focus.surface.contains_focused(window, cx)
                }))
            || self
                .action
                .as_ref()
                .is_some_and(|action| action.focus.is_focused(window));
        let surface_focus = observed.focus.as_ref().map(|focus| focus.surface.clone());
        let trigger_focus = observed.focus.as_ref().map(|focus| focus.trigger.clone());
        let action_focus = self.action.as_ref().map(|action| action.focus.clone());
        let dismiss_action = action_focus.clone();
        let action_callback = self.action.as_ref().map(|action| action.callback.clone());
        let hover = state.clone();
        let dismiss = state.clone();
        let to_action = action_focus;
        let to_trigger = trigger_focus.clone();
        Popup::new(
            self.id,
            div()
                .id("tooltip-trigger")
                .when(open && to_action.is_some(), |trigger| {
                    trigger.key_context("MaterialTooltipTrigger").on_action(
                        move |_: &FocusTooltipAction, window, cx| {
                            if let Some(action) = &to_action {
                                action.focus(window, cx);
                            }
                        },
                    )
                })
                .role(Role::Group)
                .aria_description(self.text.clone())
                .on_hover(move |hovering, window, cx| {
                    hover.update(cx, |state, cx| {
                        state.trigger_hover = *hovering;
                        state.schedule(window, cx);
                    });
                })
                .child(self.trigger),
        )
        .anchor(Anchor::TopCenter)
        .offset(px(4.))
        .when(open, |popup| {
            popup
                .key_context("MaterialTooltip")
                .on_action(move |_: &DismissTooltip, window, cx| {
                    let return_to = dismiss.read(cx).focus.as_ref().and_then(|focus| {
                        (focus.surface.contains_focused(window, cx)
                            || dismiss_action
                                .as_ref()
                                .is_some_and(|action| action.is_focused(window)))
                        .then(|| focus.trigger.clone())
                    });
                    if let Some(trigger) = return_to {
                        trigger.focus(window, cx);
                    }
                    dismiss.update(cx, |state, _| state.dismiss(window));
                })
        })
        .when(open, |popup| {
            popup.content(
                div()
                    .id("tooltip")
                    .test_support()
                    .role(Role::Tooltip)
                    .key_context("MaterialTooltipAction")
                    .on_action(move |_: &FocusTooltipTrigger, window, cx| {
                        if let Some(trigger) = &to_trigger {
                            trigger.focus(window, cx);
                        } else {
                            cx.propagate();
                        }
                    })
                    .on_action(move |_: &ActivateTooltipAction, window, cx| {
                        if let Some(action) = &action_callback {
                            action(window, cx);
                        } else {
                            cx.propagate();
                        }
                    })
                    .when_some(surface_focus, |tip, focus| tip.track_focus(&focus))
                    .aria_label(self.text.clone())
                    .on_hover(move |hovering, window, cx| {
                        state.update(cx, |state, cx| {
                            state.surface_hover = *hovering;
                            state.schedule(window, cx);
                        });
                    })
                    .child(
                        Surface::new(role)
                            .flex()
                            .flex_col()
                            .rounded_sm()
                            .max_w(width.min(px(320.)))
                            .when_else(
                                self.rich,
                                |surface| surface.gap_2().p_4().shadow_elevation(Elevation::Level2),
                                |surface| surface.px_2().py_1(),
                            )
                            .children(
                                self.title
                                    .map(|title| Text::new(title).role(TypeRole::TitleSmall)),
                            )
                            .child(self.text)
                            .children(self.action.map(|action| {
                                Button::new("tooltip-action", action.label)
                                    .track_focus(&action.focus)
                                    .variant(ButtonVariant::Text)
                                    .on_activate(move |window, cx| (action.callback)(window, cx))
                            })),
                    ),
            )
        })
    }
}
