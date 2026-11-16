use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, AvailableSpace, BorderStyle, Bounds, ContentMask, Corners, ElementId, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce, Role, ScrollHandle,
    SharedString, StatefulInteractiveElement, Styled, Window, canvas, div, point, px, quad, size,
};
use gpui_material_foundation::{animate, window_material_theme};
use gpui_material_theme::{CornerShape, Direction, MotionKind, MotionSpec, MotionSpeed};

use super::items::{NavigationItems, Presentation};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabVariant {
    #[default]
    Primary,
    Secondary,
}

/// Optional tab row pigments. Per-tab overrides take precedence over `content`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TabsColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub divider: Option<Hsla>,
}

/// Width of the indicator relative to the selected tab.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TabIndicatorWidth {
    #[default]
    Default,
    Tab,
    Content,
    Fixed(Pixels),
}

/// Overrides for the indicator that tracks the selected tab.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TabIndicatorStyle {
    pub width: TabIndicatorWidth,
    pub height: Option<Pixels>,
    pub color: Option<Hsla>,
    pub shape: Option<CornerShape>,
    pub motion: Option<MotionSpec>,
}

/// Geometry and tint supplied to a decorative custom indicator.
#[derive(Clone, Debug, PartialEq)]
pub struct TabIndicatorLayout {
    pub selected: SharedString,
    pub bounds: Bounds<Pixels>,
    pub tab_bounds: Bounds<Pixels>,
    pub content_width: Pixels,
    pub color: Hsla,
    pub direction: Direction,
}

type Indicator = Box<dyn Fn(TabIndicatorLayout) -> AnyElement>;

pub(super) struct TabOptions {
    pub colors: TabsColors,
    pub scroll: Option<ScrollHandle>,
    pub edge_padding: Pixels,
    pub min_tab_width: Pixels,
    pub divider: Option<AnyElement>,
    pub indicator_style: Option<TabIndicatorStyle>,
    pub indicator_content: Option<Indicator>,
}

impl Default for TabOptions {
    fn default() -> Self {
        Self {
            colors: TabsColors::default(),
            scroll: None,
            edge_padding: px(52.),
            min_tab_width: px(90.),
            divider: None,
            indicator_style: None,
            indicator_content: None,
        }
    }
}

/// A fixed Material tab row by default. Enable scrolling for long tab lists.
#[derive(IntoElement)]
pub struct Tabs {
    items: NavigationItems,
    variant: TabVariant,
    scrollable: bool,
    options: TabOptions,
}

impl Tabs {
    pub fn new(items: NavigationItems) -> Self {
        Self {
            items,
            variant: TabVariant::Primary,
            scrollable: false,
            options: TabOptions::default(),
        }
    }

