use gpui::{
    App, AppContext, Context, Hsla, InteractiveElement, IntoElement, ParentElement, Render,
    RenderOnce, Role, StatefulInteractiveElement, Styled, TestAppContext, Window, div, px, rgb,
    size,
};
use gpui_base::test_support::find;
use gpui_material::{
    Badge, Button, Icon, IconName,
    foundation::{MaterialRoot, color, set_window_material_theme},
    theme::{Appearance, ColorRole, Direction, MaterialTheme},
};

#[gpui::test]
fn badge_content_color_geometry_and_host_focus(cx: &mut TestAppContext) {
    #[derive(IntoElement)]
    struct Mark;

    impl RenderOnce for Mark {
        fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
            div()
                .id("visual-mark")
                .role(Role::Label)
                .aria_label("Decorative detail")
                .w(px(12.))
                .h(px(8.))
                .bg(window.text_style().color)
        }
    }

    struct Example;

    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let icon = || Icon::new(IconName::Inbox).size(px(24.));
            div()
                .id("badges")
                .tab_group()
                .flex()
                .flex_col()
                .items_start()
                .gap_3()
                .p_4()
                .child(Badge::standalone())
                .child(
                    Badge::standalone()
                        .content("status-marker", Mark)
                        .container_color(rgb(0x325578))
                        .content_color(rgb(0xcdf4ee))
                        .accessible_label("status-badge", "Priority 1"),
                )
                .child(
                    Badge::dot(icon().label("dot-host", "New inbox activity"))
                        .container_color(rgb(0x683437)),
                )
                .child(
                    Badge::dot(Button::new("badge-host", "Inbox, 8 unread").on_activate(|_, _| {}))
                        .content("inbox-marker", Mark)
                        .container_color(rgb(0x215c45))
                        .content_color(rgb(0xfaf5e8)),
                )
                .child(Badge::count(icon(), 1250, 99).container_color(rgb(0x643880)))
                .child(Badge::label(icon(), "99+").container_color(rgb(0x806428)))
                .child(Button::new("after-badge", "Archive").on_activate(|_, _| {}))
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
            let default = color(theme.colors.color(ColorRole::Error));
            set_window_material_theme(Some(theme), window, cx);
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);

            let quads = window.painted_quads();
            let bounds = |fill: Hsla| {
                quads
                    .iter()
                    .find(|quad| quad.background == fill.into())
                    .unwrap_or_else(|| panic!("Missing badge color {fill:?}"))
                    .bounds
            };
            let dot = bounds(default);
            assert_eq!(dot.size, size(px(6.).scale(1.), px(6.).scale(1.)));
            let standalone = bounds(rgb(0x325578).into());
            assert_eq!(standalone.size, size(px(20.).scale(1.), px(16.).scale(1.)));
            assert_eq!(standalone.center(), bounds(rgb(0xcdf4ee).into()).center());
            assert_eq!(
                bounds(rgb(0x643880).into()).size,
                bounds(rgb(0x806428).into()).size,
            );

            for (id, fill, overlap, offset) in [
                ("dot-host", 0x683437, 6., 0.),
                ("badge-host", 0x215c45, 12., -2.),
            ] {
                let host = find(window, &[], &id.into()).unwrap().bounds();
                let mark = bounds(rgb(fill).into());
                assert_eq!(mark.top(), (host.top() + px(offset)).scale(1.));
                if direction == Direction::Ltr {
                    assert_eq!(mark.left(), (host.right() - px(overlap)).scale(1.));
                } else {
                    assert_eq!(mark.right(), (host.left() + px(overlap)).scale(1.));
                }
            }
            assert_eq!(
                bounds(rgb(0x215c45).into()).center(),
                bounds(rgb(0xfaf5e8).into()).center(),
            );
            assert_eq!(
                find(window, &[], &"badge-host".into()).unwrap().label(),
                Some("Inbox, 8 unread"),
            );
            window.focus_next(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(
                find(window, &[], &"badge-host".into()).unwrap().focused(),
                Some(true),
            );
            window.focus_next(cx);
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(
                find(window, &[], &"after-badge".into()).unwrap().focused(),
                Some(true),
            );
        })
        .unwrap();
    }
}
