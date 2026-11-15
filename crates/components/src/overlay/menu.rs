use std::cell::{Cell, RefCell};
use std::rc::Rc;

mod appearance;
pub use appearance::{MenuColor, MenuItemStyle, MenuSurfaceStyle};
use appearance::{Paint, Palette, axis_padding};

use crate::motion::{OverlayMotion, ScaledOverlay, menu_pivot};
use crate::{Icon, IconName};
use gpui::{
    Anchor, AnyElement, App, Axis, Bounds, ElementId, FocusHandle, Focusable, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Pixels, Point, RenderOnce, Role, ScrollHandle,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{
    Popover, PopoverState, Positioner, TestSupportExt,
    actions::{Cancel, Confirm},
};
use gpui_material_foundation::{logical_anchor, move_roving_focus, window_material_theme};
use gpui_material_theme::{CornerRole, CornerShape, Direction, StateLayer, TypeRole};

type Action = Rc<dyn Fn(&mut Window, &mut App)>;
type Change = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type ItemContent = Rc<dyn Fn() -> AnyElement>;
type ComposedTrigger = Box<dyn FnOnce(bool, Action) -> AnyElement>;

/// An item in a Material menu. Callers retain command and selection state.
pub struct MenuItem {
    id: ElementId,
    label: SharedString,
    disabled: bool,
    kind: Kind,
    slots: ItemSlots,
    on_checked_change: Option<Change>,
    style: MenuItemStyle,
}

enum Kind {
    Action(Option<Action>),
    Check(bool, Option<Action>),
    Select(bool, Option<Action>),
    Heading,
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
        motion: OverlayMotion,
        supporting_color: gpui::Hsla,
    ) -> Vec<AnyElement> {
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
        let body = div()
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .child(div().truncate().child(self.label(label)))
            .children(self.supporting.as_ref().map(|supporting| {
                div()
                    .text_size(motion.px(12.))
                    .line_height(motion.px(16.))
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
        children.extend(self.trailing.as_ref().map(|trailing| trailing()));
        children.extend(submenu.then(|| {
            Icon::new(IconName::ChevronRight)
                .size(icon_size)
                .mirror_in_rtl(true)
                .into_any_element()
        }));
        children
    }
}

struct MenuExitRow {
    label: SharedString,
    disabled: bool,
    heading: bool,
    section: Option<(MenuSurfaceStyle, Vec<MenuExitRow>)>,
    selected: bool,
    submenu: bool,
    slots: ItemSlots,
    style: MenuItemStyle,
}

impl From<&MenuItem> for MenuExitRow {
    fn from(item: &MenuItem) -> Self {
        Self {
            label: item.label.clone(),
            disabled: item.disabled,
            heading: matches!(item.kind, Kind::Heading),
            section: match &item.kind {
                Kind::Section { rows, style, .. } => {
                    Some((*style, rows.iter().map(Self::from).collect()))
                }
                _ => None,
            },
            selected: matches!(item.kind, Kind::Check(true, _) | Kind::Select(true, _)),
            submenu: matches!(item.kind, Kind::Submenu(_)),
            slots: item.slots.clone(),
            style: item.style,
        }
    }
}

impl MenuItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            kind: Kind::Action(None),
            slots: ItemSlots::default(),
            on_checked_change: None,
            style: MenuItemStyle::default(),
        }
    }

    pub fn group(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            kind: Kind::Heading,
            ..Self::new(id, title)
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

    pub fn submenu(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.kind = Kind::Submenu(items.into_iter().collect());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.kind = Kind::Check(checked, self.action());
        self
    }

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
    items: Vec<MenuItem>,
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
    trigger: Cell<Bounds<gpui::Pixels>>,
    popup: Cell<Bounds<gpui::Pixels>>,
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
    items: Vec<MenuExitRow>,
    presentation: MenuPresentation,
}

impl PopupBounds {
    fn contains(&self, point: &gpui::Point<gpui::Pixels>) -> bool {
        self.trigger.get().contains(point)
            || self.popup.get().contains(point)
            || self
                .child
                .borrow()
                .as_ref()
                .is_some_and(|child| child.open.get() && child.contains(point))
    }
}

impl Menu {
    pub fn new(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        items: impl IntoIterator<Item = MenuItem>,
    ) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            items: items.into_iter().collect(),
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

    /// Opens on secondary click; Enter and Space still work on the trigger.
    pub fn context(mut self) -> Self {
        self.mouse_button = MouseButton::Right;
        self
    }

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

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let direction = window_material_theme(window, cx).layout.direction;
        let submenu = self.close_parent.is_some();
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
        let popover_request = request.clone();
        let popup_request = request;
        let motion = OverlayMotion::sample(&self.id, open, window, cx);
        let position = bounds.clone();
        let ancestor = self.ancestor.clone();
        let anchor = logical_anchor(self.anchor, direction);
        let explicit_position = self.position;
        let presentation = MenuPresentation {
            color: self.color,
            appearance: self.appearance,
            decoration: self.decoration,
            scroll: self.scroll.unwrap_or_else(|| bounds.scroll.clone()),
        };
        if open {
            *bounds.presented.borrow_mut() = Some(PresentedMenu {
                items: self.items.iter().map(MenuExitRow::from).collect(),
                presentation: presentation.clone(),
            });
        }
        bounds.motion.set(motion);
        let forward = if direction == Direction::Rtl {
            "left"
        } else {
            "right"
        };
        let exit_view = motion.filter(|_| !open).map(|motion| {
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
                    (&self.id, depth),
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
        });
        if !open && motion.is_none() {
            bounds.presented.borrow_mut().take();
            bounds.child.borrow_mut().take();
        }
        let menu_id = self.id.clone();
        let menu = Popover::new(self.id)
            .open(open)
            .on_open_change(move |open, window, cx| popover_request(*open, window, cx))
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
                let content = window.with_id(menu_id, |window| {
                    menu_content(
                        self.items,
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
                    menu_pivot(menu, pivot.trigger.get())
                });
                if let Some(position) = explicit_position {
                    Positioner::corner(anchor, position)
                        .on_position(move |resolved| item_bounds.popup.set(resolved.bounds))
                        .occlude()
                        .child(overlay)
                        .into_any_element()
                } else {
                    overlay.into_any_element()
                }
            });
        div()
            .relative()
            .child(menu)
            .children(exit_view)
            .into_any_element()
    }
}

