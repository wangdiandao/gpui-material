use gpui::{
    App, AppContext, Context, ElementId, Entity, InputEvent, InteractiveElement, IntoElement,
    KeyDownEvent, KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement,
    Pixels, Point, Render, Role, SharedString, Styled, TestAppContext, Window, div, px,
};
use gpui_base::test_support::{find, find_all};
use gpui_material::{
    CheckboxState, CheckboxTile, ChoiceOption, ChoiceTile, DialogState, ListItem, ListSection,
    ListVariant, RadioTile, SliderState, SliderTile, SwitchTile,
    foundation::{MaterialRoot, set_window_material_theme},
    theme::{Appearance, Direction, MaterialTheme},
};

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
fn choice_tile_opens_anchored_menu_selects_option_and_restores_focus(cx: &mut TestAppContext) {
    struct Example {
        selected: SharedString,
        menu: Entity<DialogState>,
        disabled: bool,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity();
            div().id("choice-tile-scene").p_4().child(
                ChoiceTile::new(
                    "density-tile",
                    "Density",
                    [
                        ChoiceOption::new("compact", "Compact"),
                        ChoiceOption::new("comfortable", "Comfortable"),
                        ChoiceOption::new("spacious", "Spacious").disabled(true),
                    ],
                    self.selected.clone(),
                )
                .supporting("Row density preference")
                .disabled(self.disabled)
                .menu_state(&self.menu)
                .on_select(move |choice, _, cx| {
                    view.update(cx, |this, cx| {
                        this.selected = choice;
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
            let menu = cx.new(DialogState::new);
            let e = cx.new(|_| Example {
                selected: "comfortable".into(),
                menu,
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

    let activate_id: ElementId = (ElementId::from("density-tile"), "activate").into();
    let compact_id: ElementId = (
        ElementId::from("density-tile"),
        SharedString::from("compact"),
    )
        .into();

    cx.update_window(handle.into(), |_, window, cx| {
        let mut theme = MaterialTheme::baseline(Appearance::Light);
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        window.activate_window();
        draw(window, cx);

        let trigger = find(window, &[], &activate_id).unwrap();
        let center = trigger.bounds().center();

        // Click row to open anchored menu.
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
        assert!(view.read(cx).menu.read(cx).is_open());
    })
    .unwrap();

    // Flush deferred popover focus into the menu, then click the "Compact" option.
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);
        let compact = find_all(window, &[], &compact_id)
            .into_iter()
            .next()
            .expect("Compact menu option should render when open");
        let option_center = compact.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: option_center,
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
                position: option_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
    })
    .unwrap();

    // Flush deferred focus restoration back to the trigger tile.
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);
        assert_eq!(view.read(cx).selected, "compact");
        assert!(!view.read(cx).menu.read(cx).is_open());
        assert_eq!(
            find(window, &[], &activate_id).unwrap().focused(),
            Some(true)
        );
    })
    .unwrap();
}

fn click_at(position: Point<Pixels>, window: &mut Window, cx: &mut App) {
    window.dispatch_event(
        MouseDownEvent {
            button: MouseButton::Left,
            position,
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
            position,
            click_count: 1,
            modifiers: Default::default(),
        }
        .to_platform_input(),
        cx,
    );
}

fn press_key(name: &'static str, window: &mut Window, cx: &mut App) {
    let keystroke = Keystroke::parse(name).unwrap();
    window.dispatch_event(
        KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        }
        .to_platform_input(),
        cx,
    );
    window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
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

#[gpui::test]
fn slider_tile_renders_live_formatted_value_and_preserves_slider_interaction(
    cx: &mut TestAppContext,
) {
    struct Example {
        slider: Entity<SliderState>,
        disabled: bool,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().id("slider-tile-scene").w(px(360.)).p_4().child(
                SliderTile::new("volume-tile", "Volume", &self.slider)
                    .supporting("Desktop alerts")
                    .disabled(self.disabled)
                    .format_value(|v| format!("{}%", v.round() as i32).into())
                    .slider(|slider| slider.tick_intervals(5)),
            )
        }
    }

    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let slider = cx.new(|_| {
                SliderState::new()
                    .min(0.)
                    .max(100.)
                    .step(10.)
                    .default_value(40.)
            });
            let e = cx.new(|cx| {
                cx.observe(&slider, |_, _, cx| cx.notify()).detach();
                Example {
                    slider,
                    disabled: false,
                }
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

        let value_id: ElementId = (ElementId::from("volume-tile"), "slider-value").into();
        assert_eq!(find(window, &[], &value_id).unwrap().label(), Some("40%"));

        // Focus the embedded slider and adjust with ArrowRight.
        window.focus_next(cx);
        draw(window, cx);
        press_key("right", window, cx);
        draw(window, cx);
        assert_eq!(find(window, &[], &value_id).unwrap().label(), Some("50%"));

        // Disabled slider tile ignores keyboard adjustments.
        view.update(cx, |this, cx| {
            this.disabled = true;
            cx.notify();
        });
        draw(window, cx);
        press_key("right", window, cx);
        draw(window, cx);
        assert_eq!(find(window, &[], &value_id).unwrap().label(), Some("50%"));
    })
    .unwrap();
}

#[gpui::test]
fn list_section_segmented_applies_grouped_corners_and_unified_roving_focus(
    cx: &mut TestAppContext,
) {
    struct Example {
        switch_on: bool,
        checkbox: CheckboxState,
        slider: Entity<SliderState>,
        choice: SharedString,
        choice_menu: Entity<DialogState>,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let v1 = cx.entity();
            let v2 = cx.entity();
            let v3 = cx.entity();
            div()
                .id("list-section-scene")
                .tab_group()
                .w(px(420.))
                .p_4()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    ListSection::segmented("general-section")
                        .title("General")
                        .item(ListItem::new("row-item", "Profile").on_activate(|_, _| {}))
                        .item(
                            SwitchTile::new("sec-switch", "Sync", self.switch_on).on_change(
                                move |next, _, cx| {
                                    v1.update(cx, |this, cx| {
                                        this.switch_on = next;
                                        cx.notify();
                                    });
                                },
                            ),
                        )
                        .item(
                            CheckboxTile::new("sec-checkbox", "Telemetry", self.checkbox)
                                .on_change(move |next, _, cx| {
                                    v2.update(cx, |this, cx| {
                                        this.checkbox = next;
                                        cx.notify();
                                    });
                                }),
                        )
                        .item(
                            RadioTile::new("sec-disabled-radio", "Locked option", false)
                                .disabled(true)
                                .on_select(|_, _| {}),
                        )
                        .item(SliderTile::new("sec-slider", "Brightness", &self.slider))
                        .item(
                            ChoiceTile::new(
                                "sec-choice",
                                "Theme mode",
                                [
                                    ChoiceOption::new("light", "Light"),
                                    ChoiceOption::new("dark", "Dark"),
                                ],
                                self.choice.clone(),
                            )
                            .menu_state(&self.choice_menu)
                            .on_select(move |choice, _, cx| {
                                v3.update(cx, |this, cx| {
                                    this.choice = choice;
                                    cx.notify();
                                });
                            }),
                        ),
                )
                .child(
                    ListSection::segmented("single-section").item(
                        SwitchTile::new("only-tile", "Only item", false).on_change(|_, _, _| {}),
                    ),
                )
        }
    }

    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let slider = cx.new(|_| SliderState::new().min(0.).max(100.).default_value(70.));
            let choice_menu = cx.new(DialogState::new);
            let e = cx.new(|_| Example {
                switch_on: false,
                checkbox: CheckboxState::Unchecked,
                slider,
                choice: "light".into(),
                choice_menu,
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

    cx.simulate_window_scale_factor_change(handle.into(), 1.);
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        draw(window, cx);

        // Section heading and Role::List label are exposed.
        let heading_id: ElementId = (ElementId::from("general-section"), "section-title").into();
        let list_id: ElementId = (ElementId::from("general-section"), "list").into();
        assert_eq!(
            find(window, &[], &heading_id).unwrap().label(),
            Some("General")
        );
        let list = find(window, &[], &list_id).unwrap();
        assert_eq!(list.role(), Some(Role::List));
        assert_eq!(list.label(), Some("General"));

        // Verify true M3E Segmented grouped corner geometry on painted row quads.
        let scale = window.scale_factor();
        let sp = |v: f32| px(v).scale(scale);
        let row_corners = |id: &'static str, window: &Window| {
            let row_bounds = find(window, &[], &id.into()).unwrap().bounds().scale(scale);
            crate::painted_quads(window)
                .into_iter()
                .find(|quad| {
                    (quad.bounds.size.width - row_bounds.size.width).0.abs() < sp(6.).0
                        && (quad.bounds.origin.y - row_bounds.origin.y).0.abs() < sp(6.).0
                        && quad.bounds.size.height >= sp(50.)
                })
                .map(|quad| quad.corner_radii)
                .unwrap_or_else(|| panic!("missing surface quad for {id}"))
        };
        let first = row_corners("row-item", window);
        assert_eq!(
            (
                first.top_left,
                first.top_right,
                first.bottom_right,
                first.bottom_left
            ),
            (sp(16.), sp(16.), sp(4.), sp(4.))
        );
        let middle = row_corners("sec-switch", window);
        assert_eq!(
            (
                middle.top_left,
                middle.top_right,
                middle.bottom_right,
                middle.bottom_left
            ),
            (sp(4.), sp(4.), sp(4.), sp(4.))
        );
        let last = row_corners("sec-choice", window);
        assert_eq!(
            (
                last.top_left,
                last.top_right,
                last.bottom_right,
                last.bottom_left
            ),
            (sp(4.), sp(4.), sp(16.), sp(16.))
        );
        let single = row_corners("only-tile", window);
        assert_eq!(
            (
                single.top_left,
                single.top_right,
                single.bottom_right,
                single.bottom_left
            ),
            (sp(16.), sp(16.), sp(16.), sp(16.))
        );

        // Verify 2 dp vertical gap between adjacent segmented rows.
        let r0 = find(window, &[], &"row-item".into()).unwrap().bounds();
        let r1 = find(window, &[], &"sec-switch".into()).unwrap().bounds();
        assert!(
            ((r1.top() - r0.bottom()) - px(2.)).abs() < px(0.5),
            "expected 2 dp segmented gap, got {:?}",
            r1.top() - r0.bottom()
        );

        // Unified vertical roving focus: Down/Up/Home/End skips disabled RadioTile & static SliderTile.
        let act = |id: &'static str| ElementId::from((ElementId::from(id), "activate"));
        window.focus_next(cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &act("row-item")).unwrap().focused(),
            Some(true)
        );

        press_key("down", window, cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &act("sec-switch")).unwrap().focused(),
            Some(true)
        );

        press_key("down", window, cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &act("sec-checkbox")).unwrap().focused(),
            Some(true)
        );

        // Next Down skips disabled sec-disabled-radio AND static sec-slider to land on sec-choice!
        press_key("down", window, cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &act("sec-choice")).unwrap().focused(),
            Some(true)
        );

        // Home jumps back to first interactive row, End jumps to last interactive row.
        press_key("home", window, cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &act("row-item")).unwrap().focused(),
            Some(true)
        );

        press_key("end", window, cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &act("sec-choice")).unwrap().focused(),
            Some(true)
        );

        // Pressing Enter on focused ChoiceTile opens its anchored menu cleanly.
        press_key("enter", window, cx);
        draw(window, cx);
        assert!(view.read(cx).choice_menu.read(cx).is_open());
    })
    .unwrap();
}

