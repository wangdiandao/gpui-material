use std::time::Duration;

use gpui::{
    AppContext as _, Bounds, Context, Edges, ElementId, Entity, EntityInputHandler as _,
    InteractiveElement as _, IntoElement, ListAlignment, ListState, Modifiers, ParentElement as _,
    Pixels, Render, Role, ScrollHandle, SharedString, Styled as _, Subscription, TestAppContext,
    VisualTestContext, Window, div, point, px,
};
use gpui_base::test_support::{find, find_all};
use gpui_material::{
    Button, IconName, MaterialTheme, SearchBar, SearchEvent, SearchMotion, SearchOutcome,
    SearchPopupOptions, SearchRequest, SearchResult, SearchScrim, SearchScrollState, SearchState,
    SearchStyle, SearchValue, SearchViewVariant, SearchWindowOptions, Text, TextFieldState,
    foundation::set_window_material_theme,
    theme::{CornerShape, Direction},
};

struct Harness {
    state: Entity<SearchState>,
    editor: Entity<TextFieldState>,
    scroll: ListState,
    bar_scroll: Entity<SearchScrollState>,
    linked_scroll: ScrollHandle,
    variant: SearchViewVariant,
    requests: Vec<SearchRequest>,
    submits: Vec<SharedString>,
    selected: Vec<ElementId>,
    background_count: usize,
    _subscription: Subscription,
}

impl Render for Harness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        let mut bar = SearchBar::new("custom-search", "Search catalogue", &self.state)
            .variant(self.variant)
            .widths(px(204.), px(310.))
            .trigger_content(Text::new("Find records"))
            .style(SearchStyle {
                shape: Some(CornerShape::Radii([6., 8., 6., 8.])),
                divider: Some(gpui::rgb(0x3f5680).into()),
                ..Default::default()
            });
        if self.variant == SearchViewVariant::Docked {
            bar = bar.expanded_view(|view| {
                view.docked_size(px(380.), px(304.))
                    .result_gap(px(16.))
                    .popup_options(SearchPopupOptions {
                        offset: px(10.),
                        scrim: Some(SearchScrim {
                            opacity: 0.21,
                            ..Default::default()
                        }),
                        dismiss_on_outside: false,
                        ..Default::default()
                    })
                    .input(|field| {
                        div().id("custom-input").child(
                            field
                                .prefix(Text::new("Topic:").id("query-prefix"))
                                .suffix(Text::new("catalogue").id("query-suffix")),
                        )
                    })
                    .filters(Button::new("catalogue-filter", "Only guides").on_activate(|_, _| {}))
                    .result_row(|row, result, _, _| {
                        let title = result.label().clone();
                        row.content(Text::new(title).id("custom-title"))
                    })
            });
        } else {
            bar = bar
                .motion(SearchMotion {
                    expand: Duration::from_millis(300),
                    collapse: Duration::from_millis(260),
                })
                .expanded_view(|view| {
                    view.window_options(SearchWindowOptions {
                        insets: Edges::all(px(20.)),
                        dismiss_on_escape: false,
                        ..Default::default()
                    })
                    .result_content(|state, _, cx| {
                        let query = state.read(cx).query().clone();
                        let select = state.clone();
                        div().id("application-results").child(
                            Button::new("custom-result", format!("Open {query}")).on_activate(
                                move |window, cx| {
                                    select.update(cx, |state, cx| {
                                        state.select_result(
                                            SearchResult::new("curated", "Curated result"),
                                            window,
                                            cx,
                                        )
                                    });
                                },
                            ),
                        )
                    })
                });
        }

        div()
            .id("search-surroundings")
            .flex()
            .flex_col()
            .child(
                bar.scroll_behavior(&self.bar_scroll)
                    .track_scroll(&self.linked_scroll),
            )
            .child(
                Button::new("background-action", "Background command").on_activate(move |_, cx| {
                    owner.update(cx, |view, _| view.background_count += 1);
                }),
            )
    }
}

