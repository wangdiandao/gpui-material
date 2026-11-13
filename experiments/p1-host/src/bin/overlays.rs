use gpui::{
    Anchor, AppContext, Context, FocusHandle, IntoElement, Render, Role, Styled, TitlebarOptions,
    Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_base::{Button, Popover, TestSupportExt, Theme};
use p1_host::open_probe;

struct OverlayProbe {
    trigger: FocusHandle,
    child_trigger: FocusHandle,
}

impl OverlayProbe {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            trigger: cx.focus_handle(),
            child_trigger: cx.focus_handle(),
        }
    }
}

impl Render for OverlayProbe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::global(cx).tokens.colors;
        let child_trigger = self.child_trigger.clone();
        let trigger = move |id, label, focus: &FocusHandle| {
            Button::new(id)
                .track_focus(focus)
                .border_1()
                .border_color(colors.border)
                .px_3()
                .py_2()
                .focus_visible(move |style| style.border_color(colors.ring))
                .accessibility_label(label)
                .child(label)
        };

        div()
            .size_full()
            .tab_group()
            .flex()
            .justify_end()
            .items_start()
            .p_4()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(
                Popover::new("outer")
                    .anchor(Anchor::TopLeft)
                    .offset(px(4.))
                    .trigger(trigger("outer-trigger", "打开浮层…", &self.trigger))
                    .content(move |_, _, _| {
                        div()
                            .id("outer-panel")
                            .role(Role::Group)
                            .test_support()
                            .aria_label("外层浮层")
                            .min_w(px(176.))
                            .bg(colors.surface)
                            .border_1()
                            .border_color(colors.border)
                            .p_2()
                            .child(
                                Popover::new("inner")
                                    .anchor(Anchor::TopLeft)
                                    .offset(px(4.))
                                    .trigger(trigger(
                                        "inner-trigger",
                                        "打开子浮层…",
                                        &child_trigger,
                                    ))
                                    .content(move |_, _, _| {
                                        div()
                                            .id("inner-panel")
                                            .role(Role::Status)
                                            .test_support()
                                            .aria_label("子浮层已打开")
                                            .min_w(px(176.))
                                            .bg(colors.surface)
                                            .border_1()
                                            .border_color(colors.border)
                                            .p_2()
                                            .child("子浮层已打开")
                                    }),
                            )
                    }),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx| {
        gpui_base::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(480.), px(320.)), cx)),
            titlebar: Some(TitlebarOptions {
                title: Some("P1-07 浮层探针".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        open_probe(cx, options, |_, cx| cx.new(OverlayProbe::new)).unwrap();
    });
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn nested_popovers_keep_focus_and_fit_in_window(cx: &mut gpui::TestAppContext) {
    use gpui::App;
    use gpui_base::test_support;

    let (window, view) = cx.update(|cx| {
        gpui_base::init(cx);
        let view = cx.new(OverlayProbe::new);
        let window = open_probe(cx, WindowOptions::default(), |_, _| view.clone()).unwrap();
        (window, view)
    });

    let frame = |window: &mut Window, cx: &mut App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };

    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        frame(window, cx);
        view.read(cx).trigger.clone().focus(window, cx);
        frame(window, cx);
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "enter");
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        frame(window, cx);
        let outer = test_support::find(window, &[], &"outer-panel".into()).unwrap();
        assert!(outer.bounds().right() <= window.viewport_size().width);
        view.read(cx).child_trigger.clone().focus(window, cx);
        frame(window, cx);
    })
    .unwrap();

    cx.simulate_keystrokes(window.into(), "enter");
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        frame(window, cx);
        let inner = test_support::find(window, &[], &"inner-panel".into()).unwrap();
        assert!(inner.bounds().right() <= window.viewport_size().width);
    })
    .unwrap();

    cx.simulate_keystrokes(window.into(), "escape");
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        frame(window, cx);
        assert!(test_support::find(window, &[], &"inner-panel".into()).is_none());
        assert!(test_support::find(window, &[], &"outer-panel".into()).is_some());
        assert!(view.read(cx).child_trigger.is_focused(window));
    })
    .unwrap();

    cx.simulate_keystrokes(window.into(), "escape");
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        frame(window, cx);
        assert!(test_support::find(window, &[], &"outer-panel".into()).is_none());
        assert!(view.read(cx).trigger.is_focused(window));
    })
    .unwrap();
}
