use std::{cell::Cell, rc::Rc};

use gpui::{
    AppContext, Bounds, Context, InputEvent, InteractiveElement, IntoElement, KeyDownEvent,
    KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Pixels,
    Render, Role, StatefulInteractiveElement, Styled, TestAppContext, Window, div, px, size,
};
use gpui_base::{
    TestSupportExt,
    test_support::{find, find_all},
};
use gpui_material::{
    Button, Fab, Icon, IconName, MaterialRoot, Scaffold, ScaffoldContentLayout,
    ScaffoldFabPosition, ScaffoldInsets,
    foundation::{set_window_material_theme, window_material_theme},
    theme::Direction,
};

struct Example {
    top: Option<Pixels>,
    bottom: Option<Pixels>,
    rail: Option<Pixels>,
    fab: bool,
    position: Option<ScaffoldFabPosition>,
    snackbar: bool,
    insets: ScaffoldInsets,
    manual: bool,
    resolved: Rc<Cell<Option<ScaffoldContentLayout>>>,
    applied: Rc<Cell<usize>>,
    activations: Rc<Cell<usize>>,
}

impl Default for Example {
    fn default() -> Self {
        Self {
            top: Some(px(40.)),
            bottom: Some(px(56.)),
            rail: Some(px(64.)),
            fab: false,
            position: None,
            snackbar: false,
            insets: ScaffoldInsets::default(),
            manual: false,
            resolved: Rc::default(),
            applied: Rc::default(),
            activations: Rc::default(),
        }
    }
}

fn usable(activations: Rc<Cell<usize>>) -> impl IntoElement {
    div().id("usable").test_support().size_full().child(
        Button::new("content-action", "Content action")
            .on_activate(move |_, _| activations.set(activations.get() + 1)),
    )
}

impl Render for Example {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut scaffold = if self.manual {
            let resolved = self.resolved.clone();
            let applied = self.applied.clone();
            let activations = self.activations.clone();
            Scaffold::with_layout("example-scaffold", move |layout, _, _| {
                resolved.set(Some(layout));
                applied.set(applied.get() + 1);
                let p = layout.padding.resolve(layout.direction);
                div()
                    .size_full()
                    .pt(p.top)
                    .pb(p.bottom)
                    .pl(p.left)
                    .pr(p.right)
                    .child(usable(activations))
            })
        } else {
            Scaffold::new("example-scaffold", usable(self.activations.clone()))
        };
        if let Some(top) = self.top {
            scaffold = scaffold.app_bar(div().id("top").test_support().h(top).w_full());
        }
        if let Some(bottom) = self.bottom {
            scaffold = scaffold.bottom_bar(div().id("bottom").test_support().h(bottom).w_full());
        }
        if let Some(rail) = self.rail {
            scaffold = scaffold.rail(div().id("rail").test_support().w(rail).h_full());
        }
        if self.fab {
            let activations = self.activations.clone();
            scaffold = scaffold.fab(
                Fab::new("create", "Create item", Icon::new(IconName::Add))
                    .on_activate(move |_, _| activations.set(activations.get() + 1)),
            );
        }
        if let Some(position) = self.position {
            scaffold = scaffold.fab_position(position);
        }
        if self.snackbar {
            scaffold = scaffold.snackbar(
                div()
                    .id("notice")
                    .test_support()
                    .role(Role::Status)
                    .aria_label("Item created")
                    .h(px(36.)),
            );
        }
        scaffold.content_insets(self.insets)
    }
}

fn bounds(id: &'static str, window: &Window) -> Bounds<Pixels> {
    find(window, &[], &id.into()).unwrap().bounds()
}

fn open(
    cx: &mut TestAppContext,
    example: Example,
) -> (gpui::AnyWindowHandle, gpui::Entity<Example>) {
    cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|_| example);
        let window =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (window.into(), view)
    })
}

