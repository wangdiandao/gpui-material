use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AccessibleAction, AnyElement, App, AvailableSpace, Bounds, ContentMask, Element, ElementId,
    FocusHandle, GlobalElementId, Hsla, InspectorElementId, InteractiveElement, IntoElement,
    LayoutId, ParentElement, Pixels, Point, RenderOnce, Role, SharedString, Size,
    StatefulInteractiveElement, Style, Styled, Window, div, point, px, relative, size,
};
use gpui_base::Button;
use gpui_material_foundation::current_material_theme;
use gpui_material_theme::Direction;

use crate::{ButtonColors, IconButton, Menu, MenuItem};

const TRIGGER_SIZE: Pixels = px(48.);
type OnActivate = Rc<dyn Fn(&mut Window, &mut App)>;
type MenuItemBuilder = Box<dyn FnOnce(AppBarMenuState) -> MenuItem>;
type OverflowIndicator = Box<dyn FnOnce(AppBarMenuState) -> AnyElement>;
type DisplacedFocus = Rc<RefCell<Option<(ElementId, FocusHandle)>>>;

enum ItemContent {
    Clickable {
        label: SharedString,
        icon: Rc<dyn Fn() -> AnyElement>,
        on_activate: Option<OnActivate>,
    },
    Custom {
        inline: AnyElement,
        menu: MenuItemBuilder,
    },
}

/// One action with an inline representation and an overflow menu representation.
pub struct AppBarItem {
    id: ElementId,
    disabled: bool,
    content: ItemContent,
}

impl AppBarItem {
    /// Creates an icon button and a matching named menu command.
    /// The icon factory may be called for either representation.
    pub fn clickable<E: IntoElement>(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        icon: impl Fn() -> E + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            disabled: false,
            content: ItemContent::Clickable {
                label: label.into(),
                icon: Rc::new(move || icon().into_any_element()),
                on_activate: None,
            },
        }
    }

    /// Supplies an inline element and the corresponding Material menu item.
    pub fn custom(
        id: impl Into<ElementId>,
        inline: impl IntoElement,
        menu: impl FnOnce(AppBarMenuState) -> MenuItem + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            disabled: false,
            content: ItemContent::Custom {
                inline: inline.into_any_element(),
                menu: Box::new(menu),
            },
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the activation callback for both representations of a clickable item.
    pub fn on_activate(mut self, action: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        if let ItemContent::Clickable { on_activate, .. } = &mut self.content {
            *on_activate = Some(Rc::new(action));
        }
        self
    }

    fn prepare(self, ink: Hsla, window: &mut Window, cx: &mut App) -> Prepared {
        let focus = window
            .use_keyed_state((self.id.clone(), "app-bar-item"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let (mut inline, menu) = match self.content {
            ItemContent::Clickable {
                label,
                icon,
                on_activate,
            } => {
                let mut inline = IconButton::new(self.id.clone(), label.clone(), icon())
                    .colors(ButtonColors {
                        content: Some(ink),
                        ..Default::default()
                    })
                    .disabled(self.disabled)
                    .track_focus(&focus);
                if let Some(on_activate) = on_activate.clone() {
                    inline = inline.on_activate(move |window, cx| on_activate(window, cx));
                }
                let id = self.id.clone();
                let disabled = self.disabled;
                let menu = move |_| {
                    let mut item = MenuItem::new(id, label)
                        .leading(move || icon())
                        .disabled(disabled);
                    if let Some(on_activate) = on_activate {
                        item = item.on_activate(move |window, cx| on_activate(window, cx));
                    }
                    item
                };
                (inline.into_any_element(), Box::new(menu) as MenuItemBuilder)
            }
            ItemContent::Custom { inline, menu } => (
                div()
                    .id(self.id.clone())
                    .track_focus(&focus)
                    .child(inline)
                    .into_any_element(),
                menu,
            ),
        };
        let preferred = inline.layout_as_root(
            size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
            window,
            cx,
        );
        Prepared {
            id: self.id,
            inline,
            menu: Some(menu),
            focus,
            preferred,
            enabled: !self.disabled,
        }
    }
}

struct Prepared {
    id: ElementId,
    inline: AnyElement,
    menu: Option<MenuItemBuilder>,
    focus: FocusHandle,
    preferred: Size<Pixels>,
    enabled: bool,
}

/// Handle to the active strip's overflow menu, available to its item and trigger builders.
#[derive(Clone)]
pub struct AppBarMenuState {
    shown: Rc<Cell<bool>>,
    has_overflow: Rc<Cell<bool>>,
    view: gpui::EntityId,
}

impl AppBarMenuState {
    pub fn is_showing(&self) -> bool {
        self.shown.get()
    }

    pub fn show(&self, window: &mut Window, cx: &mut App) {
        if self.has_overflow.get() {
            self.set(true, window, cx);
        }
    }

    pub fn dismiss(&self, window: &mut Window, cx: &mut App) {
        self.set(false, window, cx);
    }

    fn set(&self, shown: bool, window: &mut Window, cx: &mut App) {
        if self.shown.replace(shown) != shown {
            cx.notify(self.view);
            window.refresh();
        }
    }
}

struct OverflowState {
    shown: Rc<Cell<bool>>,
    has_overflow: Rc<Cell<bool>>,
    focus: FocusHandle,
    displaced: DisplacedFocus,
}

struct Strip {
    id: ElementId,
    items: Vec<AppBarItem>,
    label: SharedString,
    overflow_label: SharedString,
    overflow_indicator: Option<OverflowIndicator>,
    max_item_count: usize,
}

impl Strip {
    fn new(id: impl Into<ElementId>, items: impl IntoIterator<Item = AppBarItem>) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
            label: "App bar actions".into(),
            overflow_label: "More actions".into(),
            overflow_indicator: None,
            max_item_count: usize::MAX,
        }
    }
}

