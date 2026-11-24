use gpui::{
    AppContext, Context, ElementId, InputEvent, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, MouseUpEvent, ParentElement, Render, Role, Styled, TestAppContext, Window, div,
    px, rgb, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    Banner, BannerColors, Button, Icon, IconName, SemanticTone,
    foundation::{MaterialRoot, color, set_window_material_theme},
    theme::{Appearance, ColorRole, Direction, MaterialTheme},
};

fn click(window: &mut Window, cx: &mut gpui::App, pos: gpui::Point<gpui::Pixels>) {
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

#[gpui::test]
fn banner_supports_tones_overrides_actions_dismiss_roles_and_rtl(cx: &mut TestAppContext) {
    const CUSTOM_BG: u32 = 0x224466;
    const CUSTOM_BORDER: u32 = 0x88aa44;

    struct Example {
        visible: bool,
        primary_clicks: usize,
        secondary_clicks: usize,
        custom_clicks: usize,
        dismiss_clicks: usize,
    }

    impl Render for Example {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let v1 = cx.entity();
            let v2 = cx.entity();
            let v3 = cx.entity();
            let v4 = cx.entity();
            div()
                .id("banner-test-root")
                .w(px(640.))
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .children(self.visible.then(|| {
                    Banner::new("danger-banner", "Connection lost while saving.")
                        .title("Sync error")
                        .tone(SemanticTone::Danger)
                        .icon(Icon::new(IconName::Error))
                        .action("Retry", move |_, cx| {
                            v1.update(cx, |this, cx| {
                                this.primary_clicks += 1;
                                cx.notify();
                            });
                        })
                        .secondary_action("Ignore", move |_, cx| {
                            v2.update(cx, |this, cx| {
                                this.secondary_clicks += 1;
                                cx.notify();
                            });
                        })
                        .actions_content(Button::outlined("custom-banner-btn", "Logs").on_activate(
                            move |_, cx| {
                                v3.update(cx, |this, cx| {
                                    this.custom_clicks += 1;
                                    cx.notify();
                                });
                            },
                        ))
                        .dismiss_label("Close error banner")
                        .on_dismiss(move |_, cx| {
                            v4.update(cx, |this, cx| {
                                this.dismiss_clicks += 1;
                                this.visible = false;
                                cx.notify();
                            });
                        })
                }))
                .child(
                    Banner::new("status-override", "Danger toned status")
                        .tone(SemanticTone::Danger)
                        .alert(false),
                )
                .child(
                    Banner::new("custom-color-banner", "Custom overrides win over tone")
                        .tone(SemanticTone::Success)
                        .alert(true)
                        .colors(BannerColors {
                            container: Some(rgb(CUSTOM_BG).into()),
                            border: Some(rgb(CUSTOM_BORDER).into()),
                            ..Default::default()
                        }),
                )
                .child(
                    Banner::new("default-banner", "Default neutral banner")
                        .icon(Icon::new(IconName::Info)),
                )
        }
    }

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut entity = None;
        let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let created = cx.new(|_| Example {
                visible: true,
                primary_clicks: 0,
                secondary_clicks: 0,
                custom_clicks: 0,
                dismiss_clicks: 0,
            });
            entity = Some(created.clone());
            created
        })
        .unwrap();
        (handle, entity.unwrap())
    });
    cx.simulate_window_resize(handle.into(), size(px(720.), px(720.)));

    for direction in [Direction::Ltr, Direction::Rtl] {
        cx.update_window(handle.into(), |_, window, cx| {
            let mut theme = MaterialTheme::baseline(Appearance::Light);
            theme.layout.direction = direction;
            let error_bg = color(theme.colors.color(ColorRole::ErrorContainer));
            set_window_material_theme(Some(theme), window, cx);
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);

            // Role checks: Danger defaults to Alert, overrides work both ways, Default is Status.
            let danger = find(window, &[], &"danger-banner".into()).unwrap();
            assert_eq!(danger.role(), Some(Role::Alert));
            assert_eq!(
                danger.label(),
                Some("Sync error. Connection lost while saving.")
            );

            let status_override = find(window, &[], &"status-override".into()).unwrap();
            assert_eq!(status_override.role(), Some(Role::Status));

            let custom = find(window, &[], &"custom-color-banner".into()).unwrap();
            assert_eq!(custom.role(), Some(Role::Alert));

            let default_banner = find(window, &[], &"default-banner".into()).unwrap();
            assert_eq!(default_banner.role(), Some(Role::Status));

            // Tone palette painted for danger banner, and custom override wins over Success tone.
            let quads = window.painted_quads();
            assert!(
                quads
                    .iter()
                    .any(|q| q.background.as_solid() == Some(error_bg)),
                "Danger banner should paint ErrorContainer background"
            );
            assert!(
                quads
                    .iter()
                    .any(|q| q.background.as_solid() == Some(rgb(CUSTOM_BG).into())),
                "Explicit BannerColors::container must override SemanticTone"
            );
            assert!(
                quads
                    .iter()
                    .any(|q| q.border_color == rgb(CUSTOM_BORDER).into()),
                "Explicit BannerColors::border must override SemanticTone"
            );

            // RTL mirroring check on icon vs title vs dismiss button.
            let icon = find(
                window,
                &[],
                &ElementId::from((ElementId::from("danger-banner"), "icon")),
            )
            .unwrap();
            let title = find(
                window,
                &[],
                &ElementId::from((ElementId::from("danger-banner"), "title")),
            )
            .unwrap();
            let dismiss = find(
                window,
                &[],
                &ElementId::from((ElementId::from("danger-banner"), "dismiss")),
            )
            .unwrap();
            assert_eq!(dismiss.label(), Some("Close error banner"));

            if direction == Direction::Ltr {
                assert!(icon.bounds().left() < title.bounds().left());
                assert!(title.bounds().right() <= dismiss.bounds().left() + px(1.));
            } else {
                assert!(icon.bounds().left() > title.bounds().left());
                assert!(dismiss.bounds().right() <= title.bounds().left() + px(1.));
            }
        })
        .unwrap();
    }

    // Reset to LTR and test actions + dismiss hiding caller-owned banner.
    cx.update_window(handle.into(), |_, window, cx| {
        let theme = MaterialTheme::baseline(Appearance::Light);
        set_window_material_theme(Some(theme), window, cx);
        window.refresh();
        window.draw(cx).clear(cx);

        let primary = find(
            window,
            &[],
            &ElementId::from((ElementId::from("danger-banner"), "action")),
        )
        .unwrap()
        .bounds()
        .center();
        let secondary = find(
            window,
            &[],
            &ElementId::from((ElementId::from("danger-banner"), "secondary-action")),
        )
        .unwrap()
        .bounds()
        .center();
        let custom_btn = find(window, &[], &"custom-banner-btn".into())
            .unwrap()
            .bounds()
            .center();
        let dismiss = find(
            window,
            &[],
            &ElementId::from((ElementId::from("danger-banner"), "dismiss")),
        )
        .unwrap()
        .bounds()
        .center();

        click(window, cx, primary);
        click(window, cx, secondary);
        click(window, cx, custom_btn);
        click(window, cx, dismiss);
    })
    .unwrap();
    cx.run_until_parked();

    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let state = view.read(cx);
        assert_eq!(state.primary_clicks, 1);
        assert_eq!(state.secondary_clicks, 1);
        assert_eq!(state.custom_clicks, 1);
        assert_eq!(state.dismiss_clicks, 1);
        assert!(!state.visible);
        assert!(find(window, &[], &"danger-banner".into()).is_none());
    })
    .unwrap();
}
