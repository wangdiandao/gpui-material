use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    App, AppContext, Context, Entity, InputEvent as _, IntoElement, ParentElement, Render,
    RenderOnce, Styled, TestAppContext, Window, WindowOptions, canvas, div, px,
};
use gpui_base::{Theme, ThemeAppearance, test_support::find};
use gpui_material::foundation::{
    color, current_material_theme, material_theme, set_material_theme, set_window_material_theme,
};
use gpui_material::theme::{Appearance, Argb, ColorRole, Direction, MaterialTheme};
use gpui_material::{
    Button, Dialog, DialogState, MaterialRoot, TextField, TextFieldState, ThemeScope,
};

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

type ThemeSamples = Rc<RefCell<Vec<(&'static str, Argb, Direction, f32)>>>;

#[derive(IntoElement)]
struct ScopedProbe(&'static str, ThemeSamples);

impl RenderOnce for ScopedProbe {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let record = move |window: &Window, cx: &App| {
            let theme = current_material_theme(window, cx);
            self.1.borrow_mut().push((
                self.0,
                theme.colors.color(ColorRole::Primary),
                theme.layout.direction,
                theme.typography.text_scale(),
            ));
        };
        record(window, cx);
        canvas(|_, _, _| {}, move |_, _, window, cx| record(window, cx)).size(px(16.))
    }
}

struct ScopedThemeView {
    theme: MaterialTheme,
    dialog: Entity<DialogState>,
    samples: ThemeSamples,
}

impl Render for ScopedThemeView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let probe = |name| ScopedProbe(name, self.samples.clone());
        div()
            .child(
                ThemeScope::new(
                    div()
                        .child(probe("outer"))
                        .child(ThemeScope::new(probe("inner")).direction(Direction::Ltr))
                        .child(
                            Dialog::new("scoped-dialog", &self.dialog, "Scoped")
                                .body(probe("dialog")),
                        ),
                )
                .theme(self.theme.clone())
                .direction(Direction::Rtl),
            )
            .child(probe("sibling"))
    }
}

#[gpui::test]
fn subtree_theme_reaches_the_dialog_and_restores_the_parent(cx: &mut TestAppContext) {
    let samples = ThemeSamples::default();
    let application = MaterialTheme::baseline(Appearance::Light);
    let mut scope = MaterialTheme::baseline(Appearance::Dark);
    scope.typography = scope.typography.with_text_scale(1.75);
    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        set_material_theme(application.clone(), cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let dialog = cx.new(DialogState::new);
            dialog.update(cx, |state, cx| state.open(window, cx));
            cx.new(|_| ScopedThemeView {
                theme: scope.clone(),
                dialog,
                samples: samples.clone(),
            })
        })
        .unwrap()
    });
    samples.borrow_mut().clear();
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            Theme::global(cx).tokens.colors.primary,
            color(application.colors.color(ColorRole::Primary))
        );
        assert_eq!(Theme::global(cx).tokens.typography.md.size, px(16.));
    })
    .unwrap();

    let scoped = scope.colors.color(ColorRole::Primary);
    let app = application.colors.color(ColorRole::Primary);
    let samples = samples.borrow();
    for expected in [
        ("outer", scoped, Direction::Rtl, 1.75),
        ("inner", scoped, Direction::Ltr, 1.75),
        ("dialog", scoped, Direction::Rtl, 1.75),
        ("sibling", app, Direction::Ltr, 1.0),
    ] {
        let matching: Vec<_> = samples
            .iter()
            .filter(|sample| sample.0 == expected.0)
            .collect();
        assert_eq!(matching, [&expected, &expected]);
    }
}

struct ScopedControlView {
    theme: MaterialTheme,
    field: Entity<TextFieldState>,
    activated: ThemeSamples,
}

impl Render for ScopedControlView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let activated = self.activated.clone();
        div().size_full().p_4().child(
            ThemeScope::new(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(TextField::new("scoped-field", "Name", &self.field))
                    .child(Button::new("scoped-trigger", "Inspect").on_activate(
                        move |window, cx| {
                            let theme = current_material_theme(window, cx);
                            activated.borrow_mut().push((
                                "trigger",
                                theme.colors.color(ColorRole::Primary),
                                theme.layout.direction,
                                theme.typography.text_scale(),
                            ));
                        },
                    )),
            )
            .theme(self.theme.clone())
            .direction(Direction::Rtl),
        )
    }
}

#[gpui::test]
fn subtree_theme_keeps_base_editor_and_bound_action_local(cx: &mut TestAppContext) {
    let activated = ThemeSamples::default();
    let application = MaterialTheme::baseline(Appearance::Light);
    let mut scope = MaterialTheme::baseline(Appearance::Dark);
    scope.typography = scope.typography.with_text_scale(1.5);
    let (window, field) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        set_material_theme(application.clone(), cx);
        let mut editor = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let field = cx.new(|cx| TextFieldState::new(window, cx).default_value("Scoped"));
            editor = Some(field.clone());
            cx.new(|_| ScopedControlView {
                theme: scope.clone(),
                field,
                activated: activated.clone(),
            })
        })
        .unwrap();
        (handle, editor.unwrap())
    });

    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        field.update(cx, |state, cx| state.focus(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
        let field_node = find(window, &[], &"scoped-field".into()).unwrap();
        let field_bounds = field_node.bounds().scale(window.scale_factor());
        let scoped_primary = color(scope.colors.color(ColorRole::Primary));
        assert!(
            window.painted_quads().iter().any(|quad| {
                quad.background == scoped_primary.into()
                    && quad.bounds.size.width <= px(2.).scale(window.scale_factor())
                    && field_bounds.contains(&quad.bounds.center())
            }),
            "scoped Base editor caret must use the subtree palette"
        );
        assert_eq!(
            Theme::global(cx).tokens.colors.primary,
            color(application.colors.color(ColorRole::Primary))
        );
        let trigger = find(window, &[], &"scoped-trigger".into()).unwrap();
        let position = trigger.bounds().center();
        for input in [
            gpui::MouseDownEvent {
                button: gpui::MouseButton::Left,
                position,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            gpui::MouseUpEvent {
                button: gpui::MouseButton::Left,
                position,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
        ] {
            window.dispatch_event(input, cx);
        }
    })
    .unwrap();
    cx.run_until_parked();

    assert_eq!(
        *activated.borrow(),
        [(
            "trigger",
            scope.colors.color(ColorRole::Primary),
            Direction::Rtl,
            1.5
        )]
    );
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
