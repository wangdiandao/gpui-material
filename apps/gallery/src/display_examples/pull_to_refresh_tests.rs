use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use gpui::{
    AnyWindowHandle, AppContext, Context, Entity, InputEvent, InteractiveElement, IntoElement,
    ParentElement, Pixels, Point, Render, Role, ScrollDelta, ScrollEvent, ScrollSource,
    StatefulInteractiveElement, Styled, TestAppContext, TouchPhase, Window, WindowOptions, div,
    point, px,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{MaterialRoot, PullToRefresh, PullToRefreshProgress, PullToRefreshState};
use gpui_material::{
    foundation::{set_window_material_theme, window_material_theme},
    theme::Direction,
};

#[derive(Default)]
struct DemoState {
    refreshing: Cell<bool>,
    disabled: Cell<bool>,
    retargeted: Cell<bool>,
    presses: Cell<usize>,
    progress: RefCell<Option<PullToRefreshProgress>>,
}

struct Fixture {
    state: Entity<PullToRefreshState>,
    owner: Rc<DemoState>,
    external: bool,
}

impl Render for Fixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let requests = self.owner.clone();
        let indicator = self.owner.clone();
        let children =
            || (0..8).map(|index| div().h(px(70.)).flex_none().child(format!("Row {index}")));
        let viewport = if self.owner.retargeted.get() {
            "other-pull"
        } else {
            "pull"
        };
        let refresh = PullToRefresh::new(
            viewport,
            &self.state,
            self.owner.refreshing.get(),
            move |_, _| {
                requests.presses.set(requests.presses.get() + 1);
            },
        )
        .enabled(!self.owner.disabled.get())
        .refresh_label("Reload inbox")
        .h(px(230.));
        let refresh = if self.external {
            let scroll = self.state.read(cx).scroll_handle();
            refresh
                .track_scroll(&scroll)
                .threshold(px(40.))
                .indicator_distance(px(64.))
                .indicator(move |progress, _, _| {
                    *indicator.progress.borrow_mut() = Some(progress);
                    div().id("custom-progress").test_support().size(px(48.))
                })
                .child(
                    div()
                        .id("nested-scroll")
                        .overflow_y_scroll()
                        .track_scroll(&scroll)
                        .size_full()
                        .flex()
                        .flex_col()
                        .children(children()),
                )
        } else {
            refresh.children(children())
        };
        div().size_full().child(refresh)
    }
}

struct Harness {
    window: AnyWindowHandle,
    view: Entity<Fixture>,
    state: Entity<PullToRefreshState>,
    owner: Rc<DemoState>,
}

