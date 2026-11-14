//! Material 3 carousel strategies and desktop navigation.

mod geometry;

use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    AccessibleAction, AnyElement, App, Bounds, ContentMask, Element, ElementId, FocusHandle,
    GlobalElementId, ImageSource, InspectorElementId, InteractiveElement, IntoElement, LayoutId,
    ObjectFit, ParentElement, Pixels, RenderOnce, Role, ScrollEvent, SharedString,
    StatefulInteractiveElement, Style, Styled, StyledImage, Task, TouchPhase, Window, div, img,
    point, prelude::FluentBuilder, px, relative, size,
};
use gpui_base::{Button, TestSupportExt};
use gpui_material_foundation::{animate, color, window_material_theme};
use gpui_material_theme::{ColorRole, CornerRole, Direction, MotionKind, MotionSpec, MotionSpeed};

use geometry::Layout;

const SCROLL_SETTLE: Duration = Duration::from_millis(120);
const WIDTH_TO_HEIGHT_MIN: f32 = 9. / 16.;
const WIDTH_TO_HEIGHT_MAX: f32 = 16. / 9.;

/// Material carousel layouts. Full-screen pages scroll vertically.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselVariant {
    #[default]
    MultiBrowse,
    Uncontained,
    UncontainedMultiAspectRatio,
    Hero,
    CenteredHero,
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
    snap: Option<bool>,
    on_activate: Option<Activate>,
}

impl Carousel {
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
            snap: None,
            on_activate: None,
        }
    }

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

    /// Sets snapping for uncontained layouts. Other layouts always snap.
    pub fn snap_scrolling(mut self, snap: bool) -> Self {
        self.snap = Some(snap);
        self
    }

    pub fn on_activate(
        mut self,
        callback: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Rc::new(callback));
        self
    }
}

struct ItemFocus {
    id: SharedString,
    enabled: bool,
    focus: FocusHandle,
}

#[derive(Default)]
struct Position {
    current: Option<SharedString>,
    requested: f32,
    direct: bool,
    settle: Option<Task<()>>,
    gesture_start: Option<f32>,
}

impl Position {
    fn sync(&mut self, items: &[ItemFocus]) -> f32 {
        let index = self
            .current
            .as_ref()
            .and_then(|id| items.iter().position(|item| &item.id == id))
            .unwrap_or_else(|| {
                (self.requested.round() as usize).min(items.len().saturating_sub(1))
            });
        let current = items.get(index).map(|item| item.id.clone());
        if current != self.current || index as f32 != self.requested.round() {
            self.current = current;
            self.requested = index as f32;
            self.direct = false;
            self.settle = None;
        }
        self.requested
    }

    fn select(&mut self, index: usize, item: &ItemFocus) {
        self.current = Some(item.id.clone());
        self.requested = index as f32;
        self.direct = false;
        self.settle = None;
        self.gesture_start = None;
    }

    fn scroll_to(&mut self, position: f32, items: &[ItemFocus]) {
        self.requested = position;
        self.direct = true;
        self.current = items
            .get(position.round() as usize)
            .map(|item| item.id.clone());
    }
}

struct CarouselState {
    position: Rc<RefCell<Position>>,
    items: Rc<[ItemFocus]>,
    width: Rc<RefCell<Pixels>>,
}

impl Default for CarouselState {
    fn default() -> Self {
        Self {
            position: Rc::default(),
            items: Rc::from([]),
            width: Rc::default(),
        }
    }
}

