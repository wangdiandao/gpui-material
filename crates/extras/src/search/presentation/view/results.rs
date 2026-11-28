use std::{rc::Rc, time::Duration};

use gpui::{
    AnyElement, App, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, Role,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_components::{Icon, ListItem, LoadingIndicator, Text};
use gpui_material_foundation::current_material_strings;
use gpui_material_theme::TypeRole;

use super::SearchView;
use crate::search::{Results, SearchOutcome, SearchResult, SearchState};

pub(super) type RowBuilder = Rc<dyn Fn(ListItem, &SearchResult, &mut Window, &mut App) -> ListItem>;
pub(super) type ResultBuilder =
    Box<dyn FnOnce(&Entity<SearchState>, &mut Window, &mut App) -> AnyElement>;

impl SearchView {
    pub(super) fn render_results_region(
        &mut self,
        duration: Duration,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement + use<> {
        let mut results = div()
            .id((self.id.clone(), "results"))
            .test_support()
            .occlude()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .children(
                self.style
                    .divider
                    .map(|color| div().flex_none().h(px(1.)).bg(color)),
            )
            .children(self.filters.take().map(|filters| {
                div()
                    .id((self.id.clone(), "filters"))
                    .flex_none()
                    .child(filters)
            }));
        if let Some(builder) = self.result_content.take() {
            results = results.child(builder(&self.state, window, cx));
        } else {
            let strings = current_material_strings(window, cx);
            let prompt = self
                .prompt
                .clone()
                .unwrap_or_else(|| SharedString::from(strings.search_prompt.clone()));
            let loading_label = self
                .loading_label
                .clone()
                .unwrap_or_else(|| SharedString::from(strings.searching.clone()));
            let empty_label = self
                .empty_label
                .clone()
                .unwrap_or_else(|| SharedString::from(strings.no_results.clone()));
            drop(strings);
            let status = status_info(self.state.read(cx), &prompt, &loading_label, &empty_label);
            if let Some((message, loading, error)) = status {
                results = results.child(search_status(&self.id, message, loading, error));
            } else {
                results = results.child(provider_results(
                    &self.id,
                    &self.state,
                    self.row.take(),
                    duration,
                    window,
                    cx,
                ));
            }
        }
        results
    }
}

fn status_info(
    model: &SearchState,
    prompt: &SharedString,
    loading: &SharedString,
    empty: &SharedString,
) -> Option<(SharedString, bool, bool)> {
    match &model.response.results {
        Results::Idle => Some((prompt.clone(), false, false)),
        Results::Loading => Some((loading.clone(), true, false)),
        Results::Ready(SearchOutcome::Error(error)) => Some((error.clone(), false, true)),
        Results::Ready(SearchOutcome::Results(results)) if results.is_empty() => {
            Some((empty.clone(), false, false))
        }
        Results::Ready(SearchOutcome::Results(_)) => None,
    }
}

fn search_status(
    id: &ElementId,
    message: SharedString,
    loading: bool,
    error: bool,
) -> impl IntoElement + use<> {
    let loading_id = ElementId::from((id.clone(), "search-loading"));
    div()
        .id((id.clone(), "search-status"))
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
            status.child(LoadingIndicator::new(loading_id, message.clone()))
        })
        .child(Text::new(message).role(TypeRole::BodyMedium))
}

fn provider_results(
    id: &ElementId,
    state: &Entity<SearchState>,
    custom_row: Option<RowBuilder>,
    duration: Duration,
    window: &Window,
    cx: &App,
) -> impl IntoElement + use<> {
    let model = state.read(cx);
    let scroll = model.scroll.clone();
    let results_scope = model.results_scope.clone();
    let entry = scroll.logical_scroll_top().item_ix;
    let roving = results_scope.contains_focused(window, cx)
        || model
            .active_result
            .as_ref()
            .and_then(|id| model.result_focus.get(id))
            .is_some_and(|focus| focus.is_focused(window));
    let in_overlay = model.presented();
    let keys = state.clone();
    let row_state = state.clone();
    let view_id = id.clone();
    div()
        .id((id.clone(), "result-list"))
        .role(Role::List)
        .track_focus(&results_scope)
        .flex_1()
        .min_h_0()
        .overflow_hidden()
        .on_key_down(move |event, window, cx| {
            if event.keystroke.modifiers.modified() {
                return;
            }
            let (next_id, focus) = {
                let model = keys.read(cx);
                let rows = model.rows();
                if rows.is_empty() {
                    return;
                }
                let Some(current) = model
                    .active_result
                    .as_ref()
                    .and_then(|id| {
                        model
                            .result_focus
                            .get(id)
                            .filter(|focus| focus.is_focused(window))
                            .and_then(|_| rows.iter().position(|row| &row.id == id))
                    })
                    .or_else(|| {
                        rows.iter()
                            .position(|row| model.result_focus[&row.id].is_focused(window))
                    })
                else {
                    return;
                };
                let next = match event.keystroke.key.as_str() {
                    "up" => (current + rows.len() - 1) % rows.len(),
                    "down" => (current + 1) % rows.len(),
                    "home" => 0,
                    "end" => rows.len() - 1,
                    _ => return,
                };
                let next_id = rows[next].id.clone();
                let focus = model.result_focus[&next_id].clone();
                if model.scroll.bounds_for_item(next).is_some() {
                    model.scroll.scroll_to_reveal_item(next);
                } else {
                    model.scroll.scroll_to(gpui::ListOffset {
                        item_ix: next,
                        offset_in_item: px(0.),
                    });
                }
                (next_id, focus)
            };
            keys.update(cx, |state, _| {
                state.active_result = Some(next_id);
            });
            focus.focus(window, cx);
            window.refresh();
            cx.stop_propagation();
        })
        .child(
            gpui::list(scroll, move |index, window, cx| {
                let model = row_state.read(cx);
                let Some(result) = model.rows().get(index).cloned() else {
                    return div().into_any_element();
                };
                let focus = model.result_focus[&result.id].clone();
                let group = result.group.clone().filter(|group| {
                    index == 0 || model.rows()[index - 1].group.as_ref() != Some(group)
                });
                let tab_stop = focus.is_focused(window) || index == entry && !roving;
                let row = ListItem::new(result.id.clone(), result.label.clone())
                    .when_some(result.supporting.clone(), ListItem::supporting)
                    .when_some(result.icon, |item, icon| {
                        item.leading(Icon::new(icon).size(px(24.)))
                    });
                let row = match &custom_row {
                    Some(builder) => builder(row, &result, window, cx),
                    None => row,
                };
                let state = row_state.clone();
                let cell_id = (
                    ElementId::from((view_id.clone(), "search-result-cell")),
                    SharedString::from(index.to_string()),
                );
                let group_title_id = (view_id.clone(), "search-group-title");
                let row =
                    row.track_focus(&focus)
                        .tab_stop(tab_stop)
                        .on_activate(move |window, cx| {
                            state.update(cx, |state, cx| {
                                if !in_overlay || state.is_open() {
                                    state.select_with_motion(result.clone(), duration, window, cx);
                                }
                            });
                        });
                div()
                    .id(cell_id)
                    .children(group.map(|group| {
                        div()
                            .id(group_title_id)
                            .test_support()
                            .role(Role::Heading)
                            .aria_label(group.clone())
                            .px_4()
                            .pt_3()
                            .child(Text::new(group).role(TypeRole::LabelMedium))
                    }))
                    .child(row)
                    .into_any_element()
            })
            .size_full(),
        )
}
