use std::{ops::Range, rc::Rc};

use gpui::{
    App, Axis, Bounds, Context, ElementId, Entity, FocusHandle, Hsla, InteractiveElement,
    IntoElement, KeyDownEvent, ListSizingBehavior, ParentElement, Pixels, Point, Render,
    RenderOnce, Role, ScrollHandle, ScrollStrategy, SharedString, Size, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, canvas, div, point, prelude::FluentBuilder, px,
    transparent_black,
};
use gpui_base::{
    StyledExt as _, TestSupportExt as _, VirtualList as BaseVirtualList,
    VirtualListScrollHandle as BaseVirtualListScrollHandle, virtual_list as base_virtual_list,
};
use gpui_material_components::{
    DisplayScrollbar, DisplayScrollbarFade, DisplayScrollbarStyle, ScrollbarHandle,
};
use gpui_material_foundation::{
    FocusRingPlacement, InteractionFeedback, InteractionState, Shape, color,
    current_material_theme, resolve_edges,
};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Density, Direction, LogicalEdges};

const DEFAULT_KEYBOARD_STEP: f32 = 48.0;
const MIN_KEYBOARD_STEP: f32 = 32.0;

/// Retained scroll handle for [`VirtualList`].
///
/// Wraps `gpui-base`'s virtual list scroll state and implements [`ScrollbarHandle`]
/// so it integrates directly with [`DisplayScrollbar`] and programmatic scrolling.
#[derive(Clone)]
pub struct VirtualListScrollHandle {
    inner: BaseVirtualListScrollHandle,
}

impl Default for VirtualListScrollHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ScrollHandle> for VirtualListScrollHandle {
    fn from(handle: ScrollHandle) -> Self {
        Self {
            inner: BaseVirtualListScrollHandle::from(handle),
        }
    }
}

impl AsRef<ScrollHandle> for VirtualListScrollHandle {
    fn as_ref(&self) -> &ScrollHandle {
        self.inner.as_ref()
    }
}

impl ScrollbarHandle for VirtualListScrollHandle {
    fn viewport_bounds(&self) -> Bounds<Pixels> {
        self.inner.viewport_bounds()
    }

    fn offset(&self) -> Point<Pixels> {
        self.inner.offset()
    }

    fn set_offset(&self, offset: Point<Pixels>) {
        self.inner.set_offset(offset);
    }

    fn content_size(&self) -> Size<Pixels> {
        self.inner.content_size()
    }
}

impl VirtualListScrollHandle {
    /// Creates a new scroll handle for a [`VirtualList`].
    pub fn new() -> Self {
        Self {
            inner: BaseVirtualListScrollHandle::new(),
        }
    }

    /// Returns the underlying GPUI [`ScrollHandle`].
    pub fn base_handle(&self) -> &ScrollHandle {
        self.inner.base_handle()
    }

    /// Scrolls the list so the item at `index` is positioned according to `strategy`.
    pub fn scroll_to_item(&self, index: usize, strategy: ScrollStrategy) {
        self.inner.scroll_to_item(index, strategy);
    }

    /// Scrolls to the end of the list.
    pub fn scroll_to_bottom(&self) {
        self.inner.scroll_to_bottom();
    }

    /// Returns the current scroll offset in logical pixels.
    pub fn offset(&self) -> Point<Pixels> {
        self.inner.offset()
    }

    /// Sets the scroll offset in logical pixels.
    pub fn set_offset(&self, offset: Point<Pixels>) {
        self.inner.set_offset(offset);
    }

    /// Returns the measured total content size in logical pixels.
    pub fn content_size(&self) -> Size<Pixels> {
        self.inner.content_size()
    }

    /// Returns the viewport bounds from the most recent frame.
    pub fn viewport_bounds(&self) -> Bounds<Pixels> {
        self.inner.viewport_bounds()
    }
}

