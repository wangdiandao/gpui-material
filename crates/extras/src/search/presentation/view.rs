use std::{rc::Rc, time::Duration};

use gpui::{
    AnyElement, App, ElementId, Entity, EntityInputHandler, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_components::{
    Icon, IconButton, IconName, ListItem, LoadingIndicator, Text, TextField, TextFieldState,
};
use gpui_material_foundation::{Surface, current_material_theme};
use gpui_material_theme::{ColorRole, CornerShape, Direction, Elevation, TypeRole};

use super::{
    SearchMotion, SearchPopupOptions, SearchStyle, SearchViewVariant, SearchWindowOptions, length,
    valid_dimension,
};
use crate::search::{Results, SearchOutcome, SearchResult, SearchState, SearchValue};

type InputBuilder = Box<dyn FnOnce(TextField) -> AnyElement>;
type RowBuilder = Rc<dyn Fn(ListItem, &SearchResult, &mut Window, &mut App) -> ListItem>;
type ResultBuilder = Box<dyn FnOnce(&Entity<SearchState>, &mut Window, &mut App) -> AnyElement>;

/// Expanded Material search editor and result region, also usable as an independent view.
#[derive(IntoElement)]
pub struct SearchView {
    id: ElementId,
    pub(super) label: SharedString,
    pub(super) state: Entity<SearchState>,
    pub(super) variant: SearchViewVariant,
    pub(super) close_label: SharedString,
    loading_label: SharedString,
    empty_label: SharedString,
    prompt: SharedString,
    input: Option<InputBuilder>,
    filters: Option<AnyElement>,
    row: Option<RowBuilder>,
    result_content: Option<ResultBuilder>,
    pub(super) focused_width: Option<Pixels>,
    docked_dimensions: (Pixels, Pixels),
    gap: Pixels,
    pub(super) style: SearchStyle,
    pub(super) motion: SearchMotion,
    pub(super) popup: SearchPopupOptions,
    pub(super) window: SearchWindowOptions,
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
            input: None,
            filters: None,
            row: None,
            result_content: None,
            focused_width: None,
            docked_dimensions: (px(440.), px(424.)),
            gap: px(0.),
            style: SearchStyle::default(),
            motion: SearchMotion::default(),
            popup: SearchPopupOptions::default(),
            window: SearchWindowOptions::default(),
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

    /// Configures the editor. Keep the provided field in the returned content.
    pub fn input<E: IntoElement + 'static>(
        mut self,
        build: impl FnOnce(TextField) -> E + 'static,
    ) -> Self {
        self.input = Some(Box::new(move |field| build(field).into_any_element()));
        self
    }

    pub fn filters(mut self, content: impl IntoElement) -> Self {
        self.filters = Some(content.into_any_element());
        self
    }

    /// Decorates a provider row. Search installs the stable ID's focus and selection after this callback.
    pub fn result_row(
        mut self,
        build: impl Fn(ListItem, &SearchResult, &mut Window, &mut App) -> ListItem + 'static,
    ) -> Self {
        self.row = Some(Rc::new(build));
        self
    }

    /// Takes over the entire result region while still using the SearchState provider.
    pub fn result_content<E: IntoElement + 'static>(
        mut self,
        build: impl FnOnce(&Entity<SearchState>, &mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.result_content = Some(Box::new(move |state, window, cx| {
            build(state, window, cx).into_any_element()
        }));
        self
    }

    pub fn focused_width(mut self, width: Pixels) -> Self {
        self.focused_width = valid_dimension(width);
        self
    }

    pub fn docked_size(mut self, width: Pixels, height: Pixels) -> Self {
        self.docked_dimensions = (
            valid_dimension(width).unwrap_or(px(440.)),
            valid_dimension(height).unwrap_or(px(424.)),
        );
        self
    }

    pub fn result_gap(mut self, gap: Pixels) -> Self {
        self.gap = length(gap);
        self
    }

    pub fn style(mut self, style: SearchStyle) -> Self {
        self.style = style;
        self
    }

    pub fn motion(mut self, motion: SearchMotion) -> Self {
        self.motion = motion;
        self
    }

    pub fn popup_options(mut self, options: SearchPopupOptions) -> Self {
        self.popup = options;
        self
    }

    pub fn window_options(mut self, options: SearchWindowOptions) -> Self {
        self.window = options;
        self
    }

    fn render_input_region(
        &mut self,
        editor: &Entity<TextFieldState>,
        editor_width: Option<Pixels>,
        direction: Direction,
        duration: Duration,
    ) -> gpui::Stateful<gpui::Div> {
        let close = self.state.clone();
        let field = TextField::new((self.id.clone(), "editor"), self.label.clone(), editor)
            .when_some(self.style.input_colors, TextField::colors);
        let field = match self.input.take() {
            Some(builder) => builder(field),
            None => field.into_any_element(),
        };
        div()
            .id((self.id.clone(), "input-region"))
            .test_support()
            .flex()
            .flex_none()
            .items_center()
            .p_2()
            .gap_2()
            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
            .child(
                IconButton::new(
                    (self.id.clone(), "close-search"),
                    self.close_label.clone(),
                    Icon::new(IconName::Close),
                )
                .on_activate(move |window, cx| {
                    dismiss_search(&close, duration, window, cx);
                }),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .when_some(editor_width, |editor, width| editor.max_w(width))
                    .child(field),
            )
    }

    fn render_results_region(
        &mut self,
        duration: Duration,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
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
            .children(
                self.filters
                    .take()
                    .map(|filters| div().id("filters").flex_none().child(filters)),
            );
        if let Some(builder) = self.result_content.take() {
            results = results.child(builder(&self.state, window, cx));
        } else {
            let status = status_info(
                self.state.read(cx),
                &self.prompt,
                &self.loading_label,
                &self.empty_label,
            );
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

impl RenderOnce for SearchView {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let measurement = (
            current_material_theme(window, cx).typography.clone(),
            window.rem_size(),
            gpui_material_foundation::font_cache_revision(cx),
        );
        let animate = self.state.update(cx, |state, cx| {
            let animate = state.sample_animation(window, cx);
            if state.result_layout.as_ref() != Some(&measurement) {
                state.result_layout = Some(measurement);
                state.invalidate_result_measurements();
            }
            animate
        });
        if animate && window.is_visible() {
            window.request_animation_frame();
        }
        let model = self.state.read(cx);
        let progress = if model.presented() {
            model.progress()
        } else {
            1.
        };
        let editor = model.input.clone();
        let layout = current_material_theme(window, cx).layout;
        let direction = layout.direction;
        let size = window.viewport_size();
        let docked = self.variant == SearchViewVariant::Docked;
        let contained = self.variant == SearchViewVariant::FullScreenContained;
        let split = docked && (self.gap > px(0.) || self.style.results_shape.is_some());
        let geometry = self.window.insets;
        let (left, right) = layout.horizontal(length(geometry.left), length(geometry.right));
        let width = if docked {
            self.docked_dimensions
                .0
                .min((size.width - px(16.)).max(px(0.)))
        } else {
            (size.width - left - right).max(px(0.))
        };
        let height = if docked {
            self.docked_dimensions.1.min(size.height * 0.65)
        } else {
            (size.height - length(geometry.top) - length(geometry.bottom)).max(px(0.))
        };
        let editor_limit = if contained {
            (width - px(24.)).max(px(0.))
        } else {
            width
        };
        let editor_width = self
            .focused_width
            .or(contained.then_some(px(720.)))
            .map(|width| width.min(editor_limit));
        let dismiss = self.state.clone();
        let duration = self.motion.collapse;
        let close_on_escape = if docked {
            self.popup.dismiss_on_escape
        } else {
            self.window.dismiss_on_escape
        };

        let input = self.render_input_region(&editor, editor_width, direction, duration);
        let results = self.render_results_region(duration, window, cx);

        let panel = panel_surface(self.style, docked, false).w(width).h(height);
        let panel = if split {
            div()
                .flex()
                .flex_col()
                .w(width)
                .h(height)
                .child(panel_surface(self.style, true, false).child(input))
                .child(div().flex_none().h(self.gap.min(height * 0.35)))
                .child(
                    panel_surface(self.style, true, true)
                        .flex_1()
                        .min_h_0()
                        .child(results),
                )
                .into_any_element()
        } else {
            panel
                .child(if contained {
                    div()
                        .flex()
                        .justify_center()
                        .p_3()
                        .child(
                            panel_surface(self.style, false, false)
                                .shape(self.style.shape.unwrap_or(CornerShape::Full))
                                .w(editor_width.unwrap_or(width))
                                .child(input),
                        )
                        .into_any_element()
                } else {
                    input.into_any_element()
                })
                .child(results)
                .into_any_element()
        };
        div()
            .id(self.id)
            .test_support()
            .when(!docked, |view| {
                view.pl(left)
                    .pr(right)
                    .pt(length(geometry.top))
                    .pb(length(geometry.bottom))
            })
            .opacity(progress)
            .capture_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape"
                    && close_on_escape
                    && dismiss_search(&dismiss, duration, window, cx)
                {
                    cx.stop_propagation();
                }
            })
            .child(panel)
    }
}

