use gpui::{
    App, AppContext, Context, ElementId, Hsla, InputEvent, IntoElement, KeyDownEvent, KeyUpEvent,
    Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Render, Styled,
    TestAppContext, Window, WindowOptions, div, px, rgb,
};
use gpui_base::test_support::find;
use gpui_material::{
    Button, ButtonColors, ButtonVariant, Icon, IconButton, IconButtonVariant, IconName,
    IconToggleButton, SemanticTone, ToggleButton, ToggleButtonVariant,
    foundation::{MaterialRoot, color, window_material_theme},
    theme::ColorRole,
};

use super::super::row;

#[gpui::test]
fn button_tone_applies_semantic_palettes_and_yields_to_explicit_colors(cx: &mut TestAppContext) {
    const OVERRIDE_BG: u32 = 0x1a4d2e;

    struct ToneView;

    impl Render for ToneView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            row()
                .child(
                    Button::new("danger-filled", "Delete")
                        .variant(ButtonVariant::Filled)
                        .tone(SemanticTone::Danger)
                        .on_activate(|_, _| {}),
                )
                .child(
                    Button::new("success-tonal", "Approve")
                        .variant(ButtonVariant::FilledTonal)
                        .tone(SemanticTone::Success)
                        .on_activate(|_, _| {}),
                )
                .child(
                    Button::new("warning-outlined", "Caution")
                        .variant(ButtonVariant::Outlined)
                        .tone(SemanticTone::Warning)
                        .on_activate(|_, _| {}),
                )
                .child(
                    ToggleButton::new("info-toggle", "Info", true)
                        .variant(ToggleButtonVariant::Filled)
                        .tone(SemanticTone::Info)
                        .on_change(|_, _, _| {}),
                )
                .child(
                    IconButton::new("danger-icon", "Remove", Icon::new(IconName::Close))
                        .variant(IconButtonVariant::Filled)
                        .tone(SemanticTone::Danger)
                        .on_activate(|_, _| {}),
                )
                .child(
                    Button::new("override-danger", "Override")
                        .variant(ButtonVariant::Filled)
                        .tone(SemanticTone::Danger)
                        .colors(ButtonColors {
                            container: Some(rgb(OVERRIDE_BG).into()),
                            ..Default::default()
                        })
                        .on_activate(|_, _| {}),
                )
        }
    }

    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| ToneView))
            .unwrap()
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);

        let theme = window_material_theme(window, cx);
        let quads = crate::painted_quads(window);
        let has_fill = |fill: Hsla| quads.iter().any(|quad| quad.background == fill.into());
        let has_border = |stroke: Hsla| {
            quads
                .iter()
                .any(|quad| quad.border_color == stroke && quad.border_widths.top > 0.0_f32.into())
        };

        assert!(has_fill(color(theme.colors.color(ColorRole::Error))));
        assert!(has_fill(color(
            theme.colors.color(ColorRole::SuccessContainer)
        )));
        assert!(has_border(color(theme.colors.color(ColorRole::Warning))));
        assert!(has_fill(color(theme.colors.color(ColorRole::Info))));
        assert!(has_fill(rgb(OVERRIDE_BG).into()));

        assert_eq!(
            find(window, &[], &"danger-filled".into()).unwrap().label(),
            Some("Delete")
        );
        assert_eq!(
            find(window, &[], &"danger-icon".into()).unwrap().label(),
            Some("Remove")
        );
    })
    .unwrap();
}

