use gpui::{
    App, AppContext, Context, ElementId, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, Role, SharedString, Styled, TestAppContext, Window, div, px,
};
use gpui_base::test_support::find;
use gpui_material::{
    CheckboxState, CheckboxTile, ChoiceOption, ChoiceTile, DialogState, ListItem, ListSection,
    RadioTile, SliderState, SliderTile, SwitchTile, foundation::MaterialRoot,
};

use super::press_key;

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
