use std::{
    cell::{Cell, RefCell},
    ops::Range,
    rc::Rc,
};

use gpui::{
    AccessibleAction, AnyElement, App, AvailableSpace, Bounds, ContentMask, Element, ElementId,
    FocusHandle, GlobalElementId, Hsla, InspectorElementId, InteractiveElement, IntoElement,
    LayoutId, ParentElement, Pixels, Point, RenderOnce, Role, SharedString, Size,
    StatefulInteractiveElement, Style, Styled, Window, div, point, prelude::FluentBuilder, px,
    relative, size,
};
use gpui_base::{Button as BaseButton, TestSupportExt};
use gpui_material_foundation::{ThemeScope, color, current_material_theme, text_style};
use gpui_material_theme::{ColorRole, Direction, TypeRole, TypographyEmphasis};

use crate::{Icon, IconName, Menu, MenuItem};

const ROW_MIN_HEIGHT: Pixels = px(32.);
const ITEM_GAP: Pixels = px(4.);
const OVERFLOW_TRIGGER_WIDTH: Pixels = px(32.);
const OVERFLOW_TRIGGER_HEIGHT: Pixels = px(32.);

type BreadcrumbSelect = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type SeparatorFactory = Rc<dyn Fn() -> AnyElement>;
type DisplacedFocus = Rc<RefCell<Option<(SharedString, FocusHandle)>>>;

/// Optional color overrides for [`Breadcrumbs`].
///
/// Omitted fields default to `OnSurfaceVariant` for ancestors and separators,
/// `OnSurface` for the current item, and `OnSurface` at 38% opacity for disabled items.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BreadcrumbsColors {
    pub ancestor: Option<Hsla>,
    pub current: Option<Hsla>,
    pub separator: Option<Hsla>,
    pub disabled: Option<Hsla>,
}

#[derive(Clone, Copy)]
struct ResolvedColors {
    ancestor: Hsla,
    current: Hsla,
    separator: Hsla,
    disabled: Hsla,
    hover: Hsla,
    focus_ring: Hsla,
}

impl BreadcrumbsColors {
    fn resolve(self, window: &Window, cx: &App) -> ResolvedColors {
        let theme = current_material_theme(window, cx);
        let role = |r| color(theme.colors.color(r));
        let on_surface = role(ColorRole::OnSurface);
        let on_surface_variant = role(ColorRole::OnSurfaceVariant);
        ResolvedColors {
            ancestor: self.ancestor.unwrap_or(on_surface_variant),
            current: self.current.unwrap_or(on_surface),
            separator: self.separator.unwrap_or(on_surface_variant),
            disabled: self.disabled.unwrap_or_else(|| on_surface.opacity(0.38)),
            hover: on_surface.opacity(0.08),
            focus_ring: role(ColorRole::Secondary),
        }
    }
}

/// One path segment inside [`Breadcrumbs`].
pub struct BreadcrumbItem {
    id: SharedString,
    label: SharedString,
    icon: Option<AnyElement>,
    disabled: bool,
}

impl BreadcrumbItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            disabled: false,
        }
    }

    /// Adds an optional leading decorative icon to the inline segment.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Disables navigation for this segment.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Desktop path navigation with RTL mirroring and automatic middle-segment
/// overflow collapse into an anchored Material [`Menu`].
#[derive(IntoElement)]
pub struct Breadcrumbs {
    id: ElementId,
    items: Vec<BreadcrumbItem>,
    accessible_label: SharedString,
    overflow_label: SharedString,
    on_select: Option<BreadcrumbSelect>,
    max_items: Option<usize>,
    separator: Option<SeparatorFactory>,
    colors: BreadcrumbsColors,
}

