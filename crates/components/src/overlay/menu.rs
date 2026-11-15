use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::motion::{OverlayMotion, ScaledOverlay, menu_pivot};
use crate::{Icon, IconName};
use gpui::{
    Anchor, AnyElement, App, Axis, Bounds, ElementId, FocusHandle, Focusable, InteractiveElement,
    IntoElement, MouseButton, ParentElement, RenderOnce, Role, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{
    Popover, PopoverState, TestSupportExt,
    actions::{Cancel, Confirm},
};
use gpui_material_foundation::{
    Surface, color, logical_anchor, move_roving_focus, window_material_theme,
};
use gpui_material_theme::{ColorRole, Direction, Elevation};

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
}

enum Kind {
    Action(Option<Action>),
    Check(bool, Option<Action>),
    Select(bool, Option<Action>),
    Group,
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
    group: bool,
    selected: bool,
    submenu: bool,
    slots: ItemSlots,
}

impl From<&MenuItem> for MenuExitRow {
    fn from(item: &MenuItem) -> Self {
        Self {
            label: item.label.clone(),
            disabled: item.disabled,
            group: matches!(item.kind, Kind::Group),
            selected: matches!(item.kind, Kind::Check(true, _) | Kind::Select(true, _)),
            submenu: matches!(item.kind, Kind::Submenu(_)),
            slots: item.slots.clone(),
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
        }
    }

    pub fn group(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            kind: Kind::Group,
            ..Self::new(id, title)
        }
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
}

