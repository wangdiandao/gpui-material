use gpui::{
    App, AppContext, Context, Entity, Hsla, InputEvent, InteractiveElement, IntoElement,
    KeyDownEvent, KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement,
    Render, Role, StrokeOptions, Styled, TestAppContext, Window, div, px, rgb, size,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{
    Checkbox, CheckboxState, InteractionSource, InteractionUpdate, MaterialRoot,
    foundation::{color, window_material_theme},
    theme::ColorRole,
};

use super::custom_checkbox_colors;

fn checked_boxes(window: &Window) -> usize {
    let checked: Hsla = rgb(0x285844).into();
    crate::painted_quads(window)
        .into_iter()
        .filter(|quad| quad.background.as_solid() == Some(checked))
        .count()
}

struct Checkboxes {
    state: CheckboxState,
    source: Entity<InteractionSource>,
    attach: bool,
    events: Vec<InteractionUpdate>,
    requests: Vec<CheckboxState>,
}

impl Render for Checkboxes {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut column = div().flex().flex_col().items_start();
        let colors = custom_checkbox_colors();
        let states = [
            ("mixed", "Selected items", self.state, false, false),
            ("unchecked", "Off", CheckboxState::Unchecked, false, false),
            ("checked", "On", CheckboxState::Checked, false, false),
            (
                "invalid",
                "Invalid",
                CheckboxState::Indeterminate,
                true,
                false,
            ),
            (
                "disabled",
                "Unavailable",
                CheckboxState::Checked,
                true,
                true,
            ),
        ];
        for (id, label, state, error, disabled) in states {
            let owner = cx.entity();
            let mut control = Checkbox::new(id, label, state)
                .colors(colors)
                .checkmark_stroke(StrokeOptions {
                    line_width: 4.,
                    ..StrokeOptions::default()
                })
                .outline_stroke(if id == "unchecked" { px(0.) } else { px(1.) })
                .error(error)
                .disabled(disabled)
                .on_change(move |next, _, cx| {
                    owner.update(cx, |this, _| this.requests.push(next));
                });
            if self.attach {
                control = control.interaction_source(&self.source);
            }
            column = column.child(control);
        }
        column.child(Checkbox::new("default", "Default", CheckboxState::Checked))
    }
}

fn draw(window: &mut Window, cx: &mut App) {
    window.refresh();
    window.draw(cx).clear(cx);
}

fn key(window: &mut Window, cx: &mut App) {
    let keystroke = Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
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

fn click(id: &'static str, window: &mut Window, cx: &mut App) {
    let position = find(window, &[], &id.into()).unwrap().bounds().center();
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
}

#[gpui::test]
fn checkbox_facade_paints_states_and_observes_controlled_input(cx: &mut TestAppContext) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|cx: &mut Context<Checkboxes>| {
            let source = cx.new(|_| InteractionSource::new());
            cx.subscribe(&source, |view, _, event, _| view.events.push(event.clone()))
                .detach();
            Checkboxes {
                state: CheckboxState::Indeterminate,
                source,
                attach: true,
                events: Vec::new(),
                requests: Vec::new(),
            }
        });
        let window =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (window.into(), view)
    });
    cx.simulate_window_scale_factor_change(window, 1.);
    cx.update_window(window, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        draw(window, cx);
        for (id, label, checked, mixed, disabled) in [
            ("mixed", "Selected items", false, true, false),
            ("unchecked", "Off", false, false, false),
            ("checked", "On", true, false, false),
            ("invalid", "Invalid", false, true, false),
            ("disabled", "Unavailable", true, false, true),
        ] {
            let node = find(window, &[], &id.into()).unwrap();
            assert_eq!(node.role(), Some(Role::CheckBox));
            assert_eq!(node.label(), Some(label));
            assert_eq!(node.checked(), Some(checked));
            assert_eq!(node.indeterminate(), Some(mixed));
            assert_eq!(node.disabled().unwrap_or(false), disabled);
        }
        for (fill, border, stroke) in [
            (0xe8ecf6, gpui::transparent_black(), 0.),
            (0x285844, rgb(0x182f29).into(), 1.),
            (0x415380, rgb(0x29364f).into(), 1.),
            (0x8b3a31, rgb(0x682c32).into(), 1.),
            (0xbbb7be, rgb(0x76717a).into(), 1.),
        ] {
            let fill: Hsla = rgb(fill).into();
            let quad = crate::painted_quads(window)
                .into_iter()
                .find(|quad| quad.background.as_solid() == Some(fill))
                .unwrap();
            assert_eq!(quad.bounds.size, size(px(18.).scale(1.), px(18.).scale(1.)));
            assert_eq!(quad.border_widths.top, px(stroke).scale(1.));
            assert_eq!(quad.border_color, border);
        }
        let default_fill = color(
            window_material_theme(window, cx)
                .colors
                .color(ColorRole::Primary),
        );
        let default = crate::painted_quads(window)
            .into_iter()
            .find(|quad| quad.background.as_solid() == Some(default_fill))
            .unwrap();
        assert_eq!(default.border_widths.top, px(2.).scale(1.));

        window.focus_next(cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"mixed".into()).unwrap().focused(),
            Some(true)
        );
        key(window, cx);
        click("mixed", window, cx);
        click("disabled", window, cx);
        assert_eq!(view.read(cx).requests, [CheckboxState::Checked; 2]);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"mixed".into()).unwrap().indeterminate(),
            Some(true)
        );
    })
    .unwrap();
    cx.run_until_parked();

    cx.update_window(window, |_, window, cx| {
        let model = view.read(cx);
        let window_id = window.window_handle().window_id();
        let disabled = model
            .source
            .read(cx)
            .state(window_id, &"disabled".into())
            .unwrap();
        assert!(disabled.disabled && !disabled.pressed);
        let states: Vec<_> = model
            .events
            .iter()
            .filter_map(|event| {
                (event.id == "mixed".into())
                    .then_some(event.state)
                    .flatten()
            })
            .collect();
        assert!(states.iter().any(|state| state.focus_visible));
        assert!(states.iter().any(|state| state.hovered && state.pressed));
        assert!(
            states
                .iter()
                .any(|state| state.focus_visible && state.pressed)
        );
        assert!(
            states
                .windows(2)
                .any(|pair| pair[0].pressed && !pair[1].pressed)
        );

        cx.set_reduce_motion(false);
        draw(window, cx);
        assert_eq!(checked_boxes(window), 1);
        view.update(cx, |view, cx| {
            view.state = CheckboxState::Checked;
            view.attach = false;
            cx.notify();
        });
        draw(window, cx);
        let node = find(window, &[], &"mixed".into()).unwrap();
        assert_eq!(node.checked(), Some(true));
        assert_eq!(node.indeterminate(), Some(false));
        assert_eq!(
            checked_boxes(window),
            1,
            "color retargets from its current value"
        );
    })
    .unwrap();
    cx.run_until_parked();
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(1));
    cx.update_window(window, |_, window, cx| {
        draw(window, cx);
        assert_eq!(checked_boxes(window), 2);
        let model = view.read(cx);
        assert!(
            model
                .source
                .read(cx)
                .state(window.window_handle().window_id(), &"mixed".into())
                .is_none()
        );
        assert!(
            model
                .events
                .iter()
                .any(|event| event.id == "mixed".into() && event.state.is_none())
        );
    })
    .unwrap();
}

