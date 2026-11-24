use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::menu_appearance::Paint;
pub use crate::menu_appearance::{MenuColor, MenuItemStyle, MenuSurfaceStyle};

use crate::menu_appearance::menu_pivot;
use crate::{Icon, IconName, SemanticTone};
use gpui::{
    Anchor, AnyElement, App, Bounds, ElementId, FocusHandle, Focusable, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Pixels, Point, RenderOnce, Role, ScrollHandle,
    SharedString, Size, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
    px,
};
use gpui_base::{
    Popover, PopoverState, Popup, Positioner, TestSupportExt,
    actions::{Cancel, Confirm},
};
use gpui_material_foundation::{OverlayMotion, ScaledOverlay};
use gpui_material_foundation::{ThemeScope, current_material_theme, logical_anchor};
use gpui_material_theme::{CornerShape, Direction, TypeRole, TypographyEmphasis};

mod content;

use content::{MenuExitRow, menu_content, menu_exit};

type Action = Rc<dyn Fn(&mut Window, &mut App)>;
type Change = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type ItemContent = Rc<dyn Fn() -> AnyElement>;
type ComposedTrigger = Box<dyn FnOnce(bool, Action) -> AnyElement>;
type LazyItems = Box<dyn FnOnce(&mut Window, &mut App) -> Vec<MenuItem>>;

enum MenuItems {
    Eager(Vec<MenuItem>),
    Lazy(LazyItems),
}

impl MenuItems {
    fn resolve(self, window: &mut Window, cx: &mut App) -> Vec<MenuItem> {
        match self {
            Self::Eager(items) => items,
            Self::Lazy(build) => build(window, cx),
        }
    }
}

/// An item in a Material menu. Callers retain command and selection state.
pub struct MenuItem {
    id: ElementId,
    label: SharedString,
    disabled: bool,
    kind: Kind,
    slots: ItemSlots,
    on_checked_change: Option<Change>,
    tone: SemanticTone,
    style: MenuItemStyle,
}

enum Kind {
    Action(Option<Action>),
    Check(bool, Option<Action>),
    Select(bool, Option<Action>),
    Heading,
    Divider,
    Section {
        rows: Vec<MenuItem>,
        style: MenuSurfaceStyle,
        inactive_shape: Option<CornerShape>,
    },
    Submenu(Vec<MenuItem>),
}

#[derive(Clone, Default)]
struct ItemSlots {
    content: Option<ItemContent>,
    leading: Option<ItemContent>,
    supporting: Option<ItemContent>,
    trailing: Option<ItemContent>,
    shortcut: Option<SharedString>,
    selection_icon: Option<ItemContent>,
}

impl ItemSlots {
    fn label(&self, label: SharedString) -> AnyElement {
        self.content
            .as_ref()
            .map_or_else(|| label.into_any_element(), |content| content())
    }

    fn children(
        &self,
        label: SharedString,
        marked: bool,
        submenu: bool,
        paint: Paint<'_>,
        (supporting_color, shortcut_color): (gpui::Hsla, gpui::Hsla),
        active: bool,
    ) -> Vec<AnyElement> {
        let motion = paint.motion;
        let icon_size = motion.px(18.);
        let leading = if marked {
            Some(self.selection_icon.as_ref().map_or_else(
                || {
                    Icon::new(IconName::Check)
                        .size(icon_size)
                        .into_any_element()
                },
                |icon| icon(),
            ))
        } else {
            self.leading.as_ref().map(|icon| icon())
        };
        let supporting_scale = paint.theme.typography.text_scale();
        let body = div()
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .child(div().truncate().child(self.label(label)))
            .children(self.supporting.as_ref().map(|supporting| {
                div()
                    .text_size(motion.px(12. * supporting_scale))
                    .line_height(motion.px(16. * supporting_scale))
                    .text_color(supporting_color)
                    .child(supporting())
            }));
        let mut children = vec![
            div()
                .min_w(icon_size)
                .flex_none()
                .children(leading)
                .into_any_element(),
            body.into_any_element(),
        ];
        if let Some(trailing) = self.trailing.as_ref() {
            children.push(trailing());
        } else if let Some(shortcut) = self.shortcut.as_ref().filter(|text| !text.is_empty()) {
            let label_small = paint
                .theme
                .typography
                .scale_style(TypeRole::LabelSmall.style(TypographyEmphasis::Baseline));
            let hint = div()
                .flex_none()
                .text_size(motion.px(label_small.size))
                .line_height(motion.px(label_small.line_height))
                .text_color(shortcut_color)
                .child(shortcut.clone());
            children.push(if active {
                hint.id("shortcut").test_support().into_any_element()
            } else {
                hint.into_any_element()
            });
        }
        children.extend(submenu.then(|| {
            Icon::new(IconName::ChevronRight)
                .size(icon_size)
                .mirror_in_rtl(true)
                .into_any_element()
        }));
        children
    }
}

