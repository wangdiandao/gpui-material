use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    AppContext, Context, IntoElement, ParentElement, Render, Styled, TestAppContext, Window,
    WindowOptions, div, px,
};
use gpui_material::foundation::{MotionStatus, animate_with_status};
use gpui_material::theme::{CubicBezier, MotionSpec};
use gpui_material::{
    AdaptiveNavigationState, AdaptiveNavigationVisibility, LinearProgress, LoadingIndicator,
    MaterialRoot, NavigationRailState, RailMode,
};

struct ClippedAnimationView {
    offscreen: bool,
}

impl Render for ClippedAnimationView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(300.))
            .h(px(100.))
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(if self.offscreen { px(300.) } else { px(0.) })
                    .flex_none(),
            )
            .child(LoadingIndicator::new("clipped-loading", "Loading"))
            .child(LinearProgress::new("clipped-progress", "Busy").indeterminate())
    }
}

#[gpui::test]
fn offscreen_clipped_loading_and_progress_pause_without_scheduling_frames(cx: &mut TestAppContext) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(false);
        let mut handle_view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|_| ClippedAnimationView { offscreen: true });
            handle_view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, handle_view.unwrap())
    });

    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();

    cx.update_window(window.into(), |_, window, cx| {
        assert!(window.is_window_active());
        window.refresh();
        window.draw(cx).clear(cx);
        // Both indicators are clipped outside the 100px overflow_hidden viewport:
        // zero animation frames should be scheduled.
        assert_eq!(window.simulate_next_frame(cx), 0);

        // Bring indicators into the visible viewport -> animations resume cleanly.
        view.update(cx, |state, cx| {
            state.offscreen = false;
            cx.notify();
        });
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(window.simulate_next_frame(cx) > 0);

        // Turning on reduced motion mid-flight stops requesting frames immediately.
        cx.set_reduce_motion(true);
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(window.simulate_next_frame(cx), 0);
    })
    .unwrap();
}

struct AnimatedChannelView {
    target: f32,
    spec: MotionSpec,
    last: Rc<RefCell<(f32, MotionStatus)>>,
}

impl Render for AnimatedChannelView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sample = animate_with_status("test-channel", self.target, self.spec, window, cx);
        *self.last.borrow_mut() = (sample.value, sample.status);
        div().size(px(32.))
    }
}

#[gpui::test]
fn unified_animation_reverses_proportionally_and_settles_on_reduced_motion(
    cx: &mut TestAppContext,
) {
    let linear = MotionSpec::Tween {
        duration_ms: 200,
        easing: CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        },
    };
    let last = Rc::new(RefCell::new((0.0, MotionStatus::Idle)));
    let (window, view, nav, rail) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(false);
        let nav = cx.new(|_| {
            AdaptiveNavigationState::new(AdaptiveNavigationVisibility::Shown)
                .transition_duration(Duration::from_millis(200))
        });
        let rail = cx.new(|_| {
            NavigationRailState::new(RailMode::Collapsed)
                .transition_duration(Duration::from_millis(200))
        });
        let mut handle_view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|_| AnimatedChannelView {
                target: 0.0,
                spec: linear,
                last: last.clone(),
            });
            handle_view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, handle_view.unwrap(), nav, rail)
    });

    // Establish initial resting value at 0.0.
    cx.update_window(window.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
        assert_eq!(*last.borrow(), (0.0, MotionStatus::Idle));
        assert_eq!(window.simulate_next_frame(cx), 0);

        // Start 200ms linear tween to 1.0.
        view.update(cx, |v, cx| {
            v.target = 1.0;
            cx.notify();
        });
        window.draw(cx).clear(cx);
        assert_eq!(last.borrow().1, MotionStatus::Running);
        assert!(window.simulate_next_frame(cx) > 0);
    })
    .unwrap();

    // Advance 50ms (25%) and reverse back to 0.0 -> reversal should finish in 50ms (at 100ms total).
    cx.executor().advance_clock(Duration::from_millis(50));
    cx.update_window(window.into(), |_, window, cx| {
        view.update(cx, |v, cx| {
            v.target = 0.0;
            cx.notify();
        });
        window.draw(cx).clear(cx);
        assert!((last.borrow().0 - 0.25).abs() < 0.02);
        assert_eq!(last.borrow().1, MotionStatus::Running);
        assert!(window.simulate_next_frame(cx) > 0);
    })
    .unwrap();

    cx.executor().advance_clock(Duration::from_millis(55));
    cx.update_window(window.into(), |_, window, cx| {
        // Drain any frame scheduled prior to the clock jump, then render the settled frame.
        window.simulate_next_frame(cx);
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(*last.borrow(), (0.0, MotionStatus::Finished));
        assert_eq!(window.simulate_next_frame(cx), 0);

        // Start another transition and flip reduce_motion mid-flight -> settles immediately.
        view.update(cx, |v, cx| {
            v.target = 1.0;
            cx.notify();
        });
        window.draw(cx).clear(cx);
        assert_eq!(last.borrow().1, MotionStatus::Running);
        assert!(window.simulate_next_frame(cx) > 0);

        cx.set_reduce_motion(true);
        window.draw(cx).clear(cx);
        assert_eq!(*last.borrow(), (1.0, MotionStatus::Finished));
        assert_eq!(window.simulate_next_frame(cx), 0);

        // Verify entity-owned AdaptiveNavigationState and NavigationRailState also honor reduce_motion.
        nav.update(cx, |s, cx| s.hide(cx));
        assert!(!nav.read(cx).is_animating());
        assert_eq!(nav.read(cx).progress(), 0.0);

        rail.update(cx, |s, cx| s.expand(cx));
        assert!(!rail.read(cx).is_animating());
        assert_eq!(rail.read(cx).current_value(), RailMode::Expanded);
    })
    .unwrap();
}