fn menu_exit(
    (id, depth): (&ElementId, usize),
    bounds: &PopupBounds,
    motion: OverlayMotion,
    direction: Direction,
    window: &Window,
    cx: &App,
) -> Option<AnyElement> {
    let exit = bounds.popup.get();
    if exit.size.width <= px(0.) {
        return None;
    }
    let presented = bounds.presented.borrow();
    let PresentedMenu {
        items,
        presentation,
    } = presented.as_ref()?;
    let trigger = bounds.trigger.get();
    let paint = Paint {
        theme: window_material_theme(window, cx),
        motion,
        direction,
    };
    let palette = presentation
        .color
        .palette(false, presentation.appearance.container);
    let surface = div()
        .id(if depth == 0 {
            ElementId::from((id.clone(), "menu-exit"))
        } else {
            ElementId::from((id.clone(), format!("submenu-exit-{depth}")))
        })
        .test_support()
        .h(exit.size.height * motion.scale)
        .w(exit.size.width * motion.scale)
        .opacity(motion.alpha)
        .overflow_y_scroll()
        .track_scroll(&presentation.scroll)
        .child(
            paint
                .surface(presentation.appearance, palette, false)
                .w_full()
                .children(presentation.decoration.as_ref().map(|build| build()))
                .children(exit_rows(items, paint, presentation.color, palette, false)),
        )
        .child(div().absolute().inset_0().occlude());
    Some(
        gpui::deferred(
            Positioner::corner(Anchor::TopLeft, exit.origin).child(ScaledOverlay::new(
                super::text_style(surface, TypeRole::LabelLarge, motion.scale, window, cx),
                motion,
                move |bounds| menu_pivot(bounds, trigger),
            )),
        )
        .with_priority(100 + depth)
        .into_any_element(),
    )
}

