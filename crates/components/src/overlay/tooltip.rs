use std::time::Duration;

use gpui::{
    Anchor, AnyElement, App, Context, ElementId, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    Subscription, Task, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::Popup;
use gpui_material_foundation::Surface;
use gpui_material_theme::{ColorRole, Elevation, TypeRole};

use crate::Text;

/// A delayed plain or interactive rich Material tooltip anchored to a trigger.
#[derive(IntoElement)]
pub struct Tooltip {
    id: ElementId,
    trigger: AnyElement,
    text: SharedString,
    title: Option<SharedString>,
    action: Option<AnyElement>,
    rich: bool,
    trigger_focus: Option<FocusHandle>,
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
    pub fn action(mut self, element: impl IntoElement) -> Self {
        self.rich = true;
        self.action = Some(element.into_any_element());
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
    task: Option<Task<()>>,
    focus: Option<(FocusHandle, Subscription, Subscription)>,
}

impl TooltipState {
    fn schedule(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let open = self.trigger_hover || self.surface_hover || self.keyboard;
        self.task = None;
        if open == self.open {
            return;
        }
        let delay = if self.keyboard {
            Duration::ZERO
        } else if open {
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

    fn dismiss(&mut self, window: &mut Window) -> bool {
        let was_active = self.open || self.task.is_some();
        self.open = false;
        self.task = None;
        self.surface_hover = false;
        self.keyboard = false;
        if was_active {
            window.refresh();
        }
        was_active
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
                    .is_some_and(|(previous, ..)| previous == &focus)
                {
                    return;
                }
                let enter = cx.entity().downgrade();
                let leave = enter.clone();
                let on_enter = window.on_focus_in(&focus, cx, move |window, cx| {
                    let _ = enter.update(cx, |state, cx| {
                        state.keyboard = true;
                        state.schedule(window, cx);
                    });
                });
                let on_leave = window.on_focus_out(&focus, cx, move |_, window, cx| {
                    let _ = leave.update(cx, |state, cx| {
                        state.keyboard = false;
                        state.schedule(window, cx);
                    });
                });
                state.focus = Some((focus, on_enter, on_leave));
            });
        }
        let open = state.read(cx).open;
        let hover = state.clone();
        let dismiss = state.clone();
        Popup::new(
            self.id,
            div()
                .id("tooltip-trigger")
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
        .capture_key_down(move |event, window, cx| {
            if event.keystroke.key == "escape"
                && dismiss.update(cx, |state, _| state.dismiss(window))
            {
                cx.stop_propagation();
            }
        })
        .when(open, |popup| {
            popup.content(
                div()
                    .id("tooltip")
                    .role(Role::Tooltip)
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
                            .children(self.action),
                    ),
            )
        })
    }
}
