use gpui::{
    AppContext, ClipboardItem, Context, ElementId, Entity, EntityInputHandler, Focusable,
    InputEvent, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Render,
    Role, Styled, TestAppContext, Window, WindowOptions, div, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    MaterialRoot, TextField, TextFieldMaskPattern, TextFieldState, TextFieldVariant,
    material_theme, set_material_theme, theme::Direction,
};

use super::super::TextFieldExamples;

#[gpui::test]
fn decorated_field_keeps_its_editor_value_and_accessible_metadata(cx: &mut TestAppContext) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let examples = cx.new(|cx| TextFieldExamples::new(window, cx));
            view = Some(examples.clone());
            examples
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(560.), px(1000.)));
    let verify = |cx: &mut TestAppContext, direction, value, description| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let field = find(window, &[], &"amount".into()).unwrap();
            assert_eq!(field.role(), Some(Role::TextInput));
            assert_eq!(field.label(), Some("Budget amount"));
            assert_eq!(field.placeholder(), Some("Amount"));
            assert_eq!(field.value(), Some(value));
            assert_eq!(field.description(), Some(description));
            assert_eq!(
                find(window, field.path(), &"placeholder".into()).is_some(),
                value.is_empty(),
            );
            let label = find(window, field.path(), &"label".into()).unwrap();
            let prefix = find(window, field.path(), &"currency-prefix".into()).unwrap();
            let suffix = find(window, field.path(), &"currency-suffix".into()).unwrap();
            assert!(label.bounds().bottom() <= prefix.bounds().top());
            if direction == Direction::Ltr {
                assert!(prefix.bounds().right() < suffix.bounds().left());
            } else {
                assert!(suffix.bounds().right() < prefix.bounds().left());
            }
            let password = find(window, &[], &"password".into()).unwrap();
            assert_eq!(password.role(), Some(Role::PasswordInput));
            assert_eq!(password.value(), None);
            assert_eq!(password.description(), None);
            assert_eq!(
                find(window, password.path(), &"placeholder".into()).is_some(),
                value.is_empty(),
            );
        })
        .unwrap();
    };
    verify(cx, Direction::Ltr, "", "Amount in US dollars. 0 / 8");
    cx.update_window(window.into(), |_, window, cx| {
        let field = find(window, &[], &"amount".into()).unwrap();
        let position = find(window, field.path(), &"placeholder".into())
            .unwrap()
            .bounds()
            .center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.refresh();
        window.draw(cx).clear(cx);
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        assert!(
            view.read(cx)
                .amount
                .read(cx)
                .presentation()
                .focus_handle()
                .is_focused(window)
        );
    })
    .unwrap();
    cx.simulate_input(window.into(), "１２🙂");
    cx.update_window(window.into(), |_, window, cx| {
        let mut theme = material_theme(cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_material_theme(theme, cx);
        let password = view.read(cx).password.clone();
        password.update(cx, |state, cx| state.replace_all("masked", window, cx));
    })
    .unwrap();
    verify(cx, Direction::Rtl, "１２🙂", "Amount in US dollars. 3 / 8");
}

