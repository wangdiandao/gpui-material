use std::time::Duration;

use gpui::{
    AppContext, Context, ElementId, Entity, FocusHandle, IntoElement, ParentElement, Render,
    SharedString, Styled, Subscription, Window, WindowOptions, div,
};
use gpui_material::{
    Button, CommandGroup, CommandItem, CommandPalette, CommandPaletteEvent, CommandPaletteState,
    IconName, MaterialRoot, MaterialStrings, MaterialTheme, ReduceMotionPreference, ThemeScope,
    set_reduce_motion_preference, set_window_material_theme, test::TestWindowExt, theme::Direction,
};

struct PaletteHarness {
    palette: Entity<CommandPaletteState>,
    trigger_focus: FocusHandle,
    executed: Vec<SharedString>,
    events: Vec<SharedString>,
    use_custom_filter: bool,
    custom_strings: Option<MaterialStrings>,
    _sub: Subscription,
}

impl PaletteHarness {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let palette = cx.new(|cx| CommandPaletteState::new(window, cx));
        let sub = cx.subscribe(&palette, |this, _, event, _| {
            if let CommandPaletteEvent::Execute(item) = event {
                this.events.push(item.id().clone());
            }
        });
        Self {
            palette,
            trigger_focus: cx.focus_handle(),
            executed: Vec::new(),
            events: Vec::new(),
            use_custom_filter: false,
            custom_strings: None,
            _sub: sub,
        }
    }
}

impl Render for PaletteHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open_palette = self.palette.clone();
        let entity = cx.entity();
        let groups = vec![
            CommandGroup::new(
                "Navigation",
                [
                    CommandItem::new("open-inbox", "Open Inbox")
                        .supporting("Jump to unread messages")
                        .icon(IconName::Inbox)
                        .shortcut("G I")
                        .keywords(["mail", "messages"]),
                    CommandItem::new("locked", "Locked Command")
                        .icon(IconName::Archive)
                        .disabled(true),
                    CommandItem::new("open-docs", "Open Documentation")
                        .supporting("Browse component guide")
                        .icon(IconName::Description)
                        .shortcut("Ctrl+D")
                        .keywords(["guide", "reference"]),
                ],
            ),
            CommandGroup::new(
                "Theme",
                [CommandItem::new("dark-mode", "Switch to Dark Mode")
                    .icon(IconName::DarkMode)
                    .shortcut("Ctrl+Shift+L")
                    .keywords(["night"])],
            ),
        ];

        let mut palette =
            CommandPalette::grouped("cp", &self.palette, groups).on_select(move |item, _, cx| {
                let id = item.id().clone();
                entity.update(cx, |this, cx| {
                    this.executed.push(id);
                    cx.notify();
                });
            });
        if self.use_custom_filter {
            palette = palette
                .filter(|item, query| query.is_empty() || item.id().as_ref() == query.trim());
        }

        let content = div()
            .p_4()
            .child(
                Button::new("open-btn", "Open palette")
                    .track_focus(&self.trigger_focus)
                    .on_activate(move |window, cx| {
                        open_palette.update(cx, |state, cx| state.open(window, cx));
                    }),
            )
            .child(palette);

        if let Some(strings) = &self.custom_strings {
            ThemeScope::new(content)
                .strings(strings.clone())
                .into_any_element()
        } else {
            content.into_any_element()
        }
    }
}

#[gpui::test]
fn command_palette_open_filter_keyboard_nav_execute_and_focus_restoration(
    cx: &mut gpui::TestAppContext,
) {
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        set_reduce_motion_preference(ReduceMotionPreference::Reduce, cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| PaletteHarness::new(window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);

        // Focus trigger and click to open palette.
        let trigger_focus = view.read(cx).trigger_focus.clone();
        trigger_focus.focus(window, cx);
        window.render_frame(cx);
        window.click("open-btn", cx);

        let palette_state = view.read(cx).palette.clone();
        assert!(palette_state.read(cx).is_open(cx));
        assert!(palette_state.read(cx).input_focus(cx).is_focused(window));

        // First enabled item ("open-inbox") is active by default.
        let id_inbox = ElementId::from((ElementId::from("cp"), SharedString::from("open-inbox")));
        let id_locked = ElementId::from((ElementId::from("cp"), SharedString::from("locked")));
        let id_docs = ElementId::from((ElementId::from("cp"), SharedString::from("open-docs")));
        let id_dark = ElementId::from((ElementId::from("cp"), SharedString::from("dark-mode")));

        assert_eq!(window.find(id_inbox.clone()).selected(), Some(true));
        assert_eq!(window.find(id_locked).disabled(), Some(true));

        // Down skips disabled "locked" and highlights "open-docs".
        window.press("down", cx);
        assert_eq!(window.find(id_docs.clone()).selected(), Some(true));

        // End jumps to "dark-mode", Home returns to "open-inbox".
        window.press("end", cx);
        assert_eq!(window.find(id_dark.clone()).selected(), Some(true));
        window.press("home", cx);
        assert_eq!(window.find(id_inbox).selected(), Some(true));

        // Up wraps to "dark-mode".
        window.press("up", cx);
        assert_eq!(window.find(id_dark.clone()).selected(), Some(true));

        // Type keyword "night" -> filters down to "dark-mode" only, Enter executes & restores focus.
        window.input("night", cx);
        assert!(window.try_find(id_docs).is_none());
        assert_eq!(window.find(id_dark).selected(), Some(true));

        window.press("enter", cx);
        assert!(!palette_state.read(cx).is_open(cx));
        assert_eq!(
            view.read(cx).executed,
            vec![SharedString::from("dark-mode")]
        );
        assert!(
            view.read(cx).trigger_focus.is_focused(window),
            "closing command palette must restore focus to prior trigger"
        );
    })
    .unwrap();

    cx.update(|cx| {
        assert_eq!(view.read(cx).events, vec![SharedString::from("dark-mode")]);
    });
}

