use gpui::{
    AppContext, Context, ElementId, InputEvent, InteractiveElement, IntoElement, KeyDownEvent,
    KeyUpEvent, Keystroke, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Pixels,
    Render, Role, SharedString, Styled, TestAppContext, Window, div, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    BreadcrumbItem, Breadcrumbs, Icon, IconName,
    foundation::{MaterialRoot, set_window_material_theme},
    theme::{Appearance, Direction, MaterialTheme},
};

fn click(window: &mut Window, cx: &mut gpui::App, pos: gpui::Point<Pixels>) {
    window.dispatch_event(
        MouseDownEvent {
            button: MouseButton::Left,
            position: pos,
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
            position: pos,
            click_count: 1,
            modifiers: Default::default(),
        }
        .to_platform_input(),
        cx,
    );
}

fn press_key(window: &mut Window, cx: &mut gpui::App, name: &str) {
    let keystroke = Keystroke::parse_with_os(name, cx.platform_info().os).unwrap();
    window.dispatch_event(
        KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        }
        .to_platform_input(),
        cx,
    );
    window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
}

fn seg_id(root: &'static str, segment: &'static str) -> ElementId {
    ElementId::from((ElementId::from(root), SharedString::from(segment)))
}

#[gpui::test]
fn breadcrumbs_handles_navigation_rtl_max_items_and_width_overflow_menu(cx: &mut TestAppContext) {
    struct Example {
        container_width: Pixels,
        selected: Vec<SharedString>,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let v1 = cx.entity();
            let v2 = cx.entity();
            div()
                .id("breadcrumbs-root")
                .w(px(720.))
                .flex()
                .flex_col()
                .gap_4()
                .p_4()
                .child(
                    div().w(self.container_width).child(
                        Breadcrumbs::new(
                            "responsive-path",
                            [
                                BreadcrumbItem::new("home", "Workspace")
                                    .icon(|| Icon::new(IconName::Inbox)),
                                BreadcrumbItem::new("org", "Engineering"),
                                BreadcrumbItem::new("team", "Desktop UI").disabled(true),
                                BreadcrumbItem::new("crate", "Components"),
                                BreadcrumbItem::new("file", "breadcrumbs.rs"),
                            ],
                        )
                        .accessible_label("Document path")
                        .on_select(move |id, _, cx| {
                            v1.update(cx, |this, cx| {
                                this.selected.push(id);
                                cx.notify();
                            });
                        }),
                    ),
                )
                .child(
                    Breadcrumbs::new(
                        "capped-path",
                        [
                            BreadcrumbItem::new("a", "Root"),
                            BreadcrumbItem::new("b", "Folder A"),
                            BreadcrumbItem::new("c", "Folder B"),
                            BreadcrumbItem::new("d", "Active File"),
                        ],
                    )
                    .max_items(2)
                    .separator(|| "/")
                    .on_select(move |id, _, cx| {
                        v2.update(cx, |this, cx| {
                            this.selected.push(id);
                            cx.notify();
                        });
                    }),
                )
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let view = cx.new(|_| Example {
            container_width: px(680.),
            selected: Vec::new(),
        });
        let handle =
            MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, _| view.clone())
                .unwrap();
        (handle.into(), view)
    });
    cx.simulate_window_resize(handle, size(px(760.), px(480.)));

    // Wide layout: all 5 segments visible, no overflow button on responsive-path,
    // while capped-path (max_items = 2) collapses middle segments.
    cx.update_window(handle, |_, window, cx| {
        set_window_material_theme(Some(MaterialTheme::baseline(Appearance::Light)), window, cx);
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);

        let nav = find(window, &[], &"responsive-path".into()).unwrap();
        assert_eq!(nav.role(), Some(Role::Navigation));
        assert_eq!(nav.label(), Some("Document path"));

        let home = find(window, &[], &seg_id("responsive-path", "home")).unwrap();
        let org = find(window, &[], &seg_id("responsive-path", "org")).unwrap();
        let team = find(window, &[], &seg_id("responsive-path", "team")).unwrap();
        let current = find(window, &[], &seg_id("responsive-path", "file")).unwrap();

        assert_eq!(home.role(), Some(Role::Button));
        assert_eq!(team.disabled(), Some(true));
        assert_eq!(current.role(), Some(Role::Label));
        assert_eq!(current.label(), Some("breadcrumbs.rs, current page"));
        assert!(home.bounds().left() < org.bounds().left());
        assert!(org.bounds().left() < current.bounds().left());

        assert!(
            find(
                window,
                &[],
                &ElementId::from((ElementId::from("responsive-path"), "overflow"))
            )
            .is_none(),
            "Wide Breadcrumbs must not show overflow trigger"
        );
        assert!(
            find(
                window,
                &[],
                &ElementId::from((ElementId::from("capped-path"), "overflow"))
            )
            .is_some(),
            "Explicit max_items(2) must collapse middle segments"
        );

        // Clicking an enabled ancestor fires on_select; disabled ancestor and current item do not.
        click(window, cx, org.bounds().center());
        click(window, cx, team.bounds().center());
        click(window, cx, current.bounds().center());
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, _, cx| {
        assert_eq!(view.read(cx).selected, vec![SharedString::from("org")]);
    })
    .unwrap();

    // RTL mirroring places the first item to the physical right of the current item.
    cx.update_window(handle, |_, window, cx| {
        let mut rtl = MaterialTheme::baseline(Appearance::Light);
        rtl.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(rtl), window, cx);
        window.refresh();
        window.draw(cx).clear(cx);

        let home = find(window, &[], &seg_id("responsive-path", "home")).unwrap();
        let current = find(window, &[], &seg_id("responsive-path", "file")).unwrap();
        assert!(
            home.bounds().left() > current.bounds().left(),
            "RTL Breadcrumbs must mirror segment order"
        );
    })
    .unwrap();

    // Shrink container width to force width-responsive collapse into the overflow Menu,
    // open the Menu, and select a collapsed middle segment ("org")!
    cx.update_window(handle, |_, window, cx| {
        set_window_material_theme(Some(MaterialTheme::baseline(Appearance::Light)), window, cx);
        view.update(cx, |this, cx| {
            this.container_width = px(260.);
            this.selected.clear();
            cx.notify();
        });
        window.refresh();
        window.draw(cx).clear(cx);

        assert!(
            find(window, &[], &seg_id("responsive-path", "home")).is_some(),
            "First segment must remain visible when collapsed"
        );
        assert!(
            find(window, &[], &seg_id("responsive-path", "file")).is_some(),
            "Current segment must remain visible when collapsed"
        );
        assert!(
            find(window, &[], &seg_id("responsive-path", "org")).is_none(),
            "Middle segment must collapse out of inline bar when width is constrained"
        );

        let overflow = find(
            window,
            &[],
            &ElementId::from((ElementId::from("responsive-path"), "overflow")),
        )
        .expect("Width-constrained Breadcrumbs must render overflow menu trigger");
        click(window, cx, overflow.bounds().center());
    })
    .unwrap();
    cx.run_until_parked();

    // Overflow menu is now open: find and click the collapsed "org" menu item!
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);

        let collapsed_org = find(window, &[], &seg_id("responsive-path", "org"))
            .expect("Collapsed middle segment must appear inside open overflow Menu");
        click(window, cx, collapsed_org.bounds().center());
    })
    .unwrap();
    cx.run_until_parked();

    // Also test keyboard activation on an ancestor ("home").
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            view.read(cx).selected,
            vec![SharedString::from("org")],
            "Selecting a collapsed segment from the overflow Menu must invoke on_select"
        );

        let home = find(window, &[], &seg_id("responsive-path", "home")).unwrap();
        click(window, cx, home.bounds().center());
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        press_key(window, cx, "enter");
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, _, cx| {
        assert_eq!(
            view.read(cx).selected,
            vec![
                SharedString::from("org"),
                SharedString::from("home"),
                SharedString::from("home"),
            ]
        );
    })
    .unwrap();
}
