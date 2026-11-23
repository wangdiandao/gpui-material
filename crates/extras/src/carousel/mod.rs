//! Material 3 carousel strategies and desktop navigation.

mod geometry;
mod parallax;
mod state;

use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    AccessibleAction, AnyElement, App, Bounds, ContentMask, Element, ElementId, Entity,
    GlobalElementId, ImageSource, InspectorElementId, InteractiveElement, IntoElement, LayoutId,
    ObjectFit, ParentElement, Pixels, RenderOnce, Role, ScrollEvent, SharedString,
    StatefulInteractiveElement, Style, Styled, StyledImage, TouchPhase, Window, div, img, point,
    prelude::FluentBuilder, px, relative, size,
};
use gpui_base::{Button, TestSupportExt};
use gpui_material_foundation::{animate, color, current_material_theme};
use gpui_material_theme::{ColorRole, CornerRole, Direction, MotionKind, MotionSpec, MotionSpeed};

use geometry::{Layout, LayoutOptions};
use state::ItemFocus;

pub use parallax::{CarouselBorder, CarouselParallax};
pub use state::CarouselState;

const SCROLL_SETTLE: Duration = Duration::from_millis(120);
const WIDTH_TO_HEIGHT_MIN: f32 = 9. / 16.;
const WIDTH_TO_HEIGHT_MAX: f32 = 16. / 9.;

/// Material carousel layouts. Full-screen pages scroll vertically.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselVariant {
    /// Displays multiple sized items (large, medium, and small) across the viewport.
    #[default]
    MultiBrowse,
    /// Scrolls uniform-width items freely without contained keyline shrinking.
    Uncontained,
    /// Scrolls items freely with per-item aspect ratios clamped to Material bounds.
    UncontainedMultiAspectRatio,
    /// Features one large hero item alongside smaller peeking items.
    Hero,
    /// Centers the large hero item between smaller peeking items on both sides.
    CenteredHero,
    /// Displays one full-bleed page at a time along the vertical axis.
    FullScreen,
}

impl CarouselVariant {
    fn is_vertical(self) -> bool {
        self == Self::FullScreen
    }

    fn is_uncontained(self) -> bool {
        matches!(self, Self::Uncontained | Self::UncontainedMultiAspectRatio)
    }
}

/// Resting alignment for Multi-browse, Hero, and Centered Hero keylines.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselAlignment {
    /// Aligns the focal keyline toward the leading edge of the carousel.
    #[default]
    Start,
    /// Centers the focal keyline within the carousel viewport.
    Center,
}

/// Padding in logical pixels. Start and end follow the window direction.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CarouselPadding {
    /// Leading-edge padding along the horizontal axis (respects RTL).
    pub start: Pixels,
    /// Trailing-edge padding along the horizontal axis (respects RTL).
    pub end: Pixels,
    /// Top-edge padding.
    pub top: Pixels,
    /// Bottom-edge padding.
    pub bottom: Pixels,
}

/// Positions before and after a precise scroll gesture, in fractional item units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarouselFlingContext {
    /// Fractional item position when the scroll gesture began.
    pub start: f32,
    /// Fractional item position when the scroll gesture ended.
    pub position: f32,
    /// Total number of items currently in the carousel.
    pub item_count: usize,
}

/// Chooses the resting position when a precise scroll gesture ends.
#[derive(Clone, Default)]
pub enum CarouselFling {
    /// Snap on contained layouts and Full-screen. Scroll freely on uncontained layouts.
    #[default]
    Default,
    /// Leaves the fractional position at the end of the gesture.
    Free,
    /// Snaps to the nearest item after any number of items have been scrolled.
    Snap,
    /// Snaps to the nearest item at most one index away from the gesture's start.
    SingleAdvance,
    /// Returns the resting item index, or None to leave the position where the gesture ended.
    Custom(Rc<dyn Fn(CarouselFlingContext) -> Option<usize>>),
}

impl CarouselFling {
    /// Creates a [`CarouselFling::Custom`] strategy from a callback that chooses the target item index.
    pub fn custom(target: impl Fn(CarouselFlingContext) -> Option<usize> + 'static) -> Self {
        Self::Custom(Rc::new(target))
    }
}

fn nonnegative(value: Pixels) -> f32 {
    let value = f32::from(value);
    if value.is_finite() { value.max(0.) } else { 0. }
}

type Activate = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

