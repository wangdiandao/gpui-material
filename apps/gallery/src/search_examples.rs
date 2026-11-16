use std::time::Duration;

use gpui::{
    AppContext, Context, Entity, ParentElement, Render, SharedString, Subscription, Window, px,
};
use gpui_material::{
    AppBar, Button, ButtonVariant, Icon, IconName, SearchBar, SearchEvent, SearchOutcome,
    SearchPopupOptions, SearchResult, SearchScrim, SearchState, SearchViewVariant, Text,
    theme::TypeRole,
};

use crate::story::{section, stack};

#[cfg(all(test, feature = "test-support"))]
mod customization_tests;

pub(crate) struct SearchExamples {
    docked: Entity<SearchState>,
    fullscreen: Entity<SearchState>,
    message: SharedString,
    _subscriptions: [Subscription; 2],
}

impl SearchExamples {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let docked = cx.new(|cx| SearchState::new(window, cx));
        let fullscreen = cx.new(|cx| SearchState::new(window, cx));
        let subscriptions = [&docked, &fullscreen].map(|state| {
            cx.subscribe_in(state, window, |example, state, event, _, cx| match event {
                SearchEvent::Request(request) => {
                    let request = request.clone();
                    let timer = cx.background_executor().timer(Duration::from_millis(
                        if request.query() == "slow" { 800 } else { 180 },
                    ));
                    let weak = state.downgrade();
                    cx.spawn(async move |_, cx| {
                        timer.await;
                        let outcome = matches(request.query());
                        let _ = weak.update(cx, |state, cx| state.complete(&request, outcome, cx));
                    })
                    .detach();
                }
                SearchEvent::Submit(query) => {
                    example.message = format!("Submitted “{query}”").into();
                    cx.notify();
                }
                SearchEvent::Select(result) => {
                    example.message = format!("Opened {}", result.label()).into();
                    cx.notify();
                }
            })
        });
        Self {
            docked,
            fullscreen,
            message: "".into(),
            _subscriptions: subscriptions,
        }
    }
}

fn matches(query: &str) -> SearchOutcome {
    if query.eq_ignore_ascii_case("error") {
        return SearchOutcome::Error("Search is unavailable. Try a different query.".into());
    }
    let query = query.to_lowercase();
    SearchOutcome::Results(
        [
            ("design", "Design brief", "Documents"),
            ("meeting", "Meeting notes", "Documents"),
            ("colours", "Colors and typography", "Reference"),
            ("beijing", "北京指南", "中文"),
        ]
        .into_iter()
        .filter(|(_, label, _)| label.to_lowercase().contains(&query))
        .map(|(id, label, category)| {
            SearchResult::new(id, label)
                .group(category)
                .icon(IconName::Description)
        })
        .collect(),
    )
}

