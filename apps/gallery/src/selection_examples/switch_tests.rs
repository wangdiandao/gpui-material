use gpui::{
    AnyWindowHandle, App, AppContext, Context, ElementId, Entity, Hsla, InputEvent,
    InteractiveElement, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Render, RenderOnce, Role, Styled,
    TestAppContext, Window, WindowOptions, div, px, rgb, size,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{
    InteractionSource, InteractionUpdate, MaterialRoot, Switch, SwitchColors, SwitchStateColors,
    foundation::{set_window_material_theme, window_material_theme},
    theme::Direction,
};

// Track, border, thumb, content in unchecked, checked, disabled unchecked, and disabled checked order.
const PALETTES: [[u32; 4]; 4] = [
    [0x253b52, 0x76a4cf, 0xb8c8da, 0x282442],
    [0x14604d, 0x86aa92, 0xbde2ab, 0x243c29],
    [0x4e4045, 0x846970, 0xb39aa3, 0xf2dfdc],
    [0x5b5141, 0x9a8b6b, 0xb3af98, 0x2d2b2e],
];

fn state_colors([track, border, thumb, content]: [u32; 4]) -> SwitchStateColors {
    let paint = |color| Some(rgb(color).into());
    SwitchStateColors {
        track: paint(track),
        border: paint(border),
        thumb: paint(thumb),
        content: paint(content),
    }
}

#[derive(IntoElement)]
struct Thumb(bool);

impl RenderOnce for Thumb {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let (id, width, height) = if self.0 {
            ("marked-on", 10., 6.)
        } else {
            ("marked-off", 6., 10.)
        };
        div()
            .id(id)
            .test_support()
            .w(px(width))
            .h(px(height))
            .bg(window.text_style().color)
    }
}

struct Fixture {
    source: Entity<InteractionSource>,
    checked: bool,
    disabled: bool,
    requests: Vec<bool>,
    interactions: Vec<InteractionUpdate>,
}

impl Render for Fixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        let [unchecked, checked, disabled_unchecked, disabled_checked] = PALETTES.map(state_colors);
        div()
            .flex()
            .flex_col()
            .items_start()
            .p_4()
            .child(
                Switch::new("custom", "Sync notes", self.checked)
                    .checked_icon(div().id("unused-icon").test_support().size(px(16.)))
                    .thumb_content(Thumb(self.checked))
                    .colors(SwitchColors {
                        unchecked,
                        checked,
                        disabled_unchecked,
                        disabled_checked,
                    })
                    .disabled(self.disabled)
                    .interaction_source(&self.source)
                    .on_change(move |next, _, cx| {
                        owner.update(cx, |model, _| model.requests.push(next))
                    }),
            )
            .child(
                Switch::new("peer", "Sync is managed", false)
                    .disabled(true)
                    .interaction_source(&self.source),
            )
    }
}

fn draw(window: &mut Window, cx: &mut App) {
    window.refresh();
    window.draw(cx).clear(cx);
}