enum Content {
    Element(AnyElement),
    Image(ImageSource),
}

/// One slide, with a stable unique identifier and a localized accessible name.
pub struct CarouselItem {
    id: SharedString,
    label: SharedString,
    aspect: f32,
    disabled: bool,
    content: Content,
}

impl CarouselItem {
    /// Creates a carousel item with a stable `id`, accessible `label`, and custom element `content`.
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        content: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            aspect: 1.,
            disabled: false,
            content: Content::Element(content.into_any_element()),
        }
    }

    /// A full-bleed image cropped to the changing item mask.
    pub fn image(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        source: impl Into<ImageSource>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            aspect: 1.,
            disabled: false,
            content: Content::Image(source.into()),
        }
    }

    /// Width divided by height, used by the multi-aspect ratio layout.
    /// Values outside the Material 9:16–16:9 range are clamped.
    pub fn aspect_ratio(mut self, width_over_height: f32) -> Self {
        if width_over_height.is_finite() && width_over_height > 0. {
            self.aspect = width_over_height.clamp(WIDTH_TO_HEIGHT_MIN, WIDTH_TO_HEIGHT_MAX);
        }
        self
    }

    /// Disables activation and state-layer feedback on this item when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// A viewport-sized collection of Material carousel items.
///
/// The component retains the visible item by ID across frames. The caller
/// owns the item data and activation outcome. Tab enters the carousel at its
/// current item. Arrow keys move within it and Enter or Space activates.
/// Wheel and trackpad input follow the scrolling axis. Uncontained variants
/// scroll freely by default. The contained layouts settle on a keyline.
/// Supply descriptive item labels and noninteractive item content.
#[derive(IntoElement)]
pub struct Carousel {
    id: ElementId,
    label: SharedString,
    items: Vec<CarouselItem>,
    variant: CarouselVariant,
    height: Pixels,
    preferred_width: Pixels,
    layout_options: LayoutOptions,
    snap: Option<bool>,
    fling: CarouselFling,
    user_scroll_enabled: bool,
    state: Option<Entity<CarouselState>>,
    on_activate: Option<Activate>,
}

impl Carousel {
    /// Creates a carousel with an element `id`, accessible region `label`, and initial `items`.
    ///
    /// Defaults to [`CarouselVariant::MultiBrowse`], a `220px` height, `192px` preferred item
    /// width, and user scrolling enabled.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = CarouselItem>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: items.into_iter().collect(),
            variant: CarouselVariant::default(),
            height: px(220.),
            preferred_width: px(192.),
            layout_options: LayoutOptions::default(),
            snap: None,
            fling: CarouselFling::Default,
            user_scroll_enabled: true,
            state: None,
            on_activate: None,
        }
    }

    /// Sets the Material carousel layout strategy.
    pub fn variant(mut self, variant: CarouselVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Carousel viewport height. Full-screen pages span this height and the
    /// available width. A host can supply the window's content height.
    pub fn height(mut self, height: Pixels) -> Self {
        if f32::from(height).is_finite() && height > px(0.) {
            self.height = height;
        }
        self
    }

    /// Preferred large item width for multi-browse and regular uncontained.
    pub fn preferred_item_width(mut self, width: Pixels) -> Self {
        if f32::from(width).is_finite() && width > px(0.) {
            self.preferred_width = width;
        }
        self
    }

    /// Sets default snapping for uncontained layouts.
    pub fn snap_scrolling(mut self, snap: bool) -> Self {
        self.snap = Some(snap);
        self
    }

    /// Overrides the resting keyline alignment for contained variants.
    pub fn alignment(mut self, alignment: CarouselAlignment) -> Self {
        self.layout_options.alignment = Some(alignment);
        self
    }

    /// Sets the distance between items in logical pixels.
    pub fn item_spacing(mut self, gap: Pixels) -> Self {
        self.layout_options.spacing = Some(nonnegative(gap));
        self
    }

    /// Sets the content padding around the carousel viewport, clamping negative values to `0`.
    pub fn content_padding(mut self, padding: CarouselPadding) -> Self {
        self.layout_options.padding = Some(CarouselPadding {
            start: px(nonnegative(padding.start)),
            end: px(nonnegative(padding.end)),
            top: px(nonnegative(padding.top)),
            bottom: px(nonnegative(padding.bottom)),
        });
        self
    }

    /// Overrides the ordered positive width bounds for small contained keylines.
    pub fn small_item_widths(mut self, min: Pixels, max: Pixels) -> Self {
        let (min, max) = (nonnegative(min), nonnegative(max));
        if min > 0. && max >= min {
            self.layout_options.small = (min, max);
        }
        self
    }

    /// Overrides the variant's default precise-scroll settling behavior.
    pub fn fling_behavior(mut self, fling: CarouselFling) -> Self {
        self.fling = fling;
        self
    }

    /// Enables wheel, trackpad, keyboard navigation, and accessibility scrolling.
    pub fn user_scroll_enabled(mut self, enabled: bool) -> Self {
        self.user_scroll_enabled = enabled;
        self
    }

    /// Uses a caller-retained entity for this carousel's position.
    pub fn state(mut self, state: &Entity<CarouselState>) -> Self {
        self.state = Some(state.clone());
        self
    }

    /// Registers a callback invoked with the activated item's ID on click or accessibility click.
    pub fn on_activate(
        mut self,
        callback: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Rc::new(callback));
        self
    }
}

