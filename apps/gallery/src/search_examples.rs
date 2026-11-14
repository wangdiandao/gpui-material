use std::time::Duration;

use gpui::{
    AppContext, Context, Entity, ParentElement, Render, SharedString, Subscription, Window,
};
use gpui_material::{
    AppBar, SearchBar, SearchEvent, SearchOutcome, SearchResult, SearchState, SearchViewVariant,
    Text, theme::TypeRole,
};

use crate::story::{section, stack};

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
        .map(|(id, label, category)| SearchResult::new(id, label).supporting(category))
        .collect(),
    )
}

impl Render for SearchExamples {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        stack()
            .child(section("Docked").child(SearchBar::new(
                "docked",
                "Search documents",
                &self.docked,
            )))
            .child(
                section("Full-screen app bar").child(
                    AppBar::new("search-app-bar", "Documents").search(
                        SearchBar::new("fullscreen", "Search documents", &self.fullscreen)
                            .variant(SearchViewVariant::FullScreen),
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
}