fn mouse(handle: AnyWindowHandle, pressed: bool, cx: &mut TestAppContext) {
    cx.update_window(handle, |_, window, cx| {
        let id = (ElementId::from("custom"), "switch-track").into();
        let position = find(window, &[], &id).unwrap().bounds().center();
        let input = if pressed {
            window.dispatch_event(
                MouseMoveEvent {
                    position,
                    pressed_button: None,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            MouseDownEvent {
                button: MouseButton::Left,
                position,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input()
        } else {
            MouseUpEvent {
                button: MouseButton::Left,
                position,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input()
        };
        window.dispatch_event(input, cx);
        draw(window, cx);
    })
    .unwrap();
}

#[gpui::test]
fn switch_custom_thumb_inherits_state_palette_and_stays_centered(cx: &mut TestAppContext) {
    let (handle, model) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let model = cx.new(|cx: &mut Context<Fixture>| {
            let source = cx.new(|_| InteractionSource::new());
            cx.subscribe(&source, |model, _, update, _| {
                model.interactions.push(update.clone())
            })
            .detach();
            Fixture {
                source,
                checked: false,
                disabled: false,
                requests: Vec::new(),
                interactions: Vec::new(),
            }
        });
        let handle =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| model.clone()).unwrap();
        (AnyWindowHandle::from(handle), model)
    });
    cx.simulate_window_scale_factor_change(handle, 1.);
    cx.update_window(handle, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();

    let assert_visual =
        |direction, checked, disabled, diameter, state_ix: usize, cx: &mut TestAppContext| {
            cx.update_window(handle, |_, window, cx| {
                draw(window, cx);
                let node = find(window, &[], &"custom".into()).unwrap();
                assert_eq!(node.role(), Some(Role::Switch));
                assert_eq!(node.label(), Some("Sync notes"));
                assert_eq!(node.checked(), Some(checked));
                assert_eq!(node.disabled().unwrap_or_default(), disabled);
                let bounds = |id| find(window, &[], &id).unwrap().bounds();
                let track = bounds((ElementId::from("custom"), "switch-track").into());
                let thumb = bounds((ElementId::from("custom"), "switch-thumb").into());
                let ink = bounds(if checked { "marked-on" } else { "marked-off" }.into());
                assert_eq!(track.size, size(px(52.), px(32.)));
                assert_eq!(
                    thumb.size,
                    size(px(diameter), px(diameter)),
                    "thumb in {direction:?}, checked={checked}, disabled={disabled}"
                );
                assert_eq!(ink.center(), thumb.center());
                assert_eq!(thumb.center().y, track.center().y);
                assert_eq!(
                    thumb.center().x > track.center().x,
                    checked == (direction == Direction::Ltr)
                );
                assert!(find(window, &[], &"unused-icon".into()).is_none());

                let [track, border, thumb, content] =
                    PALETTES[state_ix].map(|paint| Hsla::from(rgb(paint)));
                let quads = window.painted_quads();
                for paint in [track, thumb, content] {
                    assert!(
                        quads.iter().any(|quad| quad.background == paint.into()),
                        "missing fill {paint:?}"
                    );
                }
                assert!(
                    quads
                        .iter()
                        .any(|quad| quad.border_color == border && quad.border_widths.top.0 == 2.)
                );
            })
            .unwrap();
        };

    assert_visual(Direction::Ltr, false, false, 24., 0, cx);
    mouse(handle, true, cx);
    assert_visual(Direction::Ltr, false, false, 28., 0, cx);
    mouse(handle, false, cx);
    cx.run_until_parked();
    cx.update(|cx| {
        let model = model.read(cx);
        assert_eq!(model.requests, [true]);
        assert!(!model.checked);
        let states = model
            .interactions
            .iter()
            .filter(|update| update.id == "custom".into())
            .filter_map(|update| update.state)
            .collect::<Vec<_>>();
        assert!(states.iter().any(|state| state.pressed));
        assert!(states.last().is_some_and(|state| !state.pressed));
        let source = model.source.read(cx);
        assert!(
            source
                .state(handle.window_id(), &"peer".into())
                .is_some_and(|state| state.disabled)
        );
        assert!(
            source
                .state(handle.window_id(), &"custom".into())
                .is_some_and(|state| !state.disabled)
        );
    });

    cx.update_window(handle, |_, window, cx| {
        let mut theme = window_material_theme(window, cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    })
    .unwrap();
    assert_visual(Direction::Rtl, false, false, 24., 0, cx);
    cx.update(|cx| {
        model.update(cx, |model, cx| {
            model.checked = true;
            cx.notify();
        })
    });
    assert_visual(Direction::Rtl, true, false, 24., 1, cx);
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            MouseMoveEvent {
                position: gpui::point(px(1.), px(1.)),
                pressed_button: None,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.focus_next(cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"custom".into()).unwrap().focused(),
            Some(true)
        );
    })
    .unwrap();
    cx.run_until_parked();

    let key = |pressed, cx: &mut TestAppContext| {
        cx.update_window(handle, |_, window, cx| {
            let keystroke = Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
            let input = if pressed {
                KeyDownEvent {
                    keystroke,
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input()
            } else {
                KeyUpEvent { keystroke }.to_platform_input()
            };
            window.dispatch_event(input, cx);
            draw(window, cx);
        })
        .unwrap()
    };
    key(true, cx);
    assert_visual(Direction::Rtl, true, false, 28., 1, cx);
    key(false, cx);
    cx.update(|cx| assert_eq!(model.read(cx).requests, [true, false]));

    cx.update(|cx| {
        model.update(cx, |model, cx| {
            model.disabled = true;
            cx.notify();
        })
    });
    assert_visual(Direction::Rtl, true, true, 24., 3, cx);
    mouse(handle, true, cx);
    mouse(handle, false, cx);
    key(true, cx);
    key(false, cx);
    cx.update(|cx| {
        model.update(cx, |model, cx| {
            assert_eq!(model.requests, [true, false]);
            model.checked = false;
            cx.notify();
        })
    });
    assert_visual(Direction::Rtl, false, true, 24., 2, cx);
    cx.run_until_parked();
    cx.update(|cx| {
        let snapshot = model
            .read(cx)
            .source
            .read(cx)
            .state(handle.window_id(), &"custom".into())
            .unwrap();
        assert!(snapshot.disabled && !snapshot.pressed);
    });
}

#[gpui::test]
fn keyboard_switch_press_uses_the_same_thumb_feedback_as_pointer(cx: &mut TestAppContext) {
    use super::SelectionExamples;
    use crate::story::Story;

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let selection = cx.new(|cx| SelectionExamples::for_story(Story::Switch, cx));
            view = Some(selection.clone());
            selection
        })
        .unwrap();
        (handle, view.unwrap())
    });
    cx.update_window(handle.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        assert!(window.is_window_active());
        draw(window, cx);
        window.focus_next(cx);
        window.focus_next(cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &"sync-switch".into()).unwrap().focused(),
            Some(true)
        );
        let width = |window: &Window| {
            find(
                window,
                &[],
                &(ElementId::from("sync-switch"), "switch-thumb").into(),
            )
            .unwrap()
            .bounds()
            .size
            .width
        };
        assert_eq!(width(window), px(16.));
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
        draw(window, cx);
        assert_eq!(
            width(window),
            px(28.),
            "keyboard press must expand the handle like pointer press"
        );
        window.dispatch_event(KeyUpEvent { keystroke: key }.to_platform_input(), cx);
        draw(window, cx);
        assert!(view.read(cx).plain_switch);
        assert_eq!(width(window), px(24.));
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
        draw(window, cx);
        assert_eq!(width(window), px(28.));
        window.focus_prev(cx);
        draw(window, cx);
        assert_eq!(
            width(window),
            px(24.),
            "blur cancels a key press whose release goes elsewhere"
        );
        window.dispatch_event(KeyUpEvent { keystroke: key }.to_platform_input(), cx);
        draw(window, cx);
        assert!(
            view.read(cx).plain_switch,
            "cancelled press must not toggle the owner"
        );
    })
    .unwrap();
}