fn open(
    cx: &mut TestAppContext,
    variant: SearchViewVariant,
) -> (Entity<Harness>, &mut VisualTestContext) {
    cx.update(crate::init_gallery);
    let mut view = None;
    let (_, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        let example: Entity<Harness> = cx.new(|cx: &mut Context<Harness>| {
            let editor = cx.new(|cx| TextFieldState::new(window, cx));
            let scroll = ListState::new(0, ListAlignment::Top, px(120.));
            let state = cx.new(|cx| {
                let mut state = SearchState::with_editor(&editor, window, cx);
                state.set_scroll_state(&scroll, cx);
                state
            });
            let subscription = cx.subscribe(&state, |view, _, event, _| match event {
                SearchEvent::Request(request) => view.requests.push(request.clone()),
                SearchEvent::Submit(query) => view.submits.push(query.clone()),
                SearchEvent::Select(result) => view.selected.push(result.id().clone()),
            });
            Harness {
                state,
                editor,
                scroll,
                variant,
                linked_scroll: ScrollHandle::new(),
                bar_scroll: cx.new(|_| SearchScrollState::new()),
                requests: Vec::new(),
                submits: Vec::new(),
                selected: Vec::new(),
                background_count: 0,
                _subscription: subscription,
            }
        });
        view = Some(example.clone());
        gpui_base::Root::new(example, window, cx)
    });
    draw(cx);
    (view.unwrap(), cx)
}

fn draw(cx: &mut VisualTestContext) {
    for _ in 0..2 {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
    }
}

fn trigger_id() -> ElementId {
    (ElementId::from("custom-search"), "search-trigger").into()
}

fn view_child(id: &'static str) -> ElementId {
    let view: ElementId = (ElementId::from("custom-search"), "search-view").into();
    (view, id).into()
}

fn bounds(cx: &mut VisualTestContext, id: ElementId) -> Bounds<Pixels> {
    cx.update(|window, _| {
        find(window, &[], &id)
            .unwrap_or_else(|| panic!("missing {id:?}"))
            .bounds()
    })
}