impl RenderOnce for Carousel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state.unwrap_or_else(|| {
            window.use_keyed_state((self.id.clone(), "carousel-state"), cx, |_, _| {
                CarouselState::new(0)
            })
        });
        let (target, direct, focus) = state.update(cx, |state, cx| {
            state.sync(&self.items, cx);
            if !self.user_scroll_enabled {
                state.cancel_input(cx);
            }
            (state.requested, state.direct, state.items.clone())
        });
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let ring = color(theme.colors.color(ColorRole::Secondary));
        let hover = color(theme.colors.color(ColorRole::OnSurface));
        let surface = color(theme.colors.color(ColorRole::SurfaceContainer));
        let reduced = cx.reduce_motion();
        let motion = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let sample = animate(
            (self.id.clone(), "carousel-position"),
            target,
            if direct { MotionSpec::Instant } else { motion },
            window,
            cx,
        );
        state.update(cx, |state, cx| state.sample(sample, cx));
        let sample = state.read(cx).sampled;
        let vertical = self.variant.is_vertical();
        let shared_layout = Rc::new(RefCell::new(None));
        let fling = match self.fling {
            CarouselFling::Default if !self.variant.is_uncontained() || self.snap == Some(true) => {
                CarouselFling::Snap
            }
            CarouselFling::Default => CarouselFling::Free,
            explicit => explicit,
        };
        let wheel = Interaction {
            layout: shared_layout.clone(),
            focus: focus.clone(),
            state: state.clone(),
            vertical,
            direction,
            fling,
        };
        let keys = wheel.clone();
        let previous = wheel.clone();
        let next = wheel.clone();
        let (backward, forward) = match (vertical, direction) {
            (true, _) => (AccessibleAction::ScrollUp, AccessibleAction::ScrollDown),
            (false, Direction::Ltr) => {
                (AccessibleAction::ScrollLeft, AccessibleAction::ScrollRight)
            }
            (false, Direction::Rtl) => {
                (AccessibleAction::ScrollRight, AccessibleAction::ScrollLeft)
            }
        };
        let focused = focus.iter().position(|item| item.focus.is_focused(window));
        let tab_stop = focus_at_position(&focus, target);

        div()
            .id(self.id.clone())
            .test_support()
            .role(Role::Region)
            .aria_label(self.label)
            .w_full()
            .min_w_0()
            .relative()
            .overflow_hidden()
            .when(self.user_scroll_enabled, move |region| {
                region
                    .on_key_down(move |event, window, cx| {
                        if event.keystroke.modifiers.modified() {
                            return;
                        }
                        let current = keys.state.read(cx).requested;
                        if let Some(index) = navigation_index(
                            &keys.focus,
                            current,
                            &event.keystroke.key,
                            keys.vertical,
                            keys.direction,
                        ) {
                            keys.state
                                .update(cx, |state, cx| state.animate_scroll_to_item(index, cx));
                            keys.focus[index].focus.focus(window, cx);
                            window.refresh();
                            cx.stop_propagation();
                        }
                    })
                    .on_a11y_action(backward, move |_, window, cx| previous.page(-1, window, cx))
                    .on_a11y_action(forward, move |_, window, cx| next.page(1, window, cx))
                    .on_scroll(move |event, window, cx| wheel.on_scroll(event, window, cx))
            })
            .child(Viewport {
                id: self.id,
                variant: self.variant,
                height: self.height,
                preferred_width: self.preferred_width,
                layout_options: self.layout_options,
                user_scroll_enabled: self.user_scroll_enabled,
                items: self.items.into_iter().map(Some).collect(),
                focus,
                focused,
                tab_stop,
                state,
                sample,
                direction,
                ring,
                hover,
                surface,
                activate: self.on_activate,
                layout: shared_layout,
            })
    }
}

