use gpui::{
    AppContext, Context, Hsla, InteractiveElement, IntoElement, ParentElement, Render, Role,
    Styled, TestAppContext, Window, div, px, rgb, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    Avatar, AvatarColors, AvatarShape, AvatarSize, Button, Icon, IconName,
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
                        .fallback("FB"),
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

            let quads = window.painted_quads();
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