#[gpui::test]
fn checkbox_show_label_and_label_content_keep_accessibility_and_interactivity(
    cx: &mut TestAppContext,
) {
    struct LabelVariants {
        compact_requests: Vec<CheckboxState>,
        rich_requests: Vec<CheckboxState>,
    }

    impl Render for LabelVariants {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let v1 = cx.entity();
            let v2 = cx.entity();
            div()
                .flex()
                .flex_col()
                .items_start()
                .child(Checkbox::new(
                    "labeled",
                    "Labeled option",
                    CheckboxState::Unchecked,
                ))
                .child(
                    Checkbox::new("compact", "Accept terms", CheckboxState::Unchecked)
                        .show_label(false)
                        .on_change(move |next, _, cx| {
                            v1.update(cx, |this, _| this.compact_requests.push(next));
                        }),
                )
                .child(
                    Checkbox::new("rich", "Accessible checkbox", CheckboxState::Unchecked)
                        .label_content(
                            div()
                                .id("rich-checkbox-content")
                                .test_support()
                                .child("Rich label"),
                        )
                        .on_change(move |next, _, cx| {
                            v2.update(cx, |this, _| this.rich_requests.push(next));
                        }),
                )
        }
    }

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut entity = None;
        let window = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let v = cx.new(|_| LabelVariants {
                compact_requests: Vec::new(),
                rich_requests: Vec::new(),
            });
            entity = Some(v.clone());
            v
        })
        .unwrap();
        (window.into(), entity.unwrap())
    });

    cx.update_window(window, |_, window, cx| {
        window.activate_window();
        draw(window, cx);

        let labeled = find(window, &[], &"labeled".into()).unwrap();
        let compact = find(window, &[], &"compact".into()).unwrap();
        let rich = find(window, &[], &"rich".into()).unwrap();

        assert_eq!(compact.label(), Some("Accept terms"));
        assert!(find(window, compact.path(), &"selection-label".into()).is_none());
        assert_eq!(compact.bounds().size.width, px(52.));
        assert!(compact.bounds().size.width < labeled.bounds().size.width);

        assert_eq!(rich.label(), Some("Accessible checkbox"));
        assert!(find(window, rich.path(), &"rich-checkbox-content".into()).is_some());

        click("compact", window, cx);
        click("rich", window, cx);
        draw(window, cx);

        assert_eq!(view.read(cx).compact_requests, vec![CheckboxState::Checked]);
        assert_eq!(view.read(cx).rich_requests, vec![CheckboxState::Checked]);
    })
    .unwrap();
}
