use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::motion::{OverlayMotion, ScaledOverlay, menu_pivot};
use gpui::{
    Anchor, AnyElement, App, Axis, Bounds, ElementId, FocusHandle, Focusable, InteractiveElement,
    IntoElement, MouseButton, ParentElement, RenderOnce, Role, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Popover, PopoverState, TestSupportExt};
use gpui_material_foundation::{
    Surface, color, logical_anchor, move_roving_focus, window_material_theme,
};
use gpui_material_theme::{ColorRole, Direction, Elevation};

type Action = Rc<dyn Fn(&mut Window, &mut App)>;

/// An item in a Material menu. Callers retain command and selection state.
pub struct MenuItem {
    id: ElementId,
    label: SharedString,
    disabled: bool,
    kind: Kind,
}

enum Kind {
    Action(Option<Action>),
    Check(bool, Option<Action>),
    Select(bool, Option<Action>),
    Group,
    Submenu(Vec<MenuItem>),
}

struct MenuExitRow {
    label: SharedString,
    disabled: bool,
    group: bool,
    selected: bool,
    submenu: bool,
}

impl From<&MenuItem> for MenuExitRow {
    fn from(item: &MenuItem) -> Self {
        Self {
            label: item.label.clone(),
            disabled: item.disabled,
            group: matches!(item.kind, Kind::Group),
            selected: matches!(item.kind, Kind::Check(true, _) | Kind::Select(true, _)),
            submenu: matches!(item.kind, Kind::Submenu(_)),
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

    fn action(&self) -> Option<Action> {
        match &self.kind {
            Kind::Action(action) | Kind::Check(_, action) | Kind::Select(_, action) => {
                action.clone()
            }
            _ => None,
        }
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
        let open = open_state.get();
        let key_state = open_state.clone();
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
            .on_open_change(move |open, _, cx| {
                open_state.set(*open);
                cx.notify(view);
            })
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
                div()
                    .id("menu-trigger")
                    .role(Role::Group)
                    .aria_expanded(open)
                    .on_key_down(move |event, _, cx| {
                        if submenu && event.keystroke.key == forward && !key_state.get() {
                            key_state.set(true);
                            cx.notify(view);
                            cx.stop_propagation();
                        }
                    })
                    .child(self.trigger)
                    .into_any_element()
            })
            .content(move |state, window, cx| {
                let motion = motion.expect("open menu has motion");
                let own = cx.entity().downgrade();
                let close_level = own.clone();
                let close_level: Action = Rc::new(move |window, cx| {
                    let _ = close_level.update(cx, |state: &mut PopoverState, cx| {
                        state.dismiss(window, cx);
                    });
                });
                let close: Action = Rc::new(move |window, cx| {
                    let _ =
                        own.update(cx, |state: &mut PopoverState, cx| state.dismiss(window, cx));
                    if let Some(close_parent) = &self.close_parent {
                        close_parent(window, cx);
                    }
                });
                let dismissal = close_level.clone();
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
    let dp = |value| motion.px(value);
    let spacing = window.rem_size() * motion.scale;
    let items = bounds.items.borrow();
    let rows = items
        .iter()
        .map(|item| {
            let label = item.label.clone();
            let tint = if item.group {
                exit_muted
            } else if item.disabled {
                exit_ink.opacity(0.38)
            } else {
                exit_ink
            };
            div().px(spacing * 0.75).text_color(tint).when_else(
                item.group,
                |row| row.py(spacing * 0.5).child(label.clone()),
                |row| {
                    row.min_h(dp(40.))
                        .flex()
                        .items_center()
                        .line_height(gpui::relative(1.))
                        .gap(spacing * 0.5)
                        .justify_start()
                        .when(direction == Direction::Rtl, Styled::flex_row_reverse)
                        .when(item.selected, |row| row.bg(exit_selected))
                        .child(div().w(dp(18.)).flex_none().child(if item.selected {
                            "\u{2713}"
                        } else {
                            ""
                        }))
                        .child(div().flex_1().min_w_0().truncate().child(label.clone()))
                        .when(item.submenu, |row| {
                            row.child(if direction == Direction::Rtl {
                                "\u{2039}"
                            } else {
                                "\u{203a}"
                            })
                        })
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
                ScaledOverlay::new(motion.text(surface, window), motion, move |bounds| {
                    menu_pivot(bounds, trigger)
                }),
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
    let spacing = window.rem_size() * motion.scale;
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
                                    .child(item.label)
                                    .into_any_element();
                            }
                            let action = item.action();
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
                                .min_h(dp(40.))
                                .px(spacing * 0.75)
                                .text_color(if enabled { ink } else { ink.opacity(0.38) })
                                .when(marked, |row| row.bg(selected_color))
                                .when(direction == Direction::Rtl, Styled::flex_row_reverse)
                                .when(enabled, |row| {
                                    row.hover(move |style| style.bg(ink.opacity(0.08)))
                                        .focus_visible(move |style| style.bg(ink.opacity(0.12)))
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
                                .child(div().w(dp(18.)).flex_none().child(if marked {
                                    "✓"
                                } else {
                                    ""
                                }))
                                .child(div().flex_1().min_w_0().truncate().child(item.label))
                                .when(submenu, |row| {
                                    row.child(if direction == Direction::Rtl {
                                        "‹"
                                    } else {
                                        "›"
                                    })
                                });
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
    motion.text(menu, window).into_any_element()
}
