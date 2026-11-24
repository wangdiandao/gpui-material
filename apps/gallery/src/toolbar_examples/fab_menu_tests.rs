use std::time::Duration;

use gpui::{
    AppContext, Bounds, Context, ElementId, Entity, FocusHandle, Hsla, InputEvent, IntoElement,
    MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Pixels, Point, Render, Role, Styled,
    TestAppContext, Window, WindowHandle, canvas, div, point, px, size,
};
use gpui_base::{
    Root,
    test_support::{ElementSnapshot, find},
};
use gpui_material::{
    FabAlignment, FabMenu, FabMenuAlignment, FabMenuItem, FabMenuItemColors,
    FabMenuTriggerAppearance, FabMenuTriggerStyle, FabVisibilityAnimation, IconName, MaterialRoot,
    MaterialTheme,
    foundation::set_window_material_theme,
    theme::{Appearance, CornerShape, Direction, Elevation},
};

const CLOSED: Hsla = gpui::hsla(0.39, 0.68, 0.45, 1.);
const OPEN: Hsla = gpui::hsla(0.87, 0.42, 0.5, 1.);
const CLOSED_INK: Hsla = gpui::hsla(0.15, 0.81, 0.67, 1.);
const OPEN_INK: Hsla = gpui::hsla(0.46, 0.79, 0.24, 1.);
const ACTION: Hsla = gpui::hsla(0.62, 0.41, 0.35, 1.);
const ACTION_INK: Hsla = gpui::hsla(0.48, 0.61, 0.68, 1.);
const DISABLED: Hsla = gpui::hsla(0.15, 0.55, 0.37, 1.);

struct Example {
    focus: FocusHandle,
    expanded: bool,
    accepts_requests: bool,
    visible: bool,
    empty: bool,
    alignment: FabMenuAlignment,
    style: FabMenuTriggerStyle,
    requests: Vec<bool>,
    commands: usize,
}

fn swatch() -> gpui::Canvas<Hsla> {
    canvas(
        |_, window, _| window.text_style().color,
        |bounds, ink, window, _| window.paint_quad(gpui::fill(bounds, ink)),
    )
}

impl Render for Example {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.weak_entity();
        let item_owner = owner.clone();
        let items = (!self.empty)
            .then(|| {
                [
                    FabMenuItem::new("disabled", "Unavailable", IconName::Archive)
                        .disabled(true)
                        .colors(FabMenuItemColors {
                            disabled_container: Some(DISABLED),
                            ..Default::default()
                        }),
                    FabMenuItem::custom("custom", "Write a note", swatch().w(px(85.)).h(px(18.)))
                        .colors(FabMenuItemColors {
                            container: Some(ACTION),
                            content: Some(ACTION_INK),
                            ..Default::default()
                        })
                        .on_activate(move |_, cx| {
                            let _ = item_owner.update(cx, |view, cx| {
                                view.commands += 1;
                                cx.notify();
                            });
                        }),
                ]
            })
            .into_iter()
            .flatten();
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(
                FabMenu::new("menu", "Open note actions", IconName::Add, items)
                    .expanded(self.expanded)
                    .visible(self.visible)
                    .visibility_animation(FabVisibilityAnimation {
                        alignment: FabAlignment::BottomEnd,
                        ..Default::default()
                    })
                    .open_content("Replaced decorative text")
                    .open_icon(swatch().size_full())
                    .close_icon(swatch().size_full())
                    .close_label("Close note actions")
                    .alignment(self.alignment)
                    .trigger_style(self.style)
                    .track_focus(&self.focus)
                    .on_expanded_change(move |value, _, cx| {
                        let _ = owner.update(cx, |view, cx| {
                            view.requests.push(value);
                            if view.accepts_requests {
                                view.expanded = value;
                            }
                            cx.notify();
                        });
                    }),
            )
    }
}

type TestWindow = WindowHandle<Root>;

fn open(cx: &mut TestAppContext, reduced: bool) -> (TestWindow, Entity<Example>) {
    cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(reduced);
        let focus = cx.focus_handle();
        let model = cx.new(|_| Example {
            focus,
            expanded: false,
            accepts_requests: false,
            visible: true,
            empty: false,
            alignment: FabMenuAlignment::End,
            style: FabMenuTriggerStyle {
                collapsed: FabMenuTriggerAppearance {
                    size: Some(px(88.)),
                    shape: Some(CornerShape::Radii([4., 14., 20., 9.])),
                    icon_size: Some(px(30.)),
                    container_color: Some(CLOSED),
                    content_color: Some(CLOSED_INK),
                    elevation: Some(Elevation::Level0),
                },
                expanded: FabMenuTriggerAppearance {
                    size: Some(px(64.)),
                    shape: Some(CornerShape::Radii([8., 18., 23., 12.])),
                    icon_size: Some(px(18.)),
                    container_color: Some(OPEN),
                    content_color: Some(OPEN_INK),
                    elevation: Some(Elevation::Level2),
                },
            },
            requests: Vec::new(),
            commands: 0,
        });
        let window =
            MaterialRoot::open_window(Default::default(), cx, |_, _| model.clone()).unwrap();
        (window, model)
    })
}