#[derive(Default)]
struct PopupBounds {
    trigger: Cell<Bounds<gpui::Pixels>>,
    popup: Cell<Bounds<gpui::Pixels>>,
    child: RefCell<Option<Rc<PopupBounds>>>,
    items: RefCell<Vec<MenuExitRow>>,
    scroll: ScrollHandle,
    motion: Cell<Option<OverlayMotion>>,
    open: Rc<Cell<bool>>,
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
            Rc::new(move |window, cx| request(!open, window, cx))
        };
        let key_trigger = trigger.clone();
        let mouse_trigger = trigger.clone();
        let popover_request = request.clone();
        let popup_request = request;
        let motion = OverlayMotion::sample(&self.id, open, window, cx);
        let position = bounds.clone();
        let ancestor = self.ancestor.clone();
        if open {
            let mut rows = bounds.items.borrow_mut();
            rows.clear();
            rows.extend(self.items.iter().map(MenuExitRow::from));
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
            bounds.items.borrow_mut().clear();
            bounds.child.borrow_mut().take();
        }
        let menu = Popover::new(self.id)
            .open(open)
            .on_open_change(move |open, window, cx| popover_request(*open, window, cx))
            .anchor(logical_anchor(self.anchor, direction))
            .offset(px(4.))
            .mouse_button(self.mouse_button)
            .overlay_closable(false)
            .on_position(move |resolved, trigger| {
                position.trigger.set(trigger);
                position.popup.set(resolved.bounds);
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
                let content = menu_content(
                    self.items,
                    close,
                    submenu.then_some(close_level),
                    state.focus_handle(cx),
                    (direction, bounds.clone(), bounds.scroll.clone(), motion),
                    window,
                    cx,
                );
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
                ScaledOverlay::new(popup, motion, move |menu| {
                    menu_pivot(menu, item_bounds.trigger.get())
                })
                .into_any_element()
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
    let trigger = bounds.trigger.get();
    let theme = window_material_theme(window, cx);
    let exit_ink = color(theme.colors.color(ColorRole::OnSurface));
    let exit_muted = color(theme.colors.color(ColorRole::OnSurfaceVariant));
    let exit_selected = color(theme.colors.color(ColorRole::SecondaryContainer));
    let exit_selected_ink = color(theme.colors.color(ColorRole::OnSecondaryContainer));
    let dp = |value| motion.px(value);
    let spacing = motion.px(16.);
    let items = bounds.items.borrow();
    let rows = items
        .iter()
        .map(|item| {
            let label = item.label.clone();
            let tint = if item.group {
                exit_muted
            } else if item.disabled {
                exit_ink.opacity(0.38)
            } else if item.selected {
                exit_selected_ink
            } else {
                exit_ink
            };
            div().px(spacing * 0.75).text_color(tint).when_else(
                item.group,
                |row| row.py(spacing * 0.5).child(item.slots.label(label.clone())),
                |row| {
                    row.min_h(dp(48.))
                        .flex()
                        .items_center()
                        .line_height(dp(20.))
                        .gap(spacing * 0.5)
                        .justify_start()
                        .when(direction == Direction::Rtl, Styled::flex_row_reverse)
                        .when(item.selected, |row| row.bg(exit_selected))
                        .children(item.slots.children(
                            label.clone(),
                            item.selected,
                            item.submenu,
                            motion,
                        ))
                },
            )
        })
        .collect::<Vec<_>>();
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
        .track_scroll(&bounds.scroll)
        .child(
            Surface::new(ColorRole::SurfaceContainer)
                .shadow_elevation(Elevation::Level2)
                .shadow_scale(motion.scale)
                .w_full()
                .rounded(spacing * 0.25)
                .py(spacing * 0.5)
                .children(rows),
        )
        .child(div().absolute().inset_0().occlude());
    Some(
        gpui::deferred(
            gpui_base::Positioner::corner(gpui::Anchor::TopLeft, exit.origin).child(
                ScaledOverlay::new(
                    super::text_style(
                        surface,
                        gpui_material_theme::TypeRole::LabelLarge,
                        motion.scale,
                        window,
                        cx,
                    ),
                    motion,
                    move |bounds| menu_pivot(bounds, trigger),
                ),
            ),
        )
        .with_priority(100 + depth)
        .into_any_element(),
    )
}

fn menu_content(
    items: Vec<MenuItem>,
    close: Action,
    close_level: Option<Action>,
    popup_focus: FocusHandle,
    (direction, bounds, scroll, motion): (Direction, Rc<PopupBounds>, ScrollHandle, OverlayMotion),
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let theme = window_material_theme(window, cx);
    let ink = color(theme.colors.color(ColorRole::OnSurface));
    let muted = color(theme.colors.color(ColorRole::OnSurfaceVariant));
    let selected_color = color(theme.colors.color(ColorRole::SecondaryContainer));
    let selected_ink = color(theme.colors.color(ColorRole::OnSecondaryContainer));
    let container = color(theme.colors.color(ColorRole::SurfaceContainer));
    let focus = items
        .iter()
        .map(|item| {
            let handle =
                window.use_keyed_state((item.id.clone(), "focus"), cx, |_, cx| cx.focus_handle());
            (
                handle.read(cx).clone(),
                !item.disabled && !matches!(item.kind, Kind::Group),
            )
        })
        .collect::<Vec<_>>();
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
    let dp = |value| motion.px(value);
    let spacing = motion.px(16.);
    let menu = div()
        .id(if close_level.is_some() {
            "submenu-content"
        } else {
            "menu"
        })
        .test_support()
        .role(Role::Menu)
        .overflow_y_scroll()
        .track_scroll(&scroll)
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
            Surface::new(ColorRole::SurfaceContainer)
                .shadow_elevation(Elevation::Level2)
                .shadow_scale(motion.scale)
                .rounded(spacing * 0.25)
                .min_w(dp(112.).min(available_width))
                .max_w(dp(280.).min(available_width))
                .py(spacing * 0.5)
                .children(
                    items
                        .into_iter()
                        .zip(focus)
                        .map(|(item, (focus, enabled))| {
                            if matches!(item.kind, Kind::Group) {
                                return div()
                                    .id(item.id)
                                    .role(Role::Label)
                                    .px(spacing * 0.75)
                                    .py(spacing * 0.5)
                                    .text_color(muted)
                                    .child(item.slots.label(item.label))
                                    .into_any_element();
                            }
                            let action = item.activation();
                            let (role, marked) = match item.kind {
                                Kind::Check(checked, _) => (Role::MenuItemCheckBox, checked),
                                Kind::Select(selected, _) => (Role::MenuItem, selected),
                                _ => (Role::MenuItem, false),
                            };
                            let submenu = matches!(&item.kind, Kind::Submenu(_));
                            let row = gpui_base::Button::new(item.id.clone())
                                .role(role)
                                .track_focus(&focus)
                                .disabled(!enabled)
                                .accessibility_label(item.label.clone())
                                .flex()
                                .justify_start()
                                .gap(spacing * 0.5)
                                .w_full()
                                .min_h(dp(48.))
                                .line_height(dp(20.))
                                .px(spacing * 0.75)
                                .text_color(if !enabled {
                                    ink.opacity(0.38)
                                } else if marked {
                                    selected_ink
                                } else {
                                    ink
                                })
                                .when(marked, |row| row.bg(selected_color))
                                .when(direction == Direction::Rtl, Styled::flex_row_reverse)
                                .when(enabled, |row| {
                                    let background =
                                        if marked { selected_color } else { container };
                                    let foreground = if marked { selected_ink } else { ink };
                                    row.hover(move |style| {
                                        style.bg(background.blend(foreground.opacity(
                                            gpui_material_theme::StateLayer::Hover.opacity(),
                                        )))
                                    })
                                    .focus_visible(move |style| {
                                        style.bg(background.blend(foreground.opacity(
                                            gpui_material_theme::StateLayer::Focus.opacity(),
                                        )))
                                    })
                                    .active(move |style| {
                                        style.bg(background.blend(foreground.opacity(
                                            gpui_material_theme::StateLayer::Pressed.opacity(),
                                        )))
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
                                    let activate: Action = {
                                        let close = close.clone();
                                        Rc::new(move |window, cx| {
                                            close(window, cx);
                                            if let Some(action) = &action {
                                                action(window, cx);
                                            }
                                        })
                                    };
                                    let keyboard = activate.clone();
                                    let accessibility = activate.clone();
                                    row.capture_key_down(move |event, window, cx| {
                                        if matches!(event.keystroke.key.as_str(), "enter" | "space")
                                        {
                                            keyboard(window, cx);
                                            cx.stop_propagation();
                                        }
                                    })
                                    .on_a11y_action(
                                        gpui::AccessibleAction::Click,
                                        move |_, window, cx| accessibility(window, cx),
                                    )
                                    .on_click(move |_, window, cx| activate(window, cx))
                                })
                                .children(item.slots.children(item.label, marked, submenu, motion));
                            if let Kind::Submenu(children) = item.kind {
                                Menu {
                                    close_parent: Some(close.clone()),
                                    enabled,
                                    ancestor: Some(bounds.clone()),
                                    ..Menu::new((item.id, "submenu"), row, children)
                                        .anchor(Anchor::LeftCenter)
                                }
                                .into_any_element()
                            } else {
                                row.into_any_element()
                            }
                        }),
                ),
        );
    super::text_style(
        menu,
        gpui_material_theme::TypeRole::LabelLarge,
        motion.scale,
        window,
        cx,
    )
    .into_any_element()
}