struct Viewport {
    id: ElementId,
    variant: CarouselVariant,
    height: Pixels,
    preferred_width: Pixels,
    layout_options: LayoutOptions,
    user_scroll_enabled: bool,
    items: Vec<Option<CarouselItem>>,
    focus: Rc<[ItemFocus]>,
    focused: Option<usize>,
    tab_stop: Option<usize>,
    state: Entity<CarouselState>,
    sample: f32,
    direction: Direction,
    ring: gpui::Hsla,
    hover: gpui::Hsla,
    surface: gpui::Hsla,
    activate: Option<Activate>,
    layout: Rc<RefCell<Option<Layout>>>,
}

struct Prepaint {
    children: Vec<AnyElement>,
    mask: Bounds<Pixels>,
}

impl IntoElement for Viewport {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for Viewport {
    type RequestLayoutState = ();
    type PrepaintState = Prepaint;

    fn id(&self) -> Option<ElementId> {
        Some((self.id.clone(), "viewport").into())
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
    ) -> (LayoutId, ()) {
        let style = Style {
            size: size(relative(1.).into(), self.height.into()),
            ..Style::default()
        };
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let vertical = self.variant.is_vertical();
        let layout = layout(
            self.variant,
            bounds.size.width,
            bounds.size.height,
            self.preferred_width,
            self.layout_options,
            self.items.iter().flatten(),
        );
        let (inset, cross) = layout.cross_bounds(self.direction);
        let (inset, cross) = (px(inset), px(cross));
        let mut visible = layout.visible(self.sample, self.direction);
        // When keyboard or UI Automation focuses an item outside the current
        // mask, reveal it in this frame rather than leaving an invisible focus
        // target during a long animation.
        if let Some(focused) = self.focused
            && !self.state.read(cx).direct
            && self.state.read(cx).requested.round() as usize == focused
            && !visible.iter().any(|(index, item)| {
                *index == focused
                    && item.size >= 40.
                    && item.start >= -0.5
                    && item.start + item.size
                        <= f32::from(if vertical {
                            bounds.size.height
                        } else {
                            bounds.size.width
                        }) + 0.5
            })
        {
            visible = layout.visible(focused as f32, self.direction);
        }
        let (start, end) = layout.clip_range(self.direction);
        *self.layout.borrow_mut() = Some(layout);
        let mask = if vertical {
            Bounds::new(
                bounds.origin + point(inset, px(start)),
                size(cross, px(end - start)),
            )
        } else {
            Bounds::new(
                bounds.origin + point(px(start), inset),
                size(px(end - start), cross),
            )
        };
        let mut children = Vec::with_capacity(visible.len());
        let count = self.items.len();
        window.with_content_mask(Some(ContentMask { bounds: mask }), |window| {
            for (index, geometry) in visible {
                let Some(item) = self.items[index].take() else {
                    continue;
                };
                let (size, offset) = if vertical {
                    (
                        size(cross, px(geometry.size)),
                        point(inset, px(geometry.start)),
                    )
                } else {
                    (
                        size(px(geometry.size), cross),
                        point(px(geometry.start), inset),
                    )
                };
                let radius = if vertical {
                    0.
                } else {
                    CornerRole::ExtraLarge.shape().radii(
                        f32::from(size.width),
                        f32::from(size.height),
                        self.direction,
                    )[0]
                };
                let id = item.id;
                let activate_id = id.clone();
                let accessible_id = id.clone();
                let activate = self.activate.clone();
                let activate_accessible = activate.clone();
                let state = self.state.clone();
                let accessible_state = state.clone();
                let accessible_focus = self.focus[index].focus.clone();
                let user_scroll_enabled = self.user_scroll_enabled;
                let content = match item.content {
                    Content::Element(content) => content,
                    Content::Image(source) => img(source)
                        .size_full()
                        .object_fit(ObjectFit::Cover)
                        .rounded(px(radius))
                        .into_any_element(),
                };
                let focused = self.focused == Some(index) && window.last_input_was_keyboard();
                let mut child = Button::new((self.id.clone(), id))
                    .accessibility_label(item.label)
                    .aria_position_in_set(index + 1)
                    .aria_size_of_set(count)
                    .track_focus(&self.focus[index].focus)
                    .tab_stop(self.tab_stop == Some(index))
                    .disabled(item.disabled)
                    .w(size.width)
                    .h(size.height)
                    .bg(self.surface)
                    .rounded(px(radius))
                    .overflow_hidden()
                    .relative()
                    .group("carousel-item")
                    .when(!item.disabled, |button| {
                        button
                            .on_click(move |_, window, cx| {
                                if user_scroll_enabled {
                                    state.update(cx, |state, cx| {
                                        state.animate_scroll_to_item(index, cx)
                                    });
                                }
                                window.refresh();
                                if let Some(activate) = &activate {
                                    activate(&activate_id, window, cx);
                                }
                            })
                            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                if user_scroll_enabled {
                                    accessible_state.update(cx, |state, cx| {
                                        state.animate_scroll_to_item(index, cx)
                                    });
                                }
                                accessible_focus.focus(window, cx);
                                window.refresh();
                                if let Some(activate) = &activate_accessible {
                                    activate(&accessible_id, window, cx);
                                }
                            })
                    })
                    .child(content)
                    .child(
                        div()
                            .id("state-layer")
                            .absolute()
                            .inset_0()
                            .rounded(px(radius))
                            .when(!item.disabled, |state| {
                                state
                                    .group_hover("carousel-item", |style| {
                                        style.bg(self.hover.opacity(0.08))
                                    })
                                    .group_active("carousel-item", |style| {
                                        style.bg(self.hover.opacity(0.1))
                                    })
                                    .when(focused, |state| state.border_2().border_color(self.ring))
                            }),
                    )
                    .into_any_element();
                child.prepaint_as_root(bounds.origin + offset, size.map(Into::into), window, cx);
                children.push(child);
            }
        });
        Prepaint { children, mask }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        state: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_content_mask(Some(ContentMask { bounds: state.mask }), |window| {
            for child in &mut state.children {
                child.paint(window, cx);
            }
        });
    }
}