#[gpui::test]
fn settings_tiles_preserve_track_focus_interaction_source_trailing_slot_and_choice_description(
    cx: &mut TestAppContext,
) {
    use gpui::FocusHandle;
    use gpui_base::TestSupportExt as _;
    use gpui_material::{InteractionSource, ListSectionItem};

    struct Example {
        handles: [FocusHandle; 5],
        source: Entity<InteractionSource>,
        slider: Entity<SliderState>,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(360.)).p_4().child(
                ListSection::segmented("tracked-section").items([
                    ListSectionItem::Switch(
                        SwitchTile::new("st-switch", "Notifications", true)
                            .track_focus(&self.handles[0])
                            .interaction_source(&self.source)
                            .on_change(|_, _, _| {}),
                    ),
                    ListSectionItem::Checkbox(
                        CheckboxTile::bool("st-check", "Auto download", true)
                            .trailing_checkbox(false)
                            .trailing(
                                div()
                                    .id("check-trailing")
                                    .test_support()
                                    .child("Wi-Fi only"),
                            )
                            .track_focus(&self.handles[1])
                            .interaction_source(&self.source)
                            .on_change(|_, _, _| {}),
                    ),
                    ListSectionItem::Radio(
                        RadioTile::new("st-radio", "Compact", false)
                            .trailing_radio(false)
                            .trailing(div().id("radio-trailing").test_support().child("Beta"))
                            .track_focus(&self.handles[2])
                            .interaction_source(&self.source)
                            .on_select(|_, _| {}),
                    ),
                    ListSectionItem::Slider(
                        SliderTile::new("st-slider", "Brightness", &self.slider)
                            .track_focus(&self.handles[3])
                            .interaction_source(&self.source),
                    ),
                    ListSectionItem::Choice(
                        ChoiceTile::new(
                            "st-choice",
                            "Theme",
                            [
                                ChoiceOption::new("light", "Light"),
                                ChoiceOption::new("dark", "Dark"),
                            ],
                            "dark",
                        )
                        .track_focus(&self.handles[4])
                        .interaction_source(&self.source)
                        .on_select(|_, _, _| {}),
                    ),
                ]),
            )
        }
    }

    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let source = cx.new(|_| InteractionSource::default());
            let slider = cx.new(|_| SliderState::new());
            let e = cx.new(|cx| Example {
                handles: std::array::from_fn(|_| cx.focus_handle()),
                source,
                slider,
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

        let check_row = find(window, &[], &"st-check".into()).unwrap().bounds();
        let check_trailing = find(window, &[], &"check-trailing".into())
            .unwrap()
            .bounds();
        let radio_row = find(window, &[], &"st-radio".into()).unwrap().bounds();
        let radio_trailing = find(window, &[], &"radio-trailing".into())
            .unwrap()
            .bounds();
        assert!(check_trailing.left() > check_row.center().x);
        assert!(radio_trailing.left() > radio_row.center().x);

        let choice_act = find(
            window,
            &[],
            &(ElementId::from("st-choice"), "activate").into(),
        )
        .unwrap();
        assert_eq!(choice_act.description(), Some("Dark"));

        let wid = window.window_handle().window_id();
        let source = view.read(cx).source.read(cx);
        for id in [
            "st-switch",
            "st-check",
            "st-radio",
            "st-slider",
            "st-choice",
        ] {
            assert!(
                source.state(wid, &ElementId::from(id)).is_some(),
                "missing interaction state for {id}"
            );
        }

        // Caller focus handles are preserved by ListSection roving navigation.
        view.read(cx).handles[0].clone().focus(window, cx);
        draw(window, cx);
        assert!(view.read(cx).handles[0].is_focused(window));

        press_key("down", window, cx);
        draw(window, cx);
        assert!(view.read(cx).handles[1].is_focused(window));

        press_key("end", window, cx);
        draw(window, cx);
        assert!(view.read(cx).handles[4].is_focused(window));
    })
    .unwrap();
}
