use gpui::{AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, px};
use gpui_material::{TextAreaState, TextField, TextFieldState, TextFieldVariant};

use crate::story::{section, stack};

use gpui_material::{DropdownOption, ExposedDropdown, ExposedDropdownState, Text};

pub(crate) struct DropdownExamples {
    city: Entity<ExposedDropdownState>,
    language: Entity<ExposedDropdownState>,
    destination: gpui::SharedString,
}

impl DropdownExamples {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let city = cx.new(|cx| ExposedDropdownState::new("", true, window, cx));
        let language = cx.new(|cx| ExposedDropdownState::new("English", false, window, cx));
        cx.observe(&city, |_, _, cx| cx.notify()).detach();
        cx.observe(&language, |_, _, cx| cx.notify()).detach();
        Self {
            city,
            language,
            destination: "".into(),
        }
    }
}

impl Render for DropdownExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        stack()
            .max_w(px(440.))
            .child(
                ExposedDropdown::new(
                    "city",
                    "Destination",
                    &self.city,
                    [
                        DropdownOption::new("hk", "香港"),
                        DropdownOption::new("bj", "北京"),
                        DropdownOption::new("sh", "上海"),
                    ],
                )
                .on_select(move |option, _, cx| {
                    let _ = owner.update(cx, |view, cx| {
                        view.destination = option.label.clone();
                        cx.notify();
                    });
                }),
            )
            .child(ExposedDropdown::new(
                "language",
                "Display language",
                &self.language,
                [
                    DropdownOption::new("en", "English"),
                    DropdownOption::new("zh", "简体中文"),
                    DropdownOption::new("ja", "日本語"),
                ],
            ))
            .children(
                (!self.destination.is_empty())
                    .then(|| Text::new(format!("Destination selected: {}", self.destination))),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn dropdown_filters_cancels_and_commits_to_owner(cx: &mut gpui::TestAppContext) {
    use gpui::{InputEvent as _, WindowOptions};
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;
    use std::time::Duration;

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| DropdownExamples::new(window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let focus = window.focused(cx);
        if focus.is_none() {
            window.focus_next(cx);
        }
        window.dispatch_event(
            gpui::MouseDownEvent {
                button: gpui::MouseButton::Left,
                position: gpui::point(px(60.), px(36.)),
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
    cx.simulate_input(window.into(), "北");
    cx.run_until_parked();
    cx.simulate_keystrokes(window.into(), "escape");
    cx.update(|cx| assert!(view.read(cx).destination.is_empty()));
    cx.simulate_input(window.into(), "上");
    cx.run_until_parked();
    let options_bounds = |cx: &mut gpui::TestAppContext, state| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &state).map(|row| row.bounds())
        })
        .unwrap()
    };
    let collapsed = options_bounds(cx, "options".into()).unwrap();
    cx.executor().advance_clock(Duration::from_millis(250));
    let expanded = options_bounds(cx, "options".into()).unwrap();
    assert!(collapsed.size.width < expanded.size.width);
    assert!(collapsed.size.height < expanded.size.height);
    assert_eq!(collapsed.top(), expanded.top());
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "enter");
    cx.update(|cx| assert_eq!(view.read(cx).destination.as_ref(), "上海"));
    assert!(options_bounds(cx, "options-exit".into()).is_some());
    cx.executor().advance_clock(Duration::from_millis(60));
    assert!(
        options_bounds(cx, "options-exit".into())
            .unwrap()
            .size
            .width
            < expanded.size.width
    );
    cx.executor().advance_clock(Duration::from_millis(250));
    assert!(options_bounds(cx, "options-exit".into()).is_none());
}

pub(crate) struct TextFieldExamples {
    name: Entity<TextFieldState>,
    email: Entity<TextFieldState>,
    notes: Entity<TextAreaState>,
    password: Entity<TextFieldState>,
    account: Entity<TextFieldState>,
    disabled: Entity<TextFieldState>,
    scaled: Entity<TextFieldState>,
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
            scaled: cx.new(|cx| TextFieldState::new(window, cx)),
        }
    }
}

