use gpui::{
    AppContext, ClickEvent, Context, FocusHandle, IntoElement, Render, Role, Styled,
    TitlebarOptions, Window, WindowOptions, div, prelude::*,
};
use gpui_base::{Button, TestSupportExt, Theme};
use p1_host::open_probe;

#[derive(Default)]
struct Counts {
    mouse: usize,
    keyboard: usize,
}

struct Activation {
    focus: [FocusHandle; 3],
    disabled: bool,
    counts: Counts,
}

impl Activation {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: std::array::from_fn(|_| cx.focus_handle()),
            disabled: false,
            counts: Counts::default(),
        }
    }
}

impl Render for Activation {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::global(cx).tokens.colors;
        let status = format!("指针 {} · 键盘 {}", self.counts.mouse, self.counts.keyboard);
        div()
            .size_full()
            .tab_group()
            .flex()
            .flex_col()
            .items_start()
            .gap_2()
            .p_4()
            .bg(colors.background)
            .text_color(colors.foreground)
            .children(
                [
                    Button::new("activate")
                        .track_focus(&self.focus[0])
                        .disabled(self.disabled)
                        .accessibility_label("计数")
                        .child("计数")
                        .on_click(cx.listener(|this, event, _, cx| {
                            match event {
                                ClickEvent::Mouse(_) | ClickEvent::Touch(_) => {
                                    this.counts.mouse += 1
                                }
                                ClickEvent::Keyboard(_) => this.counts.keyboard += 1,
                            }
                            cx.notify();
                        })),
                    Button::new("toggle-disabled")
                        .track_focus(&self.focus[1])
                        .accessibility_label("切换计数可用性")
                        .child(if self.disabled {
                            "启用计数"
                        } else {
                            "禁用计数"
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.disabled = !this.disabled;
                            cx.notify();
                        })),
                    Button::new("end")
                        .track_focus(&self.focus[2])
                        .accessibility_label("末尾焦点")
                        .child("末尾焦点"),
                ]
                .into_iter()
                .map(|button| {
                    button
                        .border_1()
                        .border_color(colors.border)
                        .py_2()
                        .px_3()
                        .focus_visible(|style| style.border_color(colors.ring))
                        .hover(|style| style.bg(colors.muted))
                        .active(|style| style.bg(colors.selection))
                        .styles(|styles| {
                            styles.disabled(|style| style.text_color(colors.muted_foreground))
                        })
                }),
            )
            .child(
                div()
                    .id("counts")
                    .role(Role::Status)
                    .aria_label(status.clone())
                    .test_support()
                    .child(status),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx| {
        gpui_base::init(cx);
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("P1-02 激活探针".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        open_probe(cx, options, |_, cx| cx.new(Activation::new)).expect("open activation probe");
    });
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        App, InputEvent, KeyDownEvent, KeyUpEvent, Keystroke, MouseButton, MouseDownEvent,
        MouseMoveEvent, MouseUpEvent, TestAppContext, point, px,
    };
    use gpui_base::test_support;

    fn frame(window: &mut Window, cx: &mut App) {
        window.refresh();
        window.draw(cx).clear(cx);
    }

    fn key(window: &mut Window, cx: &mut App, key: &str, repeats: usize) {
        let keystroke = Keystroke::parse_with_os(key, cx.platform_info().os).unwrap();
        for n in 0..=repeats {
            window.dispatch_event(
                KeyDownEvent {
                    keystroke: keystroke.clone(),
                    is_held: n > 0,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            frame(window, cx);
        }
        window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
        frame(window, cx);
    }

    #[gpui::test]
    fn pointer_keyboard_repeat_cancellation_and_focus(cx: &mut TestAppContext) {
        let (window, view) = cx.update(|cx| {
            gpui_base::init(cx);
            let view = cx.new(Activation::new);
            let window = open_probe(cx, WindowOptions::default(), |_, _| view.clone()).unwrap();
            (window, view)
        });

        cx.update_window(window.into(), |_, window, cx| {
            window.activate_window();
            frame(window, cx);
            let button = test_support::find(window, &[], &"activate".into()).unwrap();
            let inside = button.bounds().center();
            let outside = point(inside.x, button.bounds().bottom() + px(4.));
            for release in [inside, outside] {
                let modifiers = window.modifiers();
                window.dispatch_event(
                    MouseDownEvent {
                        button: MouseButton::Left,
                        position: inside,
                        modifiers,
                        click_count: 1,
                        first_mouse: false,
                    }
                    .to_platform_input(),
                    cx,
                );
                frame(window, cx);
                window.dispatch_event(
                    MouseMoveEvent {
                        position: release,
                        pressed_button: Some(MouseButton::Left),
                        modifiers,
                    }
                    .to_platform_input(),
                    cx,
                );
                frame(window, cx);
                window.dispatch_event(
                    MouseUpEvent {
                        button: MouseButton::Left,
                        position: release,
                        modifiers,
                        click_count: 1,
                    }
                    .to_platform_input(),
                    cx,
                );
                frame(window, cx);
            }
            assert_eq!(view.read(cx).counts.mouse, 1);
            assert!(!window.last_input_was_keyboard());

            for activation_key in ["enter", "space"] {
                key(window, cx, activation_key, 3);
            }
            assert_eq!(view.read(cx).counts.keyboard, 2);
            assert!(window.last_input_was_keyboard());

            key(window, cx, "tab", 0);
            assert!(view.read(cx).focus[1].is_focused(window));
            key(window, cx, "enter", 0);
            assert!(view.read(cx).disabled);
            key(window, cx, "shift-tab", 0);
            assert!(view.read(cx).focus[2].is_focused(window));
            key(window, cx, "enter", 0);
            assert_eq!(view.read(cx).counts.keyboard, 2);
            key(window, cx, "tab", 0);
            assert!(view.read(cx).focus[1].is_focused(window));
        })
        .unwrap();
    }
}
