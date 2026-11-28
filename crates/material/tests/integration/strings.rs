//! Integration tests for centralized `MaterialStrings`, `set_material_strings`,
//! `current_material_strings`, and `ThemeScope::strings`.

use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext, Context, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, Styled, TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use gpui_material::{
    Banner, Dialog, DialogState, DropdownOption, ExposedDropdown, ExposedDropdownState,
    MaterialRoot, MaterialStrings, MaterialTheme, Scrim, Sheet, SheetEdge, SheetMode, TextField,
    TextFieldState, ThemeScope, current_material_strings, material_strings, set_material_strings,
    test::TestWindowExt as _, theme::Density,
};

fn test_window_options() -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(gpui::Bounds {
            origin: gpui::Point::default(),
            size: size(px(900.), px(700.)),
        })),
        ..Default::default()
    }
}

fn id_pair(parent: &'static str, child: &'static str) -> ElementId {
    (ElementId::Name(parent.into()), child).into()
}

#[gpui::test]
fn global_material_strings_update_default_component_labels(cx: &mut TestAppContext) {
    struct ComponentsView {
        clearable: Entity<TextFieldState>,
        password: Entity<TextFieldState>,
        password_visible: bool,
        dropdown: Entity<ExposedDropdownState>,
        dialog: Entity<DialogState>,
        sheet: Entity<DialogState>,
    }

    impl ComponentsView {
        fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
            let clearable = cx.new(|cx| TextFieldState::new(window, cx).default_value("hello"));
            let password = cx.new(|cx| {
                TextFieldState::new(window, cx)
                    .masked(true)
                    .default_value("secret")
            });
            let dropdown = cx.new(|cx| ExposedDropdownState::new("", false, window, cx));
            let dialog = cx.new(|cx| {
                let mut state = DialogState::new(cx);
                state.open(window, cx);
                state
            });
            let sheet = cx.new(|cx| {
                let mut state = DialogState::new(cx);
                state.open(window, cx);
                state
            });
            Self {
                clearable,
                password,
                password_visible: false,
                dropdown,
                dialog,
                sheet,
            }
        }
    }

    impl Render for ComponentsView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let entity = cx.entity();
            div()
                .size_full()
                .flex()
                .flex_col()
                .gap_2()
                .p_4()
                .child(Banner::new("notice", "Offline mode").on_dismiss(|_, _| {}))
                .child(TextField::new("search", "Search", &self.clearable).clearable(true))
                .child(
                    TextField::secure("secret", "Password", &self.password).visibility_toggle(
                        self.password_visible,
                        move |next, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.password_visible = next;
                                cx.notify();
                            });
                        },
                    ),
                )
                .child(ExposedDropdown::new(
                    "picker",
                    "Fruit",
                    &self.dropdown,
                    [
                        DropdownOption::new("a", "Alpha"),
                        DropdownOption::new("b", "Beta"),
                    ],
                ))
                .child(
                    div()
                        .relative()
                        .w(px(80.))
                        .h(px(32.))
                        .child(Scrim::new("standalone-scrim").on_click(|_, _| {})),
                )
                .child(
                    Sheet::new("side-sheet", &self.sheet, "Details", div().child("Body"))
                        .edge(SheetEdge::End)
                        .mode(SheetMode::Standard)
                        .on_closed(|_, _, _| {}),
                )
                .child(Dialog::new(
                    "confirm-dialog",
                    &self.dialog,
                    "Confirm action",
                ))
        }
    }

    let (handle, view) = cx.update(|cx| {
        gpui_material::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
        assert_eq!(material_strings(cx), &MaterialStrings::default());
        let mut entity = None;
        let handle = MaterialRoot::open_window(test_window_options(), cx, |window, cx| {
            let v = cx.new(|cx| ComponentsView::new(window, cx));
            entity = Some(v.clone());
            v
        })
        .unwrap();
        (handle, entity.unwrap())
    });

    // Baseline English defaults before any override.
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window.find(id_pair("notice", "dismiss")).label(),
            Some("Dismiss")
        );
        assert_eq!(
            window.find(id_pair("search", "clear-button")).label(),
            Some("Clear")
        );
        assert_eq!(
            window.find(id_pair("secret", "visibility-toggle")).label(),
            Some("Show password")
        );
        assert_eq!(
            window.find(id_pair("picker", "toggle-options")).label(),
            Some("Open options")
        );
        assert_eq!(
            window.find("standalone-scrim").label(),
            Some("Dismiss overlay")
        );
        assert_eq!(
            window.within("side-sheet").find("close").label(),
            Some("Close")
        );
        assert_eq!(
            window.within("confirm-dialog").find("cancel").label(),
            Some("Cancel")
        );
    })
    .unwrap();

    // Install localized global strings and verify all built-in defaults update.
    let zh = MaterialStrings::default()
        .dismiss("关闭横幅")
        .dismiss_overlay("关闭遮罩")
        .close("关闭面板")
        .cancel("取消")
        .clear("清除输入")
        .show_password("显示密码")
        .hide_password("隐藏密码")
        .open_options("展开选项")
        .close_options("收起选项");

    cx.update(|cx| {
        set_material_strings(zh.clone(), cx);
        assert_eq!(material_strings(cx), &zh);
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window.find(id_pair("notice", "dismiss")).label(),
            Some("关闭横幅")
        );
        assert_eq!(
            window.find(id_pair("search", "clear-button")).label(),
            Some("清除输入")
        );
        assert_eq!(
            window.find(id_pair("secret", "visibility-toggle")).label(),
            Some("显示密码")
        );
        assert_eq!(
            window.find(id_pair("picker", "toggle-options")).label(),
            Some("展开选项")
        );
        assert_eq!(window.find("standalone-scrim").label(), Some("关闭遮罩"));
        assert_eq!(
            window.within("side-sheet").find("close").label(),
            Some("关闭面板")
        );
        assert_eq!(
            window.within("confirm-dialog").find("cancel").label(),
            Some("取消")
        );

        // Close modal dialog, toggle password visibility, and click dropdown toggle to verify
        // hide_password & close_options.
        view.update(cx, |this, cx| {
            this.password_visible = true;
            this.dialog
                .update(cx, |dialog, cx| dialog.close(window, cx));
            cx.notify();
        });
        window.render_frame(cx);
        window.click(id_pair("picker", "toggle-options"), cx);
        window.render_frame(cx);
        assert_eq!(
            window.find(id_pair("secret", "visibility-toggle")).label(),
            Some("隐藏密码")
        );
        assert_eq!(
            window.find(id_pair("picker", "toggle-options")).label(),
            Some("收起选项")
        );
    })
    .unwrap();
}

