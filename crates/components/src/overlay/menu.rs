use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, ElementId, IntoElement, MouseButton, ParentElement, Pixels, Point,
    RenderOnce, ScrollHandle, Styled, Window, div, px,
};
use gpui_material_foundation::{OverlayMotion, current_material_theme, logical_anchor};

pub use crate::menu_appearance::{MenuColor, MenuItemStyle, MenuSurfaceStyle};

mod content;
mod exit;
mod item;
mod popup;
mod state;

pub use item::MenuItem;

use exit::build_exit_branch;
use item::{Action, Change, ComposedTrigger, ItemContent, MenuItems};
use popup::{PopupHostConfig, render_popup_host};
use state::{MenuPresentation, PopupBounds};

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
    on_dismiss_request: Option<Action>,
    color: MenuColor,
    appearance: MenuSurfaceStyle,
    decoration: Option<ItemContent>,
    scroll: Option<ScrollHandle>,
    offset: Pixels,
    position: Option<Point<Pixels>>,
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
            on_dismiss_request: None,
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
    pub fn on_dismiss_request(
        mut self,
        callback: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss_request = Some(Rc::new(callback));
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.enabled {
            return match self.composed_trigger {
                Some(build) => build(false, Rc::new(|_, _| {})),
                None => self.trigger,
            };
        }
        let direction = current_material_theme(window, cx).layout.direction;
        let submenu = self.close_parent.is_some();
        let is_context = self.mouse_button == MouseButton::Right;
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
        let sibling_close: Action = {
            let on_open_change = self.on_open_change.clone();
            Rc::new(move |window, cx| {
                if !controlled {
                    cx.notify(view);
                } else if let Some(callback) = &on_open_change {
                    callback(false, window, cx);
                }
            })
        };
        *bounds.close_handler.borrow_mut() = Some(sibling_close);
        let request: Change = {
            let on_open_change = self.on_open_change;
            let on_dismiss_request = self.on_dismiss_request;
            let ancestor = self.ancestor.clone();
            let bounds = bounds.clone();
            Rc::new(move |requested, window, cx| {
                if requested && let Some(ancestor) = &ancestor {
                    ancestor.activate_child(&bounds, window, cx);
                }
                if !controlled {
                    open_state.set(requested);
                    cx.notify(view);
                }
                if let Some(callback) = &on_open_change {
                    callback(requested, window, cx);
                }
                if !requested && let Some(callback) = &on_dismiss_request {
                    callback(window, cx);
                }
            })
        };
        let motion = OverlayMotion::sample(&self.id, open, window, cx);
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
            bounds.sync_presented(items, &presentation);
        }
        bounds.motion.set(motion);
        let exit_view = motion
            .filter(|_| !open)
            .map(|motion| build_exit_branch(&self.id, &bounds, motion, direction, window, cx));
        let menu_element = render_popup_host(
            PopupHostConfig {
                id: self.id,
                trigger: self.trigger,
                composed_trigger: self.composed_trigger,
                items: resolved_items,
                mouse_button: self.mouse_button,
                anchor,
                offset: self.offset,
                explicit_position,
                open,
                controlled,
                submenu,
                direction,
                motion,
                presentation,
                bounds,
                ancestor: self.ancestor,
                close_parent: self.close_parent,
                request,
                view,
            },
            window,
            cx,
        );
        div()
            .relative()
            .child(menu_element)
            .children(exit_view)
            .into_any_element()
    }
}