fn exit_rows(
    items: &[MenuExitRow],
    paint: Paint<'_>,
    preset: MenuColor,
    palette: Palette,
    grouped: bool,
) -> Vec<AnyElement> {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            if let Some((style, children)) = &item.section {
                let palette = preset.palette(true, style.container);
                return paint
                    .surface(*style, palette, true)
                    .child(menu_heading(
                        item.slots.label(item.label.clone()),
                        paint,
                        palette,
                    ))
                    .children(exit_rows(children, paint, preset, palette, true))
                    .into_any_element();
            }
            if item.heading {
                return menu_heading(item.slots.label(item.label.clone()), paint, palette)
                    .into_any_element();
            }
            let style = row_shape(item.style, grouped, index, items.len(), item.selected);
            let (background, ink, supporting) =
                row_colors(item.style, paint, palette, item.selected, !item.disabled);
            row_layout(div().flex().items_center(), item.style, style, paint)
                .text_color(ink)
                .when(item.selected || item.style.container.is_some(), |row| {
                    row.bg(background)
                })
                .children(item.slots.children(
                    item.label.clone(),
                    item.selected,
                    item.submenu,
                    paint.motion,
                    supporting,
                ))
                .into_any_element()
        })
        .collect()
}

fn menu_heading(label: AnyElement, paint: Paint<'_>, palette: Palette) -> gpui::Div {
    div()
        .px(paint.motion.px(12.))
        .py(paint.motion.px(8.))
        .text_color(paint.color(palette.muted))
        .child(label)
}

fn row_shape(
    style: MenuItemStyle,
    grouped: bool,
    index: usize,
    count: usize,
    selected: bool,
) -> CornerShape {
    if selected {
        if let Some(shape) = style.selected_shape {
            return shape;
        }
        if grouped {
            return CornerRole::Medium.shape();
        }
    }
    if let Some(shape) = style.shape {
        return shape;
    }
    if !grouped {
        return CornerRole::None.shape();
    }
    match (index, count) {
        (0, 2..) => CornerShape::Radii([12., 12., 4., 4.]),
        (index, 2..) if index + 1 == count => CornerShape::Radii([4., 4., 12., 12.]),
        _ => CornerRole::ExtraSmall.shape(),
    }
}

fn row_layout<E: Styled + FluentBuilder>(
    element: E,
    style: MenuItemStyle,
    shape: CornerShape,
    paint: Paint<'_>,
) -> E {
    let element = element
        .justify_start()
        .w_full()
        .min_h(paint.motion.px(48.))
        .line_height(paint.motion.px(20.))
        .gap(paint.length(style.spacing.unwrap_or(px(8.))))
        .when(paint.direction == Direction::Rtl, Styled::flex_row_reverse);
    let element = paint.padding(
        element,
        style
            .padding
            .unwrap_or_else(|| axis_padding(px(12.), px(0.))),
    );
    paint.shape(element, shape)
}

fn row_colors(
    style: MenuItemStyle,
    paint: Paint<'_>,
    palette: Palette,
    marked: bool,
    enabled: bool,
) -> (gpui::Hsla, gpui::Hsla, gpui::Hsla) {
    let (background, foreground, muted) = if marked {
        let ink = style.selected_content.unwrap_or(palette.selected_ink);
        (
            style.selected_container.unwrap_or(palette.selected),
            ink,
            ink,
        )
    } else {
        (
            style.container.unwrap_or(palette.container),
            style.content.unwrap_or(palette.ink),
            style.supporting.unwrap_or(palette.muted),
        )
    };
    let factor = if enabled { 1. } else { 0.38 };
    (
        paint.color(background),
        paint.color(foreground).opacity(factor),
        paint.color(muted).opacity(factor),
    )
}