#[gpui::test]
fn scaffold_resolves_occupied_space_and_consumes_insets_once(cx: &mut TestAppContext) {
    let (handle, view) = open(
        cx,
        Example {
            manual: true,
            insets: ScaffoldInsets {
                top: px(14.),
                start: px(12.),
                bottom: px(16.),
                end: px(7.),
            },
            ..Default::default()
        },
    );
    cx.simulate_window_resize(handle, size(px(420.), px(360.)));
    let draw = |cx: &mut TestAppContext| {
        cx.update_window(handle, |_, window, cx| {
            let before = view.read(cx).applied.get();
            window.refresh();
            window.draw(cx).clear(cx);
            if view.read(cx).manual {
                assert_eq!(view.read(cx).applied.get(), before + 1);
            }
            let actions = find_all(window, &[], &"content-action".into());
            assert_eq!(actions.len(), 1);
            assert_eq!(actions[0].role(), Some(Role::Button));
            assert_eq!(actions[0].label(), Some("Content action"));
            (
                view.read(cx).resolved.get().unwrap(),
                bounds("example-scaffold", window),
                bounds("scaffold-content", window),
                bounds("usable", window),
            )
        })
        .unwrap()
    };
    let (layout, root, body, content) = draw(cx);
    assert_eq!(body, layout.bounds);
    assert_eq!(
        layout.occupied,
        ScaffoldInsets {
            top: px(40.),
            start: px(64.),
            bottom: px(56.),
            end: px(0.),
        }
    );
    assert_eq!(
        layout.padding,
        ScaffoldInsets {
            end: px(7.),
            ..Default::default()
        }
    );
    assert_eq!(content.left() - root.left(), px(64.));
    assert_eq!(content.top() - root.top(), px(40.));
    assert_eq!(root.bottom() - content.bottom(), px(56.));
    assert_eq!(root.right() - content.right(), px(7.));

    let large = ScaffoldInsets {
        top: px(90.),
        start: px(110.),
        bottom: px(92.),
        end: px(17.),
    };
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.top = Some(px(50.));
            view.rail = Some(px(80.));
            view.bottom = Some(px(66.));
            view.insets = large;
            cx.notify();
        })
    });
    cx.simulate_window_resize(handle, size(px(520.), px(420.)));
    let (layout, root, _, manual) = draw(cx);
    assert_eq!(root.size, size(px(520.), px(420.)));
    assert_eq!(
        layout.occupied,
        ScaffoldInsets {
            top: px(50.),
            start: px(80.),
            bottom: px(66.),
            end: px(0.),
        }
    );
    assert_eq!(
        layout.padding,
        ScaffoldInsets {
            top: px(40.),
            start: px(30.),
            bottom: px(26.),
            end: px(17.),
        }
    );
    assert_eq!(manual.left() - root.left(), large.start);
    assert_eq!(manual.top() - root.top(), large.top);
    assert_eq!(root.bottom() - manual.bottom(), large.bottom);
    assert_eq!(root.right() - manual.right(), large.end);

    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.manual = false;
            cx.notify();
        })
    });
    assert_eq!(
        draw(cx).3,
        manual,
        "automatic and explicit consumption match"
    );

    cx.update_window(handle, |_, window, cx| {
        let mut theme = window_material_theme(window, cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    })
    .unwrap();
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.manual = true;
            view.top = None;
            view.rail = None;
            view.bottom = None;
            cx.notify();
        })
    });
    let (layout, root, body, content) = draw(cx);
    assert_eq!(layout.direction, Direction::Rtl);
    assert_eq!(body, root);
    assert_eq!(layout.occupied, ScaffoldInsets::default());
    assert_eq!(layout.padding, large);
    assert_eq!(content.left() - root.left(), large.end);
    assert_eq!(root.right() - content.right(), large.start);

    cx.update_window(handle, |_, window, cx| {
        window.activate_window();
        window.focus_next(cx);
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            find(window, &[], &"content-action".into())
                .unwrap()
                .focused(),
            Some(true),
        );
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
        assert_eq!(view.read(cx).activations.get(), 1);
    })
    .unwrap();
}

