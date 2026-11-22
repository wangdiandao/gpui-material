use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    App, AppContext, Context, Entity, InputEvent as _, IntoElement, ParentElement, Render,
    RenderOnce, Styled, TestAppContext, Window, WindowOptions, canvas, div, point, px,
};
use gpui_base::{Theme, ThemeAppearance, test_support::find};
use gpui_material::foundation::{
    MotionStatus, animate_with_status, color, current_base_theme, current_material_theme,
    material_theme, set_material_theme, set_window_material_theme,
};
use gpui_material::theme::{
    Appearance, Argb, ColorRole, CubicBezier, Direction, MaterialTheme, MotionSpec,
};
use gpui_material::{
    AdaptiveNavigationState, AdaptiveNavigationVisibility, Button, Dialog, DialogState,
    LinearProgress, LoadingIndicator, MaterialRoot, NavigationRailState, RailMode, TextAreaState,
    TextField, TextFieldState, ThemeScope,
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

type BaseThemeSample = Rc<RefCell<Option<(ThemeAppearance, gpui::Hsla, Option<gpui::Hsla>)>>>;

struct ScopedMultilineView {
    scope_theme: MaterialTheme,
    area: Entity<TextAreaState>,
    sampled_base: BaseThemeSample,
}

impl Render for ScopedMultilineView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let sampled_base = self.sampled_base.clone();
        let probe = canvas(
            |_, _, _| (),
            move |_, _, window, cx| {
                let base = current_base_theme(window, cx);
                *sampled_base.borrow_mut() = Some((
                    base.appearance,
                    base.tokens.colors.primary,
                    base.resizable.handle,
                ));
            },
        )
        .size_0();
        div().size_full().p_4().child(
            ThemeScope::new(
                div()
                    .w(px(320.))
                    .h(px(140.))
                    .child(TextField::multiline("scoped-area", "Notes", &self.area))
                    .child(probe),
            )
            .theme(self.scope_theme.clone()),
        )
    }
}

#[gpui::test]
fn subtree_theme_isolates_multiline_editor_scrollbar_and_current_base_theme(
    cx: &mut TestAppContext,
) {
    let light = MaterialTheme::baseline(Appearance::Light);
    let dark = MaterialTheme::baseline(Appearance::Dark);
    let sampled_base = Rc::new(RefCell::new(None));
    let long_text = (0..40)
        .map(|i| format!("Line {i:02} of multiline editor content"))
        .collect::<Vec<_>>()
        .join("\n");

    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        set_material_theme(light.clone(), cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let area = cx.new(|cx| {
                TextAreaState::new(window, cx)
                    .rows(3)
                    .default_value(long_text.clone())
            });
            cx.new(|_| ScopedMultilineView {
                scope_theme: dark.clone(),
                area,
                sampled_base: sampled_base.clone(),
            })
        })
        .unwrap()
    });

    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);
        let field_node = find(window, &[], &"scoped-area".into()).unwrap();
        let position = field_node.bounds().center();
        // Trigger a wheel scroll so DisplayScrollbar becomes visible.
        window.dispatch_event(
            gpui::ScrollEvent {
                position,
                delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-120.))),
                touch_phase: gpui::TouchPhase::Moved,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.refresh();
        window.draw(cx).clear(cx);

        // Global Base theme must remain Light while current_base_theme matches Dark.
        assert_eq!(Theme::global(cx).appearance, ThemeAppearance::Light);
        assert_eq!(
            Theme::global(cx).tokens.colors.primary,
            color(light.colors.color(ColorRole::Primary))
        );
        assert_eq!(
            *sampled_base.borrow(),
            Some((
                ThemeAppearance::Dark,
                color(dark.colors.color(ColorRole::Primary)),
                Some(color(dark.colors.color(ColorRole::OutlineVariant))),
            ))
        );

        // Painted Material editor scrollbar thumb must use the Dark scope's Outline token.
        let dark_thumb = color(dark.colors.color(ColorRole::Outline)).opacity(0.7);
        let light_thumb = color(light.colors.color(ColorRole::Outline)).opacity(0.7);
        assert_ne!(dark_thumb, light_thumb);
        let quads = window.painted_quads();
        assert!(
            quads
                .iter()
                .any(|quad| quad.background == dark_thumb.into()),
            "multiline editor scrollbar must paint with the local Dark ThemeScope token"
        );
    })
    .unwrap();
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

