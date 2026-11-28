use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, AvailableSpace, Bounds, ContentMask, Element, ElementId,
    FocusHandle, GlobalElementId, Hsla, InspectorElementId, InteractiveElement, IntoElement,
    LayoutId, ParentElement, Pixels, Point, RenderOnce, Role, SharedString, Size,
    StatefulInteractiveElement, Style, Styled, Window, div, point, px, relative, size,
};
use gpui_base::Button;
use gpui_material_foundation::{current_material_strings, current_material_theme};
use gpui_material_theme::Direction;

use crate::{
    ButtonColors, IconButton, Menu, MenuItem,
    navigation::overflow_strip::{OverflowMenuController, OverflowStripHandle},
};

const TRIGGER_SIZE: Pixels = px(48.);
type OnActivate = Rc<dyn Fn(&mut Window, &mut App)>;
type MenuItemBuilder = Box<dyn FnOnce(AppBarMenuState) -> MenuItem>;
type OverflowIndicator = Box<dyn FnOnce(AppBarMenuState) -> AnyElement>;

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

    /// Disables the action in both its inline and overflow menu representations.
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

    fn prepare(
        self,
        strip_id: &ElementId,
        ink: Hsla,
        window: &mut Window,
        cx: &mut App,
    ) -> Prepared {
        let scoped_id: ElementId = (strip_id.clone(), self.id.to_string()).into();
        let focus = window
            .use_keyed_state((scoped_id.clone(), "app-bar-item"), cx, |_, cx| {
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
                let mut inline = IconButton::new(scoped_id.clone(), label.clone(), icon())
                    .colors(ButtonColors {
                        content: Some(ink),
                        ..Default::default()
                    })
                    .disabled(self.disabled)
                    .track_focus(&focus);
                if let Some(on_activate) = on_activate.clone() {
                    inline = inline.on_activate(move |window, cx| on_activate(window, cx));
                }
                let menu_id = scoped_id.clone();
                let disabled = self.disabled;
                let menu = move |_| {
                    let mut item = MenuItem::new(menu_id, label)
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
                    .id(scoped_id.clone())
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
    controller: OverflowMenuController,
}

impl AppBarMenuState {
    /// Returns whether the overflow menu is currently open.
    pub fn is_showing(&self) -> bool {
        self.controller.is_showing()
    }

    /// Opens the overflow menu if any items are currently overflowed.
    pub fn show(&self, window: &mut Window, cx: &mut App) {
        self.controller.show(window, cx);
    }

    /// Closes the overflow menu.
    pub fn dismiss(&self, window: &mut Window, cx: &mut App) {
        self.controller.dismiss(window, cx);
    }

    fn set(&self, shown: bool, window: &mut Window, cx: &mut App) {
        self.controller.set(shown, window, cx);
    }
}

struct Strip {
    id: ElementId,
    items: Vec<AppBarItem>,
    label: Option<SharedString>,
    overflow_label: Option<SharedString>,
    overflow_indicator: Option<OverflowIndicator>,
    max_item_count: usize,
}

impl Strip {
    fn new(id: impl Into<ElementId>, items: impl IntoIterator<Item = AppBarItem>) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
            label: None,
            overflow_label: None,
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
            /// Creates an action strip with the given element ID and items.
            pub fn new(
                id: impl Into<ElementId>,
                items: impl IntoIterator<Item = AppBarItem>,
            ) -> Self {
                Self(Strip::new(id, items))
            }

            /// Sets the accessible label of the action strip toolbar (defaults to
            /// [`gpui_material_theme::MaterialStrings::app_bar_actions`], `"App bar actions"`).
            pub fn label(mut self, label: impl Into<SharedString>) -> Self {
                self.0.label = Some(label.into());
                self
            }

            /// Limits inline children, including the overflow trigger when it is needed.
            pub fn max_item_count(mut self, count: usize) -> Self {
                self.0.max_item_count = count.max(1);
                self
            }

            /// Sets the accessible label of the overflow menu trigger button (defaults to
            /// [`gpui_material_theme::MaterialStrings::more_actions`], `"More actions"`).
            pub fn overflow_label(mut self, label: impl Into<SharedString>) -> Self {
                self.0.overflow_label = Some(label.into());
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
    label: SharedString,
    overflow_label: SharedString,
    axis: Axis,
    direction: Direction,
    ink: Hsla,
    overflow: OverflowStripHandle<ElementId>,
}

impl StripLayout {
    fn new(mut strip: Strip, axis: Axis, window: &mut Window, cx: &mut App) -> Self {
        let strings = current_material_strings(window, cx);
        let label = strip
            .label
            .take()
            .unwrap_or_else(|| SharedString::from(strings.app_bar_actions.clone()));
        let overflow_label = strip
            .overflow_label
            .take()
            .unwrap_or_else(|| SharedString::from(strings.more_actions.clone()));
        drop(strings);
        let overflow = OverflowStripHandle::use_state(&strip.id, "app-bar-overflow", window, cx);
        Self {
            strip,
            label,
            overflow_label,
            axis,
            direction: current_material_theme(window, cx).layout.direction,
            ink: window.text_style().color,
            overflow,
        }
    }

    fn menu_state(&self) -> AppBarMenuState {
        AppBarMenuState {
            controller: self.overflow.controller.clone(),
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
        let hidden_focused = overflow.then(|| {
            items[shown..]
                .iter()
                .find(|item| item.focus.contains_focused(window, cx))
                .map(|item| (item.id.clone(), item.focus.clone()))
        });
        self.overflow.transfer_focus(
            overflow,
            hidden_focused.flatten(),
            |id| {
                items[..shown]
                    .iter()
                    .any(|item| item.enabled && item.id == *id)
            },
            || {
                items[..shown]
                    .iter()
                    .find(|item| item.enabled)
                    .map(|item| item.focus.clone())
            },
            window,
            cx,
        );
    }

    fn overflow_menu(&mut self, hidden: &mut [Prepared], size: Size<Pixels>) -> AnyElement {
        let state = self.menu_state();
        let entries: Vec<_> = hidden
            .iter_mut()
            .filter_map(|item| item.menu.take().map(|build| build(state.clone())))
            .collect();
        let trigger_focus = self.overflow.focus.clone();
        let label = self.overflow_label.clone();
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
        node.set_label(self.label.as_ref());
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let strip_id = self.strip.id.clone();
        let items: Vec<_> = std::mem::take(&mut self.strip.items)
            .into_iter()
            .map(|item| item.prepare(&strip_id, self.ink, window, cx))
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

#[cfg(test)]
mod tests {
    use super::fit;
    use gpui::px;

    #[test]
    fn fit_keeps_all_when_within_limit_and_reserves_trigger_on_overflow() {
        let widths = [px(48.), px(48.), px(48.)];
        assert_eq!(fit(widths.into_iter(), 5, px(200.), px(48.)), 3);
        assert_eq!(fit(widths.into_iter(), 2, px(200.), px(48.)), 1);
        assert_eq!(fit(widths.into_iter(), 5, px(120.), px(48.)), 1);
        assert_eq!(fit(widths.into_iter(), 5, px(40.), px(48.)), 0);
    }
}