impl Harness {
    fn new(external: bool, reduced: bool, cx: &mut TestAppContext) -> Self {
        let result = cx.update(|cx| {
            crate::init_gallery(cx);
            cx.set_reduce_motion(reduced);
            let owner = Rc::new(DemoState::default());
            let state = cx.new(|_| PullToRefreshState::new());
            let view = cx.new(|cx| {
                cx.observe(&state, |_, _, cx| cx.notify()).detach();
                Fixture {
                    state: state.clone(),
                    owner: owner.clone(),
                    external,
                }
            });
            let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                let mut theme = window_material_theme(window, cx).clone();
                theme.layout.direction = Direction::Rtl;
                set_window_material_theme(Some(theme), window, cx);
                view.clone()
            })
            .unwrap()
            .into();
            Self {
                window,
                view,
                owner,
                state,
            }
        });
        result.frame(cx);
        result
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

    fn dispatch(
        &self,
        phase: TouchPhase,
        delta: ScrollDelta,
        source: ScrollSource,
        cx: &mut TestAppContext,
    ) {
        cx.update_window(self.window, |_, window, cx| {
            let viewport = find(window, &[], &"refresh-scroll".into())
                .unwrap()
                .bounds();
            assert!(
                viewport.size.height > px(0.),
                "refresh viewport must be laid out"
            );
            window.dispatch_event(
                ScrollEvent {
                    position: viewport.center(),
                    delta,
                    source,
                    touch_phase: phase,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
        self.frame(cx);
    }

    fn pan(&self, phase: TouchPhase, x: f32, y: f32, cx: &mut TestAppContext) {
        self.dispatch(
            phase,
            ScrollDelta::Pixels(point(px(x), px(y))),
            ScrollSource::Touch,
            cx,
        );
    }

    fn fraction(&self, cx: &mut TestAppContext) -> f32 {
        cx.update(|cx| self.state.read(cx).distance_fraction())
    }

    fn offset(&self, cx: &mut TestAppContext) -> Point<Pixels> {
        cx.update(|cx| self.state.read(cx).scroll_handle().offset())
    }

    fn set_scroll(&self, offset: Pixels, cx: &mut TestAppContext) {
        cx.update(|cx| {
            self.state
                .read(cx)
                .scroll_handle()
                .set_offset(point(px(0.), offset))
        });
        self.frame(cx);
    }

    fn node(
        &self,
        id: &str,
        cx: &mut TestAppContext,
    ) -> Option<gpui_base::test_support::ElementSnapshot> {
        let id = gpui::ElementId::from(gpui::SharedString::from(id.to_owned()));
        cx.update_window(self.window, |_, window, _| find(window, &[], &id))
            .unwrap()
    }
}

#[gpui::test]
fn pull_to_refresh_requests_once_and_waits_for_owner(cx: &mut TestAppContext) {
    let refresh = Harness::new(false, true, cx);
    refresh.pan(TouchPhase::Started, 0., 0., cx);
    refresh.pan(TouchPhase::Moved, 0., 90., cx);
    assert_eq!(refresh.fraction(cx), 0.5625);
    let loading = refresh.node("refresh-progress", cx).unwrap();
    assert_eq!(loading.role(), Some(Role::ProgressIndicator));
    refresh.pan(TouchPhase::Ended, 0., 0., cx);
    assert_eq!(refresh.fraction(cx), 0.);
    assert_eq!(refresh.owner.presses.get(), 0);

    refresh.pan(TouchPhase::Started, 0., 200., cx);
    assert!(refresh.fraction(cx) > 1.);
    refresh.pan(TouchPhase::Ended, 0., 0., cx);
    assert_eq!(refresh.owner.presses.get(), 1);
    assert!(cx.update(|cx| refresh.state.read(cx).is_refresh_pending()));
    assert_eq!(
        refresh.node("refresh-action", cx).unwrap().disabled(),
        Some(true)
    );
    refresh.pan(TouchPhase::Moved, 0., 280., cx);
    refresh.pan(TouchPhase::Ended, 0., 0., cx);
    refresh.pan(TouchPhase::Started, 0., 280., cx);
    refresh.pan(TouchPhase::Ended, 0., 0., cx);
    assert_eq!(refresh.owner.presses.get(), 1);

    cx.update(|cx| {
        refresh
            .state
            .update(cx, |state, cx| state.reject_refresh(cx))
    });
    refresh.frame(cx);
    assert!(!cx.update(|cx| refresh.state.read(cx).is_refresh_pending()));
    assert_eq!(refresh.fraction(cx), 0.);

    let button = refresh.node("refresh-action", cx).unwrap();
    assert_eq!(button.label(), Some("Reload inbox"));
    assert_eq!(button.role(), Some(Role::Button));
    cx.update_window(refresh.window, |_, window, cx| {
        window.activate_window();
        window.focus_next(cx);
        assert!(window.focused(cx).is_some());
    })
    .unwrap();
    refresh.frame(cx);
    cx.update_window(refresh.window, |_, window, cx| {
        let keystroke = gpui::Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap();
        window.dispatch_event(
            gpui::KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(gpui::KeyUpEvent { keystroke }.to_platform_input(), cx);
    })
    .unwrap();
    assert_eq!(refresh.owner.presses.get(), 2);

    refresh.owner.refreshing.set(true);
    refresh.frame(cx);
    assert_eq!(refresh.fraction(cx), 1.);
    assert!(!cx.update(|cx| refresh.state.read(cx).is_refresh_pending()));
    refresh.pan(TouchPhase::Started, 0., 280., cx);
    refresh.pan(TouchPhase::Ended, 0., 0., cx);
    assert_eq!(refresh.owner.presses.get(), 2);
    refresh.owner.refreshing.set(false);
    refresh.frame(cx);
    assert_eq!(refresh.fraction(cx), 0.);
    assert_ne!(
        refresh.node("refresh-action", cx).unwrap().disabled(),
        Some(true)
    );
}

#[gpui::test]
fn nested_scroll_hands_off_at_top_and_cancels_on_retarget(cx: &mut TestAppContext) {
    let refresh = Harness::new(true, true, cx);
    refresh.set_scroll(px(-100.), cx);
    assert_eq!(refresh.offset(cx).y, px(-100.));
    for (phase, delta) in [(TouchPhase::Started, 50.), (TouchPhase::Moved, 82.)] {
        refresh.dispatch(
            phase,
            ScrollDelta::Pixels(point(px(0.), px(delta))),
            ScrollSource::Pointer,
            cx,
        );
    }
    assert_eq!(refresh.offset(cx).y, px(0.));
    assert_eq!(refresh.fraction(cx), 0.4);
    let indicator = refresh.owner.progress.borrow().unwrap();
    assert_eq!(indicator.offset, px(25.6));
    let bounds = refresh.node("custom-progress", cx).unwrap().bounds();
    assert_eq!(
        bounds.center().x,
        refresh
            .node("refresh-scroll", cx)
            .unwrap()
            .bounds()
            .center()
            .x
    );
    refresh.pan(TouchPhase::Cancelled, 0., 0., cx);
    assert_eq!(
        refresh.owner.presses.get(),
        0,
        "a different stream cannot commit either"
    );
    refresh.dispatch(
        TouchPhase::Cancelled,
        ScrollDelta::Pixels(Point::default()),
        ScrollSource::Pointer,
        cx,
    );
    assert_eq!(refresh.fraction(cx), 0.);

    refresh.pan(TouchPhase::Started, 0., 60., cx);
    assert_eq!(refresh.fraction(cx), 0.75);
    refresh.pan(TouchPhase::Moved, 0., -60., cx);
    assert_eq!(refresh.fraction(cx), 0.);
    refresh.pan(TouchPhase::Moved, 0., -30., cx);
    assert_eq!(refresh.offset(cx).y, px(-30.));
    refresh.pan(TouchPhase::Ended, 0., 0., cx);

    refresh.dispatch(
        TouchPhase::Moved,
        ScrollDelta::Lines(point(0., 4.)),
        ScrollSource::Pointer,
        cx,
    );
    assert_eq!(refresh.offset(cx).y, px(0.));
    refresh.pan(TouchPhase::Started, 200., 40., cx);
    assert_eq!(refresh.fraction(cx), 0.);
    refresh.pan(TouchPhase::Moved, 0., 145., cx);
    assert!(refresh.fraction(cx) > 1.);
    refresh.owner.retargeted.set(true);
    refresh.frame(cx);
    refresh.pan(TouchPhase::Ended, 0., 0., cx);
    assert_eq!(refresh.fraction(cx), 0.);
    assert_eq!(refresh.owner.presses.get(), 0);

    refresh.pan(TouchPhase::Started, 0., 160., cx);
    refresh.owner.disabled.set(true);
    refresh.frame(cx);
    refresh.pan(TouchPhase::Ended, 0., 0., cx);
    assert_eq!(refresh.fraction(cx), 0.);
    assert_eq!(refresh.owner.presses.get(), 0);
}

#[gpui::test]
fn pull_to_refresh_supports_snap_settle_and_reset(cx: &mut TestAppContext) {
    let refresh = Harness::new(true, false, cx);
    let animate = |refresh: &Harness, cx: &mut TestAppContext| {
        cx.update(|cx| {
            refresh
                .state
                .update(cx, |state, cx| state.animate_to_threshold(cx))
        });
        refresh.frame(cx);
    };
    animate(&refresh, cx);
    assert!(cx.update(|cx| refresh.state.read(cx).is_animating()));
    cx.executor().advance_clock(Duration::from_millis(90));
    refresh.frame(cx);
    assert!(refresh.fraction(cx) > 0. && refresh.fraction(cx) < 1.);
    cx.executor().advance_clock(Duration::from_millis(100));
    refresh.frame(cx);
    assert_eq!(refresh.fraction(cx), 1.);
    assert!(!cx.update(|cx| refresh.state.read(cx).is_animating()));
    cx.update(|cx| refresh.state.update(cx, |state, cx| state.snap_to(0.4, cx)));
    refresh.frame(cx);
    assert_eq!(refresh.fraction(cx), 0.4);

    refresh.pan(TouchPhase::Started, 0., 140., cx);
    refresh.pan(TouchPhase::Cancelled, 0., 0., cx);
    cx.executor().advance_clock(Duration::from_millis(200));
    refresh.frame(cx);
    assert_eq!(refresh.fraction(cx), 0.);
    assert_eq!(refresh.owner.presses.get(), 0);

    refresh.pan(TouchPhase::Started, 0., 140., cx);
    cx.update(|cx| refresh.state.update(cx, |state, cx| state.reset(cx)));
    refresh.frame(cx);
    refresh.pan(TouchPhase::Ended, 0., 0., cx);
    assert_eq!(refresh.fraction(cx), 0.);
    assert_eq!(refresh.owner.presses.get(), 0);
}