impl Breadcrumbs {
    pub fn new(id: impl Into<ElementId>, items: impl IntoIterator<Item = BreadcrumbItem>) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
            accessible_label: "Breadcrumb".into(),
            overflow_label: "More breadcrumbs".into(),
            on_select: None,
            max_items: None,
            separator: None,
            colors: BreadcrumbsColors::default(),
        }
    }

    /// Sets the `Role::Navigation` landmark label. Defaults to `"Breadcrumb"`.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = label.into();
        self
    }

    /// Sets the accessible name of the collapsed-segments overflow trigger button.
    pub fn overflow_label(mut self, label: impl Into<SharedString>) -> Self {
        self.overflow_label = label.into();
        self
    }

    /// Invoked when clicking or activating any non-current enabled ancestor segment,
    /// including collapsed segments selected from the overflow [`Menu`].
    pub fn on_select(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Caps visible inline segments before collapsing middle segments into the overflow menu.
    /// Values below `2` are clamped to `2` when three or more segments exist.
    pub fn max_items(mut self, count: usize) -> Self {
        self.max_items = Some(count.max(2));
        self
    }

    /// Replaces the decorative separator between segments.
    /// Defaults to `Icon::new(IconName::ChevronRight).size(px(16.)).mirror_in_rtl(true)`.
    pub fn separator<E: IntoElement>(mut self, separator: impl Fn() -> E + 'static) -> Self {
        self.separator = Some(Rc::new(move || separator().into_any_element()));
        self
    }

    /// Overrides ancestor, current, separator, and disabled colors.
    pub fn colors(mut self, colors: BreadcrumbsColors) -> Self {
        self.colors = colors;
        self
    }
}

impl RenderOnce for Breadcrumbs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let label = self.accessible_label.clone();
        div()
            .id(id)
            .test_support()
            .role(Role::Navigation)
            .aria_label(label)
            .w_full()
            .child(BreadcrumbsLayout::new(self, window, cx))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CollapsePlan {
    leading: Range<usize>,
    collapsed: Range<usize>,
    trailing: Range<usize>,
}

fn compute_collapse_plan(
    widths: &[Pixels],
    sep_step: Pixels,
    trigger_width: Pixels,
    max_items: Option<usize>,
    available: Pixels,
) -> CollapsePlan {
    let n = widths.len();
    if n == 0 {
        return CollapsePlan {
            leading: 0..0,
            collapsed: 0..0,
            trailing: 0..0,
        };
    }
    if n <= 2 {
        return CollapsePlan {
            leading: 0..n,
            collapsed: 0..0,
            trailing: n..n,
        };
    }

    let cap = max_items.map(|c| c.max(2)).unwrap_or(usize::MAX);
    let full_width = widths.iter().copied().sum::<Pixels>() + sep_step * (n - 1) as f32;
    if n <= cap && full_width <= available {
        return CollapsePlan {
            leading: 0..n,
            collapsed: 0..0,
            trailing: n..n,
        };
    }

    // Keep first segment (0), overflow trigger, and as many trailing segments as fit
    // (always keeping at least the current trailing segment n - 1 and collapsing at least index 1).
    let mut trailing_start = n - 1;
    let mut used = widths[0] + trigger_width + widths[n - 1] + sep_step * 2.;

    for idx in (2..=(n - 2)).rev() {
        let visible_items = 1 + (n - idx);
        if visible_items > cap {
            break;
        }
        let extra = widths[idx] + sep_step;
        if used + extra <= available {
            used += extra;
            trailing_start = idx;
        } else {
            break;
        }
    }

    CollapsePlan {
        leading: 0..1,
        collapsed: 1..trailing_start,
        trailing: trailing_start..n,
    }
}

struct OverflowState {
    shown: Rc<Cell<bool>>,
    has_overflow: Rc<Cell<bool>>,
    focus: FocusHandle,
    displaced: DisplacedFocus,
}

#[derive(Clone)]
struct MenuController {
    shown: Rc<Cell<bool>>,
    has_overflow: Rc<Cell<bool>>,
    view: gpui::EntityId,
}

impl MenuController {
    fn is_showing(&self) -> bool {
        self.shown.get()
    }

    fn set(&self, shown: bool, window: &mut Window, cx: &mut App) {
        if self.shown.replace(shown) != shown {
            cx.notify(self.view);
            window.refresh();
        }
    }
}

struct PreparedItem {
    id: SharedString,
    label: SharedString,
    disabled: bool,
    is_current: bool,
    focus: Option<FocusHandle>,
    inline: AnyElement,
    preferred: Size<Pixels>,
}

struct BreadcrumbsLayout {
    breadcrumbs: Breadcrumbs,
    direction: Direction,
    colors: ResolvedColors,
    menu: MenuController,
    overflow_focus: FocusHandle,
    displaced: DisplacedFocus,
}

