use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, AppContext, Context, ElementId, Entity, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, Render, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{
    DragCapture, Surface, color, isolated_scroll, scrim_color, window_material_theme,
};
use gpui_material_theme::{ColorRole, Direction, TypeRole};

use crate::{Button, ButtonVariant, Text};

use super::dialog::DialogState;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SheetEdge {
    #[default]
    Bottom,
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SheetMode {
    #[default]
    Standard,
    Modal,
}

type Change = Rc<dyn Fn(bool, &mut Window, &mut App)>;

struct SheetDrag;

impl Render for SheetDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Default)]
struct Gesture {
    origin: Option<Pixels>,
    capture: Option<DragCapture>,
}

fn drag_handle(
    id: &ElementId,
    expanded: bool,
    change: Change,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let tint = color(
        window_material_theme(window, cx)
            .colors
            .color(ColorRole::OnSurfaceVariant),
    );
    let gesture = window
        .use_keyed_state((id.clone(), "drag"), cx, |_, _| {
            Rc::new(RefCell::new(Gesture::default()))
        })
        .read(cx)
        .clone();
    let focus = window
        .use_keyed_state((id.clone(), "drag-focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let down = gesture.clone();
    let start = gesture.clone();
    let release = gesture.clone();
    let outside = gesture.clone();
    let cancel = gesture;
    let drag_focus = focus.clone();
    let release_change = change.clone();
    let settle = move |gesture: &RefCell<Gesture>,
                       y: Pixels,
                       change: &Change,
                       window: &mut Window,
                       cx: &mut App| {
        let active = std::mem::take(&mut *gesture.borrow_mut());
        if active.capture.is_none() {
            return;
        }
        if let Some(origin) = active.origin {
            let distance = y - origin;
            if distance.abs() >= px(32.) && expanded == (distance > px(0.)) {
                change(!expanded, window, cx);
            }
        }
    };
    div()
        .id("drag-handle")
        .test_support()
        .track_focus(&focus)
        .w_full()
        .h(px(24.))
        .flex()
        .items_center()
        .justify_center()
        .cursor_ns_resize()
        .child(
            div()
                .w(px(32.))
                .h(px(4.))
                .rounded_full()
                .bg(tint.opacity(0.55)),
        )
        .on_mouse_down(MouseButton::Left, move |event, _, _| {
            down.borrow_mut().origin = Some(event.position.y);
        })
        .on_drag(SheetDrag, move |_, _, window, cx| {
            drag_focus.focus(window, cx);
            let weak = Rc::downgrade(&start);
            start.borrow_mut().capture = Some(DragCapture::watch(window, cx, move |_, _| {
                if let Some(state) = weak.upgrade() {
                    *state.borrow_mut() = Gesture::default();
                }
            }));
            cx.stop_propagation();
            cx.new(|_| SheetDrag)
        })
        .on_mouse_up(MouseButton::Left, move |event, window, cx| {
            settle(&release, event.position.y, &release_change, window, cx);
        })
        .on_mouse_up_out(MouseButton::Left, move |event, window, cx| {
            settle(&outside, event.position.y, &change, window, cx);
        })
        .capture_key_down(move |event, window, cx| {
            if event.keystroke.key == "escape"
                && let Some(capture) = std::mem::take(&mut *cancel.borrow_mut()).capture
            {
                capture.cancel(window, cx);
                cx.stop_propagation();
            }
        })
        .into_any_element()
}

/// A docked or modal Material sheet. Standard sheets take part in parent layout.
/// The caller owns the bottom sheet's expanded value and handles changes.
#[derive(IntoElement)]
pub struct Sheet {
    id: ElementId,
    state: Entity<DialogState>,
    title: SharedString,
    content: AnyElement,
    edge: SheetEdge,
    mode: SheetMode,
    extent: Pixels,
    expansion: Option<(bool, Change)>,
}

impl Sheet {
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<DialogState>,
        title: impl Into<SharedString>,
        content: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            title: title.into(),
            content: content.into_any_element(),
            edge: SheetEdge::Bottom,
            mode: SheetMode::Standard,
            extent: px(320.),
            expansion: None,
        }
    }

    pub fn edge(mut self, edge: SheetEdge) -> Self {
        self.edge = edge;
        self
    }

    pub fn mode(mut self, mode: SheetMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets expanded height for a bottom sheet, or width for a side sheet.
    pub fn extent(mut self, extent: Pixels) -> Self {
        self.extent = extent.max(px(48.));
        self
    }

    /// Enables a keyboard-operable expand and collapse action on a bottom sheet.
    pub fn expandable(
        mut self,
        expanded: bool,
        on_change: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.expansion = Some((expanded, Rc::new(on_change)));
        self
    }
}

impl RenderOnce for Sheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let modal = self.mode == SheetMode::Modal;
        if modal && !self.state.read(cx).is_open() {
            return div().into_any_element();
        }
        let viewport = window.viewport_size();
        let theme = window_material_theme(window, cx);
        let left = match self.edge {
            SheetEdge::Start => theme.layout.direction == Direction::Ltr,
            SheetEdge::End => theme.layout.direction == Direction::Rtl,
            SheetEdge::Bottom => false,
        };
        let bottom = self.edge == SheetEdge::Bottom;
        let scrim = scrim_color(theme, 0.32);
        let expanded = !bottom
            || self
                .expansion
                .as_ref()
                .is_none_or(|(expanded, _)| *expanded);
        let close = self.state.clone();
        let focus = self.state.read(cx).focus.clone();
        let scroll = self.state.read(cx).scroll.clone();
        let close_button = close.clone();
        let handle = self
            .expansion
            .as_ref()
            .filter(|_| bottom)
            .map(|(expanded, change)| drag_handle(&self.id, *expanded, change.clone(), window, cx));
        let panel = div()
            .id(self.id)
            .test_support()
            .role(if modal { Role::Dialog } else { Role::Group })
            .aria_label(self.title.clone())
            .occlude()
            .when(modal, |panel| {
                panel.absolute().when_else(
                    bottom,
                    |panel| panel.bottom_0().left_0().right_0(),
                    |panel| {
                        panel
                            .top_0()
                            .bottom_0()
                            .when_else(left, Styled::left_0, Styled::right_0)
                    },
                )
            })
            .child(
                Surface::new(ColorRole::SurfaceContainerLow)
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .children(handle)
                    .when_else(
                        bottom,
                        |surface| {
                            surface.w_full().when(expanded, |surface| {
                                surface.h(self.extent.min(viewport.height))
                            })
                        },
                        |surface| surface.w(self.extent.min(viewport.width)).h_full(),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .flex_wrap()
                            .child(Text::new(self.title).role(TypeRole::TitleMedium))
                            .when_some(
                                self.expansion.filter(|_| bottom),
                                |header, (expanded, change)| {
                                    header.child(
                                        Button::new(
                                            "expand",
                                            if expanded { "Collapse" } else { "Expand" },
                                        )
                                        .variant(ButtonVariant::Text)
                                        .on_activate(
                                            move |window, cx| change(!expanded, window, cx),
                                        ),
                                    )
                                },
                            )
                            .when(modal, |header| {
                                header.child(
                                    Button::new("close", "Close")
                                        .variant(ButtonVariant::Text)
                                        .on_activate(move |window, cx| {
                                            close_button
                                                .update(cx, |state, cx| state.close(window, cx))
                                        }),
                                )
                            }),
                    )
                    .when(expanded, |surface| {
                        surface.child(
                            isolated_scroll("sheet-content", &scroll)
                                .flex_1()
                                .child(self.content),
                        )
                    }),
            );
        if modal {
            gpui::deferred(
                gpui_base::Sheet::new(cx)
                    .focus_handle(focus)
                    .overlay(div().absolute().inset_0().bg(scrim))
                    .surface(panel)
                    .request_close(move |window, cx| {
                        close.update(cx, |state, cx| state.close(window, cx));
                    }),
            )
            .with_priority(9)
            .into_any_element()
        } else {
            panel.into_any_element()
        }
    }
}
