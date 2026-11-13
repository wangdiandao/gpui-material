use gpui::{
    AppContext, Bounds, Context, Entity, IntoElement, Render, Role, SharedString, Styled,
    TextAlign, TextRun, TitlebarOptions, Window, WindowBounds, WindowOptions, canvas, div, fill,
    font, point, prelude::*, px, size,
};
use gpui_base::{
    Button, Popover, Theme,
    input::{Input, InputAccessibility, InputState},
};
use p1_host::open_probe;

struct TextProbe {
    text: Entity<InputState>,
    password: Entity<InputState>,
    popup_text: Entity<InputState>,
    trace_metrics: bool,
}

impl TextProbe {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = |id: &'static str,
                     label: &'static str,
                     masked,
                     window: &mut Window,
                     cx: &mut Context<InputState>| {
            let mut state = InputState::new(window, cx)
                .masked(masked)
                .default_value(if masked { "P1-sample" } else { "" });
            state.set_accessibility(
                InputAccessibility::new(Some(if masked {
                    Role::PasswordInput
                } else {
                    Role::TextInput
                }))
                .with_label(Some(label.into()))
                .with_accessibility_id(Some(id.into()))
                .with_value_exposed(!masked),
            );
            state
        };
        Self {
            text: cx.new(|cx| input("probe-text", "中文输入", false, window, cx)),
            password: cx.new(|cx| input("probe-password", "密码", true, window, cx)),
            popup_text: cx.new(|cx| input("probe-popup-text", "浮层输入", false, window, cx)),
            trace_metrics: std::env::var_os("P1_TEXT_METRICS").is_some(),
        }
    }
}

impl Render for TextProbe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::global(cx).tokens.colors;
        let popup_text = self.popup_text.clone();
        div()
            .size_full()
            .tab_group()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .bg(colors.background)
            .text_color(colors.foreground)
            .children(
                [("中文输入", &self.text), ("密码", &self.password)].map(|(label, input)| {
                    div().flex().flex_col().gap_1().child(label).child(
                        div()
                            .border_1()
                            .border_color(colors.border)
                            .p_2()
                            .child(Input::new(input)),
                    )
                }),
            )
            .child(
                Popover::new("text-popup")
                    .trigger(
                        Button::new("open-text-popup")
                            .border_1()
                            .border_color(colors.border)
                            .focus_visible(|style| style.border_color(colors.ring))
                            .px_3()
                            .py_2()
                            .accessibility_label("打开浮层输入")
                            .child("打开浮层输入…"),
                    )
                    .content(move |_, _, _| {
                        div()
                            .w_64()
                            .border_1()
                            .border_color(colors.border)
                            .p_2()
                            .bg(colors.surface)
                            .child("浮层输入")
                            .child(Input::new(&popup_text))
                    }),
            )
            .when(self.trace_metrics, |this| {
                this.child(
                    canvas(
                        move |_, window, cx| {
                            let sample: SharedString = "A中文B".into();
                            let primary = font("Consolas");
                            let primary_id = cx.text_system().resolve_font(&primary);
                            let run = TextRun {
                                len: sample.len(),
                                font: primary,
                                color: colors.foreground,
                                ..Default::default()
                            };
                            let line = window.text_system().shape_line(sample, px(24.), &[run], None);
                            let cjk = line.font_id_for_index(1).expect("font for CJK");
                            assert_ne!(cjk, primary_id, "CJK should use the fallback face");
                            assert_eq!(line.font_id_for_index(0), Some(primary_id));
                            assert!(line.x_for_index(4) > line.x_for_index(1));
                            assert_eq!(line.x_for_index(line.len()), line.width());
                            println!(
                                "P1-03 scale={} primary={primary_id:?} CJK={cjk:?} advance={:?} CJK-start={:?} CJK-next={:?}",
                                window.scale_factor(), line.width(), line.x_for_index(1), line.x_for_index(4)
                            );
                            line
                        },
                        move |bounds, line, window, cx| {
                            line.paint(bounds.origin, px(32.), TextAlign::Left, None, window, cx)
                                .expect("paint shaped fallback line");
                            let end = point(bounds.origin.x + line.width(), bounds.origin.y);
                            window.paint_quad(fill(
                                Bounds::new(end, size(px(1.), px(32.))),
                                colors.ring,
                            ));
                        },
                    )
                    .w_full()
                    .h(px(32.)),
                )
            })
    }
}

