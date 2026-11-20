use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    Anchor, AnyElement, App, Edges, ElementId, Entity, EntityInputHandler, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, Role, ScrollHandle,
    SharedString, StatefulInteractiveElement, Styled, Window, canvas, deferred, div, point,
    prelude::FluentBuilder, px,
};
use gpui_base::{Dialog, FocusTrapElement, Popup, Positioner, TestSupportExt};
use gpui_material_components::{
    ButtonColors, ButtonShapes, ButtonVariant, ControlSize, Icon, IconButton, IconName, ListItem,
    LoadingIndicator, Text, TextField, TextFieldColors, button_composition::ButtonPart,
};
use gpui_material_foundation::{
    Scrim, Surface, ThemeScope, current_material_theme, logical_anchor,
};
use gpui_material_theme::{ColorRole, CornerShape, Direction, Elevation, TypeRole};

use super::{Results, SearchOutcome, SearchResult, SearchScrollState, SearchState, SearchValue};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchViewVariant {
    #[default]
    Docked,
    FullScreen,
    FullScreenContained,
}

/// Independent appearance overrides. Omitted values use the current window theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct SearchStyle {
    pub trigger_colors: Option<ButtonColors>,
    pub trigger_shapes: Option<ButtonShapes>,
    pub input_colors: Option<TextFieldColors>,
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub divider: Option<Hsla>,
    pub shape: Option<CornerShape>,
    pub results_shape: Option<CornerShape>,
    pub tonal_elevation: Option<Elevation>,
    pub shadow_elevation: Option<Elevation>,
}

/// Optional screen dimming with a window theme fallback.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchScrim {
    pub color: Option<Hsla>,
    pub opacity: f32,
}

impl Default for SearchScrim {
    fn default() -> Self {
        Self {
            color: None,
            opacity: 0.32,
        }
    }
}

/// Behavior for a docked popup anchored to the search trigger.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchPopupOptions {
    pub logical_anchor: Anchor,
    pub offset: Pixels,
    pub dismiss_on_outside: bool,
    pub dismiss_on_escape: bool,
    pub trap_focus: bool,
    pub scrim: Option<SearchScrim>,
}

impl Default for SearchPopupOptions {
    fn default() -> Self {
        Self {
            logical_anchor: Anchor::TopLeft,
            offset: px(4.),
            dismiss_on_outside: true,
            dismiss_on_escape: true,
            trap_focus: false,
            scrim: None,
        }
    }
}

/// Properties of the full-window search overlay. Insets use left and right as logical start and end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchWindowOptions {
    pub insets: Edges<Pixels>,
    pub dismiss_on_outside: bool,
    pub dismiss_on_escape: bool,
    pub scrim: SearchScrim,
}

impl Default for SearchWindowOptions {
    fn default() -> Self {
        Self {
            insets: Edges::all(px(0.)),
            dismiss_on_outside: true,
            dismiss_on_escape: true,
            scrim: SearchScrim::default(),
        }
    }
}

/// Show and dismissal durations. The default settles immediately.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SearchMotion {
    pub expand: Duration,
    pub collapse: Duration,
}

type InputBuilder = Box<dyn FnOnce(TextField) -> AnyElement>;
type RowBuilder = Rc<dyn Fn(ListItem, &SearchResult, &mut Window, &mut App) -> ListItem>;
type ResultBuilder = Box<dyn FnOnce(&Entity<SearchState>, &mut Window, &mut App) -> AnyElement>;

/// A Material search entry that can occupy the existing AppBar search slot.
#[derive(IntoElement)]
pub struct SearchBar {
    id: ElementId,
    view: SearchView,
    collapsed_width: Option<Pixels>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    trigger_content: Option<AnyElement>,
    scroll: Option<Entity<SearchScrollState>>,
    linked_scroll: Option<ScrollHandle>,
}

impl SearchBar {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<SearchState>,
    ) -> Self {
        let id = id.into();
        Self {
            view: SearchView::new((id.clone(), "search-view"), label, state),
            id,
            collapsed_width: None,
            leading: None,
            trailing: None,
            trigger_content: None,
            scroll: None,
            linked_scroll: None,
        }
    }

    pub fn variant(mut self, variant: SearchViewVariant) -> Self {
        self.view.variant = variant;
        self
    }

    pub fn labels(
        mut self,
        close: impl Into<SharedString>,
        loading: impl Into<SharedString>,
        empty: impl Into<SharedString>,
        prompt: impl Into<SharedString>,
    ) -> Self {
        self.view = self
            .view
            .labels(close.into(), loading.into(), empty.into(), prompt.into());
        self
    }

    /// Applies explicit widths to the trigger and its focused editor.
    pub fn widths(mut self, collapsed: Pixels, focused: Pixels) -> Self {
        self.collapsed_width = valid_dimension(collapsed);
        self.view.focused_width = valid_dimension(focused);
        self
    }

    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    pub fn trigger_content(mut self, element: impl IntoElement) -> Self {
        self.trigger_content = Some(element.into_any_element());
        self
    }

    pub fn style(mut self, style: SearchStyle) -> Self {
        self.view.style = style;
        self
    }

    pub fn motion(mut self, motion: SearchMotion) -> Self {
        self.view.motion = motion;
        self
    }

    pub fn expanded_view(mut self, configure: impl FnOnce(SearchView) -> SearchView) -> Self {
        self.view = configure(self.view);
        self
    }

    pub fn scroll_behavior(mut self, state: &Entity<SearchScrollState>) -> Self {
        self.scroll = Some(state.clone());
        self
    }

    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.linked_scroll = Some(handle.clone());
        self
    }
}

