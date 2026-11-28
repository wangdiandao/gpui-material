use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, TestAppContext,
    Window, WindowOptions, div, px,
};
use gpui_base::{Theme, ThemeAppearance, test_support::find};
use gpui_material::foundation::{
    color, current_material_theme, material_theme, set_material_theme, set_window_material_theme,
};
use gpui_material::theme::{Appearance, ColorRole, MaterialTheme};
use gpui_material::{Dialog, DialogState, MaterialRoot, TextField, TextFieldState, ThemeScope};

struct ThemePreview {
    field: Entity<TextFieldState>,
    dialog_field: Entity<TextFieldState>,
    dialog: Entity<DialogState>,
}

impl Render for ThemePreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_4()
            .child(TextField::new("field", "Name", &self.field))
            .child(
                Dialog::new("dialog", &self.dialog, "Edit name").body(TextField::new(
                    "dialog-field",
                    "Name",
                    &self.dialog_field,
                )),
            )
    }
}

#[gpui::test]
fn window_themes_keep_base_editors_and_dialogs_local(cx: &mut TestAppContext) {
    cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
    });
    let mut windows = Vec::new();
    for appearance in [Appearance::Light, Appearance::Dark] {
        windows.push(cx.update(|cx| {
            let mut view = None;
            let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                if appearance == Appearance::Dark {
                    set_window_material_theme(
                        Some(MaterialTheme::baseline(appearance)),
                        window,
                        cx,
                    );
                }
                let entity = cx.new(|cx| ThemePreview {
                    field: cx.new(|cx| TextFieldState::new(window, cx).default_value("Initial")),
                    dialog_field: cx
                        .new(|cx| TextFieldState::new(window, cx).default_value("Draft")),
                    dialog: cx.new(DialogState::new),
                });
                view = Some(entity.clone());
                entity
            })
            .unwrap();
            (handle, view.unwrap())
        }));
    }

    // An application commit must update the non-overridden window while the
    // other keeps its local palette, regardless of the order they render.
    for application in [Appearance::Light, Appearance::Dark, Appearance::Light] {
        cx.update(|cx| set_material_theme(MaterialTheme::baseline(application), cx));
        for (ix, (handle, view)) in windows.iter().enumerate().rev() {
            let appearance = if ix == 1 {
                Appearance::Dark
            } else {
                application
            };
            let theme = MaterialTheme::baseline(appearance);
            for modal in [false, true] {
                cx.update_window((*handle).into(), |_, window, _| window.activate_window())
                    .unwrap();
                cx.run_until_parked();
                cx.update_window((*handle).into(), |_, window, cx| {
                    let preview = view.read(cx);
                    let dialog = preview.dialog.clone();
                    let field = if modal {
                        preview.dialog_field.clone()
                    } else {
                        preview.field.clone()
                    };
                    dialog.update(cx, |state, cx| {
                        if modal {
                            state.open(window, cx);
                        } else {
                            state.close(window, cx);
                        }
                    });
                    field.update(cx, |state, cx| state.focus(window, cx));
                    window.refresh();
                    window.draw(cx).clear(cx);
                    let id = if modal { "dialog-field" } else { "field" };
                    let field_node = find(window, &[], &id.into()).unwrap();
                    assert!(field_node.visible());
                    let field_bounds = field_node.bounds().scale(window.scale_factor());
                    let quads = window.painted_quads();
                    let primary = color(theme.colors.color(ColorRole::Primary));
                    assert!(
                        quads.iter().any(|quad| quad.background == primary.into()
                            && quad.bounds.size.width <= px(2.).scale(window.scale_factor())
                            && field_bounds.contains(&quad.bounds.center())),
                        "the Base caret must use the window's Material palette: {appearance:?}, modal={modal}, active={}", window.is_window_active()
                    );
                    if modal {
                        assert!(
                            quads.iter().any(|quad| quad.background
                                == color(theme.colors.color(ColorRole::SurfaceContainerHigh))
                                    .into()),
                            "the dialog surface must use the same window palette"
                        );
                    }
                    let base = Theme::global(cx);
                    assert_eq!(
                        base.appearance,
                        if application == Appearance::Light {
                            ThemeAppearance::Light
                        } else {
                            ThemeAppearance::Dark
                        }
                    );
                    assert_eq!(
                        base.tokens.colors.primary,
                        color(
                            MaterialTheme::baseline(application)
                                .colors
                                .color(ColorRole::Primary)
                        )
                    );
                })
                .unwrap();
            }
        }
    }

    // Clearing the local override rejoins the application theme on the next draw.
    let (handle, _) = &windows[1];
    cx.update_window((*handle).into(), |_, window, cx| {
        set_window_material_theme(None, window, cx);
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(window.painted_quads().iter().any(|quad| {
            quad.background
                == color(
                    MaterialTheme::baseline(Appearance::Light)
                        .colors
                        .color(ColorRole::SurfaceContainerHigh),
                )
                .into()
        }));
    })
    .unwrap();
}