#[gpui::test]
fn docked_search_composes_grouped_results_with_an_external_viewport_and_ime(
    cx: &mut TestAppContext,
) {
    let (view, cx) = open(cx, SearchViewVariant::Docked);
    cx.update(|window, cx| {
        let mut theme = MaterialTheme::default();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    });
    draw(cx);
    let collapsed = bounds(cx, trigger_id());
    assert_eq!(collapsed.size.width, px(204.));
    let background = bounds(cx, "background-action".into()).center();
    cx.simulate_click(collapsed.center(), Modifiers::default());
    draw(cx);
    let editor = bounds(cx, view_child("input-region"));
    let results = bounds(cx, view_child("results"));
    assert_eq!(results.top() - editor.bottom(), px(16.));
    assert_eq!(bounds(cx, trigger_id()).size.width, px(310.));
    assert!(bounds(cx, "query-prefix".into()).left() > bounds(cx, "query-suffix".into()).right());
    cx.update(|window, cx| {
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| state.set_query("manual", window, cx));
    });
    let pending = cx.update(|_, cx| view.read(cx).requests[0].clone());
    cx.update(|window, cx| {
        let editor = view.read(cx).editor.clone();
        editor.update(cx, |editor, cx| {
            editor.replace_and_mark_text_in_range(None, "候", Some(1..1), window, cx);
            assert!(editor.marked_text_range(window, cx).is_some());
        });
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| {
            assert!(
                state.complete(
                    &pending,
                    SearchOutcome::Results(
                        (0_usize..1000)
                            .map(|i| {
                                SearchResult::new(("row", i), format!("Guide {i}"))
                                    .group(format!("Group {}", i / 2))
                                    .icon(IconName::Description)
                            })
                            .collect(),
                    ),
                    cx
                )
            );
        });
    });
    draw(cx);
    cx.update(|window, cx| {
        let current = view.read(cx);
        assert_eq!(current.requests.len(), 1);
        assert_eq!(current.state.read(cx).query(), "manual");
        assert_eq!(current.scroll.item_count(), 1000);
        let group_title_id: ElementId = (
            ElementId::from((ElementId::from("custom-search"), "search-view")),
            "search-group-title",
        )
            .into();
        let headings = find_all(window, &[], &group_title_id);
        assert!(headings.len() > 1);
        assert!(
            headings
                .iter()
                .all(|heading| heading.role() == Some(Role::Heading))
        );
        assert!(find(window, &[], &("row", 700_usize).into()).is_none());
        current.scroll.scroll_to(gpui::ListOffset {
            item_ix: 750,
            offset_in_item: px(0.),
        });
    });
    draw(cx);
    assert!(cx.update(|window, _| {
        find(window, &[], &("row", 750_usize).into())
            .unwrap()
            .visible()
    }));
    cx.simulate_click(background, Modifiers::default());
    draw(cx);
    cx.update(|window, cx| {
        let current = view.read(cx);
        assert_eq!(current.background_count, 0);
        assert!(current.state.read(cx).is_open());
        let editor = current.editor.clone();
        editor.update(cx, |editor, cx| {
            editor.replace_text_in_range(None, "committed", window, cx);
            editor.unmark_text(window, cx);
        });
    });
    draw(cx);
    cx.update(|_, cx| {
        let current = view.read(cx);
        assert_eq!(current.requests.len(), 2);
        assert_ne!(current.requests[1].query(), "manual");
        assert!(current.submits.is_empty());
        let new_request = current.requests[1].clone();
        let scroll = current.scroll.clone();
        current.state.clone().update(cx, |state, cx| {
            assert!(!state.complete(&pending, SearchOutcome::Error("stale".into()), cx));
            assert!(
                state.complete(
                    &new_request,
                    SearchOutcome::Results(
                        (0_usize..1000)
                            .rev()
                            .map(|i| {
                                SearchResult::new(("row", i), format!("Updated guide {i}"))
                                    .group(format!("Updated group {}", i / 3))
                            })
                            .collect()
                    ),
                    cx,
                )
            );
        });
        assert_eq!(scroll.item_count(), 1000);
        scroll.scroll_to(gpui::ListOffset {
            item_ix: 499,
            offset_in_item: px(0.),
        });
    });
    draw(cx);
    cx.update(|window, _| {
        let row = find(window, &[], &("row", 500_usize).into()).unwrap();
        assert!(row.visible());
        let group_title_id: ElementId = (
            ElementId::from((ElementId::from("custom-search"), "search-view")),
            "search-group-title",
        )
            .into();
        assert!(find_all(window, &[], &group_title_id).iter().all(|title| {
            title
                .label()
                .is_some_and(|label| label.starts_with("Updated group "))
        }));
    });
}

#[gpui::test]
fn contained_search_retargets_partial_motion_and_restores_original_focus(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, SearchViewVariant::FullScreenContained);
    let trigger = bounds(cx, trigger_id()).center();
    cx.simulate_click(trigger, Modifiers::default());
    draw(cx);
    cx.update(|window, cx| {
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| {
            assert!(state.is_animating());
            state.snap_to(0.35, window, cx);
            assert_eq!(state.current_value(), SearchValue::Expanded);
            state.animate_to(
                SearchValue::Expanded,
                Duration::from_millis(300),
                window,
                cx,
            );
        });
    });
    cx.executor().advance_clock(Duration::from_millis(130));
    draw(cx);
    let current = cx.update(|window, cx| {
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| {
            let progress = state.progress();
            assert!(progress > 0.35 && progress < 1.);
            state.animate_to(
                SearchValue::Collapsed,
                Duration::from_millis(260),
                window,
                cx,
            );
            assert!(!state.is_open());
            progress
        })
    });
    cx.executor().advance_clock(Duration::from_millis(95));
    draw(cx);
    cx.update(|window, cx| {
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| {
            assert!(state.progress() < current && state.progress() > 0.);
            state.animate_to(
                SearchValue::Expanded,
                Duration::from_millis(300),
                window,
                cx,
            );
            assert!(state.input_focus(cx).is_focused(window));
        });
    });
    cx.executor().advance_clock(Duration::from_millis(310));
    draw(cx);
    assert_eq!(bounds(cx, view_child("input-region")).size.width, px(310.));
    assert!(bounds(cx, "custom-result".into()).size.height > px(0.));
    cx.simulate_keystrokes("escape");
    draw(cx);
    cx.update(|window, cx| {
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| {
            assert_eq!(state.progress(), 1.);
            assert!(state.is_open());
            state.animate_to(
                SearchValue::Collapsed,
                Duration::from_millis(260),
                window,
                cx,
            );
        });
    });
    cx.executor().advance_clock(Duration::from_millis(265));
    draw(cx);
    cx.update(|window, cx| {
        let model = view.read(cx).state.read(cx);
        assert_eq!(model.progress(), 0.);
        assert!(!model.is_animating());
        assert!(find(window, &[], &"custom-result".into()).is_none());
        assert_eq!(
            find(window, &[], &trigger_id()).unwrap().focused(),
            Some(true)
        );
    });
}