fn menu_content(
    items: Vec<MenuItem>,
    close: Action,
    close_level: Option<Action>,
    popup_focus: FocusHandle,
    (direction, bounds, presentation, motion): (
        Direction,
        Rc<PopupBounds>,
        MenuPresentation,
        OverlayMotion,
    ),
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    fn collect_focus(
        items: &[MenuItem],
        focus: &mut Vec<(FocusHandle, bool)>,
        window: &mut Window,
        cx: &mut App,
    ) {
        for item in items {
            match &item.kind {
                Kind::Heading => {}
                Kind::Section { rows, .. } => collect_focus(rows, focus, window, cx),
                _ => {
                    let handle = window
                        .use_keyed_state((item.id.clone(), "focus"), cx, |_, cx| cx.focus_handle());
                    focus.push((handle.read(cx).clone(), !item.disabled));
                }
            }
        }
    }
    let mut focus = Vec::new();
    collect_focus(&items, &mut focus, window, cx);
    let handles = focus.clone();
    if popup_focus.is_focused(window)
        && let Some((first, _)) = handles.iter().find(|(_, enabled)| *enabled)
    {
        let first = first.clone();
        window.defer(cx, move |window, cx| {
            if popup_focus.is_focused(window) {
                first.focus(window, cx);
            }
        });
    }
    let viewport = window.viewport_size();
    let limit = (viewport.height * 0.7).min(px(420.)) * motion.scale;
    let available_width = (viewport.width - px(16.)).max(px(0.)) * motion.scale;
    let theme = window_material_theme(window, cx).clone();
    let paint = Paint {
        theme: &theme,
        motion,
        direction,
    };
    let palette = presentation
        .color
        .palette(false, presentation.appearance.container);
    let mut rows = ActiveRows {
        paint,
        focus: focus.into_iter(),
        close,
        bounds,
        preset: presentation.color,
    };
    let menu = div()
        .id(if close_level.is_some() {
            "submenu-content"
        } else {
            "menu"
        })
        .test_support()
        .role(Role::Menu)
        .overflow_y_scroll()
        .track_scroll(&presentation.scroll)
        .max_h(limit)
        .on_key_down(move |event, window, cx| {
            let backward = if direction == Direction::Rtl {
                "right"
            } else {
                "left"
            };
            if event.keystroke.key == backward
                && let Some(close_level) = &close_level
            {
                close_level(window, cx);
                cx.stop_propagation();
                return;
            }
            let items: Vec<_> = handles
                .iter()
                .map(|(focus, enabled)| (focus, *enabled))
                .collect();
            if move_roving_focus(
                &items,
                &event.keystroke.key,
                Axis::Vertical,
                direction,
                window,
                cx,
            )
            .is_some()
            {
                cx.stop_propagation();
            }
        })
        .child(
            paint
                .surface(presentation.appearance, palette, false)
                .min_w(motion.px(112.).min(available_width))
                .max_w(motion.px(280.).min(available_width))
                .children(presentation.decoration.as_ref().map(|build| build()))
                .children(rows.render(items, palette, false, window, cx)),
        );
    super::text_style(menu, TypeRole::LabelLarge, motion.scale, window, cx).into_any_element()
}

struct ActiveRows<'a> {
    paint: Paint<'a>,
    focus: std::vec::IntoIter<(FocusHandle, bool)>,
    close: Action,
    bounds: Rc<PopupBounds>,
    preset: MenuColor,
}

