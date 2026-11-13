use gpui::{
    AppContext, ClickEvent, Context, FocusHandle, IntoElement, KeyBinding, Render, Role, Styled,
    Subscription, TitlebarOptions, Window, WindowOptions, div, prelude::*,
};
use gpui_base::{Button, Radio, RadioGroup, TestSupportExt, Theme};
use p1_host::open_probe;

gpui::actions!(activation_probe, [MoveRadioFocus, CancelDrag]);

fn init(cx: &mut gpui::App) {
    gpui_base::init(cx);
    cx.bind_keys([
        KeyBinding::new("left", MoveRadioFocus, Some("RovingRadio")),
        KeyBinding::new("right", MoveRadioFocus, Some("RovingRadio")),
        KeyBinding::new("escape", CancelDrag, Some("ActivationProbe")),
    ]);
}

#[derive(Default)]
struct Counts {
    mouse: usize,
    keyboard: usize,
}

struct Activation {
    focus: [FocusHandle; 3],
    disabled: bool,
    counts: Counts,
    radios: [FocusHandle; 2],
    selected_radio: usize,
    drag_generation: usize,
    drag_status: &'static str,
    _activation: Subscription,
}

struct DragPreview;

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child("拖动中")
    }
}

impl Activation {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let activation = cx.observe_window_activation(window, |this, window, cx| {
            if !window.is_window_active() {
                this.cancel_drag("窗口已失焦", window, cx);
            }
        });
        Self {
            focus: std::array::from_fn(|_| cx.focus_handle()),
            disabled: false,
            counts: Counts::default(),
            radios: std::array::from_fn(|_| cx.focus_handle()),
            selected_radio: 0,
            drag_generation: 0,
            drag_status: "待拖动",
            _activation: activation,
        }
    }

    fn cancel_drag(&mut self, reason: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        if self.drag_status == "拖动中" {
            window.release_pointer();
            cx.stop_active_drag(window);
            self.drag_status = reason;
            cx.notify();
        }
    }

    fn toggle_disabled(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.disabled = !self.disabled;
        if self.disabled {
            self.cancel_drag("禁用已取消拖动", window, cx);
        }
        cx.notify();
    }

    #[cfg(target_os = "windows")]
    fn observe_native_capture(&self, window: &Window, cx: &Context<Self>) {
        use std::time::Duration;
        use windows::Win32::UI::Input::KeyboardAndMouse::{GetCapture, ReleaseCapture};

        // Both the GPUI handler and this task run on the window's UI thread.
        let capture = unsafe { GetCapture() };
        if capture.is_invalid() {
            return;
        }
        let generation = self.drag_generation;
        let exercise_capture_loss = std::env::var_os("P1_ACTIVATION_RELEASE_CAPTURE").is_some();
        let executor = cx.background_executor().clone();
        cx.spawn_in(window, async move |view, cx| {
            let mut held_frames = 0;
            loop {
                executor.timer(Duration::from_millis(16)).await;
                let still_dragging = cx.update(|window, cx| {
                    view.update(cx, |this, cx| {
                        if this.drag_generation != generation
                            || this.drag_status != "拖动中"
                            || !cx.has_active_drag()
                        {
                            return false;
                        }
                        if unsafe { GetCapture() } != capture {
                            this.cancel_drag("系统捕获已丢失", window, cx);
                            return false;
                        }
                        held_frames += 1;
                        if exercise_capture_loss && held_frames == 12 {
                            // Exercise the OS transition independently of focus and mouse-up.
                            unsafe { ReleaseCapture() }
                                .expect("release the probe's native capture");
                        }
                        true
                    })
                    .unwrap_or(false)
                });
                if !still_dragging.unwrap_or(false) {
                    break;
                }
            }
        })
        .detach();
    }
}