fn update(model: &Entity<Example>, cx: &mut TestAppContext, mutate: impl FnOnce(&mut Example)) {
    cx.update(|cx| {
        model.update(cx, |model, cx| {
            mutate(model);
            cx.notify();
        })
    });
}

fn frame(window: TestWindow, cx: &mut TestAppContext) {
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn node(window: &Window, id: &'static str) -> Option<ElementSnapshot> {
    find(window, &[], &(ElementId::from("menu"), id).into())
}

fn click(window: TestWindow, cx: &mut TestAppContext, position: Point<Pixels>) {
    cx.update_window(window.into(), |_, window, cx| {
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
    })
    .unwrap();
    frame(window, cx);
    frame(window, cx);
}

fn read(window: TestWindow, cx: &mut TestAppContext, id: &'static str) -> ElementSnapshot {
    cx.update_window(window.into(), |_, window, _| node(window, id).unwrap())
        .unwrap()
}

#[gpui::test]
fn fab_menu_controlled_requests_do_not_commit_or_move_focus_by_themselves(cx: &mut TestAppContext) {
    let (window, model) = open(cx, true);
    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        model.read(cx).focus.clone().focus(window, cx);
    })
    .unwrap();
    frame(window, cx);
    let trigger = read(window, cx, "trigger");
    assert_eq!(trigger.label(), Some("Open note actions"));
    let position = trigger.bounds().center();
    click(window, cx, position);
    let trigger = read(window, cx, "trigger");
    assert_eq!(trigger.expanded(), Some(false));
    assert_eq!(trigger.focused(), Some(true));
    cx.update(|cx| assert_eq!(model.read(cx).requests, [true]));

    update(&model, cx, |model| model.accepts_requests = true);
    frame(window, cx);
    click(window, cx, position);
    let trigger = read(window, cx, "trigger");
    assert_eq!(trigger.expanded(), Some(true));
    assert_eq!(trigger.label(), Some("Close note actions"));
    let custom = read(window, cx, "custom");
    assert_eq!(custom.role(), Some(Role::MenuItem));
    assert_eq!(custom.label(), Some("Write a note"));
    assert_eq!(custom.focused(), Some(true));
    let disabled = read(window, cx, "disabled");
    assert_eq!(disabled.disabled(), Some(true));
    click(window, cx, disabled.bounds().center());
    cx.update(|cx| assert_eq!(model.read(cx).requests, [true, true]));

    update(&model, cx, |model| model.accepts_requests = false);
    frame(window, cx);
    cx.simulate_keystrokes(window.into(), "escape");
    frame(window, cx);
    assert_eq!(read(window, cx, "custom").focused(), Some(true));
    click(window, cx, point(px(14.), px(14.)));
    assert_eq!(read(window, cx, "trigger").expanded(), Some(true));
    cx.simulate_keystrokes(window.into(), "enter");
    frame(window, cx);
    let item_position = read(window, cx, "custom").bounds().center();
    click(window, cx, item_position);
    cx.update(|cx| {
        let model = model.read(cx);
        assert_eq!(model.requests, [true, true, false, false, false, false]);
        assert_eq!(model.commands, 2);
    });

    update(&model, cx, |model| model.accepts_requests = true);
    frame(window, cx);
    click(window, cx, point(px(14.), px(14.)));
    assert_eq!(read(window, cx, "trigger").focused(), Some(true));
    assert_eq!(read(window, cx, "trigger").expanded(), Some(false));
    cx.simulate_keystrokes(window.into(), "enter");
    frame(window, cx);
    frame(window, cx);
    assert_eq!(read(window, cx, "custom").focused(), Some(true));
    cx.simulate_keystrokes(window.into(), "escape");
    frame(window, cx);
    assert_eq!(read(window, cx, "trigger").focused(), Some(true));

    update(&model, cx, |model| model.expanded = true);
    frame(window, cx);
    frame(window, cx);
    assert_eq!(read(window, cx, "custom").focused(), Some(true));
    update(&model, cx, |model| model.visible = false);
    frame(window, cx);
    cx.update_window(window.into(), |_, window, cx| {
        assert!(node(window, "trigger").is_none());
        assert!(node(window, "custom").is_none());
        assert!(window.focused(cx).is_none());
    })
    .unwrap();
    update(&model, cx, |model| model.visible = true);
    frame(window, cx);
    frame(window, cx);
    assert_eq!(read(window, cx, "custom").focused(), Some(true));
    update(&model, cx, |model| model.empty = true);
    frame(window, cx);
    let disabled = read(window, cx, "trigger");
    assert_eq!(disabled.disabled(), Some(true));
    assert_eq!(disabled.expanded(), Some(false));
    click(window, cx, disabled.bounds().center());
    cx.update(|cx| assert_eq!(model.read(cx).commands, 2));
}