#[gpui::test]
fn application_theme_transition_retargets_from_the_displayed_palette(cx: &mut TestAppContext) {
    let light = MaterialTheme::baseline(Appearance::Light);
    let mut dark = MaterialTheme::baseline(Appearance::Dark);
    dark.typography = dark.typography.with_text_scale(2.);
    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        set_material_theme(light.clone(), cx);
        cx.set_reduce_motion(false);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| EmptyThemeView)
        })
        .unwrap()
    });
    cx.update(|cx| set_material_theme(dark.clone(), cx));
    cx.update(|cx| {
        assert_eq!(material_theme(cx).colors, light.colors);
        assert_eq!(Theme::global(cx).tokens.typography.md.size, px(32.));
        assert_eq!(Theme::global(cx).tokens.typography.md.line_height, px(48.));
    });

    let sample = |window: &mut Window, cx: &mut App| {
        window.simulate_next_frame(cx);
        let theme = material_theme(cx);
        assert_eq!(
            Theme::global(cx).tokens.colors.primary,
            color(theme.colors.color(ColorRole::Primary))
        );
        theme.colors.clone()
    };
    cx.executor().advance_clock(Duration::from_millis(50));
    let midway = window
        .update(cx, |_, window, cx| sample(window, cx))
        .unwrap();
    assert_ne!(midway, light.colors);
    assert_ne!(midway, dark.colors);

    window
        .update(cx, |_, window, cx| {
            set_material_theme(light.clone(), cx);
            assert_eq!(sample(window, cx), midway);
        })
        .unwrap();
    cx.executor().advance_clock(Duration::from_secs(1));
    assert_eq!(
        window
            .update(cx, |_, window, cx| sample(window, cx))
            .unwrap(),
        light.colors
    );
    window
        .update(cx, |_, window, cx| {
            assert_eq!(window.simulate_next_frame(cx), 0);
        })
        .unwrap();

    cx.update(|cx| set_material_theme(dark.clone(), cx));
    cx.update(|cx| cx.set_reduce_motion(true));
    assert_eq!(
        window
            .update(cx, |_, window, cx| sample(window, cx))
            .unwrap(),
        dark.colors
    );
}

struct EmptyThemeView;

impl Render for EmptyThemeView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[gpui::test]
fn application_theme_transition_handles_driver_window_close(cx: &mut TestAppContext) {
    let light = MaterialTheme::baseline(Appearance::Light);
    let dark = MaterialTheme::baseline(Appearance::Dark);
    let (first, second) = cx.update(|cx| {
        crate::init_gallery(cx);
        set_material_theme(light.clone(), cx);
        cx.set_reduce_motion(false);
        let first = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| EmptyThemeView)
        })
        .unwrap();
        let second = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| EmptyThemeView)
        })
        .unwrap();
        (first, second)
    });

    cx.update_window(first.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.update(|cx| set_material_theme(dark.clone(), cx));
    cx.executor().advance_clock(Duration::from_millis(50));
    first
        .update(cx, |_, window, cx| {
            window.simulate_next_frame(cx);
            window.remove_window();
        })
        .unwrap();
    cx.run_until_parked();
    assert_ne!(
        cx.update(|cx| material_theme(cx).colors.clone()),
        dark.colors
    );

    cx.executor().advance_clock(Duration::from_secs(1));
    second
        .update(cx, |_, window, cx| {
            window.simulate_next_frame(cx);
        })
        .unwrap();
    assert_eq!(
        cx.update(|cx| material_theme(cx).colors.clone()),
        dark.colors
    );

    // Closing the last window during an active transition snaps to the target palette.
    cx.update(|cx| set_material_theme(light.clone(), cx));
    second
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
    cx.run_until_parked();
    assert_eq!(
        cx.update(|cx| material_theme(cx).colors.clone()),
        light.colors
    );
}

#[gpui::test]
fn window_override_bind_action_preserves_originating_window_theme(cx: &mut TestAppContext) {
    let light = MaterialTheme::baseline(Appearance::Light);
    let dark = MaterialTheme::baseline(Appearance::Dark);
    let (light_window, dark_window) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        set_material_theme(light.clone(), cx);
        let light_window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| EmptyThemeView)
        })
        .unwrap();
        let dark_window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            set_window_material_theme(Some(dark.clone()), window, cx);
            cx.new(|_| EmptyThemeView)
        })
        .unwrap();
        (light_window, dark_window)
    });

    let observed = Rc::new(RefCell::new(None));
    let observed_ref = observed.clone();
    let action = cx
        .update_window(dark_window.into(), |_, window, cx| {
            ThemeScope::bind_action(
                move |window, cx| {
                    *observed_ref.borrow_mut() =
                        Some(current_material_theme(window, cx).colors.appearance());
                },
                window,
                cx,
            )
        })
        .unwrap();

    // Invoking the bound action in the Light window still resolves the originating Dark window theme.
    cx.update_window(light_window.into(), |_, window, cx| {
        action(window, cx);
    })
    .unwrap();
    assert_eq!(*observed.borrow(), Some(Appearance::Dark));
}

#[gpui::test]
fn redundant_theme_updates_skip_window_redraws(cx: &mut TestAppContext) {
    struct CountingView {
        renders: Rc<std::cell::Cell<usize>>,
    }

    impl Render for CountingView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.renders.set(self.renders.get() + 1);
            div().size_full()
        }
    }

    let renders = Rc::new(std::cell::Cell::new(0usize));
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let renders = renders.clone();
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| CountingView { renders })
        })
        .unwrap()
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.run_until_parked();
    renders.set(0);

    // Identical application and window theme calls should be no-ops.
    cx.update_window(handle.into(), |_, window, cx| {
        let current = material_theme(cx).clone();
        set_material_theme(current, cx);
        set_window_material_theme(None, window, cx);
    })
    .unwrap();
    cx.run_until_parked();
    assert_eq!(renders.get(), 0);

    // Applying a distinct window override refreshes once; repeating it does not.
    let dark = MaterialTheme::baseline(Appearance::Dark);
    cx.update_window(handle.into(), |_, window, cx| {
        set_window_material_theme(Some(dark.clone()), window, cx);
    })
    .unwrap();
    cx.run_until_parked();
    assert_eq!(renders.get(), 1);

    renders.set(0);
    cx.update_window(handle.into(), |_, window, cx| {
        set_window_material_theme(Some(dark), window, cx);
    })
    .unwrap();
    cx.run_until_parked();
    assert_eq!(renders.get(), 0);
}