#[gpui::test]
fn scaffold_fab_positions_and_snackbar_keep_independent_slots(cx: &mut TestAppContext) {
    use ScaffoldFabPosition::{Center, End, EndOverlay, Start};

    let (handle, view) = open(
        cx,
        Example {
            fab: true,
            snackbar: true,
            ..Example::default()
        },
    );
    cx.simulate_window_resize(handle, size(px(460.), px(400.)));
    let draw = |cx: &mut TestAppContext| {
        cx.update_window(handle, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let fab = find(window, &[], &"create".into()).unwrap();
            assert_eq!(find_all(window, &[], &"create".into()).len(), 1);
            assert_eq!(
                (fab.role(), fab.label()),
                (Some(Role::Button), Some("Create item"))
            );
            (
                bounds("example-scaffold", window),
                bounds("scaffold-content", window),
                fab.bounds(),
                find(window, &[], &"notice".into()).map(|notice| {
                    assert_eq!(notice.role(), Some(Role::Status));
                    notice.bounds()
                }),
            )
        })
        .unwrap()
    };
    let (_, body, default_fab, default_snackbar) = draw(cx);
    let default_snackbar = default_snackbar.unwrap();
    assert_eq!(default_fab.right(), body.right() - px(16.));
    assert_eq!(default_fab.bottom(), default_snackbar.top() - px(8.));
    assert_eq!(default_snackbar.bottom(), body.bottom() - px(16.));

    for rtl in [false, true] {
        cx.update_window(handle, |_, window, cx| {
            let mut theme = window_material_theme(window, cx).clone();
            theme.layout.direction = if rtl { Direction::Rtl } else { Direction::Ltr };
            set_window_material_theme(Some(theme), window, cx);
        })
        .unwrap();
        for position in [Start, Center, End, EndOverlay] {
            cx.update(|cx| {
                view.update(cx, |view, cx| {
                    view.position = Some(position);
                    view.insets.bottom = px(9.);
                    view.insets.start = px(80.);
                    view.insets.end = px(24.);
                    cx.notify();
                })
            });
            let (root, body, fab, notice) = draw(cx);
            let notice = notice.unwrap();
            let (left, right) = if rtl { (24., 16.) } else { (16., 24.) };
            let left = body.left() + px(left);
            let right = body.right() - px(right);
            match (position, rtl) {
                (Center, _) => assert_eq!(fab.center().x, left + (right - left) / 2.),
                (Start, false) | (End | EndOverlay, true) => {
                    assert_eq!(fab.left(), left + px(16.));
                }
                _ => assert_eq!(fab.right(), right - px(16.)),
            }
            if position == EndOverlay {
                assert_eq!(root.bottom() - fab.bottom(), px(25.));
                assert!(notice.bottom() <= fab.top() - px(8.));
                assert!(notice.bottom() <= body.bottom() - px(16.));
            } else {
                assert_eq!(fab.bottom(), notice.top() - px(8.));
                assert_eq!(notice.bottom(), body.bottom() - px(16.));
            }
        }
    }

    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.bottom = Some(px(100.));
            cx.notify();
        })
    });
    let (root, body, fab, notice) = draw(cx);
    assert_eq!(fab.bottom(), root.bottom() - px(25.));
    assert_eq!(notice.unwrap().bottom(), body.bottom() - px(16.));
    assert!(
        fab.top() > body.bottom(),
        "FAB overlaps the tall bottom bar"
    );
    cx.update_window(handle, |_, window, cx| {
        window.activate_window();
        let position = fab.center();
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
        assert_eq!(view.read(cx).activations.get(), 1);
    })
    .unwrap();

    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.position = Some(Center);
            view.top = None;
            view.bottom = None;
            view.rail = None;
            view.snackbar = false;
            view.insets.start = px(0.);
            view.insets.end = px(0.);
            cx.notify();
        })
    });
    let (root, body, fab, notice) = draw(cx);
    assert_eq!(body, root);
    assert_eq!(fab.center().x, body.center().x);
    assert_eq!(fab.bottom(), body.bottom() - px(25.));
    assert!(notice.is_none());

    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.fab = false;
            view.snackbar = true;
            cx.notify();
        })
    });
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(find(window, &[], &"create".into()).is_none());
        assert_eq!(bounds("notice", window).bottom(), root.bottom() - px(25.));
    })
    .unwrap();
}
