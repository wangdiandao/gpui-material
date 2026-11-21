use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AnyWindowHandle, AppContext, Bounds, Context, Entity, FocusHandle, InputEvent,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement,
    Pixels, Point, Render, Role, ScrollDelta, ScrollEvent, ScrollHandle,
    StatefulInteractiveElement, Styled, TestAppContext, Window, WindowOptions, div, point,
    prelude::FluentBuilder, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    Button, DialogCloseReason as Reason, DialogProperties, DialogState, MaterialRoot, Sheet,
    SheetEdge, SheetMargins, SheetMode, SheetPlacement,
    foundation::{set_window_material_theme, window_material_theme},
    theme::Direction,
};

#[derive(Default)]
struct ProbeState {
    mode: Cell<SheetMode>,
    edge: Cell<SheetEdge>,
    placement: Cell<SheetPlacement>,
    direction: Cell<Direction>,
    properties: Cell<DialogProperties>,
    allow_close: Cell<bool>,
    underlay_clicks: Cell<usize>,
    back_clicks: Cell<usize>,
    footer_clicks: Cell<usize>,
    requests: RefCell<Vec<Reason>>,
    closes: RefCell<Vec<Reason>>,
}

struct Probe {
    shared: Rc<ProbeState>,
    state: Entity<DialogState>,
    opener: FocusHandle,
    underlay: ScrollHandle,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let clicked = self.shared.clone();
        let back = self.shared.clone();
        let footer = self.shared.clone();
        let request = self.shared.clone();
        let closed = self.shared.clone();
        let standard = self.shared.mode.get() == SheetMode::Standard;
        let is_start = self.shared.edge.get() == SheetEdge::Start;
        let left = is_start != (self.shared.direction.get() == Direction::Rtl);
        div()
            .size_full()
            .flex()
            .when(standard && left, Styled::flex_row_reverse)
            .child(
                div()
                    .id("underlay-scroll")
                    .h_full()
                    .min_w_0()
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&self.underlay)
                    .child(
                        Button::new("side-opener", "Open inspection")
                            .track_focus(&self.opener)
                            .on_activate(move |_, _| {
                                clicked
                                    .underlay_clicks
                                    .set(clicked.underlay_clicks.get() + 1)
                            }),
                    )
                    .child(div().h(px(1000.))),
            )
            .child(
                Sheet::new(
                    "sheet-probe",
                    &self.state,
                    "Inspect workspace",
                    div().flex().flex_col().children(
                        (0..20).map(|index| {
                            div().h(px(60.)).flex_none().child(format!("Entry {index}"))
                        }),
                    ),
                )
                .edge(self.shared.edge.get())
                .mode(self.shared.mode.get())
                .extent(px(245.))
                .placement(self.shared.placement.get())
                .margins(SheetMargins {
                    top: px(11.),
                    bottom: px(19.),
                    start: px(13.),
                    end: px(27.),
                })
                .header_content(div().child("Inspect filters"))
                .back("Back to inspection", move |_, _| {
                    back.back_clicks.set(back.back_clicks.get() + 1)
                })
                .close_content(|close| {
                    Button::new("close-sheet-probe", "Close inspection")
                        .on_activate(move |window, cx| close.activate(window, cx))
                })
                .footer(
                    Button::new("keep-settings", "Keep settings").on_activate(move |_, _| {
                        footer.footer_clicks.set(footer.footer_clicks.get() + 1)
                    }),
                )
                .footer_divider(true)
                .edge_divider(true)
                .scrim_opacity(0.)
                .properties(self.shared.properties.get())
                .on_dismiss_request(move |reason, _, _| {
                    request.requests.borrow_mut().push(reason);
                    request.allow_close.get()
                })
                .on_close(move |reason, _, _| closed.closes.borrow_mut().push(reason)),
            )
    }
}

struct Harness {
    window: AnyWindowHandle,
    view: Entity<Probe>,
    shared: Rc<ProbeState>,
    state: Entity<DialogState>,
    opener: FocusHandle,
    underlay: ScrollHandle,
}

