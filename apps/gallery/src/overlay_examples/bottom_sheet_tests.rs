use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use gpui::{
    AnyWindowHandle, AppContext, Context, ElementId, Entity, FocusHandle, InputEvent,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement, Pixels, Point, Render, Role, Styled, TestAppContext, VisualTestContext, Window,
    WindowOptions, div, point, px, size,
};
use gpui_base::{
    TestSupportExt,
    test_support::{ElementSnapshot, find},
};
use gpui_material::{
    BottomSheetState, BottomSheetThresholds, BottomSheetValue as Value, Button, DialogCloseReason,
    DialogProperties, LogicalEdges, MaterialRoot, Scaffold, ScaffoldContentLayout, Sheet,
    SheetMode,
};

#[derive(Clone, Copy)]
enum Presentation {
    Scaffold,
    Standard,
    Modal,
}

#[derive(Default)]
struct Configuration {
    layout: Cell<Option<ScaffoldContentLayout>>,
    observed: Cell<Option<(Value, bool, Pixels, Pixels)>>,
    notices: Cell<usize>,
    reject_expand: Cell<bool>,
    reject_close: Cell<bool>,
    drag_enabled: Cell<bool>,
    show_grip: Cell<bool>,
    closes: RefCell<Vec<DialogCloseReason>>,
}

struct Fixture {
    state: Entity<BottomSheetState>,
    presentation: Presentation,
    config: Rc<Configuration>,
    opener: FocusHandle,
}

impl Render for Fixture {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let config = self.config.clone();
        let closes = self.config.clone();
        let state = self.state.clone();
        let sheet = Sheet::bottom(
            "bottom-test",
            &self.state,
            "Sheet under test",
            div()
                .id("sheet-descendant")
                .test_support()
                .h(px(90.))
                .child("Recent activity"),
        )
        .extent(px(260.))
        .peek_height(px(90.))
        .max_width(px(460.))
        .drag_enabled(self.config.drag_enabled.get())
        .show_handle(self.config.show_grip.get())
        .drag_thresholds(BottomSheetThresholds {
            position: px(65.),
            velocity: px(180.),
        })
        .handle_content(
            div()
                .id("custom-sheet-mark")
                .test_support()
                .size(px(16.))
                .bg(gpui::rgb(0xaf6baa)),
        )
        .handle_label("Resize activity")
        .content_insets(LogicalEdges {
            top: 10.,
            start: 14.,
            ..LogicalEdges::default()
        })
        .on_dismiss_request(move |_, _, _| !config.reject_close.get())
        .on_closed(move |reason, _, _| closes.closes.borrow_mut().push(reason));
        let opener = Button::new("test-bottom-opener", "Show activities")
            .track_focus(&self.opener)
            .on_activate(move |window, cx| {
                state.update(cx, |state, cx| state.show(window, cx));
            });
        match self.presentation {
            Presentation::Scaffold => {
                let layout = self.config.clone();
                Scaffold::with_layout("snap-scaffold", move |value, _, _| {
                    layout.layout.set(Some(value));
                    opener
                })
                .bottom_bar(div().id("snap-navigation").test_support().h(px(35.)))
                .bottom_sheet(sheet)
                .content_insets(LogicalEdges {
                    bottom: 150.,
                    ..LogicalEdges::default()
                })
                .into_any_element()
            }
            Presentation::Standard => div()
                .size_full()
                .flex()
                .flex_col()
                .child(div().flex_1().child(opener))
                .child(sheet)
                .into_any_element(),
            Presentation::Modal => div()
                .size_full()
                .child(opener)
                .child(sheet.mode(SheetMode::Modal).properties(DialogProperties {
                    dismiss_on_outside_click: false,
                    ..DialogProperties::default()
                }))
                .into_any_element(),
        }
    }
}

struct Harness {
    window: AnyWindowHandle,
    view: Entity<Fixture>,
    state: Entity<BottomSheetState>,
    config: Rc<Configuration>,
    opener: FocusHandle,
}

