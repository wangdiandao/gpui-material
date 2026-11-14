use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, Axis, ElementId, FocusHandle, Focusable, InteractiveElement,
    IntoElement, MouseButton, ParentElement, RenderOnce, Role, SharedString,
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
        let open_state = window
            .use_keyed_state((self.id.clone(), "menu-open"), cx, |_, _| {
                Rc::new(Cell::new(false))
            })
            .read(cx)
            .clone();
        let open = open_state.get();
        let key_state = open_state.clone();
        let view = window.current_view();
        let forward = if direction == Direction::Rtl {
            "left"
        } else {
            "right"
        };
        Popover::new(self.id)
            .open(open)
            .on_open_change(move |open, _, cx| {
                open_state.set(*open);
                cx.notify(view);
            })
            .anchor(logical_anchor(self.anchor, direction))
            .offset(px(4.))
            .mouse_button(self.mouse_button)
            .overlay_closable(!submenu)
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
                menu_content(
                    self.items,
                    close,
                    submenu.then_some(close_level),
                    state.focus_handle(cx),
                    direction,
                    window,
                    cx,
                )
            })
            .into_any_element()
    }
}

fn menu_content(
    items: Vec<MenuItem>,
    close: Action,
    close_level: Option<Action>,
    popup_focus: FocusHandle,
    direction: Direction,
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
    let limit = (window.viewport_size().height * 0.7).min(px(420.));
    div()
        .id(if close_level.is_some() {
            "submenu-content"
        } else {
            "menu"
        })
        .test_support()
        .role(Role::Menu)
        .overflow_y_scroll()
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
                .rounded_sm()
                .min_w(px(180.).min(window.viewport_size().width))
                .py_2()
                .children(
                    items
                        .into_iter()
                        .zip(focus)
                        .map(|(item, (focus, enabled))| {
                            if matches!(item.kind, Kind::Group) {
                                return div()
                                    .id(item.id)
                                    .role(Role::Label)
                                    .px_3()
                                    .py_2()
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
                                .gap_2()
                                .w_full()
                                .min_h(px(40.))
                                .px_3()
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
                                .child(div().w(px(18.)).flex_none().child(if marked {
                                    "✓"
                                } else {
                                    ""
                                }))
                                .child(div().flex_1().min_w_0().child(item.label))
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
                                    ..Menu::new((item.id, "submenu"), row, children)
                                        .anchor(Anchor::LeftCenter)
                                }
                                .into_any_element()
                            } else {
                                row.into_any_element()
                            }
                        }),
                ),
        )
        .into_any_element()
}
