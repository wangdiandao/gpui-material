use gpui::{
    AppContext, ElementId, InputEvent as _, Role, TestAppContext, WindowHandle, WindowOptions, px,
    size,
};
use gpui_base::{Root, Theme, ThemeAppearance, test_support::find};
use gpui_material::foundation::{
    MaterialRoot, color, material_theme, set_material_theme, shape_text, window_material_theme,
};
use gpui_material::theme::{Appearance, ColorRole, MaterialTheme, TypeRole, TypographyEmphasis};

use crate::{Gallery, Story, init_gallery, open_gallery};

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
        cx.set_reduce_motion(true);
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

#[gpui::test]
fn gallery_lazy_loads_stories_and_preserves_visited_views(cx: &mut TestAppContext) {
    let (handle, gallery) = cx.update(|cx| {
        init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut gallery = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let view = cx.new(|cx| Gallery::new(Story::Introduction, window, cx));
            gallery = Some(view.clone());
            view
        })
        .unwrap();
        (handle, gallery.unwrap())
    });
    cx.simulate_window_resize(handle.into(), size(px(1040.), px(960.)));

    cx.update(|cx| {
        let g = gallery.read(cx);
        assert_eq!(g.loaded_story_count(), 0);
        assert!(g.story_view_id(Story::AdaptiveLayout).is_none());
        assert!(g.story_view_id(Story::Button).is_none());
    });

    let adaptive_story = ElementId::from((ElementId::from("gallery-story"), "adaptive-layout"));
    click(cx, handle, (adaptive_story.clone(), "activate").into());
    let adaptive_view_id = cx.update(|cx| {
        let g = gallery.read(cx);
        assert_eq!(g.loaded_story_count(), 1);
        assert!(g.story_view_id(Story::Button).is_none());
        g.story_view_id(Story::AdaptiveLayout)
            .expect("adaptive layout story view loaded")
    });

    let feed = ElementId::from("adaptive-feed");
    click(cx, handle, feed.clone());
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(find(window, &[], &feed).unwrap().selected(), Some(true));
    })
    .unwrap();

    let button_story = ElementId::from((ElementId::from("gallery-story"), "button"));
    click(cx, handle, (button_story, "activate").into());
    let button_view_id = cx.update(|cx| {
        let g = gallery.read(cx);
        assert_eq!(g.loaded_story_count(), 2);
        assert_eq!(
            g.story_view_id(Story::AdaptiveLayout),
            Some(adaptive_view_id)
        );
        g.story_view_id(Story::Button)
            .expect("button story view loaded")
    });

    click(cx, handle, (adaptive_story, "activate").into());
    cx.update(|cx| {
        let g = gallery.read(cx);
        assert_eq!(g.loaded_story_count(), 2);
        assert_eq!(
            g.story_view_id(Story::AdaptiveLayout),
            Some(adaptive_view_id)
        );
        assert_eq!(g.story_view_id(Story::Button), Some(button_view_id));
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(find(window, &[], &feed).unwrap().selected(), Some(true));
    })
    .unwrap();

    let initial_button_gallery = cx.update(|cx| {
        let mut gallery = None;
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let view = cx.new(|cx| Gallery::new(Story::Button, window, cx));
            gallery = Some(view.clone());
            view
        })
        .unwrap();
        gallery.unwrap()
    });
    cx.update(|cx| {
        let g = initial_button_gallery.read(cx);
        assert_eq!(g.loaded_story_count(), 1);
        assert!(g.story_view_id(Story::Button).is_some());
        assert!(g.story_view_id(Story::AdaptiveLayout).is_none());
    });
}

