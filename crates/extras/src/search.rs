use std::collections::HashMap;

use gpui::{
    Anchor, App, AppContext, Context, ElementId, Entity, EntityInputHandler, EventEmitter,
    FocusHandle, Focusable, InteractiveElement, IntoElement, ListAlignment, ListState,
    ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    Subscription, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Dialog, Popup, TestSupportExt};
use gpui_material_components::{
    Button, ButtonVariant, Icon, IconButton, IconName, ListItem, LoadingIndicator, Text, TextField,
    TextFieldEvent, TextFieldState,
};
use gpui_material_foundation::{Surface, logical_anchor, scrim_color, window_material_theme};
use gpui_material_theme::{ColorRole, Direction, Elevation, TypeRole};

/// Identifies one committed query. Complete requests through [SearchState::complete].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchRequest {
    revision: u64,
    query: SharedString,
}

impl SearchRequest {
    pub fn query(&self) -> &SharedString {
        &self.query
    }
}

/// A result row whose ID remains stable when results reorder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    id: ElementId,
    label: SharedString,
    supporting: Option<SharedString>,
}

impl SearchResult {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            supporting: None,
        }
    }

    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub fn label(&self) -> &SharedString {
        &self.label
    }
}

/// Result from the application's search provider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchOutcome {
    Results(Vec<SearchResult>),
    Error(SharedString),
}

/// Search providers subscribe to requests. Submissions and result selection
/// are separate commands and the application owns their consequences.
#[derive(Clone, Debug)]
pub enum SearchEvent {
    Request(SearchRequest),
    Submit(SharedString),
    Select(SearchResult),
}

enum Results {
    Idle,
    Loading,
    Ready(SearchOutcome),
}

struct SearchResponse {
    revision: u64,
    pending: Option<SearchRequest>,
    results: Results,
}

impl Default for SearchResponse {
    fn default() -> Self {
        Self {
            revision: 0,
            pending: None,
            results: Results::Idle,
        }
    }
}

impl SearchResponse {
    fn begin(&mut self, query: SharedString) -> Option<SearchRequest> {
        self.revision = self
            .revision
            .checked_add(1)
            .expect("search revision exhausted");
        self.pending = (!query.trim().is_empty()).then_some(SearchRequest {
            revision: self.revision,
            query,
        });
        self.results = if self.pending.is_some() {
            Results::Loading
        } else {
            Results::Idle
        };
        self.pending.clone()
    }

    fn finish(&mut self, request: &SearchRequest, outcome: SearchOutcome) -> bool {
        if self.pending.as_ref() != Some(request) {
            return false;
        }
        self.pending = None;
        self.results = Results::Ready(outcome);
        true
    }
}

/// Retains editing, IME, result and focus state while the search view expands.
/// The application performs asynchronous work and owns resulting navigation.
pub struct SearchState {
    input: Entity<TextFieldState>,
    trigger: FocusHandle,
    scope: FocusHandle,
    restore: Option<FocusHandle>,
    scroll: ListState,
    result_focus: HashMap<ElementId, FocusHandle>,
    result_layout: Option<(gpui_material_theme::Typography, gpui::Pixels)>,
    query: SharedString,
    response: SearchResponse,
    open: bool,
    _input: Subscription,
}

impl EventEmitter<SearchEvent> for SearchState {}