/// Surface presentation variant for [`VirtualList`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VirtualListVariant {
    /// Transparent scroll container suitable for embedding inside an existing surface (default).
    #[default]
    Standard,
    /// Material 3 container with `SurfaceContainerLow` fill, `OutlineVariant` border, and `Medium` rounded corners.
    Contained,
}

/// Optional per-instance color overrides for [`VirtualList`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VirtualListColors {
    /// Container background fill (defaults to transparent for `Standard` or `SurfaceContainerLow` for `Contained`).
    pub container: Option<Hsla>,
    /// Container border color (defaults to transparent for `Standard` or `OutlineVariant` for `Contained`).
    pub border: Option<Hsla>,
}

/// Material 3 Expressive virtualized list container over `gpui-base`'s variable-size [`BaseVirtualList`].
///
/// Automatically mounts a Material [`DisplayScrollbar`] along the active scroll axis, respects
/// `theme.layout.direction` (RTL scrollbar placement and horizontal keyboard scrolling) and
/// `theme.layout.density`, and provides keyboard scrolling (`Up`/`Down`/`PageUp`/`PageDown`/`Home`/`End`
/// vertically, `Left`/`Right`/`Home`/`End` horizontally) with a Material keyboard focus ring.
#[derive(IntoElement)]
pub struct VirtualList {
    id: ElementId,
    axis: Axis,
    base: BaseVirtualList,
    scroll_handle: Option<VirtualListScrollHandle>,
    focus_handle: Option<FocusHandle>,
    variant: VirtualListVariant,
    shape: Option<CornerShape>,
    colors: VirtualListColors,
    content_padding: LogicalEdges,
    item_gap: Pixels,
    show_scrollbar: bool,
    scrollbar_style: DisplayScrollbarStyle,
    scrollbar_fade: DisplayScrollbarFade,
    accessible_label: Option<SharedString>,
    keyboard_scroll_step: Option<Pixels>,
    style: StyleRefinement,
}

impl VirtualList {
    /// Creates a vertical [`VirtualList`] for variable-height rows.
    pub fn vertical<R, V>(
        view: Entity<V>,
        id: impl Into<ElementId>,
        item_sizes: Rc<Vec<Size<Pixels>>>,
        render_range: impl 'static + Fn(&mut V, Range<usize>, &mut Window, &mut Context<V>) -> Vec<R>,
    ) -> Self
    where
        R: IntoElement,
        V: Render,
    {
        Self::new(view, id, Axis::Vertical, item_sizes, render_range)
    }

    /// Creates a horizontal [`VirtualList`] for variable-width columns.
    pub fn horizontal<R, V>(
        view: Entity<V>,
        id: impl Into<ElementId>,
        item_sizes: Rc<Vec<Size<Pixels>>>,
        render_range: impl 'static + Fn(&mut V, Range<usize>, &mut Window, &mut Context<V>) -> Vec<R>,
    ) -> Self
    where
        R: IntoElement,
        V: Render,
    {
        Self::new(view, id, Axis::Horizontal, item_sizes, render_range)
    }

    /// Creates a [`VirtualList`] along `axis` for variable-size items.
    pub fn new<R, V>(
        view: Entity<V>,
        id: impl Into<ElementId>,
        axis: Axis,
        item_sizes: Rc<Vec<Size<Pixels>>>,
        render_range: impl 'static + Fn(&mut V, Range<usize>, &mut Window, &mut Context<V>) -> Vec<R>,
    ) -> Self
    where
        R: IntoElement,
        V: Render,
    {
        let id: ElementId = id.into();
        let inner_id = ElementId::from((id.clone(), "viewport"));
        let base = base_virtual_list(view, inner_id, axis, item_sizes, render_range);
        Self {
            id,
            axis,
            base,
            scroll_handle: None,
            focus_handle: None,
            variant: VirtualListVariant::Standard,
            shape: None,
            colors: VirtualListColors::default(),
            content_padding: LogicalEdges::ZERO,
            item_gap: px(0.),
            show_scrollbar: true,
            scrollbar_style: DisplayScrollbarStyle::default(),
            scrollbar_fade: DisplayScrollbarFade::default(),
            accessible_label: None,
            keyboard_scroll_step: None,
            style: StyleRefinement::default(),
        }
    }