#[gpui::test]
fn gallery_virtualizes_catalog_and_skips_noop_navigation_notifications(cx: &mut TestAppContext) {
    use std::{cell::Cell, rc::Rc};

    let notifications = Rc::new(Cell::new(0usize));
    let (handle, gallery) = cx.update(|cx| {
        init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut gallery = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let view = cx.new(|cx| Gallery::new(Story::Button, window, cx));
            let counter = notifications.clone();
            cx.observe(&view, move |_, _| {
                counter.set(counter.get() + 1);
            })
            .detach();
            gallery = Some(view.clone());
            view
        })
        .unwrap();
        (handle, gallery.unwrap())
    });
    cx.simulate_window_resize(handle.into(), size(px(1040.), px(744.)));
    cx.run_until_parked();

    let rendered_rows = cx
        .update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            Story::ALL
                .iter()
                .filter(|story| {
                    let id = ElementId::from((ElementId::from("gallery-story"), story.id()));
                    find(window, &[], &(id, "activate").into()).is_some()
                })
                .count()
        })
        .unwrap();
    assert!(rendered_rows > 0);
    assert!(
        rendered_rows < Story::ALL.len(),
        "expected virtualized catalog to render fewer than {} stories, rendered {}",
        Story::ALL.len(),
        rendered_rows
    );

    // Re-activating the already-selected story in two-pane mode must not notify Gallery.
    notifications.set(0);
    let button_story = ElementId::from((ElementId::from("gallery-story"), "button"));
    click(cx, handle, (button_story.clone(), "activate").into());
    assert_eq!(notifications.get(), 0);

    // Focus the selected catalog row, then press End to reveal and focus the last story.
    let last_story = *Story::ALL.last().expect("catalog is non-empty");
    let last_id = ElementId::from((ElementId::from("gallery-story"), last_story.id()));
    cx.update_window(handle.into(), |_, window, cx| {
        let focus = gallery
            .read(cx)
            .catalog_story_focus(Story::Button)
            .expect("button catalog focus");
        focus.focus(window, cx);
    })
    .unwrap();
    cx.simulate_keystrokes(handle.into(), "end");
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let last = find(window, &[], &(last_id, "activate").into());
        assert!(
            last.is_some(),
            "expected End key to reveal last story {}",
            last_story.id()
        );
    })
    .unwrap();
}

