use super::*;
use gpui::{ElementId, InputEvent as _, Role, TestAppContext};
use gpui_base::{Theme, ThemeAppearance, test_support::find};
use gpui_material::foundation::{color, shape_text, window_material_theme};
use gpui_material::theme::{ColorRole, TypeRole, TypographyEmphasis};

fn click(cx: &mut TestAppContext, handle: WindowHandle<Root>, id: ElementId) {
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let target = find(window, &[], &id).unwrap();
        assert!(target.visible());
        let position = target.bounds().center();
        for input in [
            gpui::MouseDownEvent {
                button: gpui::MouseButton::Left,
                position,
                click_count: 1,
                first_mouse: false,
                modifiers: Default::default(),
            }
            .to_platform_input(),
            gpui::MouseUpEvent {
                button: gpui::MouseButton::Left,
                position,
                click_count: 1,
                modifiers: Default::default(),
            }
            .to_platform_input(),
        ] {
            window.dispatch_event(input, cx);
        }
    })
    .unwrap();
    cx.run_until_parked();
}

#[gpui::test]
fn standalone_story_opens_a_dialog_in_its_host_window(cx: &mut TestAppContext) {
    let story = Story::find("dialog").unwrap();
    let handle = cx.update(|cx| {
        init_gallery(cx);
        cx.set_reduce_motion(true);
        cx.open_window(WindowOptions::default(), |window, cx| {
            let view = story.new_view(window, cx).unwrap();
            cx.new(|cx| Root::new(view, window, cx))
        })
        .unwrap()
    });
    click(cx, handle, "rename".into());
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(
            find(window, &[], &"project-dialog".into())
                .unwrap()
                .visible()
        );
    })
    .unwrap();
}

#[gpui::test]
fn repository_button_opens_github_and_sits_before_motion_toggle(cx: &mut TestAppContext) {
    let handle = cx.update(|cx| {
        init_gallery(cx);
        open_gallery(cx, false, Story::Introduction, false).unwrap()
    });
    let repository = ElementId::from("open-repository");
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let button = find(window, &[], &repository).unwrap();
        let motion = find(window, &[], &"toggle-motion".into()).unwrap();
        assert_eq!(button.role(), Some(Role::Button));
        assert_eq!(button.label(), Some("Open GitHub repository"));
        assert!(button.bounds().right() <= motion.bounds().left());
    })
    .unwrap();
    click(cx, handle, repository);
    assert_eq!(
        cx.opened_url().as_deref(),
        Some("https://github.com/wangdiandao/gpui-material")
    );
}