#[gpui::test]
fn fields_preserve_native_editing_and_secure_copy(cx: &mut TestAppContext) {
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
    cx.simulate_input(window.into(), "sample-password");
    cx.update_window(window.into(), |_, window, cx| {
        assert_eq!(password.read(cx).value().as_ref(), "sample-password");
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

#[gpui::test]
fn secure_field_toggles_obfuscation_mode_and_clipboard_policy(cx: &mut TestAppContext) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| TextFieldExamples::new(window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(560.), px(1200.)));
    let password = cx.update(|cx| view.read(cx).password.clone());
    let copy = if cfg!(target_os = "macos") {
        "cmd-c"
    } else {
        "ctrl-c"
    };
    let click_toggle = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let field = find(window, &[], &"password".into()).unwrap();
            let toggle = find(window, field.path(), &"password-visibility".into())
                .unwrap()
                .bounds()
                .center();
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position: toggle,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    position: toggle,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    };

    cx.update_window(window.into(), |_, window, cx| {
        password.update(cx, |state, cx| {
            state.replace_all("s3cr3t!", window, cx);
            state.focus(window, cx);
            state.select_all(window, cx);
        });
        window.refresh();
        window.draw(cx).clear(cx);
        let field = find(window, &[], &"password".into()).unwrap();
        assert_eq!(field.role(), Some(Role::PasswordInput));
        assert_eq!(field.value(), None);
        assert_eq!(field.description(), None);
        assert!(!password.read(cx).is_copyable());
        let (ime_text, adjusted) = password.update(cx, |state, cx| {
            let mut adjusted = None;
            let text =
                gpui::EntityInputHandler::text_for_range(state, 0..7, &mut adjusted, window, cx);
            (text, adjusted)
        });
        assert_eq!(ime_text.as_deref(), Some("•••••••"));
        assert_eq!(adjusted, Some(0..7));
        cx.write_to_clipboard(ClipboardItem::new_string("sentinel".into()));
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), copy);
    cx.update(|cx| {
        assert_eq!(
            cx.read_from_clipboard().unwrap().text().as_deref(),
            Some("sentinel")
        );
    });

    click_toggle(cx);
    cx.update_window(window.into(), |_, window, cx| {
        let field = find(window, &[], &"password".into()).unwrap();
        assert_eq!(field.role(), Some(Role::TextInput));
        assert_eq!(field.value(), Some("s3cr3t!"));
        assert_eq!(field.description(), Some("7 / 24"));
        assert!(password.read(cx).is_copyable());
        let (ime_text, adjusted) = password.update(cx, |state, cx| {
            let mut adjusted = None;
            let text =
                gpui::EntityInputHandler::text_for_range(state, 0..7, &mut adjusted, window, cx);
            (text, adjusted)
        });
        assert_eq!(ime_text.as_deref(), Some("s3cr3t!"));
        assert_eq!(adjusted, Some(0..7));
        password.update(cx, |state, cx| {
            state.focus(window, cx);
            state.select_all(window, cx);
        });
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), copy);
    cx.update(|cx| {
        assert_eq!(
            cx.read_from_clipboard().unwrap().text().as_deref(),
            Some("s3cr3t!")
        );
    });

    click_toggle(cx);
    cx.update_window(window.into(), |_, window, cx| {
        let field = find(window, &[], &"password".into()).unwrap();
        assert_eq!(field.role(), Some(Role::PasswordInput));
        assert_eq!(field.value(), None);
        assert_eq!(field.description(), None);
        assert!(!password.read(cx).is_copyable());
        let (ime_text, adjusted) = password.update(cx, |state, cx| {
            let mut adjusted = None;
            let text =
                gpui::EntityInputHandler::text_for_range(state, 0..7, &mut adjusted, window, cx);
            (text, adjusted)
        });
        assert_eq!(ime_text.as_deref(), Some("•••••••"));
        assert_eq!(adjusted, Some(0..7));
        password.update(cx, |state, cx| {
            state.focus(window, cx);
            state.select_all(window, cx);
        });
        cx.write_to_clipboard(ClipboardItem::new_string("sentinel-2".into()));
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), copy);
    cx.update(|cx| {
        assert_eq!(
            cx.read_from_clipboard().unwrap().text().as_deref(),
            Some("sentinel-2")
        );
    });
}