impl SearchState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextFieldState::new(window, cx));
        let subscription = cx.subscribe_in(&input, window, |state, input, event, window, cx| {
            if !state.open
                || !matches!(
                    event,
                    TextFieldEvent::Change
                        | TextFieldEvent::PressEnter {
                            secondary: false,
                            shift: false
                        }
                )
            {
                return;
            }
            if input.update(cx, |input, cx| {
                input.marked_text_range(window, cx).is_some()
            }) {
                return;
            }
            let query = input.read(cx).value();
            state.update_query(query.clone(), cx);
            if matches!(event, TextFieldEvent::PressEnter { .. }) && !query.trim().is_empty() {
                cx.emit(SearchEvent::Submit(query));
            }
        });
        Self {
            input,
            trigger: cx.focus_handle(),
            scope: cx.focus_handle(),
            restore: None,
            scroll: ListState::new(0, ListAlignment::Top, px(100.)),
            result_focus: HashMap::new(),
            result_layout: None,
            query: "".into(),
            response: SearchResponse::default(),
            open: false,
            _input: subscription,
        }
    }

    pub fn query(&self) -> &SharedString {
        &self.query
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Returns the stable focus handle of the expanded editor.
    pub fn input_focus(&self, cx: &App) -> FocusHandle {
        self.input.read(cx).focus_handle(cx)
    }

    /// Opens the view and focuses its editor.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            return;
        }
        self.restore = window.focused(cx).or_else(|| Some(self.trigger.clone()));
        self.open = true;
        self.input_focus(cx).focus(window, cx);
        cx.notify();
    }

    /// Dismisses the view and restores its entry focus.
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        self.open = false;
        if let Some(focus) = self.restore.take() {
            focus.focus(window, cx);
        }
        cx.notify();
    }

    /// Synchronizes the query from the application. Query changes issue a
    /// request, followed by the normal completion path.
    pub fn set_query(
        &mut self,
        query: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let query = query.into();
        self.input.update(cx, |input, cx| {
            if input.value() != query {
                input.set_value(query.clone(), window, cx);
            }
        });
        self.update_query(query, cx);
    }

    /// Accepts only the latest pending request. Returns false for superseded
    /// requests and duplicate completions.
    pub fn complete(
        &mut self,
        request: &SearchRequest,
        outcome: SearchOutcome,
        cx: &mut Context<Self>,
    ) -> bool {
        let accepted = self.response.finish(request, outcome);
        if accepted {
            self.sync_result_focus(cx);
            cx.notify();
        }
        accepted
    }

    fn update_query(&mut self, query: SharedString, cx: &mut Context<Self>) {
        if query == self.query {
            return;
        }
        self.query = query.clone();
        if let Some(request) = self.response.begin(query) {
            cx.emit(SearchEvent::Request(request));
        }
        self.scroll.reset(0);
        cx.notify();
    }

    fn rows(&self) -> &[SearchResult] {
        match &self.response.results {
            Results::Ready(SearchOutcome::Results(rows)) => rows,
            _ => &[],
        }
    }

    fn sync_result_focus(&mut self, cx: &App) {
        let mut previous = std::mem::take(&mut self.result_focus);
        self.result_focus = self
            .rows()
            .iter()
            .map(|row| {
                let focus = previous
                    .remove(&row.id)
                    .unwrap_or_else(|| cx.focus_handle());
                (row.id.clone(), focus)
            })
            .collect();
        self.invalidate_result_measurements();
    }

    fn invalidate_result_measurements(&self) {
        let anchor = self.scroll.logical_scroll_top();
        // Focus metadata keeps the focused row mounted when it scrolls out of
        // view; keyboard navigation still addresses the complete result model.
        self.scroll.splice_focusable(
            0..self.scroll.item_count(),
            self.rows()
                .iter()
                .map(|row| self.result_focus.get(&row.id).cloned()),
        );
        self.scroll.scroll_to(anchor);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchViewVariant {
    #[default]
    Docked,
    FullScreen,
}

/// Collapsed Material search trigger. It can occupy the [AppBar](gpui_material_components::AppBar)
/// search slot or be placed directly in a desktop layout.
#[derive(IntoElement)]
pub struct SearchBar {
    id: ElementId,
    label: SharedString,
    state: Entity<SearchState>,
    variant: SearchViewVariant,
    close_label: SharedString,
    loading_label: SharedString,
    empty_label: SharedString,
    prompt: SharedString,
}

impl SearchBar {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<SearchState>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: state.clone(),
            variant: SearchViewVariant::default(),
            close_label: "Close search".into(),
            loading_label: "Searching".into(),
            empty_label: "No results".into(),
            prompt: "Enter a search term".into(),
        }
    }

    pub fn variant(mut self, variant: SearchViewVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Overrides the localized labels used inside the view.
    pub fn labels(
        mut self,
        close: impl Into<SharedString>,
        loading: impl Into<SharedString>,
        empty: impl Into<SharedString>,
        prompt: impl Into<SharedString>,
    ) -> Self {
        self.close_label = close.into();
        self.loading_label = loading.into();
        self.empty_label = empty.into();
        self.prompt = prompt.into();
        self
    }
}