impl BreadcrumbsLayout {
    fn new(breadcrumbs: Breadcrumbs, window: &mut Window, cx: &mut App) -> Self {
        let overflow = window.use_keyed_state(
            (breadcrumbs.id.clone(), "breadcrumbs-overflow"),
            cx,
            |_, cx| OverflowState {
                shown: Rc::new(Cell::new(false)),
                has_overflow: Rc::new(Cell::new(false)),
                focus: cx.focus_handle(),
                displaced: Rc::new(RefCell::new(None)),
            },
        );
        let overflow = overflow.read(cx);
        Self {
            direction: current_material_theme(window, cx).layout.direction,
            colors: breadcrumbs.colors.resolve(window, cx),
            menu: MenuController {
                shown: overflow.shown.clone(),
                has_overflow: overflow.has_overflow.clone(),
                view: window.current_view(),
            },
            overflow_focus: overflow.focus.clone(),
            displaced: overflow.displaced.clone(),
            breadcrumbs,
        }
    }

    fn make_separator(&self) -> AnyElement {
        let glyph = match &self.breadcrumbs.separator {
            Some(factory) => factory(),
            None => Icon::new(IconName::ChevronRight)
                .size(px(16.))
                .mirror_in_rtl(true)
                .into_any_element(),
        };
        div()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .px(ITEM_GAP)
            .text_color(self.colors.separator)
            .child(glyph)
            .into_any_element()
    }