impl MenuItem {
    /// Creates a standard action menu item with the given element ID and label.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            kind: Kind::Action(None),
            slots: ItemSlots::default(),
            on_checked_change: None,
            tone: SemanticTone::Default,
            style: MenuItemStyle::default(),
        }
    }

    /// Creates a non-interactive section heading row that is skipped by keyboard navigation.
    pub fn group(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            kind: Kind::Heading,
            ..Self::new(id, title)
        }
    }

    /// Creates a non-interactive horizontal divider row inside a menu that is
    /// skipped by keyboard navigation.
    pub fn divider(id: impl Into<ElementId>) -> Self {
        Self {
            kind: Kind::Divider,
            ..Self::new(id, "")
        }
    }

    /// Creates a surfaced group with rows in the owning menu's navigation order.
    pub fn section(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        rows: impl IntoIterator<Item = MenuItem>,
    ) -> Self {
        Self {
            kind: Kind::Section {
                rows: rows.into_iter().collect(),
                style: MenuSurfaceStyle::default(),
                inactive_shape: None,
            },
            ..Self::new(id, name)
        }
    }

    /// Sets the styled section's surface appearance.
    pub fn group_appearance(mut self, appearance: MenuSurfaceStyle) -> Self {
        if let Kind::Section { style, .. } = &mut self.kind {
            *style = appearance;
        }
        self
    }

    /// Sets the section shape used after the pointer leaves the section.
    pub fn inactive_shape(mut self, shape: CornerShape) -> Self {
        if let Kind::Section { inactive_shape, .. } = &mut self.kind {
            *inactive_shape = Some(shape);
        }
        self
    }

    /// Configures the appearance of an action, checked or selected row.
    pub fn appearance(mut self, style: MenuItemStyle) -> Self {
        self.style = style;
        self
    }

    /// Applies a Material 3 semantic tone to this row's foreground and selected roles.
    /// Explicit [`MenuItemStyle`] overrides and disabled styling still take precedence.
    pub fn tone(mut self, tone: SemanticTone) -> Self {
        self.tone = tone;
        self
    }

    /// Shorthand for `.tone(SemanticTone::Danger)` on destructive menu commands.
    pub fn destructive(self) -> Self {
        self.tone(SemanticTone::Danger)
    }

    /// Renders a desktop shortcut hint in the trailing slot using `LabelSmall`
    /// and `OnSurfaceVariant` (or disabled color when disabled), unless an
    /// explicit `.trailing(...)` overrides it.
    pub fn shortcut(mut self, text: impl Into<SharedString>) -> Self {
        self.slots.shortcut = Some(text.into());
        self
    }

    /// Turns this row into a submenu trigger containing the given child items.
    pub fn submenu(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.kind = Kind::Submenu(items.into_iter().collect());
        self
    }

    /// Marks the item as disabled, preventing activation and applying disabled styling.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Turns this row into a checkable menu item with the given caller-owned checked state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.kind = Kind::Check(checked, self.action());
        self
    }

    /// Turns this row into a selectable menu item with the given caller-owned selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.kind = Kind::Select(selected, self.action());
        self
    }

    /// Replaces the visible label while retaining the name passed to `new`.
    /// Supply a repeatable factory so the content also appears during exit.
    pub fn content<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
        self.slots.content = Some(Rc::new(move || build().into_any_element()));
        self
    }

    /// Supplies a decorative element at the logical start of an unmarked row.
    pub fn leading<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
        self.slots.leading = Some(Rc::new(move || build().into_any_element()));
        self
    }

    /// Supplies additional visual content below the label.
    pub fn supporting<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
        self.slots.supporting = Some(Rc::new(move || build().into_any_element()));
        self
    }

    /// Supplies decorative content at the logical end of the row.
    pub fn trailing<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
        self.slots.trailing = Some(Rc::new(move || build().into_any_element()));
        self
    }

    /// Replaces the leading check mark of a checked or selected row.
    pub fn selection_icon<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
        self.slots.selection_icon = Some(Rc::new(move || build().into_any_element()));
        self
    }

    /// Requests the inverse of the supplied checked value on activation.
    /// The checked callback takes precedence over `on_activate` on checked rows.
    pub fn on_checked_change(
        mut self,
        callback: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_checked_change = Some(Rc::new(callback));
        self
    }

    fn action(&self) -> Option<Action> {
        match &self.kind {
            Kind::Action(action) | Kind::Check(_, action) | Kind::Select(_, action) => {
                action.clone()
            }
            _ => None,
        }
    }

    fn activation(&self) -> Option<Action> {
        if let Kind::Check(checked, _) = self.kind
            && let Some(callback) = &self.on_checked_change
        {
            let callback = callback.clone();
            return Some(Rc::new(move |window, cx| callback(!checked, window, cx)));
        }
        self.action()
    }

    /// Registers a callback invoked when an action, checked, or selected row is activated.
    pub fn on_activate(mut self, action: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        match &mut self.kind {
            Kind::Action(handler) | Kind::Check(_, handler) | Kind::Select(_, handler) => {
                *handler = Some(Rc::new(action));
            }
            _ => {}
        }
        self
    }
}

