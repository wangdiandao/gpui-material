use std::rc::Rc;

use gpui::{
    AppContext, Context, ElementId, Entity, Focusable, InputEvent, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Render, Styled, TestAppContext,
    Window, WindowOptions, div, px, size,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{
    MaterialRoot, MaterialTheme, TextField, TextFieldState,
    foundation::set_window_material_theme,
    theme::{Appearance, Direction},
};

#[gpui::test]
fn text_field_clearable_shows_when_editable_nonempty_clears_and_keeps_focus(
    cx: &mut TestAppContext,
) {
    struct ClearableView {
        editable: Entity<TextFieldState>,
        readonly: Entity<TextFieldState>,
        disabled: Entity<TextFieldState>,
    }

    impl Render for ClearableView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(360.))
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    TextField::outlined("query", "Search", &self.editable)
                        .clearable(true)
                        .trailing(div().id("custom-end").test_support().child("GO")),
                )
                .child(TextField::outlined("ro", "Readonly", &self.readonly).clearable(true))
                .child(TextField::outlined("dis", "Disabled", &self.disabled).clearable(true))
        }
    }

    let mut states = None;
    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let editable = cx.new(|cx| TextFieldState::new(window, cx));
            let readonly = cx.new(|cx| {
                let mut state = TextFieldState::new(window, cx).default_value("Fixed");
                state.set_readonly(true, cx);
                state
            });
            let disabled = cx.new(|cx| {
                let mut state = TextFieldState::new(window, cx).default_value("Locked");
                state.set_disabled(true, cx);
                state
            });
            states = Some(editable.clone());
            cx.new(|_| ClearableView {
                editable,
                readonly,
                disabled,
            })
        })
        .unwrap()
    });
    let editable = states.unwrap();

    let clear_id = |id: &'static str| ElementId::from((ElementId::from(id), "clear-button"));
    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);

        // Hidden when empty, readonly, or disabled.
        assert!(find(window, &[], &clear_id("query")).is_none());
        assert!(find(window, &[], &clear_id("ro")).is_none());
        assert!(find(window, &[], &clear_id("dis")).is_none());

        // Populating text reveals the clear button with default "Clear" accessible label.
        editable.update(cx, |state, cx| state.set_value("hello", window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
        let clear = find(window, &[], &clear_id("query")).expect("clear button visible");
        assert_eq!(clear.label(), Some("Clear"));
        let custom_end = find(window, &[], &"custom-end".into()).unwrap();
        assert!(
            clear.bounds().center().x < custom_end.bounds().center().x,
            "clear button should precede custom trailing in LTR"
        );

        // Clicking clear empties the editor, hides the button, and keeps focus on the editor.
        let pos = clear.bounds().center();
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
        assert_eq!(editable.read(cx).value(), "");
        assert!(find(window, &[], &clear_id("query")).is_none());
        assert!(
            editable.read(cx).focus_handle(cx).is_focused(window),
            "editor must retain focus after clear"
        );

        // In RTL, clear button sits to the right of custom trailing (logical order mirrored).
        let mut rtl = MaterialTheme::baseline(Appearance::Light);
        rtl.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(rtl), window, cx);
        editable.update(cx, |state, cx| state.set_value("rtl", window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
        let clear_rtl = find(window, &[], &clear_id("query")).unwrap();
        let custom_rtl = find(window, &[], &"custom-end".into()).unwrap();
        assert!(
            clear_rtl.bounds().center().x > custom_rtl.bounds().center().x,
            "clear button should precede custom trailing in RTL"
        );
    })
    .unwrap();
}