    fn prepare_item(
        &self,
        item: BreadcrumbItem,
        is_current: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> PreparedItem {
        let rtl = self.direction == Direction::Rtl;
        let elem_id = ElementId::from((self.breadcrumbs.id.clone(), item.id.clone()));

        if is_current {
            let fg = if item.disabled {
                self.colors.disabled
            } else {
                self.colors.current
            };
            let current_aria: SharedString = format!("{}, current page", item.label).into();
            let mut inline = div()
                .id(elem_id)
                .test_support()
                .role(Role::Label)
                .aria_label(current_aria)
                .h(ROW_MIN_HEIGHT)
                .px(px(6.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.))
                .when(rtl, Styled::flex_row_reverse)
                .text_color(fg)
                .children(
                    item.icon
                        .map(|icon| div().flex_none().text_size(px(16.)).child(icon)),
                )
                .child(
                    text_style(
                        div().whitespace_nowrap(),
                        TypeRole::TitleSmall.style(TypographyEmphasis::Baseline),
                        1.,
                        window,
                        cx,
                    )
                    .child(item.label.clone()),
                )
                .into_any_element();
            let preferred = inline.layout_as_root(
                size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
                window,
                cx,
            );
            return PreparedItem {
                id: item.id,
                label: item.label,
                disabled: item.disabled,
                is_current: true,
                focus: None,
                inline,
                preferred,
            };
        }

        let focus = window.with_id(
            ElementId::from((self.breadcrumbs.id.clone(), "breadcrumb-focus")),
            |window| {
                window
                    .use_keyed_state(item.id.clone(), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone()
            },
        );

        let fg = if item.disabled {
            self.colors.disabled
        } else {
            self.colors.ancestor
        };
        let hover_bg = self.colors.hover;
        let focus_ring = self.colors.focus_ring;

        let activate = (!item.disabled).then(|| {
            self.breadcrumbs.on_select.as_ref().map(|on_select| {
                let on_select = on_select.clone();
                let item_id = item.id.clone();
                Rc::new(ThemeScope::bind_action(
                    move |window, cx| on_select(item_id.clone(), window, cx),
                    window,
                    cx,
                ))
            })
        });
        let activate = activate.flatten();

        let click_handler = activate.clone();
        let a11y_handler = activate;

        let mut inline = BaseButton::new(elem_id)
            .track_focus(&focus)
            .disabled(item.disabled)
            .accessibility_label(item.label.clone())
            .h(ROW_MIN_HEIGHT)
            .px(px(6.))
            .rounded(px(8.))
            .border_2()
            .border_color(gpui::transparent_black())
            .flex()
            .flex_row()
            .items_center()
            .gap(px(4.))
            .when(rtl, Styled::flex_row_reverse)
            .text_color(fg)
            .when(!item.disabled, move |button| {
                button
                    .hover(move |b| b.bg(hover_bg))
                    .focus_visible(move |b| b.border_color(focus_ring))
            })
            .when_some(click_handler, |button, handler| {
                button.on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    handler(window, cx);
                })
            })
            .when_some(a11y_handler, |button, handler| {
                button.on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    handler(window, cx);
                })
            })
            .children(
                item.icon
                    .map(|icon| div().flex_none().text_size(px(16.)).child(icon)),
            )
            .child(
                text_style(
                    div().whitespace_nowrap(),
                    TypeRole::LabelLarge.style(TypographyEmphasis::Baseline),
                    1.,
                    window,
                    cx,
                )
                .child(item.label.clone()),
            )
            .into_any_element();

        let preferred = inline.layout_as_root(
            size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
            window,
            cx,
        );

        PreparedItem {
            id: item.id,
            label: item.label,
            disabled: item.disabled,
            is_current: false,
            focus: Some(focus),
            inline,
            preferred,
        }
    }

    fn transfer_focus(
        &self,
        plan: &CollapsePlan,
        items: &[PreparedItem],
        window: &mut Window,
        cx: &mut App,
    ) {
        let overflow = !plan.collapsed.is_empty();
        let in_overflow = self.overflow_focus.is_focused(window);
        let previous_overflow = self.menu.has_overflow.replace(overflow);
        let menu_was_open = self.menu.is_showing();
        let mut displaced = self.displaced.borrow_mut();

        if overflow
            && let Some(hidden) = items[plan.collapsed.clone()].iter().find(|item| {
                item.focus
                    .as_ref()
                    .is_some_and(|focus| focus.contains_focused(window, cx))
            })
        {
            let old_focus = window
                .focused(cx)
                .or_else(|| hidden.focus.clone())
                .unwrap_or_else(|| self.overflow_focus.clone());
            *displaced = Some((hidden.id.clone(), old_focus));
            focus_after_layout(self.overflow_focus.clone(), window, cx);
            return;
        }

        if !overflow && previous_overflow {
            self.menu.shown.set(false);
        }

        let is_visible = |id: &SharedString| {
            plan.leading
                .clone()
                .chain(plan.trailing.clone())
                .any(|idx| !items[idx].disabled && !items[idx].is_current && items[idx].id == *id)
        };
        let returned = displaced
            .as_ref()
            .filter(|(id, _)| is_visible(id))
            .map(|(_, focus)| focus.clone());

        let restore = if !overflow && (in_overflow || menu_was_open) {
            returned.or_else(|| {
                items
                    .iter()
                    .find(|item| !item.disabled && !item.is_current)
                    .and_then(|item| item.focus.clone())
            })
        } else if in_overflow {
            returned
        } else {
            None
        };

        if let Some(focus) = restore {
            displaced.take();
            focus_after_layout(focus, window, cx);
        } else if !overflow {
            displaced.take();
        }
    }

    fn build_overflow_menu(
        &self,
        collapsed: &[PreparedItem],
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let entries: Vec<MenuItem> = collapsed
            .iter()
            .map(|item| {
                let mut entry = MenuItem::new(
                    ElementId::from((self.breadcrumbs.id.clone(), item.id.clone())),
                    item.label.clone(),
                )
                .disabled(item.disabled);
                if !item.disabled
                    && let Some(on_select) = self.breadcrumbs.on_select.clone()
                {
                    let item_id = item.id.clone();
                    let bound = ThemeScope::bind_action(
                        move |window, cx| on_select(item_id.clone(), window, cx),
                        window,
                        cx,
                    );
                    entry = entry.on_activate(move |window, cx| bound(window, cx));
                }
                entry
            })
            .collect();

        let state = self.menu.clone();
        let observed = state.clone();
        let trigger_focus = self.overflow_focus.clone();
        let trigger_id = ElementId::from((self.breadcrumbs.id.clone(), "overflow"));
        let label = self.breadcrumbs.overflow_label.clone();
        let ink = self.colors.ancestor;
        let hover_bg = self.colors.hover;
        let focus_ring = self.colors.focus_ring;

        Menu::composed(
            (self.breadcrumbs.id.clone(), "overflow-menu"),
            entries,
            move |_, toggle| {
                let return_focus = trigger_focus.clone();
                let dots = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(2.5))
                    .children((0..3).map(|_| div().size(px(3.5)).rounded_full().bg(ink)));
                BaseButton::new(trigger_id)
                    .track_focus(&trigger_focus)
                    .accessibility_label(label)
                    .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        return_focus.focus(window, cx);
                        toggle(window, cx);
                    })
                    .w(OVERFLOW_TRIGGER_WIDTH)
                    .h(OVERFLOW_TRIGGER_HEIGHT)
                    .p_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.))
                    .border_2()
                    .border_color(gpui::transparent_black())
                    .text_color(ink)
                    .hover(move |button| button.bg(hover_bg))
                    .focus_visible(move |button| button.border_color(focus_ring))
                    .child(dots)
                    .into_any_element()
            },
        )
        .open(state.is_showing())
        .on_open_change(move |open, window, cx| observed.set(open, window, cx))
        .into_any_element()
    }

    fn origin(&self, bounds: Bounds<Pixels>, child: Size<Pixels>, offset: Pixels) -> Point<Pixels> {
        let dy = (bounds.size.height - child.height).max(px(0.)) / 2.;
        match self.direction {
            Direction::Ltr => point(bounds.left() + offset, bounds.top() + dy),
            Direction::Rtl => point(bounds.right() - offset - child.width, bounds.top() + dy),
        }
    }
}