impl Harness {
    fn new(cx: &mut TestAppContext) -> Self {
        let this = cx.update(|cx| {
            crate::init_gallery(cx);
            cx.set_reduce_motion(true);
            let shared = Rc::new(ProbeState::default());
            shared.edge.set(SheetEdge::End);
            let state = cx.new(DialogState::new);
            let opener = cx.focus_handle();
            let underlay = ScrollHandle::new();
            let view = cx.new(|cx| {
                cx.observe(&state, |_, _, cx| cx.notify()).detach();
                Probe {
                    shared: shared.clone(),
                    state: state.clone(),
                    opener: opener.clone(),
                    underlay: underlay.clone(),
                }
            });
            let window =
                MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone())
                    .unwrap()
                    .into();
            Self {
                window,
                view,
                shared,
                state,
                opener,
                underlay,
            }
        });
        cx.simulate_window_resize(this.window, size(px(460.), px(382.)));
        this.frame(cx);
        this
    }

    fn frame(&self, cx: &mut TestAppContext) {
        cx.update_window(self.window, |_, window, cx| {
            let direction = self.shared.direction.get();
            if window_material_theme(window, cx).layout.direction != direction {
                let mut theme = window_material_theme(window, cx).clone();
                theme.layout.direction = direction;
                set_window_material_theme(Some(theme), window, cx);
            }
            self.view.update(cx, |_, cx| cx.notify());
            window.refresh();
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }

    fn open(&self, cx: &mut TestAppContext) {
        cx.update_window(self.window, |_, window, cx| {
            self.opener.focus(window, cx);
            self.state.update(cx, |state, cx| state.open(window, cx));
        })
        .unwrap();
        self.frame(cx);
    }

    fn bounds(&self, id: &'static str, cx: &mut TestAppContext) -> Bounds<Pixels> {
        cx.update_window(self.window, |_, window, _| {
            find(window, &[], &id.into()).unwrap().bounds()
        })
        .unwrap()
    }

    fn click(&self, position: Point<Pixels>, cx: &mut TestAppContext) {
        cx.update_window(self.window, |_, window, cx| {
            window.dispatch_event(
                MouseDownEvent {
                    position,
                    button: MouseButton::Left,
                    click_count: 1,
                    ..MouseDownEvent::default()
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseUpEvent {
                    position,
                    button: MouseButton::Left,
                    click_count: 1,
                    ..MouseUpEvent::default()
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
        self.frame(cx);
    }

    fn scroll(&self, position: Point<Pixels>, cx: &mut TestAppContext) {
        cx.update_window(self.window, |_, window, cx| {
            window.dispatch_event(
                ScrollEvent {
                    position,
                    delta: ScrollDelta::Lines(point(0., -3.)),
                    ..ScrollEvent::default()
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
        self.frame(cx);
    }
}

#[gpui::test]
fn side_sheet_places_docked_and_detached_panels_on_each_logical_edge(cx: &mut TestAppContext) {
    let probe = Harness::new(cx);
    cx.simulate_window_resize(probe.window, size(px(286.), px(290.)));
    for mode in [SheetMode::Standard, SheetMode::Modal] {
        probe.shared.mode.set(mode);
        if mode == SheetMode::Modal {
            probe.open(cx);
        }
        for side in [Direction::Ltr, Direction::Rtl] {
            probe.shared.direction.set(side);
            for edge in [SheetEdge::Start, SheetEdge::End] {
                probe.shared.edge.set(edge);
                let left = (edge == SheetEdge::Start) == (side == Direction::Ltr);
                for placement in [SheetPlacement::Docked, SheetPlacement::Detached] {
                    probe.shared.placement.set(placement);
                    probe.frame(cx);
                    let bounds = probe.bounds("sheet-probe", cx);
                    let detached = placement == SheetPlacement::Detached;
                    let edge_margin = if !detached {
                        px(0.)
                    } else if edge == SheetEdge::Start {
                        px(13.)
                    } else {
                        px(27.)
                    };
                    let top = if detached { px(11.) } else { px(0.) };
                    let bottom = if detached { px(271.) } else { px(290.) };
                    assert!(
                        (bounds.top() - top).abs() <= px(1.),
                        "{mode:?} {edge:?} {side:?} {placement:?} {bounds:?}"
                    );
                    assert!((bounds.bottom() - bottom).abs() <= px(1.), "{bounds:?}");
                    assert!(bounds.size.width > px(0.) && bounds.size.width <= px(245.));
                    let actual = if left {
                        bounds.left()
                    } else {
                        px(286.) - bounds.right()
                    };
                    assert!(
                        (actual - edge_margin).abs() <= px(1.),
                        "{mode:?} {edge:?} {side:?} {placement:?} {bounds:?}"
                    );
                }
            }
        }
    }
}

#[gpui::test]
fn side_sheet_retains_fixed_actions_and_leaves_standard_underlay_interactive(
    cx: &mut TestAppContext,
) {
    let probe = Harness::new(cx);
    let opener = probe.bounds("side-opener", cx).center();
    probe.click(opener, cx);
    probe.scroll(opener, cx);
    assert_eq!(probe.shared.underlay_clicks.get(), 1);
    assert!(probe.underlay.offset().y < px(0.));
    let before = probe.bounds("sheet-footer", cx);
    let content = probe.bounds("sheet-content", cx);
    assert!(content.bottom() <= before.top());
    assert!(probe.bounds("sheet-footer-divider", cx).size.height > px(0.));
    assert!(probe.bounds("sheet-edge-divider", cx).size.width > px(0.));
    let scroll_at_entry = probe.underlay.offset();
    probe.scroll(content.center(), cx);
    assert_eq!(probe.underlay.offset(), scroll_at_entry);
    assert_eq!(probe.bounds("sheet-footer", cx), before);
    let footer_action = probe.bounds("keep-settings", cx).center();
    probe.click(footer_action, cx);
    assert_eq!(probe.shared.footer_clicks.get(), 1);
    let close = probe.bounds("close-sheet-probe", cx).center();
    probe.shared.allow_close.set(true);
    probe.click(close, cx);
    assert_eq!(*probe.shared.closes.borrow(), [Reason::Cancel]);
    assert!(!cx.update(|cx| probe.state.read(cx).is_open()));
}

#[gpui::test]
fn side_sheet_modal_gates_user_close_and_restores_its_opener(cx: &mut TestAppContext) {
    let probe = Harness::new(cx);
    probe.shared.mode.set(SheetMode::Modal);
    probe.shared.placement.set(SheetPlacement::Detached);
    probe.shared.properties.set(DialogProperties {
        dismiss_on_escape: false,
        dismiss_on_outside_click: false,
    });
    probe.open(cx);
    let opener = probe.bounds("side-opener", cx);
    let outside = point(opener.left() + px(10.), opener.center().y);
    probe.click(outside, cx);
    probe.scroll(outside, cx);
    cx.simulate_keystrokes(probe.window, "escape");
    assert_eq!(probe.shared.underlay_clicks.get(), 0);
    assert_eq!(probe.underlay.offset().y, px(0.));
    assert!(probe.shared.requests.borrow().is_empty());
    assert!(cx.update(|cx| probe.state.read(cx).is_open()));
    cx.update_window(probe.window, |_, window, _| {
        let sheet = find(window, &[], &"sheet-probe".into()).unwrap();
        assert_eq!(sheet.role(), Some(Role::Dialog));
        assert_eq!(sheet.label(), Some("Inspect workspace"));
        let close = find(window, &[], &"close-sheet-probe".into()).unwrap();
        assert_eq!(close.role(), Some(Role::Button));
        assert_eq!(close.label(), Some("Close inspection"));
    })
    .unwrap();
    for key in ["tab", "tab", "tab", "shift-tab"] {
        cx.simulate_keystrokes(probe.window, key);
        probe.frame(cx);
        assert!(
            !cx.update_window(probe.window, |_, window, _| probe.opener.is_focused(window))
                .unwrap()
        );
    }
    let back = probe.bounds("back", cx).center();
    probe.click(back, cx);
    assert_eq!(probe.shared.back_clicks.get(), 1);
    assert!(cx.update(|cx| probe.state.read(cx).is_open()));

    probe.shared.properties.set(DialogProperties::default());
    probe.frame(cx);
    probe.click(outside, cx);
    assert_eq!(*probe.shared.requests.borrow(), [Reason::OutsideClick]);
    assert!(cx.update(|cx| probe.state.read(cx).is_open()));

    probe.shared.allow_close.set(true);
    probe.click(outside, cx);
    assert_eq!(probe.shared.underlay_clicks.get(), 0);
    assert_eq!(*probe.shared.closes.borrow(), [Reason::OutsideClick]);
    assert!(
        cx.update_window(probe.window, |_, window, _| probe.opener.is_focused(window))
            .unwrap()
    );

    probe.open(cx);
    cx.simulate_keystrokes(probe.window, "escape");
    assert_eq!(
        *probe.shared.closes.borrow(),
        [Reason::OutsideClick, Reason::Escape]
    );
    probe.open(cx);
    let close = probe.bounds("close-sheet-probe", cx).center();
    probe.click(close, cx);
    assert_eq!(
        *probe.shared.closes.borrow(),
        [Reason::OutsideClick, Reason::Escape, Reason::Cancel]
    );
    assert_eq!(
        cx.update(|cx| probe.state.read(cx).last_close_reason()),
        Some(Reason::Cancel)
    );
    assert!(
        cx.update_window(probe.window, |_, window, _| probe.opener.is_focused(window))
            .unwrap()
    );
}

#[gpui::test]
fn nested_side_sheets_close_the_active_layer_and_restore_each_opener(cx: &mut TestAppContext) {
    struct Nesting {
        lower: Entity<DialogState>,
        upper: Entity<DialogState>,
        opener: FocusHandle,
        inner_opener: FocusHandle,
    }
    impl Render for Nesting {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let lower = self.lower.clone();
            let upper = self.upper.clone();
            div()
                .size_full()
                .child(
                    Button::new("open-lower", "Open inspection")
                        .track_focus(&self.opener)
                        .on_activate(move |window, cx| {
                            lower.update(cx, |state, cx| state.open(window, cx));
                        }),
                )
                .child(
                    Sheet::new(
                        "lower-sheet",
                        &self.lower,
                        "Inspection",
                        Button::new("open-upper", "Open notes")
                            .track_focus(&self.inner_opener)
                            .on_activate(move |window, cx| {
                                upper.update(cx, |state, cx| state.open(window, cx));
                            }),
                    )
                    .edge(SheetEdge::End)
                    .mode(SheetMode::Modal),
                )
                .child(
                    Sheet::new("upper-sheet", &self.upper, "Notes", "Inspection notes")
                        .edge(SheetEdge::Start)
                        .mode(SheetMode::Modal)
                        .layer(1),
                )
        }
    }

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|cx| {
            let lower = cx.new(DialogState::new);
            let upper = cx.new(DialogState::new);
            for state in [&lower, &upper] {
                cx.observe(state, |_, _, cx| cx.notify()).detach();
            }
            Nesting {
                lower,
                upper,
                opener: cx.focus_handle(),
                inner_opener: cx.focus_handle(),
            }
        });
        let window =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (window.into(), view)
    });
    let (lower, upper, opener, inner) = cx.update(|cx| {
        let view = view.read(cx);
        (
            view.lower.clone(),
            view.upper.clone(),
            view.opener.clone(),
            view.inner_opener.clone(),
        )
    });
    cx.update_window(window, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        opener.focus(window, cx);
        lower.update(cx, |state, cx| state.open(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
        inner.focus(window, cx);
        upper.update(cx, |state, cx| state.open(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.simulate_keystrokes(window, "escape");
    cx.update_window(window, |_, window, cx| {
        assert!(lower.read(cx).is_open());
        assert!(!upper.read(cx).is_open());
        assert!(inner.is_focused(window));
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.simulate_keystrokes(window, "escape");
    cx.update_window(window, |_, window, cx| {
        assert!(!lower.read(cx).is_open());
        assert!(opener.is_focused(window));
    })
    .unwrap();
}

#[gpui::test]
fn modal_side_sheet_mirrors_and_fits_a_compact_scaled_window(cx: &mut TestAppContext) {
    use super::ModalExamples;
    use crate::story::Story;
    use gpui_material::{material_theme, set_material_theme, theme::Direction};

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| ModalExamples::new(Story::Sheets, window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(408.), px(420.)));
    cx.simulate_window_scale_factor_change(window.into(), 1.5);
    let sheet_content = |window: &Window| {
        find(window, &[], &"details-sheet".into())
            .and_then(|panel| find(window, panel.path(), &"sheet-content".into()))
    };
    let assert_side = |cx: &mut TestAppContext, side, reduced| {
        cx.update_window(window.into(), |_, window, cx| {
            cx.set_reduce_motion(reduced);
            let mut theme = material_theme(cx).clone();
            theme.layout.direction = side;
            set_material_theme(theme, cx);
            let state = view.read(cx).secondary.clone();
            state.update(cx, |state, cx| state.open(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            let bounds = find(window, &[], &"details-sheet".into()).unwrap().bounds();
            match (side, reduced) {
                (Direction::Ltr, false) => assert!(bounds.right() > window.viewport_size().width),
                (Direction::Rtl, false) => assert!(bounds.left() < px(0.)),
                (Direction::Ltr, true) => assert_eq!(bounds.right(), window.viewport_size().width),
                (Direction::Rtl, true) => assert_eq!(bounds.left(), px(0.)),
            }
        })
        .unwrap();
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(60));
        if !reduced {
            cx.update_window(window.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                let bounds = find(window, &[], &"details-sheet".into()).unwrap().bounds();
                match side {
                    Direction::Ltr => assert!(
                        bounds.left() < window.viewport_size().width
                            && bounds.right() > window.viewport_size().width
                    ),
                    Direction::Rtl => assert!(bounds.right() > px(0.) && bounds.left() < px(0.)),
                }
            })
            .unwrap();
        }
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(250));
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let sheet = find(window, &[], &"details-sheet".into()).unwrap();
            assert_eq!(sheet.role(), Some(Role::Dialog));
            let bounds = sheet.bounds();
            let viewport = window.viewport_size();
            assert!(bounds.size.width > px(0.) && bounds.size.width < viewport.width);
            assert!((bounds.bottom() - viewport.height).abs() <= px(1.));
            match side {
                Direction::Ltr => assert_eq!(bounds.right(), viewport.width),
                Direction::Rtl => assert_eq!(bounds.left(), px(0.)),
            }
            let state = view.read(cx).secondary.clone();
            state.update(cx, |state, cx| state.close(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(sheet_content(window).is_none(), reduced);
        })
        .unwrap();
        if !reduced {
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(60));
            cx.update_window(window.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                let body = sheet_content(window).unwrap().bounds();
                match side {
                    Direction::Ltr => assert!(body.right() > window.viewport_size().width),
                    Direction::Rtl => assert!(body.left() < px(0.)),
                }
            })
            .unwrap();
        }
    };
    assert_side(cx, Direction::Ltr, false);
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(250));
    assert_side(cx, Direction::Rtl, true);
}

#[gpui::test]
fn expandable_sheet_drag_handle_is_not_a_keyboard_tab_stop(cx: &mut TestAppContext) {
    struct ExpandableProbe {
        state: Entity<DialogState>,
        before: FocusHandle,
    }

    impl Render for ExpandableProbe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .child(Button::new("before-sheet", "Before").track_focus(&self.before))
                .child(
                    Sheet::new("expandable-sheet", &self.state, "Notes", "Sheet body")
                        .expandable(false, |_, _, _| {}),
                )
        }
    }

    let (window, before) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let state = cx.new(DialogState::new);
        let before = cx.focus_handle();
        let handle = before.clone();
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| ExpandableProbe {
                state,
                before: handle,
            })
        })
        .unwrap();
        (window, before)
    });
    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        before.focus(window, cx);
        window.refresh();
        window.draw(cx).clear(cx);
        window.focus_next(cx);
        window.refresh();
        window.draw(cx).clear(cx);
        let drag = find(window, &[], &"drag-handle".into()).unwrap();
        let expand = find(window, &[], &"expand".into()).unwrap();
        assert_ne!(drag.focused(), Some(true));
        assert_eq!(expand.focused(), Some(true));
    })
    .unwrap();
}
