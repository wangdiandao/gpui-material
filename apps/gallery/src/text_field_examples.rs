use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use gpui_material::{TextAreaState, TextField, TextFieldState, TextFieldVariant};

use crate::story::stack;

pub(crate) struct TextFieldExamples {
    name: Entity<TextFieldState>,
    email: Entity<TextFieldState>,
    notes: Entity<TextAreaState>,
    password: Entity<TextFieldState>,
    account: Entity<TextFieldState>,
    disabled: Entity<TextFieldState>,
}

impl TextFieldExamples {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            name: cx.new(|cx| TextFieldState::new(window, cx).placeholder("王小明")),
            email: cx.new(|cx| TextFieldState::new(window, cx).default_value("name@")),
            notes: cx.new(|cx| {
                TextAreaState::new(window, cx)
                    .rows(3)
                    .placeholder("输入备注")
            }),
            password: cx.new(|cx| {
                TextFieldState::new(window, cx)
                    .masked(true)
                    .placeholder("Enter a sample password")
            }),
            account: cx.new(|cx| {
                let mut state = TextFieldState::new(window, cx).default_value("Demo");
                state.set_readonly(true, cx);
                state
            }),
            disabled: cx.new(|cx| {
                let mut state = TextFieldState::new(window, cx).default_value("Pending");
                state.set_disabled(true, cx);
                state
            }),
        }
    }
}

impl Render for TextFieldExamples {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        stack()
            .max_w(px(440.))
            .child(TextField::new("name", "姓名", &self.name).supporting_text("可使用微软拼音输入"))
            .child(
                TextField::new("email", "Email", &self.email)
                    .variant(TextFieldVariant::Outlined)
                    .error("Enter a domain after @"),
            )
            .child(
                TextField::multiline("notes", "备注", &self.notes)
                    .variant(TextFieldVariant::Outlined),
            )
            .child(TextField::secure("password", "Password", &self.password))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(TextField::new(
                        "account",
                        "Account (read only)",
                        &self.account,
                    ))
                    .child(TextField::new("status", "Account status", &self.disabled)),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn fields_preserve_native_editing_and_secure_copy(cx: &mut gpui::TestAppContext) {
    use gpui::{ClipboardItem, EntityInputHandler, WindowOptions};
    use gpui_material::MaterialRoot;

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| TextFieldExamples::new(window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let (name, password, account) = cx.update(|cx| {
        let view = view.read(cx);
        (
            view.name.clone(),
            view.password.clone(),
            view.account.clone(),
        )
    });
    cx.update_window(window.into(), |_, window, cx| {
        name.update(cx, |state, cx| state.focus(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
        name.update(cx, |state, cx| {
            state.replace_text_in_range(None, "🙂", window, cx);
            state.replace_and_mark_text_in_range(None, "zhong", Some(5..5), window, cx);
            assert_eq!(state.marked_text_range(window, cx), Some(2..7));
            state.replace_text_in_range(None, "中", window, cx);
            assert_eq!(state.marked_text_range(window, cx), None);
            assert_eq!(state.value().as_ref(), "🙂中");
        });
        account.update(cx, |state, cx| {
            state.replace_text_in_range(None, "changed", window, cx);
            assert_eq!(state.value().as_ref(), "Demo");
        });
        password.update(cx, |state, cx| state.focus(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.simulate_input(window.into(), "p6-password");
    cx.update_window(window.into(), |_, window, cx| {
        assert_eq!(password.read(cx).value().as_ref(), "p6-password");
        assert!(!password.read(cx).is_copyable());
        password.update(cx, |state, cx| state.select_all(window, cx));
        cx.write_to_clipboard(ClipboardItem::new_string("sentinel".into()));
    })
    .unwrap();
    let copy = if cfg!(target_os = "macos") {
        "cmd-c"
    } else {
        "ctrl-c"
    };
    cx.simulate_keystrokes(window.into(), copy);
    cx.update(|cx| {
        assert_eq!(
            cx.read_from_clipboard().unwrap().text().as_deref(),
            Some("sentinel")
        );
    });
}