#[gpui::test]
fn theme_scope_strings_override_subtree_and_deferred_overlays_without_mutating_global(
    cx: &mut TestAppContext,
) {
    struct ScopedView {
        scoped_field: Entity<TextFieldState>,
        scoped_dialog: Entity<DialogState>,
        bound_action_observed: Rc<RefCell<Option<SharedString>>>,
        bound_action: Rc<RefCell<Option<Box<dyn Fn(&mut Window, &mut gpui::App)>>>>,
    }

    impl Render for ScopedView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let fr = MaterialStrings::default()
                .dismiss("Fermer")
                .clear("Effacer")
                .cancel("Annuler")
                .dismiss_overlay("Fermer le voile");

            let observed = self.bound_action_observed.clone();
            let slot = self.bound_action.clone();

            #[derive(gpui::IntoElement)]
            struct BindProbe {
                observed: Rc<RefCell<Option<SharedString>>>,
                slot: Rc<RefCell<Option<Box<dyn Fn(&mut Window, &mut gpui::App)>>>>,
            }

            impl gpui::RenderOnce for BindProbe {
                fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
                    let observed = self.observed;
                    let action = ThemeScope::bind_action(
                        move |window, cx| {
                            let label = SharedString::from(
                                current_material_strings(window, cx).cancel.clone(),
                            );
                            *observed.borrow_mut() = Some(label);
                        },
                        window,
                        cx,
                    );
                    *self.slot.borrow_mut() = Some(Box::new(action));
                    div()
                }
            }

            div()
                .size_full()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .child(Banner::new("global-banner", "Global").on_dismiss(|_, _| {}))
                .child(
                    ThemeScope::new(
                        // Nest a density-only ThemeScope to verify strings survive nested theme scopes.
                        ThemeScope::new(
                            div()
                                .id("scoped-subtree")
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(Banner::new("scoped-banner", "Local").on_dismiss(|_, _| {}))
                                .child(
                                    TextField::new("scoped-field", "Nom", &self.scoped_field)
                                        .clearable(true),
                                )
                                .child(
                                    Dialog::new(
                                        "scoped-dialog",
                                        &self.scoped_dialog,
                                        "Confirmation",
                                    )
                                    .body(
                                        div().relative().w(px(64.)).h(px(24.)).child(
                                            Scrim::new("deferred-scrim").on_click(|_, _| {}),
                                        ),
                                    ),
                                )
                                .child(BindProbe { observed, slot }),
                        )
                        .density(Density::Compact),
                    )
                    .strings(fr),
                )
        }
    }

    let bound_action_observed = Rc::new(RefCell::new(None));
    let bound_action = Rc::new(RefCell::new(None));

    let handle = cx.update(|cx| {
        gpui_material::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
        let observed = bound_action_observed.clone();
        let slot = bound_action.clone();
        MaterialRoot::open_window(test_window_options(), cx, |window, cx| {
            let scoped_field =
                cx.new(|cx| TextFieldState::new(window, cx).default_value("bonjour"));
            let scoped_dialog = cx.new(|cx| {
                let mut state = DialogState::new(cx);
                state.open(window, cx);
                state
            });
            cx.new(|_| ScopedView {
                scoped_field,
                scoped_dialog,
                bound_action_observed: observed,
                bound_action: slot,
            })
        })
        .unwrap()
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);

        // Global strings remain untouched outside the ThemeScope.
        assert_eq!(material_strings(cx), &MaterialStrings::default());
        assert_eq!(
            window.find(id_pair("global-banner", "dismiss")).label(),
            Some("Dismiss")
        );

        // Scoped controls and deferred Dialog overlay (including Scrim via ThemeScope::inherit) use French strings.
        assert_eq!(
            window.find(id_pair("scoped-banner", "dismiss")).label(),
            Some("Fermer")
        );
        assert_eq!(
            window.find(id_pair("scoped-field", "clear-button")).label(),
            Some("Effacer")
        );
        assert_eq!(
            window.within("scoped-dialog").find("cancel").label(),
            Some("Annuler")
        );
        assert_eq!(
            window.find("deferred-scrim").label(),
            Some("Fermer le voile")
        );

        // ThemeScope::bind_action preserves the scoped MaterialStrings when invoked later.
        let action = bound_action.borrow_mut().take().expect("bound action");
        action(window, cx);
        assert_eq!(bound_action_observed.borrow().as_deref(), Some("Annuler"));
    })
    .unwrap();
}