fn layout<'a>(
    variant: CarouselVariant,
    width: Pixels,
    height: Pixels,
    preferred_width: Pixels,
    options: LayoutOptions,
    items: impl IntoIterator<Item = &'a CarouselItem>,
) -> Layout {
    let (main, cross) = if variant.is_vertical() {
        (height, width)
    } else {
        (width, height)
    };
    Layout::new(
        variant,
        main.into(),
        cross.into(),
        preferred_width.into(),
        options,
        items.into_iter().map(|item| item.aspect),
    )
}

#[derive(Clone)]
struct Interaction {
    layout: Rc<RefCell<Option<Layout>>>,
    focus: Rc<[ItemFocus]>,
    state: Entity<CarouselState>,
    vertical: bool,
    direction: Direction,
    fling: CarouselFling,
}

impl Interaction {
    fn page(&self, delta: isize, window: &mut Window, cx: &mut App) {
        let state = self.state.read(cx);
        let index = (state.requested.round() as usize).saturating_add_signed(delta);
        if index < self.focus.len() {
            self.state
                .update(cx, |state, cx| state.animate_scroll_to_item(index, cx));
            window.refresh();
        }
    }

    fn on_scroll(&self, event: &ScrollEvent, window: &mut Window, cx: &mut App) {
        let delta = event.delta.pixel_delta(px(32.));
        let x = f32::from(delta.x);
        let y = f32::from(delta.y);
        let horizontal_input = if event.modifiers.shift && x.abs() < y.abs() {
            y
        } else {
            x
        };
        let main = if self.vertical {
            if x.abs() > y.abs() {
                return;
            }
            y
        } else {
            if !event.modifiers.shift && y.abs() > x.abs() {
                return;
            }
            if self.direction == Direction::Rtl {
                -horizontal_input
            } else {
                horizontal_input
            }
        };
        if self.focus.len() < 2 || !main.is_finite() {
            return;
        }

        let moved = self.state.update(cx, |state, cx| {
            if event.touch_phase == TouchPhase::Cancelled {
                return finish_gesture(state, &self.fling, true, cx);
            }

            let previous = state.requested;
            let limit = (self.focus.len() - 1) as f32;
            if !event.delta.precise() {
                let sign = if main == 0. { 0. } else { main.signum() };
                let target = (previous - sign).clamp(0., limit);
                if target == previous {
                    return false;
                }
                state.animate_scroll_to_item(target.round() as usize, cx);
                return true;
            }

            let delta = {
                let layout = self.layout.borrow();
                let Some(layout) = layout.as_ref() else {
                    return false;
                };
                layout.scroll_delta(previous, -main)
            };
            let target = (previous + delta).clamp(0., limit);
            if target != previous {
                state.gesture_start.get_or_insert(previous);
                state.set_requested(target, true, cx);
            }

            if event.touch_phase == TouchPhase::Ended {
                return finish_gesture(state, &self.fling, false, cx);
            }
            if target == previous {
                return false;
            }

            // Some desktop drivers omit the terminal phase of a precise gesture.
            let timer = cx.background_executor().timer(SCROLL_SETTLE);
            let weak_state = cx.weak_entity();
            let fling = self.fling.clone();
            state.settle = Some(window.spawn(cx, async move |cx| {
                timer.await;
                let _ = cx.update(|window, cx| {
                    if weak_state
                        .update(cx, |state, cx| finish_gesture(state, &fling, false, cx))
                        .unwrap_or(false)
                    {
                        window.refresh();
                    }
                });
            }));
            true
        });
        if !moved {
            return;
        }

        // A pointer scroll can move already-contained focus to a visible item.
        let target = self.state.read(cx).requested;
        let viewport = {
            let layout = self.layout.borrow();
            layout
                .as_ref()
                .map(|layout| layout.visible(target, self.direction))
                .unwrap_or_default()
        };
        let current_focus = self
            .focus
            .iter()
            .position(|item| item.focus.is_focused(window));
        if current_focus.is_some_and(|index| !viewport.iter().any(|(visible, _)| *visible == index))
            && let Some((index, _)) = viewport
                .iter()
                .filter(|(index, _)| self.focus[*index].enabled)
                .max_by(|(_, a), (_, b)| a.size.total_cmp(&b.size))
        {
            self.focus[*index].focus.focus(window, cx);
        }
        window.refresh();
        cx.stop_propagation();
    }
}