#[gpui::test]
fn switch_interpolates_size_and_position_and_respects_reduced_motion(cx: &mut TestAppContext) {
    use super::SelectionExamples;
    use std::time::Duration;

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let selection = cx.new(SelectionExamples::new);
            view = Some(selection.clone());
            selection
        })
        .unwrap();
        (window, view.unwrap())
    });
    let sample = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            draw(window, cx);
            let thumb = find(
                window,
                &[],
                &(ElementId::from("sync-switch"), "switch-thumb").into(),
            )
            .unwrap()
            .bounds();
            (thumb.center().x, thumb.size.width)
        })
        .unwrap()
    };
    let toggle = |cx: &mut TestAppContext, checked| {
        cx.update(|cx| {
            view.update(cx, |selection, cx| {
                selection.plain_switch = checked;
                cx.notify();
            })
        });
    };
    let start = sample(cx);
    toggle(cx, true);
    assert_eq!(sample(cx), start);
    cx.executor().advance_clock(Duration::from_millis(40));
    let middle = sample(cx);
    assert!(middle.0 > start.0);
    assert!(middle.1 > start.1 && middle.1 < px(24.));
    toggle(cx, false);
    assert_eq!(sample(cx), middle);
    cx.executor().advance_clock(Duration::from_secs(1));
    assert_eq!(sample(cx), start);
    cx.update(|cx| cx.set_reduce_motion(true));
    toggle(cx, true);
    let end = sample(cx);
    assert_eq!(end.1, px(24.));
    assert!(end.0 > middle.0);
}

#[gpui::test]
fn switch_show_label_and_label_content_keep_accessibility_and_interactivity(
    cx: &mut TestAppContext,
) {
    struct SwitchVariants {
        compact_requests: Vec<bool>,
        rich_requests: Vec<bool>,
    }

    impl Render for SwitchVariants {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let v1 = cx.entity();
            let v2 = cx.entity();
            div()
                .flex()
                .flex_col()
                .items_start()
                .p_2()
                .child(Switch::new("labeled-switch", "Standard label", false))
                .child(
                    Switch::new("compact-switch", "Dark mode", false)
                        .show_label(false)
                        .on_change(move |next, _, cx| {
                            v1.update(cx, |this, _| this.compact_requests.push(next));
                        }),
                )
                .child(
                    Switch::new("rich-switch", "Accessible switch", false)
                        .label_content(
                            div()
                                .id("rich-switch-content")
                                .test_support()
                                .child("Rich switch label"),
                        )
                        .on_change(move |next, _, cx| {
                            v2.update(cx, |this, _| this.rich_requests.push(next));
                        }),
                )
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut entity = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let v = cx.new(|_| SwitchVariants {
                compact_requests: Vec::new(),
                rich_requests: Vec::new(),
            });
            entity = Some(v.clone());
            v
        })
        .unwrap();
        (window.into(), entity.unwrap())
    });

    cx.update_window(handle, |_, window, cx| {
        window.activate_window();
        draw(window, cx);

        let labeled = find(window, &[], &"labeled-switch".into()).unwrap();
        let compact = find(window, &[], &"compact-switch".into()).unwrap();
        let rich = find(window, &[], &"rich-switch".into()).unwrap();

        assert_eq!(compact.label(), Some("Dark mode"));
        assert!(find(window, compact.path(), &"selection-label".into()).is_none());
        assert_eq!(compact.bounds().size.width, px(64.));
        assert!(compact.bounds().size.width < labeled.bounds().size.width);

        assert_eq!(rich.label(), Some("Accessible switch"));
        assert!(find(window, rich.path(), &"rich-switch-content".into()).is_some());

        for pos in [compact.bounds().center(), rich.bounds().center()] {
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
        draw(window, cx);

        assert_eq!(view.read(cx).compact_requests, vec![true]);
        assert_eq!(view.read(cx).rich_requests, vec![true]);
    })
    .unwrap();
}
