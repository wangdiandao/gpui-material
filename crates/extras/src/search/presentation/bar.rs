use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    Anchor, AnyElement, App, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Role, ScrollHandle, SharedString, Size,
    StatefulInteractiveElement, Styled, Window, canvas, deferred, div, point,
    prelude::FluentBuilder, px,
};
use gpui_base::{Dialog, FocusTrapElement, Popup, Positioner, TestSupportExt};
use gpui_material_components::{
    ButtonVariant, ControlSize, Icon, IconName, button_composition::ButtonPart,
};
use gpui_material_foundation::{Scrim, ThemeScope, current_material_theme, logical_anchor};
use gpui_material_theme::Direction;

use super::{
    SearchMotion, SearchScrim, SearchStyle, SearchView, SearchViewVariant, length, valid_dimension,
    view::dismiss_search,
};
use crate::search::{SearchScrollState, SearchState, SearchValue};

/// A Material search entry that can occupy the existing AppBar search slot.
#[derive(IntoElement)]
pub struct SearchBar {
    id: ElementId,
    view: SearchView,
    collapsed_width: Option<Pixels>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    trigger_content: Option<AnyElement>,
    track_focus: Option<FocusHandle>,
    scroll: Option<Entity<SearchScrollState>>,
    linked_scroll: Option<ScrollHandle>,
}

impl SearchBar {
    /// Creates a search bar trigger paired with an expandable [`SearchView`] backed by `state`.
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
            track_focus: None,
            scroll: None,
            linked_scroll: None,
        }
    }

    /// Sets the expanded search view's presentation variant (`Docked`, `FullScreen`, or `FullScreenContained`).
    pub fn variant(mut self, variant: SearchViewVariant) -> Self {
        self.view.variant = variant;
        self
    }

    /// Sets localized accessibility and status labels for closing, loading, empty results, and idle prompt.
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

    /// Places a leading slot element before the search trigger button.
    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    /// Places a trailing slot element after the search trigger button.
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    /// Replaces the visible content inside the trigger button while retaining its accessible label.
    pub fn trigger_content(mut self, element: impl IntoElement) -> Self {
        self.trigger_content = Some(element.into_any_element());
        self
    }

    /// Applies visual style overrides to both the trigger and the expanded search view.
    pub fn style(mut self, style: SearchStyle) -> Self {
        self.view.style = style;
        self
    }

    /// Configures expand and collapse animation durations.
    pub fn motion(mut self, motion: SearchMotion) -> Self {
        self.view.motion = motion;
        self
    }

    /// Customizes the inner [`SearchView`] directly via a builder closure.
    pub fn expanded_view(mut self, configure: impl FnOnce(SearchView) -> SearchView) -> Self {
        self.view = configure(self.view);
        self
    }

    /// Connects a [`SearchScrollState`] to hide and reveal the collapsed bar on scroll.
    pub fn scroll_behavior(mut self, state: &Entity<SearchScrollState>) -> Self {
        self.scroll = Some(state.clone());
        self
    }

    /// Links a GPUI [`ScrollHandle`] whose vertical offset drives [`Self::scroll_behavior`].
    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.linked_scroll = Some(handle.clone());
        self
    }

    /// Attaches a caller-owned focus handle to the search bar trigger button.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.track_focus = Some(focus.clone());
        self
    }
}

impl RenderOnce for SearchBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let view = self.view;
        let state = view.state.clone();
        if state.read(cx).is_animating()
            && state.update(cx, |model, cx| model.sample_animation(window, cx))
            && window.is_visible()
        {
            window.request_animation_frame();
        }
        let model = state.read(cx);
        let focus = self.track_focus.unwrap_or_else(|| model.trigger.clone());
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

        match view.variant {
            SearchViewVariant::Docked => render_docked_overlay(
                self.id, trigger, view, state, scope, direction, viewport, progress, window, cx,
            ),
            SearchViewVariant::FullScreen | SearchViewVariant::FullScreenContained => {
                render_fullscreen_overlay(
                    self.id, trigger, view, state, scope, progress, window, cx,
                )
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_docked_overlay(
    id: ElementId,
    trigger: impl IntoElement,
    view: SearchView,
    state: Entity<SearchState>,
    scope: FocusHandle,
    direction: Direction,
    viewport: Size<Pixels>,
    progress: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let label = view.label.clone();
    let close_label = view.close_label.clone();
    let motion = view.motion;
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
            .use_keyed_state((id.clone(), "dock-trap"), cx, |_, cx| {
                cx.focus_handle().tab_stop(false)
            })
            .read(cx)
            .clone();
        mount.focus_trap("docked-search", &focus).into_any_element()
    } else {
        mount.into_any_element()
    };
    let popup = Popup::new(id.clone(), trigger)
        .anchor(logical_anchor(options.logical_anchor, direction))
        .offset(length(options.offset))
        .content(ThemeScope::inherit(content, window, cx));
    div()
        .relative()
        .children(options.scrim.map(|style| {
            let backdrop = backdrop(
                id,
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

#[allow(clippy::too_many_arguments)]
fn render_fullscreen_overlay(
    id: ElementId,
    trigger: impl IntoElement,
    view: SearchView,
    state: Entity<SearchState>,
    scope: FocusHandle,
    progress: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let label = view.label.clone();
    let close_label = view.close_label.clone();
    let motion = view.motion;
    let options = view.window;
    let insets = options.insets;
    let exposed = [insets.left, insets.right, insets.top, insets.bottom]
        .into_iter()
        .any(|inset| length(inset) > px(0.));
    let backdrop = backdrop(
        id,
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
