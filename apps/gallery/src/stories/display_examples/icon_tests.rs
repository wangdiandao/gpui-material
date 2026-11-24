use gpui::{
    AppContext, Context, IntoElement, ParentElement, Render, Role, Styled, TestAppContext, Window,
    div, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{Icon, IconName, foundation::MaterialRoot};

#[gpui::test]
fn symbols_inherit_slot_size_and_keep_image_semantics(cx: &mut TestAppContext) {
    struct Example;
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .children(
                    [16., 24., 40.]
                        .into_iter()
                        .enumerate()
                        .map(|(index, side)| {
                            div().text_size(px(side)).child(
                                Icon::new(IconName::Favorite)
                                    .filled(index == 1)
                                    .label(("symbol", index), "Favorite"),
                            )
                        }),
                )
                .child(
                    div().text_size(px(40.)).child(
                        Icon::new(IconName::ArrowForward)
                            .size(px(20.))
                            .mirror_in_rtl(true)
                            .label("explicit-symbol", "Forward"),
                    ),
                )
        }
    }
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| Example)
        })
        .unwrap()
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        for (index, side) in [16., 24., 40.].into_iter().enumerate() {
            let icon = find(window, &[], &("symbol", index).into()).unwrap();
            assert_eq!(icon.bounds().size, size(px(side), px(side)));
            assert_eq!(icon.role(), Some(Role::Image));
        }
        let explicit = find(window, &[], &"explicit-symbol".into()).unwrap();
        assert_eq!(explicit.bounds().size, size(px(20.), px(20.)));
        assert_eq!(explicit.role(), Some(Role::Image));
    })
    .unwrap();
}

#[gpui::test]
fn icon_supports_color_override_and_inherit_color(cx: &mut TestAppContext) {
    use gpui::rgb;
    use gpui_material::theme::ColorRole;

    struct Example;
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .text_color(rgb(0x4a148c))
                .child(
                    Icon::new(IconName::Favorite)
                        .size(px(24.))
                        .color(ColorRole::Primary)
                        .color_override(rgb(0x2e7d32))
                        .label("override-icon", "Override icon"),
                )
                .child(
                    Icon::new(IconName::Check)
                        .size(px(20.))
                        .color_override(rgb(0x2e7d32))
                        .inherit_color()
                        .label("inherit-icon", "Inherited icon"),
                )
        }
    }

    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| Example)
        })
        .unwrap()
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let override_icon = find(window, &[], &"override-icon".into()).unwrap();
        let inherit_icon = find(window, &[], &"inherit-icon".into()).unwrap();
        assert_eq!(override_icon.role(), Some(Role::Image));
        assert_eq!(override_icon.label(), Some("Override icon"));
        assert_eq!(override_icon.bounds().size, size(px(24.), px(24.)));
        assert_eq!(inherit_icon.role(), Some(Role::Image));
        assert_eq!(inherit_icon.label(), Some("Inherited icon"));
        assert_eq!(inherit_icon.bounds().size, size(px(20.), px(20.)));
    })
    .unwrap();
}