fn painted(window: &Window, pigment: Hsla) -> (Bounds<Pixels>, f32) {
    let scale = window.scale_factor();
    let quad = crate::painted_quads(window)
        .into_iter()
        .find(|quad| quad.background.as_solid() == Some(pigment))
        .expect("expected menu pigment");
    (
        quad.bounds.map(|side| px(side.0 / scale)),
        quad.corner_radii.top_left.0 / scale,
    )
}

#[gpui::test]
fn fab_menu_trigger_animates_and_items_align_in_rtl_and_narrow_windows(cx: &mut TestAppContext) {
    let (window, model) = open(cx, false);
    frame(window, cx);
    cx.update_window(window.into(), |_, window, _| {
        let (bounds, corner) = painted(window, CLOSED);
        assert_eq!(bounds.size.width, px(88.));
        assert_eq!(corner, 4.);
        assert_eq!(painted(window, CLOSED_INK).0.size.width, px(30.));
    })
    .unwrap();
    update(&model, cx, |model| model.expanded = true);
    frame(window, cx);
    cx.executor().advance_clock(Duration::from_millis(40));
    frame(window, cx);
    let intermediate = read(window, cx, "trigger").bounds().size.width;
    assert!(px(64.) < intermediate && intermediate < px(88.));
    cx.executor().advance_clock(Duration::from_secs(1));
    frame(window, cx);
    cx.update_window(window.into(), |_, window, _| {
        let (bounds, corner) = painted(window, OPEN);
        assert_eq!(bounds.size.width, px(64.));
        assert_eq!(corner, 8.);
        assert_eq!(painted(window, OPEN_INK).0.size.width, px(18.));
        let action = painted(window, ACTION).0;
        assert!(action.contains(&painted(window, ACTION_INK).0.center()));
        assert!(painted(window, DISABLED).0.size.width > px(0.));
    })
    .unwrap();

    let open = read(window, cx, "trigger").bounds();
    update(&model, cx, |model| model.visible = false);
    frame(window, cx);
    cx.update_window(window.into(), |_, window, _| {
        assert!(node(window, "trigger").is_none());
        assert!(node(window, "custom").is_none());
        assert!(node(window, "exit").is_some());
    })
    .unwrap();
    click(window, cx, open.center());
    cx.update(|cx| assert!(model.read(cx).requests.is_empty()));
    update(&model, cx, |model| model.visible = true);
    frame(window, cx);
    frame(window, cx);
    assert_eq!(read(window, cx, "trigger").expanded(), Some(true));

    cx.update(|cx| cx.set_reduce_motion(true));
    for appearance in [Appearance::Light, Appearance::Dark] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for alignment in [
                FabMenuAlignment::Start,
                FabMenuAlignment::Center,
                FabMenuAlignment::End,
            ] {
                update(&model, cx, |model| model.alignment = alignment);
                cx.update_window(window.into(), |_, window, cx| {
                    let mut theme = MaterialTheme::baseline(appearance);
                    theme.layout.direction = direction;
                    set_window_material_theme(Some(theme), window, cx);
                })
                .unwrap();
                frame(window, cx);
                let target = read(window, cx, "trigger").bounds();
                let item = read(window, cx, "custom").bounds();
                let difference = match (alignment, direction) {
                    (FabMenuAlignment::Center, _) => item.center().x - target.center().x,
                    (FabMenuAlignment::Start, Direction::Ltr)
                    | (FabMenuAlignment::End, Direction::Rtl) => {
                        item.left() - px(4.) - target.left()
                    }
                    _ => item.right() + px(4.) - target.right(),
                };
                assert!(
                    difference.abs() <= px(1.),
                    "{appearance:?} {direction:?} {alignment:?}: {difference:?}"
                );
            }
        }
    }

    cx.simulate_window_resize(window.into(), size(px(180.), px(200.)));
    update(&model, cx, |model| {
        model.style.expanded.size = Some(px(f32::NAN));
        model.style.expanded.icon_size = Some(px(1000.));
    });
    frame(window, cx);
    let target = read(window, cx, "trigger").bounds();
    assert_eq!(target.size.width, px(56.));
    let item = read(window, cx, "custom").bounds();
    assert!(item.left() >= px(0.) && item.right() <= px(180.));
    assert!(item.size.width <= px(156.));
}
