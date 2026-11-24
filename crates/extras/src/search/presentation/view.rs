use std::{rc::Rc, time::Duration};

use gpui::{
    AnyElement, App, ElementId, Entity, EntityInputHandler, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, Styled, Window, div, prelude::FluentBuilder,
    px,
};
use gpui_base::TestSupportExt;
use gpui_material_components::{Icon, IconButton, IconName, ListItem, TextField, TextFieldState};
use gpui_material_foundation::{Surface, current_material_theme};
use gpui_material_theme::{ColorRole, CornerShape, Direction, Elevation};

use super::{
    SearchMotion, SearchPopupOptions, SearchStyle, SearchViewVariant, SearchWindowOptions, length,
    valid_dimension,
};
use crate::search::{SearchResult, SearchState, SearchValue};

mod results;

use results::{ResultBuilder, RowBuilder};

type InputBuilder = Box<dyn FnOnce(TextField) -> AnyElement>;

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
    pub(super) sampled_in_bar: bool,
}

impl SearchView {
    /// Creates an expanded search view bound to `state` with default English status labels.
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
            sampled_in_bar: false,
        }
    }

    /// Sets the presentation variant (`Docked`, `FullScreen`, or `FullScreenContained`).
    pub fn variant(mut self, variant: SearchViewVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Overrides the close-button label and the loading, empty-result, and idle prompt messages.
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

    /// Places a filter bar or chip row directly above the search results list.
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

    /// Sets the maximum width of the search input editor in logical pixels.
    pub fn focused_width(mut self, width: Pixels) -> Self {
        self.focused_width = valid_dimension(width);
        self
    }

    /// Sets the preferred width and height of the docked popup panel. Defaults to `440px x 424px`.
    pub fn docked_size(mut self, width: Pixels, height: Pixels) -> Self {
        self.docked_dimensions = (
            valid_dimension(width).unwrap_or(px(440.)),
            valid_dimension(height).unwrap_or(px(424.)),
        );
        self
    }

    /// Adds a vertical gap between the input header and results surface in [`SearchViewVariant::Docked`].
    pub fn result_gap(mut self, gap: Pixels) -> Self {
        self.gap = length(gap);
        self
    }

    /// Applies visual style overrides to the search view surfaces and editor.
    pub fn style(mut self, style: SearchStyle) -> Self {
        self.style = style;
        self
    }

    /// Configures expand and collapse transition durations.
    pub fn motion(mut self, motion: SearchMotion) -> Self {
        self.motion = motion;
        self
    }

    /// Configures popup anchoring, dismissal, focus trapping, and scrim for [`SearchViewVariant::Docked`].
    pub fn popup_options(mut self, options: SearchPopupOptions) -> Self {
        self.popup = options;
        self
    }

    /// Configures overlay insets, dismissal, and scrim for full-screen variants.
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
    ) -> impl IntoElement + use<> {
        let close = self.state.clone();
        let down_state = self.state.clone();
        let down_editor = editor.clone();
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
                    .capture_key_down(move |event, window, cx| {
                        if event.keystroke.key != "down" || event.keystroke.modifiers.modified() {
                            return;
                        }
                        if down_editor.update(cx, |input, cx| {
                            input.marked_text_range(window, cx).is_some()
                        }) {
                            return;
                        }
                        let target = {
                            let model = down_state.read(cx);
                            model.rows().first().and_then(|row| {
                                model
                                    .result_focus
                                    .get(&row.id)
                                    .cloned()
                                    .map(|focus| (row.id.clone(), focus, model.scroll.clone()))
                            })
                        };
                        if let Some((first_id, focus, scroll)) = target {
                            if scroll.bounds_for_item(0).is_some() {
                                scroll.scroll_to_reveal_item(0);
                            } else {
                                scroll.scroll_to(gpui::ListOffset {
                                    item_ix: 0,
                                    offset_in_item: px(0.),
                                });
                            }
                            down_state.update(cx, |state, _| {
                                state.active_result = Some(first_id);
                            });
                            focus.focus(window, cx);
                            window.refresh();
                            cx.stop_propagation();
                        }
                    })
                    .child(field),
            )
    }
}

impl RenderOnce for SearchView {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let rem_size = window.rem_size();
        let font_revision = gpui_material_foundation::font_cache_revision(cx);
        let (needs_animation, needs_layout) = {
            let state = self.state.read(cx);
            let needs_layout = match &state.result_layout {
                Some((typography, cached_rem, cached_rev)) => {
                    *cached_rem != rem_size
                        || *cached_rev != font_revision
                        || *typography != theme.typography
                }
                None => true,
            };
            (!self.sampled_in_bar && state.is_animating(), needs_layout)
        };
        let animate = if needs_animation || needs_layout {
            let measurement =
                needs_layout.then(|| (theme.typography.clone(), rem_size, font_revision));
            self.state.update(cx, |state, cx| {
                let animate = if needs_animation {
                    state.sample_animation(window, cx)
                } else {
                    false
                };
                if let Some(measurement) = measurement {
                    state.result_layout = Some(measurement);
                    state.invalidate_result_measurements();
                }
                animate
            })
        } else {
            false
        };
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
        let layout = theme.layout;
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