#[gpui::test]
fn button_loading_suppresses_activation_preserves_width_and_respects_motion_and_visibility(
    cx: &mut TestAppContext,
) {
    struct LoadingView {
        loading: bool,
        disabled: bool,
        offscreen: bool,
        button_clicks: usize,
        icon_clicks: usize,
    }

    impl Render for LoadingView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let v1 = cx.entity();
            let v2 = cx.entity();
            let content = row()
                .child(
                    Button::filled("save-btn", "Save changes")
                        .loading(self.loading)
                        .disabled(self.disabled)
                        .on_activate(move |_, cx| {
                            v1.update(cx, |this, _| this.button_clicks += 1);
                        }),
                )
                .child(
                    IconButton::filled("sync-btn", "Sync", Icon::new(IconName::Add))
                        .loading(self.loading)
                        .disabled(self.disabled)
                        .on_activate(move |_, cx| {
                            v2.update(cx, |this, _| this.icon_clicks += 1);
                        }),
                );
            if self.offscreen {
                // Clip completely off-screen inside a bounded container.
                div()
                    .w(px(200.))
                    .h(px(60.))
                    .overflow_hidden()
                    .child(div().mt(px(400.)).child(content))
            } else {
                div().p_4().child(content)
            }
        }
    }

    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let e = cx.new(|_| LoadingView {
                loading: false,
                disabled: false,
                offscreen: false,
                button_clicks: 0,
                icon_clicks: 0,
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

    cx.update_window(handle.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);

        let resting_bounds = find(window, &[], &"save-btn".into()).unwrap().bounds();
        let icon_resting_bounds = find(window, &[], &"sync-btn".into()).unwrap().bounds();
        let primary = color(
            window_material_theme(window, cx)
                .colors
                .color(ColorRole::Primary),
        );

        // Switch to loading: resting width must stay identical and spinner must render.
        view.update(cx, |this, cx| {
            this.loading = true;
            cx.notify();
        });
        draw(window, cx);

        let loading_bounds = find(window, &[], &"save-btn".into()).unwrap().bounds();
        let icon_loading_bounds = find(window, &[], &"sync-btn".into()).unwrap().bounds();
        assert_eq!(loading_bounds.size.width, resting_bounds.size.width);
        assert_eq!(
            icon_loading_bounds.size.width,
            icon_resting_bounds.size.width
        );

        let btn_spinner: ElementId = (ElementId::from("save-btn"), "loading-spinner").into();
        let icon_spinner: ElementId = (ElementId::from("sync-btn"), "loading-spinner").into();
        assert!(find(window, &[], &btn_spinner).is_some());
        assert!(find(window, &[], &icon_spinner).is_some());

        // Active palette is preserved while loading without disabled.
        assert!(
            crate::painted_quads(window)
                .iter()
                .any(|quad| quad.background == primary.into()),
            "loading button must preserve active container color"
        );

        // Pointer click and Space key are suppressed while loading.
        for center in [loading_bounds.center(), icon_loading_bounds.center()] {
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
        }
        window.focus_next(cx);
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
        assert_eq!(view.read(cx).button_clicks, 0);
        assert_eq!(view.read(cx).icon_clicks, 0);

        // Reduced motion must request zero animation frames while loading.
        assert_eq!(window.simulate_next_frame(cx), 0);

        // With full motion enabled, visible loading animates, while offscreen loading requests zero frames.
        cx.set_reduce_motion(false);
        draw(window, cx);
        assert!(
            window.simulate_next_frame(cx) > 0,
            "visible animated loading button should schedule frames"
        );

        view.update(cx, |this, cx| {
            this.offscreen = true;
            cx.notify();
        });
        draw(window, cx);
        assert_eq!(
            window.simulate_next_frame(cx),
            0,
            "off-screen clipped loading spinner must not schedule frames"
        );

        // Disabled appearance takes precedence over loading active palette.
        cx.set_reduce_motion(true);
        view.update(cx, |this, cx| {
            this.offscreen = false;
            this.disabled = true;
            cx.notify();
        });
        draw(window, cx);
        assert!(
            !crate::painted_quads(window)
                .iter()
                .any(|quad| quad.background == primary.into()),
            "disabled + loading must use disabled palette"
        );

        // Returning to idle re-enables activation cleanly.
        view.update(cx, |this, cx| {
            this.loading = false;
            this.disabled = false;
            cx.notify();
        });
        draw(window, cx);
        let center = find(window, &[], &"save-btn".into())
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
        assert_eq!(view.read(cx).button_clicks, 1);
    })
    .unwrap();
}

