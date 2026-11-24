use gpui::{
    AppContext, Context, ElementId, InputEvent, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, MouseUpEvent, ParentElement, Render, Role, Styled, TestAppContext, Window, div,
    px, rgb, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    Button, ButtonVariant, EmptyView, Icon, IconName,
    foundation::{MaterialRoot, color, set_window_material_theme},
    theme::{Appearance, ColorRole, CornerRole, Direction, MaterialShape, MaterialTheme, TypeRole},
};

#[gpui::test]
fn empty_state_renders_slots_shapes_rtl_actions_and_focus_order(cx: &mut TestAppContext) {
    const CUSTOM_VISUAL_BG: u32 = 0x2a9d8f;

    struct Example {
        primary_clicks: usize,
        secondary_clicks: usize,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let primary_view = cx.entity();
            let secondary_view = cx.entity();
            div()
                .id("empty-state-root")
                .tab_group()
                .w(px(640.))
                .flex()
                .flex_col()
                .gap_4()
                .p_4()
                .child(Button::new("before-empty", "Before").on_activate(|_, _| {}))
                .child(
                    EmptyView::new("projects-empty", "No projects yet")
                        .icon(Icon::new(IconName::Inbox))
                        .supporting("Create a project to start tracking work.")
                        .max_width(px(320.))
                        .primary_action(Button::new("empty-primary", "Create project").on_activate(
                            move |_, cx| {
                                primary_view.update(cx, |this, cx| {
                                    this.primary_clicks += 1;
                                    cx.notify();
                                });
                            },
                        ))
                        .secondary_action(
                            Button::new("empty-secondary", "Learn more")
                                .variant(ButtonVariant::Outlined)
                                .on_activate(move |_, cx| {
                                    secondary_view.update(cx, |this, cx| {
                                        this.secondary_clicks += 1;
                                        cx.notify();
                                    });
                                }),
                        ),
                )
                .child(
                    EmptyView::new("rounded-empty", "No results")
                        .icon(Icon::new(IconName::Search))
                        .shape(CornerRole::ExtraLarge.shape())
                        .icon_container_color_role(
                            ColorRole::TertiaryContainer,
                            ColorRole::OnTertiaryContainer,
                        )
                        .title_role(TypeRole::HeadlineSmall)
                        .supporting_content(
                            div().id("custom-supporting").child("Try another query"),
                        ),
                )
                .child(
                    EmptyView::new("custom-visual-empty", "Nothing pinned")
                        .icon_container_shape(MaterialShape::SoftBurst)
                        .visual(
                            div()
                                .id("custom-visual-box")
                                .size(px(48.))
                                .bg(rgb(CUSTOM_VISUAL_BG)),
                        ),
                )
                .child(Button::new("after-empty", "After").on_activate(|_, _| {}))
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut entity = None;
        let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let created = cx.new(|_| Example {
                primary_clicks: 0,
                secondary_clicks: 0,
            });
            entity = Some(created.clone());
            created
        })
        .unwrap();
        (handle, entity.unwrap())
    });
    cx.simulate_window_resize(handle.into(), size(px(720.), px(900.)));
    cx.simulate_window_scale_factor_change(handle.into(), 1.);

    for direction in [Direction::Ltr, Direction::Rtl] {
        cx.update_window(handle.into(), |_, window, cx| {
            let mut theme = MaterialTheme::baseline(Appearance::Light);
            theme.layout.direction = direction;
            let tertiary_bg = color(theme.colors.color(ColorRole::TertiaryContainer));
            set_window_material_theme(Some(theme), window, cx);
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);

            // Title and supporting labels keep normal semantics.
            let title = find(
                window,
                &[],
                &ElementId::from((ElementId::from("projects-empty"), "title")),
            )
            .unwrap();
            assert_eq!(title.role(), Some(Role::Label));
            assert_eq!(title.label(), Some("No projects yet"));

            let supporting = find(
                window,
                &[],
                &ElementId::from((ElementId::from("projects-empty"), "supporting")),
            )
            .unwrap();
            assert_eq!(supporting.role(), Some(Role::Label));
            assert_eq!(
                supporting.label(),
                Some("Create a project to start tracking work.")
            );

            // Max width constrains content column to 320 px.
            let content = find(
                window,
                &[],
                &ElementId::from((ElementId::from("projects-empty"), "content")),
            )
            .unwrap();
            assert!(content.bounds().size.width <= px(320.5));

            // Decorative icon container renders at 72x72 and does not expose a focusable role.
            let icon_container = find(
                window,
                &[],
                &ElementId::from((ElementId::from("projects-empty"), "icon-container")),
            )
            .unwrap();
            assert_eq!(icon_container.bounds().size, size(px(72.), px(72.)));
            assert_eq!(icon_container.focused(), None);

            // Rounded CornerShape container and custom visual both render as expected.
            let quads = window.painted_quads();
            assert!(
                quads
                    .iter()
                    .any(|quad| quad.background == tertiary_bg.into())
            );
            assert!(
                quads
                    .iter()
                    .any(|quad| quad.background == rgb(CUSTOM_VISUAL_BG).into())
            );

            // Action row mirrors primary & secondary order in RTL.
            let primary = find(window, &[], &"empty-primary".into()).unwrap();
            let secondary = find(window, &[], &"empty-secondary".into()).unwrap();
            if direction == Direction::Ltr {
                assert!(primary.bounds().origin.x < secondary.bounds().origin.x);
            } else {
                assert!(primary.bounds().origin.x > secondary.bounds().origin.x);
            }

            // Tab order visits only interactive controls — icon containers add zero focus stops.
            window.blur(cx);
            for expected in [
                "before-empty",
                "empty-primary",
                "empty-secondary",
                "after-empty",
            ] {
                window.focus_next(cx);
                window.refresh();
                window.draw(cx).clear(cx);
                assert_eq!(
                    find(window, &[], &expected.into()).unwrap().focused(),
                    Some(true),
                    "expected {expected} to receive focus in {direction:?}"
                );
            }
        })
        .unwrap();
    }

    // Clicking primary and secondary actions invokes caller handlers.
    let (primary_center, secondary_center) = cx
        .update_window(handle.into(), |_, window, _| {
            (
                find(window, &[], &"empty-primary".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(window, &[], &"empty-secondary".into())
                    .unwrap()
                    .bounds()
                    .center(),
            )
        })
        .unwrap();

    for center in [primary_center, secondary_center] {
        cx.update_window(handle.into(), |_, window, cx| {
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position: center,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    position: center,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    }
    cx.run_until_parked();
    cx.update(|cx| {
        let state = view.read(cx);
        assert_eq!(state.primary_clicks, 1);
        assert_eq!(state.secondary_clicks, 1);
    });
}