impl RenderOnce for SearchBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let view = self.view;
        let state = view.state.clone();
        if state.update(cx, |model, cx| model.sample_animation(window, cx)) {
            window.request_animation_frame();
        }
        let model = state.read(cx);
        let focus = model.trigger.clone();
        let scope = model.scope.clone();
        let visible = model.presented();
        let focused = focus.is_focused(window);
        let progress = model.progress();
        let label = if model.query.is_empty() {
            view.label.clone()
        } else {
            model.query.clone()
        };
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let viewport = window.viewport_size();
        let size_limit = (viewport.width - px(16.)).max(px(1.));
        let width = match (self.collapsed_width, view.focused_width) {
            (Some(rest), Some(active)) if visible => Some(rest + (active - rest) * progress),
            (_, Some(active)) if focused => Some(active),
            (rest, _) => rest,
        };
        let id: ElementId = (self.id.clone(), "search-trigger").into();
        let button = match width {
            Some(width) => ButtonPart::new(id, label, width.min(size_limit)),
            None => ButtonPart::intrinsic(id, label),
        };
        let open = state.clone();
        let trigger_focus = focus.clone();
        let motion = view.motion;
        let button = button
            .track_focus(&focus)
            .size(ControlSize::Medium)
            .variant(ButtonVariant::FilledTonal)
            .leading_icon(Icon::new(IconName::Search))
            .when_some(view.style.trigger_shapes, ButtonPart::shapes)
            .when_some(view.style.trigger_colors, ButtonPart::colors)
            .when_some(self.trigger_content, ButtonPart::content)
            .on_activate(move |window, cx| {
                trigger_focus.focus(window, cx);
                open.update(cx, |state, cx| {
                    state.animate_to(SearchValue::Expanded, motion.expand, window, cx)
                });
            });
        let trigger = div()
            .id((self.id.clone(), "trigger-slots"))
            .flex()
            .items_center()
            .gap_2()
            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
            .children(
                self.leading
                    .map(|element| div().id("leading").child(element)),
            )
            .child(button)
            .children(
                self.trailing
                    .map(|element| div().id("trailing").child(element)),
            );

        if !visible {
            return match self.scroll {
                Some(scroll) => scroll_trigger(
                    self.id,
                    trigger,
                    scroll,
                    self.linked_scroll,
                    focused,
                    window,
                    cx,
                ),
                None => trigger.into_any_element(),
            };
        }
        if let Some(scroll) = self.scroll {
            scroll.update(cx, |scroll, _| {
                scroll.measure(
                    self.linked_scroll.as_ref().map(|handle| -handle.offset().y),
                    px(56.),
                    true,
                );
            });
        }

        let label = view.label.clone();
        let close_label = view.close_label.clone();
        match view.variant {
            SearchViewVariant::Docked => {
                let options = view.popup;
                let outside = state.clone();
                let mount = div()
                    .id("docked-search")
                    .role(Role::Dialog)
                    .aria_label(label)
                    .track_focus(&scope)
                    .occlude()
                    .when(
                        options.dismiss_on_outside && options.scrim.is_none(),
                        |dock| {
                            dock.on_mouse_down_out(move |_, window, cx| {
                                dismiss_search(&outside, motion.collapse, window, cx);
                            })
                        },
                    )
                    .child(view);
                let trap = options.trap_focus || options.scrim.is_some();
                let content = if trap {
                    let focus = window
                        .use_keyed_state((self.id.clone(), "dock-trap"), cx, |_, cx| {
                            cx.focus_handle().tab_stop(false)
                        })
                        .read(cx)
                        .clone();
                    mount.focus_trap("docked-search", &focus).into_any_element()
                } else {
                    mount.into_any_element()
                };
                let popup = Popup::new(self.id.clone(), trigger)
                    .anchor(logical_anchor(options.logical_anchor, direction))
                    .offset(length(options.offset))
                    .content(ThemeScope::inherit(content, window, cx));
                div()
                    .relative()
                    .children(options.scrim.map(|style| {
                        let backdrop = backdrop(
                            self.id,
                            style,
                            close_label,
                            progress,
                            options.dismiss_on_outside,
                            state,
                            motion.collapse,
                        );
                        deferred(
                            Positioner::corner(Anchor::TopLeft, point(px(0.), px(0.)))
                                .margin(px(0.))
                                .occlude()
                                .child(
                                    div()
                                        .relative()
                                        .w(viewport.width)
                                        .h(viewport.height)
                                        .child(ThemeScope::inherit(backdrop, window, cx)),
                                ),
                        )
                        .with_priority(99)
                    }))
                    .child(popup)
                    .into_any_element()
            }
            SearchViewVariant::FullScreen | SearchViewVariant::FullScreenContained => {
                let options = view.window;
                let insets = options.insets;
                let exposed = [insets.left, insets.right, insets.top, insets.bottom]
                    .into_iter()
                    .any(|inset| length(inset) > px(0.));
                let backdrop = backdrop(
                    self.id,
                    options.scrim,
                    close_label,
                    progress,
                    options.dismiss_on_outside && exposed,
                    state.clone(),
                    motion.collapse,
                );
                div()
                    .child(trigger)
                    .child(
                        Dialog::new(cx)
                            .accessibility_label(label)
                            .focus_handle(scope)
                            .close_on_backdrop_press(false)
                            .backdrop(ThemeScope::inherit(backdrop, window, cx))
                            .on_ok(|_, _, _| false)
                            .on_cancel(move |_, window, cx| {
                                if options.dismiss_on_escape {
                                    dismiss_search(&state, motion.collapse, window, cx);
                                }
                                false
                            })
                            .popup(ThemeScope::inherit(view, window, cx)),
                    )
                    .into_any_element()
            }
        }
    }
}