#[gpui::test]
fn masked_pattern_and_validator_enforce_formatting_and_undo(cx: &mut TestAppContext) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| TextFieldExamples::new(window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(560.), px(1400.)));
    let (phone, code) = cx.update(|cx| {
        let view = view.read(cx);
        (view.phone.clone(), view.code.clone())
    });
    let undo = if cfg!(target_os = "macos") {
        "cmd-z"
    } else {
        "ctrl-z"
    };

    cx.update_window(window.into(), |_, window, cx| {
        phone.update(cx, |state, cx| state.focus(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.simulate_input(window.into(), "4155550199");
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(phone.read(cx).value().as_ref(), "(415) 555-0199");
        assert_eq!(phone.read(cx).unmask_value().as_ref(), "4155550199");
        let field = find(window, &[], &"phone".into()).unwrap();
        assert_eq!(field.value(), Some("(415) 555-0199"));
        assert_eq!(field.description(), Some("Unmasked value: 4155550199"));
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), undo);
    cx.update(|cx| {
        assert!(phone.read(cx).unmask_value().len() < 10);
    });

    cx.update_window(window.into(), |_, window, cx| {
        let number = cx.new(|cx| {
            TextFieldState::new(window, cx).mask_pattern(TextFieldMaskPattern::number(Some(',')))
        });
        number.update(cx, |state, cx| state.replace_all("12345", window, cx));
        assert_eq!(number.read(cx).value().as_ref(), "12,345");
        assert_eq!(number.read(cx).unmask_value().as_ref(), "12345");

        code.update(cx, |state, cx| {
            state.replace_text_in_range(None, "AB12", window, cx);
            assert_eq!(state.value().as_ref(), "AB12");
            state.replace_text_in_range(None, "-", window, cx);
            assert_eq!(state.value().as_ref(), "AB12");
            state.replace_all("AB12345", window, cx);
            assert_eq!(state.value().as_ref(), "AB12");
            state.replace_all("AB1234", window, cx);
            assert_eq!(state.value().as_ref(), "AB1234");
        });
        window.refresh();
        window.draw(cx).clear(cx);
        let field = find(window, &[], &"code".into()).unwrap();
        assert_eq!(field.value(), Some("AB1234"));
        assert_eq!(
            field.description(),
            Some("Rejects non-alphanumeric characters and excess length. 6 / 6")
        );
    })
    .unwrap();
}

#[gpui::test]
fn text_field_visibility_toggle_syncs_obfuscation_and_toggles_labels(cx: &mut TestAppContext) {
    struct SecretView {
        secret: Entity<TextFieldState>,
        disabled_secret: Entity<TextFieldState>,
        plain: Entity<TextFieldState>,
        visible: bool,
        disabled_clicks: usize,
    }

    impl Render for SecretView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity();
            let dis_view = cx.entity();
            div()
                .w(px(360.))
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    TextField::secure("secret-field", "Secret", &self.secret)
                        .variant(TextFieldVariant::Outlined)
                        .visibility_toggle(self.visible, move |next, _, cx| {
                            view.update(cx, |this, cx| {
                                this.visible = next;
                                cx.notify();
                            });
                        }),
                )
                .child(
                    TextField::secure("disabled-secret", "Locked secret", &self.disabled_secret)
                        .visibility_toggle(false, move |_, _, cx| {
                            dis_view.update(cx, |this, _| this.disabled_clicks += 1);
                        }),
                )
                .child(
                    TextField::new("plain-field", "Plain", &self.plain)
                        .visibility_toggle(false, |_, _, _| {}),
                )
        }
    }

    let mut view_handle = None;
    let mut secret_entity = None;
    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let secret = cx.new(|cx| {
                TextFieldState::new(window, cx)
                    .masked(true)
                    .default_value("hunter2")
            });
            let disabled_secret = cx.new(|cx| {
                let mut state = TextFieldState::new(window, cx)
                    .masked(true)
                    .default_value("locked");
                state.set_disabled(true, cx);
                state
            });
            let plain = cx.new(|cx| TextFieldState::new(window, cx).default_value("abc"));
            secret_entity = Some(secret.clone());
            let entity = cx.new(|_| SecretView {
                secret,
                disabled_secret,
                plain,
                visible: false,
                disabled_clicks: 0,
            });
            view_handle = Some(entity.clone());
            entity
        })
        .unwrap()
    });
    let view = view_handle.unwrap();
    let secret = secret_entity.unwrap();

    let toggle_id = |id: &'static str| ElementId::from((ElementId::from(id), "visibility-toggle"));
    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);

        // Non-secure field ignores visibility_toggle.
        assert!(find(window, &[], &toggle_id("plain-field")).is_none());

        // Secure field starts masked with "Show password" label.
        let field = find(window, &[], &"secret-field".into()).unwrap();
        assert_eq!(field.role(), Some(Role::PasswordInput));
        assert_eq!(field.value(), None);
        let toggle = find(window, &[], &toggle_id("secret-field")).unwrap();
        assert_eq!(toggle.label(), Some("Show password"));

        // Click toggle to reveal password.
        let pos = toggle.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: pos,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: pos,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.refresh();
        window.draw(cx).clear(cx);

        assert!(view.read(cx).visible);
        let field = find(window, &[], &"secret-field".into()).unwrap();
        assert_eq!(field.role(), Some(Role::TextInput));
        assert_eq!(field.value(), Some("hunter2"));
        assert!(secret.read(cx).focus_handle(cx).is_focused(window));
        assert_eq!(
            find(window, &[], &toggle_id("secret-field"))
                .unwrap()
                .label(),
            Some("Hide password")
        );

        // Disabled secure field disables its toggle and ignores clicks.
        let dis_toggle = find(window, &[], &toggle_id("disabled-secret")).unwrap();
        assert_eq!(dis_toggle.disabled(), Some(true));
        let dis_pos = dis_toggle.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: dis_pos,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: dis_pos,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(view.read(cx).disabled_clicks, 0);
    })
    .unwrap();
}