fn finish_gesture(
    state: &mut CarouselState,
    fling: &CarouselFling,
    cancelled: bool,
    cx: &mut gpui::Context<CarouselState>,
) -> bool {
    let Some(start) = state.gesture_start.take() else {
        return false;
    };
    state.settle = None;
    let position = state.requested;
    let resting_item = if cancelled {
        if matches!(fling, CarouselFling::Free) {
            state.set_requested(start, true, cx);
            None
        } else {
            Some(start.round() as usize)
        }
    } else {
        let item = position.round() as usize;
        let initial = start.round() as usize;
        match fling {
            CarouselFling::Free => None,
            CarouselFling::Default | CarouselFling::Snap => Some(item),
            CarouselFling::SingleAdvance => {
                Some(item.clamp(initial.saturating_sub(1), initial + 1))
            }
            CarouselFling::Custom(target) => target(CarouselFlingContext {
                start,
                position,
                item_count: state.items.len(),
            }),
        }
    };
    if let Some(item) = resting_item {
        state.animate_scroll_to_item(item, cx);
    }
    true
}

fn focus_at_position(items: &[ItemFocus], position: f32) -> Option<usize> {
    let index = (position.round() as usize).min(items.len().saturating_sub(1));
    (index..items.len())
        .chain((0..index).rev())
        .find(|index| items[*index].enabled)
}

fn navigation_index(
    items: &[ItemFocus],
    position: f32,
    key: &str,
    vertical: bool,
    direction: Direction,
) -> Option<usize> {
    if items.is_empty() {
        return None;
    }
    let current = (position.round() as usize).min(items.len() - 1);
    let forward = match key {
        "home" => return items.iter().position(|item| item.enabled),
        "end" => return items.iter().rposition(|item| item.enabled),
        "up" if vertical => false,
        "down" if vertical => true,
        "left" if !vertical => direction == Direction::Rtl,
        "right" if !vertical => direction == Direction::Ltr,
        _ => return None,
    };
    if forward {
        (current + 1..items.len()).find(|index| items[*index].enabled)
    } else {
        (0..current).rev().find(|index| items[*index].enabled)
    }
}
