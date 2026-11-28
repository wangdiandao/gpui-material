use std::rc::Rc;

use gpui::{
    App, AppContext, Context, InputEvent as _, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, MouseUpEvent, ParentElement, Pixels, Point, Render, Role, ScrollHandle,
    TestAppContext, Window, WindowOptions, div, point, px,
};
use gpui_base::{TestSupportExt as _, test_support::find};
use gpui_material::{
    MaterialRoot, Menu, MenuColor, MenuItem, MenuItemStyle, MenuSurfaceStyle,
    foundation::{color, window_material_theme},
    theme::{ColorRole, CornerRole, Elevation},
};

#[gpui::test]
fn expressive_menu_paints_group_row_and_position_overrides(cx: &mut TestAppContext) {
    struct Example {
        scroll: ScrollHandle,
        position: Point<Pixels>,
        activations: usize,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity().downgrade();
            div().child(
                Menu::new(
                    "styled-menu",
                    gpui_base::Button::new("styled-trigger").child("Menu"),
                    [
                        MenuItem::section(
                            "project-group",
                            "Project",
                            [
                                MenuItem::new("group-first", "First").on_activate(move |_, cx| {
                                    let _ = owner.update(cx, |state, cx| {
                                        state.activations += 1;
                                        cx.notify();
                                    });
                                }),
                                MenuItem::new("group-selected", "Selected")
                                    .selected(true)
                                    .appearance(MenuItemStyle {
                                        selected_shape: Some(CornerRole::Medium.shape()),
                                        selected_container: Some(ColorRole::PrimaryContainer),
                                        selected_content: Some(ColorRole::OnPrimaryContainer),
                                        ..Default::default()
                                    }),
                            ],
                        )
                        .group_appearance(MenuSurfaceStyle {
                            shape: Some(CornerRole::ExtraLarge.shape()),
                            container: Some(ColorRole::SecondaryContainer),
                            border: Some((px(2.), ColorRole::Outline)),
                            ..Default::default()
                        }),
                        MenuItem::section(
                            "following-group",
                            "More actions",
                            (0..20usize).map(|index| {
                                MenuItem::new(("following-item", index), format!("Action {index}"))
                            }),
                        ),
                    ],
                )
                .color(MenuColor::Vibrant)
                .appearance(MenuSurfaceStyle {
                    shadow_elevation: Some(Elevation::Level4),
                    spacing: Some(px(4.)),
                    ..Default::default()
                })
                .content(|| {
                    div()
                        .id("menu-header")
                        .test_support()
                        .child("Style controls")
                })
                .position(self.position)
                .track_scroll(&self.scroll)
                .open(true),
            )
        }
    }

    let scroll = ScrollHandle::new();
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let view = cx.new(|_| Example {
            scroll: scroll.clone(),
            position: point(px(180.), px(100.)),
            activations: 0,
        });
        let window =
            MaterialRoot::open_window(WindowOptions::default(), cx, |_, _| view.clone()).unwrap();
        (window, view)
    });
    let window = window.into();
    for _ in 0..2 {
        cx.update_window(window, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
    let menu_position = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, _| {
            find(window, &[], &"menu".into()).unwrap().bounds().origin
        })
        .unwrap()
    };
    assert_eq!(menu_position(cx), point(px(180.), px(100.)));
    cx.update_window(window, |_, window, cx| {
        let group = find(window, &[], &"project-group".into()).unwrap();
        assert_eq!(group.role(), Some(Role::Group));
        assert_eq!(group.label(), Some("Project"));
        assert!(find(window, &[], &"menu-header".into()).is_some());
        let theme = window_material_theme(window, cx);
        let tint = |role| color(theme.colors.color(role));
        let quads = window.painted_quads();
        assert!(
            quads
                .iter()
                .any(|quad| quad.background == tint(ColorRole::TertiaryContainer).into())
        );
        let group_fill = quads
            .iter()
            .find(|quad| {
                quad.background == tint(ColorRole::SecondaryContainer).into()
                    && quad.corner_radii.top_left.0 == 28. * window.scale_factor()
            })
            .expect("styled group fill");
        assert!(quads.iter().any(|quad| {
            quad.bounds == group_fill.bounds
                && quad.border_color == tint(ColorRole::Outline)
                && quad.border_widths.top.0 == 2. * window.scale_factor()
        }));
        assert!(quads.iter().any(|quad| {
            quad.background == tint(ColorRole::PrimaryContainer).into()
                && quad.corner_radii.top_left.0 == 12. * window.scale_factor()
        }));
    })
    .unwrap();

    let focus = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| window.focused(cx))
            .unwrap()
    };
    let first = focus(cx).unwrap();
    cx.simulate_keystrokes(window, "down");
    let selected = focus(cx).unwrap();
    assert_ne!(selected, first);
    cx.simulate_keystrokes(window, "down up");
    assert_eq!(focus(cx), Some(selected));

    cx.update_window(window, |_, window, cx| {
        let row = find(window, &[], &"group-first".into()).unwrap();
        let position = row.bounds().center();
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
        assert_eq!(view.read(cx).activations, 1);
    })
    .unwrap();

    assert!(scroll.max_offset().y > px(60.));
    scroll.set_offset(point(px(0.), -px(60.)));
    cx.update_window(window, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    assert_eq!(scroll.offset().y, -px(60.));
    assert_eq!(menu_position(cx), point(px(180.), px(100.)));

    cx.update_window(window, |_, window, cx| {
        let viewport = window.viewport_size();
        view.update(cx, |state, cx| {
            state.position = point(viewport.width + px(40.), viewport.height + px(40.));
            cx.notify();
        });
        window.refresh();
        window.draw(cx).clear(cx);
        let menu = find(window, &[], &"menu".into()).unwrap().bounds();
        assert!(menu.right() <= viewport.width && menu.bottom() <= viewport.height);
    })
    .unwrap();
}