/// Horizontal actions with responsive overflow.
#[derive(IntoElement)]
pub struct AppBarRow(Strip);

/// Vertical actions with responsive overflow.
#[derive(IntoElement)]
pub struct AppBarColumn(Strip);

macro_rules! action_strip {
    ($name:ident, $axis:expr) => {
        impl $name {
            pub fn new(
                id: impl Into<ElementId>,
                items: impl IntoIterator<Item = AppBarItem>,
            ) -> Self {
                Self(Strip::new(id, items))
            }

            pub fn label(mut self, label: impl Into<SharedString>) -> Self {
                self.0.label = label.into();
                self
            }

            /// Limits inline children, including the overflow trigger when it is needed.
            pub fn max_item_count(mut self, count: usize) -> Self {
                self.0.max_item_count = count.max(1);
                self
            }

            pub fn overflow_label(mut self, label: impl Into<SharedString>) -> Self {
                self.0.overflow_label = label.into();
                self
            }

            /// Replaces the decorative content of the built-in overflow button.
            pub fn overflow_indicator<E: IntoElement>(
                mut self,
                build: impl FnOnce(AppBarMenuState) -> E + 'static,
            ) -> Self {
                self.0.overflow_indicator =
                    Some(Box::new(move |state| build(state).into_any_element()));
                self
            }
        }

        impl RenderOnce for $name {
            fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
                StripLayout::new(self.0, $axis, window, cx)
            }
        }
    };
}

action_strip!(AppBarRow, Axis::Horizontal);
action_strip!(AppBarColumn, Axis::Vertical);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn extent(self, size: Size<Pixels>) -> Pixels {
        match self {
            Self::Horizontal => size.width,
            Self::Vertical => size.height,
        }
    }

    fn cross(self, size: Size<Pixels>) -> Pixels {
        match self {
            Self::Horizontal => size.height,
            Self::Vertical => size.width,
        }
    }

    fn size(self, extent: Pixels, cross: Pixels) -> Size<Pixels> {
        match self {
            Self::Horizontal => size(extent, cross),
            Self::Vertical => size(cross, extent),
        }
    }

    fn origin(
        self,
        bounds: Bounds<Pixels>,
        child: Size<Pixels>,
        offset: Pixels,
        direction: Direction,
    ) -> Point<Pixels> {
        let delta = (self.cross(bounds.size) - self.cross(child)).max(px(0.)) / 2.;
        match (self, direction) {
            (Self::Horizontal, Direction::Rtl) => {
                point(bounds.right() - offset - child.width, bounds.top() + delta)
            }
            (Self::Horizontal, Direction::Ltr) => {
                point(bounds.left() + offset, bounds.top() + delta)
            }
            (Self::Vertical, _) => point(bounds.left() + delta, bounds.top() + offset),
        }
    }
}

