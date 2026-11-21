use gpui::{
    AppContext, Context, ElementId, Entity, InputEvent, IntoElement, KeyDownEvent, KeyUpEvent,
    Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Render, Role,
    SharedString, Styled, TestAppContext, Window, div, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    Badge, Button, DialogCloseReason, DialogState, Icon, IconName, NavigationDrawer,
    NavigationDrawerMode, NavigationItem, NavigationItems,
    foundation::{MaterialRoot, set_window_material_theme},
    theme::{Appearance, Direction, MaterialTheme},
};

#[gpui::test]
fn navigation_drawer_supports_standard_roving_keyboard_rtl_and_modal_dismissal(
    cx: &mut TestAppContext,
) {
    struct Example {
        selected: SharedString,
        modal: Entity<DialogState>,
        closed_reasons: Vec<DialogCloseReason>,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let on_open = self.modal.clone();
            let on_closed = cx.entity();
            let make_items = |id: &'static str| {
                NavigationItems::new(id, "Main navigation", [])
                    .selected(self.selected.clone())
                    .on_select(cx.listener(|this, id: &SharedString, _, cx| {
                        this.selected = id.clone();
                        cx.notify();
                    }))
            };
            div()
                .size_full()
                .flex()
                .child(
                    NavigationDrawer::new("std-drawer", make_items("std-nav"))
                        .headline("Workspace")
                        .section(
                            "Primary",
                            [
                                NavigationItem::new("inbox", "Inbox")
                                    .icon(Icon::new(IconName::Inbox))
                                    .badge(
                                        Badge::standalone()
                                            .content("cnt", "5")
                                            .accessible_label("unread", "5 unread"),
                                    ),
                                NavigationItem::new("starred", "Starred").disabled(true),
                                NavigationItem::new("archive", "Archive")
                                    .icon(Icon::new(IconName::Archive)),
                            ],
                        )
                        .divider()
                        .section("More", [NavigationItem::new("docs", "Documents")]),
                )
                .child(
                    div()
                        .p_4()
                        .child(Button::new("open-drawer", "Open drawer").on_activate(
                            move |window, cx| {
                                on_open.update(cx, |state, cx| state.open(window, cx));
                            },
                        )),
                )
                .child(
                    NavigationDrawer::new("modal-drawer", make_items("modal-nav"))
                        .mode(NavigationDrawerMode::Modal)
                        .headline("Modal workspace")
                        .modal_state(&self.modal)
                        .section(
                            "Primary",
                            [
                                NavigationItem::new("inbox", "Inbox"),
                                NavigationItem::new("archive", "Archive"),
                            ],
                        )
                        .on_closed(move |reason, _, cx| {
                            on_closed.update(cx, |this, cx| {
                                this.closed_reasons.push(reason);
                                cx.notify();
                            });
                        }),
                )
        }
    }

    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let modal = cx.new(DialogState::new);
            let e = cx.new(|_| Example {
                selected: "inbox".into(),
                modal,
                closed_reasons: Vec::new(),
            });
            entity = Some(e.clone());
            e
        })
        .unwrap()
    });
    let view = entity.unwrap();
    cx.simulate_window_resize(handle.into(), size(px(800.), px(520.)));

    let draw = |window: &mut Window, cx: &mut gpui::App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };

    // 1. Standard drawer geometry, role, badge description, roving keyboard skipping disabled item.
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        draw(window, cx);

        let drawer = find(window, &[], &"std-drawer".into()).unwrap();
        assert_eq!(drawer.role(), Some(Role::Navigation));
        assert_eq!(drawer.bounds().size.width, px(360.));

        let inbox_id: ElementId = (ElementId::from("std-nav"), SharedString::from("inbox")).into();
        let archive_id: ElementId =
            (ElementId::from("std-nav"), SharedString::from("archive")).into();
        let inbox = find(window, &[], &inbox_id).unwrap();
        assert_eq!(inbox.selected(), Some(true));
        assert_eq!(inbox.description(), Some("5 unread"));

        // Click inbox to focus it, then press Down (skips disabled "starred" -> focuses "archive").
        let center = inbox.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: center,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);

        let down = Keystroke::parse("down").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: down.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: down }.to_platform_input(), cx);
        draw(window, cx);
        assert_eq!(
            find(window, &[], &archive_id).unwrap().focused(),
            Some(true)
        );

        let enter = Keystroke::parse("enter").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: enter.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: enter }.to_platform_input(), cx);
        draw(window, cx);
        assert_eq!(view.read(cx).selected, "archive");
    })
    .unwrap();

    // 2. Modal drawer opens via DialogState, renders in RTL on right edge, closes on Escape, restores focus.
    cx.update_window(handle.into(), |_, window, cx| {
        let mut theme = MaterialTheme::baseline(Appearance::Light);
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        draw(window, cx);

        let open_btn = find(window, &[], &"open-drawer".into()).unwrap();
        let open_center = open_btn.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: open_center,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: open_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert!(view.read(cx).modal.read(cx).is_open());

        let modal_panel_id: ElementId = (ElementId::from("modal-drawer"), "drawer-panel").into();
        let modal_panel = find(window, &[], &modal_panel_id).unwrap();
        // In RTL, SheetEdge::Start docks on the right half of the 800px window.
        assert!(modal_panel.bounds().left() > px(300.));

        let esc = Keystroke::parse("escape").unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: esc.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: esc }.to_platform_input(), cx);
        draw(window, cx);

        assert!(!view.read(cx).modal.read(cx).is_open());
        assert_eq!(view.read(cx).closed_reasons, [DialogCloseReason::Escape]);
        assert_eq!(
            find(window, &[], &"open-drawer".into()).unwrap().focused(),
            Some(true)
        );
    })
    .unwrap();
}
