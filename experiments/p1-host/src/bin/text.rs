use gpui::{
    AppContext, Context, Entity, IntoElement, Render, Role, Styled, TitlebarOptions, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_base::{
    Theme,
    input::{Input, InputAccessibility, InputState},
};
use p1_host::open_probe;

struct TextProbe {
    text: Entity<InputState>,
    password: Entity<InputState>,
}

impl TextProbe {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = |label: &str, masked, window: &mut Window, cx: &mut Context<InputState>| {
            let mut state = InputState::new(window, cx)
                .masked(masked)
                .default_value(if masked { "P1-sample" } else { "" });
            state.set_accessibility(
                InputAccessibility::new(Some(if masked {
                    Role::PasswordInput
                } else {
                    Role::TextInput
                }))
                .with_label(Some(label.to_owned().into()))
                .with_accessibility_id(Some(
                    if masked {
                        "probe-password"
                    } else {
                        "probe-text"
                    }
                    .into(),
                ))
                .with_value_exposed(!masked),
            );
            state
        };
        Self {
            text: cx.new(|cx| input("中文输入", false, window, cx)),
            password: cx.new(|cx| input("密码", true, window, cx)),
        }
    }
}

impl Render for TextProbe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::global(cx).tokens.colors;
        div()
            .size_full()
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
