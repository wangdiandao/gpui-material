use std::rc::Rc;

use gpui::{
    AppContext, Context, InputEvent as _, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, MouseUpEvent, ParentElement, Pixels, Point, Render, TestAppContext, Window,
    WindowOptions, div, point, px,
};
use gpui_base::{TestSupportExt as _, test_support::find};
use gpui_material::{MaterialRoot, MenuItem};

#[gpui::test]
fn context_menu_positions_at_cursor_repositions_preserves_enter_supports_keyboard_lazy_and_disabled(
    cx: &mut TestAppContext,
) {
    use std::cell::Cell;

    use gpui::{FocusHandle, KeyDownEvent, Keystroke, Modifiers, Styled};
    use gpui_material::{Button, MaterialElementExt};

    struct SurfaceContextView {
        focus: FocusHandle,
        primary_clicks: Rc<Cell<usize>>,
        lazy_builds: Rc<Cell<usize>>,
        disabled: bool,
    }

    impl Render for SurfaceContextView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let primary_clicks = self.primary_clicks.clone();
            let lazy_builds = self.lazy_builds.clone();
            div().size_full().p_4().child(
                div()
                    .id("ctx-canvas")
                    .test_support()
                    .w(px(420.))
                    .h(px(300.))
                    .child(
                        Button::outlined("ctx-primary-btn", "Primary")
                            .track_focus(&self.focus)
                            .on_activate(move |_, _| {
                                primary_clicks.set(primary_clicks.get() + 1);
                            }),
                    )
                    .context_menu_with("canvas-context", move |_, _| {
                        lazy_builds.set(lazy_builds.get() + 1);
                        [
                            MenuItem::new("ctx-copy", "Copy"),
                            MenuItem::new("ctx-paste", "Paste"),
                        ]
                    })
                    .disabled(self.disabled),
            )
        }
    }

    let primary_clicks = Rc::new(Cell::new(0));
    let lazy_builds = Rc::new(Cell::new(0));
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let p = primary_clicks.clone();
        let l = lazy_builds.clone();
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|cx| SurfaceContextView {
                focus: cx.focus_handle(),
                primary_clicks: p,
                lazy_builds: l,
                disabled: false,
            });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let window = window.into();
    let frame = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    let click = |cx: &mut TestAppContext, button: MouseButton, position: Point<Pixels>| {
        cx.update_window(window, |_, window, cx| {
            window.dispatch_event(
                MouseDownEvent {
                    button,
                    position,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseUpEvent {
                    button,
                    position,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
        frame(cx);
        frame(cx);
    };

    frame(cx);
    assert_eq!(
        lazy_builds.get(),
        0,
        "lazy context menu must not build items while closed"
    );

    // Focusing inner button and pressing Enter triggers the button, NOT the context menu.
    cx.update_window(window, |_, window, cx| {
        view.read(cx).focus.clone().focus(window, cx);
    })
    .unwrap();
    frame(cx);
    cx.update_window(window, |_, window, cx| {
        let keystroke = Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap();
        window.dispatch_event(
            KeyDownEvent {
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
    frame(cx);
    assert_eq!(primary_clicks.get(), 1);
    assert_eq!(lazy_builds.get(), 0);
    cx.update_window(window, |_, window, _| {
        assert!(find(window, &[], &"menu".into()).is_none());
    })
    .unwrap();

    // Secondary click inside the large canvas positions the menu at the cursor.
    let first_click = point(px(260.), px(180.));
    click(cx, MouseButton::Right, first_click);
    assert!(lazy_builds.get() >= 1);
    cx.update_window(window, |_, window, _| {
        let menu = find(window, &[], &"menu".into()).unwrap().bounds();
        assert_eq!(menu.origin, first_click);
    })
    .unwrap();

    // A second right-click elsewhere inside the canvas repositions the open menu.
    let second_click = point(px(120.), px(90.));
    click(cx, MouseButton::Right, second_click);
    cx.update_window(window, |_, window, _| {
        let menu = find(window, &[], &"menu".into()).unwrap().bounds();
        assert_eq!(menu.origin, second_click);
    })
    .unwrap();

    // Left-clicking the canvas outside the popup dismisses the context menu.
    click(cx, MouseButton::Left, point(px(360.), px(260.)));
    cx.update_window(window, |_, window, _| {
        assert!(find(window, &[], &"menu".into()).is_none());
    })
    .unwrap();

    // Shift+F10 on the focused trigger opens the context menu anchored to the trigger.
    cx.update_window(window, |_, window, cx| {
        view.read(cx).focus.clone().focus(window, cx);
    })
    .unwrap();
    frame(cx);
    cx.update_window(window, |_, window, cx| {
        let keystroke = Keystroke {
            modifiers: Modifiers::shift(),
            key: "f10".into(),
            key_char: None,
        };
        window.dispatch_event(
            KeyDownEvent {
                keystroke,
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx);
    frame(cx);
    cx.update_window(window, |_, window, _| {
        assert!(find(window, &[], &"menu".into()).is_some());
    })
    .unwrap();

    // Escape closes the menu; disabling prevents reopening.
    cx.simulate_keystrokes(window, "escape");
    frame(cx);
    cx.update_window(window, |_, window, cx| {
        assert!(find(window, &[], &"menu".into()).is_none());
        view.update(cx, |v, cx| {
            v.disabled = true;
            cx.notify();
        });
    })
    .unwrap();
    frame(cx);
    click(cx, MouseButton::Right, first_click);
    cx.update_window(window, |_, window, _| {
        assert!(find(window, &[], &"menu".into()).is_none());
    })
    .unwrap();
}