impl ActiveRows<'_> {
    fn render(
        &mut self,
        items: Vec<MenuItem>,
        palette: Palette,
        grouped: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> Vec<AnyElement> {
        let count = items.len();
        items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                if let Kind::Section {
                    rows,
                    mut style,
                    inactive_shape,
                } = item.kind
                {
                    let palette = self.preset.palette(true, style.container);
                    let state = inactive_shape.map(|shape| {
                        let state = window.use_keyed_state(
                            (item.id.clone(), "section-hover"),
                            cx,
                            |_, _| Cell::new(false),
                        );
                        if state.read(cx).get() {
                            style.shape = Some(shape);
                        }
                        state
                    });
                    let children = self.render(rows, palette, true, window, cx);
                    return self
                        .paint
                        .surface(style, palette, true)
                        .id(item.id)
                        .test_support()
                        .role(Role::Group)
                        .aria_label(item.label.clone())
                        .when_some(state, |section, state| {
                            section.on_hover(move |hovered, window, cx| {
                                let state = state.read(cx);
                                if state.replace(!hovered) != !hovered {
                                    window.refresh();
                                }
                            })
                        })
                        .child(menu_heading(
                            item.slots.label(item.label),
                            self.paint,
                            palette,
                        ))
                        .children(children)
                        .into_any_element();
                }
                if matches!(item.kind, Kind::Heading) {
                    return menu_heading(item.slots.label(item.label), self.paint, palette)
                        .id(item.id)
                        .role(Role::Label)
                        .into_any_element();
                }
                let shape = row_shape(
                    item.style,
                    grouped,
                    index,
                    count,
                    matches!(item.kind, Kind::Check(true, _) | Kind::Select(true, _)),
                );
                self.row(item, palette, shape)
            })
            .collect()
    }

    fn row(&mut self, item: MenuItem, palette: Palette, shape: CornerShape) -> AnyElement {
        let (focus, enabled) = self
            .focus
            .next()
            .expect("each menu command has a focus entry");
        let action = item.activation();
        let (role, marked) = match item.kind {
            Kind::Check(checked, _) => (Role::MenuItemCheckBox, checked),
            Kind::Select(selected, _) => (Role::MenuItem, selected),
            _ => (Role::MenuItem, false),
        };
        let submenu = matches!(item.kind, Kind::Submenu(_));
        let (background, ink, supporting) =
            row_colors(item.style, self.paint, palette, marked, enabled);
        let row = gpui_base::Button::new(item.id.clone())
            .role(role)
            .track_focus(&focus)
            .disabled(!enabled)
            .accessibility_label(item.label.clone())
            .flex()
            .text_color(ink);
        let row = row_layout(row, item.style, shape, self.paint)
            .when(marked || item.style.container.is_some(), |row| {
                row.bg(background)
            })
            .when(enabled, |row| {
                row.hover(move |style| {
                    style.bg(background.blend(ink.opacity(StateLayer::Hover.opacity())))
                })
                .focus_visible(move |style| {
                    style.bg(background.blend(ink.opacity(StateLayer::Focus.opacity())))
                })
                .active(move |style| {
                    style.bg(background.blend(ink.opacity(StateLayer::Pressed.opacity())))
                })
            })
            .when(role == Role::MenuItemCheckBox, |row| {
                row.aria_toggled(if marked {
                    gpui::accesskit::Toggled::True
                } else {
                    gpui::accesskit::Toggled::False
                })
            })
            .when(matches!(item.kind, Kind::Select(..)), |row| {
                row.aria_selected(marked)
            })
            .when(enabled && !submenu, |row| {
                let close = self.close.clone();
                let activate: Action = Rc::new(move |window, cx| {
                    close(window, cx);
                    if let Some(action) = &action {
                        action(window, cx);
                    }
                });
                let keyboard = activate.clone();
                let accessibility = activate.clone();
                row.capture_key_down(move |event, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        keyboard(window, cx);
                        cx.stop_propagation();
                    }
                })
                .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                    accessibility(window, cx)
                })
                .on_click(move |_, window, cx| activate(window, cx))
            })
            .children(item.slots.children(
                item.label,
                marked,
                submenu,
                self.paint.motion,
                supporting,
            ));
        if let Kind::Submenu(children) = item.kind {
            Menu {
                close_parent: Some(self.close.clone()),
                enabled,
                ancestor: Some(self.bounds.clone()),
                ..Menu::new((item.id, "submenu"), row, children)
                    .anchor(Anchor::LeftCenter)
                    .color(self.preset)
            }
            .into_any_element()
        } else {
            row.into_any_element()
        }
    }
}