#[gpui::test]
fn standalone_search_scroll_reveals_on_reverse_and_keeps_active_editor(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, SearchViewVariant::Docked);
    let scroll: ElementId = (ElementId::from("custom-search"), "search-scroll").into();
    let initial = bounds(cx, scroll.clone()).size.height;
    cx.update(|_, cx| {
        view.read(cx)
            .linked_scroll
            .set_offset(point(px(0.), px(-90.)))
    });
    draw(cx);
    assert_eq!(bounds(cx, scroll.clone()).size.height, px(0.));
    cx.update(|_, cx| {
        view.read(cx)
            .linked_scroll
            .set_offset(point(px(0.), px(-64.)))
    });
    draw(cx);
    assert_eq!(bounds(cx, scroll).size.height, px(26.).min(initial));
    cx.update(|window, cx| {
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| state.open(window, cx));
    });
    draw(cx);
    cx.update(|window, cx| {
        assert!(
            view.read(cx)
                .state
                .read(cx)
                .input_focus(cx)
                .is_focused(window)
        );
        assert!(
            find(window, &[], &view_child("input-region"))
                .unwrap()
                .visible()
        );
    });
}

#[gpui::test]
fn search_bar_track_focus_and_view_keyboard_navigation(cx: &mut TestAppContext) {
    use gpui::{
        EntityInputHandler as _, FocusHandle, InputEvent as _, KeyDownEvent, KeyUpEvent, Keystroke,
    };

    struct NavSearch {
        custom_state: Entity<SearchState>,
        default_state: Entity<SearchState>,
        custom_trigger_focus: FocusHandle,
    }

    impl Render for NavSearch {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    SearchBar::new("nav-custom", "Custom focus search", &self.custom_state)
                        .track_focus(&self.custom_trigger_focus),
                )
                .child(SearchBar::new(
                    "nav-default",
                    "Default focus search",
                    &self.default_state,
                ))
        }
    }

    cx.update(crate::init_gallery);
    let (mut custom_state, mut default_state) = (None, None);
    let mut custom_focus = None;
    let (_, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        let cs = cx.new(|cx| SearchState::new(window, cx));
        let ds = cx.new(|cx| SearchState::new(window, cx));
        let cf = cx.focus_handle();
        custom_state = Some(cs.clone());
        default_state = Some(ds.clone());
        custom_focus = Some(cf.clone());
        let entity = cx.new(|cx| {
            cx.observe(&cs, |_, _, cx| cx.notify()).detach();
            cx.observe(&ds, |_, _, cx| cx.notify()).detach();
            NavSearch {
                custom_state: cs,
                default_state: ds,
                custom_trigger_focus: cf,
            }
        });
        gpui_base::Root::new(entity, window, cx)
    });
    let custom_state = custom_state.unwrap();
    let default_state = default_state.unwrap();
    let custom_focus = custom_focus.unwrap();
    let (pending_tx, pending_rx) = std::sync::mpsc::channel();
    cx.update(|_, cx| {
        cx.subscribe(&custom_state, move |_, event, _| {
            if let SearchEvent::Request(req) = event {
                let _ = pending_tx.send(req.clone());
            }
        })
        .detach();
    });
    draw(cx);

    // Default SearchState exposes trigger_focus(), and SearchBar::track_focus binds caller focus.
    cx.update(|window, cx| {
        default_state
            .read(cx)
            .trigger_focus()
            .clone()
            .focus(window, cx);
    });
    draw(cx);
    cx.update(|window, _| {
        assert_eq!(
            find(
                window,
                &[],
                &(ElementId::from("nav-default"), "search-trigger").into(),
            )
            .unwrap()
            .focused(),
            Some(true)
        );
    });
    cx.update(|window, cx| {
        custom_focus.focus(window, cx);
    });
    draw(cx);
    cx.update(|window, _| {
        assert_eq!(
            find(
                window,
                &[],
                &(ElementId::from("nav-custom"), "search-trigger").into(),
            )
            .unwrap()
            .focused(),
            Some(true)
        );
    });

    // Open custom search and populate 5 results via pending request.
    cx.update(|window, cx| {
        custom_state.update(cx, |state, cx| {
            state.open(window, cx);
            state.set_query("items", window, cx);
        });
    });
    let req = pending_rx.recv().unwrap();
    cx.update(|_, cx| {
        custom_state.update(cx, |state, cx| {
            assert!(
                state.complete(
                    &req,
                    SearchOutcome::Results(
                        (0_usize..5)
                            .map(|i| SearchResult::new(("res", i), format!("Result {i}")))
                            .collect(),
                    ),
                    cx,
                )
            );
        });
    });
    draw(cx);

    // Active IME composition ignores Down in search input.
    cx.update(|window, cx| {
        let editor = custom_state.read(cx).editor().clone();
        editor.update(cx, |ed, cx| {
            ed.replace_and_mark_text_in_range(None, "n", Some(1..1), window, cx);
        });
    });
    cx.simulate_keystrokes("down");
    draw(cx);
    cx.update(|window, cx| {
        assert!(custom_state.read(cx).input_focus(cx).is_focused(window));
        let editor = custom_state.read(cx).editor().clone();
        editor.update(cx, |ed, cx| {
            ed.unmark_text(window, cx);
        });
    });
    let req_after_ime = pending_rx.recv().unwrap();
    cx.update(|_, cx| {
        custom_state.update(cx, |state, cx| {
            assert!(
                state.complete(
                    &req_after_ime,
                    SearchOutcome::Results(
                        (0_usize..5)
                            .map(|i| SearchResult::new(("res", i), format!("Result {i}")))
                            .collect(),
                    ),
                    cx,
                )
            );
        });
    });
    draw(cx);

    // Pressing Down from unfocused-IME search input focuses first result ("res", 0).
    let press = |cx: &mut VisualTestContext, key: &str| {
        cx.update(|window, cx| {
            let ks = Keystroke::parse_with_os(key, cx.platform_info().os).unwrap();
            window.dispatch_event(
                KeyDownEvent {
                    keystroke: ks.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(KeyUpEvent { keystroke: ks }.to_platform_input(), cx);
        });
        draw(cx);
    };
    let row_activate_id =
        |index: usize| ElementId::from((ElementId::from(("res", index)), "activate"));
    press(cx, "down");
    cx.update(|window, _| {
        assert_eq!(
            find(window, &[], &row_activate_id(0)).unwrap().focused(),
            Some(true)
        );
    });

    // End jumps to last result ("res", 4); Home jumps back to first ("res", 0).
    press(cx, "end");
    cx.update(|window, _| {
        assert_eq!(
            find(window, &[], &row_activate_id(4)).unwrap().focused(),
            Some(true)
        );
    });

    press(cx, "home");
    cx.update(|window, _| {
        assert_eq!(
            find(window, &[], &row_activate_id(0)).unwrap().focused(),
            Some(true)
        );
    });
}