impl RenderOnce for Carousel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state((self.id.clone(), "carousel-state"), cx, |_, _| {
            CarouselState::default()
        });
        let (position, focus, width) = state.update(cx, |state, cx| {
            if state.items.len() != self.items.len()
                || state
                    .items
                    .iter()
                    .zip(&self.items)
                    .any(|(known, item)| known.id != item.id || known.enabled == item.disabled)
            {
                let previous: std::collections::HashMap<_, _> = state
                    .items
                    .iter()
                    .map(|item| (&item.id, &item.focus))
                    .collect();
                state.items = self
                    .items
                    .iter()
                    .map(|item| ItemFocus {
                        id: item.id.clone(),
                        enabled: !item.disabled,
                        focus: previous
                            .get(&item.id)
                            .map_or_else(|| cx.focus_handle(), |focus| (*focus).clone()),
                    })
                    .collect();
            }
            (
                state.position.clone(),
                state.items.clone(),
                state.width.clone(),
            )
        });
        let theme = window_material_theme(window, cx);
        let direction = theme.layout.direction;
        let ring = color(theme.colors.color(ColorRole::Secondary));
        let hover = color(theme.colors.color(ColorRole::OnSurface));
        let surface = color(theme.colors.color(ColorRole::SurfaceContainer));
        let reduced = cx.reduce_motion();
        let motion = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let (target, direct) = {
            let mut position = position.borrow_mut();
            let target = position.sync(&focus);
            (target, position.direct)
        };
        let sample = animate(
            (self.id.clone(), "carousel-position"),
            target,
            if direct { MotionSpec::Instant } else { motion },
            window,
            cx,
        );
        let vertical = self.variant.is_vertical();
        let measured_width = *width.borrow();
        let fallback_width = if measured_width > px(0.) {
            measured_width
        } else {
            window.viewport_size().width
        };
        let layout = Rc::new(layout(
            self.variant,
            fallback_width,
            self.height,
            self.preferred_width,
            &self.items,
        ));
        let snap = !self.variant.is_uncontained() || self.snap.unwrap_or(false);
        let wheel = Interaction {
            layout,
            focus: focus.clone(),
            position: position.clone(),
            vertical,
            direction,
            snap,
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
        let view = window.current_view();

        div()
            .id(self.id.clone())
            .test_support()
            .role(Role::Region)
            .aria_label(self.label)
            .w_full()
            .min_w_0()
            .relative()
            .overflow_hidden()
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let current = keys.position.borrow().requested;
                if let Some(index) = navigation_index(
                    &keys.focus,
                    current,
                    &event.keystroke.key,
                    keys.vertical,
                    keys.direction,
                ) {
                    keys.position.borrow_mut().select(index, &keys.focus[index]);
                    keys.focus[index].focus.focus(window, cx);
                    window.refresh();
                    cx.stop_propagation();
                }
            })
            .on_a11y_action(backward, move |_, window, _| {
                previous.page(-1, window);
            })
            .on_a11y_action(forward, move |_, window, _| {
                next.page(1, window);
            })
            .on_scroll(move |event, window, cx| wheel.on_scroll(event, window, cx))
            .child(Viewport {
                id: self.id,
                variant: self.variant,
                height: self.height,
                preferred_width: self.preferred_width,
                items: self.items.into_iter().map(Some).collect(),
                focus,
                focused,
                tab_stop,
                position,
                sample,
                direction,
                ring,
                hover,
                surface,
                activate: self.on_activate,
                width,
                view,
            })
    }
}