fn focus_after_layout(target: FocusHandle, window: &mut Window, cx: &mut App) {
    let origin = window.focused(cx);
    window.on_next_frame(move |window, cx| {
        let current = window.focused(cx);
        if current.is_none() || current == origin {
            target.focus(window, cx);
        }
    });
}

impl IntoElement for BreadcrumbsLayout {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

struct BreadcrumbsLayoutState {
    items: Vec<PreparedItem>,
    sep_width: Pixels,
}

struct BreadcrumbsPrepaintState {
    plan: CollapsePlan,
    separators: Vec<AnyElement>,
    menu: Option<AnyElement>,
}

impl Element for BreadcrumbsLayout {
    type RequestLayoutState = BreadcrumbsLayoutState;
    type PrepaintState = BreadcrumbsPrepaintState;

    fn id(&self) -> Option<ElementId> {
        Some(ElementId::from((self.breadcrumbs.id.clone(), "layout")))
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn a11y_role(&self) -> Option<Role> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let raw_items = std::mem::take(&mut self.breadcrumbs.items);
        let len = raw_items.len();
        let items: Vec<PreparedItem> = raw_items
            .into_iter()
            .enumerate()
            .map(|(idx, item)| self.prepare_item(item, idx + 1 == len, window, cx))
            .collect();

        let mut probe_sep = self.make_separator();
        let sep_size = probe_sep.layout_as_root(
            size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
            window,
            cx,
        );
        let sep_width = sep_size.width;

        let widths: Vec<Pixels> = items.iter().map(|i| i.preferred.width).collect();
        let preferred_plan = compute_collapse_plan(
            &widths,
            sep_width,
            OVERFLOW_TRIGGER_WIDTH,
            self.breadcrumbs.max_items,
            px(f32::INFINITY),
        );

        let visible_segments = preferred_plan.leading.len()
            + usize::from(!preferred_plan.collapsed.is_empty())
            + preferred_plan.trailing.len();
        let mut preferred_width = preferred_plan
            .leading
            .clone()
            .chain(preferred_plan.trailing.clone())
            .map(|i| items[i].preferred.width)
            .sum::<Pixels>();
        if !preferred_plan.collapsed.is_empty() {
            preferred_width += OVERFLOW_TRIGGER_WIDTH;
        }
        if visible_segments > 1 {
            preferred_width += sep_width * (visible_segments - 1) as f32;
        }

        let cross = items.iter().map(|item| item.preferred.height).fold(
            if items.is_empty() {
                px(0.)
            } else {
                ROW_MIN_HEIGHT.max(sep_size.height)
            },
            Pixels::max,
        );

        let style = Style {
            size: size(preferred_width.into(), cross.into()),
            min_size: size(px(0.).into(), px(0.).into()),
            max_size: size(relative(1.).into(), relative(1.).into()),
            ..Default::default()
        };

        (
            window.request_layout(style, None, cx),
            BreadcrumbsLayoutState { items, sep_width },
        )
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let widths: Vec<Pixels> = state.items.iter().map(|i| i.preferred.width).collect();
        let plan = compute_collapse_plan(
            &widths,
            state.sep_width,
            OVERFLOW_TRIGGER_WIDTH,
            self.breadcrumbs.max_items,
            bounds.size.width,
        );
        self.transfer_focus(&plan, &state.items, window, cx);

        let mut offset = px(0.);
        let mut separators = Vec::new();
        let mut placed_any = false;

        let mut place_separator = |offset: &mut Pixels, window: &mut Window, cx: &mut App| {
            let mut sep = self.make_separator();
            let sep_size = sep.layout_as_root(
                size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
                window,
                cx,
            );
            let origin = self.origin(bounds, sep_size, *offset);
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                sep.prepaint_at(origin, window, cx);
            });
            *offset += sep_size.width;
            separators.push(sep);
        };

