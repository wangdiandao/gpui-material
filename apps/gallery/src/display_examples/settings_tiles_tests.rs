use gpui::{
    AppContext, Context, ElementId, Entity, InputEvent, InteractiveElement, IntoElement,
    KeyDownEvent, KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement,
    Render, Role, SharedString, Styled, TestAppContext, Window, div,
};
use gpui_base::test_support::{find, find_all};
use gpui_material::{
    ChoiceOption, ChoiceTile, DialogState, ListVariant, SwitchTile,
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

    cx.update_window(handle.into(), |_, window, cx| {
        let mut theme = MaterialTheme::baseline(Appearance::Light);
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        window.activate_window();
        draw(window, cx);

        let activate_id: ElementId = (ElementId::from("density-tile"), "activate").into();
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

        // Find and click the "Compact" option in the open menu.
        let compact_id: ElementId = (
            ElementId::from("density-tile"),
            SharedString::from("compact"),
        )
            .into();
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
        assert_eq!(view.read(cx).selected, "compact");
        assert!(!view.read(cx).menu.read(cx).is_open());
        assert_eq!(
            find(window, &[], &activate_id).unwrap().focused(),
            Some(true)
        );
    })
    .unwrap();
}