impl Render for SearchExamples {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        stack()
            .child(
                section("Docked")
                    .description("Search for “design” or “meeting” to see matching documents.")
                    .child(
                        SearchBar::new("docked", "Search documents", &self.docked)
                            .widths(px(250.), px(340.))
                            .expanded_view(|view| {
                                view.result_gap(px(8.))
                                    .popup_options(SearchPopupOptions {
                                        scrim: Some(SearchScrim {
                                            opacity: 0.18,
                                            ..Default::default()
                                        }),
                                        ..Default::default()
                                    })
                                    .input(|field| {
                                        field
                                            .leading(Icon::new(IconName::Search))
                                            .suffix(Text::new("docs").id("search-suffix"))
                                    })
                            }),
                    ),
            )
            .child(
                section("Contained app bar")
                    .description("Try “error” to preview a failed search.")
                    .child(
                        AppBar::new("search-app-bar", "Documents").search(
                            SearchBar::new("fullscreen", "Search documents", &self.fullscreen)
                                .variant(SearchViewVariant::FullScreenContained)
                                .expanded_view(|view| {
                                    view.focused_width(px(580.)).filters(
                                        Button::new("search-recent", "Search recent meetings")
                                            .variant(ButtonVariant::Text)
                                            .on_activate({
                                                let state = self.fullscreen.clone();
                                                move |window, cx| {
                                                    state.update(cx, |state, cx| {
                                                        state.set_query("meeting", window, cx);
                                                        state.input_focus(cx).focus(window, cx);
                                                    })
                                                }
                                            }),
                                    )
                                }),
                        ),
                    ),
            )
            .child(Text::new(self.message.clone()).role(TypeRole::BodyMedium))
    }
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn search_preview_updates_results_and_restores_focus(cx: &mut gpui::TestAppContext) {
    use gpui::{ElementId, InputEvent as _};
    use gpui_base::test_support::find;
    use gpui_material::MaterialRoot;

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| SearchExamples::new(window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });
    let frame = |cx: &mut gpui::TestAppContext| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    frame(cx);
    let trigger = ElementId::from((ElementId::from("docked"), "search-trigger"));
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        let position = find(window, &[], &trigger).unwrap().bounds().center();
        for event in [
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
            window.dispatch_event(event, cx);
        }
    })
    .unwrap();
    frame(cx);
    frame(cx);
    cx.simulate_input(handle.into(), "design");
    frame(cx);
    cx.update(|cx| {
        let state = view.read(cx).docked.read(cx);
        assert!(state.is_open());
        assert_eq!(state.query(), "design");
    });
    cx.executor().advance_clock(Duration::from_millis(250));
    frame(cx);
    cx.update_window(handle.into(), |_, window, _| {
        assert!(find(window, &[], &"design".into()).is_some());
    })
    .unwrap();
    cx.simulate_keystrokes(handle.into(), "escape");
    frame(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        assert!(!view.read(cx).docked.read(cx).is_open());
        assert!(window.focused(cx).is_some());
        assert_eq!(find(window, &[], &trigger).unwrap().focused(), Some(true));
    })
    .unwrap();

    cx.update_window(handle.into(), |_, window, cx| {
        let fullscreen = view.read(cx).fullscreen.clone();
        fullscreen.update(cx, |state, cx| state.open(window, cx));
    })
    .unwrap();
    frame(cx);
    cx.simulate_input(handle.into(), "meeting");
    frame(cx);
    cx.update(|cx| assert_eq!(view.read(cx).fullscreen.read(cx).query(), "meeting"));
    cx.simulate_keystrokes(handle.into(), "escape");
    frame(cx);
    cx.update(|cx| assert!(!view.read(cx).fullscreen.read(cx).is_open()));
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn large_search_virtualizes_variable_rows_and_navigates_by_stable_id(
    cx: &mut gpui::TestAppContext,
) {
    use gpui::{IntoElement, Role};
    use gpui_base::test_support::{find, snapshots};
    use gpui_material::{MaterialRoot, SearchRequest};

    struct Harness {
        state: Entity<SearchState>,
        request: Option<SearchRequest>,
        selected: Option<gpui::ElementId>,
        _subscription: Subscription,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            SearchBar::new("large-search", "Search", &self.state)
        }
    }
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let handle = MaterialRoot::open_window(Default::default(), cx, |window, cx| {
            let entity = cx.new(|cx: &mut Context<Harness>| {
                let state = cx.new(|cx| SearchState::new(window, cx));
                let subscription = cx.subscribe(&state, |this, _, event, _| match event {
                    SearchEvent::Request(request) => this.request = Some(request.clone()),
                    SearchEvent::Select(result) => this.selected = Some(result.id().clone()),
                    _ => {}
                });
                Harness {
                    state,
                    request: None,
                    selected: None,
                    _subscription: subscription,
                }
            });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, view.unwrap())
    });
    let frame = |cx: &mut gpui::TestAppContext| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    frame(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        view.read(cx).state.clone().update(cx, |state, cx| {
            state.open(window, cx);
            state.set_query("many", window, cx);
        });
    })
    .unwrap();
    let values = || {
        (0..2000)
            .map(|ix| {
                let result = SearchResult::new(format!("result-{ix}"), format!("Result {ix}"));
                if ix % 2 == 0 {
                    result.supporting(
                        "Supporting text that wraps over multiple lines in a narrow result surface",
                    )
                } else {
                    result
                }
            })
            .collect()
    };
    cx.update(|cx| {
        let request = view.read(cx).request.clone().unwrap();
        view.read(cx).state.clone().update(cx, |state, cx| {
            assert!(state.complete(&request, SearchOutcome::Results(values()), cx));
        });
    });
    frame(cx);
    frame(cx);
    cx.update_window(handle.into(), |_, window, _| {
        let count = snapshots(window)
            .iter()
            .filter(|item| item.role() == Some(Role::ListItem))
            .count();
        assert!(count > 0 && count < 30, "rendered {count} rows");
        assert!(find(window, &[], &"result-1000".into()).is_none());
    })
    .unwrap();
    // Scrolling with the editor focused must leave a visible Tab entry into
    // the collection even though result zero is no longer mounted.
    cx.update_window(handle.into(), |_, window, cx| {
        use gpui::InputEvent as _;
        let position = find(window, &[], &"result-1".into())
            .unwrap()
            .bounds()
            .center();
        window.dispatch_event(
            gpui::ScrollEvent {
                position,
                delta: gpui::ScrollDelta::Pixels(gpui::point(gpui::px(0.), gpui::px(-400.))),
                modifiers: Default::default(),
                touch_phase: gpui::TouchPhase::Moved,
                source: Default::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx);
    frame(cx);
    cx.simulate_keystrokes(handle.into(), "tab");
    frame(cx);
    cx.update_window(handle.into(), |_, window, _| {
        assert!(snapshots(window).iter().any(|node| {
            node.focused() == Some(true)
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("Result "))
                && node.visible()
        }));
    })
    .unwrap();
    for _ in 0..15 {
        cx.simulate_keystrokes(handle.into(), "up");
        frame(cx);
        if cx
            .update_window(handle.into(), |_, window, _| {
                find(
                    window,
                    &[],
                    &(gpui::ElementId::from("result-1999"), "activate").into(),
                )
                .is_some_and(|node| node.focused() == Some(true))
            })
            .unwrap()
        {
            break;
        }
    }
    cx.update_window(handle.into(), |_, window, _| {
        let last = find(
            window,
            &[],
            &(gpui::ElementId::from("result-1999"), "activate").into(),
        )
        .unwrap();
        assert_eq!(last.focused(), Some(true));
        assert!(last.visible());
    })
    .unwrap();
    // Refreshing the installed-font snapshot must retain the distant row anchor.
    cx.update_window(handle.into(), |_, window, cx| {
        let theme = gpui_material::foundation::window_material_theme(window, cx).clone();
        let before = gpui_material::foundation::font_cache_revision(cx);
        gpui_material::foundation::set_window_material_theme(Some(theme), window, cx);
        assert_ne!(before, gpui_material::foundation::font_cache_revision(cx));
    })
    .unwrap();
    frame(cx);
    cx.update_window(handle.into(), |_, window, _| {
        let last = find(
            window,
            &[],
            &(gpui::ElementId::from("result-1999"), "activate").into(),
        )
        .unwrap();
        assert!(last.visible());
        assert_eq!(last.focused(), Some(true));
    })
    .unwrap();
    // Replace/reorder the model before the next frame. Focus remains attached
    // to the result's ID, not its old numeric position.
    cx.update_window(handle.into(), |_, window, cx| {
        view.read(cx)
            .state
            .clone()
            .update(cx, |state, cx| state.set_query("reorder", window, cx));
    })
    .unwrap();
    cx.update(|cx| {
        let request = view.read(cx).request.clone().unwrap();
        let mut rows: Vec<_> = values();
        rows.reverse();
        view.read(cx).state.clone().update(cx, |state, cx| {
            assert!(state.complete(&request, SearchOutcome::Results(rows), cx));
        });
    });
    frame(cx);
    cx.simulate_keystrokes(handle.into(), "down");
    frame(cx);
    cx.simulate_keystrokes(handle.into(), "enter");
    cx.update_window(handle.into(), |_, window, cx| {
        use gpui::InputEvent as _;
        let keystroke = gpui::Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap();
        window.dispatch_event(gpui::KeyUpEvent { keystroke }.to_platform_input(), cx);
    })
    .unwrap();
    frame(cx);
    cx.update(|cx| {
        assert_eq!(view.read(cx).selected, Some("result-1998".into()));
        assert!(!view.read(cx).state.read(cx).is_open());
    });
}