        for idx in plan.leading.clone() {
            if placed_any {
                place_separator(&mut offset, window, cx);
            }
            let item = &mut state.items[idx];
            let origin = self.origin(bounds, item.preferred, offset);
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                item.inline.prepaint_at(origin, window, cx);
            });
            offset += item.preferred.width;
            placed_any = true;
        }

        let menu = (!plan.collapsed.is_empty()).then(|| {
            if placed_any {
                place_separator(&mut offset, window, cx);
            }
            let mut menu =
                self.build_overflow_menu(&state.items[plan.collapsed.clone()], window, cx);
            let trigger_size = size(
                OVERFLOW_TRIGGER_WIDTH,
                OVERFLOW_TRIGGER_HEIGHT.min(bounds.size.height.max(ROW_MIN_HEIGHT)),
            );
            let measured =
                menu.layout_as_root(trigger_size.map(AvailableSpace::Definite), window, cx);
            let origin = self.origin(bounds, measured, offset);
            menu.prepaint_at(origin, window, cx);
            offset += measured.width;
            placed_any = true;
            menu
        });

        for idx in plan.trailing.clone() {
            if placed_any {
                place_separator(&mut offset, window, cx);
            }
            let item = &mut state.items[idx];
            let origin = self.origin(bounds, item.preferred, offset);
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                item.inline.prepaint_at(origin, window, cx);
            });
            offset += item.preferred.width;
            placed_any = true;
        }

        BreadcrumbsPrepaintState {
            plan,
            separators,
            menu,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for idx in prepaint.plan.leading.clone() {
                state.items[idx].inline.paint(window, cx);
            }
            for sep in &mut prepaint.separators {
                sep.paint(window, cx);
            }
            for idx in prepaint.plan.trailing.clone() {
                state.items[idx].inline.paint(window, cx);
            }
        });
        if let Some(menu) = &mut prepaint.menu {
            menu.paint(window, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_plan_fits_when_space_permits_and_collapses_middle_by_max_items_or_width() {
        let widths = [px(60.), px(90.), px(80.), px(50.), px(70.)];
        let sep = px(20.);
        let trigger = px(32.);
        // Full width = 350 + 4 * 20 = 430.

        assert_eq!(
            compute_collapse_plan(&widths, sep, trigger, None, px(500.)),
            CollapsePlan {
                leading: 0..5,
                collapsed: 0..0,
                trailing: 5..5,
            }
        );

        // Explicit max_items(3) keeps first + 2 trailing, collapsing indices 1..3.
        assert_eq!(
            compute_collapse_plan(&widths, sep, trigger, Some(3), px(500.)),
            CollapsePlan {
                leading: 0..1,
                collapsed: 1..3,
                trailing: 3..5,
            }
        );

        // Explicit max_items(2) keeps first + current, collapsing indices 1..4.
        assert_eq!(
            compute_collapse_plan(&widths, sep, trigger, Some(2), px(500.)),
            CollapsePlan {
                leading: 0..1,
                collapsed: 1..4,
                trailing: 4..5,
            }
        );

        // Width-constrained (340 px): fits Home (60) + sep (20) + trigger (32) + sep (20) + 2026 (50) + sep (20) + Report (70) = 272 <= 340,
        // but adding Projects (80 + 20 = 100) would reach 372 > 340.
        assert_eq!(
            compute_collapse_plan(&widths, sep, trigger, None, px(340.)),
            CollapsePlan {
                leading: 0..1,
                collapsed: 1..3,
                trailing: 3..5,
            }
        );

        // Tighter width (220 px): fits Home + trigger + Report (202 <= 220), collapsing 1..4.
        assert_eq!(
            compute_collapse_plan(&widths, sep, trigger, None, px(220.)),
            CollapsePlan {
                leading: 0..1,
                collapsed: 1..4,
                trailing: 4..5,
            }
        );

        // Fewer than 3 items never collapse middle segments.
        assert_eq!(
            compute_collapse_plan(&[px(80.), px(90.)], sep, trigger, Some(2), px(100.)),
            CollapsePlan {
                leading: 0..2,
                collapsed: 0..0,
                trailing: 2..2,
            }
        );
    }
}
