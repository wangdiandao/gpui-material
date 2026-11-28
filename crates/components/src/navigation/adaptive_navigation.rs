use std::time::Duration;

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, ContentMask, Context, Element, ElementId, Entity,
    GlobalElementId, InspectorElementId, InteractiveElement, IntoElement, LayoutId, ParentElement,
    Pixels, Point, RenderOnce, Size, Style, Styled, Window, div, point, prelude::FluentBuilder,
    relative, size,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{RetargetTransition, current_material_theme};
use gpui_material_theme::{CubicBezier, Direction, WidthClass};

use super::{
    NavigationBar, NavigationBarLayout, NavigationItems, NavigationRail, RailMode, Scaffold,
    rail::length,
};

/// Current Material 3 Expressive navigation presentations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdaptiveNavigationType {
    /// Compact bottom navigation bar with vertical icon-over-label items.
    ShortBarCompact,
    /// Medium bottom navigation bar with horizontal icon-beside-label items.
    ShortBarMedium,
    /// Collapsed side navigation rail.
    RailCollapsed,
    /// Expanded side navigation rail with inline labels.
    RailExpanded,
    /// Leaves the scaffold's existing slots in place.
    None,
}

impl AdaptiveNavigationType {
    /// Selects the default Material navigation presentation for the given available width.
    pub fn for_width(width: Pixels) -> Self {
        match WidthClass::from_width(width.into()) {
            WidthClass::Compact => Self::ShortBarCompact,
            WidthClass::Medium => Self::ShortBarMedium,
            _ => Self::RailCollapsed,
        }
    }

    fn is_rail(self) -> bool {
        matches!(self, Self::RailCollapsed | Self::RailExpanded)
    }
}

/// Available scaffold space for an application-supplied navigation policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdaptiveNavigationInfo {
    /// Bounded width and height available to the scaffold.
    pub available_size: Size<Pixels>,
    /// Material width class derived from `available_size.width`.
    pub width_class: WidthClass,
    /// Active layout direction from the window theme.
    pub direction: Direction,
}

/// Settled visibility of the adaptive navigation slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AdaptiveNavigationVisibility {
    /// The navigation bar or rail is visible.
    #[default]
    Shown,
    /// The navigation bar or rail is hidden.
    Hidden,
}

impl AdaptiveNavigationVisibility {
    fn fraction(self) -> f32 {
        f32::from(self == Self::Shown)
    }
}

/// Retained navigation visibility, independent of the current presentation.
pub struct AdaptiveNavigationState {
    current: AdaptiveNavigationVisibility,
    target: AdaptiveNavigationVisibility,
    fraction: f32,
    duration: Duration,
    transition: RetargetTransition,
}

impl Default for AdaptiveNavigationState {
    fn default() -> Self {
        Self::new(AdaptiveNavigationVisibility::Shown)
    }
}

impl AdaptiveNavigationState {
    /// Creates retained navigation visibility state starting at `initial` with a 240 ms default transition.
    pub fn new(initial: AdaptiveNavigationVisibility) -> Self {
        Self {
            current: initial,
            target: initial,
            fraction: initial.fraction(),
            duration: Duration::from_millis(240),
            transition: RetargetTransition::settled(initial.fraction()),
        }
    }

    /// Sets the duration used when animating between shown and hidden states.
    pub fn transition_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Last settled visibility.
    pub fn current_value(&self) -> AdaptiveNavigationVisibility {
        self.current
    }

    /// Visibility state currently settled at or transitioning toward.
    pub fn target_value(&self) -> AdaptiveNavigationVisibility {
        self.target
    }

    /// Returns whether a show/hide transition is currently running.
    pub fn is_animating(&self) -> bool {
        self.transition.is_animating()
    }

    /// The currently visible fraction of the slot, in 0..=1.
    pub fn progress(&self) -> f32 {
        self.fraction
    }

    /// Animates the navigation slot to [`AdaptiveNavigationVisibility::Shown`].
    pub fn show(&mut self, cx: &mut Context<Self>) {
        self.animate_to(AdaptiveNavigationVisibility::Shown, cx);
    }

