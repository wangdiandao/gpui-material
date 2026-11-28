use std::{cell::RefCell, rc::Rc};

use gpui::{
    App, AppContext, Context, Entity, InputEvent as _, IntoElement, ParentElement, Render,
    RenderOnce, Styled, TestAppContext, Window, WindowOptions, canvas, div, point, px,
};
use gpui_base::{Theme, ThemeAppearance, test_support::find};
use gpui_material::foundation::{
    color, current_base_theme, current_material_theme, set_material_theme,
};
use gpui_material::theme::{Appearance, Argb, ColorRole, Direction, MaterialTheme};
use gpui_material::{
    Button, Dialog, DialogState, MaterialRoot, TextAreaState, TextField, TextFieldState, ThemeScope,
};

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
