use gpui::{
    AnyWindowHandle, App, AppContext as _, Context, Entity, InputEvent as _,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, Role, ScrollDelta,
    ScrollEvent, Styled as _, TestAppContext, TouchPhase, Window, WindowOptions, div, point, px,
    rgb,
};
use gpui_base::{TestSupportExt as _, test_support::find};
use gpui_material::{
    InteractionSource, MaterialRoot,
    date_time::{ScrollField, ScrollFieldColors, ScrollFieldState},
};

use super::time_picker_tests::{assert_colors, click, element, frame};

const FIELD_COLORS: [u32; 6] = [0x28445a, 0x9bb6ca, 0xe9f7ff, 0x5a4539, 0x957c66, 0xd1b293];

struct WheelFixture {
    state: Entity<ScrollFieldState>,
    source: Entity<InteractionSource>,
    disabled: bool,
}

#[derive(IntoElement)]
struct RowInk {
    index: usize,
    selected: bool,
    enabled: bool,
}

impl gpui::RenderOnce for RowInk {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .id((
                if self.enabled && self.selected {
                    "chosen-option"
                } else {
                    "option"
                },
                self.index,
            ))
            .test_support()
            .size(px(14.))
            .bg(window.text_style().color)
    }
}

impl Render for WheelFixture {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let [
            container,
            text,
            selected,
            disabled_container,
            disabled_text,
            disabled_selected,
        ] = FIELD_COLORS.map(|tint| Some(rgb(tint).into()));
        div().p_4().w(px(180.)).child(
            ScrollField::new("test-wheel", "Duration in hours", &self.state)
                .value_description(|index| format!("{index} hours"))
                .colors(ScrollFieldColors {
                    container,
                    content: text,
                    selected_content: selected,
                    disabled_container,
                    disabled_content: disabled_text,
                    disabled_selected_content: disabled_selected,
                })
                .interaction_source(&self.source)
                .disabled(self.disabled)
                .item_content(|index, selected, enabled| RowInk {
                    index,
                    selected,
                    enabled,
                }),
        )
    }
}

fn wheel_fixture(
    cx: &mut TestAppContext,
    reduced: bool,
) -> (AnyWindowHandle, Entity<WheelFixture>) {
    cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(reduced);
        let state = cx.new(|cx| ScrollFieldState::new(9, 8, cx));
        let source = cx.new(|_| InteractionSource::new());
        let model = cx.new(|_| WheelFixture {
            state,
            source,
            disabled: false,
        });
        let handle =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| model.clone()).unwrap();
        (handle.into(), model)
    })
}

fn wheel_selection(
    state: &Entity<ScrollFieldState>,
    cx: &mut TestAppContext,
) -> (Option<usize>, Option<usize>, bool) {
    cx.update(|cx| {
        let state = state.read(cx);
        (
            state.selected_option(),
            state.target_option(),
            state.is_scroll_in_progress(),
        )
    })
}

