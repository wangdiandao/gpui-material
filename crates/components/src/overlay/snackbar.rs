use std::{collections::VecDeque, rc::Rc, time::Duration};

use gpui::{
    App, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, Render, Role,
    SharedString, StatefulInteractiveElement, Styled, Task, Window, div, prelude::FluentBuilder,
    px,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{Surface, color, window_material_theme};
use gpui_material_theme::{ColorRole, Direction, Elevation};

type Action = Rc<dyn Fn(&mut Window, &mut App)>;

/// A message for a window's snackbar queue. The default timeout is four seconds.
pub struct Snackbar {
    message: SharedString,
    timeout: Option<Duration>,
    action: Option<(SharedString, Action)>,
}

impl Snackbar {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            timeout: Some(Duration::from_secs(4)),
            action: None,
        }
    }

    /// Sets how long the message is visible. `None` waits for explicit dismissal.
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Adds a keyboard-accessible action. By default, messages with an action
    /// persist. Call `timeout` after this method to set a duration explicitly.
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        action: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(action)));
        self.timeout = None;
        self
    }
}

/// One FIFO message queue per host. Mount the entity in the window layout.
/// A new message leaves keyboard focus wherever it is.
#[derive(Default)]
pub struct SnackbarHost {
    active: Option<(usize, Snackbar)>,
    pending: VecDeque<Snackbar>,
    next_id: usize,
    timer: Option<Task<()>>,
}

impl SnackbarHost {
    /// Queues the message after any already pending messages.
    pub fn push(&mut self, snackbar: Snackbar, window: &mut Window, cx: &mut Context<Self>) {
        self.pending.push_back(snackbar);
        if self.active.is_none() {
            self.show_next(window, cx);
        }
    }

    /// Dismisses the active message. If any are queued, the next begins its own timeout.
    pub fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((id, _)) = &self.active {
            self.finish(*id, false, window, cx);
        }
    }

    fn show_next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active = self.pending.pop_front().map(|message| {
            self.next_id += 1;
            (self.next_id, message)
        });
        self.timer = self.active.as_ref().and_then(|(id, snackbar)| {
            let id = *id;
            let timeout = snackbar.timeout?;
            Some(cx.spawn_in(window, async move |this, cx| {
                cx.background_executor().timer(timeout).await;
                let _ = this.update_in(cx, |this, window, cx| {
                    this.finish(id, false, window, cx);
                });
            }))
        });
        cx.notify();
    }

    fn finish(&mut self, id: usize, run_action: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .active
            .as_ref()
            .is_none_or(|(current, _)| *current != id)
        {
            return;
        }
        let message = self.active.take().unwrap().1;
        self.show_next(window, cx);
        if let Some((_, action)) = message.action.filter(|_| run_action) {
            // The application may enqueue another message from this callback.
            window.defer(cx, move |window, cx| action(window, cx));
        }
    }
}

impl Render for SnackbarHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some((id, message)) = &self.active else {
            return div().into_any_element();
        };
        let id = *id;
        let theme = window_material_theme(window, cx);
        let action_color = color(theme.colors.color(ColorRole::InversePrimary));
        let rtl = theme.layout.direction == Direction::Rtl;
        let activate = cx.listener(move |this, _: &ClickEvent, window, cx| {
            this.finish(id, true, window, cx);
        });
        let dismiss = cx.listener(move |this, _: &ClickEvent, window, cx| {
            this.finish(id, false, window, cx);
        });
        let keyboard_style = move |button: gpui_base::Button| {
            button
                .min_h(px(36.))
                .px_2()
                .rounded_md()
                .text_color(action_color)
                .hover(move |style| style.bg(action_color.opacity(0.12)))
                .focus_visible(move |style| style.border_2().border_color(action_color))
        };
        div()
            .id(("snackbar", id))
            .child(
                Surface::new(ColorRole::InverseSurface)
                    .shadow_elevation(Elevation::Level3)
                    .flex()
                    .items_center()
                    .min_h(px(48.))
                    .gap_2()
                    .px_4()
                    .py_1()
                    .rounded_sm()
                    .when(rtl, Styled::flex_row_reverse)
                    .child(
                        div()
                            .id(("snackbar-message", id))
                            .test_support()
                            .role(Role::Status)
                            .aria_label(message.message.clone())
                            .a11y_synthetic_children(|tree| {
                                let node = tree.parent_node();
                                node.set_live(gpui::accesskit::Live::Polite);
                                node.set_live_atomic();
                            })
                            .min_w_0()
                            .flex_1()
                            .child(message.message.clone()),
                    )
                    .when_some(message.action.as_ref(), |surface, (label, _)| {
                        surface.child(
                            div().id("snackbar-action-region").test_support().child(
                                keyboard_style(gpui_base::Button::new("snackbar-action"))
                                    .accessibility_label(label.clone())
                                    .on_click(activate)
                                    .child(label.clone()),
                            ),
                        )
                    })
                    .child(
                        keyboard_style(gpui_base::Button::new("snackbar-dismiss"))
                            .accessibility_label("Dismiss message")
                            .on_click(dismiss)
                            .child("Dismiss"),
                    ),
            )
            .into_any_element()
    }
}