struct ClippedAnimationView {
    offscreen: bool,
}

impl Render for ClippedAnimationView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(300.))
            .h(px(100.))
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(if self.offscreen { px(300.) } else { px(0.) })
                    .flex_none(),
            )
            .child(LoadingIndicator::new("clipped-loading", "Loading"))
            .child(LinearProgress::new("clipped-progress", "Busy").indeterminate())
    }
}

#[gpui::test]
fn offscreen_clipped_loading_and_progress_pause_without_scheduling_frames(cx: &mut TestAppContext) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(false);
        let mut handle_view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|_| ClippedAnimationView { offscreen: true });
            handle_view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, handle_view.unwrap())
    });

    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();

    cx.update_window(window.into(), |_, window, cx| {
        assert!(window.is_window_active());
        window.refresh();
        window.draw(cx).clear(cx);
        // Both indicators are clipped outside the 100px overflow_hidden viewport:
        // zero animation frames should be scheduled.
        assert_eq!(window.simulate_next_frame(cx), 0);

        // Bring indicators into the visible viewport -> animations resume cleanly.
        view.update(cx, |state, cx| {
            state.offscreen = false;
            cx.notify();
        });
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(window.simulate_next_frame(cx) > 0);

        // Turning on reduced motion mid-flight stops requesting frames immediately.
        cx.set_reduce_motion(true);
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(window.simulate_next_frame(cx), 0);
    })
    .unwrap();
}

struct AnimatedChannelView {
    target: f32,
    spec: MotionSpec,
    last: Rc<RefCell<(f32, MotionStatus)>>,
}

impl Render for AnimatedChannelView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sample = animate_with_status("test-channel", self.target, self.spec, window, cx);
        *self.last.borrow_mut() = (sample.value, sample.status);
        div().size(px(32.))
    }
}

#[gpui::test]
fn unified_animation_reverses_proportionally_and_settles_on_reduced_motion(
    cx: &mut TestAppContext,
) {
    let linear = MotionSpec::Tween {
        duration_ms: 200,
        easing: CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        },
    };
    let last = Rc::new(RefCell::new((0.0, MotionStatus::Idle)));
    let (window, view, nav, rail) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(false);
        let nav = cx.new(|_| {
            AdaptiveNavigationState::new(AdaptiveNavigationVisibility::Shown)
                .transition_duration(Duration::from_millis(200))
        });
        let rail = cx.new(|_| {
            NavigationRailState::new(RailMode::Collapsed)
                .transition_duration(Duration::from_millis(200))
        });
        let mut handle_view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|_| AnimatedChannelView {
                target: 0.0,
                spec: linear,
                last: last.clone(),
            });
            handle_view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, handle_view.unwrap(), nav, rail)
    });

    // Establish initial resting value at 0.0.
    cx.update_window(window.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
        assert_eq!(*last.borrow(), (0.0, MotionStatus::Idle));
        assert_eq!(window.simulate_next_frame(cx), 0);

        // Start 200ms linear tween to 1.0.
        view.update(cx, |v, cx| {
            v.target = 1.0;
            cx.notify();
        });
        window.draw(cx).clear(cx);
        assert_eq!(last.borrow().1, MotionStatus::Running);
        assert!(window.simulate_next_frame(cx) > 0);
    })
    .unwrap();

    // Advance 50ms (25%) and reverse back to 0.0 -> reversal should finish in 50ms (at 100ms total).
    cx.executor().advance_clock(Duration::from_millis(50));
    cx.update_window(window.into(), |_, window, cx| {
        view.update(cx, |v, cx| {
            v.target = 0.0;
            cx.notify();
        });
        window.draw(cx).clear(cx);
        assert!((last.borrow().0 - 0.25).abs() < 0.02);
        assert_eq!(last.borrow().1, MotionStatus::Running);
        assert!(window.simulate_next_frame(cx) > 0);
    })
    .unwrap();

    cx.executor().advance_clock(Duration::from_millis(55));
    cx.update_window(window.into(), |_, window, cx| {
        // Drain any frame scheduled prior to the clock jump, then render the settled frame.
        window.simulate_next_frame(cx);
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(*last.borrow(), (0.0, MotionStatus::Finished));
        assert_eq!(window.simulate_next_frame(cx), 0);

        // Start another transition and flip reduce_motion mid-flight -> settles immediately.
        view.update(cx, |v, cx| {
            v.target = 1.0;
            cx.notify();
        });
        window.draw(cx).clear(cx);
        assert_eq!(last.borrow().1, MotionStatus::Running);
        assert!(window.simulate_next_frame(cx) > 0);

        cx.set_reduce_motion(true);
        window.draw(cx).clear(cx);
        assert_eq!(*last.borrow(), (1.0, MotionStatus::Finished));
        assert_eq!(window.simulate_next_frame(cx), 0);

        // Verify entity-owned AdaptiveNavigationState and NavigationRailState also honor reduce_motion.
        nav.update(cx, |s, cx| s.hide(cx));
        assert!(!nav.read(cx).is_animating());
        assert_eq!(nav.read(cx).progress(), 0.0);

        rail.update(cx, |s, cx| s.expand(cx));
        assert!(!rail.read(cx).is_animating());
        assert_eq!(rail.read(cx).current_value(), RailMode::Expanded);
    })
    .unwrap();
}