    /// Animates the navigation slot to [`AdaptiveNavigationVisibility::Hidden`].
    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.animate_to(AdaptiveNavigationVisibility::Hidden, cx);
    }

    /// Toggles the target visibility between shown and hidden.
    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        match self.target {
            AdaptiveNavigationVisibility::Shown => self.hide(cx),
            AdaptiveNavigationVisibility::Hidden => self.show(cx),
        }
    }

    /// Moves immediately to `value` without animation.
    pub fn snap_to(&mut self, value: AdaptiveNavigationVisibility, cx: &mut Context<Self>) {
        if self.current != value || self.target != value || self.is_animating() {
            self.target = value;
            self.settle();
            cx.notify();
        }
    }

    fn animate_to(&mut self, target: AdaptiveNavigationVisibility, cx: &mut Context<Self>) {
        if self.duration.is_zero() || cx.reduce_motion() {
            self.snap_to(target, cx);
            return;
        }
        self.sample(cx);
        if self.target != target {
            self.target = target;
            let now = cx.background_executor().now();
            self.transition.animate_to(
                target.fraction(),
                self.duration,
                CubicBezier::EASE_IN_OUT,
                now,
                false,
            );
            self.fraction = self.transition.value();
            if !self.transition.is_animating() {
                self.settle();
            }
            cx.notify();
        }
    }

    fn settle(&mut self) {
        self.current = self.target;
        self.fraction = self.target.fraction();
        self.transition.snap_to(self.fraction);
    }

    fn sample(&mut self, cx: &mut Context<Self>) {
        if !self.transition.is_animating() {
            return;
        }
        let now = cx.background_executor().now();
        let (next, running) = self.transition.sample(now, cx.reduce_motion());
        let notify = if !running {
            self.settle();
            true
        } else if next != self.fraction {
            self.fraction = next;
            true
        } else {
            false
        };
        if notify {
            let state = cx.weak_entity();
            cx.defer(move |cx| {
                let _ = state.update(cx, |_, cx| cx.notify());
            });
        }
    }
}

type Policy = Box<dyn FnOnce(AdaptiveNavigationInfo) -> AdaptiveNavigationType>;
type NavigationContent = Box<dyn FnOnce(AdaptiveNavigationType, AnyElement) -> AnyElement>;
type Configure<T> = Box<dyn FnOnce(AdaptiveNavigationType, T) -> T>;

/// Material 3 navigation suite for the existing Scaffold bar and rail slots.
#[derive(IntoElement)]
pub struct AdaptiveNavigation {
    scaffold: Scaffold,
    items: NavigationItems,
    width: Pixels,
    height: Option<Pixels>,
    policy: Option<Policy>,
    state: Option<Entity<AdaptiveNavigationState>>,
    content: Option<NavigationContent>,
    bar: Option<Configure<NavigationBar>>,
    rail: Option<Configure<NavigationRail>>,
}

impl AdaptiveNavigation {
    /// Uses the available width of the scaffold in logical units.
    pub fn new(scaffold: Scaffold, items: NavigationItems, available_width: Pixels) -> Self {
        Self {
            scaffold,
            items,
            width: available_width,
            height: None,
            policy: None,
            state: None,
            content: None,
            bar: None,
            rail: None,
        }
    }

    /// Supplies the bounded scaffold size when it differs from the viewport.
    pub fn available_size(mut self, size: Size<Pixels>) -> Self {
        self.width = size.width;
        self.height = Some(size.height);
        self
    }

    /// Forces a fixed navigation presentation regardless of available width.
    pub fn navigation_type(self, kind: AdaptiveNavigationType) -> Self {
        self.navigation_policy(move |_| kind)
    }

    /// Supplies a custom policy that maps [`AdaptiveNavigationInfo`] to a presentation type.
    pub fn navigation_policy(
        mut self,
        policy: impl FnOnce(AdaptiveNavigationInfo) -> AdaptiveNavigationType + 'static,
    ) -> Self {
        self.policy = Some(Box::new(policy));
        self
    }

    /// Binds retained visibility state for animated showing and hiding of the navigation slot.
    pub fn state(mut self, state: &Entity<AdaptiveNavigationState>) -> Self {
        self.state = Some(state.clone());
        self
    }

    /// Composes or replaces the standard navigation in its scaffold slot.
    pub fn navigation_content<E: IntoElement>(
        mut self,
        build: impl FnOnce(AdaptiveNavigationType, AnyElement) -> E + 'static,
    ) -> Self {
        self.content = Some(Box::new(move |kind, default| {
            build(kind, default).into_any_element()
        }));
        self
    }

    /// Customizes the generated [`NavigationBar`] when a bottom-bar presentation is active.
    pub fn navigation_bar(
        mut self,
        build: impl FnOnce(AdaptiveNavigationType, NavigationBar) -> NavigationBar + 'static,
    ) -> Self {
        self.bar = Some(Box::new(build));
        self
    }

    /// Customizes the generated [`NavigationRail`] when a side-rail presentation is active.
    pub fn navigation_rail(
        mut self,
        build: impl FnOnce(AdaptiveNavigationType, NavigationRail) -> NavigationRail + 'static,
    ) -> Self {
        self.rail = Some(Box::new(build));
        self
    }
}