struct StripLayout {
    strip: Strip,
    axis: Axis,
    direction: Direction,
    ink: Hsla,
    menu: AppBarMenuState,
    overflow_focus: FocusHandle,
    displaced: DisplacedFocus,
}

impl StripLayout {
    fn new(strip: Strip, axis: Axis, window: &mut Window, cx: &mut App) -> Self {
        let overflow =
            window.use_keyed_state((strip.id.clone(), "app-bar-overflow"), cx, |_, cx| {
                OverflowState {
                    shown: Rc::new(Cell::new(false)),
                    has_overflow: Rc::new(Cell::new(false)),
                    focus: cx.focus_handle(),
                    displaced: Rc::new(RefCell::new(None)),
                }
            });
        let overflow = overflow.read(cx);
        Self {
            strip,
            axis,
            direction: current_material_theme(window, cx).layout.direction,
            ink: window.text_style().color,
            menu: AppBarMenuState {
                shown: overflow.shown.clone(),
                has_overflow: overflow.has_overflow.clone(),
                view: window.current_view(),
            },
            overflow_focus: overflow.focus.clone(),
            displaced: overflow.displaced.clone(),
        }
    }

    fn transfer_focus(
        &self,
        shown: usize,
        items: &[Prepared],
        overflow: bool,
        window: &mut Window,
        cx: &mut App,
    ) {
        let in_overflow = self.overflow_focus.is_focused(window);
        let previous_overflow = self.menu.has_overflow.replace(overflow);
        let menu_was_open = self.menu.is_showing();
        let mut displaced = self.displaced.borrow_mut();
        if overflow
            && let Some(hidden) = items[shown..]
                .iter()
                .find(|item| item.focus.contains_focused(window, cx))
        {
            let old_focus = window.focused(cx).unwrap_or_else(|| hidden.focus.clone());
            *displaced = Some((hidden.id.clone(), old_focus));
            focus_after_layout(self.overflow_focus.clone(), window, cx);
            return;
        }
        if !overflow && previous_overflow {
            self.menu.shown.set(false);
        }
        let returned = displaced
            .as_ref()
            .filter(|(id, _)| {
                items[..shown]
                    .iter()
                    .any(|item| item.enabled && item.id == *id)
            })
            .map(|(_, focus)| focus.clone());
        let restore = if !overflow && (in_overflow || menu_was_open) {
            returned.or_else(|| {
                items[..shown]
                    .iter()
                    .find(|item| item.enabled)
                    .map(|item| item.focus.clone())
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

    fn overflow_menu(&mut self, hidden: &mut [Prepared], size: Size<Pixels>) -> AnyElement {
        let state = self.menu.clone();
        let entries: Vec<_> = hidden
            .iter_mut()
            .filter_map(|item| item.menu.take().map(|build| build(state.clone())))
            .collect();
        let trigger_focus = self.overflow_focus.clone();
        let label = self.strip.overflow_label.clone();
        let trigger_id = ElementId::from((self.strip.id.clone(), "overflow"));
        let ink = self.ink;
        let glyph = self.strip.overflow_indicator.take().map_or_else(
            || {
                let unit = px(4.).min(size.width / 4.).min(size.height / 4.);
                div()
                    .flex()
                    .flex_col()
                    .gap(unit / 2.)
                    .children((0..3).map(|_| div().size(unit).rounded_full().bg(ink)))
                    .into_any_element()
            },
            |build| build(state.clone()),
        );
        let observed = state.clone();
        Menu::composed(
            (self.strip.id.clone(), "menu"),
            entries,
            move |_, toggle| {
                let return_focus = trigger_focus.clone();
                Button::new(trigger_id)
                    .track_focus(&trigger_focus)
                    .accessibility_label(label)
                    .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                        return_focus.focus(window, cx);
                        toggle(window, cx);
                    })
                    .w(size.width)
                    .h(size.height)
                    .p_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .rounded_full()
                    .text_color(ink)
                    .hover(move |button| button.bg(ink.opacity(0.08)))
                    .focus_visible(move |button| button.border_2().border_color(ink))
                    .child(glyph)
                    .into_any_element()
            },
        )
        .open(state.is_showing())
        .on_open_change(move |open, window, cx| observed.set(open, window, cx))
        .into_any_element()
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

impl IntoElement for StripLayout {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for StripLayout {
    type RequestLayoutState = Vec<Prepared>;
    type PrepaintState = (usize, Option<AnyElement>);

    fn id(&self) -> Option<ElementId> {
        Some(self.strip.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn a11y_role(&self) -> Option<Role> {
        Some(Role::Group)
    }

    fn write_a11y_info(&self, node: &mut gpui::accesskit::Node) {
        node.set_label(self.strip.label.as_ref());
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let items: Vec<_> = std::mem::take(&mut self.strip.items)
            .into_iter()
            .map(|item| item.prepare(self.ink, window, cx))
            .collect();
        let limited = items.len() > self.strip.max_item_count;
        let preferred_count = if limited {
            self.strip.max_item_count - 1
        } else {
            items.len()
        };
        let main = items[..preferred_count]
            .iter()
            .map(|item| self.axis.extent(item.preferred))
            .sum::<Pixels>()
            + if limited { TRIGGER_SIZE } else { px(0.) };
        let cross = items
            .iter()
            .map(|item| self.axis.cross(item.preferred))
            .fold(
                if items.is_empty() {
                    px(0.)
                } else {
                    TRIGGER_SIZE
                },
                Pixels::max,
            );
        let style = Style {
            size: self.axis.size(main, cross).map(Into::into),
            min_size: size(px(0.).into(), px(0.).into()),
            max_size: size(relative(1.).into(), relative(1.).into()),
            ..Default::default()
        };
        (window.request_layout(style, None, cx), items)
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        items: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let length = self.axis.extent(bounds.size);
        let breadth = self.axis.cross(bounds.size);
        let has_space = length > px(0.) && breadth > px(0.);
        let shown = if has_space {
            fit(
                items.iter().map(|item| self.axis.extent(item.preferred)),
                self.strip.max_item_count,
                length,
                TRIGGER_SIZE,
            )
        } else {
            0
        };
        let overflow = has_space && shown < items.len();
        self.transfer_focus(shown, items, overflow, window, cx);
        let mut offset = px(0.);
        for item in &mut items[..shown] {
            let extent = self.axis.extent(item.preferred);
            let child = if self.axis.cross(item.preferred) > breadth {
                let cap = self.axis.size(extent, breadth);
                item.inline
                    .layout_as_root(cap.map(AvailableSpace::Definite), window, cx)
            } else {
                item.preferred
            };
            let origin = self.axis.origin(bounds, child, offset, self.direction);
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                item.inline.prepaint_at(origin, window, cx)
            });
            offset += extent;
        }
        let menu = overflow.then(|| {
            let size = self
                .axis
                .size(TRIGGER_SIZE.min(length - offset), TRIGGER_SIZE.min(breadth));
            let mut menu = self.overflow_menu(&mut items[shown..], size);
            let child = menu.layout_as_root(size.map(AvailableSpace::Definite), window, cx);
            menu.prepaint_at(
                self.axis.origin(bounds, child, offset, self.direction),
                window,
                cx,
            );
            menu
        });
        (shown, menu)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        items: &mut Self::RequestLayoutState,
        (shown, menu): &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for item in &mut items[..*shown] {
                item.inline.paint(window, cx);
            }
        });
        if let Some(menu) = menu {
            menu.paint(window, cx);
        }
    }
}

fn fit<I: Clone + ExactSizeIterator<Item = Pixels>>(
    extents: I,
    max_count: usize,
    available: Pixels,
    trigger: Pixels,
) -> usize {
    if extents.len() <= max_count && extents.clone().sum::<Pixels>() <= available {
        return extents.len();
    }
    let mut remaining = available - trigger;
    extents
        .take(max_count.saturating_sub(1))
        .take_while(|extent| {
            remaining -= *extent;
            remaining >= px(0.)
        })
        .count()
}
