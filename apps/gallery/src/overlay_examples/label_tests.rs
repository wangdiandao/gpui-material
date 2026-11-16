use gpui::{
    FocusHandle, InputEvent as _, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, MouseButton,
    Point, Role, TestAppContext, VisualTestContext, WindowOptions, point, size,
};
use gpui_base::test_support::{ElementSnapshot, find};
use gpui_material::{MaterialRoot, Surface, theme::ColorRole};

use super::*;

#[derive(Clone, Copy)]
enum Scope {
    Primary,
    Neighbor,
    Alternate,
    Unbound,
}

struct Observation {
    first_source: Entity<InteractionSource>,
    other_source: Entity<InteractionSource>,
    scope: Scope,
    mount_alternate: bool,
    disabled: bool,
    focus: FocusHandle,
    activated: usize,
}

impl Render for Observation {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.weak_entity();
        let label = Label::new(
            "labelled-control",
            Button::new("primary", "Measure")
                .track_focus(&self.focus)
                .interaction_source(&self.first_source)
                .on_activate(move |_, cx| {
                    let _ = owner.update(cx, |view, cx| {
                        view.activated += 1;
                        cx.notify();
                    });
                }),
            "Measurement description",
        )
        .content(Text::new("Measurement description").id("custom-description"))
        .placement(TooltipPlacement::Below);
        let label = match self.scope {
            Scope::Primary => label.interaction_source(&self.first_source, "primary"),
            Scope::Neighbor => label.interaction_source(&self.first_source, "neighbor"),
            Scope::Alternate => label.interaction_source(&self.other_source, "alternate"),
            Scope::Unbound => label,
        };
        div()
            .pt(px(36.))
            .flex()
            .items_start()
            .gap_4()
            .child(div().w(px(140.)).child(label))
            .child(
                Button::new("neighbor", "Neighbor")
                    .disabled(self.disabled)
                    .interaction_source(&self.first_source),
            )
            .children(self.mount_alternate.then(|| {
                Button::new("alternate", "Alternate").interaction_source(&self.other_source)
            }))
    }
}

type Handle = gpui::WindowHandle<gpui_base::Root>;

fn draw(cx: &mut TestAppContext, handle: Handle) {
    for _ in 0..3 {
        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            window.simulate_next_frame(cx);
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
}

fn snapshot(window: &Window, id: &'static str) -> ElementSnapshot {
    find(window, &[], &id.into()).unwrap()
}

fn label(cx: &mut TestAppContext, handle: Handle) -> Option<ElementSnapshot> {
    cx.update_window(handle.into(), |_, window, _| {
        find(window, &[], &"label".into())
    })
    .unwrap()
}

fn move_to(cx: &mut TestAppContext, handle: Handle, position: Point<gpui::Pixels>) {
    VisualTestContext::from_window(handle.into(), cx).simulate_mouse_move(
        position,
        None,
        Modifiers::default(),
    );
    draw(cx, handle);
}

fn center(cx: &mut TestAppContext, handle: Handle, id: &'static str) -> Point<gpui::Pixels> {
    cx.update_window(handle.into(), |_, window, _| {
        snapshot(window, id).bounds().center()
    })
    .unwrap()
}

#[gpui::test]
fn label_observes_the_bound_control_and_leaves_keyboard_activation_with_the_anchor(
    cx: &mut TestAppContext,
) {
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|cx| Observation {
                first_source: cx.new(|_| InteractionSource::new()),
                other_source: cx.new(|_| InteractionSource::new()),
                scope: Scope::Primary,
                mount_alternate: true,
                disabled: false,
                focus: cx.focus_handle(),
                activated: 0,
            });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });
    draw(cx, handle);
    assert!(label(cx, handle).is_none());
    let primary = center(cx, handle, "primary");
    let neighbor = center(cx, handle, "neighbor");
    move_to(cx, handle, neighbor);
    assert!(label(cx, handle).is_none());
    move_to(cx, handle, primary);
    let open = label(cx, handle).unwrap();
    assert_eq!(open.role(), Some(Role::Tooltip));
    assert_eq!(open.label(), Some("Measurement description"));
    assert_eq!(open.focused(), None);
    cx.update_window(handle.into(), |_, window, _| {
        assert_eq!(snapshot(window, "primary").role(), Some(Role::Button));
        assert_eq!(snapshot(window, "primary").label(), Some("Measure"));
        assert!(
            find(window, open.path(), &"custom-description".into())
                .unwrap()
                .visible()
        );
    })
    .unwrap();

    VisualTestContext::from_window(handle.into(), cx).simulate_mouse_down(
        primary,
        MouseButton::Left,
        Modifiers::default(),
    );
    draw(cx, handle);
    cx.update(|cx| {
        assert!(
            view.read(cx)
                .first_source
                .read(cx)
                .state(handle.window_id(), &"primary".into())
                .unwrap()
                .pressed
        )
    });
    assert!(label(cx, handle).is_some());
    VisualTestContext::from_window(handle.into(), cx).simulate_mouse_up(
        primary,
        MouseButton::Left,
        Modifiers::default(),
    );
    move_to(cx, handle, point(px(12.), px(210.)));
    assert!(label(cx, handle).is_none());
    assert_eq!(cx.update(|cx| view.read(cx).activated), 1);

    cx.update_window(handle.into(), |_, window, cx| {
        view.read(cx).focus.clone().focus(window, cx);
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
    })
    .unwrap();
    draw(cx, handle);
    assert_eq!(cx.update(|cx| view.read(cx).activated), 2);
    assert!(label(cx, handle).is_some());
    cx.update_window(handle.into(), |_, window, cx| {
        assert!(view.read(cx).focus.is_focused(window));
        window.blur(cx);
    })
    .unwrap();
    draw(cx, handle);
    assert!(label(cx, handle).is_none());

    move_to(cx, handle, neighbor);
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.scope = Scope::Neighbor;
            cx.notify();
        })
    });
    draw(cx, handle);
    assert!(label(cx, handle).is_some());
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.disabled = true;
            cx.notify();
        })
    });
    draw(cx, handle);
    assert!(label(cx, handle).is_none());
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.scope = Scope::Alternate;
            cx.notify();
        })
    });
    draw(cx, handle);
    let alternate = center(cx, handle, "alternate");
    move_to(cx, handle, alternate);
    assert!(label(cx, handle).is_some());
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.mount_alternate = false;
            cx.notify();
        })
    });
    draw(cx, handle);
    assert!(label(cx, handle).is_none());
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.scope = Scope::Unbound;
            cx.notify();
        })
    });
    draw(cx, handle);
    move_to(cx, handle, primary);
    assert!(label(cx, handle).is_none());
}