/// Positions a Material menu against a caller-owned, keyboard-focusable trigger.
/// Pointer and keyboard activation use the same Base popover focus lifecycle.
#[derive(IntoElement)]
pub struct Menu {
    id: ElementId,
    trigger: AnyElement,
    items: MenuItems,
    mouse_button: MouseButton,
    anchor: Anchor,
    close_parent: Option<Action>,
    enabled: bool,
    ancestor: Option<Rc<PopupBounds>>,
    composed_trigger: Option<ComposedTrigger>,
    open: Option<bool>,
    on_open_change: Option<Change>,
    on_dismiss: Option<Action>,
    color: MenuColor,
    appearance: MenuSurfaceStyle,
    decoration: Option<ItemContent>,
    scroll: Option<ScrollHandle>,
    offset: Pixels,
    position: Option<Point<Pixels>>,
}

#[derive(Default)]
struct PopupBounds {
    trigger: Cell<Bounds<Pixels>>,
    popup: Cell<Bounds<Pixels>>,
    context_position: Cell<Option<Point<Pixels>>>,
    context_was_open: Cell<bool>,
    context_focus: RefCell<Option<FocusHandle>>,
    context_previous_focus: RefCell<Option<FocusHandle>>,
    child: RefCell<Option<Rc<PopupBounds>>>,
    presented: RefCell<Option<PresentedMenu>>,
    scroll: ScrollHandle,
    motion: Cell<Option<OverlayMotion>>,
    open: Rc<Cell<bool>>,
}

#[derive(Clone)]
struct MenuPresentation {
    color: MenuColor,
    appearance: MenuSurfaceStyle,
    decoration: Option<ItemContent>,
    scroll: ScrollHandle,
}

struct PresentedMenu {
    items: Rc<[MenuExitRow]>,
    presentation: MenuPresentation,
}

impl PopupBounds {
    fn popup_contains(&self, point: &Point<Pixels>) -> bool {
        self.popup.get().contains(point)
            || self
                .child
                .borrow()
                .as_ref()
                .is_some_and(|child| child.open.get() && child.contains(point))
    }

    fn contains(&self, point: &Point<Pixels>) -> bool {
        self.trigger.get().contains(point) || self.popup_contains(point)
    }
}

impl Menu {
    /// Creates a menu anchored to `trigger` with eagerly constructed items.
    pub fn new(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        items: impl IntoIterator<Item = MenuItem>,
    ) -> Self {
        Self::with_items(
            id.into(),
            trigger.into_any_element(),
            MenuItems::Eager(items.into_iter().collect()),
        )
    }

