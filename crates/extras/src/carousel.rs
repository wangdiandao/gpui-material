//! Material 3 carousel strategies and desktop navigation.

mod geometry;
mod interaction;
mod parallax;
mod state;
mod viewport;

use std::{cell::RefCell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, ElementId, Entity, ImageSource, InteractiveElement,
    IntoElement, ParentElement, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement,
    Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{animate, color, current_material_theme};
use gpui_material_theme::{
    ColorRole, Direction, LogicalEdges, MotionKind, MotionSpec, MotionSpeed,
};

use geometry::LayoutOptions;
use interaction::{Interaction, focus_at_position, navigation_index};
use viewport::Viewport;

pub use parallax::CarouselParallax;
pub use state::CarouselState;

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
    pub(crate) fn is_vertical(self) -> bool {
        self == Self::FullScreen
    }

    pub(crate) fn is_uncontained(self) -> bool {
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
    nonnegative_f32(f32::from(value))
}

fn nonnegative_f32(value: f32) -> f32 {
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
    pub fn content_padding(mut self, padding: LogicalEdges) -> Self {
        self.layout_options.padding = Some(LogicalEdges {
            start: nonnegative_f32(padding.start),
            end: nonnegative_f32(padding.end),
            top: nonnegative_f32(padding.top),
            bottom: nonnegative_f32(padding.bottom),
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