#[derive(Clone, Copy)]
enum Edge {
    Top,
    Right,
    Bottom,
    Left,
    Clipped,
}

struct PersistentLabel {
    persistent: bool,
    edge: Edge,
    clicks: usize,
}

impl Render for PersistentLabel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.weak_entity();
        let anchor = div().absolute();
        let (anchor, placement) = match self.edge {
            Edge::Top => (anchor.top(px(0.)).left(px(2.)), TooltipPlacement::Above),
            Edge::Right => (anchor.right(px(0.)).top(px(44.)), TooltipPlacement::Right),
            Edge::Bottom => (anchor.bottom(px(0.)).right(px(2.)), TooltipPlacement::Below),
            Edge::Left => (anchor.left(px(0.)).bottom(px(44.)), TooltipPlacement::Left),
            Edge::Clipped => (anchor.left(px(-150.)).top(px(44.)), TooltipPlacement::Above),
        };
        div()
            .size_full()
            .relative()
            .child(
                Surface::clickable("beneath-label", "Beneath", ColorRole::Surface)
                    .size_full()
                    .on_activate(move |_, cx| {
                        let _ = owner.update(cx, |view, cx| {
                            view.clicks += 1;
                            cx.notify();
                        });
                    }),
            )
            .child(
                anchor.child(
                    Label::new("edge-label", div().size(px(28.)), "Details")
                        .persistent(self.persistent)
                        .placement(placement)
                        .max_width(px(110.)),
                ),
            )
    }
}

#[gpui::test]
fn persistent_label_fits_all_edges_and_does_not_capture_input(cx: &mut TestAppContext) {
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|_| PersistentLabel {
            persistent: true,
            edge: Edge::Top,
            clicks: 0,
        });
        let handle =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (handle, view)
    });
    cx.simulate_window_resize(handle.into(), size(px(320.), px(250.)));
    for edge in [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left] {
        cx.update(|cx| {
            view.update(cx, |view, cx| {
                view.edge = edge;
                cx.notify();
            })
        });
        draw(cx, handle);
        cx.update_window(handle.into(), |_, window, _| {
            let bounds = snapshot(window, "label").bounds();
            let viewport = window.viewport_size();
            let anchor = snapshot(window, "beneath-label").bounds();
            assert!(anchor.contains(&bounds.center()));
            assert!(bounds.top() >= px(8.) && bounds.bottom() <= viewport.height - px(8.));
            assert!(bounds.left() >= px(8.) && bounds.right() <= viewport.width - px(8.));
            assert!(bounds.size.width <= px(110.));
        })
        .unwrap();
    }
    let overlay = center(cx, handle, "label");
    VisualTestContext::from_window(handle.into(), cx).simulate_click(overlay, Modifiers::default());
    cx.simulate_keystrokes(handle.into(), "escape");
    draw(cx, handle);
    assert_eq!(cx.update(|cx| view.read(cx).clicks), 1);
    assert!(label(cx, handle).is_some());
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.edge = Edge::Clipped;
            cx.notify();
        })
    });
    draw(cx, handle);
    assert!(label(cx, handle).is_none());
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.edge = Edge::Top;
            cx.notify();
        })
    });
    draw(cx, handle);
    assert!(label(cx, handle).is_some());
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.persistent = false;
            cx.notify();
        })
    });
    draw(cx, handle);
    assert!(label(cx, handle).is_none());
}
