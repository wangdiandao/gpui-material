use gpui::{
    AnyWindowHandle, App, AppContext, Context, Entity, Hsla, InputEvent, InteractiveElement,
    IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent,
    ParentElement, Render, RenderOnce, Role, Styled, TestAppContext, Window, WindowOptions, div,
    px, rgb, size,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{
    Button, ButtonColors, ControlBorder, ControlShapes, ControlSize, IconButton, IconButtonVariant,
    IconButtonWidth, IconToggleButton, InteractionSource, InteractionUpdate,
    foundation::{MaterialRoot, color, set_window_material_theme, window_material_theme},
    theme::{ColorRole, CornerShape, Direction},
};

use super::super::{
    custom_action, custom_action_color, custom_border_color, custom_toggle, custom_toggle_color,
    row,
};

#[gpui::test]
fn icon_buttons_customize_paint_and_keep_controlled_keyboard_behavior(cx: &mut TestAppContext) {
    const ACTION: u32 = 0x355675;
    const ACTION_INK: u32 = 0xb6d5ff;
    const DISABLED: u32 = 0x413e4a;
    const DISABLED_INK: u32 = 0xc3b9ce;
    const BORDER: u32 = 0x70529e;
    const CHECKED: u32 = 0x1f664c;
    const CHECKED_INK: u32 = 0xc1f1d3;

    #[derive(IntoElement)]
    struct Ink(&'static str);

    impl RenderOnce for Ink {
        fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
            div()
                .id(self.0)
                .test_support()
                .size(px(10.))
                .bg(window.text_style().color)
        }
    }

    struct Controls {
        source: Entity<InteractionSource>,
        events: Vec<InteractionUpdate>,
        activations: usize,
        requests: Vec<bool>,
        selected: bool,
        disabled: bool,
    }

    impl Render for Controls {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let action = cx.entity();
            let toggle = cx.entity();
            row()
                .child(
                    IconButton::new("action", "Add item", Ink("add-ink"))
                        .variant(IconButtonVariant::Outlined)
                        .size(ControlSize::Medium)
                        .width(IconButtonWidth::Wide)
                        .shapes(ControlShapes {
                            resting: Some(CornerShape::Radii([7.; 4])),
                            pressed: Some(CornerShape::Radii([3.; 4])),
                            ..Default::default()
                        })
                        .colors(ButtonColors {
                            container: Some(rgb(ACTION).into()),
                            content: Some(rgb(ACTION_INK).into()),
                            disabled_container: Some(rgb(DISABLED).into()),
                            disabled_content: Some(rgb(DISABLED_INK).into()),
                            ..Default::default()
                        })
                        .border(ControlBorder {
                            width: px(2.),
                            color: rgb(BORDER).into(),
                        })
                        .disabled(self.disabled)
                        .interaction_source(&self.source)
                        .on_activate(move |_, cx| {
                            action.update(cx, |model, _| model.activations += 1);
                        }),
                )
                .child(
                    IconToggleButton::new("toggle", "Pin item", Ink("pin-ink"), self.selected)
                        .selected_icon(Ink("pinned-ink"))
                        .variant(IconButtonVariant::FilledTonal)
                        .size(ControlSize::ExtraSmall)
                        .width(IconButtonWidth::Narrow)
                        .shapes(ControlShapes {
                            checked: Some(CornerShape::Radii([9.; 4])),
                            ..Default::default()
                        })
                        .colors(ButtonColors {
                            checked_container: Some(rgb(CHECKED).into()),
                            checked_content: Some(rgb(CHECKED_INK).into()),
                            ..Default::default()
                        })
                        .border(ControlBorder {
                            width: px(1.),
                            color: rgb(BORDER).into(),
                        })
                        .interaction_source(&self.source)
                        .on_change(move |selected, _, cx| {
                            toggle.update(cx, |model, _| model.requests.push(selected));
                        }),
                )
        }
    }

    let (handle, model) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let model = cx.new(|cx: &mut Context<Controls>| {
            let source = cx.new(|_| InteractionSource::new());
            cx.subscribe(&source, |model, _, event, _| {
                model.events.push(event.clone())
            })
            .detach();
            Controls {
                source,
                events: Vec::new(),
                activations: 0,
                requests: Vec::new(),
                selected: false,
                disabled: false,
            }
        });
        let handle =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| model.clone()).unwrap();
        (handle, model)
    });
    cx.simulate_window_scale_factor_change(handle.into(), 1.);
    cx.update_window(handle.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        let draw = |window: &mut Window, cx: &mut App| {
            window.refresh();
            window.draw(cx).clear(cx);
        };
        let quad = |fill, window: &Window| {
            let fill: Hsla = rgb(fill).into();
            crate::painted_quads(window)
                .into_iter()
                .find(|quad| quad.background == fill.into())
                .unwrap_or_else(|| panic!("Missing icon button color {fill:?}"))
        };
        let key = Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
        let press = |window: &mut Window, cx: &mut App| {
            window.dispatch_event(
                KeyDownEvent {
                    keystroke: key.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
        };
        let release = |window: &mut Window, cx: &mut App| {
            window.dispatch_event(
                KeyUpEvent {
                    keystroke: key.clone(),
                }
                .to_platform_input(),
                cx,
            );
        };

        draw(window, cx);
        let action = find(window, &[], &"action".into()).unwrap();
        let toggle = find(window, &[], &"toggle".into()).unwrap();
        assert_eq!(action.role(), Some(Role::Button));
        assert_eq!(action.label(), Some("Add item"));
        assert_eq!(toggle.role(), Some(Role::Button));
        assert_eq!(toggle.label(), Some("Pin item"));
        assert_eq!(toggle.checked(), Some(false));
        let paint = quad(ACTION, window);
        assert_eq!(
            paint.bounds.size,
            size(px(72.).scale(1.), px(56.).scale(1.))
        );
        assert_eq!(paint.corner_radii.top_left, px(7.).scale(1.));
        assert_eq!(paint.border_widths.top, px(2.).scale(1.));
        assert_eq!(paint.border_color, rgb(BORDER).into());
        assert_eq!(
            quad(ACTION_INK, window).bounds.center(),
            paint.bounds.center()
        );

        window.focus_next(cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"action".into()).unwrap().focused(),
            Some(true)
        );
        press(window, cx);
        draw(window, cx);
        assert_eq!(quad(ACTION, window).corner_radii.top_left, px(3.).scale(1.));
        release(window, cx);
        assert_eq!(model.read(cx).activations, 1);

        window.focus_next(cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"toggle".into()).unwrap().focused(),
            Some(true)
        );
        assert!(find(window, &[], &"pin-ink".into()).is_some());
        press(window, cx);
        release(window, cx);
        draw(window, cx);
        assert_eq!(model.read(cx).requests, [true]);
        assert_eq!(
            find(window, &[], &"toggle".into()).unwrap().checked(),
            Some(false)
        );

        model.update(cx, |model, cx| {
            model.selected = true;
            model.disabled = true;
            cx.notify();
        });
        draw(window, cx);
        window.focus_prev(cx);
        draw(window, cx);
        let paint = quad(CHECKED, window);
        assert_eq!(
            paint.bounds.size,
            size(px(28.).scale(1.), px(32.).scale(1.))
        );
        assert_eq!(paint.corner_radii.top_left, px(9.).scale(1.));
        assert_eq!(paint.border_widths.top, px(1.).scale(1.));
        assert_eq!(paint.border_color, rgb(BORDER).into());
        assert_eq!(
            quad(CHECKED_INK, window).bounds.center(),
            paint.bounds.center()
        );
        assert!(find(window, &[], &"pin-ink".into()).is_none());
        assert!(find(window, &[], &"pinned-ink".into()).is_some());
        assert_eq!(
            find(window, &[], &"toggle".into()).unwrap().checked(),
            Some(true)
        );

        let action = find(window, &[], &"action".into()).unwrap();
        assert_eq!(action.disabled(), Some(true));
        assert_ne!(action.focused(), Some(true));
        let paint = quad(DISABLED, window);
        assert_eq!(paint.border_color, rgb(BORDER).into());
        assert_eq!(
            quad(DISABLED_INK, window).bounds.center(),
            paint.bounds.center()
        );
        let position = action.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position,
                modifiers: Default::default(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position,
                modifiers: Default::default(),
                click_count: 1,
            }
            .to_platform_input(),
            cx,
        );
        assert_eq!(model.read(cx).activations, 1);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update(|cx| {
        let model = model.read(cx);
        for id in ["action", "toggle"] {
            assert!(model.events.iter().any(|event| {
                event.id == id.into() && event.state.is_some_and(|state| state.pressed)
            }));
        }
        let handle: AnyWindowHandle = handle.into();
        let state = model
            .source
            .read(cx)
            .state(handle.window_id(), &"action".into());
        assert!(state.is_some_and(|state| state.disabled && !state.pressed));
    });
}