#[gpui::test]
fn catalog_filters_and_opens_a_story_on_a_compact_window(cx: &mut TestAppContext) {
    let handle = cx.update(|cx| {
        init_gallery(cx);
        open_gallery(cx, false, Story::default(), false).unwrap()
    });
    cx.simulate_window_resize(handle.into(), size(px(340.), px(560.)));
    let select_all = if cfg!(target_os = "macos") {
        "cmd-a"
    } else {
        "ctrl-a"
    };

    let catalog_button = ElementId::from("browse-components");
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let button = find(window, &[], &catalog_button).unwrap();
        assert_eq!(button.role(), Some(Role::Button));
        assert_eq!(button.label(), Some("Show components"));
    })
    .unwrap();
    click(cx, handle, catalog_button.clone());
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let button = find(window, &[], &catalog_button).unwrap();
        assert_eq!(button.label(), Some("Hide components"));
    })
    .unwrap();
    cx.simulate_input(handle.into(), "slider");
    cx.run_until_parked();
    let story = ElementId::from((ElementId::from("gallery-story"), "slider"));
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let excluded = ElementId::from((ElementId::from("gallery-story"), "checkbox"));
        assert!(find(window, &[], &(excluded, "activate").into()).is_none());
    })
    .unwrap();
    click(cx, handle, (story, "activate").into());
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let level = ElementId::from((ElementId::from("level"), "end"));
        let slider = find(window, &[], &level).unwrap();
        assert_eq!(slider.role(), Some(Role::Slider));
        assert!(slider.visible());
    })
    .unwrap();

    click(cx, handle, catalog_button.clone());
    cx.simulate_keystrokes(handle.into(), select_all);
    cx.simulate_input(handle.into(), "no matching page");
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(find(window, &[], &"no-components".into()).is_some());
    })
    .unwrap();

    cx.simulate_keystrokes(handle.into(), select_all);
    cx.simulate_input(handle.into(), "button");
    cx.run_until_parked();
    cx.simulate_keystrokes(handle.into(), "enter");
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(find(window, &[], &("action-variant", 0_usize).into()).is_some());
        assert!(find(window, &[], &"gallery-search".into()).is_none());
    })
    .unwrap();

    let catalog_id = ElementId::from("gallery-catalog-pane");
    let detail_id = ElementId::from((ElementId::from("gallery-story-pane"), "button"));
    for (width, inset) in [
        (599., 16.),
        (600., 24.),
        (839., 24.),
        (840., 24.),
        (839., 24.),
    ] {
        cx.simulate_window_resize(handle.into(), size(px(width), px(560.)));
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let detail = find(window, &[], &detail_id).unwrap().bounds();
            let catalog = find(window, &[], &catalog_id);
            assert_eq!(
                detail.right(),
                px(width - if width >= 840. { inset } else { 0. })
            );
            if width >= 840. {
                let catalog = catalog.unwrap().bounds();
                assert_eq!(catalog.left(), px(inset));
                assert_eq!(catalog.size.width, px(360.));
                assert_eq!(detail.left() - catalog.right(), px(24.));
                assert_eq!(detail.top(), catalog.top());
                assert_eq!(detail.bottom(), catalog.bottom());
            } else {
                assert!(catalog.is_none());
                let heading = find(window, &[], &"story-title".into()).unwrap();
                assert_eq!(heading.bounds().left(), px(inset));
            }
        })
        .unwrap();
    }

    cx.simulate_window_resize(handle.into(), size(px(840.), px(560.)));
    cx.update(|cx| {
        let mut theme = material_theme(cx).clone();
        theme.layout.direction = gpui_material::theme::Direction::Rtl;
        set_material_theme(theme, cx);
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let detail = find(window, &[], &detail_id).unwrap().bounds();
        let catalog = find(window, &[], &catalog_id).unwrap().bounds();
        assert_eq!(detail.left(), px(24.));
        assert_eq!(catalog.left() - detail.right(), px(24.));
        assert_eq!(catalog.right(), px(816.));
        let catalog_toggle = find(window, &[], &catalog_button).unwrap();
        let theme_toggle = find(window, &[], &"toggle-theme".into()).unwrap();
        assert!(catalog_toggle.bounds().left() > px(420.));
        assert!(theme_toggle.bounds().right() < px(420.));
    })
    .unwrap();
}

#[gpui::test]
fn theme_control_stays_clear_of_adaptive_navigation(cx: &mut TestAppContext) {
    let handle = cx.update(|cx| {
        init_gallery(cx);
        open_gallery(cx, false, Story::AdaptiveLayout, false).unwrap()
    });
    cx.simulate_window_resize(handle.into(), size(px(340.), px(560.)));
    let feed = ElementId::from("adaptive-feed");
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let theme = find(window, &[], &"toggle-theme".into()).unwrap();
        let destination = find(window, &[], &feed).unwrap();
        assert!(theme.visible() && destination.visible());
        assert!(theme.bounds().bottom() <= destination.bounds().top());
    })
    .unwrap();
    click(cx, handle, feed.clone());
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(find(window, &[], &feed).unwrap().selected(), Some(true));
    })
    .unwrap();
}

