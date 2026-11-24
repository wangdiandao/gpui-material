mod driver;
mod state;

use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, ScrollHandle, SharedString, StatefulInteractiveElement, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{StyledExt, TestSupportExt};
use gpui_material_foundation::current_material_theme;
use gpui_material_theme::Direction;

use crate::{Button, ButtonVariant, LoadingIndicator};

use driver::{Refresh, RefreshDriver, RefreshViewport};
use state::{ScrollBinding, ViewportKey};

pub use state::{PullToRefreshProgress, PullToRefreshState};

const DEFAULT_DISTANCE: Pixels = px(80.);

type Indicator = Box<dyn FnOnce(PullToRefreshProgress, &mut Window, &mut App) -> AnyElement>;

/// A vertical viewport with an Expressive refresh indicator and accessible action.
#[derive(IntoElement)]
pub struct PullToRefresh {
    id: ElementId,
    state: Entity<PullToRefreshState>,
    refreshing: bool,
    enabled: bool,
    refresh: Refresh,
    label: SharedString,
    threshold: Pixels,
    indicator_distance: Pixels,
    binding: Option<ScrollBinding>,
    indicator: Option<Indicator>,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl PullToRefresh {
    /// Creates a pull-to-refresh container bound to `state`, caller-owned `refreshing` status, and refresh callback.
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<PullToRefreshState>,
        refreshing: bool,
        on_refresh: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            refreshing,
            enabled: true,
            refresh: Rc::new(on_refresh),
            label: "Refresh".into(),
            threshold: DEFAULT_DISTANCE,
            indicator_distance: DEFAULT_DISTANCE,
            binding: None,
            indicator: None,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// Enables or disables pull gestures and the accessible refresh action button. Defaults to `true`.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the label used for both the accessible refresh button and default loading indicator. Defaults to `"Refresh"`.
    pub fn refresh_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the pull threshold in logical pixels. Invalid values use the default.
    pub fn threshold(mut self, threshold: Pixels) -> Self {
        if f32::from(threshold).is_finite() && threshold > px(0.) {
            self.threshold = threshold;
        }
        self
    }

    /// Sets indicator travel at the threshold in logical pixels.
    pub fn indicator_distance(mut self, distance: Pixels) -> Self {
        if f32::from(distance).is_finite() && distance >= px(0.) {
            self.indicator_distance = distance;
        }
        self
    }

    /// Wraps a caller-owned viewport that tracks this scroll handle.
    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.binding = Some(ScrollBinding::Handle(handle.clone()));
        self
    }

    /// Wraps a caller-owned viewport using a positive distance from its top edge.
    pub fn scroll_position(mut self, position: impl Fn(&Window, &App) -> Pixels + 'static) -> Self {
        self.binding = Some(ScrollBinding::Position(Rc::new(position)));
        self
    }

    /// Replaces the loading indicator with decorative content using the same reveal.
    pub fn indicator<E: IntoElement>(
        mut self,
        indicator: impl FnOnce(PullToRefreshProgress, &mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.indicator = Some(Box::new(move |progress, window, cx| {
            indicator(progress, window, cx).into_any_element()
        }));
        self
    }
}

impl Styled for PullToRefresh {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for PullToRefresh {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl RenderOnce for PullToRefresh {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let viewport = ViewportKey {
            window: window.window_handle().window_id(),
            id: self.id.clone(),
        };
        let (progress, can_request, scroll) = self.state.update(cx, |state, cx| {
            state.configure(&viewport, self.refreshing, self.enabled, window, cx);
            (
                PullToRefreshProgress {
                    distance_fraction: state.fraction,
                    offset: (self.indicator_distance * state.fraction.min(2.))
                        .min(window.viewport_size().height),
                    refreshing: state.refreshing,
                    refresh_pending: state.pending,
                    animating: state.is_animating(),
                },
                state.can_request(),
                state.scroll_handle(),
            )
        });
        let rtl = current_material_theme(window, cx).layout.direction == Direction::Rtl;
        let owned = self.binding.is_none();
        let mut body = div()
            .id("refresh-scroll")
            .test_support()
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .children(self.children);
        if owned {
            body = body.overflow_y_scroll().track_scroll(&scroll);
        }
        let driver = RefreshDriver {
            state: self.state.downgrade(),
            viewport,
            refresh: self.refresh,
            threshold: self.threshold,
            binding: self.binding.unwrap_or(ScrollBinding::Handle(scroll)),
        };
        let command = driver.clone();
        let indicator = (progress.distance_fraction > 0.).then(|| {
            let element = match self.indicator {
                Some(indicator) => indicator(progress, window, cx),
                None => {
                    let loading = LoadingIndicator::new("refresh-progress", self.label.clone())
                        .contained(true);
                    if progress.refreshing {
                        loading.into_any_element()
                    } else {
                        loading.value(progress.distance_fraction).into_any_element()
                    }
                }
            };
            div()
                .absolute()
                .left_0()
                .right_0()
                .top(progress.offset - px(48.))
                .flex()
                .justify_center()
                .child(element)
        });
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .refine_style(&self.style)
            .child(
                div()
                    .flex()
                    .flex_none()
                    .when(rtl, Styled::justify_start)
                    .when(!rtl, Styled::justify_end)
                    .child(
                        Button::new("refresh-action", self.label)
                            .variant(ButtonVariant::Text)
                            .disabled(!can_request)
                            .on_activate(move |window, cx| command.request(window, cx)),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .relative()
                    .overflow_hidden()
                    .child(RefreshViewport {
                        driver,
                        body: body.into_any_element(),
                    })
                    .children(indicator),
            )
    }
}