fn panel_surface(style: SearchStyle, docked: bool, results: bool) -> Surface {
    Surface::new(ColorRole::SurfaceContainerHigh)
        .flex()
        .flex_col()
        .when_some(style.container, Surface::container_color)
        .when_some(style.content, Surface::content_color)
        .when_some(
            if results {
                style.results_shape.or(style.shape)
            } else {
                style.shape
            }
            .or(docked.then_some(CornerShape::Radii([28.; 4]))),
            Surface::shape,
        )
        .when_some(style.tonal_elevation, Surface::tonal_elevation)
        .when_some(
            style
                .shadow_elevation
                .or(docked.then_some(Elevation::Level3)),
            Surface::shadow_elevation,
        )
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
            status.child(LoadingIndicator::new("search-loading", message.clone()))
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
    let entry = scroll.logical_scroll_top().item_ix;
    let roving = model
        .result_focus
        .values()
        .any(|focus| focus.is_focused(window));
    let in_overlay = model.presented();
    let keys = state.clone();
    let row_state = state.clone();
    div()
        .id((id.clone(), "result-list"))
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
                    .when_some(result.supporting.clone(), ListItem::supporting_text)
                    .when_some(result.icon, |item, icon| {
                        item.leading(Icon::new(icon).size(px(24.)))
                    });
                let row = match &custom_row {
                    Some(builder) => builder(row, &result, window, cx),
                    None => row,
                };
                let state = row_state.clone();
                let cell_id = (result.id.clone(), "search-result-cell");
                let row = row
                    .track_focus(&focus, tab_stop)
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
                            .id("search-group-title")
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

pub(super) fn dismiss_search(
    state: &Entity<SearchState>,
    duration: Duration,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    state.update(cx, |state, cx| {
        let composing = state.input.update(cx, |input, cx| {
            input.marked_text_range(window, cx).is_some()
        });
        if !composing {
            state.animate_to(SearchValue::Collapsed, duration, window, cx);
        }
        !composing
    })
}