    /// Creates a menu whose items are built lazily only while the menu is open.
    pub fn new_with<I>(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        build: impl FnOnce(&mut Window, &mut App) -> I + 'static,
    ) -> Self
    where
        I: IntoIterator<Item = MenuItem>,
    {
        Self::with_items(
            id.into(),
            trigger.into_any_element(),
            MenuItems::Lazy(Box::new(move |window, cx| {
                build(window, cx).into_iter().collect()
            })),
        )
    }

    fn with_items(id: ElementId, trigger: AnyElement, items: MenuItems) -> Self {
        Self {
            id,
            trigger,
            items,
            mouse_button: MouseButton::Left,
            anchor: Anchor::TopLeft,
            close_parent: None,
            enabled: true,
            ancestor: None,
            composed_trigger: None,
            open: None,
            on_open_change: None,
            on_dismiss: None,
            color: MenuColor::default(),
            appearance: MenuSurfaceStyle::default(),
            decoration: None,
            scroll: None,
            offset: px(4.),
            position: None,
        }
    }

    /// Disables menu activation while continuing to render the trigger element.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.enabled = !disabled;
        self
    }

    /// Opens on secondary click at the pointer location (viewport-clamped), or
    /// anchored to the trigger on `Shift+F10` / `ContextMenu`, without hijacking
    /// the trigger's own `Enter` or `Space` activation.
    pub fn context(mut self) -> Self {
        self.mouse_button = MouseButton::Right;
        self
    }

    /// Sets the anchor corner used to align the popup against its trigger (default [`Anchor::TopLeft`]).
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets the outward gap between the trigger and popup.
    pub fn offset(mut self, offset: Pixels) -> Self {
        if f32::from(offset).is_finite() {
            self.offset = offset;
        }
        self
    }

    /// Anchors the popup at a window-local point. It remains clamped to the viewport.
    pub fn position(mut self, position: Point<Pixels>) -> Self {
        self.position = (f32::from(position.x).is_finite() && f32::from(position.y).is_finite())
            .then_some(position);
        self
    }

    /// Applies a semantic color preset to the menu and its submenus.
    pub fn color(mut self, color: MenuColor) -> Self {
        self.color = color;
        self
    }

    /// Applies overrides to this popup's surface.
    pub fn appearance(mut self, appearance: MenuSurfaceStyle) -> Self {
        self.appearance = appearance;
        self
    }

    /// Renders arbitrary content before the menu rows. Use a repeatable factory
    /// for content that also needs to appear during exit.
    pub fn content<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
        self.decoration = Some(Rc::new(move || build().into_any_element()));
        self
    }

    /// Tracks the popup's scrolling through a caller-owned handle.
    pub fn track_scroll(mut self, scroll: &ScrollHandle) -> Self {
        self.scroll = Some(scroll.clone());
        self
    }

    /// Builds a trigger from the displayed open value and its activation callback.
    ///
    /// Keep the trigger focusable and use the supplied callback for a custom
    /// accessibility action. The menu owns pointer and keyboard activation.
    pub fn composed(
        id: impl Into<ElementId>,
        items: impl IntoIterator<Item = MenuItem>,
        trigger: impl FnOnce(bool, Action) -> AnyElement + 'static,
    ) -> Self {
        let mut menu = Self::new(id, div(), items);
        menu.composed_trigger = Some(Box::new(trigger));
        menu
    }

    /// Sets the displayed open value from caller-owned state.
    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Receives user requests to open or close the menu. In controlled mode,
    /// apply the requested value and render with `open` to accept a request.
    pub fn on_open_change(
        mut self,
        callback: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(callback));
        self
    }

    /// Receives a user request to close. Setting `open(false)` externally
    /// synchronizes the menu without running this callback.
    pub fn on_dismiss(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(callback));
        self
    }
}