#[gpui::test]
fn toggle_buttons_support_variant_shorthands_and_loading_state(cx: &mut TestAppContext) {
    struct ToggleLoadingView {
        loading: bool,
        disabled: bool,
        offscreen: bool,
        selected: bool,
        toggle_requests: Vec<bool>,
        icon_requests: Vec<bool>,
    }

    impl Render for ToggleLoadingView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let v1 = cx.entity();
            let v2 = cx.entity();
            let content = row()
                .child(
                    ToggleButton::filled("toggle-save", "Save draft", self.selected)
                        .loading(self.loading)
                        .disabled(self.disabled)
                        .on_change(move |next, _, cx| {
                            v1.update(cx, |this, _| this.toggle_requests.push(next));
                        }),
                )
                .child(ToggleButton::tonal("toggle-tonal", "Tonal", false))
                .child(ToggleButton::elevated("toggle-elevated", "Elevated", true))
                .child(ToggleButton::outlined("toggle-outlined", "Outlined", false))
                .child(
                    IconToggleButton::filled(
                        "icon-toggle-pin",
                        "Pin item",
                        Icon::new(IconName::Add),
                        self.selected,
                    )
                    .loading(self.loading)
                    .disabled(self.disabled)
                    .on_change(move |next, _, cx| {
                        v2.update(cx, |this, _| this.icon_requests.push(next));
                    }),
                )
                .child(IconToggleButton::standard(
                    "icon-toggle-std",
                    "Standard",
                    Icon::new(IconName::Add),
                    false,
                ))
                .child(IconToggleButton::tonal(
                    "icon-toggle-tonal",
                    "Tonal",
                    Icon::new(IconName::Add),
                    true,
                ))
                .child(IconToggleButton::outlined(
                    "icon-toggle-out",
                    "Outlined",
                    Icon::new(IconName::Add),
                    false,
                ));
            if self.offscreen {
                div()
                    .w(px(200.))
                    .h(px(60.))
                    .overflow_hidden()
                    .child(div().mt(px(400.)).child(content))
            } else {
                div().p_4().child(content)
            }
        }
    }

    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let e = cx.new(|_| ToggleLoadingView {
                loading: false,
                disabled: false,
                offscreen: false,
                selected: true,
                toggle_requests: Vec::new(),
                icon_requests: Vec::new(),
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

    cx.update_window(handle.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        draw(window, cx);

        let resting_bounds = find(window, &[], &"toggle-save".into()).unwrap().bounds();
        let icon_resting_bounds = find(window, &[], &"icon-toggle-pin".into())
            .unwrap()
            .bounds();

        // Enter loading state: resting width stays identical and spinners render.
        view.update(cx, |this, cx| {
            this.loading = true;
            cx.notify();
        });
        draw(window, cx);

        let loading_bounds = find(window, &[], &"toggle-save".into()).unwrap().bounds();
        let icon_loading_bounds = find(window, &[], &"icon-toggle-pin".into())
            .unwrap()
            .bounds();
        assert_eq!(loading_bounds.size.width, resting_bounds.size.width);
        assert_eq!(
            icon_loading_bounds.size.width,
            icon_resting_bounds.size.width
        );

        let toggle_spinner: ElementId = (ElementId::from("toggle-save"), "loading-spinner").into();
        let icon_spinner: ElementId =
            (ElementId::from("icon-toggle-pin"), "loading-spinner").into();
        assert!(find(window, &[], &toggle_spinner).is_some());
        assert!(find(window, &[], &icon_spinner).is_some());

        // Pointer clicks and keyboard Space are suppressed while loading.
        for center in [loading_bounds.center(), icon_loading_bounds.center()] {
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
        }
        window.focus_next(cx);
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
        assert!(view.read(cx).toggle_requests.is_empty());
        assert!(view.read(cx).icon_requests.is_empty());

        // Reduced motion schedules zero frames; visible full motion schedules frames;
        // off-screen clipped loading schedules zero frames.
        assert_eq!(window.simulate_next_frame(cx), 0);
        cx.set_reduce_motion(false);
        draw(window, cx);
        assert!(window.simulate_next_frame(cx) > 0);
        view.update(cx, |this, cx| {
            this.offscreen = true;
            cx.notify();
        });
        draw(window, cx);
        assert_eq!(window.simulate_next_frame(cx), 0);

        // Exiting loading re-enables toggle activation.
        cx.set_reduce_motion(true);
        view.update(cx, |this, cx| {
            this.offscreen = false;
            this.loading = false;
            cx.notify();
        });
        draw(window, cx);
        for id in ["toggle-save", "icon-toggle-pin"] {
            let center = find(window, &[], &id.into()).unwrap().bounds().center();
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
        }
        draw(window, cx);
        assert_eq!(view.read(cx).toggle_requests, vec![false]);
        assert_eq!(view.read(cx).icon_requests, vec![false]);
    })
    .unwrap();
}
