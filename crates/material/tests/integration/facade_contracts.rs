//! Public-facade contracts for composition (`TestInput`) and `ThemeScope` isolation.

use gpui::{
    AppContext, Context, ElementInputHandler, Entity, Hsla, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, TestAppContext, Window, WindowBounds, WindowOptions, div, px,
    size,
};
use gpui_material::{
    Button, MaterialRoot, MaterialTheme, TextField, TextFieldState, ThemeScope,
    foundation::color,
    material_theme,
    test::{TestInput, TestWindowExt as _, painted_quads},
    theme::{Appearance, ColorRole, Direction},
};

#[gpui::test]
fn test_input_bridges_utf16_composition_on_text_field_state(cx: &mut TestAppContext) {
    struct EditorHost {
        field: Entity<TextFieldState>,
    }

    impl Render for EditorHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .p_4()
                .child(TextField::new("editor", "Notes", &self.field))
        }
    }

    let (handle, host) = cx.update(|cx| {
        gpui_material::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
        let mut entity = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let field = cx.new(|cx| TextFieldState::new(window, cx));
            let view = cx.new(|_| EditorHost { field });
            entity = Some(view.clone());
            view
        })
        .unwrap();
        (handle, entity.unwrap())
    });

    let (bounds, field) = cx
        .update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            window.render_frame(cx);
            let field = host.read(cx).field.clone();
            field.update(cx, |state, cx| state.focus(window, cx));
            window.render_frame(cx);
            (window.find("editor").bounds(), field)
        })
        .unwrap();

    let mut input = TestInput::new(ElementInputHandler::new(bounds, field.clone()), handle);
    input.commit_text("🙂", cx);
    // Surrogate pair "🙂" is 2 UTF-16 code units.
    input.compose_text(None, "zhong", Some(5..5), cx);
    assert_eq!(input.marked_text_range(cx), Some(2..7));
    assert_eq!(
        input.selected_text_range(cx).map(|sel| sel.range),
        Some(7..7)
    );

    input.commit_text("中", cx);
    assert_eq!(input.marked_text_range(cx), None);
    cx.update_window(handle.into(), |_, window, _| {
        assert_eq!(window.find("editor").value(), Some("🙂中"));
    })
    .unwrap();

    input.compose_text(None, "wen", Some(3..3), cx);
    assert_eq!(input.marked_text_range(cx), Some(3..6));
    input.unmark_text(cx);
    assert_eq!(input.marked_text_range(cx), None);
    cx.update_window(handle.into(), |_, window, _| {
        assert_eq!(window.find("editor").value(), Some("🙂中wen"));
    })
    .unwrap();
}

#[gpui::test]
fn theme_scope_isolates_dark_rtl_subtree_without_mutating_global_theme(cx: &mut TestAppContext) {
    struct ScopedView {
        field: Entity<TextFieldState>,
    }

    impl Render for ScopedView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            use gpui_material::test::TestSupportExt as _;

            div()
                .size_full()
                .flex()
                .flex_col()
                .gap_4()
                .p_4()
                .child(Button::new("global-btn", "Global"))
                .child(
                    ThemeScope::new(
                        div()
                            .id("rtl-scope")
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(Button::new("scoped-btn", "Scoped"))
                            .child(
                                TextField::new("scoped-field", "Budget", &self.field)
                                    .prefix(div().id("prefix").test_support().child("$"))
                                    .suffix(div().id("suffix").test_support().child("USD")),
                            ),
                    )
                    .theme(MaterialTheme::baseline(Appearance::Dark))
                    .direction(Direction::Rtl),
                )
        }
    }

    let handle = cx.update(|cx| {
        gpui_material::init(MaterialTheme::baseline(Appearance::Light), cx);
        cx.set_reduce_motion(true);
        let bounds = gpui::Bounds {
            origin: gpui::Point::default(),
            size: size(px(480.), px(280.)),
        };
        MaterialRoot::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            cx,
            |window, cx| {
                let field = cx.new(|cx| TextFieldState::new(window, cx).default_value("42"));
                cx.new(|_| ScopedView { field })
            },
        )
        .unwrap()
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        let mut rtl = window.within("rtl-scope");
        let scoped_field = rtl.within("scoped-field");
        let prefix = scoped_field.find("prefix");
        let suffix = scoped_field.find("suffix");
        // Under RTL, the suffix sits to the left of the prefix.
        assert!(suffix.bounds().right() < prefix.bounds().left());

        let light_primary: Hsla = color(
            MaterialTheme::baseline(Appearance::Light)
                .colors
                .color(ColorRole::Primary),
        );
        let dark_primary: Hsla = color(
            MaterialTheme::baseline(Appearance::Dark)
                .colors
                .color(ColorRole::Primary),
        );
        let quads = painted_quads(window);
        assert!(quads.iter().any(|q| q.background == light_primary.into()));
        assert!(quads.iter().any(|q| q.background == dark_primary.into()));
        assert_eq!(material_theme(cx).colors.appearance(), Appearance::Light);
    })
    .unwrap();
}