#[gpui::test]
fn theme_scope_density_and_roving_focus_home_end(cx: &mut TestAppContext) {
    use gpui::{Axis, FocusHandle, InteractiveElement};
    use gpui_material::foundation::move_roving_focus;
    use gpui_material::theme::Density;

    cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
    });
    let observed = Rc::new(RefCell::new(None));
    let observed_for_view = observed.clone();
    let handles_cell = Rc::new(RefCell::new(Vec::<FocusHandle>::new()));
    let handles_for_view = handles_cell.clone();

    struct DensityProbe {
        observed: Rc<RefCell<Option<(Direction, Density)>>>,
        handles: Rc<RefCell<Vec<FocusHandle>>>,
    }

    impl Render for DensityProbe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let handles = self.handles.borrow().clone();
            let observed = self.observed.clone();
            ThemeScope::new(
                ThemeScope::new(
                    div()
                        .size_full()
                        .children(handles.into_iter().enumerate().map(|(ix, handle)| {
                            div().id(("roving-item", ix)).track_focus(&handle).size_4()
                        }))
                        .child(canvas(
                            move |_, window, cx| {
                                let layout = current_material_theme(window, cx).layout;
                                *observed.borrow_mut() = Some((layout.direction, layout.density));
                            },
                            |_, _, _, _| {},
                        )),
                )
                .density(Density::Compact),
            )
            .direction(Direction::Rtl)
        }
    }

    let window = cx.update(|cx| {
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            *handles_for_view.borrow_mut() = (0..4).map(|_| cx.focus_handle()).collect();
            cx.new(|_| DensityProbe {
                observed: observed_for_view,
                handles: handles_for_view,
            })
        })
        .unwrap()
    });
    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
        assert_eq!(*observed.borrow(), Some((Direction::Rtl, Density::Compact)));

        let handles = handles_cell.borrow();
        let items = [
            (&handles[0], false),
            (&handles[1], true),
            (&handles[2], true),
            (&handles[3], false),
        ];
        handles[2].focus(window, cx);
        assert_eq!(
            move_roving_focus(&items, "home", Axis::Horizontal, Direction::Ltr, window, cx),
            Some(1)
        );
        assert!(handles[1].is_focused(window));
        assert_eq!(
            move_roving_focus(&items, "end", Axis::Vertical, Direction::Rtl, window, cx),
            Some(2)
        );
        assert!(handles[2].is_focused(window));
    })
    .unwrap();
}