impl RenderOnce for SearchBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let collapsed = self.state.read(cx);
        let trigger = collapsed.trigger.clone();
        let scope = collapsed.scope.clone();
        let open = collapsed.open;
        let label = if collapsed.query.is_empty() {
            self.label.clone()
        } else {
            collapsed.query.clone()
        };
        let state = self.state.clone();
        let focus = trigger.clone();
        let button = Button::new((self.id.clone(), "search-trigger"), label)
            .track_focus(&trigger)
            .size(gpui_material_components::ControlSize::Medium)
            .variant(ButtonVariant::FilledTonal)
            .leading_icon(Icon::new(IconName::Search))
            .on_activate(move |window, cx| {
                focus.focus(window, cx);
                state.update(cx, |state, cx| state.open(window, cx));
            });
        if !open {
            return button.into_any_element();
        }
        let theme = window_material_theme(window, cx);
        let direction = theme.layout.direction;
        let scrim = scrim_color(theme, 0.32);
        let state = self.state.clone();
        let view = SearchView::new((self.id.clone(), "search-view"), self.label.clone(), &state)
            .variant(self.variant)
            .labels(
                self.close_label,
                self.loading_label,
                self.empty_label,
                self.prompt,
            );
        match self.variant {
            SearchViewVariant::Docked => Popup::new(self.id, button)
                .anchor(logical_anchor(Anchor::TopLeft, direction))
                .offset(px(4.))
                .content(
                    div()
                        .id("docked-search")
                        .role(Role::Dialog)
                        .aria_label(self.label)
                        .track_focus(&scope)
                        .occlude()
                        .on_mouse_down_out(move |_, window, cx| {
                            state.update(cx, |state, cx| state.close(window, cx));
                        })
                        .child(view),
                )
                .into_any_element(),
            SearchViewVariant::FullScreen => div()
                .child(button)
                .child(
                    Dialog::new(cx)
                        .accessibility_label(self.label)
                        .focus_handle(scope)
                        .backdrop(div().absolute().inset_0().bg(scrim))
                        .on_ok(|_, _, _| false)
                        .on_cancel(move |_, window, cx| {
                            dismiss_search(&state, window, cx);
                            false
                        })
                        .popup(view),
                )
                .into_any_element(),
        }
    }
}

/// Expanded Material search editor and scrollable result region.
#[derive(IntoElement)]
pub struct SearchView {
    id: ElementId,
    label: SharedString,
    state: Entity<SearchState>,
    variant: SearchViewVariant,
    close_label: SharedString,
    loading_label: SharedString,
    empty_label: SharedString,
    prompt: SharedString,
}

impl SearchView {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<SearchState>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: state.clone(),
            variant: SearchViewVariant::default(),
            close_label: "Close search".into(),
            loading_label: "Searching".into(),
            empty_label: "No results".into(),
            prompt: "Enter a search term".into(),
        }
    }

    pub fn variant(mut self, variant: SearchViewVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn labels(
        mut self,
        close: SharedString,
        loading: SharedString,
        empty: SharedString,
        prompt: SharedString,
    ) -> Self {
        self.close_label = close;
        self.loading_label = loading;
        self.empty_label = empty;
        self.prompt = prompt;
        self
    }
}