#[gpui::test]
fn text_field_focus_boundaries_outside_blur_trailing_controls_and_disable(cx: &mut TestAppContext) {
    use gpui::{FocusHandle, MouseMoveEvent, Subscription};
    use gpui_material::{
        Button, DropdownOption, ExposedDropdown, ExposedDropdownState, TextFieldEvent,
        TextFieldLabelPosition,
    };
    use std::cell::Cell;

    struct FocusHarness {
        primary: Entity<TextFieldState>,
        secondary: Entity<TextFieldState>,
        secure: Entity<TextFieldState>,
        revealed: bool,
        dropdown: Entity<ExposedDropdownState>,
        outside_button: FocusHandle,
        primary_blurs: Rc<Cell<usize>>,
        secure_blurs: Rc<Cell<usize>>,
        _subs: [Subscription; 2],
    }

    impl FocusHarness {
        fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
            let primary = cx.new(|cx| TextFieldState::new(window, cx).default_value("hello"));
            let secondary = cx.new(|cx| TextFieldState::new(window, cx));
            let secure = cx.new(|cx| {
                TextFieldState::new(window, cx)
                    .masked(true)
                    .default_value("secret")
            });
            let dropdown = cx.new(|cx| ExposedDropdownState::new("Alpha", true, window, cx));
            let primary_blurs = Rc::new(Cell::new(0));
            let secure_blurs = Rc::new(Cell::new(0));
            let primary_blurs_sub = primary_blurs.clone();
            let secure_blurs_sub = secure_blurs.clone();
            let primary_sub = cx.subscribe(&primary, move |_, _, event: &TextFieldEvent, _| {
                if matches!(event, TextFieldEvent::Blur) {
                    primary_blurs_sub.set(primary_blurs_sub.get() + 1);
                }
            });
            let secure_sub = cx.subscribe(&secure, move |_, _, event: &TextFieldEvent, _| {
                if matches!(event, TextFieldEvent::Blur) {
                    secure_blurs_sub.set(secure_blurs_sub.get() + 1);
                }
            });
            Self {
                primary,
                secondary,
                secure,
                revealed: false,
                dropdown,
                outside_button: cx.focus_handle(),
                primary_blurs,
                secure_blurs,
                _subs: [primary_sub, secure_sub],
            }
        }
    }

    impl Render for FocusHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity().downgrade();
            div()
                .size_full()
                .p_4()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    TextField::outlined("primary", "Primary", &self.primary)
                        .label_position(TextFieldLabelPosition::Above)
                        .supporting_text("Helper line")
                        .clearable(true),
                )
                .child(TextField::outlined(
                    "secondary",
                    "Secondary",
                    &self.secondary,
                ))
                .child(
                    TextField::secure("secure", "Password", &self.secure).visibility_toggle(
                        self.revealed,
                        move |next, _, cx| {
                            let _ = owner.update(cx, |this, cx| {
                                this.revealed = next;
                                cx.notify();
                            });
                        },
                    ),
                )
                .child(ExposedDropdown::new(
                    "combo",
                    "Choice",
                    &self.dropdown,
                    [
                        DropdownOption::new("alpha", "Alpha"),
                        DropdownOption::new("beta", "Beta"),
                    ],
                ))
                .child(Button::filled("outside-btn", "Outside").track_focus(&self.outside_button))
                .child(div().id("blank-space").test_support().h(px(80.)).w_full())
        }
    }

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let harness = cx.new(|cx| FocusHarness::new(window, cx));
            view = Some(harness.clone());
            harness
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(560.), px(760.)));

    let draw = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    let click = |cx: &mut TestAppContext, position| {
        cx.update_window(window.into(), |_, window, cx| {
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
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    let is_focused = |cx: &mut TestAppContext, state: &Entity<TextFieldState>| {
        cx.update_window(window.into(), |_, window, cx| {
            state
                .read(cx)
                .presentation()
                .focus_handle()
                .is_focused(window)
        })
        .unwrap()
    };

    draw(cx);
    let (primary, secondary, secure, dropdown, outside_btn, primary_blurs, secure_blurs) = cx
        .update(|cx| {
            let harness = view.read(cx);
            (
                harness.primary.clone(),
                harness.secondary.clone(),
                harness.secure.clone(),
                harness.dropdown.clone(),
                harness.outside_button.clone(),
                harness.primary_blurs.clone(),
                harness.secure_blurs.clone(),
            )
        });

    let (
        primary_container,
        primary_label,
        primary_supporting,
        primary_clear,
        secondary_container,
        secure_container,
        secure_toggle,
        blank_center,
        outside_btn_center,
    ) = cx
        .update_window(window.into(), |_, window, _| {
            let p = find(window, &[], &"primary".into()).unwrap();
            let s = find(window, &[], &"secondary".into()).unwrap();
            let sec = find(window, &[], &"secure".into()).unwrap();
            (
                find(window, p.path(), &"container".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(window, p.path(), &"label".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(window, p.path(), &"supporting".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(
                    window,
                    p.path(),
                    &(ElementId::from("primary"), "clear-button").into(),
                )
                .unwrap()
                .bounds()
                .center(),
                find(window, s.path(), &"container".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(window, sec.path(), &"container".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(
                    window,
                    sec.path(),
                    &(ElementId::from("secure"), "visibility-toggle").into(),
                )
                .unwrap()
                .bounds()
                .center(),
                find(window, &[], &"blank-space".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(window, &[], &"outside-btn".into())
                    .unwrap()
                    .bounds()
                    .center(),
            )
        })
        .unwrap();

    // 1. Clicking supporting row while unfocused does NOT focus the editor.
    click(cx, primary_supporting);
    assert!(!is_focused(cx, &primary));

    // 2. Clicking the Above label focuses the editor and clicking it again does not blur.
    click(cx, primary_label);
    assert!(is_focused(cx, &primary));
    assert_eq!(primary_blurs.get(), 0);
    click(cx, primary_label);
    assert!(is_focused(cx, &primary));
    assert_eq!(primary_blurs.get(), 0);

    // 3. Clicking the built-in clear button clears without an intermediate Blur.
    click(cx, primary_clear);
    assert!(is_focused(cx, &primary));
    assert_eq!(cx.update(|cx| primary.read(cx).value()), "");
    assert_eq!(primary_blurs.get(), 0);

    // 4. Dragging from inside the editor out to blank space keeps focus.
    cx.update_window(window.into(), |_, window, cx| {
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: primary_container,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseMoveEvent {
                position: blank_center,
                pressed_button: Some(MouseButton::Left),
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: blank_center,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    assert!(is_focused(cx, &primary));
    assert_eq!(primary_blurs.get(), 0);

    // 5. Clicking blank space outside blurs the focused editor.
    click(cx, blank_center);
    assert!(!is_focused(cx, &primary));
    assert_eq!(primary_blurs.get(), 1);

    // 6. Clicking supporting row while focused blurs the editor.
    click(cx, primary_container);
    assert!(is_focused(cx, &primary));
    click(cx, primary_supporting);
    assert!(!is_focused(cx, &primary));
    assert_eq!(primary_blurs.get(), 2);

    // 7. Clicking another TextField or Button transfers focus cleanly without losing it.
    click(cx, primary_container);
    assert!(is_focused(cx, &primary));
    click(cx, secondary_container);
    assert!(!is_focused(cx, &primary));
    assert!(is_focused(cx, &secondary));
    click(cx, outside_btn_center);
    assert!(!is_focused(cx, &secondary));
    assert!(
        cx.update_window(window.into(), |_, window, _| outside_btn.is_focused(window))
            .unwrap()
    );

    // 8. Clicking visibility_toggle toggles password visibility without intermediate Blur.
    click(cx, secure_container);
    assert!(is_focused(cx, &secure));
    assert_eq!(secure_blurs.get(), 0);
    click(cx, secure_toggle);
    assert!(is_focused(cx, &secure));
    assert!(cx.update(|cx| view.read(cx).revealed));
    assert_eq!(secure_blurs.get(), 0);

    // 9. Disabling a focused TextField blurs it.
    cx.update(|cx| secure.update(cx, |state, cx| state.set_disabled(true, cx)));
    draw(cx);
    cx.run_until_parked();
    assert!(!is_focused(cx, &secure));
    assert_eq!(secure_blurs.get(), 1);

    // 10. Open ExposedDropdown keeps editor focus when clicking an option, and blurs on outside blank click.
    let combo_toggle = cx
        .update_window(window.into(), |_, window, _| {
            let combo = find(window, &[], &(ElementId::from("combo"), "combobox").into()).unwrap();
            find(
                window,
                combo.path(),
                &(ElementId::from("combo"), "toggle-options").into(),
            )
            .unwrap()
            .bounds()
            .center()
        })
        .unwrap();
    click(cx, combo_toggle);
    cx.run_until_parked();
    draw(cx);
    let combo_editor = cx.update(|cx| {
        assert!(dropdown.read(cx).is_open());
        let combo_input = cx
            .update_window(window.into(), |_, window, _| {
                let combo =
                    find(window, &[], &(ElementId::from("combo"), "combobox").into()).unwrap();
                find(
                    window,
                    combo.path(),
                    &(ElementId::from("combo"), "value").into(),
                )
                .unwrap()
                .focused()
            })
            .unwrap();
        assert_eq!(combo_input, Some(true));
        dropdown.clone()
    });
    let beta_center = cx
        .update_window(window.into(), |_, window, _| {
            let options = find(window, &[], &(ElementId::from("combo"), "options").into()).unwrap();
            find(window, options.path(), &"beta".into())
                .unwrap()
                .bounds()
                .center()
        })
        .unwrap();
    click(cx, beta_center);
    cx.run_until_parked();
    draw(cx);
    assert_eq!(
        cx.update(|cx| combo_editor.read(cx).selected_option_id().cloned()),
        Some(ElementId::from("beta"))
    );
    assert_eq!(
        cx.update_window(window.into(), |_, window, _| {
            let combo = find(window, &[], &(ElementId::from("combo"), "combobox").into()).unwrap();
            find(
                window,
                combo.path(),
                &(ElementId::from("combo"), "value").into(),
            )
            .unwrap()
            .focused()
        })
        .unwrap(),
        Some(true)
    );

    // Reopen dropdown, then click blank space outside -> closes AND blurs editor.
    click(cx, combo_toggle);
    cx.run_until_parked();
    draw(cx);
    assert!(cx.update(|cx| dropdown.read(cx).is_open()));
    click(cx, blank_center);
    cx.run_until_parked();
    draw(cx);
    assert!(!cx.update(|cx| dropdown.read(cx).is_open()));
    assert_eq!(
        cx.update_window(window.into(), |_, window, _| {
            let combo = find(window, &[], &(ElementId::from("combo"), "combobox").into()).unwrap();
            find(
                window,
                combo.path(),
                &(ElementId::from("combo"), "value").into(),
            )
            .unwrap()
            .focused()
        })
        .unwrap(),
        Some(false)
    );
}