fn send_scroll(
    handle: AnyWindowHandle,
    delta: ScrollDelta,
    touch_phase: TouchPhase,
    cx: &mut TestAppContext,
) {
    cx.update_window(handle, |_, window, cx| {
        let position = find(window, &[], &"test-wheel".into())
            .unwrap()
            .bounds()
            .center();
        window.dispatch_event(
            ScrollEvent {
                position,
                delta,
                touch_phase,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
}

#[gpui::test]
fn scroll_field_accepts_pointer_wheel_touchpad_and_keyboard_with_dynamic_count(
    cx: &mut TestAppContext,
) {
    let (handle, model) = wheel_fixture(cx, true);
    let (state, source) =
        cx.update(|cx| (model.read(cx).state.clone(), model.read(cx).source.clone()));
    frame(handle, cx);
    let field = element(handle, "test-wheel", cx);
    assert_eq!(field.role(), Some(Role::Slider));
    assert_eq!(field.label(), Some("Duration in hours"));
    assert_eq!(field.value(), Some("8 hours"));
    assert_eq!(field.bounds().size.height, px(200.));
    let offset = element(handle, ("chosen-option", 8_usize), cx)
        .bounds()
        .center()
        - field.bounds().center();
    assert!(offset.x.abs() <= px(1.) && offset.y.abs() <= px(1.));
    assert_colors(handle, &FIELD_COLORS[..3], cx);

    click(handle, ("scroll-field-row", 1_usize), cx);
    assert_eq!(wheel_selection(&state, cx), (Some(7), Some(7), false));
    assert_eq!(element(handle, "test-wheel", cx).focused(), Some(true));
    let key = |name, cx: &mut TestAppContext| {
        cx.simulate_keystrokes(handle, name);
        frame(handle, cx);
    };
    key("down", cx);
    assert_eq!(wheel_selection(&state, cx), (Some(8), Some(8), false));
    let wheel_down = ScrollDelta::Lines(point(0., -1.));
    send_scroll(handle, wheel_down, TouchPhase::Moved, cx);
    frame(handle, cx);
    assert_eq!(wheel_selection(&state, cx), (Some(0), Some(0), false));

    let touchpad = ScrollDelta::Pixels(point(px(0.), px(-20.)));
    send_scroll(handle, touchpad, TouchPhase::Started, cx);
    frame(handle, cx);
    assert_eq!(wheel_selection(&state, cx), (Some(0), Some(0), true));
    cx.update(|cx| {
        assert!(
            source
                .read(cx)
                .state(handle.window_id(), &"test-wheel".into())
                .unwrap()
                .dragged
        );
    });
    send_scroll(handle, touchpad, TouchPhase::Moved, cx);
    frame(handle, cx);
    assert_eq!(wheel_selection(&state, cx), (Some(1), Some(1), true));
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(140));
    frame(handle, cx);
    assert_eq!(wheel_selection(&state, cx), (Some(1), Some(1), false));
    send_scroll(
        handle,
        ScrollDelta::Pixels(point(px(0.), px(-60.))),
        TouchPhase::Started,
        cx,
    );
    send_scroll(
        handle,
        ScrollDelta::Pixels(point(px(0.), px(0.))),
        TouchPhase::Cancelled,
        cx,
    );
    frame(handle, cx);
    assert_eq!(wheel_selection(&state, cx), (Some(1), Some(1), false));

    cx.update(|cx| {
        state.update(cx, |state, cx| {
            state.scroll_to_option(usize::MAX, cx);
            state.set_item_count(3, cx);
            assert_eq!(state.item_count(), 3);
        })
    });
    frame(handle, cx);
    assert_eq!(wheel_selection(&state, cx), (Some(2), Some(2), false));
    cx.update(|cx| state.update(cx, |state, cx| state.set_item_count(0, cx)));
    frame(handle, cx);
    assert_eq!(wheel_selection(&state, cx), (None, None, false));
    assert_eq!(element(handle, "test-wheel", cx).value(), None);
    cx.update(|cx| {
        state.update(cx, |state, cx| {
            state.set_item_count(12, cx);
            state.animate_scroll_to_option(usize::MAX, cx);
        })
    });
    assert_eq!(wheel_selection(&state, cx), (Some(11), Some(11), false));

    frame(handle, cx);
    send_scroll(handle, touchpad, TouchPhase::Started, cx);
    assert_eq!(wheel_selection(&state, cx), (Some(11), Some(11), true));

    cx.update(|cx| {
        model.update(cx, |model, cx| {
            model.disabled = true;
            cx.notify();
        })
    });
    frame(handle, cx);
    assert_colors(handle, &FIELD_COLORS[3..], cx);
    assert_eq!(wheel_selection(&state, cx), (Some(11), Some(11), false));
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(140));
    frame(handle, cx);
    click(handle, ("scroll-field-row", 1_usize), cx);
    send_scroll(handle, wheel_down, TouchPhase::Moved, cx);
    key("up", cx);
    assert_eq!(wheel_selection(&state, cx), (Some(11), Some(11), false));
    cx.update(|cx| {
        let snapshot = source
            .read(cx)
            .state(handle.window_id(), &"test-wheel".into())
            .unwrap();
        assert!(snapshot.disabled && !snapshot.dragged && !snapshot.pressed);
    });
}

#[gpui::test]
fn scroll_field_reports_animated_target_and_settles_after_retargeting(cx: &mut TestAppContext) {
    let (handle, model) = wheel_fixture(cx, false);
    let state = cx.update(|cx| model.read(cx).state.clone());
    frame(handle, cx);
    cx.update(|cx| state.update(cx, |state, cx| state.animate_scroll_to_option(6, cx)));
    assert_eq!(wheel_selection(&state, cx), (Some(8), Some(6), true));
    frame(handle, cx);
    assert_eq!(element(handle, "test-wheel", cx).value(), Some("6 hours"));
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(50));
    frame(handle, cx);
    cx.update(|cx| state.update(cx, |state, cx| state.animate_scroll_to_option(4, cx)));
    assert!(wheel_selection(&state, cx).2);
    frame(handle, cx);
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(2));
    frame(handle, cx);
    assert_eq!(wheel_selection(&state, cx), (Some(4), Some(4), false));

    send_scroll(
        handle,
        ScrollDelta::Lines(point(0., -1.)),
        TouchPhase::Moved,
        cx,
    );
    assert_eq!(wheel_selection(&state, cx), (Some(4), Some(5), true));
    cx.update(|cx| {
        model.update(cx, |model, cx| {
            model.disabled = true;
            cx.notify();
        })
    });
    frame(handle, cx);
    assert_eq!(wheel_selection(&state, cx), (Some(5), Some(5), false));
    cx.update(|cx| state.update(cx, |state, cx| state.animate_scroll_to_option(1, cx)));
    let (_, target, in_progress) = wheel_selection(&state, cx);
    assert_eq!(target, Some(1));
    assert!(in_progress);
    frame(handle, cx);
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(2));
    frame(handle, cx);
    assert_eq!(wheel_selection(&state, cx), (Some(1), Some(1), false));
}
