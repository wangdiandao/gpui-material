use gpui::{
    AppContext, Context, ElementId, Entity, InputEvent, IntoElement, MouseButton, MouseDownEvent,
    MouseUpEvent, ParentElement, Render, Role, Styled, TestAppContext, Window, div, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    AppBarScrollBehavior, AppBarScrollState, BottomAppBar, Fab, Icon, IconButton, IconName,
    Scaffold,
    foundation::{MaterialRoot, set_window_material_theme},
    theme::{Appearance, Direction, MaterialTheme},
};

#[gpui::test]
fn bottom_app_bar_renders_actions_fab_rtl_and_scroll_hide_in_scaffold(cx: &mut TestAppContext) {
    struct Example {
        scroll_state: Entity<AppBarScrollState>,
        action_clicks: usize,
        fab_clicks: usize,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let on_action = cx.entity();
            let on_fab = cx.entity();
            let bar = BottomAppBar::new("test-bottom-bar")
                .accessible_label("Editor bottom bar")
                .action(
                    IconButton::new("bar-search", "Search", Icon::new(IconName::Search))
                        .on_activate(move |_, cx| {
                            on_action.update(cx, |this, cx| {
                                this.action_clicks += 1;
                                cx.notify();
                            });
                        }),
                )
                .fab(
                    Fab::new("bar-fab", "Add item", Icon::new(IconName::Add)).on_activate(
                        move |_, cx| {
                            on_fab.update(cx, |this, cx| {
                                this.fab_clicks += 1;
                                cx.notify();
                            });
                        },
                    ),
                )
                .scroll_behavior(AppBarScrollBehavior::EnterAlways, &self.scroll_state);

            div().size_full().child(
                Scaffold::new("test-scaffold", div().size_full().child("Body")).bottom_bar(bar),
            )
        }
    }

    let mut entity = None;
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let scroll_state = cx.new(|_| AppBarScrollState::new());
            let e = cx.new(|_| Example {
                scroll_state,
                action_clicks: 0,
                fab_clicks: 0,
            });
            entity = Some(e.clone());
            e
        })
        .unwrap()
    });
    let view = entity.unwrap();
    cx.simulate_window_resize(handle.into(), size(px(520.), px(420.)));

    let draw = |window: &mut Window, cx: &mut gpui::App| {
        window.refresh();
        window.draw(cx).clear(cx);
    };

    // 1. LTR: actions on left, FAB on right, height 80px, Role::Toolbar.
    cx.update_window(handle.into(), |_, window, cx| {
        let mut theme = MaterialTheme::baseline(Appearance::Light);
        theme.layout.direction = Direction::Ltr;
        set_window_material_theme(Some(theme), window, cx);
        window.activate_window();
        draw(window, cx);

        let bar = find(window, &[], &"test-bottom-bar".into()).unwrap();
        assert_eq!(bar.role(), Some(Role::Toolbar));
        assert_eq!(bar.label(), Some("Editor bottom bar"));
        assert!((f32::from(bar.bounds().size.height) - 80.).abs() < 1.);

        let actions_id: ElementId = (ElementId::from("test-bottom-bar"), "actions").into();
        let fab_slot_id: ElementId = (ElementId::from("test-bottom-bar"), "fab-slot").into();
        let actions = find(window, &[], &actions_id).unwrap();
        let fab_slot = find(window, &[], &fab_slot_id).unwrap();
        assert!(actions.bounds().left() < fab_slot.bounds().left());

        let search_center = find(window, &[], &"bar-search".into())
            .unwrap()
            .bounds()
            .center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: search_center,
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
                position: search_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert_eq!(view.read(cx).action_clicks, 1);

        let fab_center = find(window, &[], &"bar-fab".into())
            .unwrap()
            .bounds()
            .center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: fab_center,
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
                position: fab_center,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
        draw(window, cx);
        assert_eq!(view.read(cx).fab_clicks, 1);
    })
    .unwrap();

    // 2. RTL: FAB mirrors to left, actions mirror to right.
    cx.update_window(handle.into(), |_, window, cx| {
        let mut theme = MaterialTheme::baseline(Appearance::Light);
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        draw(window, cx);

        let actions_id: ElementId = (ElementId::from("test-bottom-bar"), "actions").into();
        let fab_slot_id: ElementId = (ElementId::from("test-bottom-bar"), "fab-slot").into();
        let actions = find(window, &[], &actions_id).unwrap();
        let fab_slot = find(window, &[], &fab_slot_id).unwrap();
        assert!(fab_slot.bounds().left() < actions.bounds().left());

        // 3. Hide on scroll collapses bar height to 0.
        let scroll_state = view.read(cx).scroll_state.clone();
        scroll_state.update(cx, |state, cx| state.set_content_offset(px(120.), cx));
        draw(window, cx);
        let collapsed = find(window, &[], &"test-bottom-bar".into()).unwrap();
        assert_eq!(collapsed.bounds().size.height, px(0.));

        // Scrolling back reveals bar.
        scroll_state.update(cx, |state, cx| state.set_content_offset(px(0.), cx));
        draw(window, cx);
        let restored = find(window, &[], &"test-bottom-bar".into()).unwrap();
        assert!((f32::from(restored.bounds().size.height) - 80.).abs() < 1.);
    })
    .unwrap();
}