#[gpui::test]
fn explicit_component_label_setters_override_material_strings(cx: &mut TestAppContext) {
    struct ExplicitLabelsView {
        clearable: Entity<TextFieldState>,
        password: Entity<TextFieldState>,
        dialog: Entity<DialogState>,
        sheet: Entity<DialogState>,
    }

    impl Render for ExplicitLabelsView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let scoped = MaterialStrings::default()
                .dismiss("Scoped dismiss")
                .clear("Scoped clear")
                .show_password("Scoped show")
                .hide_password("Scoped hide")
                .cancel("Scoped cancel")
                .close("Scoped close")
                .dismiss_overlay("Scoped scrim");

            ThemeScope::new(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .child(
                        Banner::new("explicit-banner", "Info")
                            .on_dismiss(|_, _| {})
                            .dismiss_label("Custom dismiss"),
                    )
                    .child(
                        TextField::new("explicit-clear", "Query", &self.clearable)
                            .clearable(true)
                            .clear_label("Custom clear"),
                    )
                    .child(
                        TextField::secure("explicit-pass", "Secret", &self.password)
                            .visibility_toggle(false, |_, _, _| {})
                            .visibility_labels("Reveal key", "Conceal key"),
                    )
                    .child(
                        div().relative().w(px(80.)).h(px(32.)).child(
                            Scrim::new("explicit-scrim")
                                .accessibility_label("Custom scrim")
                                .on_click(|_, _| {}),
                        ),
                    )
                    .child(
                        Sheet::new("explicit-sheet", &self.sheet, "Panel", div().child("Body"))
                            .edge(SheetEdge::End)
                            .mode(SheetMode::Standard)
                            .close_label("Custom close")
                            .on_closed(|_, _, _| {}),
                    )
                    .child(
                        Dialog::new("explicit-dialog", &self.dialog, "Title")
                            .cancel_label("Custom cancel"),
                    ),
            )
            .strings(scoped)
        }
    }

    let handle = cx.update(|cx| {
        gpui_material::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
        set_material_strings(
            MaterialStrings::default()
                .dismiss("Global dismiss")
                .clear("Global clear")
                .show_password("Global show")
                .hide_password("Global hide")
                .cancel("Global cancel")
                .close("Global close")
                .dismiss_overlay("Global scrim"),
            cx,
        );
        MaterialRoot::open_window(test_window_options(), cx, |window, cx| {
            let clearable = cx.new(|cx| TextFieldState::new(window, cx).default_value("abc"));
            let password = cx.new(|cx| {
                TextFieldState::new(window, cx)
                    .masked(true)
                    .default_value("xyz")
            });
            let dialog = cx.new(|cx| {
                let mut state = DialogState::new(cx);
                state.open(window, cx);
                state
            });
            let sheet = cx.new(|cx| {
                let mut state = DialogState::new(cx);
                state.open(window, cx);
                state
            });
            cx.new(|_| ExplicitLabelsView {
                clearable,
                password,
                dialog,
                sheet,
            })
        })
        .unwrap()
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            window.find(id_pair("explicit-banner", "dismiss")).label(),
            Some("Custom dismiss")
        );
        assert_eq!(
            window
                .find(id_pair("explicit-clear", "clear-button"))
                .label(),
            Some("Custom clear")
        );
        assert_eq!(
            window
                .find(id_pair("explicit-pass", "visibility-toggle"))
                .label(),
            Some("Reveal key")
        );
        assert_eq!(window.find("explicit-scrim").label(), Some("Custom scrim"));
        assert_eq!(
            window.within("explicit-sheet").find("close").label(),
            Some("Custom close")
        );
        assert_eq!(
            window.within("explicit-dialog").find("cancel").label(),
            Some("Custom cancel")
        );
    })
    .unwrap();
}
