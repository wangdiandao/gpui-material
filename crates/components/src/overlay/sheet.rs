use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, AppContext, Context, DragMoveEvent, ElementId, Entity, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Pixels, Render, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, Window, anchored, div, point, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{
    DragCapture, Surface, animate, color, isolated_scroll, scrim_color, window_material_theme,
};
use gpui_material_theme::{
    ColorRole, Direction, Elevation, MotionKind, MotionSpec, MotionSpeed, TypeRole,
};

use crate::{Button, ButtonVariant, Text};

use super::dialog::{DialogState, exiting_panel};

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
    offset: Pixels,
}

fn drag_handle(
    id: &ElementId,
    gesture: Rc<RefCell<Gesture>>,
    travel: Pixels,
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
    let focus = window
        .use_keyed_state((id.clone(), "drag-focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let down = gesture.clone();
    let start = gesture.clone();
    let release = gesture.clone();
    let outside = gesture.clone();
    let moving = gesture.clone();
    let cancel = gesture;
    let drag_focus = focus.clone();
    let release_change = change.clone();
    let settle = move |gesture: &RefCell<Gesture>,
                       y: Pixels,
                       change: &Change,
                       window: &mut Window,
                       cx: &mut App| {
        let active = std::mem::take(&mut *gesture.borrow_mut());
        window.refresh();
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
        .h(px(48.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .cursor_ns_resize()
        .child(drag_handle_mark(tint))
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
        .on_drag_move(move |event: &DragMoveEvent<SheetDrag>, window, _| {
            let mut state = moving.borrow_mut();
            if state.capture.is_some()
                && let Some(origin) = state.origin
            {
                let offset = event.event.position.y - origin;
                state.offset = if expanded {
                    offset.clamp(px(0.), travel)
                } else {
                    offset.clamp(-travel, px(0.))
                };
                window.refresh();
            }
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
                window.refresh();
                cx.stop_propagation();
            }
        })
        .into_any_element()
}

fn drag_handle_mark(tint: gpui::Hsla) -> gpui::Div {
    div().w(px(32.)).h(px(4.)).rounded_full().bg(tint)
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
        let open = !modal || self.state.read(cx).is_open();
        let presence = modal.then(|| crate::motion::presence(&self.id, open, 240, window, cx));
        if presence.is_some_and(|presence| !presence.should_render()) {
            return div().into_any_element();
        }
        let visibility = presence.map_or(1., |presence| presence.progress);
        let viewport = window.viewport_size();
        let corner = px(28.);
        let bottom_width = viewport.width.min(px(640.));
        let bottom_left = (viewport.width - bottom_width) / 2.;
        let theme = window_material_theme(window, cx).clone();
        let left = match self.edge {
            SheetEdge::Start => theme.layout.direction == Direction::Ltr,
            SheetEdge::End => theme.layout.direction == Direction::Rtl,
            SheetEdge::Bottom => false,
        };
        let bottom = self.edge == SheetEdge::Bottom;
        let scrim = scrim_color(&theme, 0.32 * visibility);
        let extent = if !bottom && !modal {
            px(animate(
                (self.id.clone(), "side-sheet-width"),
                f32::from(self.extent),
                theme.motion.spec(
                    MotionKind::Spatial,
                    MotionSpeed::Default,
                    cx.reduce_motion(),
                ),
                window,
                cx,
            ))
            .min(viewport.width)
        } else {
            self.extent.min(if bottom {
                viewport.height
            } else {
                viewport.width
            })
        };

        let expanded = !bottom
            || self
                .expansion
                .as_ref()
                .is_none_or(|(expanded, _)| *expanded);
        let close = self.state.clone();
        let focus = self.state.read(cx).focus.clone();
        let scroll = self.state.read(cx).scroll.clone();
        let close_button = close.clone();
        let gesture = window
            .use_keyed_state((self.id.clone(), "drag"), cx, |_, _| {
                Rc::new(RefCell::new(Gesture::default()))
            })
            .read(cx)
            .clone();
        // The 48dp handle and 48dp header actions must remain operable at rest.
        let collapsed = px(48. + 8. + 48. + 16.).min(extent);
        let travel = extent - collapsed;
        let height = if self.expansion.is_some() && bottom {
            let drag = gesture.borrow();
            let tracked = drag.capture.is_some();
            let target = if expanded { extent } else { collapsed } - drag.offset;
            let policy = if tracked {
                MotionSpec::Instant
            } else {
                theme.motion.spec(
                    MotionKind::Spatial,
                    MotionSpeed::Default,
                    cx.reduce_motion(),
                )
            };
            px(animate(
                (self.id.clone(), "sheet-height"),
                f32::from(target),
                policy,
                window,
                cx,
            ))
            .clamp(collapsed, extent)
        } else {
            extent
        };
        let offscreen = -(if bottom { height } else { extent }) * (1. - visibility);
        let handle = self
            .expansion
            .as_ref()
            .filter(|_| bottom)
            .map(|(expanded, change)| {
                if open {
                    drag_handle(
                        &self.id,
                        gesture,
                        travel,
                        *expanded,
                        change.clone(),
                        window,
                        cx,
                    )
                } else {
                    let tint = color(theme.colors.color(ColorRole::OnSurfaceVariant));
                    div()
                        .w_full()
                        .h(px(48.))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(drag_handle_mark(tint))
                        .into_any_element()
                }
            });
        let has_handle = handle.is_some();
        let panel =
            div()
                .id(self.id.clone())
                .test_support()
                .when(open, |panel| {
                    panel
                        .role(if modal { Role::Dialog } else { Role::Group })
                        .aria_label(self.title.clone())
                        .occlude()
                })
                .when(bottom && !modal, |panel| {
                    panel.w_full().max_w(px(640.)).mx_auto()
                })
                .when(modal, |panel| {
                    panel.absolute().when_else(
                        bottom,
                        |panel| panel.bottom(offscreen).left(bottom_left).w(bottom_width),
                        |panel| {
                            panel.top_0().bottom_0().when_else(
                                left,
                                |panel| panel.left(offscreen),
                                |panel| panel.right(offscreen),
                            )
                        },
                    )
                })
                .child(
                    Surface::new(ColorRole::SurfaceContainerLow)
                        .shadow_elevation(Elevation::Level1)
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .px(px(16.))
                        .pb(px(16.))
                        .when(!has_handle, |surface| surface.pt(px(16.)))
                        .children(handle)
                        .when_else(
                            bottom,
                            |surface| {
                                surface
                                    .w_full()
                                    .h(height)
                                    .overflow_hidden()
                                    .rounded_tl(corner)
                                    .rounded_tr(corner)
                            },
                            |surface| surface.w(extent).h_full(),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_none()
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
                                            .on_activate(move |window, cx| {
                                                if open {
                                                    change(!expanded, window, cx);
                                                }
                                            }),
                                        )
                                    },
                                )
                                .when(modal, |header| {
                                    header.child(
                                        Button::new("close", "Close")
                                            .variant(ButtonVariant::Text)
                                            .on_activate(move |window, cx| {
                                                if open {
                                                    close_button.update(cx, |state, cx| {
                                                        state.close(window, cx)
                                                    });
                                                }
                                            }),
                                    )
                                }),
                        )
                        .when(expanded || height > collapsed, |surface| {
                            surface.child(
                                isolated_scroll("sheet-content", &scroll)
                                    .test_support()
                                    .flex_1()
                                    .child(self.content),
                            )
                        }),
                );
        if !open {
            gpui::deferred(
                anchored().position(point(px(0.), px(0.))).child(
                    div()
                        .id((self.id.clone(), "sheet-exit"))
                        .absolute()
                        .top_0()
                        .left_0()
                        .w(viewport.width)
                        .h(viewport.height)
                        .overflow_hidden()
                        .child(div().absolute().inset_0().bg(scrim))
                        .child(exiting_panel(panel, &self.id, &focus, window, cx)),
                ),
            )
            .with_priority(9)
            .into_any_element()
        } else if modal {
            gpui::deferred(
                gpui_base::Sheet::new(cx)
                    .focus_handle(focus)
                    .overlay(div().absolute().inset_0().bg(scrim))
                    .surface(div().absolute().inset_0().overflow_hidden().child(panel))
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