#[gpui::test]
fn noop_state_updates_skip_redundant_notifications(cx: &mut TestAppContext) {
    use std::rc::Rc;

    use gpui_material::{
        AppBarScrollState, BottomSheetState, BottomSheetValue, PullToRefreshState,
        SearchScrollState, SwipeToDismissState, SwipeToDismissValue, ToolbarScrollState,
        date_time::{DatePickerState, ScrollFieldState, TimePickerState},
    };

    let (handle, counts) = cx.update(|cx| {
        init_gallery(cx);
        cx.set_reduce_motion(true);
        let counts = Rc::new(std::cell::RefCell::new(Vec::<&'static str>::new()));
        let mut states = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let ptr = cx.new(|_| PullToRefreshState::new());
            let app_bar = cx.new(|_| AppBarScrollState::new());
            let search = cx.new(|_| SearchScrollState::new());
            let toolbar = cx.new(|_| ToolbarScrollState::new());
            let wheel = cx.new(|cx| ScrollFieldState::new(12, 0, cx));
            let swipe = cx.new(|_| SwipeToDismissState::new());
            let sheet = cx.new(|cx| BottomSheetState::new(BottomSheetValue::Hidden, cx));
            let date = cx.new(|cx| DatePickerState::new(window, cx));
            let time = cx.new(|cx| TimePickerState::new(window, cx));

            macro_rules! track {
                ($name:literal, $entity:expr) => {{
                    let c = counts.clone();
                    cx.observe(&$entity, move |_, _| c.borrow_mut().push($name))
                        .detach();
                }};
            }
            track!("ptr", ptr);
            track!("app_bar", app_bar);
            track!("search", search);
            track!("toolbar", toolbar);
            track!("wheel", wheel);
            track!("swipe", swipe);
            track!("sheet", sheet);
            track!("date", date);
            track!("time", time);

            states = Some((
                ptr, app_bar, search, toolbar, wheel, swipe, sheet, date, time,
            ));
            cx.new(|_| EmptyView)
        })
        .unwrap();
        (handle, (counts, states.unwrap()))
    });
    let (counts, (ptr, app_bar, search, toolbar, wheel, swipe, sheet, date, time)) = counts;
    cx.run_until_parked();
    counts.borrow_mut().clear();

    cx.update_window(handle.into(), |_, window, cx| {
        ptr.update(cx, |s, cx| {
            s.snap_to(0., cx);
            s.animate_to_hidden(cx);
        });
        app_bar.update(cx, |s, cx| {
            s.set_collapsed_fraction(0., cx);
            s.settle(0., cx);
            s.settle(0., cx);
        });
        search.update(cx, |s, cx| {
            s.set_hidden_fraction(0., cx);
        });
        toolbar.update(cx, |s, cx| {
            s.set_collapsed_fraction(0., cx);
            s.settle(0., cx);
            s.settle(0., cx);
        });
        wheel.update(cx, |s, cx| {
            s.scroll_to_option(0, cx);
        });
        swipe.update(cx, |s, cx| {
            s.snap_to(SwipeToDismissValue::Settled, window, cx);
        });
        sheet.update(cx, |s, cx| {
            s.hide(window, cx);
        });
        let displayed = date.read(cx).displayed_month();
        date.update(cx, |s, cx| {
            s.set_displayed_month(displayed, cx);
            s.set_displayed_month(displayed, cx);
        });
        let (value, hour, minute) = {
            let state = time.read(cx);
            (state.value(), state.hour(), state.minute())
        };
        time.update(cx, |s, cx| {
            s.set_value(value, window, cx);
            s.set_hour(hour, window, cx);
            s.set_minute(minute, window, cx);
        });
    })
    .unwrap();
    cx.run_until_parked();

    // Only the initial settle(0.) on app_bar/toolbar and the first set_displayed_month (if cursor != first_of_month)
    // can legitimately notify; repeated identical calls must be silent.
    counts.borrow_mut().clear();
    cx.update_window(handle.into(), |_, window, cx| {
        ptr.update(cx, |s, cx| {
            s.snap_to(0., cx);
            s.animate_to_hidden(cx);
        });
        app_bar.update(cx, |s, cx| {
            s.settle(0., cx);
        });
        search.update(cx, |s, cx| {
            s.set_hidden_fraction(0., cx);
        });
        toolbar.update(cx, |s, cx| {
            s.settle(0., cx);
        });
        wheel.update(cx, |s, cx| {
            s.scroll_to_option(0, cx);
        });
        swipe.update(cx, |s, cx| {
            s.snap_to(SwipeToDismissValue::Settled, window, cx);
        });
        sheet.update(cx, |s, cx| {
            s.hide(window, cx);
        });
        let displayed = date.read(cx).displayed_month();
        date.update(cx, |s, cx| {
            s.set_displayed_month(displayed, cx);
        });
        let (value, hour, minute) = {
            let state = time.read(cx);
            (state.value(), state.hour(), state.minute())
        };
        time.update(cx, |s, cx| {
            s.set_value(value, window, cx);
            s.set_hour(hour, window, cx);
            s.set_minute(minute, window, cx);
        });
    })
    .unwrap();
    cx.run_until_parked();
    assert_eq!(
        *counts.borrow(),
        Vec::<&str>::new(),
        "no-op state updates must not emit notifications"
    );

    // Genuine state changes still notify.
    cx.update_window(handle.into(), |_, window, cx| {
        ptr.update(cx, |s, cx| s.snap_to(0.5, cx));
        search.update(cx, |s, cx| s.set_hidden_fraction(0.5, cx));
        wheel.update(cx, |s, cx| s.scroll_to_option(3, cx));
        time.update(cx, |s, cx| {
            let next_minute = (s.minute() + 1) % 60;
            assert!(s.set_minute(next_minute, window, cx));
        });
    })
    .unwrap();
    cx.run_until_parked();
    assert_eq!(counts.borrow().len(), 4);
}

struct EmptyView;

impl gpui::Render for EmptyView {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        _: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        gpui::div()
    }
}
