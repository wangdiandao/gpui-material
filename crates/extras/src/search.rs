use std::{collections::HashMap, time::Duration};

use gpui::{
    App, AppContext, Context, ElementId, Entity, EntityInputHandler, EventEmitter, FocusHandle,
    Focusable, ListAlignment, ListState, SharedString, Subscription, Window, px,
};
use gpui_base::motion::Easing;
use gpui_material_components::{IconName, TextFieldEvent, TextFieldState};
use gpui_material_foundation::RetargetTransition;

mod presentation;
mod scroll;

pub use presentation::{
    SearchBar, SearchMotion, SearchPopupOptions, SearchScrim, SearchStyle, SearchView,
    SearchViewVariant, SearchWindowOptions,
};
pub use scroll::SearchScrollState;

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
    group: Option<SharedString>,
    icon: Option<IconName>,
}

impl SearchResult {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            supporting: None,
            group: None,
            icon: None,
        }
    }

    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    /// Groups consecutive rows under this heading.
    pub fn group(mut self, heading: impl Into<SharedString>) -> Self {
        self.group = Some(heading.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn group_label(&self) -> Option<&SharedString> {
        self.group.as_ref()
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SearchValue {
    #[default]
    Collapsed,
    Expanded,
}

impl SearchValue {
    fn progress(self) -> f32 {
        match self {
            Self::Collapsed => 0.,
            Self::Expanded => 1.,
        }
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
    result_layout: Option<(gpui_material_theme::Typography, gpui::Pixels, u64)>,
    query: SharedString,
    response: SearchResponse,
    target: SearchValue,
    progress: f32,
    animation: RetargetTransition,
    _input: Subscription,
}

impl EventEmitter<SearchEvent> for SearchState {}

impl SearchState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextFieldState::new(window, cx));
        Self::with_editor(&input, window, cx)
    }

    /// Uses an editor created for this window while retaining the search provider contract.
    pub fn with_editor(
        input: &Entity<TextFieldState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscription = cx.subscribe_in(input, window, |state, input, event, window, cx| {
            if !state.is_open()
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
            input: input.clone(),
            trigger: cx.focus_handle(),
            scope: cx.focus_handle(),
            restore: None,
            scroll: ListState::new(0, ListAlignment::Top, px(100.)),
            result_focus: HashMap::new(),
            result_layout: None,
            query: "".into(),
            response: SearchResponse::default(),
            target: SearchValue::Collapsed,
            progress: 0.,
            animation: RetargetTransition::settled(0.),
            _input: subscription,
        }
    }

    pub fn query(&self) -> &SharedString {
        &self.query
    }

    pub fn is_open(&self) -> bool {
        self.target == SearchValue::Expanded
    }

    pub fn current_value(&self) -> SearchValue {
        if self.progress <= 0.02 {
            SearchValue::Collapsed
        } else {
            SearchValue::Expanded
        }
    }

    pub fn target_value(&self) -> SearchValue {
        self.target
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn is_animating(&self) -> bool {
        self.animation.is_animating()
    }

    pub fn editor(&self) -> &Entity<TextFieldState> {
        &self.input
    }

    /// Returns the stable focus handle of the expanded editor.
    pub fn input_focus(&self, cx: &App) -> FocusHandle {
        self.input.read(cx).focus_handle(cx)
    }

    /// Opens the view and focuses its editor immediately.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.snap_to(1., window, cx);
    }

    /// Dismisses the view and restores its entry focus immediately.
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.snap_to(0., window, cx);
    }

    /// Positions the presentation at a fraction from collapsed to expanded.
    pub fn snap_to(&mut self, fraction: f32, window: &mut Window, cx: &mut Context<Self>) {
        if !fraction.is_finite() {
            return;
        }
        let fraction = fraction.clamp(0., 1.);
        if fraction == self.progress && !self.animation.is_animating() {
            return;
        }
        if fraction > 0. {
            self.prepare_open(window, cx);
        } else {
            self.target = SearchValue::Collapsed;
            self.restore_focus(window, cx);
        }
        self.animation.snap_to(fraction);
        self.progress = fraction;
        cx.notify();
    }

    /// Animates to an endpoint. Retargeting uses the currently displayed fraction.
    pub fn animate_to(
        &mut self,
        target: SearchValue,
        duration: Duration,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if duration.is_zero() || cx.reduce_motion() {
            self.snap_to(target.progress(), window, cx);
            return;
        }
        self.sample_animation(window, cx);
        if target == self.target
            && (self.animation.is_animating() || self.progress == target.progress())
        {
            return;
        }
        if target == SearchValue::Expanded {
            self.prepare_open(window, cx);
        }
        self.target = target;
        let now = cx.background_executor().now();
        self.animation
            .animate_to(target.progress(), duration, Easing::EaseInOut, now, false);
        self.progress = self.animation.value();
        if !self.animation.is_animating() && target == SearchValue::Collapsed {
            self.restore_focus(window, cx);
        }
        cx.notify();
    }

    pub fn results(&self) -> Option<&SearchOutcome> {
        match &self.response.results {
            Results::Ready(outcome) => Some(outcome),
            _ => None,
        }
    }

    /// Selects a caller-rendered result and dismisses using the retained focus path.
    pub fn select_result(
        &mut self,
        result: SearchResult,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_with_motion(result, Duration::ZERO, window, cx);
    }

    fn select_with_motion(
        &mut self,
        result: SearchResult,
        duration: Duration,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.animate_to(SearchValue::Collapsed, duration, window, cx);
        cx.emit(SearchEvent::Select(result));
    }

    pub fn scroll_state(&self) -> &ListState {
        &self.scroll
    }

    /// Uses a caller-owned virtual result viewport.
    pub fn set_scroll_state(&mut self, scroll: &ListState, cx: &mut Context<Self>) {
        self.scroll = scroll.clone();
        self.invalidate_result_measurements();
        cx.notify();
    }

    fn presented(&self) -> bool {
        self.is_open() || self.animation.is_animating()
    }

    fn prepare_open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_open() {
            return;
        }
        if self.restore.is_none() {
            self.restore = window.focused(cx).or_else(|| Some(self.trigger.clone()));
        }
        self.target = SearchValue::Expanded;
        self.input_focus(cx).focus(window, cx);
        if self.input.update(cx, |input, cx| {
            input.marked_text_range(window, cx).is_none()
        }) {
            let query = self.input.read(cx).value();
            self.update_query(query, cx);
        }
    }

    fn restore_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(focus) = self.restore.take() {
            focus.focus(window, cx);
        }
    }

    fn sample_animation(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if !self.animation.is_animating() {
            return false;
        }
        let now = cx.background_executor().now();
        let old = self.progress;
        let (value, running) = self.animation.sample(now, cx.reduce_motion());
        self.progress = value;
        if !running {
            if self.target == SearchValue::Collapsed {
                self.restore_focus(window, cx);
            }
            cx.notify();
            false
        } else {
            if old != self.progress {
                cx.notify();
            }
            true
        }
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
        let mut anchor = self.scroll.logical_scroll_top();
        anchor.item_ix = anchor.item_ix.min(self.rows().len().saturating_sub(1));
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