    /// Binds a caller-retained [`VirtualListScrollHandle`] for programmatic scrolling and observation.
    pub fn track_scroll(mut self, scroll_handle: &VirtualListScrollHandle) -> Self {
        self.scroll_handle = Some(scroll_handle.clone());
        self
    }

    /// Attaches a caller-owned [`FocusHandle`] for keyboard scrolling and focus management.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.focus_handle = Some(focus.clone());
        self
    }

    /// Sets the surface presentation variant (`Standard` or `Contained`).
    pub fn variant(mut self, variant: VirtualListVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the container corner shape (defaults to `CornerRole::None` for `Standard` or
    /// `CornerRole::Medium` for `Contained`).
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Applies per-instance container and border color overrides.
    pub fn colors(mut self, colors: VirtualListColors) -> Self {
        self.colors = colors;
        self
    }

    /// Sets direction-aware inner padding inside the scrolling viewport.
    pub fn content_padding(mut self, padding: LogicalEdges) -> Self {
        self.content_padding = padding.sanitize();
        self
    }

    /// Sets the spacing gap between virtualized items along the scroll axis.
    pub fn item_gap(mut self, gap: Pixels) -> Self {
        if f32::from(gap).is_finite() && gap >= px(0.) {
            self.item_gap = gap;
        }
        self
    }

    /// Configures how the underlying virtual list sizes itself when unconstrained.
    pub fn sizing_behavior(mut self, behavior: ListSizingBehavior) -> Self {
        self.base = self.base.with_sizing_behavior(behavior);
        self
    }

    /// Sets the item index measured to infer cross-axis content extent.
    pub fn item_to_measure_index(mut self, index: usize) -> Self {
        self.base = self.base.with_item_to_measure_index(index);
        self
    }

    /// Shows or hides the Material [`DisplayScrollbar`] overlay (defaults to `true`).
    pub fn show_scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }

    /// Customizes the [`DisplayScrollbar`] dimensions and colors.
    pub fn scrollbar_style(mut self, style: DisplayScrollbarStyle) -> Self {
        self.scrollbar_style = style;
        self
    }

    /// Customizes the [`DisplayScrollbar`] auto-hide fade behavior.
    pub fn scrollbar_fade(mut self, fade: DisplayScrollbarFade) -> Self {
        self.scrollbar_fade = fade;
        self
    }

    /// Sets the accessible name announced for the `Role::List` container.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Overrides the per-keypress scroll distance for arrow keys (defaults to a density-aware step).
    pub fn keyboard_scroll_step(mut self, step: Pixels) -> Self {
        if f32::from(step).is_finite() && step > px(0.) {
            self.keyboard_scroll_step = Some(step);
        }
        self
    }
}