impl RenderOnce for AdaptiveNavigation {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let viewport = window.viewport_size();
        let available = size(
            length(self.width),
            length(self.height.unwrap_or(viewport.height)),
        );
        let direction = current_material_theme(window, cx).layout.direction;
        let info = AdaptiveNavigationInfo {
            available_size: available,
            width_class: WidthClass::from_width(available.width.into()),
            direction,
        };
        let kind = match self.policy {
            Some(policy) => policy(info),
            None => AdaptiveNavigationType::for_width(available.width),
        };
        let (fraction, moving) = self.state.as_ref().map_or((1., false), |state| {
            state.update(cx, |state, cx| {
                state.sample(cx);
                (state.progress(), state.is_animating())
            })
        });
        if moving && window.is_visible() {
            window.request_animation_frame();
        }
        let scope = window
            .use_keyed_state(
                (self.items.id().clone(), "adaptive-navigation-focus"),
                cx,
                |_, cx| cx.focus_handle().tab_stop(false),
            )
            .read(cx)
            .clone();
        let items = self.items.retain(window, cx).retain_roving_stop();
        if kind == AdaptiveNavigationType::None || fraction <= 0. {
            if scope.contains_focused(window, cx) {
                let previous = window.focused(cx);
                window.defer(cx, move |window, cx| {
                    if window.focused(cx) == previous {
                        window.focus_next(cx);
                        if window.focused(cx) == previous {
                            window.blur(cx);
                        }
                    }
                });
            }
            return self.scaffold;
        }
        let items_id = items.id().clone();
        let is_rail = kind.is_rail();
        let default = if is_rail {
            let mode = if kind == AdaptiveNavigationType::RailExpanded {
                RailMode::Expanded
            } else {
                RailMode::Collapsed
            };
            let rail = NavigationRail::new((items_id.clone(), "adaptive-rail"), items).mode(mode);
            match self.rail {
                Some(build) => build(kind, rail),
                None => rail,
            }
            .into_any_element()
        } else {
            let layout = if kind == AdaptiveNavigationType::ShortBarMedium {
                NavigationBarLayout::Horizontal
            } else {
                NavigationBarLayout::Vertical
            };
            let bar = NavigationBar::new(items).layout(layout);
            match self.bar {
                Some(build) => build(kind, bar),
                None => bar,
            }
            .into_any_element()
        };
        let content = match self.content {
            Some(build) => build(kind, default),
            None => default,
        };
        let content = div()
            .flex()
            .flex_col()
            .when(is_rail, |content| content.h_full().max_w(available.width))
            .when(!is_rail, |content| content.w_full().max_h(available.height))
            .child(content);
        let slot = div()
            .id((items_id, "adaptive-navigation-slot"))
            .test_support()
            .track_focus(&scope)
            .flex_none()
            .when(is_rail, Styled::h_full)
            .when(!is_rail, Styled::w_full);
        let slot = if fraction < 1. {
            slot.child(SlidingSlot {
                content: content.into_any_element(),
                available,
                fraction,
                is_rail,
                direction,
            })
        } else {
            slot.child(content)
        };
        if is_rail {
            self.scaffold.rail(slot)
        } else {
            self.scaffold.bottom_bar(slot)
        }
    }
}

/// Measures the natural slot once and reserves its animated fraction. The
/// same content is clipped and translated, preserving its layout and IDs.
struct SlidingSlot {
    content: AnyElement,
    available: Size<Pixels>,
    fraction: f32,
    is_rail: bool,
    direction: Direction,
}

impl Element for SlidingSlot {
    type RequestLayoutState = Size<Pixels>;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let natural =
            self.content
                .layout_as_root(self.available.map(AvailableSpace::Definite), window, cx);
        let size = if self.is_rail {
            size((natural.width * self.fraction).into(), relative(1.).into())
        } else {
            size(relative(1.).into(), (natural.height * self.fraction).into())
        };
        (
            window.request_layout(
                Style {
                    size,
                    ..Style::default()
                },
                None,
                cx,
            ),
            natural,
        )
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        natural: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let cross_fit = if self.is_rail {
            size(self.available.width, bounds.size.height)
        } else {
            size(bounds.size.width, self.available.height)
        };
        let natural = if cross_fit != self.available {
            self.content
                .layout_as_root(cross_fit.map(AvailableSpace::Definite), window, cx)
        } else {
            *natural
        };
        let origin: Point<Pixels> = if self.is_rail && self.direction == Direction::Ltr {
            point(bounds.right() - natural.width, bounds.top())
        } else {
            bounds.origin
        };
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            self.content.prepaint_at(origin, window, cx);
        });
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            self.content.paint(window, cx)
        });
    }
}
