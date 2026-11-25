//! End-to-end public-facade UI test example referenced by `docs/integration.md`.

use gpui::{
    AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Role,
    SharedString, StatefulInteractiveElement, Styled, TestAppContext, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use gpui_material::{
    Button, Checkbox, CheckboxState, MaterialRoot, MaterialTheme, TextField, TextFieldState,
    TextFieldVariant,
    test::{TestSupportExt as _, TestWindowExt as _},
};

struct ProfileForm {
    name: Entity<TextFieldState>,
    agreed: bool,
    submitted: Option<SharedString>,
}

impl ProfileForm {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name = cx.new(|cx| TextFieldState::new(window, cx).placeholder("Ada Lovelace"));
        Self {
            name,
            agreed: false,
            submitted: None,
        }
    }
}

impl Render for ProfileForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.submitted.as_ref().map_or_else(
            || SharedString::from("Not saved"),
            |name| SharedString::from(format!("Saved: {name}")),
        );
        let form = cx.entity();
        let save_form = cx.entity();
        let checkbox_state = if self.agreed {
            CheckboxState::Checked
        } else {
            CheckboxState::Unchecked
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .p_4()
            .gap_4()
            .child(
                TextField::new("name", "Profile name", &self.name)
                    .variant(TextFieldVariant::Outlined),
            )
            .child(
                Checkbox::new("agree", "Accept terms", checkbox_state).on_change(
                    move |next, _, cx| {
                        form.update(cx, |this, cx| {
                            this.agreed = next == CheckboxState::Checked;
                            cx.notify();
                        });
                    },
                ),
            )
            .child(
                Button::new("save", "Save")
                    .disabled(!self.agreed)
                    .on_activate(move |_, cx| {
                        save_form.update(cx, |this, cx| {
                            this.submitted = Some(this.name.read(cx).value());
                            cx.notify();
                        });
                    }),
            )
            .child(
                div()
                    .id("status")
                    .role(Role::Status)
                    .test_support()
                    .aria_label(status.clone())
                    .child(status),
            )
    }
}

#[gpui::test]
fn saves_a_profile_through_the_public_facade(cx: &mut TestAppContext) {
    let (handle, form) = cx.update(|cx| {
        gpui_material::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
        let mut entity = None;
        let bounds = gpui::Bounds {
            origin: gpui::Point::default(),
            size: size(px(640.), px(480.)),
        };
        let handle = MaterialRoot::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            cx,
            |window, cx| {
                let view = cx.new(|cx| ProfileForm::new(window, cx));
                entity = Some(view.clone());
                view
            },
        )
        .expect("open facade test window");
        (handle, entity.unwrap())
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);

        assert_eq!(window.find("status").role(), Some(Role::Status));
        assert_eq!(window.find("status").label(), Some("Not saved"));

        // Save is disabled until terms are accepted; clicking it produces no submit.
        window.click("save", cx);
        assert_eq!(window.find("status").label(), Some("Not saved"));

        window.click("name", cx);
        window.input("Ada José", cx);
        let name = window.find("name");
        assert_eq!(name.role(), Some(Role::TextInput));
        assert_eq!(name.label(), Some("Profile name"));
        assert_eq!(name.value(), Some("Ada José"));
        assert!(name.bounds().size.width > px(0.));

        window.press("backspace", cx);
        assert_eq!(window.find("name").value(), Some("Ada Jos"));

        let before_agree = window.find("agree");
        window.click("agree", cx);
        assert_eq!(before_agree.checked(), Some(false));
        assert_eq!(window.find("agree").checked(), Some(true));

        window.click("save", cx);
        let status = window.find("status");
        assert!(status.visible());
        assert_eq!(status.label(), Some("Saved: Ada Jos"));
        assert!(status.bounds().top() >= window.find("save").bounds().bottom());
    })
    .unwrap();

    cx.update(|cx| {
        assert_eq!(form.read(cx).submitted.as_deref(), Some("Ada Jos"));
    });
}