impl Styled for VirtualList {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

fn keyboard_step(density: Density, override_step: Option<Pixels>) -> Pixels {
    override_step.unwrap_or_else(|| {
        px(density
            .visual_size(DEFAULT_KEYBOARD_STEP)
            .max(MIN_KEYBOARD_STEP))
    })
}

fn compute_keyboard_scroll_offset(
    axis: Axis,
    direction: Direction,
    key: &str,
    current: Point<Pixels>,
    viewport: Size<Pixels>,
    content: Size<Pixels>,
    step: Pixels,
) -> Option<Point<Pixels>> {
    match axis {
        Axis::Vertical => {
            let max_scroll = (content.height - viewport.height).max(px(0.));
            let min_offset = -max_scroll;
            let page = (viewport.height * 0.85).max(step);
            let next_y = match key {
                "up" => (current.y + step).min(px(0.)),
                "down" => (current.y - step).max(min_offset),
                "pageup" => (current.y + page).min(px(0.)),
                "pagedown" => (current.y - page).max(min_offset),
                "home" => px(0.),
                "end" => min_offset,
                _ => return None,
            };
            Some(point(current.x, next_y.clamp(min_offset, px(0.))))
        }
        Axis::Horizontal => {
            let max_scroll = (content.width - viewport.width).max(px(0.));
            let min_offset = -max_scroll;
            let rtl = direction == Direction::Rtl;
            let page = (viewport.width * 0.85).max(step);
            let backward = (key == "left" && !rtl) || (key == "right" && rtl);
            let forward = (key == "right" && !rtl) || (key == "left" && rtl);
            let next_x = if backward {
                (current.x + step).min(px(0.))
            } else if forward {
                (current.x - step).max(min_offset)
            } else {
                match key {
                    "pageup" => (current.x + page).min(px(0.)),
                    "pagedown" => (current.x - page).max(min_offset),
                    "home" => px(0.),
                    "end" => min_offset,
                    _ => return None,
                }
            };
            Some(point(next_x.clamp(min_offset, px(0.)), current.y))
        }
    }
}

impl RenderOnce for VirtualList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx).clone();
        let direction = theme.layout.direction;
        let density = theme.layout.density;
        let reduced_motion = cx.reduce_motion();
        let tone = |role| color(theme.colors.color(role));

        let (default_bg, default_border, default_shape) = match self.variant {
            VirtualListVariant::Standard => (
                transparent_black(),
                transparent_black(),
                CornerRole::None.shape(),
            ),
            VirtualListVariant::Contained => (
                tone(ColorRole::SurfaceContainerLow),
                tone(ColorRole::OutlineVariant),
                CornerRole::Medium.shape(),
            ),
        };
        let bg = self.colors.container.unwrap_or(default_bg);
        let border = self.colors.border.unwrap_or(default_border);
        let has_border = border.a > 0.0;
        let shape = self.shape.unwrap_or(default_shape);
        let radii = shape.radii(100.0, 100.0, direction);
        let focus_ring_color = tone(ColorRole::Secondary);

        let scroll_handle = self.scroll_handle.unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "virtual-list-scroll"), cx, |_, _| {
                    VirtualListScrollHandle::new()
                })
                .read(cx)
                .clone()
        });
        let focus_handle = self.focus_handle.unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "virtual-list-focus"), cx, |_, cx| {
                    cx.focus_handle()
                })
                .read(cx)
                .clone()
        });
        let focus_visible = focus_handle.is_focused(window) && window.last_input_was_keyboard();

        let padding = resolve_edges(self.content_padding, direction);
        let mut inner = self
            .base
            .track_scroll(&scroll_handle.inner)
            .size_full()
            .pt(padding.top)
            .pb(padding.bottom)
            .pl(padding.left)
            .pr(padding.right);
        if self.item_gap > px(0.) {
            inner = inner.gap(self.item_gap);
        }

        let axis = self.axis;
        let step = keyboard_step(density, self.keyboard_scroll_step);
        let key_scroll = scroll_handle.clone();
        let key_focus = focus_handle.clone();

        let focus_ring_overlay = focus_visible.then(|| {
            canvas(
                move |bounds, _, _| Shape::rounded(bounds, shape, direction),
                move |_, prepared, window, cx| {
                    let Some(shape) = prepared else {
                        return;
                    };
                    let mut interaction = InteractionState::default();
                    interaction.focus_visible = true;
                    InteractionFeedback::new(interaction, transparent_black(), focus_ring_color)
                        .focus_ring_placement(FocusRingPlacement::Inset)
                        .paint(
                            &shape,
                            None,
                            cx.background_executor().now(),
                            reduced_motion,
                            window,
                        );
                },
            )
            .absolute()
            .inset_0()
        });

        let mut container = div()
            .id(self.id.clone())
            .test_support()
            .role(Role::List)
            .when_some(self.accessible_label, |el, label| el.aria_label(label))
            .track_focus(&focus_handle)
            .tab_stop(true)
            .relative()
            .w_full()
            .bg(bg)
            .when(has_border, |el| el.border_1().border_color(border))
            .rounded_tl(px(radii[0]))
            .rounded_tr(px(radii[1]))
            .rounded_br(px(radii[2]))
            .rounded_bl(px(radii[3]))
            .overflow_hidden()
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                if event.keystroke.modifiers.modified() || !key_focus.is_focused(window) {
                    return;
                }
                let viewport = key_scroll.viewport_bounds().size;
                let content = key_scroll.content_size();
                if let Some(next) = compute_keyboard_scroll_offset(
                    axis,
                    direction,
                    &event.keystroke.key,
                    key_scroll.offset(),
                    viewport,
                    content,
                    step,
                ) {
                    key_scroll.set_offset(next);
                    cx.stop_propagation();
                    window.refresh();
                }
            })
            .child(inner);

        if self.show_scrollbar {
            container = container.child(
                DisplayScrollbar::new((self.id, "scrollbar"), axis, &scroll_handle)
                    .style(self.scrollbar_style)
                    .fade(self.scrollbar_fade),
            );
        }

        container
            .children(focus_ring_overlay)
            .refine_style(&self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    #[test]
    fn vertical_keyboard_scroll_clamps_to_content_extent() {
        let viewport = size(px(300.), px(200.));
        let content = size(px(300.), px(1000.));
        let step = px(48.);

        let down = compute_keyboard_scroll_offset(
            Axis::Vertical,
            Direction::Ltr,
            "down",
            point(px(0.), px(0.)),
            viewport,
            content,
            step,
        );
        assert_eq!(down, Some(point(px(0.), px(-48.))));

        let up_clamped = compute_keyboard_scroll_offset(
            Axis::Vertical,
            Direction::Ltr,
            "up",
            point(px(0.), px(-20.)),
            viewport,
            content,
            step,
        );
        assert_eq!(up_clamped, Some(point(px(0.), px(0.))));

        let end = compute_keyboard_scroll_offset(
            Axis::Vertical,
            Direction::Ltr,
            "end",
            point(px(0.), px(0.)),
            viewport,
            content,
            step,
        );
        assert_eq!(end, Some(point(px(0.), px(-800.))));

        let home = compute_keyboard_scroll_offset(
            Axis::Vertical,
            Direction::Ltr,
            "home",
            point(px(0.), px(-500.)),
            viewport,
            content,
            step,
        );
        assert_eq!(home, Some(point(px(0.), px(0.))));
    }

    #[test]
    fn horizontal_keyboard_scroll_mirrors_in_rtl() {
        let viewport = size(px(200.), px(100.));
        let content = size(px(600.), px(100.));
        let step = px(40.);

        let ltr_right = compute_keyboard_scroll_offset(
            Axis::Horizontal,
            Direction::Ltr,
            "right",
            point(px(0.), px(0.)),
            viewport,
            content,
            step,
        );
        assert_eq!(ltr_right, Some(point(px(-40.), px(0.))));

        let rtl_left = compute_keyboard_scroll_offset(
            Axis::Horizontal,
            Direction::Rtl,
            "left",
            point(px(0.), px(0.)),
            viewport,
            content,
            step,
        );
        assert_eq!(rtl_left, Some(point(px(-40.), px(0.))));

        let rtl_right = compute_keyboard_scroll_offset(
            Axis::Horizontal,
            Direction::Rtl,
            "right",
            point(px(-100.), px(0.)),
            viewport,
            content,
            step,
        );
        assert_eq!(rtl_right, Some(point(px(-60.), px(0.))));
    }
}
