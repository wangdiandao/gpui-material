use gpui::{
    App, AppContext, Context, ElementId, InputEvent, InteractiveElement, IntoElement, KeyDownEvent,
    KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Render, Role,
    SharedString, Styled, TestAppContext, Window, div,
};
use gpui_base::test_support::find;
use gpui_material::{
    CheckboxState, CheckboxTile, ListVariant, RadioTile, SwitchTile,
    foundation::{MaterialRoot, set_window_material_theme},
    theme::{Appearance, Direction, MaterialTheme},
};

use super::{click_at, press_key};

#[gpui::test]
fn switch_tile_toggles_once_with_single_focus_stop_and_switch_role(cx: &mut TestAppContext) {
    struct Example {
        checked: bool,
        changes: Vec<bool>,
        disabled: bool,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity();
            div().id("switch-tile-scene").tab_group().p_4().child(
                SwitchTile::new("sync-tile", "Auto sync", self.checked)
                    .supporting("Sync settings automatically")
                    .variant(ListVariant::Segmented)
                    .disabled(self.disabled)
                    .on_change(move |next, _, cx| {
                        view.update(cx, |this, cx| {
                            this.checked = next;
                            this.changes.push(next);
                            cx.notify();
                        });
                    }),
            )
        }
    }

    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let e = cx.new(|_| Example {
                checked: false,
                changes: Vec::new(),
                disabled: false,
            });
            entity = Some(e.clone());
            e
        })
        .unwrap()
    });
    let view = entity.unwrap();

    let draw = |window: &mut Window, cx: &mut gpui::App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        draw(window, cx);

        // Single switch role & focus stop (no duplicate inner Switch node).
        let activate_id: ElementId = (ElementId::from("sync-tile"), "activate").into();
        let row = find(window, &[], &activate_id).unwrap();
        assert_eq!(row.role(), Some(Role::Switch));
        assert_eq!(row.checked(), Some(false));

        // Clicking on the visual switch track itself toggles ONCE (no double-toggle).
        let switch_id: ElementId = (ElementId::from("sync-tile"), "switch").into();
        let track_id: ElementId = (switch_id, "switch-track").into();
        let track_center = find(window, &[], &track_id).unwrap().bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: track_center,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: track_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert_eq!(view.read(cx).changes, [true]);
        assert_eq!(
            find(window, &[], &activate_id).unwrap().checked(),
            Some(true)
        );

        // Space toggles back once via keyboard.
        let space = Keystroke::parse("space").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: space.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: space }.to_platform_input(), cx);
        draw(window, cx);
        assert_eq!(view.read(cx).changes, [true, false]);

        // Disabled tile ignores activation.
        view.update(cx, |this, cx| {
            this.disabled = true;
            cx.notify();
        });
        draw(window, cx);
        let center = find(window, &[], &"sync-tile".into())
            .unwrap()
            .bounds()
            .center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: center,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert_eq!(view.read(cx).changes, [true, false]);
    })
    .unwrap();
}

#[gpui::test]
fn checkbox_tile_supports_tristate_error_disabled_single_focus_and_rtl(cx: &mut TestAppContext) {
    struct Example {
        state: CheckboxState,
        changes: Vec<CheckboxState>,
        error: bool,
        disabled: bool,
        trailing: bool,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity();
            div().id("checkbox-tile-scene").tab_group().p_4().child(
                CheckboxTile::new("diag-tile", "Diagnostics", self.state)
                    .supporting("Share anonymous crash logs")
                    .trailing_checkbox(self.trailing)
                    .error(self.error)
                    .disabled(self.disabled)
                    .variant(ListVariant::Segmented)
                    .on_change(move |next, _, cx| {
                        view.update(cx, |this, cx| {
                            this.state = next;
                            this.changes.push(next);
                            cx.notify();
                        });
                    }),
            )
        }
    }

    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let e = cx.new(|_| Example {
                state: CheckboxState::Indeterminate,
                changes: Vec::new(),
                error: true,
                disabled: false,
                trailing: true,
            });
            entity = Some(e.clone());
            e
        })
        .unwrap()
    });
    let view = entity.unwrap();

    let draw = |window: &mut Window, cx: &mut App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        draw(window, cx);

        let activate_id: ElementId = (ElementId::from("diag-tile"), "activate").into();
        let row = find(window, &[], &activate_id).unwrap();
        assert_eq!(row.role(), Some(Role::CheckBox));
        assert_eq!(row.checked(), Some(false));
        assert_eq!(row.indeterminate(), Some(true));

        // Decorative checkbox has no separate CheckBox role.
        let checkbox_id: ElementId = (ElementId::from("diag-tile"), "checkbox").into();
        let visual = find(window, &[], &checkbox_id).unwrap();
        assert_ne!(visual.role(), Some(Role::CheckBox));

        // Clicking on the visual checkbox indicator toggles ONCE: Indeterminate -> Checked.
        click_at(visual.bounds().center(), window, cx);
        draw(window, cx);
        assert_eq!(view.read(cx).changes, [CheckboxState::Checked]);
        let row = find(window, &[], &activate_id).unwrap();
        assert_eq!(row.checked(), Some(true));
        assert_eq!(row.indeterminate(), Some(false));

        // Space toggles Checked -> Unchecked via keyboard.
        press_key("space", window, cx);
        draw(window, cx);
        assert_eq!(
            view.read(cx).changes,
            [CheckboxState::Checked, CheckboxState::Unchecked]
        );

        // Leading checkbox placement mirrors cleanly in RTL.
        let mut rtl = MaterialTheme::baseline(Appearance::Light);
        rtl.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(rtl), window, cx);
        view.update(cx, |this, cx| {
            this.trailing = false;
            cx.notify();
        });
        draw(window, cx);
        let row_bounds = find(window, &[], &activate_id).unwrap().bounds();
        let indicator_bounds = find(window, &[], &checkbox_id).unwrap().bounds();
        assert!(
            indicator_bounds.center().x > row_bounds.center().x,
            "leading checkbox should sit on the right half in RTL"
        );

        // Disabled tile ignores activation.
        view.update(cx, |this, cx| {
            this.disabled = true;
            cx.notify();
        });
        draw(window, cx);
        let center = find(window, &[], &"diag-tile".into())
            .unwrap()
            .bounds()
            .center();
        click_at(center, window, cx);
        draw(window, cx);
        assert_eq!(
            view.read(cx).changes,
            [CheckboxState::Checked, CheckboxState::Unchecked]
        );
    })
    .unwrap();
}

