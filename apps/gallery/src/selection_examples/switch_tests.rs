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