#[gpui::test]
fn custom_button_states_and_slots_keep_token_defaults_and_controlled_values(
    cx: &mut TestAppContext,
) {
    #[derive(Default)]
    struct Controls {
        checks: Vec<bool>,
        checked: bool,
        disabled: bool,
        clicks: usize,
    }

    impl Render for Controls {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let toggle = cx.entity();
            let action = cx.entity();
            row()
                .child(
                    custom_action()
                        .disabled(self.disabled)
                        .on_activate(move |_, cx| action.update(cx, |model, _| model.clicks += 1)),
                )
                .child(Button::new("plain-action", "Plain"))
                .child(
                    custom_toggle(self.checked).on_change(move |selected, _, cx| {
                        toggle.update(cx, |model, _| model.checks.push(selected));
                    }),
                )
        }
    }

    let (handle, model) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let model = cx.new(|_| Controls::default());
        let window =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| model.clone()).unwrap();
        (window, model)
    });

    cx.update_window(handle.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        let draw = |window: &mut Window, cx: &mut App| {
            window.refresh();
            window.draw(cx).clear(cx);
        };
        let quad = |tint: Hsla, window: &Window| {
            crate::painted_quads(window)
                .into_iter()
                .find(|quad| quad.background == tint.into())
                .expect("button paints its instance or default container")
        };
        let check_padding = |start_on_left: bool, window: &Window| {
            let button = quad(custom_action_color(), window);
            let scale = window.scale_factor();
            let content = find(window, &[], &"custom-action-content".into())
                .expect("custom button content is rendered")
                .bounds();
            let left = f32::from(content.left()) - button.bounds.left().0 / scale;
            let right = button.bounds.right().0 / scale - f32::from(content.right());
            let (expected_left, expected_right) = if start_on_left {
                (22., 10.)
            } else {
                (10., 22.)
            };
            assert!((left - expected_left).abs() <= 1., "left={left}");
            assert!((right - expected_right).abs() <= 1., "right={right}");
        };
        let pointer = |id: &'static str, window: &Window| {
            find(window, &[], &id.into()).unwrap().bounds().center()
        };
        let down = |position, window: &mut Window, cx: &mut App| {
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
        };
        let up = |position, window: &mut Window, cx: &mut App| {
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
        };

        draw(window, cx);
        let node = find(window, &[], &"custom-action".into()).unwrap();
        assert_eq!(node.role(), Some(Role::Button));
        assert_eq!(node.label(), Some("Run the sample action"));
        let button = quad(custom_action_color(), window);
        let scale = window.scale_factor();
        assert_eq!(button.corner_radii.top_left.0 / scale, 3.);
        assert_eq!(button.border_widths.top.0 / scale, 2.);
        assert_eq!(button.border_color, custom_border_color());
        let primary = color(
            window_material_theme(window, cx)
                .colors
                .color(ColorRole::Primary),
        );
        assert_eq!(quad(primary, window).corner_radii.top_left.0 / scale, 20.);
        check_padding(true, window);

        let position = pointer("custom-action", window);
        down(position, window, cx);
        draw(window, cx);
        assert_eq!(
            quad(custom_action_color(), window).corner_radii.top_left.0 / scale,
            2.
        );
        up(position, window, cx);
        assert_eq!(model.read(cx).clicks, 1);

        let unchecked = color(
            window_material_theme(window, cx)
                .colors
                .color(ColorRole::SurfaceContainer),
        );
        let trigger = |window: &mut Window, cx: &mut App| {
            let position = pointer("custom-toggle", window);
            down(position, window, cx);
            up(position, window, cx);
        };
        trigger(window, cx);
        draw(window, cx);
        assert_eq!(model.read(cx).checks, [true]);
        assert_eq!(quad(unchecked, window).corner_radii.top_left.0 / scale, 20.);
        model.update(cx, |model, cx| {
            model.checked = true;
            cx.notify();
        });
        draw(window, cx);
        assert_eq!(
            quad(custom_toggle_color(), window).corner_radii.top_left.0 / scale,
            6.
        );

        // The toggle still owns keyboard focus after its pointer activation.
        let key = Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: key.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: key }.to_platform_input(), cx);
        assert_eq!(model.read(cx).checks, [true, false]);

        let mut theme = window_material_theme(window, cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        draw(window, cx);
        check_padding(false, window);
        assert_eq!(
            quad(custom_action_color(), window).corner_radii.top_left.0 / scale,
            7.
        );

        model.update(cx, |model, cx| {
            model.disabled = true;
            cx.notify();
        });
        draw(window, cx);
        let disabled_container = color(
            window_material_theme(window, cx)
                .colors
                .color(ColorRole::OnSurface),
        )
        .opacity(0.10);
        assert_eq!(
            quad(disabled_container, window).border_color,
            custom_border_color()
        );
        let position = pointer("custom-action", window);
        down(position, window, cx);
        up(position, window, cx);
        assert_eq!(model.read(cx).clicks, 1);
    })
    .unwrap();
}