struct Viewport {
    id: ElementId,
    variant: CarouselVariant,
    height: Pixels,
    preferred_width: Pixels,
    items: Vec<Option<CarouselItem>>,
    focus: Rc<[ItemFocus]>,
    focused: Option<usize>,
    tab_stop: Option<usize>,
    position: Rc<RefCell<Position>>,
    sample: f32,
    direction: Direction,
    ring: gpui::Hsla,
    hover: gpui::Hsla,
    surface: gpui::Hsla,
    activate: Option<Activate>,
    width: Rc<RefCell<Pixels>>,
    view: gpui::EntityId,
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
        let previous = self.width.replace(bounds.size.width);
        if previous != bounds.size.width {
            let view = self.view;
            window.on_next_frame(move |window, cx| {
                cx.notify(view);
                window.refresh();
            });
        }
        let vertical = self.variant.is_vertical();
        let layout = layout(
            self.variant,
            bounds.size.width,
            bounds.size.height,
            self.preferred_width,
            self.items.iter().flatten(),
        );
        let inset = layout.cross_inset();
        let cross = if vertical {
            bounds.size.width
        } else {
            bounds.size.height
        } - px(inset * 2.);
        let mut visible = layout.visible(self.sample, self.direction);
        // When keyboard or UI Automation focuses an item outside the current
        // mask, reveal it in this frame rather than leaving an invisible focus
        // target during a long animation.
        if let Some(focused) = self.focused
            && !self.position.borrow().direct
            && self.position.borrow().current.as_ref() == Some(&self.focus[focused].id)
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
        let (start, end) = layout.clip_range();
        let mask = if vertical {
            Bounds::new(
                bounds.origin + point(px(0.), px(start)),
                size(cross, px(end - start)),
            )
        } else {
            Bounds::new(
                bounds.origin + point(px(start), px(0.)),
                size(px(end - start), bounds.size.height),
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
                        point(px(inset), px(geometry.start)),
                    )
                } else {
                    (
                        size(px(geometry.size), cross),
                        point(px(geometry.start), px(inset)),
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
                let position = self.position.clone();
                let accessible_position = position.clone();
                let focus = self.focus.clone();
                let accessible_focus = focus.clone();
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
                                position.borrow_mut().select(index, &focus[index]);
                                window.refresh();
                                if let Some(activate) = &activate {
                                    activate(&activate_id, window, cx);
                                }
                            })
                            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                                accessible_position
                                    .borrow_mut()
                                    .select(index, &accessible_focus[index]);
                                accessible_focus[index].focus.focus(window, cx);
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
        items.into_iter().map(|item| item.aspect),
    )
}

#[derive(Clone)]
struct Interaction {
    layout: Rc<Layout>,
    focus: Rc<[ItemFocus]>,
    position: Rc<RefCell<Position>>,
    vertical: bool,
    direction: Direction,
    snap: bool,
}

impl Interaction {
    fn page(&self, delta: isize, window: &mut Window) {
        let mut position = self.position.borrow_mut();
        let index = (position.requested.round() as usize).saturating_add_signed(delta);
        if let Some(item) = self.focus.get(index) {
            position.select(index, item);
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
            horizontal_input
                * if self.direction == Direction::Rtl {
                    -1.
                } else {
                    1.
                }
        };
        if self.focus.len() < 2 {
            return;
        }
        let mut position = self.position.borrow_mut();
        let previous = position.requested;
        let limit = (self.focus.len() - 1) as f32;
        let target = match event.touch_phase {
            TouchPhase::Cancelled => {
                let Some(start) = position.gesture_start.take() else {
                    return;
                };
                start.clamp(0., limit)
            }
            _ if event.delta.precise() => {
                (previous + self.layout.scroll_delta(previous, -main)).clamp(0., limit)
            }
            _ => (previous - main.signum() * f32::from(main != 0.)).clamp(0., limit),
        };
        let terminal = event.touch_phase == TouchPhase::Ended && position.direct && self.snap;
        if target == previous && !terminal {
            return;
        }
        if event.touch_phase == TouchPhase::Cancelled && !self.snap {
            position.scroll_to(target, &self.focus);
        } else if !event.delta.precise() || terminal || event.touch_phase == TouchPhase::Cancelled {
            let index = target.round() as usize;
            position.select(index, &self.focus[index]);
        } else {
            position.gesture_start.get_or_insert(previous);
            position.scroll_to(target, &self.focus);
            if self.snap {
                let timer = cx.background_executor().timer(SCROLL_SETTLE);
                let state = Rc::downgrade(&self.position);
                let focus = self.focus.clone();
                position.settle = Some(window.spawn(cx, async move |cx| {
                    timer.await;
                    let _ = cx.update(|window, _| {
                        if let Some(state) = state.upgrade() {
                            let mut state = state.borrow_mut();
                            let index = state.requested.round() as usize;
                            state.select(index, &focus[index]);
                            window.refresh();
                        }
                    });
                }));
            }
        }
        // If focus was already inside the carousel, keep it within the
        // visible range. A pointer scroll never steals outside focus.
        let viewport = self.layout.visible(target, self.direction);
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
        drop(position);
        window.refresh();
        cx.stop_propagation();
    }
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