    pub fn variant(mut self, variant: TabVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    /// Uses a caller-owned handle when horizontal scrolling is enabled.
    pub fn scroll_handle(mut self, handle: &ScrollHandle) -> Self {
        self.options.scroll = Some(handle.clone());
        self
    }

    /// Sets the logical start and end padding for scrolling tabs.
    pub fn edge_padding(mut self, padding: Pixels) -> Self {
        self.options.edge_padding = length(padding).unwrap_or(px(52.));
        self
    }

    /// Sets the minimum width of a scrolling tab.
    pub fn min_tab_width(mut self, width: Pixels) -> Self {
        self.options.min_tab_width = length(width).unwrap_or(px(90.));
        self
    }

    pub fn colors(mut self, colors: TabsColors) -> Self {
        self.options.colors = colors;
        self
    }

    /// Replaces the row's decorative bottom divider.
    pub fn divider(mut self, divider: impl IntoElement) -> Self {
        self.options.divider = Some(divider.into_any_element());
        self
    }

    pub fn indicator_style(mut self, style: TabIndicatorStyle) -> Self {
        self.options.indicator_style = Some(style);
        self
    }

    /// Builds a decorative indicator constrained to the resolved indicator bounds.
    pub fn indicator_content<E: IntoElement>(
        mut self,
        content: impl Fn(TabIndicatorLayout) -> E + 'static,
    ) -> Self {
        self.options.indicator_content =
            Some(Box::new(move |layout| content(layout).into_any_element()));
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.items.render_row_with_tabs(
            Presentation::Tabs(self.variant, self.scrollable),
            Some(self.options),
            window,
            cx,
        )
    }
}

fn length(value: Pixels) -> Option<Pixels> {
    let value_f = f32::from(value);
    (value_f.is_finite() && value_f >= 0.).then_some(value)
}

#[derive(Default)]
pub(super) struct TabReveal {
    selection: Option<SharedString>,
    focus: Option<SharedString>,
    target: Option<SharedString>,
    order: Vec<SharedString>,
    geometry: Option<(Pixels, Pixels, Pixels, Pixels)>,
}

impl TabReveal {
    pub(super) fn choose(
        &mut self,
        selected: Option<&SharedString>,
        focused: Option<&SharedString>,
        order: Vec<SharedString>,
        scroll: &ScrollHandle,
    ) -> Option<usize> {
        let old_target = self.target.clone();
        if self.selection.as_ref() != selected {
            self.target = selected.or(focused).cloned();
        } else if let Some(focused) =
            focused.filter(|focused| self.focus.as_ref() != Some(*focused))
        {
            self.target = Some(focused.clone());
        }
        self.selection = selected.cloned();
        self.focus = focused.cloned();
        if self.target.as_ref().is_none_or(|id| !order.contains(id)) {
            self.target = selected
                .filter(|id| order.contains(id))
                .or(focused)
                .cloned();
        }
        let index = self
            .target
            .as_ref()
            .and_then(|id| order.iter().position(|item| item == id));
        if self.order != order || self.target != old_target || scroll.bounds().size.width == px(0.)
        {
            self.geometry = None;
            if let Some(index) = index {
                scroll.scroll_to_item(index);
            }
        }
        self.order = order;
        index
    }