impl RenderOnce for SearchView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let layout = (
            window_material_theme(window, cx).typography.clone(),
            window.rem_size(),
        );
        self.state.update(cx, |state, _| {
            if state.result_layout.as_ref() != Some(&layout) {
                state.result_layout = Some(layout);
                state.invalidate_result_measurements();
            }
        });
        let model = self.state.read(cx);
        let input = model.input.clone();
        let status = match &model.response.results {
            Results::Idle => Some((self.prompt, false, false)),
            Results::Loading => Some((self.loading_label, true, false)),
            Results::Ready(SearchOutcome::Error(error)) => Some((error.clone(), false, true)),
            Results::Ready(SearchOutcome::Results(results)) if results.is_empty() => {
                Some((self.empty_label, false, false))
            }
            Results::Ready(SearchOutcome::Results(_)) => None,
        };
        let size = window.viewport_size();
        let direction = window_material_theme(window, cx).layout.direction;
        let fullscreen = self.variant == SearchViewVariant::FullScreen;
        let close = self.state.clone();
        let dismiss = self.state.clone();
        let mut results = div()
            .id((self.id.clone(), "results"))
            .occlude()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_hidden();
        if let Some((message, loading, error)) = status {
            results = results.child(
                div()
                    .id((self.id.clone(), "search-status"))
                    .test_support()
                    .role(if error { Role::Alert } else { Role::Status })
                    .aria_label(message.clone())
                    .a11y_synthetic_children(move |tree| {
                        let node = tree.parent_node();
                        node.set_live(if error {
                            gpui::accesskit::Live::Assertive
                        } else {
                            gpui::accesskit::Live::Polite
                        });
                        node.set_live_atomic();
                    })
                    .p_4()
                    .flex()
                    .items_center()
                    .gap_3()
                    .when(loading, |status| {
                        status.child(LoadingIndicator::new("search-loading", message.clone()))
                    })
                    .child(Text::new(message).role(TypeRole::BodyMedium)),
            );
        }
        let scroll = model.scroll.clone();
        let entry = scroll.logical_scroll_top().item_ix;
        let row_state = self.state.clone();
        let keys = self.state.clone();
        if !model.rows().is_empty() {
            results = results.child(
                div()
                    .id((self.id.clone(), "result-list"))
                    .role(Role::List)
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .on_key_down(move |event, window, cx| {
                        let model = keys.read(cx);
                        let rows = model.rows();
                        let Some(current) = rows
                            .iter()
                            .position(|row| model.result_focus[&row.id].is_focused(window))
                        else {
                            return;
                        };
                        let next = match event.keystroke.key.as_str() {
                            "up" => (current + rows.len() - 1) % rows.len(),
                            "down" => (current + 1) % rows.len(),
                            _ => return,
                        };
                        let focus = model.result_focus[&rows[next].id].clone();
                        if model.scroll.bounds_for_item(next).is_some() {
                            model.scroll.scroll_to_reveal_item(next);
                        } else {
                            // Unmeasured rows have no physical offset yet. A
                            // logical anchor reveals them without measuring the
                            // entire prefix merely to navigate to the last row.
                            model.scroll.scroll_to(gpui::ListOffset {
                                item_ix: next,
                                offset_in_item: px(0.),
                            });
                        }
                        focus.focus(window, cx);
                        window.refresh();
                        cx.stop_propagation();
                    })
                    .child(
                        gpui::list(scroll, move |ix, window, cx| {
                            let model = row_state.read(cx);
                            let result = model.rows()[ix].clone();
                            let focus = model.result_focus[&result.id].clone();
                            let tab_stop = focus.is_focused(window)
                                || ix == entry
                                    && !model
                                        .result_focus
                                        .values()
                                        .any(|handle| handle.is_focused(window));
                            let state = row_state.clone();
                            ListItem::new(result.id.clone(), result.label.clone())
                                .track_focus(&focus, tab_stop)
                                .when_some(result.supporting.clone(), ListItem::supporting_text)
                                .on_activate(move |window, cx| {
                                    state.update(cx, |state, cx| {
                                        state.close(window, cx);
                                        cx.emit(SearchEvent::Select(result.clone()));
                                    });
                                })
                                .into_any_element()
                        })
                        .size_full(),
                    ),
            );
        }
        div()
            .id(self.id.clone())
            .test_support()
            .capture_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape" && dismiss_search(&dismiss, window, cx) {
                    cx.stop_propagation();
                }
            })
            .child(
                Surface::new(ColorRole::SurfaceContainerHigh)
                    .flex()
                    .flex_col()
                    .w(if fullscreen {
                        size.width
                    } else {
                        px(440.).min(size.width - px(16.))
                    })
                    .h(if fullscreen {
                        size.height
                    } else {
                        px(424.).min(size.height * 0.65)
                    })
                    .when(!fullscreen, |surface| {
                        surface.rounded(px(28.)).shadow_elevation(Elevation::Level3)
                    })
                    .child(
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .p_2()
                            .gap_2()
                            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
                            .child(
                                IconButton::new(
                                    (self.id.clone(), "close-search"),
                                    self.close_label,
                                    Icon::new(IconName::Close),
                                )
                                .on_activate(move |window, cx| {
                                    close.update(cx, |state, cx| state.close(window, cx));
                                }),
                            )
                            .child(div().flex_1().min_w_0().child(TextField::new(
                                (self.id.clone(), "editor"),
                                self.label,
                                &input,
                            ))),
                    )
                    .child(results),
            )
    }
}

fn dismiss_search(state: &Entity<SearchState>, window: &mut Window, cx: &mut App) -> bool {
    state.update(cx, |state, cx| {
        let composing = state.input.update(cx, |input, cx| {
            input.marked_text_range(window, cx).is_some()
        });
        if !composing {
            state.close(window, cx);
        }
        !composing
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_revision_rejects_stale_duplicate_and_cleared_results() {
        let mut state = SearchResponse::default();
        let first = state.begin("alpha".into()).unwrap();
        let second = state.begin("beta".into()).unwrap();
        assert_eq!(second.query(), "beta");
        assert!(!state.finish(&first, SearchOutcome::Error("stale".into())));
        assert!(state.finish(&second, SearchOutcome::Results(vec![])));
        assert!(!state.finish(&second, SearchOutcome::Error("duplicate".into())));
        let third = state.begin("alpha".into()).unwrap();
        assert!(state.begin(" ".into()).is_none());
        assert!(!state.finish(&third, SearchOutcome::Error("cleared".into())));
        assert!(matches!(state.results, Results::Idle));
    }
}
