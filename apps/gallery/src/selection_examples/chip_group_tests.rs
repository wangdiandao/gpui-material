use gpui::{
    AppContext, Context, ElementId, InputEvent, InteractiveElement, IntoElement, KeyDownEvent,
    KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Render, Role,
    ScrollHandle, SharedString, Styled, TestAppContext, Window, div, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    Button, ChipGroup, ChipGroupItem, ChipGroupSelection, ChipVariant,
    foundation::{MaterialRoot, set_window_material_theme},
    theme::{Appearance, Direction, MaterialTheme},
};

fn click(window: &mut Window, cx: &mut gpui::App, pos: gpui::Point<gpui::Pixels>) {
    window.dispatch_event(
        MouseDownEvent {
            button: MouseButton::Left,
            position: pos,
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
            position: pos,
            click_count: 1,
            modifiers: Default::default(),
        }
        .to_platform_input(),
        cx,
    );
}

fn press_key(window: &mut Window, cx: &mut gpui::App, name: &str) {
    let keystroke = Keystroke::parse_with_os(name, cx.platform_info().os).unwrap();
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

fn chip_activate_id(group: &'static str, item: &'static str) -> ElementId {
    ElementId::from((
        ElementId::from((ElementId::from(group), SharedString::from(item))),
        "activate",
    ))
}

#[gpui::test]
fn chip_group_supports_selection_modes_roving_focus_rtl_and_scroll_layout(cx: &mut TestAppContext) {
    struct Example {
        multi: Vec<SharedString>,
        single: Vec<SharedString>,
        required: SharedString,
        removed: Vec<SharedString>,
        scroll: ScrollHandle,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let v_multi = cx.entity();
            let v_single = cx.entity();
            let v_req = cx.entity();
            let v_rem = cx.entity();
            div()
                .id("chip-group-root")
                .tab_group()
                .w(px(240.))
                .flex()
                .flex_col()
                .gap_4()
                .p_3()
                .child(Button::new("before-group", "Before").on_activate(|_, _| {}))
                .child(
                    ChipGroup::new(
                        "multi-group",
                        "Filter tags",
                        [
                            ChipGroupItem::new("alpha", "Alpha tag"),
                            ChipGroupItem::new("beta", "Beta disabled").disabled(true),
                            ChipGroupItem::new("gamma", "Gamma tag"),
                            ChipGroupItem::new("delta", "Delta long tag").on_remove(
                                "Remove delta",
                                move |_, cx| {
                                    v_rem.update(cx, |this, cx| {
                                        this.removed.push("delta".into());
                                        cx.notify();
                                    });
                                },
                            ),
                        ],
                    )
                    .selection(ChipGroupSelection::Multiple, self.multi.clone())
                    .on_change(move |next, _, cx| {
                        v_multi.update(cx, |this, cx| {
                            this.multi = next;
                            cx.notify();
                        });
                    }),
                )
                .child(
                    ChipGroup::new(
                        "single-group",
                        "Optional single choice",
                        [
                            ChipGroupItem::new("one", "One"),
                            ChipGroupItem::new("two", "Two"),
                        ],
                    )
                    .variant(ChipVariant::Input)
                    .selection(ChipGroupSelection::Single, self.single.clone())
                    .on_change(move |next, _, cx| {
                        v_single.update(cx, |this, cx| {
                            this.single = next;
                            cx.notify();
                        });
                    }),
                )
                .child(
                    ChipGroup::new(
                        "required-group",
                        "Required timeframe",
                        [
                            ChipGroupItem::new("day", "Today"),
                            ChipGroupItem::new("week", "This week"),
                            ChipGroupItem::new("month", "This month"),
                            ChipGroupItem::new("year", "This year"),
                        ],
                    )
                    .scrollable(true)
                    .scroll_handle(&self.scroll)
                    .single_select(self.required.clone(), move |id, _, cx| {
                        v_req.update(cx, |this, cx| {
                            this.required = id;
                            cx.notify();
                        });
                    }),
                )
                .child(Button::new("after-group", "After").on_activate(|_, _| {}))
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let view = cx.new(|_| Example {
            multi: vec!["alpha".into()],
            single: vec!["one".into()],
            required: "day".into(),
            removed: Vec::new(),
            scroll: ScrollHandle::new(),
        });
        let handle =
            MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, _| view.clone())
                .unwrap();
        (handle.into(), view)
    });
    cx.simulate_window_resize(handle, size(px(300.), px(640.)));

    // Initial draw in LTR.
    cx.update_window(handle, |_, window, cx| {
        set_window_material_theme(Some(MaterialTheme::baseline(Appearance::Light)), window, cx);
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);

        // Role::ListBox semantics on the group.
        let group = find(window, &[], &"multi-group".into()).unwrap();
        assert_eq!(group.role(), Some(Role::ListBox));
        assert_eq!(group.label(), Some("Filter tags"));

        // Wrapping layout puts later chips onto a second row at 240 px width.
        let alpha = find(window, &[], &chip_activate_id("multi-group", "alpha")).unwrap();
        let delta = find(window, &[], &chip_activate_id("multi-group", "delta")).unwrap();
        assert!(
            delta.bounds().top() > alpha.bounds().top(),
            "Wrapped ChipGroup should flow onto multiple lines in narrow container"
        );

        // Scrollable ChipGroup keeps all chips on a single horizontal row.
        let day = find(window, &[], &chip_activate_id("required-group", "day")).unwrap();
        let year = find(window, &[], &chip_activate_id("required-group", "year")).unwrap();
        assert_eq!(
            day.bounds().top(),
            year.bounds().top(),
            "Scrollable ChipGroup must remain single-row"
        );
    })
    .unwrap();

    // Single Tab stop enters multi-group on its selected chip ("alpha"), then arrow keys rove
    // skipping disabled "beta", and Home/End jump to first/last enabled chip.
    cx.update_window(handle, |_, window, cx| {
        window.focus_next(cx); // before-group
        window.focus_next(cx); // multi-group active stop ("alpha")
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            find(window, &[], &chip_activate_id("multi-group", "alpha"))
                .unwrap()
                .focused(),
            Some(true)
        );
        // Right skips disabled "beta" and lands on "gamma".
        press_key(window, cx, "right");
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            find(window, &[], &chip_activate_id("multi-group", "gamma"))
                .unwrap()
                .focused(),
            Some(true)
        );
        // Activate "gamma" with space to add it to Multiple selection.
        press_key(window, cx, "space");
        press_key(window, cx, "end");
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            view.read(cx).multi,
            vec![SharedString::from("alpha"), SharedString::from("gamma")]
        );
        assert_eq!(
            find(window, &[], &chip_activate_id("multi-group", "delta"))
                .unwrap()
                .focused(),
            Some(true)
        );
        press_key(window, cx, "home");
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            find(window, &[], &chip_activate_id("multi-group", "alpha"))
                .unwrap()
                .focused(),
            Some(true)
        );
    })
    .unwrap();

    // Test Single (deselectable) and SingleRequired (.single_select) behavior.
    cx.update_window(handle, |_, window, cx| {
        let single_one = find(window, &[], &chip_activate_id("single-group", "one"))
            .unwrap()
            .bounds()
            .center();
        click(window, cx, single_one);

        let req_day = find(window, &[], &chip_activate_id("required-group", "day"))
            .unwrap()
            .bounds()
            .center();
        click(window, cx, req_day); // SingleRequired: clicking active chip stays "day"

        let req_week = find(window, &[], &chip_activate_id("required-group", "week"))
            .unwrap()
            .bounds()
            .center();
        click(window, cx, req_week);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(
            view.read(cx).single.is_empty(),
            "Single mode must allow deselecting active chip"
        );
        assert_eq!(
            view.read(cx).required,
            SharedString::from("week"),
            "SingleRequired must keep active choice on self-click and switch on sibling click"
        );

        // Clicking the removable chip's remove button fires its on_remove handler.
        let remove_id = ElementId::from((
            ElementId::from((ElementId::from("multi-group"), SharedString::from("delta"))),
            "remove",
        ));
        let remove_pos = find(window, &[], &remove_id).unwrap().bounds().center();
        click(window, cx, remove_pos);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, _, cx| {
        assert_eq!(view.read(cx).removed, vec![SharedString::from("delta")]);
    })
    .unwrap();

    // Verify RTL mirroring reverses physical order and arrow key direction.
    cx.update_window(handle, |_, window, cx| {
        let mut rtl = MaterialTheme::baseline(Appearance::Light);
        rtl.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(rtl), window, cx);
        window.refresh();
        window.draw(cx).clear(cx);

        let one = find(window, &[], &chip_activate_id("single-group", "one")).unwrap();
        let two = find(window, &[], &chip_activate_id("single-group", "two")).unwrap();
        assert!(
            one.bounds().left() > two.bounds().left(),
            "RTL ChipGroup must place first chip to the physical right of the second chip"
        );

        // Focus "one" then press Left (forward in RTL) -> moves to "two".
        click(window, cx, one.bounds().center());
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        press_key(window, cx, "left");
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            find(window, &[], &chip_activate_id("single-group", "two"))
                .unwrap()
                .focused(),
            Some(true),
            "Left arrow in RTL should move forward to the next chip"
        );
    })
    .unwrap();
}
