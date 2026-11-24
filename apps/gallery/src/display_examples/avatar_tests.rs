use gpui::{
    AppContext, Context, Hsla, InteractiveElement, IntoElement, ParentElement, Render, Role,
    Styled, TestAppContext, Window, div, px, rgb, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    Avatar, AvatarColors, AvatarGroup, AvatarShape, AvatarSize, Button, Icon, IconName,
    foundation::{MaterialRoot, color, set_window_material_theme},
    theme::{Appearance, ColorRole, CornerShape, Direction, MaterialTheme},
};

#[gpui::test]
fn avatar_renders_sizes_shapes_colors_and_accessible_semantics(cx: &mut TestAppContext) {
    const CUSTOM_BG: u32 = 0x264653;
    const CUSTOM_FG: u32 = 0xe9c46a;
    const CUSTOM_BORDER: u32 = 0xe76f51;

    struct Example;

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .id("avatars")
                .tab_group()
                .flex()
                .items_center()
                .gap_3()
                .p_4()
                .child(Button::new("before-avatar", "Before").on_activate(|_, _| {}))
                .child(
                    Avatar::initials("Ada Lovelace")
                        .size(AvatarSize::Medium)
                        .accessible_label("ada-avatar", "Ada Lovelace"),
                )
                .child(
                    Avatar::icon(Icon::new(IconName::Favorite))
                        .size(AvatarSize::Large)
                        .shape(AvatarShape::Rounded(CornerShape::Radii([14., 4., 10., 2.])))
                        .colors(AvatarColors {
                            container: Some(rgb(CUSTOM_BG).into()),
                            content: Some(rgb(CUSTOM_FG).into()),
                            border: Some(rgb(CUSTOM_BORDER).into()),
                        })
                        .border(px(2.)),
                )
                .child(
                    Avatar::image("missing-avatar.png")
                        .size(AvatarSize::Small)
                        .fallback(|| "FB"),
                )
                .child(Button::new("after-avatar", "After").on_activate(|_, _| {}))
        }
    }

    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| Example)
        })
        .unwrap()
    });
    cx.simulate_window_scale_factor_change(handle.into(), 1.);

    for direction in [Direction::Ltr, Direction::Rtl] {
        cx.update_window(handle.into(), |_, window, cx| {
            let mut theme = MaterialTheme::baseline(Appearance::Light);
            theme.layout.direction = direction;
            let default_bg = color(theme.colors.color(ColorRole::PrimaryContainer));
            set_window_material_theme(Some(theme), window, cx);
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);

            let quads = crate::painted_quads(window);
            let find_quad = |fill: Hsla| {
                quads
                    .iter()
                    .find(|quad| quad.background == fill.into())
                    .unwrap_or_else(|| panic!("Missing avatar fill {fill:?}"))
            };

            let default_avatar = find_quad(default_bg);
            assert_eq!(
                default_avatar.bounds.size,
                size(px(40.).scale(1.), px(40.).scale(1.))
            );

            let custom_avatar = find_quad(rgb(CUSTOM_BG).into());
            assert_eq!(
                custom_avatar.bounds.size,
                size(px(48.).scale(1.), px(48.).scale(1.))
            );
            if direction == Direction::Ltr {
                assert_eq!(custom_avatar.corner_radii.top_left, px(14.).scale(1.));
                assert_eq!(custom_avatar.corner_radii.top_right, px(4.).scale(1.));
            } else {
                assert_eq!(custom_avatar.corner_radii.top_left, px(4.).scale(1.));
                assert_eq!(custom_avatar.corner_radii.top_right, px(14.).scale(1.));
            }

            assert!(
                quads
                    .iter()
                    .any(|quad| quad.border_color == rgb(CUSTOM_BORDER).into())
            );

            // Accessible avatar exposes Role::Image + label without taking keyboard Tab stops.
            let named = find(window, &[], &"ada-avatar".into()).unwrap();
            assert_eq!(named.role(), Some(Role::Image));
            assert_eq!(named.label(), Some("Ada Lovelace"));

            window.focus_next(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(
                find(window, &[], &"before-avatar".into())
                    .unwrap()
                    .focused(),
                Some(true)
            );
            window.focus_next(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(
                find(window, &[], &"after-avatar".into()).unwrap().focused(),
                Some(true)
            );
        })
        .unwrap();
    }
}

