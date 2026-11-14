use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, Context, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Role, ScrollHandle, SharedString, StatefulInteractiveElement,
    Styled, Subscription, WeakFocusHandle, Window, anchored, div, point, prelude::FluentBuilder,
    px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    Surface, color, isolated_scroll, scrim_color, window_material_theme,
};
use gpui_material_theme::{ColorRole, CornerRole, Elevation, TypeRole};

use crate::{Button, ButtonVariant, IconButton, Text};

pub(super) fn exiting_panel<E: StatefulInteractiveElement + ParentElement>(
    panel: E,
    id: &ElementId,
    focus: &FocusHandle,
    window: &mut Window,
    cx: &mut App,
) -> E {
    // The body can contain caller-owned controls. Keep their layout through
    // exit, but send focus back outside the panel and cover its hit targets.
    let guard = window.use_keyed_state((id.clone(), "exiting-focus"), cx, |_, _| {
        let previous = Rc::new(RefCell::new(None::<WeakFocusHandle>));
        (previous, None::<Subscription>)
    });
    let target = window
        .focused(cx)
        .filter(|handle| !focus.contains(handle, window));
    guard.update(cx, |(previous, guard), cx| {
        if let Some(target) = target {
            *previous.borrow_mut() = Some(target.downgrade());
        }
        if guard.is_none() {
            let previous = previous.clone();
            *guard = Some(window.on_focus_in(focus, cx, move |window, cx| {
                if let Some(handle) = previous
                    .borrow()
                    .as_ref()
                    .and_then(WeakFocusHandle::upgrade)
                {
                    handle.focus(window, cx);
                } else {
                    window.blur(cx);
                }
            }));
        }
    });
    panel
        .role(Role::Group)
        .track_focus(focus)
        .capture_key_down(|_, _, cx| cx.stop_propagation())
        .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
        .child(div().absolute().inset_0().occlude())
}

/// Retains one modal's focus scope and the focus to restore on close.
pub struct DialogState {
    pub(super) focus: FocusHandle,
    previous: Option<FocusHandle>,
    cancel_focus: FocusHandle,
    confirm_focus: FocusHandle,
    open: bool,
    pub(super) scroll: ScrollHandle,
}

impl DialogState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            previous: None,
            cancel_focus: cx.focus_handle(),
            confirm_focus: cx.focus_handle(),
            open: false,
            scroll: ScrollHandle::new(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Focuses the modal container. A caller can then focus a child editor.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            return;
        }
        self.previous = window.focused(cx);
        self.open = true;
        self.focus.focus(window, cx);
        cx.notify();
    }

    /// Closes the modal and restores the previous focus if its handle still exists.
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        self.open = false;
        let previous = self.previous.take();
        if self.focus.contains_focused(window, cx) || self.focus.is_focused(window) {
            if let Some(previous) = previous {
                previous.focus(window, cx);
            } else {
                window.blur(cx);
            }
        }
        cx.notify();
    }
}

/// Window-relative presentation for a Material dialog.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DialogVariant {
    #[default]
    Basic,
    /// Uses a full-screen task surface in compact windows, and a basic dialog
    /// at wider breakpoints, as required by Material dialog guidance.
    FullScreen,
}

type Confirm = Rc<dyn Fn(&mut Window, &mut App) -> bool>;

/// A Material modal with a title and optional description, body, and actions.
#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    state: Entity<DialogState>,
    title: SharedString,
    description: Option<SharedString>,
    body: Option<AnyElement>,
    variant: DialogVariant,
    confirm: Option<(SharedString, Confirm)>,
    cancel: SharedString,
    layer: usize,
}

impl Dialog {
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<DialogState>,
        title: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            title: title.into(),
            description: None,
            body: None,
            variant: DialogVariant::Basic,
            confirm: None,
            cancel: "Cancel".into(),
            layer: 0,
        }
    }

    pub fn variant(mut self, variant: DialogVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }

    /// Sets the default action. Returning false keeps the dialog open.
    pub fn confirm(
        mut self,
        label: impl Into<SharedString>,
        action: impl Fn(&mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.confirm = Some((label.into(), Rc::new(action)));
        self
    }

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel = label.into();
        self
    }

    /// Set a higher layer for a nested modal. Only the top modal can receive input.
    pub fn layer(mut self, layer: usize) -> Self {
        self.layer = layer;
        self
    }
}

