use std::{cell::Cell, collections::VecDeque, rc::Rc, time::Duration};

use crate::motion::{OverlayMotion, ScaledOverlay};
use gpui::{
    AnyElement, App, ClickEvent, Context, EntityId, FocusHandle, Hsla, InteractiveElement,
    IntoElement, KeyDownEvent, ParentElement, Render, Role, SharedString,
    StatefulInteractiveElement, Styled, Task, WeakFocusHandle, Window, canvas, div, point,
    prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{color, current_material_theme, elevation_shadows};
use gpui_material_theme::{ColorRole, CornerShape, Direction, Elevation, TypeRole};

type Action = Rc<dyn Fn(&mut Window, &mut App)>;
type Content = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;
type Completion = Rc<dyn Fn(SnackbarResult, &mut Window, &mut App)>;
type HostRenderer = Box<dyn Fn(&Snackbar, SnackbarParts, &mut Window, &mut App) -> AnyElement>;

/// How a current snackbar left its host's queue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnackbarResult {
    ActionPerformed,
    Dismissed,
}

/// An entry in one snackbar host's queue.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SnackbarId {
    host: EntityId,
    sequence: usize,
}

/// Optional per-region colors. Omitted fields use the window's inverse palette.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SnackbarColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub action: Option<Hsla>,
    pub dismiss: Option<Hsla>,
}

/// Host-owned status and activation elements for a custom layout.
#[non_exhaustive]
pub struct SnackbarParts {
    pub message: AnyElement,
    pub action: Option<AnyElement>,
    pub dismiss: Option<AnyElement>,
    /// Scale to apply to dimensions in the custom container.
    pub scale: f32,
}

/// A message for a window's snackbar queue. The default timeout is four seconds.
pub struct Snackbar {
    message: SharedString,
    timeout: Option<Duration>,
    action: Option<(SharedString, Action)>,
    message_content: Option<Content>,
    action_content: Option<Content>,
    dismiss_content: Option<(SharedString, Content)>,
    show_dismiss: bool,
    action_on_new_line: bool,
    shape: CornerShape,
    colors: SnackbarColors,
    on_dismiss: Option<Action>,
    on_result: Option<Completion>,
}

impl Snackbar {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            timeout: Some(Duration::from_secs(4)),
            action: None,
            message_content: None,
            action_content: None,
            dismiss_content: None,
            show_dismiss: true,
            action_on_new_line: false,
            shape: CornerShape::Radii([4.; 4]),
            colors: SnackbarColors::default(),
            on_dismiss: None,
            on_result: None,
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

    /// Returns the plain message used as the status announcement.
    pub fn message(&self) -> &SharedString {
        &self.message
    }

    /// Returns the accessible name of the optional action.
    pub fn action_label(&self) -> Option<&SharedString> {
        self.action.as_ref().map(|(label, _)| label)
    }

