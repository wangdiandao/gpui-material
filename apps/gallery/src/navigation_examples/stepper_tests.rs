use gpui::{
    AppContext, Context, ElementId, InputEvent, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke,
    MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Render, Role, SharedString, Styled,
    TestAppContext, Window, div, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    StepItem, StepStatus, Stepper, StepperOrientation, Text,
    foundation::{MaterialRoot, set_window_material_theme},
    theme::{Appearance, Direction, MaterialTheme},
};

#[gpui::test]
fn stepper_handles_horizontal_vertical_roving_focus_rtl_and_status_semantics(
    cx: &mut TestAppContext,
) {
    struct Example {
        horizontal_active: usize,
        vertical_active: usize,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let on_horiz = cx.entity();
            let on_vert = cx.entity();
            div()
                .flex()
                .flex_col()
                .gap_4()
                .p_4()
                .child(
                    Stepper::new(
                        "h-stepper",
                        [
                            StepItem::new("s1", "Account"),
                            StepItem::new("s2", "Profile"),
                            StepItem::new("s3", "Confirm"),
                        ],
                    )
                    .accessible_label("Onboarding steps")
                    .active_step(self.horizontal_active)
                    .non_linear(true)
                    .on_select(move |index, _, cx| {
                        on_horiz.update(cx, |this, cx| {
                            this.horizontal_active = index;
                            cx.notify();
                        });
                    }),
                )
                .child(
                    Stepper::new(
                        "v-stepper",
                        [
                            StepItem::new("v1", "Source")
                                .content(Text::new("Source body").id("v1-body")),
                            StepItem::new("v2", "Disabled").status(StepStatus::Disabled),
                            StepItem::new("v3", "Validate")
                                .status(StepStatus::Error)
                                .content(Text::new("Error body").id("v3-body")),
                        ],
                    )
                    .orientation(StepperOrientation::Vertical)
                    .active_step(self.vertical_active)
                    .non_linear(true)
                    .on_select(move |index, _, cx| {
                        on_vert.update(cx, |this, cx| {
                            this.vertical_active = index;
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
                horizontal_active: 1,
                vertical_active: 0,
            });
            entity = Some(e.clone());
            e
        })
        .unwrap()
    });
    let view = entity.unwrap();
    cx.simulate_window_resize(handle.into(), size(px(640.), px(520.)));

    let draw = |window: &mut Window, cx: &mut gpui::App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };

    let h_id = |step: &'static str| -> ElementId {
        (ElementId::from("h-stepper"), SharedString::from(step)).into()
    };
    let v_id = |step: &'static str| -> ElementId {
        (ElementId::from("v-stepper"), SharedString::from(step)).into()
    };

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        draw(window, cx);

        let container = find(window, &[], &"h-stepper".into()).unwrap();
        assert_eq!(container.role(), Some(Role::List));
        assert_eq!(container.label(), Some("Onboarding steps"));

        let s1 = find(window, &[], &h_id("s1")).unwrap();
        let s2 = find(window, &[], &h_id("s2")).unwrap();
        let s3 = find(window, &[], &h_id("s3")).unwrap();
        assert_eq!(s1.label(), Some("Account, completed"));
        assert_eq!(s2.label(), Some("Profile, current"));
        assert_eq!(s3.label(), Some("Confirm, upcoming"));
        assert!(s1.bounds().left() < s2.bounds().left());
        assert!(s2.bounds().left() < s3.bounds().left());

        // Click s2 to focus, press Right -> moves focus to s3, Enter selects s3.
        let s2_center = s2.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: s2_center,
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
                position: s2_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);

        let right = Keystroke::parse("right").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: right.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: right }.to_platform_input(), cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &h_id("s3")).unwrap().focused(),
            Some(true)
        );

        let enter = Keystroke::parse("enter").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: enter.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: enter }.to_platform_input(), cx);
        draw(window, cx);
        assert_eq!(view.read(cx).horizontal_active, 2);

        // Vertical stepper: v1 active content rendered, Down skips disabled v2 and focuses v3.
        let v1 = find(window, &[], &v_id("v1")).unwrap();
        let v1_center = v1.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: v1_center,
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
                position: v1_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);

        let down = Keystroke::parse("down").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: down.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: down }.to_platform_input(), cx);
        draw(window, cx);
        let v3 = find(window, &[], &v_id("v3")).unwrap();
        assert_eq!(v3.focused(), Some(true));
        assert_eq!(v3.label(), Some("Validate, error"));

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
        assert_eq!(view.read(cx).vertical_active, 2);
    })
    .unwrap();

    // RTL mirroring for horizontal stepper.
    cx.update_window(handle.into(), |_, window, cx| {
        let mut theme = MaterialTheme::baseline(Appearance::Light);
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        draw(window, cx);

        let s1 = find(window, &[], &h_id("s1")).unwrap();
        let s3 = find(window, &[], &h_id("s3")).unwrap();
        assert!(s3.bounds().left() < s1.bounds().left());
    })
    .unwrap();
}