#[gpui::test]
fn avatar_group_renders_overflow_rings_overlap_rtl_and_accessible_group(cx: &mut TestAppContext) {
    const FIRST_BG: u32 = 0x1d3557;
    const SECOND_BG: u32 = 0x457b9d;
    const RING_COLOR: u32 = 0xf1faee;
    const OVERFLOW_BG: u32 = 0xe63946;
    const OVERFLOW_FG: u32 = 0xffffff;

    struct Example;

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .id("avatar-group-root")
                .tab_group()
                .flex()
                .flex_col()
                .gap_4()
                .p_4()
                .child(Button::new("before-group", "Before").on_activate(|_, _| {}))
                .child(
                    AvatarGroup::new(
                        "team-stack",
                        [
                            Avatar::initials("Ada Lovelace")
                                .size(AvatarSize::ExtraLarge)
                                .colors(AvatarColors {
                                    container: Some(rgb(FIRST_BG).into()),
                                    ..Default::default()
                                }),
                            Avatar::initials("Grace Hopper").colors(AvatarColors {
                                container: Some(rgb(SECOND_BG).into()),
                                ..Default::default()
                            }),
                            Avatar::initials("Alan Turing"),
                            Avatar::initials("Katherine Johnson"),
                            Avatar::initials("Margaret Hamilton"),
                        ],
                    )
                    .size(AvatarSize::Small)
                    .max_visible(2)
                    .overlap(px(8.))
                    .ring_width(px(3.))
                    .ring_color(rgb(RING_COLOR))
                    .accessible_label("5 team members"),
                )
                .child(
                    AvatarGroup::new(
                        "custom-overflow-stack",
                        [
                            Avatar::initials("One"),
                            Avatar::initials("Two"),
                            Avatar::initials("Three"),
                        ],
                    )
                    .size(AvatarSize::Medium)
                    .max_visible(1)
                    .ring_width(px(0.))
                    .overflow_colors(AvatarColors {
                        container: Some(rgb(OVERFLOW_BG).into()),
                        content: Some(rgb(OVERFLOW_FG).into()),
                        ..Default::default()
                    })
                    .overflow_content(|count| {
                        div()
                            .id("custom-overflow-badge")
                            .child(format!("{count} more"))
                    }),
                )
                .child(Button::new("after-group", "After").on_activate(|_, _| {}))
        }
    }

    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| Example)
        })
        .unwrap()
    });
    cx.simulate_window_scale_factor_change(handle.into(), 1.);

    for direction in [Direction::Ltr, Direction::Rtl] {
        cx.update_window(handle.into(), |_, window, cx| {
            let mut theme = MaterialTheme::baseline(Appearance::Light);
            theme.layout.direction = direction;
            let overflow_default_bg = color(theme.colors.color(ColorRole::SurfaceContainerHighest));
            set_window_material_theme(Some(theme), window, cx);
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);

            let quads = crate::painted_quads(window);
            let find_quad = |fill: Hsla| {
                quads
                    .iter()
                    .find(|quad| quad.background == fill.into())
                    .unwrap_or_else(|| panic!("Missing avatar group fill {fill:?}"))
            };

            // Uniform Small size (32 px) overrides member ExtraLarge size and applies to +3 overflow chip.
            let first = find_quad(rgb(FIRST_BG).into());
            let second = find_quad(rgb(SECOND_BG).into());
            let overflow = find_quad(overflow_default_bg);
            assert_eq!(
                first.bounds.size,
                size(px(32.).scale(1.), px(32.).scale(1.))
            );
            assert_eq!(
                second.bounds.size,
                size(px(32.).scale(1.), px(32.).scale(1.))
            );
            assert_eq!(
                overflow.bounds.size,
                size(px(32.).scale(1.), px(32.).scale(1.))
            );

            // Cutout rings have outer size = 32 + 2*3 = 38 px with border color == RING_COLOR.
            let ring_quads: Vec<_> = quads
                .iter()
                .filter(|quad| {
                    quad.border_color == rgb(RING_COLOR).into()
                        && quad.bounds.size == size(px(38.).scale(1.), px(38.).scale(1.))
                })
                .collect();
            assert_eq!(ring_quads.len(), 3);

            // Step between adjacent items = outer_side (38) - overlap (8) = 30 px, mirrored in RTL.
            let expected_step = px(30.).scale(1.);
            if direction == Direction::Ltr {
                assert_eq!(
                    second.bounds.origin.x - first.bounds.origin.x,
                    expected_step
                );
                assert_eq!(
                    overflow.bounds.origin.x - second.bounds.origin.x,
                    expected_step
                );
            } else {
                assert_eq!(
                    first.bounds.origin.x - second.bounds.origin.x,
                    expected_step
                );
                assert_eq!(
                    second.bounds.origin.x - overflow.bounds.origin.x,
                    expected_step
                );
            }

            // Custom overflow colors and content render at uniform Medium (40 px) size.
            let custom_overflow = find_quad(rgb(OVERFLOW_BG).into());
            assert_eq!(
                custom_overflow.bounds.size,
                size(px(40.).scale(1.), px(40.).scale(1.))
            );

            // Summary Role::Group label without adding keyboard focus stops.
            let group = find(window, &[], &"team-stack".into()).unwrap();
            assert_eq!(group.role(), Some(Role::Group));
            assert_eq!(group.label(), Some("5 team members"));

            window.blur(cx);
            window.focus_next(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(
                find(window, &[], &"before-group".into()).unwrap().focused(),
                Some(true)
            );
            window.focus_next(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(
                find(window, &[], &"after-group".into()).unwrap().focused(),
                Some(true)
            );
        })
        .unwrap();
    }
}