impl Render for Activation {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::global(cx).tokens.colors;
        let status = format!("指针 {} · 键盘 {}", self.counts.mouse, self.counts.keyboard);
        div()
            .key_context("ActivationProbe")
            .on_action(cx.listener(|this, _: &CancelDrag, window, cx| {
                this.cancel_drag("Esc 已取消拖动", window, cx);
            }))
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
                        .accessibility_label("切换计数与拖动可用性")
                        .child(if self.disabled {
                            "启用计数与拖动"
                        } else {
                            "禁用计数与拖动"
                        })
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.toggle_disabled(window, cx);
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
                RadioGroup::new("radio-group")
                    .axis(gpui::Axis::Horizontal)
                    .key_context("RovingRadio")
                    .flex()
                    .gap_2()
                    .on_action(cx.listener(|this, _: &MoveRadioFocus, window, cx| {
                        // A two-item group wraps to the other item in either direction.
                        this.selected_radio = 1 - this.selected_radio;
                        this.radios[this.selected_radio].focus(window, cx);
                        cx.notify();
                    }))
                    .children(
                        [("radio-first", "第一项"), ("radio-second", "第二项")]
                            .into_iter()
                            .enumerate()
                            .map(|(index, (id, label))| {
                                let owner = cx.entity().downgrade();
                                Radio::new(id)
                                    .checked(self.selected_radio == index)
                                    .tab_stop(self.selected_radio == index)
                                    .track_focus(&self.radios[index])
                                    .accessibility_label(label)
                                    .border_1()
                                    .border_color(colors.border)
                                    .focus_visible(|style| style.border_color(colors.ring))
                                    .px_3()
                                    .py_2()
                                    .child(if self.selected_radio == index {
                                        format!("{label} · 已选")
                                    } else {
                                        label.to_owned()
                                    })
                                    .on_change(move |_, _, _, cx| {
                                        let _ = owner.update(cx, |this, cx| {
                                            this.selected_radio = index;
                                            cx.notify();
                                        });
                                    })
                            }),
                    ),
            )
            .child({
                let mut source = div()
                    .id("drag-source")
                    .test_support()
                    .role(Role::Group)
                    .aria_label("拖动此处")
                    .border_1()
                    .border_color(colors.border)
                    .px_3()
                    .py_2()
                    .child("拖动此处");
                if !self.disabled {
                    let view = cx.entity().downgrade();
                    source = source.on_drag((), move |_, _, window, cx| {
                        let _ = view.update(cx, |this, cx| {
                            this.drag_generation += 1;
                            this.drag_status = "拖动中";
                            #[cfg(target_os = "windows")]
                            this.observe_native_capture(window, cx);
                            cx.notify();
                        });
                        cx.new(|_| DragPreview)
                    });
                }
                source
            })
            .on_mouse_up_out(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    if this.drag_status == "拖动中" {
                        this.drag_status = "拖动已结束";
                        cx.notify();
                    }
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    if this.drag_status == "拖动中" {
                        this.drag_status = "拖动已结束";
                        cx.notify();
                    }
                }),
            )
            .child(
                div()
                    .id("drag-status")
                    .role(Role::Status)
                    .aria_label(self.drag_status)
                    .child(self.drag_status),
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
        init(cx);
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("P1-02 激活探针".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        open_probe(cx, options, |window, cx| {
            cx.new(|cx| Activation::new(window, cx))
        })
        .expect("open activation probe");
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
            init(cx);
            let mut view = None;
            let window = open_probe(cx, WindowOptions::default(), |window, cx| {
                let entity = cx.new(|cx| Activation::new(window, cx));
                view = Some(entity.clone());
                entity
            })
            .unwrap();
            (window, view.unwrap())
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
            assert!(view.read(cx).radios[0].is_focused(window));
            key(window, cx, "shift-tab", 0);
            assert!(view.read(cx).focus[2].is_focused(window));
            key(window, cx, "enter", 0);
            assert_eq!(view.read(cx).counts.keyboard, 2);
            key(window, cx, "tab", 0);
            assert!(view.read(cx).radios[0].is_focused(window));
            key(window, cx, "right", 0);
            assert_eq!(view.read(cx).selected_radio, 1);
            assert!(view.read(cx).radios[1].is_focused(window));
            key(window, cx, "tab", 0);
            assert!(view.read(cx).focus[1].is_focused(window));
            key(window, cx, "shift-tab", 0);
            assert!(view.read(cx).radios[1].is_focused(window));
            key(window, cx, "left", 0);
            assert_eq!(view.read(cx).selected_radio, 0);
            assert!(view.read(cx).radios[0].is_focused(window));
            view.update(cx, |this, cx| this.toggle_disabled(window, cx));
            frame(window, cx);

            let drag = test_support::find(window, &[], &"drag-source".into()).unwrap();
            let start = drag.bounds().center();
            let moved = start + point(px(24.), px(0.));
            let modifiers = window.modifiers();
            for cancel_with_disable in [false, true, false] {
                window.dispatch_event(
                    MouseDownEvent {
                        button: MouseButton::Left,
                        position: start,
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
                        position: moved,
                        pressed_button: Some(MouseButton::Left),
                        modifiers,
                    }
                    .to_platform_input(),
                    cx,
                );
                frame(window, cx);
                assert!(cx.has_active_drag());
                if cancel_with_disable {
                    view.update(cx, |this, cx| this.toggle_disabled(window, cx));
                    frame(window, cx);
                    assert!(view.read(cx).disabled);
                    assert_eq!(view.read(cx).drag_status, "禁用已取消拖动");
                } else {
                    key(window, cx, "escape", 0);
                    assert_eq!(view.read(cx).drag_status, "Esc 已取消拖动");
                }
                assert!(!cx.has_active_drag());
                assert!(window.captured_hitbox().is_none());
                window.dispatch_event(
                    MouseUpEvent {
                        button: MouseButton::Left,
                        position: moved,
                        modifiers,
                        click_count: 1,
                    }
                    .to_platform_input(),
                    cx,
                );
                frame(window, cx);
                if cancel_with_disable {
                    view.update(cx, |this, cx| this.toggle_disabled(window, cx));
                    frame(window, cx);
                }
            }
        })
        .unwrap();
    }
}