fn build_exit_branch(
    id: &ElementId,
    bounds: &Rc<PopupBounds>,
    motion: OverlayMotion,
    direction: Direction,
    window: &Window,
    cx: &App,
) -> gpui::Div {
    // Closing the parent removes Base's nested popup hosts immediately.
    // Use the last presented branch to finish their own visual exits.
    let mut exits = div();
    let mut level = Some(bounds.clone());
    let mut depth = 0;
    while let Some(bounds) = level {
        let Some(previous) = bounds.motion.get() else {
            break;
        };
        let motion = if depth == 0 {
            motion
        } else {
            OverlayMotion {
                scale: 0.8 + (motion.scale - 0.8) * (previous.scale - 0.8) / 0.2,
                alpha: motion.alpha * previous.alpha,
            }
        };
        exits = exits.children(menu_exit(
            (id, depth),
            &bounds,
            motion,
            direction,
            window,
            cx,
        ));
        level = bounds.child.borrow().clone();
        depth += 1;
    }
    exits
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let direction = current_material_theme(window, cx).layout.direction;
        let submenu = self.close_parent.is_some();
        let is_context = self.mouse_button == MouseButton::Right;
        if !self.enabled {
            return self.trigger;
        }
        let view = window.current_view();
        let bounds = window
            .use_keyed_state((self.id.clone(), "menu-bounds"), cx, |_, _| {
                Rc::new(PopupBounds::default())
            })
            .read(cx)
            .clone();
        let open_state = bounds.open.clone();
        let controlled = self.open.is_some();
        let open = self.open.unwrap_or_else(|| open_state.get());
        open_state.set(open);
        let key_state = open_state.clone();
        let trigger_state = open_state.clone();
        let context_open_state = open_state.clone();
        let request: Change = {
            let on_open_change = self.on_open_change;
            let on_dismiss = self.on_dismiss;
            Rc::new(move |requested, window, cx| {
                if !controlled {
                    open_state.set(requested);
                    cx.notify(view);
                }
                if let Some(callback) = &on_open_change {
                    callback(requested, window, cx);
                }
                if !requested && let Some(callback) = &on_dismiss {
                    callback(window, cx);
                }
            })
        };
        let trigger: Action = {
            let request = request.clone();
            Rc::new(move |window, cx| {
                let displayed = if controlled {
                    open
                } else {
                    trigger_state.get()
                };
                request(!displayed, window, cx);
            })
        };
        let key_trigger = trigger.clone();
        let mouse_trigger = trigger.clone();
        let context_open: Action = {
            let request = request.clone();
            Rc::new(move |window, cx| request(true, window, cx))
        };
        let context_key_open = context_open.clone();
        let popover_request = request.clone();
        let popup_request = request;
        let motion = OverlayMotion::sample(&self.id, open, window, cx);
        let position = bounds.clone();
        let context_mouse_bounds = bounds.clone();
        let context_key_bounds = bounds.clone();
        let ancestor = self.ancestor.clone();
        let anchor = logical_anchor(self.anchor, direction);
        if !open && motion.is_none() {
            bounds.presented.borrow_mut().take();
            bounds.child.borrow_mut().take();
            bounds.context_position.set(None);
        }
        let explicit_position = self
            .position
            .or_else(|| is_context.then(|| bounds.context_position.get()).flatten());
        let presentation = MenuPresentation {
            color: self.color,
            appearance: self.appearance,
            decoration: self.decoration,
            scroll: self.scroll.unwrap_or_else(|| bounds.scroll.clone()),
        };
        let resolved_items = open.then(|| self.items.resolve(window, cx));
        if let Some(items) = resolved_items.as_ref()
            && !cx.reduce_motion()
        {
            *bounds.presented.borrow_mut() = Some(PresentedMenu {
                items: items.iter().map(MenuExitRow::from).collect(),
                presentation: presentation.clone(),
            });
        }
        bounds.motion.set(motion);
        let forward = if direction == Direction::Rtl {
            "left"
        } else {
            "right"
        };
        let exit_view = motion
            .filter(|_| !open)
            .map(|motion| build_exit_branch(&self.id, &bounds, motion, direction, window, cx));
        let menu_id = self.id.clone();
        let menu_element: AnyElement = if is_context {
            let content_focus = {
                let mut slot = bounds.context_focus.borrow_mut();
                slot.get_or_insert_with(|| cx.focus_handle()).clone()
            };
            let was_open = bounds.context_was_open.replace(open);
            if open && !was_open {
                *bounds.context_previous_focus.borrow_mut() = window.focused(cx);
                content_focus.focus(window, cx);
            } else if !open && was_open {
                let previous = bounds.context_previous_focus.borrow_mut().take();
                if content_focus.contains_focused(window, cx) {
                    match previous {
                        Some(previous) => previous.focus(window, cx),
                        None => window.blur(cx),
                    }
                }
            }

            let content = self
                .composed_trigger
                .map_or(self.trigger, |build| build(open, trigger));
            let trigger_el = div()
                .id("menu-trigger")
                .role(Role::Group)
                .aria_expanded(open)
                .on_mouse_down(MouseButton::Right, move |event, window, cx| {
                    cx.stop_propagation();
                    context_mouse_bounds
                        .context_position
                        .set(Some(event.position));
                    if context_open_state.get() {
                        cx.notify(view);
                    } else {
                        context_open(window, cx);
                    }
                })
                .on_key_down(move |event, window, cx| {
                    if is_context_menu_key(&event.keystroke) {
                        context_key_bounds.context_position.set(None);
                        context_key_open(window, cx);
                        cx.stop_propagation();
                    }
                })
                .child(content);

            let popup = Popup::new(self.id, trigger_el)
                .anchor(anchor)
                .offset(self.offset)
                .on_position(move |resolved, trigger| {
                    position.trigger.set(trigger);
                    if explicit_position.is_none() {
                        position.popup.set(resolved.bounds);
                    }
                    if let Some(ancestor) = &ancestor {
                        *ancestor.child.borrow_mut() = Some(position.clone());
                    }
                });

            if open {
                let motion = motion.expect("open menu has motion");
                let close_level: Action =
                    Rc::new(move |window, cx| popup_request(false, window, cx));
                let close: Action = {
                    let close_level = close_level.clone();
                    Rc::new(move |window, cx| {
                        close_level(window, cx);
                        if let Some(close_parent) = &self.close_parent {
                            close_parent(window, cx);
                        }
                    })
                };
                let dismissal = close_level.clone();
                let escape = close_level.clone();
                let item_bounds = bounds.clone();
                let items = resolved_items.unwrap_or_default();
                let body = window.with_id(menu_id, |window| {
                    menu_content(
                        items,
                        close,
                        submenu.then_some(close_level),
                        content_focus.clone(),
                        (direction, bounds.clone(), presentation, motion),
                        window,
                        cx,
                    )
                });
                let dismiss_shell = div()
                    .id("dismiss-menu")
                    .opacity(motion.alpha)
                    .on_key_down(move |event, window, cx| {
                        if event.keystroke.key == "escape" && !event.keystroke.modifiers.modified()
                        {
                            escape(window, cx);
                            cx.stop_propagation();
                        }
                    })
                    .on_mouse_down_out(move |event, window, cx| {
                        let point = &event.position;
                        if bounds.popup_contains(point)
                            || (event.button == MouseButton::Right
                                && bounds.trigger.get().contains(point))
                        {
                            return;
                        }
                        dismissal(window, cx);
                    })
                    .child(body);
                let pivot = item_bounds.clone();
                let overlay = ScaledOverlay::new(dismiss_shell, motion, move |menu| {
                    let anchor_bounds = explicit_position.map_or_else(
                        || pivot.trigger.get(),
                        |pos| Bounds::new(pos, Size::default()),
                    );
                    menu_pivot(menu, anchor_bounds)
                });
                let overlay = ThemeScope::inherit(overlay, window, cx);
                let positioned: AnyElement = if let Some(position) = explicit_position {
                    Positioner::corner(anchor, position)
                        .on_position(move |resolved| item_bounds.popup.set(resolved.bounds))
                        .occlude()
                        .child(overlay)
                        .into_any_element()
                } else {
                    overlay
                };
                popup
                    .content(
                        div()
                            .id("content")
                            .role(Role::Dialog)
                            .occlude()
                            .tab_group()
                            .track_focus(&content_focus)
                            .child(positioned),
                    )
                    .into_any_element()
            } else {
                popup.into_any_element()
            }
        } else {
            Popover::new(self.id)
                .open(open)
                .on_open_change(move |requested, window, cx| {
                    popover_request(*requested, window, cx)
                })
                .anchor(anchor)
                .offset(self.offset)
                .mouse_button(self.mouse_button)
                .overlay_closable(false)
                .on_position(move |resolved, trigger| {
                    position.trigger.set(trigger);
                    if explicit_position.is_none() {
                        position.popup.set(resolved.bounds);
                    }
                    if let Some(ancestor) = &ancestor {
                        *ancestor.child.borrow_mut() = Some(position.clone());
                    }
                })
                .trigger_with(move |open, _, _| {
                    let content = self
                        .composed_trigger
                        .map_or(self.trigger, |build| build(open, trigger));
                    div()
                        .id("menu-trigger")
                        .role(Role::Group)
                        .aria_expanded(open)
                        .when(controlled, |trigger| {
                            // Accepting the request through Popover::open applies
                            // the Base focus lifecycle only after the owner agrees.
                            trigger
                                .on_action(move |_: &Confirm, window, cx| key_trigger(window, cx))
                                .on_mouse_down(self.mouse_button, move |_, window, cx| {
                                    cx.stop_propagation();
                                    mouse_trigger(window, cx);
                                })
                        })
                        .on_key_down(move |event, _, cx| {
                            if submenu && event.keystroke.key == forward && !key_state.get() {
                                key_state.set(true);
                                cx.notify(view);
                                cx.stop_propagation();
                            }
                        })
                        .child(content)
                        .into_any_element()
                })
                .content(move |state, window, cx| {
                    let motion = motion.expect("open menu has motion");
                    let close_level: Action = if controlled {
                        Rc::new(move |window, cx| popup_request(false, window, cx))
                    } else {
                        let own = cx.entity().downgrade();
                        Rc::new(move |window, cx| {
                            let _ = own.update(cx, |state: &mut PopoverState, cx| {
                                state.dismiss(window, cx);
                            });
                        })
                    };
                    let close: Action = {
                        let close_level = close_level.clone();
                        Rc::new(move |window, cx| {
                            close_level(window, cx);
                            if let Some(close_parent) = &self.close_parent {
                                close_parent(window, cx);
                            }
                        })
                    };
                    let dismissal = close_level.clone();
                    let escape = close_level.clone();
                    let item_bounds = bounds.clone();
                    let items = resolved_items.unwrap_or_default();
                    let content = window.with_id(menu_id, |window| {
                        menu_content(
                            items,
                            close,
                            submenu.then_some(close_level),
                            state.focus_handle(cx),
                            (direction, bounds.clone(), presentation, motion),
                            window,
                            cx,
                        )
                    });
                    let popup = div()
                        .id("dismiss-menu")
                        .opacity(motion.alpha)
                        .when(controlled, |popup| {
                            popup.on_action(move |_: &Cancel, window, cx| escape(window, cx))
                        })
                        .on_mouse_down_out(move |event, window, cx| {
                            let point = &event.position;
                            if bounds.contains(point) {
                                return;
                            }
                            dismissal(window, cx);
                        })
                        .child(content);
                    let pivot = item_bounds.clone();
                    let overlay = ScaledOverlay::new(popup, motion, move |menu| {
                        let anchor_bounds = explicit_position.map_or_else(
                            || pivot.trigger.get(),
                            |pos| Bounds::new(pos, Size::default()),
                        );
                        menu_pivot(menu, anchor_bounds)
                    });
                    let overlay = ThemeScope::inherit(overlay, window, cx);
                    if let Some(position) = explicit_position {
                        Positioner::corner(anchor, position)
                            .on_position(move |resolved| item_bounds.popup.set(resolved.bounds))
                            .occlude()
                            .child(overlay)
                            .into_any_element()
                    } else {
                        overlay
                    }
                })
                .into_any_element()
        };
        div()
            .relative()
            .child(menu_element)
            .children(exit_view)
            .into_any_element()
    }
}

fn is_context_menu_key(keystroke: &gpui::Keystroke) -> bool {
    let mods = &keystroke.modifiers;
    if keystroke.key == "contextmenu" && !mods.modified() {
        return true;
    }
    keystroke.key == "f10"
        && mods.shift
        && !mods.control
        && !mods.alt
        && !mods.platform
        && !mods.function
}