    /// Replaces the visible message with repeatable decorative content.
    pub fn message_content<E: IntoElement>(
        mut self,
        content: impl Fn(&mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.message_content = Some(Box::new(move |window, cx| {
            content(window, cx).into_any_element()
        }));
        self
    }

    /// Replaces the visible content of the named action supplied by [`Self::action`].
    pub fn action_content<E: IntoElement>(
        mut self,
        content: impl Fn(&mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.action_content = Some(Box::new(move |window, cx| {
            content(window, cx).into_any_element()
        }));
        self
    }

    /// Supplies the accessible name and decorative content of the close affordance.
    pub fn dismiss_content<E: IntoElement>(
        mut self,
        label: impl Into<SharedString>,
        content: impl Fn(&mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.dismiss_content = Some((
            label.into(),
            Box::new(move |window, cx| content(window, cx).into_any_element()),
        ));
        self
    }

    /// Shows or hides the explicit dismiss button. It is shown by default.
    pub fn show_dismiss(mut self, show: bool) -> Self {
        self.show_dismiss = show;
        self
    }

    /// Places the action below the message for a long action label.
    pub fn action_on_new_line(mut self, enabled: bool) -> Self {
        self.action_on_new_line = enabled;
        self
    }

    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn colors(mut self, colors: SnackbarColors) -> Self {
        self.colors = colors;
        self
    }

    /// Runs on timeout, user dismissal, or explicit dismissal by the caller.
    pub fn on_dismiss(mut self, dismissed: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(dismissed));
        self
    }

    /// Receives one completion after any action or dismissal callback has run.
    pub fn on_result(
        mut self,
        complete: impl Fn(SnackbarResult, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_result = Some(Rc::new(complete));
        self
    }
}

struct Entry {
    id: SnackbarId,
    message: Snackbar,
    controls: [FocusHandle; 2],
    return_focus: Option<WeakFocusHandle>,
}

/// One FIFO message queue per host. Mount the entity in the window layout.
/// A new message leaves keyboard focus wherever it is.
#[derive(Default)]
pub struct SnackbarHost {
    active: Option<Entry>,
    exiting: Vec<Entry>,
    pending: VecDeque<(SnackbarId, Snackbar)>,
    next_id: usize,
    timer: Option<Task<()>>,
    renderer: Option<HostRenderer>,
}

impl SnackbarHost {
    /// Uses a custom container and layout for the host-owned message and controls.
    pub fn renderer<E: IntoElement>(
        mut self,
        render: impl Fn(&Snackbar, SnackbarParts, &mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.renderer = Some(Box::new(move |message, parts, window, cx| {
            render(message, parts, window, cx).into_any_element()
        }));
        self
    }

    /// Queues the message after any already pending messages.
    pub fn push(&mut self, snackbar: Snackbar, window: &mut Window, cx: &mut Context<Self>) {
        self.enqueue(snackbar, window, cx);
    }

    /// Queues the message and returns an opaque identity scoped to this host.
    pub fn enqueue(
        &mut self,
        snackbar: Snackbar,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SnackbarId {
        self.next_id += 1;
        let id = SnackbarId {
            host: cx.entity_id(),
            sequence: self.next_id,
        };
        self.pending.push_back((id, snackbar));
        if self.active.is_none() {
            self.show_next(window, cx);
        }
        id
    }

    /// Returns the active entry. Pending and exiting entries are excluded.
    pub fn current(&self) -> Option<(SnackbarId, &Snackbar)> {
        self.active.as_ref().map(|entry| (entry.id, &entry.message))
    }

    /// Dismisses the active message. If any are queued, the next begins its own timeout.
    pub fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((id, _)) = self.current() {
            self.dismiss_message(id, window, cx);
        }
    }

    /// Dismisses the entry if it is still current.
    pub fn dismiss_message(
        &mut self,
        id: SnackbarId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.finish(id, SnackbarResult::Dismissed, window, cx)
    }

    /// Performs the entry's action if it is current and has an action.
    pub fn perform_action(
        &mut self,
        id: SnackbarId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.finish(id, SnackbarResult::ActionPerformed, window, cx)
    }

    fn show_next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active = self.pending.pop_front().map(|(id, message)| Entry {
            id,
            message,
            controls: [cx.focus_handle(), cx.focus_handle()],
            return_focus: window.focused(cx).map(|focus| focus.downgrade()),
        });
        self.timer = self.active.as_ref().and_then(|entry| {
            let id = entry.id;
            let timeout = entry.message.timeout?;
            Some(cx.spawn_in(window, async move |this, cx| {
                cx.background_executor().timer(timeout).await;
                let _ = this.update_in(cx, |this, window, cx| {
                    this.dismiss_message(id, window, cx);
                });
            }))
        });
        cx.notify();
    }

    fn finish(
        &mut self,
        id: SnackbarId,
        result: SnackbarResult,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(active) = self.active.as_ref().filter(|entry| entry.id == id) else {
            return false;
        };
        let callback = match result {
            SnackbarResult::ActionPerformed => {
                let Some((_, callback)) = &active.message.action else {
                    return false;
                };
                Some(callback.clone())
            }
            SnackbarResult::Dismissed => active.message.on_dismiss.clone(),
        };
        let mut entry = self.active.take().expect("current entry checked above");
        let complete = entry.message.on_result.take();
        if entry.controls.iter().any(|focus| focus.is_focused(window)) {
            if let Some(restore) = entry.return_focus.take().and_then(|focus| focus.upgrade()) {
                restore.focus(window, cx);
            } else {
                window.blur(cx);
            }
        }
        self.exiting.push(entry);
        self.show_next(window, cx);
        if callback.is_some() || complete.is_some() {
            window.defer(cx, move |window, cx| {
                if let Some(callback) = callback {
                    callback(window, cx);
                }
                if let Some(complete) = complete {
                    complete(result, window, cx);
                }
            });
        }
        true
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
        let renderer = self.renderer.as_ref();
        self.exiting.retain(|entry| {
            let Some(motion) =
                OverlayMotion::sample(&("snackbar", entry.id.sequence).into(), false, window, cx)
            else {
                return false;
            };
            let content = snackbar_surface(entry, motion, width, false, renderer, window, cx);
            let scaled = ScaledOverlay::new(content, motion, |_| point(0.5, 0.5)).width(width);
            let slot = div().when(has_layout, |div| div.absolute().inset_0());
            root.extend([slot.child(scaled).into_any_element()]);
            has_layout = true;
            true
        });
        if let Some(entry) = &self.active {
            let motion =
                OverlayMotion::sample(&("snackbar", entry.id.sequence).into(), true, window, cx)
                    .expect("active snackbar has motion");
            let content = snackbar_surface(entry, motion, width, true, renderer, window, cx);
            root =
                root.child(ScaledOverlay::new(content, motion, |_| point(0.5, 0.5)).width(width));
        }
        root
    }
}

fn snackbar_surface(
    entry: &Entry,
    motion: OverlayMotion,
    width: gpui::Pixels,
    active: bool,
    renderer: Option<&HostRenderer>,
    window: &mut Window,
    cx: &mut Context<SnackbarHost>,
) -> impl IntoElement {
    let id = entry.id;
    let serial = id.sequence;
    let message = &entry.message;
    let theme = current_material_theme(window, cx);
    let tone = |role| color(theme.colors.color(role));
    let inverse_primary = tone(ColorRole::InversePrimary);
    let inverse_on_surface = tone(ColorRole::InverseOnSurface);
    let content_color = message.colors.content.unwrap_or(inverse_on_surface);
    let action_color = message.colors.action.unwrap_or(inverse_primary);
    let dismiss_color = message.colors.dismiss.unwrap_or(inverse_on_surface);
    let dismiss_feedback = message.colors.dismiss.unwrap_or(inverse_primary);
    let spacing = motion.px(16.);
    let message_content = message.message_content.as_ref().map_or_else(
        || message.message.clone().into_any_element(),
        |content| content(window, cx),
    );
    let message_part = div()
        .id(("snackbar-message", serial))
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
        .text_color(content_color)
        .child(message_content)
        .into_any_element();

    let render_control = |index,
                          id_name,
                          label,
                          content,
                          foreground: Hsla,
                          feedback: Hsla,
                          result,
                          cx: &Context<SnackbarHost>| {
        if active {
            gpui_base::Button::new(id_name)
                .min_h(motion.px(48.))
                .px(spacing * 0.5)
                .rounded(spacing * 0.375)
                .text_color(foreground)
                .font_weight(gpui::FontWeight::MEDIUM)
                .hover(move |style| {
                    style.bg(feedback.opacity(gpui_material_theme::StateLayer::Hover.opacity()))
                })
                .active(move |style| {
                    style.bg(feedback.opacity(gpui_material_theme::StateLayer::Pressed.opacity()))
                })
                .focus_visible(move |style| style.border(motion.px(2.)).border_color(feedback))
                .track_focus(&entry.controls[index])
                .accessibility_label(label)
                .on_click(cx.listener(move |host, _: &ClickEvent, window, cx| {
                    host.finish(id, result, window, cx);
                }))
                .child(content)
                .into_any_element()
        } else {
            div()
                .min_h(motion.px(48.))
                .px(spacing * 0.5)
                .flex()
                .items_center()
                .line_height(gpui::relative(1.))
                .text_color(foreground)
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(content)
                .into_any_element()
        }
    };
    let action_part = message.action_label().map(|label| {
        let content = message.action_content.as_ref().map_or_else(
            || label.clone().into_any_element(),
            |content| content(window, cx),
        );
        let control = render_control(
            0,
            "snackbar-action",
            label.clone(),
            content,
            action_color,
            action_color,
            SnackbarResult::ActionPerformed,
            cx,
        );
        if active {
            div()
                .id("snackbar-action-region")
                .test_support()
                .child(control)
                .into_any_element()
        } else {
            control
        }
    });
    let dismiss_part = message.show_dismiss.then(|| {
        let (label, content) = message.dismiss_content.as_ref().map_or_else(
            || {
                (
                    "Dismiss message".into(),
                    crate::Icon::new(crate::IconName::Close)
                        .size(motion.px(24.))
                        .into_any_element(),
                )
            },
            |(label, content)| (label.clone(), content(window, cx)),
        );
        render_control(
            1,
            "snackbar-dismiss",
            label,
            content,
            dismiss_color,
            dismiss_feedback,
            SnackbarResult::Dismissed,
            cx,
        )
    });
    let parts = SnackbarParts {
        message: message_part,
        action: action_part,
        dismiss: dismiss_part,
        scale: motion.scale,
    };
    let rendered = match renderer {
        Some(render) => render(message, parts, window, cx),
        None => default_layout(message, parts, motion, window, cx).into_any_element(),
    };
    let surface = div()
        .id((if active { "snackbar" } else { "snackbar-exit" }, serial))
        .test_support()
        .w(width * motion.scale)
        .opacity(motion.alpha)
        .when(active, |surface| {
            surface.capture_key_down(cx.listener(move |host, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" && host.dismiss_message(id, window, cx) {
                    cx.stop_propagation();
                }
            }))
        })
        .child(rendered);
    super::text_style(surface, TypeRole::BodyMedium, motion.scale, window, cx)
}

fn default_layout(
    message: &Snackbar,
    parts: SnackbarParts,
    motion: OverlayMotion,
    window: &Window,
    cx: &App,
) -> gpui::Div {
    let SnackbarParts {
        message: content,
        action,
        dismiss,
        ..
    } = parts;
    let theme = current_material_theme(window, cx);
    let direction = theme.layout.direction;
    let rtl = direction == Direction::Rtl;
    let tone = |role| color(theme.colors.color(role));
    let spacing = motion.px(16.);
    let mut surface = div()
        .bg(message
            .colors
            .container
            .unwrap_or_else(|| tone(ColorRole::InverseSurface)))
        .shadow(elevation_shadows(
            Elevation::Level3.dp(),
            motion.scale,
            tone(ColorRole::Shadow),
        ))
        .flex()
        .min_h(motion.px(48.))
        .gap(spacing * 0.5)
        .px(spacing)
        .when(rtl, Styled::flex_row_reverse);
    if message.shape == CornerShape::Full {
        surface = surface.rounded_full();
    } else {
        // GPUI further clamps these radii against the actual laid-out quad.
        let [top_left, top_right, bottom_right, bottom_left] =
            message.shape.radii(f32::MAX / 2., f32::MAX / 2., direction);
        let radii = &mut surface.style().corner_radii;
        radii.top_left = Some(motion.px(top_left).into());
        radii.top_right = Some(motion.px(top_right).into());
        radii.bottom_right = Some(motion.px(bottom_right).into());
        radii.bottom_left = Some(motion.px(bottom_left).into());
    }
    match (message.action_on_new_line, action) {
        (true, Some(action)) => surface
            .flex_col()
            .items_stretch()
            .gap_0()
            .child(content)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(spacing * 0.5)
                    .when(rtl, Styled::flex_row_reverse)
                    .child(action)
                    .children(dismiss),
            ),
        (_, action) => surface
            .items_center()
            .child(content)
            .children(action)
            .children(dismiss),
    }
}
