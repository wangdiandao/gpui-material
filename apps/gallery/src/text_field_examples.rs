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
                section("Editable")
                    .description("Type 北京, 上海 or 香港 to filter the destinations.")
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
                    ),
            )
            .child(
                section("Selection only")
                    .description("Open the list to choose the display language.")
                    .child(ExposedDropdown::new(
                        "language",
                        "Display language",
                        &self.language,
                        [
                            DropdownOption::new("en", "English"),
                            DropdownOption::new("zh", "简体中文"),
                            DropdownOption::new("ja", "日本語"),
                        ],
                    )),
            )
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
        let field = find(
            window,
            &[],
            &(gpui::ElementId::from("city"), "combobox").into(),
        )
        .unwrap()
        .bounds();
        window.dispatch_event(
            gpui::MouseDownEvent {
                button: gpui::MouseButton::Left,
                position: field.origin + gpui::point(px(24.), field.size.height / 2.),
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
                section("Label at 150% scale").child(
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
        assert_eq!(
            label.bounds().left() - field.bounds().left(),
            px(12.),
            "4dp cutout padding must surround text aligned at the editor's 16dp inset"
        );
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
fn outlined_label_avoids_the_stroke_through_focus_transitions(cx: &mut gpui::TestAppContext) {
    use gpui::{InputEvent, MouseButton, WindowOptions, point};
    use gpui_base::test_support::find;
    use gpui_material::{
        foundation::{MaterialRoot, material_theme, set_material_theme},
        theme::Direction,
    };
    use std::time::Duration;

    let (handle, examples) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut examples = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| TextFieldExamples::new(window, cx));
            examples = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, examples.unwrap())
    });
    let sample = |cx: &mut gpui::TestAppContext, id: &'static str| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let field = find(window, &[], &id.into()).unwrap();
            let label = find(window, field.path(), &"label-text".into()).unwrap();
            let factor = window.scale_factor();
            let field = field.bounds().scale(factor);
            let label = label.bounds().scale(factor);
            let outline: Vec<_> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| {
                    quad.bounds.origin == gpui::point(field.left().round(), field.top().round())
                        && quad.border_widths.top > px(0.).scale(factor)
                })
                .collect();
            let border = outline.first().expect("outlined field stroke");
            let width = u32::from(border.border_widths.top);
            let top = border.bounds.top();
            if label.top() >= top + border.border_widths.top || label.bottom() <= top {
                return width;
            }
            for row in 0..width {
                let y = top + gpui::ScaledPixels(row as f32 + 0.5);
                for x in [
                    label.left() + px(5.).scale(factor),
                    label.right() - px(5.).scale(factor),
                ] {
                    assert!(
                        outline
                            .iter()
                            .all(|quad| !quad.content_mask.bounds.contains(&point(x, y))),
                        "{id}: stroke pixel row {row} crosses the label at scale {factor}"
                    );
                }
            }
            width
        })
        .unwrap()
    };
    for (direction, factor) in [(Direction::Ltr, 1.), (Direction::Rtl, 1.5)] {
        cx.simulate_window_scale_factor_change(handle.into(), factor);
        cx.update(|cx| {
            let mut theme = material_theme(cx).clone();
            theme.layout.direction = direction;
            set_material_theme(theme, cx);
        });
        for (id, value) in [
            ("email", "name@"),
            ("email", ""),
            ("notes", ""),
            ("notes", "有内容"),
        ] {
            cx.update_window(handle.into(), |_, window, cx| {
                if id == "email" {
                    let email = examples.read(cx).email.clone();
                    email.update(cx, |state, cx| state.replace_all(value, window, cx));
                } else {
                    let notes = examples.read(cx).notes.clone();
                    notes.update(cx, |state, cx| state.replace_all(value, window, cx));
                }
                window.blur(cx);
            })
            .unwrap();
            sample(cx, id);
            cx.executor().advance_clock(Duration::from_secs(1));
            let resting_width = sample(cx, id);
            cx.update_window(handle.into(), |_, window, cx| {
                let field = find(window, &[], &id.into()).unwrap();
                let position = point(field.bounds().center().x, field.bounds().top() + px(24.));
                window.dispatch_event(
                    gpui::MouseDownEvent {
                        button: MouseButton::Left,
                        position,
                        click_count: 1,
                        ..Default::default()
                    }
                    .to_platform_input(),
                    cx,
                );
                window.dispatch_event(
                    gpui::MouseUpEvent {
                        button: MouseButton::Left,
                        position,
                        click_count: 1,
                        ..Default::default()
                    }
                    .to_platform_input(),
                    cx,
                );
            })
            .unwrap();
            sample(cx, id);
            let mut focused_width = resting_width;
            for _ in 0..45 {
                cx.executor().advance_clock(Duration::from_millis(7));
                focused_width = sample(cx, id);
            }
            assert!(
                focused_width > resting_width,
                "{id}: the focus stroke did not grow"
            );
            cx.update_window(handle.into(), |_, window, cx| window.blur(cx))
                .unwrap();
            sample(cx, id);
            for _ in 0..45 {
                cx.executor().advance_clock(Duration::from_millis(7));
                sample(cx, id);
            }
            assert_eq!(
                sample(cx, id),
                resting_width,
                "{id}: the focus stroke did not return"
            );
        }
    }
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
    assert_eq!(floated, px(8.), "Compose filled label top padding");
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

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn dropdown_modes_preserve_focus_and_show_caret_only_when_editable(cx: &mut gpui::TestAppContext) {
    use gpui::{
        ElementId, InputEvent as _, MouseButton, MouseDownEvent, MouseUpEvent, WindowOptions,
    };
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;
    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| DropdownExamples::new(window, cx))
        })
        .unwrap()
    });
    let click_field = |cx: &mut gpui::TestAppContext, id: &'static str| {
        cx.update_window(window.into(), |_, window, cx| {
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);
            let trigger = find(window, &[], &(ElementId::from(id), "combobox").into()).unwrap();
            let position = trigger.bounds().center();
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
        })
        .unwrap();
    };
    let is_open = |cx: &mut gpui::TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &"options".into()).is_some()
        })
        .unwrap()
    };
    let assert_paint = |cx: &mut gpui::TestAppContext, id: &'static str, caret_visible| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let trigger = find(window, &[], &(ElementId::from(id), "combobox").into()).unwrap();
            let field = find(window, trigger.path(), &"value".into()).unwrap();
            assert_eq!(field.focused(), Some(true), "{id} retains focus");
            let factor = window.scale_factor();
            let bounds = field.bounds().scale(factor);
            let quads = window.painted_quads();
            assert!(
                quads.iter().any(|quad| {
                    quad.border_widths.top == px(2.).scale(factor)
                        && quad.bounds.top() == bounds.top().round()
                        && quad.bounds.left() == bounds.left().round()
                }),
                "{id} paints the focused outline"
            );
            let visible = quads.iter().any(|quad| {
                quad.background
                    .as_solid()
                    .is_some_and(|ink| ink.a >= 1. / 255.)
                    && quad.bounds.size.width <= px(2.).scale(factor)
                    && quad.bounds.size.height >= px(12.).scale(factor)
                    && bounds.contains(&quad.bounds.center())
            });
            assert_eq!(visible, caret_visible, "{id} caret visibility");
            field.value().unwrap_or_default().to_owned()
        })
        .unwrap()
    };
    click_field(cx, "language");
    assert!(is_open(cx), "the whole readonly field is the menu anchor");
    assert_paint(cx, "language", false);
    cx.simulate_keystrokes(window.into(), "space");
    assert!(!is_open(cx));
    cx.simulate_keystrokes(window.into(), "space");
    assert!(is_open(cx));
    cx.simulate_keystrokes(window.into(), "down down enter");
    assert!(!is_open(cx));
    assert_eq!(assert_paint(cx, "language", false), "简体中文");
    cx.simulate_keystrokes(window.into(), "space");
    cx.simulate_keystrokes(window.into(), "escape");
    assert!(!is_open(cx));
    click_field(cx, "language");
    let option_position = cx
        .update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let menu = find(window, &[], &"options-scroll-region".into()).unwrap();
            let option = find(window, menu.path(), &"ja".into()).unwrap();
            gpui::point(menu.bounds().right() - px(16.), option.bounds().center().y)
        })
        .unwrap();
    gpui::VisualTestContext::from_window(window.into(), cx)
        .simulate_click(option_position, gpui::Modifiers::default());
    assert!(!is_open(cx));
    assert_eq!(assert_paint(cx, "language", false), "日本語");
    click_field(cx, "city");
    assert_paint(cx, "city", true);
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn dropdown_keyboard_choice_scrolls_into_the_visible_list(cx: &mut gpui::TestAppContext) {
    use gpui::WindowOptions;
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;
    struct Example(Entity<ExposedDropdownState>);
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            gpui::div().w(px(300.)).child(ExposedDropdown::new(
                "long-menu",
                "Choose",
                &self.0,
                (0..20)
                    .map(|ix| DropdownOption::new(format!("option-{ix}"), format!("Option {ix}"))),
            ))
        }
    }
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let state = cx.new(|cx| ExposedDropdownState::new("Option 0", false, window, cx));
            cx.new(|_| Example(state))
        })
        .unwrap()
    });
    let draw = |cx: &mut gpui::TestAppContext| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    };
    draw(cx);
    cx.update_window(handle.into(), |_, window, cx| window.focus_next(cx))
        .unwrap();
    for _ in 0..16 {
        cx.simulate_keystrokes(handle.into(), "down");
        draw(cx);
    }
    let assert_visible = |cx: &mut gpui::TestAppContext, id: &str| {
        cx.update_window(handle.into(), |_, window, _| {
            let list = find(window, &[], &"options-scroll-region".into()).unwrap();
            let choice = find(window, list.path(), &gpui::ElementId::from(id.to_owned())).unwrap();
            assert!(choice.visible());
            assert!(choice.bounds().top() >= list.bounds().top());
            assert!(choice.bounds().bottom() <= list.bounds().bottom());
            assert_eq!(choice.bounds().size.height, px(48.));
        })
        .unwrap();
    };
    assert_visible(cx, "option-15");
    for _ in 0..15 {
        cx.simulate_keystrokes(handle.into(), "up");
        draw(cx);
    }
    assert_visible(cx, "option-0");
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn dropdown_virtualizes_full_model_navigation_and_preserves_identity(
    cx: &mut gpui::TestAppContext,
) {
    use gpui::{Role, SharedString, WindowOptions};
    use gpui_base::test_support::{find, snapshots};
    use gpui_material::MaterialRoot;

    struct Example {
        state: Entity<ExposedDropdownState>,
        reversed: bool,
        empty: bool,
        selected: Vec<SharedString>,
    }
    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity().downgrade();
            let count = if self.empty { 0 } else { 2_000 };
            let reversed = self.reversed;
            gpui::div().w(px(300.)).child(
                ExposedDropdown::new(
                    "large-menu",
                    "Choose",
                    &self.state,
                    (0..count).map(move |index| {
                        let index = if reversed { count - 1 - index } else { index };
                        DropdownOption::new(format!("option-{index}"), format!("Option {index}"))
                    }),
                )
                .on_select(move |choice, _, cx| {
                    owner
                        .update(cx, |view, cx| {
                            view.selected.push(choice.label.clone());
                            cx.notify();
                        })
                        .unwrap();
                }),
            )
        }
    }
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let state = cx.new(|cx| ExposedDropdownState::new("Option 0", false, window, cx));
            let entity = cx.new(|_| Example {
                state,
                reversed: false,
                empty: false,
                selected: vec![],
            });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });
    let draw = |cx: &mut gpui::TestAppContext| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    };
    let assert_row = |cx: &mut gpui::TestAppContext, id: &str| {
        cx.update_window(handle.into(), |_, window, _| {
            let rows: Vec<_> = snapshots(window)
                .into_iter()
                .filter(|node| node.role() == Some(Role::ListBoxOption))
                .collect();
            assert!(
                !rows.is_empty() && rows.len() <= 24,
                "only viewport and bounded overscan rows are built: {}",
                rows.len()
            );
            let list = find(window, &[], &"options-scroll-region".into()).unwrap();
            let row = find(window, list.path(), &id.to_owned().into()).unwrap();
            assert_eq!(row.selected(), Some(true));
            assert_eq!(row.bounds().size.height, px(48.));
            assert!(row.bounds().top() >= list.bounds().top());
            assert!(row.bounds().bottom() <= list.bounds().bottom());
            let editor = find(window, &[], &"value".into()).unwrap();
            assert_eq!(editor.focused(), Some(true));
        })
        .unwrap();
    };
    draw(cx);
    cx.update_window(handle.into(), |_, window, cx| window.focus_next(cx))
        .unwrap();
    cx.simulate_keystrokes(handle.into(), "down");
    draw(cx);
    assert_row(cx, "option-0");
    cx.simulate_keystrokes(handle.into(), "up");
    draw(cx);
    assert_row(cx, "option-1999");
    // Identical theme values still refresh fonts and invalidate row measurements.
    cx.update_window(handle.into(), |_, window, cx| {
        let theme = gpui_material::foundation::window_material_theme(window, cx).clone();
        let before = gpui_material::foundation::font_cache_revision(cx);
        gpui_material::foundation::set_window_material_theme(Some(theme), window, cx);
        assert_ne!(before, gpui_material::foundation::font_cache_revision(cx));
    })
    .unwrap();
    draw(cx);
    assert_row(cx, "option-1999");
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.reversed = true;
            cx.notify();
        })
    });
    draw(cx);
    assert_row(cx, "option-1999");
    cx.simulate_keystrokes(handle.into(), "enter");
    draw(cx);
    cx.update(|cx| {
        assert_eq!(
            view.read(cx).selected.as_slice(),
            &[SharedString::from("Option 1999")]
        )
    });
    cx.simulate_keystrokes(handle.into(), "space");
    draw(cx);
    cx.simulate_keystrokes(handle.into(), "down");
    draw(cx);
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.empty = true;
            cx.notify();
        })
    });
    draw(cx);
    cx.simulate_keystrokes(handle.into(), "down enter escape");
    draw(cx);
    cx.update(|cx| assert_eq!(view.read(cx).selected.len(), 1));
    cx.update_window(handle.into(), |_, window, _| {
        assert!(find(window, &[], &"options".into()).is_none());
        assert_eq!(
            find(window, &[], &"value".into()).unwrap().value(),
            Some("Option 1999")
        );
    })
    .unwrap();
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn dropdown_keeps_multiline_row_measurements(cx: &mut gpui::TestAppContext) {
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;
    struct Example(Entity<ExposedDropdownState>);
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            gpui::div().w(px(300.)).child(ExposedDropdown::new(
                "multiline-menu",
                "Choose",
                &self.0,
                [
                    DropdownOption::new("short", "Short"),
                    DropdownOption::new(
                        "multiline",
                        "First line\nSecond line\nThird line\nFourth line",
                    ),
                ],
            ))
        }
    }
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |window, cx| {
            let state = cx.new(|cx| ExposedDropdownState::new("Short", false, window, cx));
            cx.new(|_| Example(state))
        })
        .unwrap()
    });
    let draw = |cx: &mut gpui::TestAppContext| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    };
    draw(cx);
    cx.update_window(handle.into(), |_, window, cx| window.focus_next(cx))
        .unwrap();
    cx.simulate_keystrokes(handle.into(), "down");
    draw(cx);
    draw(cx);
    cx.update_window(handle.into(), |_, window, _| {
        let short = find(window, &[], &"short".into()).unwrap().bounds();
        let multiline = find(window, &[], &"multiline".into()).unwrap().bounds();
        assert_eq!(short.size.height, px(48.));
        assert!(multiline.size.height > short.size.height);
        assert_eq!(short.bottom(), multiline.top());
        let viewport = find(window, &[], &"options-scroll-region".into())
            .unwrap()
            .bounds();
        assert_eq!(short.size.width, viewport.size.width);
        assert_eq!(multiline.size.width, viewport.size.width);
        assert!(multiline.bottom() <= viewport.bottom());
    })
    .unwrap();
}