fn main() {
    gpui_platform::application().run(|cx| {
        gpui_base::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(520.), px(320.)), cx)),
            titlebar: Some(TitlebarOptions {
                title: Some("P1-03 文本探针".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        open_probe(cx, options, |window, cx| {
            cx.new(|cx| TextProbe::new(window, cx))
        })
        .expect("open text probe");
    });
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn input_handler_composes_and_commits_utf16(cx: &mut gpui::TestAppContext) {
    use gpui::EntityInputHandler;

    let (window, text) = cx.update(|cx| {
        gpui_base::init(cx);
        let mut text = None;
        let window = open_probe(cx, WindowOptions::default(), |window, cx| {
            cx.new(|cx| {
                let view = TextProbe::new(window, cx);
                text = Some(view.text.clone());
                view
            })
        })
        .unwrap();
        (window, text.unwrap())
    });

    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        text.update(cx, |state, cx| {
            state.replace_text_in_range(None, "🙂", window, cx);
            state.replace_and_mark_text_in_range(None, "zhong", Some(5..5), window, cx);
            assert_eq!(state.marked_text_range(window, cx), Some(2..7));
            assert_eq!(
                state.selected_text_range(false, window, cx).unwrap().range,
                7..7
            );
            state.replace_text_in_range(None, "中", window, cx);
            assert_eq!(state.marked_text_range(window, cx), None);
            assert_eq!(state.value().as_ref(), "🙂中");
            state.replace_and_mark_text_in_range(None, "wen", Some(3..3), window, cx);
            assert_eq!(state.marked_text_range(window, cx), Some(3..6));
            state.replace_and_mark_text_in_range(None, "", Some(0..0), window, cx);
            assert_eq!(state.marked_text_range(window, cx), None);
            assert_eq!(state.value().as_ref(), "🙂中");
        });
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn selection_clipboard_and_password_copy_use_input_bindings(cx: &mut gpui::TestAppContext) {
    use gpui::{EntityInputHandler, OperatingSystem};

    let (window, (text, password)) = cx.update(|cx| {
        gpui_base::init(cx);
        let mut fields = None;
        let window = open_probe(cx, WindowOptions::default(), |window, cx| {
            cx.new(|cx| {
                let view = TextProbe::new(window, cx);
                fields = Some((view.text.clone(), view.password.clone()));
                view
            })
        })
        .unwrap();
        (window, fields.unwrap())
    });

    let modifier = cx.update(|cx| match cx.platform_info().os {
        OperatingSystem::Mac => "cmd",
        _ => "ctrl",
    });
    let keys = |cx: &mut gpui::TestAppContext, keys: &str| {
        cx.simulate_keystrokes(window.into(), keys);
        cx.run_until_parked();
    };
    let focus = |cx: &mut gpui::TestAppContext, field: &Entity<InputState>| {
        cx.update_window(window.into(), |_, window, cx| {
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);
            field.update(cx, |state, cx| state.focus(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    };
    focus(cx, &text);
    cx.update_window(window.into(), |_, window, cx| {
        text.update(cx, |state, cx| {
            state.replace_text_in_range(None, "中文🙂", window, cx)
        });
    })
    .unwrap();
    keys(cx, &format!("{modifier}-a {modifier}-c"));
    assert_eq!(
        cx.update(|cx| cx.read_from_clipboard().and_then(|item| item.text())),
        Some("中文🙂".to_owned())
    );
    keys(cx, "backspace");
    assert!(cx.update(|cx| text.read(cx).value().is_empty()));
    keys(cx, &format!("{modifier}-v"));
    assert_eq!(cx.update(|cx| text.read(cx).value()), "中文🙂");

    focus(cx, &password);
    keys(cx, &format!("{modifier}-a {modifier}-c"));
    let clipboard = cx.update(|cx| cx.read_from_clipboard().and_then(|item| item.text()));
    assert_ne!(clipboard.as_deref(), Some("P1-sample"));
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn popup_input_accepts_text_and_preserves_value_after_dismissal(cx: &mut gpui::TestAppContext) {
    use gpui::{EntityInputHandler, InputEvent, MouseButton, MouseDownEvent};
    use gpui_base::test_support;

    let (window, view) = cx.update(|cx| {
        gpui_base::init(cx);
        let mut view = None;
        let window = open_probe(cx, WindowOptions::default(), |window, cx| {
            let content = cx.new(|cx| TextProbe::new(window, cx));
            view = Some(content.clone());
            content
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);
        let trigger = test_support::find(window, &[], &"open-text-popup".into()).unwrap();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: trigger.bounds().center(),
                modifiers: window.modifiers(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    cx.run_until_parked();

    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let popup = view.read(cx).popup_text.clone();
        let bounds = popup.read(cx).input_bounds();
        assert!(bounds.size.width > px(0.));
        popup.update(cx, |state, cx| {
            state.focus(window, cx);
            state.replace_and_mark_text_in_range(None, "zhong", Some(5..5), window, cx);
            state.replace_text_in_range(None, "中", window, cx);
        });
        window.refresh();
        window.draw(cx).clear(cx);
        popup.update(cx, |state, cx| {
            let caret = state.bounds_for_range(1..1, bounds, window, cx).unwrap();
            assert!(caret.origin.x >= bounds.origin.x);
            assert!(caret.origin.y >= bounds.origin.y);
        });
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "escape");
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let trigger = test_support::find(window, &[], &"open-text-popup".into()).unwrap();
        assert_eq!(trigger.focused(), Some(true));
        assert_eq!(view.read(cx).popup_text.read(cx).value(), "中");
    })
    .unwrap();
}
