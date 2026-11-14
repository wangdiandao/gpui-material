use std::{cell::Cell, collections::VecDeque, rc::Rc, time::Duration};

use crate::motion::{OverlayMotion, ScaledOverlay};
use gpui::{
    App, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, Render, Role,
    SharedString, StatefulInteractiveElement, Styled, Task, Window, canvas, div, point,
    prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{Surface, color, window_material_theme};
use gpui_material_theme::{ColorRole, Direction, Elevation, TypeRole};

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
    exiting: Vec<(usize, Snackbar)>,
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
        let entry = self.active.take().unwrap();
        let action = entry.1.action.as_ref().map(|(_, action)| action.clone());
        self.exiting.push(entry);
        self.show_next(window, cx);
        if let Some(action) = action.filter(|_| run_action) {
            // The application may enqueue another message from this callback.
            window.defer(cx, move |window, cx| action(window, cx));
        }
    }
}

impl Render for SnackbarHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let host_width = window
            .use_keyed_state("snackbar-host-width", cx, |_, _| Rc::new(Cell::new(px(0.))))
            .read(cx)
            .clone();
        let width = host_width.get();
        let mut root = div().relative().w_full().child(
            canvas(
                move |bounds, window, _| {
                    if host_width.replace(bounds.size.width) != bounds.size.width {
                        window.request_animation_frame();
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        );
        let mut has_layout = self.active.is_some();
        self.exiting.retain(|(id, message)| {
            let Some(motion) = OverlayMotion::sample(&("snackbar", *id).into(), false, window, cx)
            else {
                return false;
            };
            let content = snackbar_surface(*id, message, motion, width, false, window, cx);
            let scaled = ScaledOverlay::new(content, motion, |_| point(0.5, 0.5)).width(width);
            let slot = div().when(has_layout, |div| div.absolute().inset_0());
            root.extend([slot.child(scaled).into_any_element()]);
            has_layout = true;
            true
        });
        if let Some((id, message)) = &self.active {
            let motion = OverlayMotion::sample(&("snackbar", *id).into(), true, window, cx)
                .expect("active snackbar has motion");
            let content = snackbar_surface(*id, message, motion, width, true, window, cx);
            root =
                root.child(ScaledOverlay::new(content, motion, |_| point(0.5, 0.5)).width(width));
        }
        root
    }
}

fn snackbar_surface(
    id: usize,
    message: &Snackbar,
    motion: OverlayMotion,
    width: gpui::Pixels,
    active: bool,
    window: &Window,
    cx: &mut Context<SnackbarHost>,
) -> impl IntoElement {
    let theme = window_material_theme(window, cx);
    let action_color = color(theme.colors.color(ColorRole::InversePrimary));
    let icon_color = color(theme.colors.color(ColorRole::InverseOnSurface));
    let rtl = theme.layout.direction == Direction::Rtl;
    let spacing = motion.px(16.);
    let control = move |button: gpui_base::Button| {
        button
            .min_h(motion.px(48.))
            .px(spacing * 0.5)
            .rounded(spacing * 0.375)
            .text_color(action_color)
            .font_weight(gpui::FontWeight::MEDIUM)
            .hover(move |style| {
                style.bg(action_color.opacity(gpui_material_theme::StateLayer::Hover.opacity()))
            })
            .active(move |style| {
                style.bg(action_color.opacity(gpui_material_theme::StateLayer::Pressed.opacity()))
            })
            .focus_visible(move |style| style.border(motion.px(2.)).border_color(action_color))
    };
    let passive = move |label: SharedString| {
        div()
            .min_h(motion.px(48.))
            .px(spacing * 0.5)
            .flex()
            .items_center()
            .line_height(gpui::relative(1.))
            .text_color(action_color)
            .font_weight(gpui::FontWeight::MEDIUM)
            .child(label)
    };
    let surface = div()
        .id((if active { "snackbar" } else { "snackbar-exit" }, id))
        .test_support()
        .w(width * motion.scale)
        .opacity(motion.alpha)
        .child(
            Surface::new(ColorRole::InverseSurface)
                .shadow_elevation(Elevation::Level3)
                .shadow_scale(motion.scale)
                .flex()
                .items_center()
                .min_h(motion.px(48.))
                .gap(spacing * 0.5)
                .px(spacing)
                .rounded(spacing * 0.25)
                .when(rtl, Styled::flex_row_reverse)
                .child(
                    div()
                        .id(("snackbar-message", id))
                        .test_support()
                        .when(active, |label| {
                            label
                                .role(Role::Status)
                                .aria_label(message.message.clone())
                                .a11y_synthetic_children(|tree| {
                                    let node = tree.parent_node();
                                    node.set_live(gpui::accesskit::Live::Polite);
                                    node.set_live_atomic();
                                })
                        })
                        .min_w_0()
                        .flex_1()
                        .py(motion.px(14.))
                        .child(message.message.clone()),
                )
                .when_some(message.action.as_ref(), |surface, (label, _)| {
                    if active {
                        surface.child(
                            div().id("snackbar-action-region").test_support().child(
                                control(gpui_base::Button::new("snackbar-action"))
                                    .accessibility_label(label.clone())
                                    .on_click(cx.listener(
                                        move |this, _: &ClickEvent, window, cx| {
                                            this.finish(id, true, window, cx);
                                        },
                                    ))
                                    .child(label.clone()),
                            ),
                        )
                    } else {
                        surface.child(passive(label.clone()))
                    }
                })
                .when_else(
                    active,
                    |surface| {
                        surface.child(
                            control(gpui_base::Button::new("snackbar-dismiss"))
                                .accessibility_label("Dismiss message")
                                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                                    this.finish(id, false, window, cx);
                                }))
                                .child(super::close_icon(icon_color, motion.scale)),
                        )
                    },
                    |surface| {
                        surface.child(
                            div()
                                .size(motion.px(48.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(super::close_icon(icon_color, motion.scale)),
                        )
                    },
                ),
        );
    super::text_style(surface, TypeRole::BodyMedium, motion.scale, window, cx)
}