fn backdrop(
    id: ElementId,
    style: SearchScrim,
    label: SharedString,
    progress: f32,
    dismissible: bool,
    state: Entity<SearchState>,
    duration: Duration,
) -> impl IntoElement {
    let opacity = if style.opacity.is_nan() {
        0.32
    } else {
        style.opacity.clamp(0., 1.)
    };
    Scrim::new((id, "search-scrim"))
        .opacity(opacity * progress)
        .when_some(style.color, Scrim::color)
        .when(dismissible, |scrim| {
            scrim
                .accessibility_label(label)
                .on_click(move |window, cx| {
                    dismiss_search(&state, duration, window, cx);
                })
        })
}

fn scroll_trigger(
    id: ElementId,
    trigger: impl IntoElement,
    state: Entity<SearchScrollState>,
    linked: Option<ScrollHandle>,
    held_open: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let size = window
        .use_keyed_state((id.clone(), "search-scroll-height"), cx, |_, _| {
            Rc::new(Cell::new(px(56.)))
        })
        .read(cx)
        .clone();
    let linked_offset = linked.as_ref().map(|handle| -handle.offset().y);
    let distance = state.update(cx, |state, _| {
        state.measure(linked_offset, size.get(), held_open)
    });
    div()
        .id((id, "search-scroll"))
        .test_support()
        .relative()
        .h(size.get() - distance)
        .overflow_hidden()
        .child(
            div().relative().top(-distance).child(trigger).child(
                canvas(
                    move |bounds, window, _| {
                        if size.replace(bounds.size.height) != bounds.size.height
                            || linked
                                .as_ref()
                                .is_some_and(|handle| Some(-handle.offset().y) != linked_offset)
                        {
                            window.refresh();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            ),
        )
        .into_any_element()
}

/// Expanded Material search editor and result region, also usable as an independent view.
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
    input: Option<InputBuilder>,
    filters: Option<AnyElement>,
    row: Option<RowBuilder>,
    result_content: Option<ResultBuilder>,
    focused_width: Option<Pixels>,
    docked_dimensions: (Pixels, Pixels),
    gap: Pixels,
    style: SearchStyle,
    motion: SearchMotion,
    popup: SearchPopupOptions,
    window: SearchWindowOptions,
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
        if animate {
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
        let close = self.state.clone();
        let dismiss = self.state.clone();
        let duration = self.motion.collapse;
        let close_on_escape = if docked {
            self.popup.dismiss_on_escape
        } else {
            self.window.dismiss_on_escape
        };
        let field = TextField::new((self.id.clone(), "editor"), self.label.clone(), &editor)
            .when_some(self.style.input_colors, TextField::colors);
        let field = match self.input.take() {
            Some(builder) => builder(field),
            None => field.into_any_element(),
        };
        let input = div()
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
            );

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

pub(super) fn length(value: Pixels) -> Pixels {
    if f32::from(value).is_finite() {
        value.max(px(0.))
    } else {
        px(0.)
    }
}

fn valid_dimension(value: Pixels) -> Option<Pixels> {
    let value = length(value);
    (value > px(0.)).then_some(value)
}

fn dismiss_search(
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