impl Harness {
    fn new(
        presentation: Presentation,
        initial: Value,
        enabled: Vec<Value>,
        reduce_motion: bool,
        cx: &mut TestAppContext,
    ) -> Self {
        let this = cx.update(|cx| {
            crate::init_gallery(cx);
            cx.set_reduce_motion(reduce_motion);
            let config = Rc::new(Configuration::default());
            config.drag_enabled.set(true);
            config.show_grip.set(true);
            let guard = config.clone();
            let state = cx.new(|cx| {
                BottomSheetState::new(initial, cx)
                    .enabled_values(enabled)
                    .confirm_value_change(move |value| {
                        value != Value::Expanded || !guard.reject_expand.get()
                    })
            });
            let opener = cx.focus_handle();
            let view = cx.new(|cx| {
                let observed = config.clone();
                cx.observe(&state, move |_, state, cx| {
                    let state = state.read(cx);
                    observed.observed.set(Some((
                        state.current_value(),
                        state.is_animating(),
                        state.visible_height(),
                        state.offset(),
                    )));
                    observed.notices.set(observed.notices.get() + 1);
                    cx.notify();
                })
                .detach();
                Fixture {
                    state: state.clone(),
                    opener: opener.clone(),
                    presentation,
                    config: config.clone(),
                }
            });
            let window =
                MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone())
                    .unwrap()
                    .into();
            Self {
                window,
                view,
                state,
                config,
                opener,
            }
        });
        cx.simulate_window_resize(this.window, size(px(800.), px(480.)));
        this.frame(cx);
        this
    }

    fn frame(&self, cx: &mut TestAppContext) {
        cx.update_window(self.window, |_, window, cx| {
            self.view.update(cx, |_, cx| cx.notify());
            window.refresh();
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }

    fn node(&self, id: &'static str, cx: &mut TestAppContext) -> Option<ElementSnapshot> {
        cx.update_window(self.window, |_, window, _| {
            find(window, &[], &ElementId::from(id))
        })
        .unwrap()
    }

    fn event(&self, event: impl InputEvent, cx: &mut TestAppContext) {
        cx.update_window(self.window, |_, window, cx| {
            window.dispatch_event(event.to_platform_input(), cx);
        })
        .unwrap();
        self.frame(cx);
    }

    fn down(&self, position: Point<Pixels>, cx: &mut TestAppContext) {
        cx.update_window(self.window, |_, window, _| window.activate_window())
            .unwrap();
        self.event(
            MouseDownEvent {
                position,
                button: MouseButton::Left,
                click_count: 1,
                ..Default::default()
            },
            cx,
        );
    }

    fn up(&self, position: Point<Pixels>, cx: &mut TestAppContext) {
        self.event(
            MouseUpEvent {
                position,
                button: MouseButton::Left,
                click_count: 1,
                ..Default::default()
            },
            cx,
        );
    }

    fn move_left(&self, position: Point<Pixels>, cx: &mut TestAppContext) {
        self.event(
            MouseMoveEvent {
                position,
                pressed_button: Some(MouseButton::Left),
                modifiers: Default::default(),
            },
            cx,
        );
    }

    fn click(&self, id: &'static str, cx: &mut TestAppContext) {
        let position = self.node(id, cx).unwrap().bounds().center();
        self.down(position, cx);
        self.up(position, cx);
    }

    fn position(&self, cx: &mut TestAppContext) -> (Value, Value, Pixels, Pixels) {
        cx.update(|cx| {
            let state = self.state.read(cx);
            (
                state.current_value(),
                state.target_value(),
                state.visible_height(),
                state.offset(),
            )
        })
    }

    fn snap(&self, target: Value, cx: &mut TestAppContext) -> bool {
        let accepted = cx
            .update_window(self.window, |_, window, cx| {
                self.state
                    .update(cx, |state, cx| state.snap_to(target, window, cx))
            })
            .unwrap();
        self.frame(cx);
        accepted
    }
}

#[gpui::test]
fn scaffold_bottom_sheet_resolves_visible_occupancy_and_remaining_inset(cx: &mut TestAppContext) {
    let sheet = Harness::new(
        Presentation::Scaffold,
        Value::PartiallyExpanded,
        vec![Value::PartiallyExpanded, Value::Expanded],
        true,
        cx,
    );
    assert_eq!(
        sheet.position(cx),
        (
            Value::PartiallyExpanded,
            Value::PartiallyExpanded,
            px(90.),
            px(170.)
        )
    );
    let layout = sheet.config.layout.get().unwrap();
    assert_eq!(layout.occupied.bottom, 125.);
    assert_eq!(layout.padding.bottom, 25.);
    let handle = sheet.node("bottom-sheet-resize", cx).unwrap();
    assert_eq!(handle.role(), Some(Role::Button));
    assert_eq!(handle.label(), Some("Resize activity"));
    assert!(sheet.node("custom-sheet-mark", cx).is_some());
    assert!(!sheet.snap(Value::Hidden, cx));
    sheet.config.reject_expand.set(true);
    sheet.click("bottom-sheet-resize", cx);
    assert_eq!(sheet.position(cx).0, Value::PartiallyExpanded);
    sheet.config.reject_expand.set(false);
    sheet.config.drag_enabled.set(false);
    sheet.config.show_grip.set(false);
    sheet.frame(cx);
    assert!(sheet.node("custom-sheet-mark", cx).is_none());
    cx.simulate_keystrokes(sheet.window, "end");
    sheet.frame(cx);
    assert_eq!(sheet.position(cx).0, Value::Expanded);
    let layout = sheet.config.layout.get().unwrap();
    assert_eq!(layout.occupied.bottom, 295.);
    assert_eq!(layout.padding.bottom, 0.);
    let body = sheet.node("sheet-content", cx).unwrap().bounds();
    let content = sheet.node("sheet-descendant", cx).unwrap().bounds();
    assert_eq!(content.top() - body.top(), px(10.));
    assert_eq!(content.left() - body.left(), px(14.));
    assert_eq!(
        sheet.node("bottom-test", cx).unwrap().bounds().size.width,
        px(460.)
    );
    cx.simulate_keystrokes(sheet.window, "home");
    assert_eq!(sheet.position(cx).0, Value::PartiallyExpanded);
}

#[gpui::test]
fn retained_bottom_sheet_unwinds_cancel_resize_and_new_drags(cx: &mut TestAppContext) {
    let sheet = Harness::new(
        Presentation::Standard,
        Value::PartiallyExpanded,
        vec![Value::Hidden, Value::PartiallyExpanded, Value::Expanded],
        true,
        cx,
    );
    let press = |sheet: &Harness, cx: &mut TestAppContext| {
        let start = sheet
            .node("bottom-sheet-resize", cx)
            .unwrap()
            .bounds()
            .center();
        sheet.down(start, cx);
        sheet.move_left(start - point(px(0.), px(22.)), cx);
        start
    };
    let start = press(&sheet, cx);
    assert!(sheet.position(cx).2 > px(90.));
    cx.simulate_keystrokes(sheet.window, "escape");
    sheet.frame(cx);
    sheet.up(start, cx);
    assert_eq!(sheet.position(cx).2, px(90.));

    let start = press(&sheet, cx);
    assert!(sheet.position(cx).2 > px(90.));
    cx.simulate_window_resize(sheet.window, size(px(768.), px(480.)));
    sheet.frame(cx);
    sheet.up(start, cx);
    assert_eq!(sheet.position(cx).2, px(90.));

    let start = press(&sheet, cx);
    let mut visual = VisualTestContext::from_window(sheet.window, cx);
    visual.deactivate_window();
    sheet.frame(cx);
    sheet.up(start, cx);
    assert_eq!(sheet.position(cx).2, px(90.));

    let start = press(&sheet, cx);
    cx.executor().advance_clock(Duration::from_millis(20));
    let furthest = start - point(px(0.), px(35.));
    sheet.move_left(furthest, cx);
    sheet.up(furthest, cx);
    assert_eq!(
        sheet.position(cx).0,
        Value::Expanded,
        "velocity reaches the next stop below the position threshold"
    );

    assert!(sheet.snap(Value::PartiallyExpanded, cx));
    let start = press(&sheet, cx);
    cx.executor().advance_clock(Duration::from_millis(400));
    let furthest = start - point(px(0.), px(110.));
    sheet.move_left(furthest, cx);
    cx.executor().advance_clock(Duration::from_millis(180));
    sheet.up(furthest, cx);
    assert_eq!(
        sheet.position(cx).0,
        Value::Expanded,
        "held drag settles by position"
    );
}

#[gpui::test]
fn retained_modal_bottom_sheet_animates_and_restores_focus_after_accepted_close(
    cx: &mut TestAppContext,
) {
    let sheet = Harness::new(
        Presentation::Modal,
        Value::Hidden,
        vec![Value::Hidden, Value::PartiallyExpanded, Value::Expanded],
        false,
        cx,
    );
    sheet.click("test-bottom-opener", cx);
    assert_eq!(sheet.position(cx).1, Value::PartiallyExpanded);
    assert!(cx.update(|cx| sheet.state.read(cx).is_animating()));
    cx.executor().advance_clock(Duration::from_millis(270));
    sheet.frame(cx);
    assert_eq!(sheet.position(cx).0, Value::PartiallyExpanded);
    assert_eq!(
        sheet.config.observed.get(),
        Some((Value::PartiallyExpanded, false, px(90.), px(170.)))
    );
    let settled_notifications = sheet.config.notices.get();
    sheet.frame(cx);
    assert_eq!(sheet.config.notices.get(), settled_notifications);
    let outside = point(px(4.), px(4.));
    sheet.down(outside, cx);
    sheet.up(outside, cx);
    assert_eq!(sheet.position(cx).1, Value::PartiallyExpanded);
    assert!(sheet.config.closes.borrow().is_empty());

    sheet.config.reject_close.set(true);
    cx.simulate_keystrokes(sheet.window, "escape");
    assert_eq!(sheet.position(cx).1, Value::PartiallyExpanded);
    sheet.config.reject_close.set(false);
    cx.simulate_keystrokes(sheet.window, "escape");
    assert_eq!(sheet.position(cx).1, Value::Hidden);
    assert_eq!(*sheet.config.closes.borrow(), [DialogCloseReason::Escape]);
    assert!(
        cx.update_window(sheet.window, |_, window, _| sheet.opener.is_focused(window))
            .unwrap()
    );
    // Reopen while an outgoing frame still owns the previous focus guard.
    sheet.click("test-bottom-opener", cx);
    assert_eq!(sheet.position(cx).1, Value::PartiallyExpanded);
    cx.simulate_keystrokes(sheet.window, "escape");
    assert_eq!(sheet.position(cx).1, Value::Hidden);
    assert_eq!(
        *sheet.config.closes.borrow(),
        [DialogCloseReason::Escape, DialogCloseReason::Escape]
    );
    cx.executor().advance_clock(Duration::from_millis(270));
    sheet.frame(cx);
    assert!(sheet.node("bottom-test", cx).is_none());

    sheet.click("test-bottom-opener", cx);
    assert!(sheet.snap(Value::Expanded, cx));
    assert!(sheet.snap(Value::Hidden, cx));
    assert_eq!(
        *sheet.config.closes.borrow(),
        [DialogCloseReason::Escape, DialogCloseReason::Escape]
    );
    assert!(
        cx.update_window(sheet.window, |_, window, _| sheet.opener.is_focused(window))
            .unwrap()
    );
    cx.simulate_window_resize(sheet.window, size(px(800.), px(200.)));
    sheet.frame(cx);
    assert_eq!(
        sheet.config.observed.get(),
        Some((Value::Hidden, false, px(0.), px(200.)))
    );
}

#[gpui::test]
fn bottom_sheet_cancels_drag_and_accepts_next_gesture(cx: &mut TestAppContext) {
    use super::ModalExamples;
    use crate::story::Story;

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
    let drag = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let start_height = find(window, &[], &"standard".into())
                .unwrap()
                .bounds()
                .size
                .height;
            let origin = find(window, &[], &"drag-handle".into())
                .unwrap()
                .bounds()
                .center();
            let handle = find(window, &[], &"drag-handle".into()).unwrap().bounds();
            let expand = find(window, &[], &"expand".into()).unwrap().bounds();
            let panel = find(window, &[], &"standard".into()).unwrap().bounds();
            assert_eq!(handle.size.height, px(48.));
            assert!(expand.top() >= handle.bottom());
            assert!(
                expand.bottom() <= panel.bottom(),
                "collapsed sheet keeps its action visible"
            );
            let destination = origin - point(px(0.), px(36.));
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position: origin,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseMoveEvent {
                    position: destination,
                    pressed_button: Some(MouseButton::Left),
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            assert!(cx.has_active_drag());
            window.dispatch_event(
                MouseMoveEvent {
                    position: destination,
                    pressed_button: Some(MouseButton::Left),
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            window.refresh();
            window.draw(cx).clear(cx);
            let height = find(window, &[], &"standard".into())
                .unwrap()
                .bounds()
                .size
                .height;
            assert!(height > start_height && height < px(160.));
            (destination, height)
        })
        .unwrap()
    };
    let release = |cx: &mut TestAppContext, position| {
        cx.update_window(window.into(), |_, window, cx| {
            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    position,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
    };
    let (destination, dragged_height) = drag(cx);
    cx.simulate_keystrokes(window.into(), "escape");
    release(cx, destination);
    cx.update(|cx| {
        assert!(!cx.has_active_drag());
        assert!(!view.read(cx).sheet.as_ref().unwrap().expanded);
    });
    let height = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &"standard".into())
                .unwrap()
                .bounds()
                .size
                .height
        })
        .unwrap()
    };
    assert_eq!(height(cx), dragged_height);
    cx.executor().advance_clock(Duration::from_secs(1));
    assert_eq!(height(cx), px(120.));

    let (destination, _) = drag(cx);
    release(cx, destination);
    cx.update(|cx| assert!(view.read(cx).sheet.as_ref().unwrap().expanded));
}

#[gpui::test]
fn bottom_sheets_center_with_the_compose_width_limit(cx: &mut TestAppContext) {
    use super::ModalExamples;
    use crate::story::Story;

    let (handle, examples) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut examples = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let view = cx.new(|cx| ModalExamples::new(Story::Sheets, window, cx));
            examples = Some(view.clone());
            view
        })
        .unwrap();
        (handle, examples.unwrap())
    });
    cx.simulate_window_resize(handle.into(), size(px(1024.), px(720.)));
    cx.update_window(handle.into(), |_, window, cx| {
        let state = examples.read(cx).primary.clone();
        state.update(cx, |state, cx| state.open(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
        for id in ["standard", "activity-sheet"] {
            let bounds = find(window, &[], &id.into()).unwrap().bounds();
            assert_eq!(bounds.size.width, px(640.));
            assert_eq!(bounds.center().x, window.viewport_size().width / 2.);
        }
    })
    .unwrap();
}
