use gpui::{
    App, AppContext, Context, ElementId, Entity, InputEvent, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Render, SharedString, Styled,
    TestAppContext, Window, div, px,
};
use gpui_base::test_support::{find, find_all};
use gpui_material::{
    ChoiceOption, ChoiceTile, DialogState, SliderState, SliderTile,
    foundation::{MaterialRoot, set_window_material_theme},
    theme::{Appearance, Direction, MaterialTheme},
};

use super::press_key;

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
