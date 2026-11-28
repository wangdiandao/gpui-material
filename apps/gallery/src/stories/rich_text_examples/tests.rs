use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext, Entity, ParentElement, Role, Styled, TestAppContext, WindowOptions, div, point, px,
    size,
};
use gpui_base::TextSelection;
use gpui_material::{
    MaterialRoot, RichText, RichTextState, material_theme, set_material_theme, test::TestWindowExt,
    theme::Direction,
};

use super::RichTextExamples;
use crate::story::Story;

fn open_story(
    story: Story,
    cx: &mut TestAppContext,
) -> (
    gpui::WindowHandle<gpui_base::Root>,
    Entity<RichTextExamples>,
) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let entity = cx.new(|cx| RichTextExamples::new(story, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(800.), px(1000.)));
    cx.update_window(window.into(), |_, window, cx| window.render_frame(cx))
        .unwrap();
    (window, view)
}

#[gpui::test]
fn selectable_text_supports_drag_selection_copy_and_shared_groups(cx: &mut TestAppContext) {
    let (window, _view) = open_story(Story::SelectableText, cx);

    cx.update_window(window.into(), |_, window, cx| {
        let heading = window.find("sel-heading");
        assert_eq!(heading.role(), Some(Role::Label));
        assert_eq!(heading.value(), Some("Selectable Material headline"));
        let bounds = heading.bounds();

        // Drag across the selectable heading and verify window text selection + Ctrl/Cmd+C copy.
        let start = point(bounds.left() + px(2.), bounds.center().y);
        let end = point(bounds.right() - px(2.), bounds.center().y);
        window.drag(start, end, cx);
        window.render_frame(cx);

        let selected = TextSelection::selected_text(window, cx);
        assert!(
            selected.contains("Selectable"),
            "expected selected text from heading, got {selected:?}"
        );

        window.press(
            if cfg!(target_os = "macos") {
                "cmd-c"
            } else {
                "ctrl-c"
            },
            cx,
        );
        let copied = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .unwrap_or_default();
        assert!(
            copied.contains("Selectable"),
            "expected clipboard to contain selected text, got {copied:?}"
        );

        // Multi-paragraph drag selection across SelectableTextGroup.
        let p1 = window.find("sel-group-1").bounds();
        let p2 = window.find("sel-group-2").bounds();
        window.drag(
            point(p1.left() + px(2.), p1.center().y),
            point(p2.right() - px(2.), p2.center().y),
            cx,
        );
        window.render_frame(cx);
        let selected = TextSelection::selected_text(window, cx);
        assert!(
            selected.contains("First paragraph") && selected.contains("Second paragraph"),
            "expected cross-block selection, got {selected:?}"
        );

        // Switching to RTL redraws cleanly.
        let mut theme = material_theme(cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_material_theme(theme, cx);
        window.render_frame(cx);
    })
    .unwrap();
}

#[gpui::test]
fn rich_text_renders_markdown_streams_updates_and_supports_theme_scope(cx: &mut TestAppContext) {
    let (window, view) = open_story(Story::RichText, cx);

    cx.update_window(window.into(), |_, window, cx| {
        let doc = window.find("rich-markdown");
        assert_eq!(doc.role(), Some(Role::Document));
        assert!(doc.bounds().size.height > px(100.));
        let scoped = window.find("rich-dark-scope");
        assert_eq!(scoped.role(), Some(Role::Document));
        assert!(scoped.bounds().size.height > px(20.));
        let rendered = view.read(cx).rich_state.read(cx).rendered_text(cx);
        assert!(rendered.contains("Material 3 Expressive"));
        assert!(rendered.contains("SurfaceContainerHighest"));

        // Clicking "Append note" streams incremental Markdown content into RichTextState.
        window.click("rich-append", cx);
    })
    .unwrap();

    cx.run_until_parked();
    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        let rendered = view.read(cx).rich_state.read(cx).rendered_text(cx);
        assert!(
            rendered.contains("Streamed update #1"),
            "expected streamed update in rendered text, got: {rendered}"
        );
    })
    .unwrap();
}

#[gpui::test]
fn rich_text_max_lines_clamps_long_markdown(cx: &mut TestAppContext) {
    let clamped_state: Rc<RefCell<Option<Entity<RichTextState>>>> = Rc::new(RefCell::new(None));
    let state_slot = clamped_state.clone();
    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let state = cx.new(|cx| {
                RichTextState::markdown(
                    "Line one paragraph.\n\nLine two paragraph.\n\nLine three paragraph.\n\nLine four paragraph.",
                    cx,
                )
            });
            *state_slot.borrow_mut() = Some(state.clone());
            struct ClampedView(Entity<RichTextState>);
            impl gpui::Render for ClampedView {
                fn render(
                    &mut self,
                    _: &mut gpui::Window,
                    _: &mut gpui::Context<Self>,
                ) -> impl gpui::IntoElement {
                    div().w(px(300.)).child(
                        RichText::from_state("clamped-doc", &self.0)
                            .max_lines(2)
                            .stream_fade(true),
                    )
                }
            }
            cx.new(|_| ClampedView(state))
        })
        .unwrap()
    });
    cx.update_window(window.into(), |_, window, cx| {
        window.render_frame(cx);
        let state = clamped_state.borrow().clone().unwrap();
        assert!(state.read(cx).is_clamped(cx));
    })
    .unwrap();
}