#[gpui::test]
fn radio_tile_selects_once_with_single_focus_stop_and_radio_role(cx: &mut TestAppContext) {
    struct Example {
        selected: SharedString,
        selections: Vec<SharedString>,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let v1 = cx.entity();
            let v2 = cx.entity();
            let v3 = cx.entity();
            div()
                .id("radio-tile-scene")
                .tab_group()
                .flex()
                .flex_col()
                .p_4()
                .child(
                    RadioTile::new("radio-a", "Stable", self.selected == "a").on_select(
                        move |_, cx| {
                            v1.update(cx, |this, cx| {
                                this.selected = "a".into();
                                this.selections.push("a".into());
                                cx.notify();
                            });
                        },
                    ),
                )
                .child(
                    RadioTile::new("radio-b", "Beta", self.selected == "b").on_select(
                        move |_, cx| {
                            v2.update(cx, |this, cx| {
                                this.selected = "b".into();
                                this.selections.push("b".into());
                                cx.notify();
                            });
                        },
                    ),
                )
                .child(
                    RadioTile::new("radio-c", "Canary", false)
                        .disabled(true)
                        .on_select(move |_, cx| {
                            v3.update(cx, |this, cx| {
                                this.selected = "c".into();
                                this.selections.push("c".into());
                                cx.notify();
                            });
                        }),
                )
        }
    }

    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let e = cx.new(|_| Example {
                selected: "a".into(),
                selections: Vec::new(),
            });
            entity = Some(e.clone());
            e
        })
        .unwrap()
    });
    let view = entity.unwrap();

    let draw = |window: &mut Window, cx: &mut App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        draw(window, cx);

        let a_id: ElementId = (ElementId::from("radio-a"), "activate").into();
        let b_id: ElementId = (ElementId::from("radio-b"), "activate").into();
        assert_eq!(
            find(window, &[], &a_id).unwrap().role(),
            Some(Role::RadioButton)
        );
        assert_eq!(find(window, &[], &a_id).unwrap().checked(), Some(true));
        assert_eq!(find(window, &[], &b_id).unwrap().checked(), Some(false));

        // Clicking already-selected radio does not re-fire on_select.
        click_at(
            find(window, &[], &a_id).unwrap().bounds().center(),
            window,
            cx,
        );
        draw(window, cx);
        assert!(view.read(cx).selections.is_empty());

        // Clicking visual radio dot of unselected option selects it once.
        let b_radio_id: ElementId = (ElementId::from("radio-b"), "radio").into();
        click_at(
            find(window, &[], &b_radio_id).unwrap().bounds().center(),
            window,
            cx,
        );
        draw(window, cx);
        assert_eq!(view.read(cx).selected, "b");
        assert_eq!(view.read(cx).selections, ["b"]);
        assert_eq!(find(window, &[], &b_id).unwrap().checked(), Some(true));

        // Clicking disabled option ignores selection.
        click_at(
            find(window, &[], &"radio-c".into())
                .unwrap()
                .bounds()
                .center(),
            window,
            cx,
        );
        draw(window, cx);
        assert_eq!(view.read(cx).selected, "b");
        assert_eq!(view.read(cx).selections, ["b"]);
    })
    .unwrap();
}
