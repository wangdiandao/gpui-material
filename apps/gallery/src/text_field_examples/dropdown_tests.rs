use std::time::Duration;

use gpui::{
    AppContext, Context, ElementId, Entity, InputEvent as _, IntoElement, Modifiers, MouseButton,
    MouseDownEvent, MouseUpEvent, ParentElement, Render, Role, SharedString, Styled,
    TestAppContext, VisualTestContext, Window, WindowOptions, div, point, px,
};
use gpui_base::test_support::{find, snapshots};
use gpui_material::{
    DropdownOption, ExposedDropdown, ExposedDropdownState, MaterialRoot,
    foundation::{font_cache_revision, set_window_material_theme, window_material_theme},
};

use super::DropdownExamples;

#[gpui::test]
fn dropdown_filters_cancels_and_commits_to_owner(cx: &mut TestAppContext) {
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
        let field = find(window, &[], &(ElementId::from("city"), "combobox").into())
            .unwrap()
            .bounds();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: field.origin + point(px(24.), field.size.height / 2.),
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
    let options_bounds = |cx: &mut TestAppContext, state| {
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

#[gpui::test]
fn dropdown_modes_preserve_focus_and_show_caret_only_when_editable(cx: &mut TestAppContext) {
    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| DropdownExamples::new(window, cx))
        })
        .unwrap()
    });
    let click_field = |cx: &mut TestAppContext, id: &'static str| {
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
    let is_open = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            find(window, &[], &"options".into()).is_some()
        })
        .unwrap()
    };
    let assert_paint = |cx: &mut TestAppContext, id: &'static str, caret_visible| {
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
            point(menu.bounds().right() - px(16.), option.bounds().center().y)
        })
        .unwrap();
    VisualTestContext::from_window(window.into(), cx)
        .simulate_click(option_position, Modifiers::default());
    assert!(!is_open(cx));
    assert_eq!(assert_paint(cx, "language", false), "日本語");
    click_field(cx, "city");
    assert_paint(cx, "city", true);
}

#[gpui::test]
fn dropdown_keyboard_choice_scrolls_into_the_visible_list(cx: &mut TestAppContext) {
    struct Example(Entity<ExposedDropdownState>);
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(300.)).child(ExposedDropdown::new(
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
    let draw = |cx: &mut TestAppContext| {
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
    let assert_visible = |cx: &mut TestAppContext, id: &str| {
        cx.update_window(handle.into(), |_, window, _| {
            let list = find(window, &[], &"options-scroll-region".into()).unwrap();
            let choice = find(window, list.path(), &ElementId::from(id.to_owned())).unwrap();
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

#[gpui::test]
fn dropdown_virtualizes_full_model_navigation_and_preserves_identity(cx: &mut TestAppContext) {
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
            div().w(px(300.)).child(
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
    let draw = |cx: &mut TestAppContext| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    };
    let assert_row = |cx: &mut TestAppContext, id: &str| {
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
        let theme = window_material_theme(window, cx).clone();
        let before = font_cache_revision(cx);
        set_window_material_theme(Some(theme), window, cx);
        assert_ne!(before, font_cache_revision(cx));
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

#[gpui::test]
fn dropdown_keeps_multiline_row_measurements(cx: &mut TestAppContext) {
    struct Example(Entity<ExposedDropdownState>);
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(300.)).child(ExposedDropdown::new(
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
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let state = cx.new(|cx| ExposedDropdownState::new("Short", false, window, cx));
            cx.new(|_| Example(state))
        })
        .unwrap()
    });
    let draw = |cx: &mut TestAppContext| {
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