#[gpui::test]
fn menu_item_divider_shortcut_destructive_and_context_menu_ext(cx: &mut TestAppContext) {
    use std::cell::Cell;

    use gpui::{Hsla, RenderOnce, Styled};
    use gpui_material::{Button, MaterialElementExt, SemanticTone};

    #[derive(IntoElement)]
    struct InkProbe {
        slot: Rc<Cell<Option<Hsla>>>,
        label: &'static str,
    }

    impl RenderOnce for InkProbe {
        fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
            self.slot.set(Some(window.text_style().color));
            div().child(self.label)
        }
    }

    struct ContextMenuView {
        danger_ink: Rc<Cell<Option<Hsla>>>,
        disabled_danger_ink: Rc<Cell<Option<Hsla>>>,
    }

    impl Render for ContextMenuView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let danger_ink = self.danger_ink.clone();
            let disabled_danger_ink = self.disabled_danger_ink.clone();
            div().p_4().child(
                Button::outlined("ctx-trigger", "Right click me").context_menu(
                    "ext-context-menu",
                    [
                        MenuItem::new("save-cmd", "Save").shortcut("Ctrl+S"),
                        MenuItem::new("custom-trailing-cmd", "Export")
                            .shortcut("Ctrl+E")
                            .trailing(|| div().id("custom-trailing").test_support().child("TXT")),
                        MenuItem::divider("menu-sep"),
                        MenuItem::new("danger-cmd", "Delete")
                            .destructive()
                            .content(move || InkProbe {
                                slot: danger_ink.clone(),
                                label: "Delete",
                            }),
                        MenuItem::new("disabled-danger-cmd", "Purge")
                            .tone(SemanticTone::Danger)
                            .disabled(true)
                            .content(move || InkProbe {
                                slot: disabled_danger_ink.clone(),
                                label: "Purge",
                            }),
                        MenuItem::new("selected-success-cmd", "Pinned")
                            .selected(true)
                            .tone(SemanticTone::Success),
                    ],
                ),
            )
        }
    }

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|_| ContextMenuView {
                danger_ink: Rc::new(Cell::new(None)),
                disabled_danger_ink: Rc::new(Cell::new(None)),
            });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let window = window.into();
    let frame = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    frame(cx);

    // Secondary click opens the context menu created via MaterialElementExt::context_menu.
    cx.update_window(window, |_, window, cx| {
        let trigger = find(window, &[], &"ctx-trigger".into()).unwrap();
        let position = trigger.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Right,
                position,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Right,
                position,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx);
    frame(cx);

    cx.update_window(window, |_, window, cx| {
        let theme = window_material_theme(window, cx);
        let error = color(theme.colors.color(ColorRole::Error));
        let disabled_ink = color(theme.colors.color(ColorRole::OnSurface)).opacity(0.38);
        let success_container = color(theme.colors.color(ColorRole::SuccessContainer));

        assert!(find(window, &[], &"menu".into()).is_some());
        assert!(find(window, &[], &"menu-sep".into()).is_some());

        let save = find(window, &[], &"save-cmd".into()).unwrap();
        assert!(
            find(window, save.path(), &"shortcut".into()).is_some(),
            "shortcut hint renders in trailing slot"
        );

        let custom = find(window, &[], &"custom-trailing-cmd".into()).unwrap();
        assert!(find(window, custom.path(), &"custom-trailing".into()).is_some());
        assert!(
            find(window, custom.path(), &"shortcut".into()).is_none(),
            "explicit trailing slot overrides shortcut hint"
        );

        assert_eq!(view.read(cx).danger_ink.get(), Some(error));
        assert_eq!(view.read(cx).disabled_danger_ink.get(), Some(disabled_ink));
        assert!(
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background == success_container.into()),
            "selected Success menu item paints SuccessContainer"
        );
        assert_eq!(save.focused(), Some(true));
    })
    .unwrap();

    // Down arrow moves: save-cmd -> custom-trailing-cmd -> skips divider -> danger-cmd -> skips disabled -> selected-success-cmd.
    cx.simulate_keystrokes(window, "down");
    frame(cx);
    cx.update_window(window, |_, window, _| {
        assert_eq!(
            find(window, &[], &"custom-trailing-cmd".into())
                .unwrap()
                .focused(),
            Some(true)
        );
    })
    .unwrap();

    cx.simulate_keystrokes(window, "down");
    frame(cx);
    cx.update_window(window, |_, window, _| {
        assert_eq!(
            find(window, &[], &"danger-cmd".into()).unwrap().focused(),
            Some(true),
            "keyboard navigation must skip MenuItem::divider"
        );
    })
    .unwrap();

    cx.simulate_keystrokes(window, "down");
    frame(cx);
    cx.update_window(window, |_, window, _| {
        assert_eq!(
            find(window, &[], &"selected-success-cmd".into())
                .unwrap()
                .focused(),
            Some(true),
            "keyboard navigation skips disabled destructive item"
        );
    })
    .unwrap();

    // Up arrow skips back over disabled item and divider.
    cx.simulate_keystrokes(window, "up up");
    frame(cx);
    cx.update_window(window, |_, window, _| {
        assert_eq!(
            find(window, &[], &"custom-trailing-cmd".into())
                .unwrap()
                .focused(),
            Some(true)
        );
    })
    .unwrap();
}
