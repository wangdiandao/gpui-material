use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Role, ScrollHandle, SharedString, StatefulInteractiveElement,
    Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_material_foundation::{Surface, isolated_scroll, scrim_color, window_material_theme};
use gpui_material_theme::{ColorRole, CornerRole, TypeRole};

use crate::{Button, ButtonVariant, Text};

/// Retains one modal's focus scope and the focus to restore on close.
pub struct DialogState {
    pub(super) focus: FocusHandle,
    previous: Option<FocusHandle>,
    open: bool,
    pub(super) scroll: ScrollHandle,
}

impl DialogState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            previous: None,
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
        if !state.open {
            return div().into_any_element();
        }
        let scroll = state.scroll.clone();
        let full_screen = self.variant == DialogVariant::FullScreen;
        let viewport = window.viewport_size();
        let theme = window_material_theme(window, cx);
        let radius = px(CornerRole::ExtraLarge
            .shape()
            .radii(280., 280., theme.layout.direction)[0]);
        let scrim = scrim_color(theme, 0.32);
        let confirm_action = self.confirm.as_ref().map(|(_, action)| action.clone());
        let close = self.state.clone();
        let click_cancel = close.clone();
        let click_confirm = close.clone();
        gpui_base::Dialog::new(cx)
            .focus_handle(self.state.read(cx).focus.clone())
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
            .popup(
                div().id(self.id).occlude().child(
                    Surface::new(ColorRole::SurfaceContainerHigh)
                        .flex()
                        .flex_col()
                        .gap_4()
                        .p_6()
                        .w(if full_screen {
                            viewport.width
                        } else {
                            px(480.).min(viewport.width - px(24.))
                        })
                        .max_h(viewport.height)
                        .when_else(
                            full_screen,
                            |surface| surface.h(viewport.height),
                            |surface| surface.rounded(radius),
                        )
                        .child(Text::new(self.title).role(TypeRole::HeadlineSmall))
                        .children(self.description.map(|description| {
                            div().id("description").role(Role::Label).child(description)
                        }))
                        .children(
                            self.body
                                .map(|body| isolated_scroll("dialog-content", &scroll).child(body)),
                        )
                        .child(
                            div()
                                .flex()
                                .justify_end()
                                .flex_wrap()
                                .gap_2()
                                .child(
                                    Button::new("cancel", self.cancel)
                                        .variant(ButtonVariant::Text)
                                        .on_activate(move |window, cx| {
                                            click_cancel
                                                .update(cx, |state, cx| state.close(window, cx));
                                        }),
                                )
                                .children(self.confirm.map(|(label, confirm)| {
                                    Button::new("confirm", label).on_activate(move |window, cx| {
                                        if confirm(window, cx) {
                                            click_confirm
                                                .update(cx, |state, cx| state.close(window, cx));
                                        }
                                    })
                                })),
                        ),
                ),
            )
            .into_any_element()
    }
}