    pub(super) fn observe_geometry(
        &mut self,
        index: usize,
        scroll: &ScrollHandle,
        window: &mut Window,
    ) {
        let Some(tab) = scroll.bounds_for_item(index) else {
            return;
        };
        let viewport = scroll.bounds();
        let width = viewport.size.width;
        let left = tab.left() - viewport.left();
        let geometry = (width, left, tab.size.width, scroll.max_offset().x);
        if self.geometry.replace(geometry) == Some(geometry) {
            return;
        }
        let visible_left = left + scroll.offset().x;
        if visible_left < px(0.) || visible_left + tab.size.width.min(width) > width {
            scroll.scroll_to_item(index);
            window.request_animation_frame();
        }
    }
}

#[derive(Default)]
pub(super) struct TabIndicatorTarget {
    pub tab_bounds: Cell<Option<Bounds<Pixels>>>,
    pub content_width: Cell<Pixels>,
}

pub(super) struct TabDecoration {
    pub row: ElementId,
    pub selected: SharedString,
    pub variant: TabVariant,
    pub direction: Direction,
    pub target: Rc<TabIndicatorTarget>,
    pub scroll: Option<ScrollHandle>,
    pub style: TabIndicatorStyle,
    pub content: Option<Indicator>,
    pub color: Hsla,
    pub has_default_divider: bool,
}

impl TabDecoration {
    pub(super) fn element(self) -> impl IntoElement {
        canvas(
            move |viewport, window, cx| {
                let tab = self.target.tab_bounds.get()?;
                let offset = self
                    .scroll
                    .as_ref()
                    .map_or(px(0.), |handle| handle.offset().x);
                let content_width = self.target.content_width.get().min(tab.size.width);
                let default_width = if self.variant == TabVariant::Primary {
                    (tab.size.width - px(4.)).max(px(0.)) * (2. / 3.)
                } else {
                    (tab.size.width - px(4.)).max(px(0.))
                };
                let width = match self.style.width {
                    TabIndicatorWidth::Default => default_width,
                    TabIndicatorWidth::Tab => tab.size.width,
                    TabIndicatorWidth::Content => content_width.max(px(24.)),
                    TabIndicatorWidth::Fixed(width) => length(width).unwrap_or(default_width),
                }
                .min(tab.size.width);
                let divider_height = if self.has_default_divider {
                    px(1.)
                } else {
                    px(0.)
                };
                let height = self
                    .style
                    .height
                    .and_then(length)
                    .unwrap_or(px(if self.variant == TabVariant::Primary {
                        3.
                    } else {
                        2.
                    }))
                    .min((viewport.size.height - divider_height).max(px(0.)));
                let motion = if cx.reduce_motion() {
                    MotionSpec::Instant
                } else {
                    self.style.motion.unwrap_or_else(|| {
                        window_material_theme(window, cx).motion.spec(
                            MotionKind::Spatial,
                            MotionSpeed::Fast,
                            false,
                        )
                    })
                };
                // Animate within the unscrolled strip so caller-driven scrolling
                // translates the indicator together with its tab immediately.
                let center = animate(
                    (self.row.clone(), "indicator-center"),
                    f32::from(tab.center().x - viewport.left()),
                    motion,
                    window,
                    cx,
                );
                let width = px(animate(
                    (self.row.clone(), "indicator-width"),
                    f32::from(width),
                    motion,
                    window,
                    cx,
                ))
                .clamp(px(0.), viewport.size.width.max(tab.size.width));
                if width == px(0.) || height == px(0.) {
                    return None;
                }
                let edge = viewport.bottom() - divider_height;
                let bounds = Bounds::new(
                    point(
                        viewport.left() + offset + px(center) - width / 2.,
                        edge - height,
                    ),
                    size(width, height),
                );
                let tab_bounds = Bounds::new(tab.origin + point(offset, px(0.)), tab.size);
                let layout = TabIndicatorLayout {
                    selected: self.selected,
                    bounds,
                    tab_bounds,
                    content_width,
                    color: self.color,
                    direction: self.direction,
                };
                let mask = ContentMask { bounds: viewport };
                let child = self.content.map(|content| {
                    let mut child = decorative_slot(
                        (self.row, "custom-indicator").into(),
                        content(layout.clone()),
                    )
                    .w(width)
                    .h(height)
                    .text_color(self.color)
                    .into_any_element();
                    window.with_content_mask(Some(mask), |window| {
                        child.prepaint_as_root(
                            bounds.origin,
                            bounds.size.map(AvailableSpace::Definite),
                            window,
                            cx,
                        );
                    });
                    child
                });
                let shape = self
                    .style
                    .shape
                    .unwrap_or(if self.variant == TabVariant::Primary {
                        CornerShape::Radii([3., 3., 0., 0.])
                    } else {
                        CornerShape::Radii([0.; 4])
                    });
                Some((layout, mask, child, shape))
            },
            |_, state, window, cx| {
                if let Some((layout, mask, child, shape)) = state {
                    window.with_content_mask(Some(mask), |window| {
                        if let Some(mut child) = child {
                            child.paint(window, cx);
                        } else {
                            let rect = layout.bounds;
                            let [top_left, top_right, bottom_right, bottom_left] = shape
                                .radii(
                                    rect.size.width.into(),
                                    rect.size.height.into(),
                                    layout.direction,
                                )
                                .map(px);
                            window.paint_quad(quad(
                                rect,
                                Corners {
                                    top_left,
                                    top_right,
                                    bottom_right,
                                    bottom_left,
                                },
                                layout.color,
                                px(0.),
                                gpui::transparent_black(),
                                BorderStyle::Solid,
                            ));
                        }
                    });
                }
            },
        )
        .absolute()
        .inset_0()
    }
}

pub(super) fn decorative_slot(
    id: ElementId,
    content: impl IntoElement,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .role(Role::Group)
        .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
        .min_w_0()
        .max_w_full()
        .overflow_hidden()
        .child(content)
}