impl Render for TextFieldExamples {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        stack()
            .max_w(px(440.))
            .child(
                section("Filled and outlined")
                    .child(
                        TextField::new("name", "姓名", &self.name)
                            .supporting_text("可使用微软拼音输入"),
                    )
                    .child(
                        TextField::new("email", "Email", &self.email)
                            .variant(TextFieldVariant::Outlined)
                            .error("Enter a domain after @"),
                    ),
            )
            .child(
                section("Multiline and secure")
                    .child(
                        TextField::multiline("notes", "备注", &self.notes)
                            .variant(TextFieldVariant::Outlined),
                    )
                    .child(TextField::secure("password", "Password", &self.password)),
            )
            .child(
                section("Readonly and disabled")
                    .child(TextField::new(
                        "account",
                        "Account (readonly)",
                        &self.account,
                    ))
                    .child(TextField::new("status", "Account status", &self.disabled)),
            )
            .child(
                section("Scaled label").child(
                    TextField::new(
                        "scaled-label",
                        "Preferred name for project correspondence",
                        &self.scaled,
                    )
                    .font_scale(1.5)
                    .supporting_text("用于项目沟通，可输入中英文"),
                ),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn scaled_text_field_wraps_long_label_without_clipping(cx: &mut gpui::TestAppContext) {
    use gpui::{WindowOptions, size};
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;

    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| TextFieldExamples::new(window, cx))
        })
        .unwrap()
    });
    cx.simulate_window_resize(window.into(), size(px(290.), px(800.)));
    cx.simulate_window_scale_factor_change(window.into(), 1.5);
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let scaled = find(window, &[], &"scaled-label".into()).unwrap();
        let baseline = find(window, &[], &"name".into()).unwrap();
        assert!(scaled.visible());
        assert!(scaled.bounds().size.height > baseline.bounds().size.height);
        assert!(scaled.bounds().right() <= window.viewport_size().width);
        assert_eq!(
            scaled.label(),
            Some("Preferred name for project correspondence")
        );
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn outlined_floating_label_cuts_the_top_border(cx: &mut gpui::TestAppContext) {
    use gpui::{WindowOptions, point, px};
    use gpui_base::test_support::find;
    use gpui_material::foundation::MaterialRoot;

    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| TextFieldExamples::new(window, cx))
        })
        .unwrap()
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let field = find(window, &[], &"email".into()).unwrap();
        let label = find(window, field.path(), &"label".into()).unwrap();
        let factor = window.scale_factor();
        // The outlined example starts with a value, so its label overlaps the top stroke.
        let y = field.bounds().top() + px(0.25);
        let inside_label = point(label.bounds().left() + px(9.), y).scale(factor);
        let before_label = point(field.bounds().left() + px(10.), y).scale(factor);
        let outline: Vec<_> = window
            .painted_quads()
            .into_iter()
            .filter(|quad| {
                quad.bounds.origin.y == field.bounds().origin.y.scale(factor)
                    && quad.border_widths.top > px(0.).scale(factor)
            })
            .collect();
        assert!(
            outline
                .iter()
                .any(|quad| quad.content_mask.bounds.contains(&before_label))
        );
        assert!(
            outline
                .iter()
                .all(|quad| !quad.content_mask.bounds.contains(&inside_label))
        );
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn empty_text_field_floats_its_label_on_focus(cx: &mut gpui::TestAppContext) {
    use gpui::WindowOptions;
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;
    use std::time::Duration;

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let examples = cx.new(|cx| TextFieldExamples::new(window, cx));
            view = Some(examples.clone());
            examples
        })
        .unwrap();
        (window, view.unwrap())
    });
    let sample = |cx: &mut gpui::TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let field = find(window, &[], &"name".into()).unwrap();
            let label = find(window, field.path(), &"label".into()).unwrap();
            label.bounds().top() - field.bounds().top()
        })
        .unwrap()
    };
    let rest = sample(cx);
    cx.update_window(window.into(), |_, window, cx| {
        view.read(cx)
            .name
            .read(cx)
            .presentation()
            .focus_handle()
            .clone()
            .focus(window, cx);
    })
    .unwrap();
    assert_eq!(sample(cx), rest);
    cx.executor().advance_clock(Duration::from_millis(40));
    let middle = sample(cx);
    assert!(middle < rest);
    cx.executor().advance_clock(Duration::from_secs(1));
    let floated = sample(cx);
    assert!(floated < middle);
    cx.update_window(window.into(), |_, window, cx| {
        cx.set_reduce_motion(true);
        window.blur(cx);
    })
    .unwrap();
    assert_eq!(sample(cx), rest);
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