#[gpui::test]
fn projects_theme_and_paints_status_in_two_windows(cx: &mut TestAppContext) {
    let (first, second) = cx.update(|cx| {
        init_gallery(cx);
        (
            open_gallery(cx, false, Story::default(), false).unwrap(),
            open_gallery(cx, true, Story::default(), false).unwrap(),
        )
    });
    let label = |cx: &mut TestAppContext, handle: WindowHandle<Root>, id: &str| {
        let id = gpui::ElementId::from(gpui::SharedString::from(id.to_owned()));
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let status = find(window, &[], &id).expect("rendered status");
            assert_eq!(status.role(), Some(Role::Status));
            assert!(status.visible());
            let bounds = status.bounds();
            assert!(bounds.size.width > px(0.) && bounds.size.height > px(0.));
            assert!(bounds.right() <= window.viewport_size().width);
            assert!(bounds.bottom() <= window.viewport_size().height);
            status.label().unwrap().to_owned()
        })
        .unwrap()
    };
    let switch_theme = |cx: &mut TestAppContext, handle: WindowHandle<Root>, name| {
        let id = ElementId::from("toggle-theme");
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let button = find(window, &[], &id).unwrap();
            assert_eq!(button.role(), Some(Role::Button));
            assert_eq!(button.label(), Some(name));
        })
        .unwrap();
        click(cx, handle, id);
    };
    assert!(label(cx, first, "display-scale").starts_with("Display scale "));
    assert_eq!(label(cx, first, "window-theme"), "Window theme Light");
    assert_eq!(label(cx, second, "window-theme"), "Window theme Dark");
    cx.simulate_window_scale_factor_change(second.into(), 1.5);
    assert_eq!(label(cx, second, "display-scale"), "Display scale 150%");
    assert_eq!(label(cx, second, "window-theme"), "Window theme Dark");
    cx.simulate_window_scale_factor_change(second.into(), 1.);
    assert_eq!(label(cx, second, "display-scale"), "Display scale 100%");
    for (handle, appearance) in [(first, Appearance::Light), (second, Appearance::Dark)] {
        cx.update_window(handle.into(), |_, window, cx| {
            let theme = window_material_theme(window, cx);
            assert_eq!(theme.colors.appearance(), appearance);
            assert_eq!(
                theme.colors.color(ColorRole::OnPrimary),
                MaterialTheme::baseline(appearance)
                    .colors
                    .color(ColorRole::OnPrimary)
            );
        })
        .unwrap();
    }
    switch_theme(cx, second, "Switch to light theme");
    assert_eq!(label(cx, first, "window-theme"), "Window theme Light");
    assert_eq!(label(cx, second, "window-theme"), "Window theme Light");
    switch_theme(cx, second, "Switch to dark theme");
    assert_eq!(label(cx, second, "window-theme"), "Window theme Dark");
    switch_theme(cx, first, "Switch to dark theme");
    assert_eq!(label(cx, first, "window-theme"), "Window theme Dark");
    assert_eq!(label(cx, second, "window-theme"), "Window theme Dark");
    cx.update(|cx| {
        let base = Theme::global(cx);
        assert_eq!(base.appearance, ThemeAppearance::Dark);
        assert_eq!(
            base.tokens.colors.primary,
            color(material_theme(cx).colors.color(ColorRole::Primary))
        );
    });
    switch_theme(cx, first, "Switch to light theme");
    assert_eq!(label(cx, first, "window-theme"), "Window theme Light");
    assert_eq!(label(cx, second, "window-theme"), "Window theme Dark");
}

#[gpui::test]
fn mixed_script_uses_one_layout_for_metrics_and_paint(cx: &mut TestAppContext) {
    let handle = cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        open_gallery(cx, false, Story::default(), false).unwrap()
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let theme = material_theme(cx);
        let style = TypeRole::BodyLarge.style(TypographyEmphasis::Baseline);
        let mut typography = theme.typography.clone();
        typography.plain.primary = "An unavailable Material test font".into();
        let line = shape_text(
            "A中文B",
            style,
            &typography,
            1.25,
            color(theme.colors.color(ColorRole::OnSurface)),
            window,
            cx,
        )
        .unwrap();
        assert_eq!(line.line_height(), px(style.line_height * 1.25));
        assert_eq!(line.line().len(), "A中文B".len());
        assert!(line.line().font_id_for_index(1).is_some());
        assert!(line.line().x_for_index(4) > line.line().x_for_index(1));
        assert_eq!(
            line.line().x_for_index(line.line().len()),
            line.line().width()
        );
    })
    .unwrap();
}