#[gpui::test]
fn command_palette_escape_empty_state_strings_and_custom_filter(cx: &mut gpui::TestAppContext) {
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        set_reduce_motion_preference(ReduceMotionPreference::Reduce, cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| PaletteHarness::new(window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        view.update(cx, |this, cx| {
            this.custom_strings = Some(MaterialStrings::default().no_results("Nothing matched"));
            cx.notify();
        });
        window.render_frame(cx);

        let trigger_focus = view.read(cx).trigger_focus.clone();
        trigger_focus.focus(window, cx);
        window.render_frame(cx);
        let palette_state = view.read(cx).palette.clone();
        palette_state.update(cx, |state, cx| state.open(window, cx));
        window.render_frame(cx);

        // Non-matching query displays scoped MaterialStrings::no_results.
        window.input("zzzz-unknown", cx);
        let empty = window.find(ElementId::from((ElementId::from("cp"), "empty")));
        assert_eq!(empty.label(), Some("Nothing matched"));

        // Pressing Escape dismisses and restores trigger focus.
        window.press("escape", cx);
        assert!(!palette_state.read(cx).is_open(cx));
        assert!(view.read(cx).trigger_focus.is_focused(window));

        // Custom filter matches exact ID only.
        view.update(cx, |this, cx| {
            this.use_custom_filter = true;
            cx.notify();
        });
        palette_state.update(cx, |state, cx| state.open(window, cx));
        window.render_frame(cx);
        // Re-opening cleared the previous query by default.
        assert_eq!(palette_state.read(cx).query().as_ref(), "");

        window.input("open-docs", cx);
        assert!(
            window
                .try_find(ElementId::from((
                    ElementId::from("cp"),
                    SharedString::from("open-docs")
                )))
                .is_some()
        );
        assert!(
            window
                .try_find(ElementId::from((
                    ElementId::from("cp"),
                    SharedString::from("open-inbox")
                )))
                .is_none()
        );
    })
    .unwrap();
}

#[gpui::test]
fn command_palette_pointer_click_disabled_and_rtl(cx: &mut gpui::TestAppContext) {
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        set_reduce_motion_preference(ReduceMotionPreference::Reduce, cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            set_window_material_theme(
                Some(MaterialTheme::default().direction(Direction::Rtl)),
                window,
                cx,
            );
            let entity = cx.new(|cx| PaletteHarness::new(window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);

        let palette_state = view.read(cx).palette.clone();
        palette_state.update(cx, |state, cx| state.open(window, cx));
        window.render_frame(cx);

        // In RTL, close button is placed at logical end (left of input).
        let input_x = window
            .find(ElementId::from((ElementId::from("cp"), "input")))
            .bounds()
            .origin
            .x;
        let close_x = window
            .find(ElementId::from((ElementId::from("cp"), "close")))
            .bounds()
            .origin
            .x;
        assert!(
            close_x < input_x,
            "in RTL close button ({close_x:?}) must appear left of input ({input_x:?})"
        );

        // Clicking disabled command keeps palette open and does not execute.
        window.click(
            ElementId::from((ElementId::from("cp"), SharedString::from("locked"))),
            cx,
        );
        assert!(palette_state.read(cx).is_open(cx));
        assert!(view.read(cx).executed.is_empty());

        // Clicking enabled command executes and closes palette.
        window.click(
            ElementId::from((ElementId::from("cp"), SharedString::from("open-docs"))),
            cx,
        );
        assert!(!palette_state.read(cx).is_open(cx));
        assert_eq!(
            view.read(cx).executed,
            vec![SharedString::from("open-docs")]
        );
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(16));
}