impl RenderOnce for Dialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state.read(cx);
        let open = state.open;
        let focus = state.focus.clone();
        let scroll = state.scroll.clone();
        let cancel_focus = state.cancel_focus.clone();
        let confirm_focus = state.confirm_focus.clone();
        let presence = crate::motion::presence(&self.id, open, 200, window, cx);
        if !presence.should_render() {
            return div().into_any_element();
        }
        let progress = presence.progress;
        let viewport = window.viewport_size();
        let full_screen = self.variant == DialogVariant::FullScreen && viewport.width < px(600.);
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == gpui_material_theme::Direction::Rtl;
        let radius = px(CornerRole::ExtraLarge
            .shape()
            .radii(280., 280., theme.layout.direction)[0]);
        let scrim = scrim_color(theme, 0.32 * progress);
        let width = if full_screen {
            viewport.width
        } else {
            px(560.).min((viewport.width - px(48.)).max(px(0.)))
        };
        let motion_offset = px((1. - progress) * if full_screen { 32. } else { 8. });
        let confirm_action = self.confirm.as_ref().map(|(_, action)| action.clone());
        let close = self.state.clone();
        let click_cancel = close.clone();
        let click_confirm = close.clone();
        let cancel = move |window: &mut Window, cx: &mut App| {
            if open {
                click_cancel.update(cx, |state, cx| state.close(window, cx));
            }
        };
        let confirm = self.confirm.map(|(label, action)| {
            Button::new("confirm", label)
                .track_focus(&confirm_focus)
                .variant(ButtonVariant::Text)
                .on_activate(move |window, cx| {
                    if open && action(window, cx) {
                        click_confirm.update(cx, |state, cx| state.close(window, cx));
                    }
                })
        });
        let heading = if full_screen {
            Text::new(self.title.clone())
                .role(TypeRole::TitleLarge)
                .into_any_element()
        } else {
            super::text_style(
                div().child(self.title.clone()),
                TypeRole::HeadlineSmall,
                1.,
                window,
                cx,
            )
            .into_any_element()
        };
        let supporting = self.description.map(|description| {
            super::text_style(
                div()
                    .id("description")
                    .role(Role::Label)
                    .text_color(color(theme.colors.color(ColorRole::OnSurfaceVariant)))
                    .child(description),
                TypeRole::BodyMedium,
                1.,
                window,
                cx,
            )
        });
        let mut surface = Surface::new(ColorRole::SurfaceContainerHigh)
            .flex()
            .flex_col()
            .w(width)
            .max_h(if full_screen {
                viewport.height
            } else {
                (viewport.height - px(48.)).max(px(0.))
            });
        if full_screen {
            // Full-screen dialogs are task surfaces with a persistent app bar,
            // not enlarged alert dialogs with bottom-aligned confirmation.
            surface = surface
                .h(viewport.height)
                .child(
                    div()
                        .id("dialog-app-bar")
                        .test_support()
                        .flex()
                        .flex_none()
                        .items_center()
                        .h(px(56.))
                        .px(px(4.))
                        .gap(px(8.))
                        .when(rtl, Styled::flex_row_reverse)
                        .child(
                            IconButton::new(
                                "cancel",
                                self.cancel,
                                crate::Icon::new(crate::IconName::Close),
                            )
                            .track_focus(&cancel_focus)
                            .on_activate(cancel),
                        )
                        .child(div().flex_1().min_w_0().child(heading))
                        .children(confirm),
                )
                .child(
                    isolated_scroll("dialog-content", &scroll)
                        .test_support()
                        .flex_1()
                        .min_h_0()
                        .p(px(24.))
                        .flex()
                        .flex_col()
                        .gap(px(16.))
                        .children(supporting)
                        .children(self.body),
                );
        } else {
            surface = surface
                .rounded(radius)
                .shadow_elevation(Elevation::Level3)
                .p(px(24.))
                .child(div().flex_none().mb(px(16.)).child(heading))
                .child(
                    isolated_scroll("dialog-content", &scroll)
                        .test_support()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .gap(px(16.))
                        .children(supporting)
                        .children(self.body),
                )
                .child(
                    div()
                        .flex()
                        .flex_none()
                        .mt(px(24.))
                        .when(rtl, Styled::flex_row_reverse)
                        .justify_end()
                        .flex_wrap()
                        .gap(px(8.))
                        .child(
                            Button::new("cancel", self.cancel)
                                .track_focus(&cancel_focus)
                                .variant(ButtonVariant::Text)
                                .on_activate(cancel),
                        )
                        .children(confirm),
                );
        }
        let panel = div()
            .id(self.id.clone())
            .test_support()
            .relative()
            .opacity(progress)
            .mt(motion_offset)
            .when(open, |panel| panel.occlude())
            .child(surface);
        if !open {
            return gpui::deferred(
                anchored().position(point(px(0.), px(0.))).child(
                    div()
                        .id((self.id.clone(), "dialog-exit"))
                        .absolute()
                        .top_0()
                        .left_0()
                        .w(viewport.width)
                        .h(viewport.height)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(div().absolute().inset_0().bg(scrim))
                        .child(exiting_panel(panel, &self.id, &focus, window, cx)),
                ),
            )
            .with_priority(10 + self.layer)
            .into_any_element();
        }
        gpui_base::Dialog::new(cx)
            .focus_handle(focus)
            .layer(self.layer, true)
            .accessibility_label(self.title.clone())
            .backdrop(div().absolute().inset_0().bg(scrim))
            .on_ok(move |_, window, cx| {
                confirm_action
                    .as_ref()
                    .is_some_and(|action| action(window, cx))
            })
            .on_open_change(move |open, _, window, cx| {
                if !open {
                    close.update(cx, |state, cx| state.close(window, cx));
                }
            })
            .popup(panel)
            .into_any_element()
    }
}
