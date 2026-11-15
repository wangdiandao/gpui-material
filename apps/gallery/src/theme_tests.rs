use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, TestAppContext,
    Window, WindowOptions, div, px,
};
use gpui_base::{Theme, ThemeAppearance, test_support::find};
use gpui_material::foundation::{color, set_material_theme, set_window_material_theme};
use gpui_material::theme::{Appearance, ColorRole, MaterialTheme};
use gpui_material::{Dialog, DialogState, MaterialRoot, TextField, TextFieldState};

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
